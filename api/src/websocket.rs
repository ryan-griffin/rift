use crate::db::create_message;
use crate::entity::messages::Model as Message;
use axum::extract::ws::{Message as WsMsg, WebSocket};
use futures_util::{SinkExt, StreamExt, stream::SplitSink};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{
	Mutex, broadcast,
	broadcast::{Receiver, Sender},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
	#[serde(rename = "typing")]
	Typing { thread_id: i32 },
	#[serde(rename = "stop_typing")]
	StopTyping { thread_id: i32 },
	#[serde(rename = "create_message")]
	CreateMessage(Message),
	#[serde(rename = "user_typing")]
	UserTyping { username: String, thread_id: i32 },
	#[serde(rename = "user_stopped_typing")]
	UserStoppedTyping { username: String, thread_id: i32 },
	#[serde(rename = "message_created")]
	MessageCreated(Message),
	#[serde(rename = "error")]
	Error { message: String },
}

pub type WsState = Arc<Mutex<Sender<WsMessage>>>;

pub async fn handle_socket(
	socket: WebSocket,
	conn: DatabaseConnection,
	ws_state: WsState,
	username: String,
) {
	let (sender, mut receiver) = socket.split();
	let sender = Arc::new(Mutex::new(sender));

	let mut message_receiver = {
		let state = ws_state.lock().await;
		state.subscribe()
	};

	loop {
		tokio::select! {
			// Handle incoming WebSocket messages
			msg = receiver.next() => {
				match msg {
					Some(Ok(WsMsg::Text(text))) => {
						if let Ok(ws_message) = serde_json::from_str::<WsMessage>(&text) {
							if let Err(err) = handle_incoming_message(
								ws_message,
								&conn,
								&ws_state,
								&username,
							).await {
								let error_msg = WsMessage::Error { message: err };
								let _ = send_message_to_client(&sender, &error_msg).await;
							}
						}
					}
					Some(Ok(WsMsg::Close(_))) | Some(Err(_)) | None => break,
					_ => {}
				}
			}
			// Handle broadcast messages
			broadcast_msg = receive_broadcast_message(&mut message_receiver) => {
				if let Some(msg) = broadcast_msg {
					let should_send = match &msg {
						WsMessage::UserTyping { username: typing_user, .. } => typing_user != &username,
						WsMessage::UserStoppedTyping { username: typing_user, .. } => typing_user != &username,
						_ => true,
					};

					if should_send {
						if send_message_to_client(&sender, &msg).await.is_err() {
							break;
						}
					}
				}
			}
		}
	}
}

async fn handle_incoming_message(
	message: WsMessage,
	conn: &DatabaseConnection,
	ws_state: &WsState,
	username: &str,
) -> Result<(), String> {
	match message {
		WsMessage::Typing { thread_id } => {
			let typing_msg = WsMessage::UserTyping {
				username: username.to_string(),
				thread_id,
			};
			broadcast_message(ws_state, typing_msg).await;
			Ok(())
		}
		WsMessage::StopTyping { thread_id } => {
			let stop_typing_msg = WsMessage::UserStoppedTyping {
				username: username.to_string(),
				thread_id,
			};
			broadcast_message(ws_state, stop_typing_msg).await;
			Ok(())
		}
		WsMessage::CreateMessage(create_msg) => {
			match create_message(conn, username.to_string(), create_msg).await {
				Ok(message) => {
					let broadcast_msg = WsMessage::MessageCreated(message.clone());
					broadcast_message(ws_state, broadcast_msg).await;
					Ok(())
				}
				Err(err) => {
					eprintln!("{err}");
					Err(format!("Failed to create message"))
				}
			}
		}
		_ => Ok(()),
	}
}

async fn broadcast_message(ws_state: &WsState, message: WsMessage) {
	let state = ws_state.lock().await;
	let _ = state.send(message);
}

async fn receive_broadcast_message(
	message_receiver: &mut Receiver<WsMessage>,
) -> Option<WsMessage> {
	match message_receiver.recv().await {
		Ok(msg) => Some(msg),
		Err(broadcast::error::RecvError::Lagged(_)) => match message_receiver.recv().await {
			Ok(msg) => Some(msg),
			Err(_) => None,
		},
		Err(broadcast::error::RecvError::Closed) => None,
	}
}

async fn send_message_to_client(
	sender: &Arc<Mutex<SplitSink<WebSocket, WsMsg>>>,
	message: &WsMessage,
) -> Result<(), ()> {
	let json = serde_json::to_string(message).map_err(|_| ())?;
	let mut sender_guard = sender.lock().await;
	sender_guard
		.send(WsMsg::Text(json.into()))
		.await
		.map_err(|_| ())
}
