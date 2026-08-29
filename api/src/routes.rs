use crate::AppState;
use crate::auth::{
	AuthResponse, Credentials, PasswordError, authenticate_user, generate_token, hash_password,
	validate_password,
};
use crate::db;
use crate::entity::{
	directory::Model as Directory, messages::Model as Message, users::Model as User,
};
use crate::error::ApiError;
use crate::websocket::handle_socket;
use axum::{
	Extension, Json,
	body::{Body, to_bytes},
	extract::{Path, Request, State, WebSocketUpgrade},
	http::{HeaderMap, StatusCode, header::CONTENT_TYPE},
	middleware::Next,
	response::{IntoResponse, Response},
};
use serde_json::json;

/// Refuse to buffer more than this much of a rejection body; axum's own
/// rejection texts are short.
const REJECTION_BODY_LIMIT: usize = 4096;

/// REST rendering of the transport-agnostic `ApiError`: client faults map
/// to 4xx statuses with their message as a JSON body; internal failures
/// are logged and stay body-less so internals never leak.
impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		let (status, message) = match &self {
			Self::NotFound(msg) => (StatusCode::NOT_FOUND, Some(msg.clone())),
			Self::BadRequest(msg) => (StatusCode::BAD_REQUEST, Some(msg.clone())),
			Self::Conflict(msg) => (StatusCode::CONFLICT, Some(msg.clone())),
			Self::Forbidden(msg) => (StatusCode::FORBIDDEN, Some(msg.clone())),
			Self::Unauthorized => (StatusCode::UNAUTHORIZED, None),
			Self::Internal(err) => {
				// Debug prints anyhow's full context chain; Display drops it.
				eprintln!("{err:?}");
				(StatusCode::INTERNAL_SERVER_ERROR, None)
			}
		};

		match message {
			Some(msg) => (status, Json(json!({ "error": msg }))).into_response(),
			None => status.into_response(),
		}
	}
}

/// Whether the content type is a JSON media type: `application/json`
/// ignoring parameters (`; charset=utf-8`) and structured-syntax suffixes
/// (`application/problem+json`).
fn is_json_content_type(headers: &HeaderMap) -> bool {
	headers
		.get(CONTENT_TYPE)
		.and_then(|value| value.to_str().ok())
		.and_then(|value| value.split(';').next())
		.map(|media_type| media_type.trim().to_ascii_lowercase())
		.is_some_and(|media_type| {
			media_type == "application/json"
				|| (media_type.ends_with("+json") && media_type.contains('/'))
		})
}

/// Axum's built-in extractor rejections (invalid path segments, malformed
/// request bodies) answer with plain-text 4xx bodies. Rewrite those into
/// the same JSON envelope `ApiError` produces so REST errors with bodies
/// share one shape. Empty-bodied 4xx responses are passed through
/// untouched, headers included. Rejection bodies over the buffer cap
/// receive a bounded generic error instead.
pub async fn normalize_rejections(request: Request, next: Next) -> Response {
	let (parts, body) = next.run(request).await.into_parts();

	if !parts.status.is_client_error() || is_json_content_type(&parts.headers) {
		return Response::from_parts(parts, body);
	}

	let bytes = match to_bytes(body, REJECTION_BODY_LIMIT).await {
		Ok(bytes) => bytes,
		Err(_) => {
			return (parts.status, Json(json!({ "error": "Request rejected" }))).into_response();
		}
	};
	let message = String::from_utf8_lossy(&bytes).trim().to_owned();

	if message.is_empty() {
		// Rebuild from the original parts, not from the status alone:
		// these responses can carry headers worth keeping (axum's Allow
		// on 405s, future Retry-After-style additions).
		return Response::from_parts(parts, Body::empty());
	}

	(parts.status, Json(json!({ "error": message }))).into_response()
}

pub async fn get_users(State(app_state): State<AppState>) -> Result<Json<Vec<User>>, ApiError> {
	Ok(Json(db::get_users(&app_state.conn).await?))
}

pub async fn get_user(
	State(app_state): State<AppState>,
	Path(path_username): Path<String>,
) -> Result<Json<User>, ApiError> {
	Ok(Json(db::get_user(&app_state.conn, &path_username).await?))
}

