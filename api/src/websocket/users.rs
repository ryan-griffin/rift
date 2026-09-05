use crate::service;
use crate::websocket::{WsContext, WsError, WsModule, WsPayload};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteUserPayload {
	username: String,
}

#[derive(Deserialize)]
struct UserCreatedPayload {
	username: String,
}

pub struct UsersModule;

#[async_trait::async_trait]
impl WsModule for UsersModule {
	fn name(&self) -> &'static str {
		"users"
	}

	async fn handle(
		&self,
		ctx: &WsContext,
		r#type: &str,
		payload: &WsPayload,
	) -> Result<(), WsError> {
		match r#type {
			"delete_user" => {
				let payload = payload.get::<DeleteUserPayload>()?;
				let deleted =
					service::delete_user(&ctx.conn, &ctx.auth.username, payload.username).await?;

				ctx.state.invalidate_user(&ctx.auth.username);
				ctx.state
					.broadcast(self.name(), "user_deleted", &deleted)
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
			"user_created" => match payload.get::<UserCreatedPayload>() {
				Ok(p) => p.username != ctx.auth.username,
				Err(_) => false,
			},
			_ => true,
		}
	}
}
