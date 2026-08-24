use crate::AppState;
use crate::auth::{
	AuthResponse, Credentials, PasswordError, authenticate_user, generate_token, hash_password,
	validate_password,
};
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
use sea_orm::{DbErr, RuntimeErr, SqlxError};

fn db_error_status(err: DbErr) -> StatusCode {
	let status = match &err {
		DbErr::RecordNotFound(_) => StatusCode::NOT_FOUND,
		DbErr::Custom(_) => StatusCode::BAD_REQUEST,
		DbErr::Exec(RuntimeErr::SqlxError(SqlxError::Database(e)))
		| DbErr::Query(RuntimeErr::SqlxError(SqlxError::Database(e))) => match e.code().as_deref() {
			// Postgres SQLSTATEs: 23505 unique_violation, 23503 foreign_key_violation,
			// 23514 check_violation, 22001 string_data_right_truncation.
			Some("23505") => StatusCode::CONFLICT,
			Some("23503" | "23514" | "22001") => StatusCode::BAD_REQUEST,
			_ => StatusCode::INTERNAL_SERVER_ERROR,
		},
		_ => StatusCode::INTERNAL_SERVER_ERROR,
	};

	if status == StatusCode::INTERNAL_SERVER_ERROR {
		eprintln!("{err}");
	}

	status
}

#[allow(clippy::result_large_err)]
pub async fn get_users(State(app_state): State<AppState>) -> Result<Json<Vec<User>>> {
	match db::get_users(&app_state.conn).await {
		Ok(users) => Ok(Json(users)),
		Err(err) => Err(db_error_status(err).into()),
	}
}

#[allow(clippy::result_large_err)]
pub async fn get_user(
	State(app_state): State<AppState>,
	Path(path_username): Path<String>,
) -> Result<Json<User>> {
	match db::get_user(&app_state.conn, &path_username).await {
		Ok(user) => Ok(Json(user)),
		Err(err) => Err(db_error_status(err).into()),
	}
}

#[allow(clippy::result_large_err)]
pub async fn delete_user(
	State(app_state): State<AppState>,
	Extension(username): Extension<String>,
	Path(path_username): Path<String>,
) -> Result<Json<User>> {
	// Users may only delete themselves.
	if path_username.trim() != username {
		return Err(StatusCode::FORBIDDEN.into());
	}

	let deleted_user = db::delete_user(&app_state.conn, &path_username)
		.await
		.map_err(db_error_status)?;

	app_state
		.ws_state
		.broadcast("users", "user_deleted", &deleted_user)
		.await
		.map_err(|e| {
			eprintln!("{e}");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;

	Ok(Json(deleted_user))
}

#[allow(clippy::result_large_err)]
pub async fn get_directory(
	State(app_state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Vec<Directory>>> {
	match db::get_directory(&app_state.conn, id).await {
		Ok(directory) => Ok(Json(directory)),
		Err(err) => Err(db_error_status(err).into()),
	}
}

#[allow(clippy::result_large_err)]
pub async fn create_directory(
	State(app_state): State<AppState>,
	Json(directory): Json<Directory>,
) -> Result<Json<Directory>> {
	let created_directory = db::create_directory(&app_state.conn, directory)
		.await
		.map_err(db_error_status)?;

	app_state
		.ws_state
		.broadcast("directory", "directory_created", &created_directory)
		.await
		.map_err(|e| {
			eprintln!("{e}");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;

	Ok(Json(created_directory))
}

#[allow(clippy::result_large_err)]
pub async fn get_message_thread(
	State(app_state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Vec<Message>>> {
	match db::get_message_thread(&app_state.conn, id).await {
		Ok(thread) => Ok(Json(thread)),
		Err(err) => Err(db_error_status(err).into()),
	}
}

#[allow(clippy::result_large_err)]
pub async fn get_message(
	State(app_state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Message>> {
	match db::get_message(&app_state.conn, id).await {
		Ok(message) => Ok(Json(message)),
		Err(err) => Err(db_error_status(err).into()),
	}
}

#[allow(clippy::result_large_err)]
pub async fn create_message(
	State(app_state): State<AppState>,
	Extension(username): Extension<String>,
	Json(message): Json<Message>,
) -> Result<Json<Message>> {
	let created_message = db::create_message(&app_state.conn, username, message)
		.await
		.map_err(db_error_status)?;

	app_state
		.ws_state
		.broadcast("messages", "message_created", &created_message)
		.await
		.map_err(|e| {
			eprintln!("{e}");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;

	Ok(Json(created_message))
}

#[allow(clippy::result_large_err)]
pub async fn delete_message(
	State(app_state): State<AppState>,
	Extension(username): Extension<String>,
	Path(id): Path<i32>,
) -> Result<Json<Message>> {
	let message = db::get_message(&app_state.conn, id)
		.await
		.map_err(db_error_status)?;

	if message.author_username != username {
		return Err(StatusCode::FORBIDDEN.into());
	}

	let deleted_message = db::delete_message(&app_state.conn, id)
		.await
		.map_err(db_error_status)?;

	app_state
		.ws_state
		.broadcast("messages", "message_deleted", &deleted_message)
		.await
		.map_err(|e| {
			eprintln!("{e}");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;

	Ok(Json(deleted_message))
}

#[allow(clippy::result_large_err)]
pub async fn signup(
	State(app_state): State<AppState>,
	Json(mut user): Json<User>,
) -> Result<Json<AuthResponse>> {
	validate_password(&user.password).map_err(|e| match e {
		PasswordError::Empty => (StatusCode::BAD_REQUEST, "password must not be empty"),
		PasswordError::TooLong => (StatusCode::BAD_REQUEST, "password must be at most 72 bytes"),
	})?;

	match hash_password(&user.password) {
		Ok(hash) => user.password = hash,
		Err(err) => {
			eprintln!("{err}");
			return Err(StatusCode::INTERNAL_SERVER_ERROR.into());
		}
	};

	let created_user = db::create_user(&app_state.conn, user)
		.await
		.map_err(db_error_status)?;

	app_state
		.ws_state
		.broadcast("users", "user_created", &created_user)
		.await
		.map_err(|e| {
			eprintln!("{e}");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;

	let token = generate_token(&created_user.username).map_err(|e| {
		eprintln!("{e}");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Json(AuthResponse {
		user: created_user,
		token,
	}))
}

#[allow(clippy::result_large_err)]
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

	let token = generate_token(&user.username).map_err(|e| {
		eprintln!("{e}");
		StatusCode::INTERNAL_SERVER_ERROR
	})?;

	Ok(Json(AuthResponse { user, token }))
}

pub async fn ws_handler(
	ws: WebSocketUpgrade,
	State(app_state): State<AppState>,
	Extension(username): Extension<String>,
) -> Response {
	ws.on_upgrade(move |socket| handle_socket(socket, app_state.conn, app_state.ws_state, username))
}
