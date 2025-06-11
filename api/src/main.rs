mod auth;
mod db;
mod entity;
use auth::{Credentials, auth_middleware, authenticate_user, generate_token};
use axum::{
	Extension,
	Json,
	Router,
	// body::Body,
	extract::{Path, State},
	http::StatusCode,
	// http::{HeaderMap, StatusCode, Uri},
	middleware,
	response::Result,
	// response::{Response, Result},
	routing::{get, post},
};
use db::CreateMessage;
use dotenvy::dotenv;
use entity::{directory::Model as Directory, messages::Model as Message, users::Model as User};
use migration::{Migrator, MigratorTrait};
// use reqwest::Client;
use sea_orm::{Database, DatabaseConnection};
use serde_json::Value;
use std::env;
// use std::sync::LazyLock;
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

// static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| Client::new());

#[tokio::main]
async fn main() {
	dotenv().expect("Failed to load .env file");

	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let api_port = env::var("API_PORT").expect("API_PORT must be set");

	let conn = Database::connect(&database_url)
		.await
		.expect("Failed to connect to the database");

	Migrator::up(&conn, None).await.unwrap();

	let cors = CorsLayer::new()
		.allow_origin(Any)
		.allow_methods(Any)
		.allow_headers(Any);

	let app = Router::new()
		.route("/users", get(get_users))
		.route("/users/{username}", get(get_user))
		.route("/directory/{id}", get(get_directory))
		.route("/thread/{id}", get(get_thread))
		.route("/message/{id}", get(get_message))
		.route("/message", post(create_message))
		.route_layer(middleware::from_fn(auth_middleware))
		.route("/login", post(login))
		// .fallback(get(move |uri: Uri, headers: HeaderMap| {
		//     proxy(uri, 3000, headers)
		// }))
		.layer(cors)
		.with_state(conn);

	let listener = TcpListener::bind(format!("0.0.0.0:{api_port}"))
		.await
		.unwrap();
	println!("Server running on http://localhost:{api_port}");
	axum::serve(listener, app).await.unwrap();
}

// async fn proxy(uri: Uri, port: u16, headers: HeaderMap) -> Result<Response<Body>> {
//     let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
//     let proxy_url = format!("http://localhost:{}{}", port, path_and_query);

//     let mut request = HTTP_CLIENT.get(&proxy_url);

//     for (key, value) in headers.iter() {
//         if key != "host" {
//             if let Ok(header_value) = value.to_str() {
//                 request = request.header(key.as_str(), header_value);
//             }
//         }
//     }

//     match request.send().await {
//         Ok(response) => {
//             let status = response.status();
//             let response_headers = response.headers().clone();
//             let body = response.bytes().await.unwrap_or_default();

//             let mut builder = Response::builder().status(status.as_u16());

//             for (key, value) in response_headers.iter() {
//                 if key != "content-length" && key != "transfer-encoding" {
//                     builder = builder.header(key, value);
//                 }
//             }

//             Ok(builder.body(Body::from(body)).unwrap())
//         }
//         Err(e) => {
//             eprintln!("Proxy error for {}: {}", proxy_url, e);
//             Ok(Response::builder()
//                 .status(StatusCode::BAD_GATEWAY)
//                 .body(Body::from("Solid Start server unavailable"))
//                 .unwrap())
//         }
//     }
// }

async fn get_users(State(conn): State<DatabaseConnection>) -> Result<Json<Vec<User>>> {
	match db::get_users(&conn).await {
		Ok(users) => Ok(Json(users)),
		Err(err) => {
			eprintln!("{err}");
			Err(StatusCode::INTERNAL_SERVER_ERROR.into())
		}
	}
}

async fn get_user(
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

async fn get_directory(
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

async fn get_thread(
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

async fn get_message(
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

async fn create_message(
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

async fn login(
	State(conn): State<DatabaseConnection>,
	Json(credentials): Json<Credentials>,
) -> Result<Json<Value>> {
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

	Ok(Json(serde_json::json!({
		"user": user,
		"token": token
	})))
}
