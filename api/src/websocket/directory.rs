use crate::db::create_directory;
use crate::entity::directory::Model as Directory;
use crate::websocket::{WsContext, WsModule, WsPayload};
use anyhow::{Result, anyhow};

pub struct DirectoryModule;

#[async_trait::async_trait]
impl WsModule for DirectoryModule {
	fn name(&self) -> &'static str {
		"directory"
	}

	async fn handle(&self, ctx: &WsContext, r#type: &str, payload: &WsPayload) -> Result<()> {
		match r#type {
			"create_directory" => {
				let directory = payload.get::<Directory>()?;

				let created = create_directory(&ctx.conn, directory).await?;

				ctx.state
					.broadcast(self.name(), "directory_created", &created)
					.await
			}

			other => Err(anyhow!(
				"Invalid message type '{}' for module '{}'",
				other,
				self.name()
			)),
		}
	}
}
