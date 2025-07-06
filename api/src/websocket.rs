use crate::db::create_message;
use crate::entity::messages::Model as Message;
use axum::extract::ws::{Message as WsMsg, WebSocket};
use futures_util::{SinkExt, StreamExt, future, stream::SplitSink};
use sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use std::{
	collections::{HashMap, HashSet},
	sync::Arc,
};
use tokio::sync::{
	Mutex, broadcast,
	broadcast::{Receiver, Sender},
};

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
	#[serde(rename = "join_thread")]
	JoinThread { thread_id: i32 },
	#[serde(rename = "leave_thread")]
	LeaveThread { thread_id: i32 },
	#[serde(rename = "typing")]
	Typing { thread_id: i32 },
	#[serde(rename = "stop_typing")]
	StopTyping { thread_id: i32 },
	#[serde(rename = "create_message")]
	CreateMessage(Message),
	#[serde(rename = "user_joined")]
	UserJoinedThread { username: String, thread_id: i32 },
	#[serde(rename = "user_left")]
	UserLeftThread { username: String, thread_id: i32 },
	#[serde(rename = "user_typing")]
	UserTyping { username: String, thread_id: i32 },
	#[serde(rename = "user_stopped_typing")]
	UserStoppedTyping { username: String, thread_id: i32 },
	#[serde(rename = "message_created")]
	MessageCreated(Message),
	#[serde(rename = "error")]
	Error { message: String },
}

pub type WsState = Arc<Mutex<HashMap<i32, Sender<WsMessage>>>>;

pub async fn handle_socket(
	socket: WebSocket,
	conn: DatabaseConnection,
	ws_state: WsState,
	username: String,
) {
	let (sender, mut receiver) = socket.split();
	let sender = Arc::new(Mutex::new(sender));

	let mut thread_receivers = HashMap::<i32, Receiver<WsMessage>>::new();

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
								&mut thread_receivers,
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
			// Handle broadcast messages from joined threads
			broadcast_msg = receive_broadcast_message(&mut thread_receivers) => {
				if let Some(msg) = broadcast_msg {
					let should_send = match &msg {
						WsMessage::UserJoinedThread { username: join_user, .. } => join_user != &username,
						WsMessage::UserLeftThread { username: leave_user, .. } => leave_user != &username,
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

	// Cleanup: leave all joined threads and notify subscribers
	let joined_threads: HashSet<i32> = thread_receivers.keys().cloned().collect();
	cleanup_user_threads(&ws_state, &username, joined_threads).await;
}

async fn handle_incoming_message(
	message: WsMessage,
	conn: &DatabaseConnection,
	ws_state: &WsState,
	username: &str,
	thread_receivers: &mut HashMap<i32, Receiver<WsMessage>>,
) -> Result<(), String> {
	match message {
		WsMessage::JoinThread { thread_id } => {
			if !thread_receivers.contains_key(&thread_id) {
				if let Some(rx) = subscribe_to_thread(ws_state, thread_id).await {
					thread_receivers.insert(thread_id, rx);

					let join_msg = WsMessage::UserJoinedThread {
						username: username.to_string(),
						thread_id,
					};
					broadcast_to_thread(ws_state, thread_id, join_msg).await;
				}
			}
			Ok(())
		}
		WsMessage::LeaveThread { thread_id } => {
			if thread_receivers.remove(&thread_id).is_some() {
				unsubscribe_from_thread(ws_state, thread_id).await;

				let leave_msg = WsMessage::UserLeftThread {
					username: username.to_string(),
					thread_id,
				};
				broadcast_to_thread(ws_state, thread_id, leave_msg).await;
			}
			Ok(())
		}
		WsMessage::Typing { thread_id } => {
			let typing_msg = WsMessage::UserTyping {
				username: username.to_string(),
				thread_id,
			};
			broadcast_to_thread(ws_state, thread_id, typing_msg).await;
			Ok(())
		}
		WsMessage::StopTyping { thread_id } => {
			let stop_typing_msg = WsMessage::UserStoppedTyping {
				username: username.to_string(),
				thread_id,
			};
			broadcast_to_thread(ws_state, thread_id, stop_typing_msg).await;
			Ok(())
		}
		WsMessage::CreateMessage(create_msg) => {
			match create_message(conn, username.to_string(), create_msg).await {
				Ok(message) => {
					let broadcast_msg = WsMessage::MessageCreated(message.clone());
					broadcast_to_thread(ws_state, message.directory_id, broadcast_msg).await;
					Ok(())
				}
				Err(err) => Err(format!("Failed to create message: {err}")),
			}
		}
		_ => Ok(()),
	}
}

async fn subscribe_to_thread(ws_state: &WsState, thread_id: i32) -> Option<Receiver<WsMessage>> {
	let mut state = ws_state.lock().await;
	if let Some(tx) = state.get(&thread_id) {
		Some(tx.subscribe())
	} else {
		let (tx, rx) = broadcast::channel(100);
		state.insert(thread_id, tx);
		Some(rx)
	}
}

async fn unsubscribe_from_thread(ws_state: &WsState, thread_id: i32) {
	let mut state = ws_state.lock().await;
	if let Some(tx) = state.get(&thread_id) {
		if tx.receiver_count() == 0 {
			state.remove(&thread_id);
		}
	}
}

async fn broadcast_to_thread(ws_state: &WsState, thread_id: i32, message: WsMessage) {
	let state = ws_state.lock().await;
	if let Some(tx) = state.get(&thread_id) {
		let _ = tx.send(message);
	}
}

async fn receive_broadcast_message(
	thread_receivers: &mut HashMap<i32, Receiver<WsMessage>>,
) -> Option<WsMessage> {
	if thread_receivers.is_empty() {
		// If no receivers, wait indefinitely (this branch will never be selected)
		future::pending().await
	}

	loop {
		let thread_ids: Vec<i32> = thread_receivers.keys().cloned().collect();

		for thread_id in thread_ids {
			if let Some(rx) = thread_receivers.get_mut(&thread_id) {
				// Use try_recv first to check for immediate messages
				match rx.try_recv() {
					Ok(msg) => return Some(msg),
					Err(broadcast::error::TryRecvError::Empty) => continue,
					Err(broadcast::error::TryRecvError::Lagged(_)) => continue,
					Err(broadcast::error::TryRecvError::Closed) => {
						thread_receivers.remove(&thread_id);
						continue;
					}
				}
			}
		}

		// If no immediate messages, wait for one using recv() on first available receiver
		if let Some((&thread_id, rx)) = thread_receivers.iter_mut().next() {
			match rx.recv().await {
				Ok(msg) => return Some(msg),
				Err(broadcast::error::RecvError::Lagged(_)) => continue,
				Err(broadcast::error::RecvError::Closed) => {
					thread_receivers.remove(&thread_id);
					if thread_receivers.is_empty() {
						return None;
					}
					continue;
				}
			}
		} else {
			return None;
		}
	}
}

// Helper function to send messages to WebSocket client
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

// Cleanup function for when user disconnects
async fn cleanup_user_threads(ws_state: &WsState, username: &str, joined_threads: HashSet<i32>) {
	let mut state = ws_state.lock().await;
	for thread_id in joined_threads {
		if let Some(tx) = state.get(&thread_id) {
			let leave_msg = WsMessage::UserLeftThread {
				username: username.to_string(),
				thread_id,
			};
			let _ = tx.send(leave_msg);

			// Remove empty channels to prevent memory leaks
			if tx.receiver_count() == 0 {
				state.remove(&thread_id);
			}
		}
	}
}
