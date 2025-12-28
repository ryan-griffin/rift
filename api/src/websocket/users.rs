use crate::entity::users::Model as User;
use crate::websocket::{WsContext, WsEnvelope, WsModule};

pub struct UsersModule;

#[async_trait::async_trait]
impl WsModule for UsersModule {
	fn name(&self) -> &'static str {
		"users"
	}

	fn should_deliver(&self, ctx: &WsContext, env: &WsEnvelope) -> bool {
		match env.r#type.as_str() {
			"user_created" => match env.get_payload::<User>() {
				Ok(p) => p.username != ctx.username,
				Err(_) => true,
			},
			_ => true,
		}
	}
}
