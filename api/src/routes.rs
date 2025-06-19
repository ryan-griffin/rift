use crate::auth::{AuthResponse, Credentials, authenticate_user, generate_token};
use crate::db;
use crate::entity::{
	directory::Model as Directory, messages::Model as Message, users::Model as User,
};
use axum::{
	Extension, Json,
	extract::{Path, State},
	http::StatusCode,
	response::Result,
};
use db::CreateMessage;
use sea_orm::DatabaseConnection;

pub async fn get_users(State(conn): State<DatabaseConnection>) -> Result<Json<Vec<User>>> {
	match db::get_users(&conn).await {
		Ok(users) => Ok(Json(users)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn get_user(
	State(conn): State<DatabaseConnection>,
	Path(path_username): Path<String>,
) -> Result<Json<User>> {
	match db::get_user(&conn, &path_username).await {
		Ok(user) => Ok(Json(user)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn get_directory(
	State(conn): State<DatabaseConnection>,
	Path(id): Path<i32>,
) -> Result<Json<Vec<Directory>>> {
	match db::get_directory(&conn, id).await {
		Ok(directory) => Ok(Json(directory)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn get_thread(
	State(conn): State<DatabaseConnection>,
	Path(id): Path<i32>,
) -> Result<Json<Vec<Message>>> {
	match db::get_thread(&conn, id).await {
		Ok(thread) => Ok(Json(thread)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn get_message(
	State(conn): State<DatabaseConnection>,
	Path(id): Path<i32>,
) -> Result<Json<Message>> {
	match db::get_message(&conn, id).await {
		Ok(message) => Ok(Json(message)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn create_message(
	State(conn): State<DatabaseConnection>,
	Extension(username): Extension<String>,
	Json(message): Json<CreateMessage>,
) -> Result<Json<Message>> {
	match db::create_message(&conn, username, message).await {
		Ok(created_message) => Ok(Json(created_message)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

pub async fn login(
	State(conn): State<DatabaseConnection>,
	Json(credentials): Json<Credentials>,
) -> Result<Json<AuthResponse>> {
	let user = match authenticate_user(&conn, &credentials).await {
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
