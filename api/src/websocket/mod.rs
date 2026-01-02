mod messages;
mod users;

use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures_util::{
	SinkExt, StreamExt,
	stream::{SplitSink, SplitStream},
};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::sync::LazyLock;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
	Mutex, broadcast,
	broadcast::{Receiver, Sender},
};

static MODULE_LIST: LazyLock<Vec<&'static dyn WsModule>> =
	LazyLock::new(|| vec![&messages::MessagesModule, &users::UsersModule]);

#[derive(Clone, Serialize, Deserialize)]
pub struct WsPayload(Value);

impl WsPayload {
	fn new<T: Serialize>(payload: T) -> Result<Self, String> {
		serde_json::to_value(payload)
			.map(Self)
			.map_err(|e| format!("Invalid payload: {e}"))
	}

	pub fn get<T: DeserializeOwned>(&self) -> Result<T, String> {
		serde_json::from_value(self.0.clone()).map_err(|e| format!("Invalid payload: {e}"))
	}
}

#[derive(Clone, Serialize, Deserialize)]
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
	) -> Result<Self, String> {
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

#[async_trait::async_trait]
pub trait WsModule: Send + Sync + 'static {
	fn name(&self) -> &'static str;

	async fn handle(
		&self,
		_ctx: &WsContext,
		_type: &str,
		_payload: &WsPayload,
	) -> Result<(), String> {
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

	async fn subscribe(&self) -> Receiver<WsEnvelope> {
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
	) -> Result<(), String> {
		if !self.modules.contains_key(module) {
			return Err(format!("Unknown module: {module}"));
		}

		let env = WsEnvelope::new(module, r#type, &payload)?;
		self.tx
			.send(env)
			.map(|_| ())
			.map_err(|_| "Broadcast failed".into())
	}
}

async fn receive_msg_from_client(
	receiver: &mut SplitStream<WebSocket>,
) -> Option<Result<Option<WsEnvelope>, String>> {
	match receiver.next().await {
		Some(Ok(WsMessage::Text(text))) => match serde_json::from_str(&text) {
			Ok(env) => Some(Ok(Some(env))),
			Err(err) => Some(Err(format!("Invalid message: {err}"))),
		},
		Some(Ok(WsMessage::Close(_))) => Some(Ok(None)),
		Some(Err(err)) => Some(Err(format!("WebSocket error: {err}"))),
		_ => None,
	}
}

async fn send_msg_to_client(
	sender: &Arc<Mutex<SplitSink<WebSocket, WsMessage>>>,
	env: &WsEnvelope,
) -> Result<(), ()> {
	let json = serde_json::to_string(env).map_err(|_| ())?;
	let mut sender_guard = sender.lock().await;
	sender_guard
		.send(WsMessage::Text(json.into()))
		.await
		.map_err(|_| ())
}

pub async fn handle_socket(
	socket: WebSocket,
	conn: DatabaseConnection,
	state: WsState,
	username: String,
) {
	let (sender, mut receiver) = socket.split();
	let sender = Arc::new(Mutex::new(sender));

	let mut rx = state.subscribe().await;

	let ctx = WsContext {
		conn,
		state: state.clone(),
		username: username.clone(),
	};

	loop {
		tokio::select! {
			msg = receive_msg_from_client(&mut receiver) => {
				match msg {
					Some(Ok(Some(env))) => {
						if let Some(module) = state.modules.get(env.module.as_str()) {
							if let Err(err) = module.handle(&ctx, &env.r#type, &env.payload).await {
								eprintln!("{err}");
								if let Ok(msg) = WsEnvelope::new("system", "error", "Internal server error")
								&& send_msg_to_client(&sender, &msg).await.is_err() {
									break;
								}
							}
						} else if let Ok(msg) = WsEnvelope::new(
							"system",
							"error",
							format!("Unknown module: {}", &env.module)
						) && send_msg_to_client(&sender, &msg).await.is_err() {
							break;
						}
					}
					Some(Ok(None)) => break,
					Some(Err(err)) => eprintln!("{err}"),
					_ => {}
				}
			}

			env = WsState::receive(&mut rx) => {
				if let Some(env) = env && let Some(module) = state.modules.get(env.module.as_str()) {
					let should_send = module.should_deliver(&ctx, &env.r#type, &env.payload);
					if should_send && send_msg_to_client(&sender, &env).await.is_err() {
						break;
					}
				}
			}
		}
	}
}