pub async fn delete_user(
	State(app_state): State<AppState>,
	Extension(username): Extension<String>,
	Path(path_username): Path<String>,
) -> Result<Json<User>, ApiError> {
	// Users may only delete themselves.
	if path_username.trim() != username {
		return Err(ApiError::only_own_account());
	}

	let deleted_user = db::delete_user(&app_state.conn, &path_username).await?;

	app_state
		.ws_state
		.broadcast("users", "user_deleted", &deleted_user)
		.await?;

	Ok(Json(deleted_user))
}

pub async fn get_directory(
	State(app_state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Vec<Directory>>, ApiError> {
	Ok(Json(db::get_directory(&app_state.conn, id).await?))
}

pub async fn create_directory(
	State(app_state): State<AppState>,
	Json(directory): Json<Directory>,
) -> Result<Json<Directory>, ApiError> {
	let created_directory = db::create_directory(&app_state.conn, directory).await?;

	app_state
		.ws_state
		.broadcast("directory", "directory_created", &created_directory)
		.await?;

	Ok(Json(created_directory))
}

pub async fn delete_directory(
	State(app_state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Directory>, ApiError> {
	let deleted_directory = db::delete_directory(&app_state.conn, id).await?;

	app_state
		.ws_state
		.broadcast("directory", "directory_deleted", &deleted_directory)
		.await?;

	Ok(Json(deleted_directory))
}

pub async fn get_message_thread(
	State(app_state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Vec<Message>>, ApiError> {
	Ok(Json(db::get_message_thread(&app_state.conn, id).await?))
}

pub async fn get_message(
	State(app_state): State<AppState>,
	Path(id): Path<i32>,
) -> Result<Json<Message>, ApiError> {
	Ok(Json(db::get_message(&app_state.conn, id).await?))
}

pub async fn create_message(
	State(app_state): State<AppState>,
	Extension(username): Extension<String>,
	Json(message): Json<Message>,
) -> Result<Json<Message>, ApiError> {
	let created_message = db::create_message(&app_state.conn, username, message).await?;

	app_state
		.ws_state
		.broadcast("messages", "message_created", &created_message)
		.await?;

	Ok(Json(created_message))
}

pub async fn delete_message(
	State(app_state): State<AppState>,
	Extension(username): Extension<String>,
	Path(id): Path<i32>,
) -> Result<Json<Message>, ApiError> {
	let message = db::get_message(&app_state.conn, id).await?;

	if message.author_username != username {
		return Err(ApiError::only_own_messages());
	}

	let deleted_message = db::delete_message(&app_state.conn, id).await?;

	app_state
		.ws_state
		.broadcast("messages", "message_deleted", &deleted_message)
		.await?;

	Ok(Json(deleted_message))
}

pub async fn signup(
	State(app_state): State<AppState>,
	Json(mut user): Json<User>,
) -> Result<Json<AuthResponse>, ApiError> {
	validate_password(&user.password).map_err(|e| match e {
		PasswordError::Empty => ApiError::BadRequest("password must not be empty".into()),
		PasswordError::TooLong => ApiError::BadRequest("password must be at most 72 bytes".into()),
	})?;

	user.password = hash_password(&user.password).map_err(|e| ApiError::Internal(e.into()))?;

	let created_user = db::create_user(&app_state.conn, user).await?;

	app_state
		.ws_state
		.broadcast("users", "user_created", &created_user)
		.await?;

	let token = generate_token(&created_user.username)?;

	Ok(Json(AuthResponse {
		user: created_user,
		token,
	}))
}

pub async fn login(
	State(app_state): State<AppState>,
	Json(credentials): Json<Credentials>,
) -> Result<Json<AuthResponse>, ApiError> {
	let user = authenticate_user(&app_state.conn, &credentials)
		.await?
		.ok_or(ApiError::Unauthorized)?;

	let token = generate_token(&user.username)?;

	Ok(Json(AuthResponse { user, token }))
}

pub async fn ws_handler(
	ws: WebSocketUpgrade,
	State(app_state): State<AppState>,
	Extension(username): Extension<String>,
) -> Response {
	ws.on_upgrade(move |socket| handle_socket(socket, app_state.conn, app_state.ws_state, username))
}
