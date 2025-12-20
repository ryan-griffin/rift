mod messaging;

use axum::extract::ws::{Message as WsMessage, WebSocket};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{
	Mutex, broadcast,
	broadcast::{Receiver, Sender},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct WsEnvelope {
	module: String,
	#[serde(rename = "type")]
	r#type: String,
	#[serde(default)]
	payload: Value,
}

impl WsEnvelope {
	fn new<T: Serialize>(
		module: impl Into<String>,
		r#type: impl Into<String>,
		payload: T,
	) -> Result<Self, String> {
		serde_json::to_value(payload)
			.map(|v| Self {
				module: module.into(),
				r#type: r#type.into(),
				payload: v,
			})
			.map_err(|e| format!("Invalid payload: {e}"))
	}

	fn get_payload<T: for<'de> Deserialize<'de>>(&self) -> Result<T, String> {
		serde_json::from_value(self.payload.clone()).map_err(|e| format!("Invalid payload: {e}"))
	}
}

#[derive(Clone)]
pub struct WsState {
	tx: Sender<WsEnvelope>,
}

impl WsState {
	pub fn new(tx: Sender<WsEnvelope>) -> Self {
		Self { tx }
	}

	async fn subscribe(&self) -> Receiver<WsEnvelope> {
		self.tx.subscribe()
	}

	pub async fn broadcast<T: Serialize>(
		&self,
		module: &str,
		r#type: &str,
		payload: T,
	) -> Result<(), String> {
		let env = WsEnvelope::new(module, r#type, payload)?;
		self.tx
			.send(env)
			.map(|_| ())
			.map_err(|_| "Broadcast failed".into())
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

	async fn on_client_msg(&self, _ctx: &WsContext, _env: &WsEnvelope) -> Result<(), String> {
		Ok(())
	}

	fn should_deliver(&self, _ctx: &WsContext, _env: &WsEnvelope) -> bool {
		true
	}
}

fn module_registry() -> HashMap<&'static str, Arc<dyn WsModule>> {
	let mut map: HashMap<&'static str, Arc<dyn WsModule>> = HashMap::new();

	map.insert("messaging", Arc::new(messaging::MessagingModule));

	map
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

async fn receive_broadcast_msg(rx: &mut Receiver<WsEnvelope>) -> Option<WsEnvelope> {
	match rx.recv().await {
		Ok(msg) => Some(msg),
		Err(broadcast::error::RecvError::Lagged(_)) => rx.recv().await.ok(),
		Err(broadcast::error::RecvError::Closed) => None,
	}
}

pub async fn handle_socket(
	socket: WebSocket,
	conn: DatabaseConnection,
	state: WsState,
	username: String,
) {
	let modules = module_registry();

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
			msg = receiver.next() => {
				match msg {
					Some(Ok(WsMessage::Text(text))) => {
						match serde_json::from_str::<WsEnvelope>(&text) {
							Ok(env) => {
								if let Some(module) = modules.get(env.module.as_str()) {
									if let Err(err) = module.on_client_msg(&ctx, &env).await
										&& let Ok(msg) = WsEnvelope::new("system", "error", err)
									{
										send_msg_to_client(&sender, &msg).await.unwrap();
									}
								} else if let Ok(msg) = WsEnvelope::new(
									"system",
									"error",
									format!("Unknown module: {}", env.module)
								) {
									send_msg_to_client(&sender, &msg).await.unwrap();
								}
							}
							Err(err) => {
								if let Ok(msg) = WsEnvelope::new("system", "error", format!("Invalid message: {err}")) {
									send_msg_to_client(&sender, &msg).await.unwrap();
								}
							}
						}
					}
					Some(Ok(WsMessage::Close(_))) | Some(Err(_)) | None => break,
					_ => {}
				}
			}

			env = receive_broadcast_msg(&mut rx) => {
				if let Some(msg) = env {
					let should_send = match modules.get(msg.module.as_str()) {
						Some(module) => module.should_deliver(&ctx, &msg),
						None => true,
					};

					if should_send && send_msg_to_client(&sender, &msg).await.is_err() {
						break;
					}
				}
			}
		}
	}
}
