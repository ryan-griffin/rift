use crate::service::{self, CreateMessageInput};
use crate::websocket::{WsContext, WsError, WsModule, WsPayload};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteMessagePayload {
	id: i32,
}

pub struct MessagesModule;

#[async_trait::async_trait]
impl WsModule for MessagesModule {
	fn name(&self) -> &'static str {
		"messages"
	}

	async fn handle(
		&self,
		ctx: &WsContext,
		r#type: &str,
		payload: &WsPayload,
	) -> Result<(), WsError> {
		match r#type {
			"typing" => {
				let TypingPayload { thread_id } = payload.get()?;
				let thread_id = service::require_thread(&ctx.conn, thread_id).await?.id;

				let payload = UserTypingPayload {
					username: ctx.auth.username.clone(),
					thread_id,
				};

				ctx.state
					.broadcast(self.name(), "user_typing", &payload)
					.await?;
				Ok(())
			}

			"stop_typing" => {
				let StopTypingPayload { thread_id } = payload.get()?;
				let thread_id = service::validate_thread_id(thread_id)?;

				let payload = UserStoppedTypingPayload {
					username: ctx.auth.username.clone(),
					thread_id,
				};

				ctx.state
					.broadcast(self.name(), "user_stopped_typing", &payload)
					.await?;
				Ok(())
			}

			"create_message" => {
				let input = payload.get::<CreateMessageInput>()?;
				let created =
					service::create_message(&ctx.conn, ctx.auth.username.clone(), input).await?;

				ctx.state
					.broadcast(self.name(), "message_created", &created)
					.await?;
				Ok(())
			}

			"delete_message" => {
				let payload = payload.get::<DeleteMessagePayload>()?;
				let deleted =
					service::delete_message(&ctx.conn, &ctx.auth.username, payload.id).await?;

				ctx.state
					.broadcast(self.name(), "message_deleted", &deleted)
					.await?;
				Ok(())
			}

			other => Err(WsError::Client(format!(
				"Invalid message type '{}' for module '{}'",
				other,
				self.name()
			))),
		}
	}

	fn should_deliver(&self, ctx: &WsContext, r#type: &str, payload: &WsPayload) -> bool {
		match r#type {
			"user_typing" | "user_stopped_typing" => match payload.get::<UserTypingPayload>() {
				Ok(p) => p.username != ctx.auth.username,
				Err(_) => false,
			},
			_ => true,
		}
	}
}
