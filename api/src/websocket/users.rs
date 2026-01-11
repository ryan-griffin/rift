use crate::entity::users::Model as User;
use crate::websocket::{WsContext, WsModule, WsPayload};

pub struct UsersModule;

#[async_trait::async_trait]
impl WsModule for UsersModule {
	fn name(&self) -> &'static str {
		"users"
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
