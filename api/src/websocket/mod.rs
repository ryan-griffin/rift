mod directory;
mod messages;
mod users;

use crate::auth::AuthSession;
use crate::error::ServiceError;
use crate::service;
use anyhow::{Result, anyhow};
use axum::extract::ws::{CloseFrame, Message as WsMessage, WebSocket, close_code};
use chrono::Utc;
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
	time::Duration,
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
const CLOSE_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(1);
// Bound normal outbound writes so a client that stops reading cannot hold
// the connection task inside `send` while revocation or expiry is pending.
const SEND_TIMEOUT: Duration = Duration::from_secs(5);
const AUTH_INVALID_CLOSE_CODE: u16 = 4001;

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
	auth: AuthSession,
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
			| ServiceError::Gone(msg)
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
	auth_tx: Sender<AuthInvalidation>,
	modules: Arc<HashMap<&'static str, &'static dyn WsModule>>,
}

#[derive(Clone, Debug)]
enum AuthInvalidation {
	Session {
		username: String,
		token_hash: String,
	},
	User {
		username: String,
	},
}

impl AuthInvalidation {
	fn applies_to(&self, session: &AuthSession) -> bool {
		match self {
			Self::Session {
				username,
				token_hash,
			} => username == &session.username && token_hash == &session.token_hash,
			Self::User { username } => username == &session.username,
		}
	}
}

impl WsState {
	pub fn new(capacity: usize) -> Self {
		let (tx, _) = broadcast::channel::<WsEnvelope>(capacity);
		let (auth_tx, _) = broadcast::channel::<AuthInvalidation>(capacity);

		let modules = Arc::new(MODULE_LIST.iter().map(|m| (m.name(), *m)).collect());

		Self {
			tx,
			auth_tx,
			modules,
		}
	}

	fn subscribe(&self) -> Receiver<WsEnvelope> {
		self.tx.subscribe()
	}

	fn subscribe_auth(&self) -> Receiver<AuthInvalidation> {
		self.auth_tx.subscribe()
	}

	pub fn invalidate_session(&self, username: &str, token_hash: &str) {
		let _ = self.auth_tx.send(AuthInvalidation::Session {
			username: username.to_owned(),
			token_hash: token_hash.to_owned(),
		});
	}

	pub fn invalidate_user(&self, username: &str) {
		let _ = self.auth_tx.send(AuthInvalidation::User {
			username: username.to_owned(),
		});
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
	let send = async {
		let mut sender_guard = sender.lock().await;
		sender_guard.send(WsMessage::Text(json.into())).await
	};
	tokio::time::timeout(SEND_TIMEOUT, send)
		.await
		.map_err(|_| anyhow!("Timed out sending WebSocket message"))??;
	Ok(())
}

async fn send_error_to_client(
	sender: &Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
	msg: &str,
) -> Result<()> {
	let env = WsEnvelope::new("system", "error", msg)?;
	send_msg_to_client(sender, &env).await
}

async fn close_socket(
	sender: &Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
	receiver: &mut SplitStream<WebSocket>,
	code: u16,
	reason: &'static str,
) {
	let frame = CloseFrame {
		code,
		reason: reason.into(),
	};
	let send_close = async {
		let mut sender = sender.lock().await;
		sender.send(WsMessage::Close(Some(frame))).await
	};
	match tokio::time::timeout(CLOSE_HANDSHAKE_TIMEOUT, send_close).await {
		Ok(Ok(())) => {}
		Ok(Err(err)) => {
			eprintln!("Failed to close WebSocket: {err}");
			return;
		}
		Err(err) => {
			eprintln!("Failed to close WebSocket: {err}");
			return;
		}
	}

	let wait_for_acknowledgement = async {
		while let Some(message) = receiver.next().await {
			if matches!(message, Ok(WsMessage::Close(_)) | Err(_)) {
				break;
			}
		}
	};
	let _ = tokio::time::timeout(CLOSE_HANDSHAKE_TIMEOUT, wait_for_acknowledgement).await;
}

pub async fn handle_socket(
	socket: WebSocket,
	conn: DatabaseConnection,
	state: WsState,
	session: AuthSession,
) {
	let (sender, mut receiver) = socket.split();
	let sender = Arc::new(Mutex::new(sender));

	let mut rx = state.subscribe();
	let mut auth_rx = state.subscribe_auth();

	match service::is_session_active(&conn, &session).await {
		Ok(true) => {}
		Ok(false) => {
			close_socket(
				&sender,
				&mut receiver,
				AUTH_INVALID_CLOSE_CODE,
				"Authentication is no longer valid",
			)
			.await;
			return;
		}
		Err(err) => {
			eprintln!("Failed to validate WebSocket authentication: {err:?}");
			close_socket(
				&sender,
				&mut receiver,
				close_code::ERROR,
				"Unable to validate authentication",
			)
			.await;
			return;
		}
	}

	let ctx = WsContext {
		conn,
		state: state.clone(),
		auth: session,
	};
	let until_expiration = (ctx.auth.expires_at.with_timezone(&Utc) - Utc::now())
		.to_std()
		.unwrap_or_default();
	let expiration = tokio::time::sleep(until_expiration);
	tokio::pin!(expiration);

	loop {
		tokio::select! {
			biased;

			auth_event = auth_rx.recv() => {
				match auth_event {
					Ok(event) if event.applies_to(&ctx.auth) => {
						close_socket(
							&sender,
							&mut receiver,
							AUTH_INVALID_CLOSE_CODE,
							"Authentication revoked",
						).await;
						break;
					}
					Ok(_) => {}
					// A lagged receiver may have missed a revocation, so
					// treat it as revoked and let the client clear auth.
					Err(broadcast::error::RecvError::Lagged(_)) => {
						close_socket(
							&sender,
							&mut receiver,
							AUTH_INVALID_CLOSE_CODE,
							"Authentication state changed",
						).await;
						break;
					}
					Err(broadcast::error::RecvError::Closed) => {
						close_socket(
							&sender,
							&mut receiver,
							close_code::ERROR,
							"Authentication service unavailable",
						).await;
						break;
					}
				}
			}

			_ = &mut expiration => {
				close_socket(
					&sender,
					&mut receiver,
					AUTH_INVALID_CLOSE_CODE,
					"Authentication expired",
				).await;
				break;
			}

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
						close_socket(
							&sender,
							&mut receiver,
							close_code::SIZE,
							"Message exceeds the 64 KiB limit",
						).await;
						break;
					},
					ClientEvent::PeerClose => {
						let acknowledge = async {
							let mut sender = sender.lock().await;
							sender.close().await
						};
						match tokio::time::timeout(
							CLOSE_HANDSHAKE_TIMEOUT,
							acknowledge,
						)
						.await
						{
							Ok(Ok(())) => {}
							Ok(Err(err)) => {
								eprintln!("Failed to acknowledge WebSocket close: {err}");
							}
							Err(err) => {
								eprintln!("Failed to acknowledge WebSocket close: {err}");
							}
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
