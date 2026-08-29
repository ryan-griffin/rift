use crate::db::{create_directory, delete_directory};
use crate::entity::directory::Model as Directory;
use crate::websocket::{WsContext, WsError, WsModule, WsPayload};
use serde::Deserialize;

#[derive(Deserialize)]
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
				let directory = payload.get::<Directory>()?;

				let created = create_directory(&ctx.conn, directory).await?;

				ctx.state
					.broadcast(self.name(), "directory_created", &created)
					.await?;
				Ok(())
			}

			"delete_directory" => {
				let payload = payload.get::<DeleteDirectoryPayload>()?;

				let deleted = delete_directory(&ctx.conn, payload.id).await?;

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
