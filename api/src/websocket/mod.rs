mod directory;
mod messages;
mod users;

use crate::error::ServiceError;
use anyhow::{Result, anyhow};
use axum::extract::ws::{CloseFrame, Message as WsMessage, WebSocket, close_code};
use futures_util::{
	SinkExt, StreamExt,
	stream::{SplitSink, SplitStream},
};
use sea_orm::{DatabaseConnection, DbErr};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::{
	collections::HashMap,
	sync::{Arc, LazyLock},
};
use tokio::sync::{
	Mutex, broadcast,
	broadcast::{Receiver, Sender},
};
use tungstenite::{Error as TungsteniteError, error::CapacityError};

static MODULE_LIST: LazyLock<Vec<&'static dyn WsModule>> = LazyLock::new(|| {
	vec![
		&directory::DirectoryModule,
		&messages::MessagesModule,
		&users::UsersModule,
	]
});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsPayload(Value);

impl WsPayload {
	fn new<T: Serialize>(payload: T) -> Result<Self, serde_json::Error> {
		serde_json::to_value(payload).map(Self)
	}

	pub fn get<T: DeserializeOwned>(&self) -> Result<T, WsError> {
		// Deserializing a client-supplied payload is the one place a serde
		// error is the client's fault (like a Json extractor rejection on
		// the REST side); serialization errors elsewhere stay internal.
		serde_json::from_value(self.0.clone())
			.map_err(|err| WsError::Client(format!("Invalid payload: {err}")))
	}
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct WsEnvelope {
	module: String,
	#[serde(rename = "type")]
	r#type: String,
	payload: WsPayload,
}

impl WsEnvelope {
	fn new<T: Serialize>(
		module: impl Into<String>,
		r#type: impl Into<String>,
		payload: T,
	) -> Result<Self, serde_json::Error> {
		Ok(Self {
			module: module.into(),
			r#type: r#type.into(),
			payload: WsPayload::new(payload)?,
		})
	}
}

pub struct WsContext {
	conn: DatabaseConnection,
	state: WsState,
	username: String,
}

/// Socket-side rendering of the transport-agnostic [`ServiceError`]: `Client`
/// messages are safe to echo back to the sender, `Internal` failures are
/// logged and reported to the client as a generic error.
pub enum WsError {
	Client(String),
	Internal(anyhow::Error),
}

impl From<DbErr> for WsError {
	fn from(err: DbErr) -> Self {
		// The classification policy lives in error.rs, shared with REST.
		ServiceError::from(err).into()
	}
}

impl From<ServiceError> for WsError {
	fn from(err: ServiceError) -> Self {
		match err {
			ServiceError::Internal(err) => Self::Internal(err),
			ServiceError::NotFound(msg)
			| ServiceError::BadRequest(msg)
			| ServiceError::Conflict(msg)
			| ServiceError::Forbidden(msg) => Self::Client(msg),
			// Dead arm today: nothing on the socket path produces
			// Unauthorized. REST renders this variant body-less, so the
			// socket echo fabricates the one string it needs.
			ServiceError::Unauthorized => Self::Client("Unauthorized".into()),
		}
	}
}

impl From<anyhow::Error> for WsError {
	fn from(err: anyhow::Error) -> Self {
		Self::Internal(err)
	}
}

#[async_trait::async_trait]
pub trait WsModule: Send + Sync + 'static {
	fn name(&self) -> &'static str;

	async fn handle(
		&self,
		_ctx: &WsContext,
		_type: &str,
		_payload: &WsPayload,
	) -> Result<(), WsError> {
		Ok(())
	}

	fn should_deliver(&self, _ctx: &WsContext, _type: &str, _payload: &WsPayload) -> bool {
		true
	}
}

#[derive(Clone)]
pub struct WsState {
	tx: Sender<WsEnvelope>,
	modules: Arc<HashMap<&'static str, &'static dyn WsModule>>,
}

impl WsState {
	pub fn new(capacity: usize) -> Self {
		let (tx, _) = broadcast::channel::<WsEnvelope>(capacity);

		let modules = Arc::new(MODULE_LIST.iter().map(|m| (m.name(), *m)).collect());

		Self { tx, modules }
	}

	fn subscribe(&self) -> Receiver<WsEnvelope> {
		self.tx.subscribe()
	}

	async fn receive(rx: &mut Receiver<WsEnvelope>) -> Option<WsEnvelope> {
		match rx.recv().await {
			Ok(env) => Some(env),
			Err(broadcast::error::RecvError::Lagged(_)) => rx.recv().await.ok(),
			Err(broadcast::error::RecvError::Closed) => None,
		}
	}

