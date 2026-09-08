mod directory;
mod messages;
mod users;

use crate::auth::{AuthEvents, AuthInvalidation, AuthSession};
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
	ops::ControlFlow,
	sync::{Arc, LazyLock},
	time::Duration,
};
use tokio::sync::{
	broadcast,
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
// Slow readers must not block revocation or expiry handling indefinitely.
const SEND_TIMEOUT: Duration = Duration::from_secs(5);
const AUTH_INVALID_CLOSE_CODE: u16 = 4001;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsPayload(Value);

impl WsPayload {
	fn new<T: Serialize>(payload: T) -> Result<Self, serde_json::Error> {
		serde_json::to_value(payload).map(Self)
	}

	pub fn get<T: DeserializeOwned>(&self) -> Result<T, WsError> {
		// Malformed client payloads are client errors; serialization failures are internal.
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
	auth_events: AuthEvents,
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

enum SocketIoEvent {
	Client(ClientEvent),
	Broadcast(Option<WsEnvelope>),
}

// Keep normal I/O fair even though the outer select prioritizes authentication.
async fn next_io_event(
	receiver: &mut SplitStream<WebSocket>,
	rx: &mut Receiver<WsEnvelope>,
) -> SocketIoEvent {
	tokio::select! {
		msg = receive_msg_from_client(receiver) => SocketIoEvent::Client(msg),
		env = WsState::receive(rx) => SocketIoEvent::Broadcast(env),
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
				ClientEvent::Disconnect
			}
		}
	}
}

async fn send_msg_to_client(
	sender: &mut SplitSink<WebSocket, WsMessage>,
	env: &WsEnvelope,
) -> bool {
	let json = match serde_json::to_string(env) {
		Ok(json) => json,
		Err(err) => {
			eprintln!("Failed to serialize WebSocket message: {err:?}");
			return false;
		}
	};
	matches!(
		tokio::time::timeout(SEND_TIMEOUT, sender.send(WsMessage::Text(json.into()))).await,
		Ok(Ok(()))
	)
}

async fn send_error_to_client(sender: &mut SplitSink<WebSocket, WsMessage>, msg: &str) -> bool {
	let env = match WsEnvelope::new("system", "error", msg) {
		Ok(env) => env,
		Err(err) => {
			eprintln!("Failed to build WebSocket error message: {err:?}");
			return false;
		}
	};
	send_msg_to_client(sender, &env).await
}

async fn close_socket(
	sender: &mut SplitSink<WebSocket, WsMessage>,
	receiver: &mut SplitStream<WebSocket>,
	code: u16,
	reason: &'static str,
) {
	let frame = CloseFrame {
		code,
		reason: reason.into(),
	};
	match tokio::time::timeout(
		CLOSE_HANDSHAKE_TIMEOUT,
		sender.send(WsMessage::Close(Some(frame))),
	)
	.await
	{
		Ok(Ok(())) => {}
		Ok(Err(_)) | Err(_) => {
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
	auth_events: AuthEvents,
	session: AuthSession,
) {
	let (mut sender, mut receiver) = socket.split();
	// Subscribe before revalidation so a concurrent revocation cannot be missed.
	let mut rx = state.subscribe();
	let mut auth_rx = auth_events.subscribe();
	let ctx = WsContext {
		conn,
		state,
		auth: session,
		auth_events,
	};

	let close = match service::is_session_active(&ctx.conn, &ctx.auth).await {
		Ok(true) => run_socket(&mut sender, &mut receiver, &ctx, &mut rx, &mut auth_rx).await,
		Ok(false) => Some((AUTH_INVALID_CLOSE_CODE, "Authentication is no longer valid")),
		Err(err) => {
			eprintln!("Failed to validate WebSocket authentication: {err:?}");
			Some((close_code::ERROR, "Unable to validate authentication"))
		}
	};

	if let Some((code, reason)) = close {
		close_socket(&mut sender, &mut receiver, code, reason).await;
	}
}

async fn handle_client_event(
	sender: &mut SplitSink<WebSocket, WsMessage>,
	ctx: &WsContext,
	event: ClientEvent,
) -> ControlFlow<Option<(u16, &'static str)>> {
	let error = match event {
		ClientEvent::Message(env) => match ctx.state.modules.get(env.module.as_str()) {
			Some(module) => module.handle(ctx, &env.r#type, &env.payload).await.err(),
			None => Some(WsError::Client(format!("Unknown module: {}", env.module))),
		},
		ClientEvent::Invalid(msg) => Some(WsError::Client(msg)),
		ClientEvent::MessageTooLarge => {
			return ControlFlow::Break(Some((
				close_code::SIZE,
				"Message exceeds the 64 KiB limit",
			)));
		}
		ClientEvent::PeerClose => {
			let _ = tokio::time::timeout(CLOSE_HANDSHAKE_TIMEOUT, sender.close()).await;
			return ControlFlow::Break(None);
		}
		ClientEvent::Disconnect => return ControlFlow::Break(None),
		ClientEvent::Continue => None,
	};

	if let Some(err) = error {
		let msg = match err {
			WsError::Client(msg) => msg,
			WsError::Internal(err) => {
				eprintln!("{err:?}");
				"Internal server error".to_string()
			}
		};
		if !send_error_to_client(sender, &msg).await {
			return ControlFlow::Break(None);
		}
	}

	ControlFlow::Continue(())
}

async fn run_socket(
	sender: &mut SplitSink<WebSocket, WsMessage>,
	receiver: &mut SplitStream<WebSocket>,
	ctx: &WsContext,
	rx: &mut Receiver<WsEnvelope>,
	auth_rx: &mut Receiver<AuthInvalidation>,
) -> Option<(u16, &'static str)> {
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
					Ok(event) if !event.applies_to(&ctx.auth) => continue,
					Ok(AuthInvalidation::Session { .. }) => {
						return Some((AUTH_INVALID_CLOSE_CODE, "Authentication revoked"));
					}
					Ok(AuthInvalidation::User { .. })
					| Err(broadcast::error::RecvError::Lagged(_)) => {
						// User-wide events can include a concurrent new login, and
						// lag may hide a revocation. Resolve both against the database.
						match service::is_session_active(&ctx.conn, &ctx.auth).await {
							Ok(true) => continue,
							Ok(false) => {
								return Some((AUTH_INVALID_CLOSE_CODE, "Authentication is no longer valid"));
							}
							Err(err) => {
								eprintln!("Failed to revalidate WebSocket authentication: {err:?}");
								return Some((close_code::ERROR, "Unable to validate authentication"));
							}
						}
					}
					Err(broadcast::error::RecvError::Closed) => {
						return Some((close_code::ERROR, "Authentication service unavailable"));
					}
				}
			}

			_ = &mut expiration => {
				return Some((AUTH_INVALID_CLOSE_CODE, "Authentication expired"));
			}

			event = next_io_event(receiver, rx) => match event {
				SocketIoEvent::Client(msg) => {
					if let ControlFlow::Break(close) = handle_client_event(sender, ctx, msg).await {
						return close;
					}
				},
				SocketIoEvent::Broadcast(env) => {
					if let Some(env) = env && let Some(module) = ctx.state.modules.get(env.module.as_str()) {
						let should_send = module.should_deliver(ctx, &env.r#type, &env.payload);
						if should_send && !send_msg_to_client(sender, &env).await {
							return None;
						}
					}
				}
			}
		}
	}
}
