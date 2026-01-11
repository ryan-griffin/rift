use crate::db::create_message;
use crate::entity::messages::Model as Message;
use crate::websocket::{WsContext, WsModule, WsPayload};
use anyhow::{Result, anyhow};
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

pub struct MessagesModule;

#[async_trait::async_trait]
impl WsModule for MessagesModule {
	fn name(&self) -> &'static str {
		"messages"
	}

	async fn handle(&self, ctx: &WsContext, r#type: &str, payload: &WsPayload) -> Result<()> {
		match r#type {
			"typing" => {
				let TypingPayload { thread_id } = payload.get()?;

				let payload = UserTypingPayload {
					username: ctx.username.clone(),
					thread_id,
				};

				ctx.state
					.broadcast(self.name(), "user_typing", &payload)
					.await
			}

			"stop_typing" => {
				let StopTypingPayload { thread_id } = payload.get()?;

				let payload = UserStoppedTypingPayload {
					username: ctx.username.clone(),
					thread_id,
				};

				ctx.state
					.broadcast(self.name(), "user_stopped_typing", &payload)
					.await
			}

			"create_message" => {
				let msg = payload.get::<Message>()?;

				let created = create_message(&ctx.conn, ctx.username.clone(), msg).await?;

				ctx.state
					.broadcast(self.name(), "message_created", &created)
					.await
			}

			other => Err(anyhow!(
				"Invalid message type '{}' for module '{}'",
				other,
				self.name()
			)),
		}
	}

	fn should_deliver(&self, ctx: &WsContext, r#type: &str, payload: &WsPayload) -> bool {
		match r#type {
			"user_typing" | "user_stopped_typing" => match payload.get::<UserTypingPayload>() {
				Ok(p) => p.username != ctx.username,
				Err(_) => false,
			},
			_ => true,
		}
	}
}