	pub async fn broadcast<T: Serialize>(
		&self,
		module: &str,
		r#type: &str,
		payload: T,
	) -> Result<()> {
		if !self.modules.contains_key(module) {
			return Err(anyhow!("Unknown module: {module}"));
		}

		let env = WsEnvelope::new(module, r#type, &payload)?;

		// Fails only when no clients are connected.
		let _ = self.tx.send(env);

		Ok(())
	}
}

enum ClientEvent {
	Message(WsEnvelope),
	Invalid(String),
	MessageTooLarge,
	PeerClose,
	Disconnect,
	Continue,
}

async fn receive_msg_from_client(receiver: &mut SplitStream<WebSocket>) -> ClientEvent {
	match receiver.next().await {
		Some(Ok(WsMessage::Text(text))) => match serde_json::from_str(&text) {
			Ok(env) => ClientEvent::Message(env),
			Err(err) => ClientEvent::Invalid(format!("Invalid message envelope: {err}")),
		},
		Some(Ok(WsMessage::Close(_))) => ClientEvent::PeerClose,
		None => ClientEvent::Disconnect,
		Some(Ok(_)) => ClientEvent::Continue,
		Some(Err(err)) => {
			let err = err.into_inner();
			if matches!(
				err.downcast_ref::<TungsteniteError>(),
				Some(TungsteniteError::Capacity(
					CapacityError::MessageTooLong { .. }
				))
			) {
				ClientEvent::MessageTooLarge
			} else {
				eprintln!("WebSocket error: {err}");
				ClientEvent::Disconnect
			}
		}
	}
}

async fn send_msg_to_client(
	sender: &Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
	env: &WsEnvelope,
) -> Result<()> {
	let json = serde_json::to_string(env)?;
	let mut sender_guard = sender.lock().await;
	sender_guard.send(WsMessage::Text(json.into())).await?;
	Ok(())
}

async fn send_error_to_client(
	sender: &Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
	msg: &str,
) -> Result<()> {
	let env = WsEnvelope::new("system", "error", msg)?;
	send_msg_to_client(sender, &env).await
}

pub async fn handle_socket(
	socket: WebSocket,
	conn: DatabaseConnection,
	state: WsState,
	username: String,
) {
	let (sender, mut receiver) = socket.split();
	let sender = Arc::new(Mutex::new(sender));

	let mut rx = state.subscribe();

	let ctx = WsContext {
		conn,
		state: state.clone(),
		username,
	};

	loop {
		tokio::select! {
			msg = receive_msg_from_client(&mut receiver) => {
				match msg {
					ClientEvent::Message(env) => {
						if let Some(module) = state.modules.get(env.module.as_str()) {
							if let Err(err) = module.handle(&ctx, &env.r#type, &env.payload).await {
								let msg = match err {
									WsError::Client(msg) => msg,
									WsError::Internal(err) => {
										// Debug prints anyhow's full context chain.
										eprintln!("{err:?}");
										"Internal server error".to_string()
									}
								};

								if let Err(err) = send_error_to_client(&sender, &msg).await {
									eprintln!("{err}");
									break;
								}
							}
						} else if let Err(err) = send_error_to_client(
							&sender,
							&format!("Unknown module: {}", env.module)
						).await {
							eprintln!("{err}");
							break;
						}
					}
					ClientEvent::Invalid(msg) => {
						if let Err(err) = send_error_to_client(&sender, &msg).await {
							eprintln!("{err}");
							break;
						}
					},
					ClientEvent::MessageTooLarge => {
						let frame = CloseFrame {
							code: close_code::SIZE,
							reason: "Message exceeds the 64 KiB limit".into(),
						};
						let mut sender = sender.lock().await;
						if let Err(err) = sender.send(WsMessage::Close(Some(frame))).await {
							eprintln!("Failed to close oversized WebSocket message: {err}");
						}
						break;
					},
					ClientEvent::PeerClose => {
						let mut sender = sender.lock().await;
						if let Err(err) = sender.close().await {
							eprintln!("Failed to acknowledge WebSocket close: {err}");
						}
						break;
					},
					ClientEvent::Disconnect => break,
					ClientEvent::Continue => {},
				}
			}

			env = WsState::receive(&mut rx) => {
				if let Some(env) = env && let Some(module) = state.modules.get(env.module.as_str()) {
					let should_send = module.should_deliver(&ctx, &env.r#type, &env.payload);
					if should_send && let Err(err) = send_msg_to_client(&sender, &env).await {
						eprintln!("{err}");
						break;
					}
				}
			}
		}
	}
}
