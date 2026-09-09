use crate::service::{self, CreateDirectoryInput};
use crate::websocket::{WsContext, WsError, WsModule, WsPayload};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DeleteDirectoryPayload {
	id: i32,
}

pub struct DirectoryModule;

#[async_trait::async_trait]
impl WsModule for DirectoryModule {
	fn name(&self) -> &'static str {
		"directory"
	}

	async fn handle(
		&self,
		ctx: &WsContext,
		r#type: &str,
		payload: &WsPayload,
	) -> Result<(), WsError> {
		match r#type {
			"create_directory" => {
				let input = payload.get::<CreateDirectoryInput>()?;
				let created = service::create_directory(&ctx.conn, input).await?;

				ctx.state
					.broadcast(self.name(), "directory_created", &created)
					.await?;
				Ok(())
			}

			"delete_directory" => {
				let payload = payload.get::<DeleteDirectoryPayload>()?;
				let deleted = service::delete_directory(&ctx.conn, payload.id).await?;

				ctx.state
					.broadcast(self.name(), "directory_deleted", &deleted)
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
}
