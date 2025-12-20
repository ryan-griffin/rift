use crate::AppState;
use crate::auth::{AuthResponse, Credentials, authenticate_user, generate_token, hash_password};
use crate::db;
use crate::entity::{
	directory::Model as Directory, messages::Model as Message, users::Model as User,
};
use crate::websocket::handle_socket;
use axum::{
	Extension, Json,
	extract::{Path, State, WebSocketUpgrade},
	http::StatusCode,
	response::{Response, Result},
};

pub async fn get_users(State(app_state): State<AppState>) -> Result<Json<Vec<User>>> {
	match db::get_users(&app_state.conn).await {
		Ok(users) => Ok(Json(users)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn get_user(
	State(app_state): State<AppState>,
	Path(path_username): Path<String>,
) -> Result<Json<User>> {
	match db::get_user(&app_state.conn, &path_username).await {
		Ok(user) => Ok(Json(user)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn get_directory(
	State(app_state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Vec<Directory>>> {
	match db::get_directory(&app_state.conn, id).await {
		Ok(directory) => Ok(Json(directory)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn get_thread(
	State(app_state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Vec<Message>>> {
	match db::get_thread(&app_state.conn, id).await {
		Ok(thread) => Ok(Json(thread)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn get_message(
	State(app_state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Message>> {
	match db::get_message(&app_state.conn, id).await {
		Ok(message) => Ok(Json(message)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn create_message(
	State(app_state): State<AppState>,
	Extension(username): Extension<String>,
	Json(message): Json<Message>,
) -> Result<Json<Message>> {
	let created_message = db::create_message(&app_state.conn, username, message)
		.await
		.map_err(|e| {
			eprintln!("{e}");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;

	app_state
		.ws_state
		.broadcast("messaging", "message_created", &created_message)
		.await
		.map_err(|e| {
			eprintln!("{e}");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;

	Ok(Json(created_message))
}

pub async fn login(
	State(app_state): State<AppState>,
	Json(credentials): Json<Credentials>,
) -> Result<Json<AuthResponse>> {
	let user = match authenticate_user(&app_state.conn, &credentials).await {
		Ok(Some(user)) => user,
		Ok(None) => return Err(StatusCode::UNAUTHORIZED.into()),
		Err(err) => {
			eprintln!("{err}");
			return Err(StatusCode::INTERNAL_SERVER_ERROR.into());
		}
	};

	let token = match generate_token(&user.username) {
		Ok(token) => token,
		Err(err) => {
			eprintln!("{err}");
			return Err(StatusCode::INTERNAL_SERVER_ERROR.into());
		}
	};

	Ok(Json(AuthResponse { user, token }))
}

pub async fn signup(
	State(app_state): State<AppState>,
	Json(mut user): Json<User>,
) -> Result<Json<AuthResponse>> {
	match hash_password(&user.password) {
		Ok(hash) => user.password = hash,
		Err(err) => {
			eprintln!("{err}");
			return Err(StatusCode::INTERNAL_SERVER_ERROR.into());
		}
	};

	let created_user = match db::create_user(&app_state.conn, user).await {
		Ok(created_user) => created_user,
		Err(err) => {
			eprintln!("{err}");
			return Err(StatusCode::INTERNAL_SERVER_ERROR.into());
		}
	};

	let token = match generate_token(&created_user.username) {
		Ok(token) => token,
		Err(err) => {
			eprintln!("{err}");
			return Err(StatusCode::INTERNAL_SERVER_ERROR.into());
		}
	};

	Ok(Json(AuthResponse {
		user: created_user,
		token,
	}))
}

pub async fn ws_handler(
	ws: WebSocketUpgrade,
	State(app_state): State<AppState>,
	Extension(username): Extension<String>,
) -> Response {
	ws.on_upgrade(move |socket| handle_socket(socket, app_state.conn, app_state.ws_state, username))
}
