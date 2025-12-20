use crate::db::create_message;
use crate::entity::messages::Model as Message;
use crate::websocket::{WsContext, WsEnvelope, WsModule};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct TypingPayload {
	thread_id: i32,
}

type StopTypingPayload = TypingPayload;

#[derive(Deserialize, Serialize)]
struct UserTypingPayload {
	username: String,
	thread_id: i32,
}

type UserStoppedTypingPayload = UserTypingPayload;

pub struct MessagingModule;

#[async_trait::async_trait]
impl WsModule for MessagingModule {
	fn name(&self) -> &'static str {
		"messaging"
	}

	async fn on_client_msg(&self, ctx: &WsContext, env: &WsEnvelope) -> Result<(), String> {
		match env.r#type.as_str() {
			"typing" => {
				let TypingPayload { thread_id } = env.get_payload()?;

				let payload = UserTypingPayload {
					username: ctx.username.clone(),
					thread_id,
				};

				ctx.state
					.broadcast(self.name(), "user_typing", &payload)
					.await
			}

			"stop_typing" => {
				let StopTypingPayload { thread_id } = env.get_payload()?;

				let payload = UserStoppedTypingPayload {
					username: ctx.username.clone(),
					thread_id,
				};

				ctx.state
					.broadcast(self.name(), "user_stopped_typing", &payload)
					.await
			}

			"create_message" => {
				let msg = env.get_payload::<Message>()?;

				let created = create_message(&ctx.conn, ctx.username.clone(), msg)
					.await
					.map_err(|e| format!("create_message failed: {e}"))?;

				ctx.state
					.broadcast(self.name(), "message_created", &created)
					.await
			}

			other => Err(format!("Unknown message type: {other}")),
		}
	}

	fn should_deliver(&self, ctx: &WsContext, env: &WsEnvelope) -> bool {
		match env.r#type.as_str() {
			"user_typing" | "user_stopped_typing" => match env.get_payload::<UserTypingPayload>() {
				Ok(p) => p.username != ctx.username,
				Err(_) => true,
			},
			_ => true,
		}
	}
}
