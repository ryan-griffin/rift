use crate::db::delete_user;
use crate::entity::users::Model as User;
use crate::error::ApiError;
use crate::websocket::{WsContext, WsError, WsModule, WsPayload};
use serde::Deserialize;

#[derive(Deserialize)]
struct DeleteUserPayload {
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

				// Users may only delete themselves, as on the REST side. The
				// payload username is only cross-checked; the delete itself
				// uses the authenticated connection identity so payload
				// whitespace games can't select a different row.
				if payload.username.trim() != ctx.username {
					return Err(ApiError::only_own_account().into());
				}

				let deleted = delete_user(&ctx.conn, &ctx.username).await?;

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
			"user_created" => match payload.get::<User>() {
				Ok(p) => p.username != ctx.username,
				Err(_) => false,
			},
			_ => true,
		}
	}
}
