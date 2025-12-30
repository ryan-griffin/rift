mod auth;
mod db;
mod entity;
mod routes;
mod websocket;
use auth::auth_middleware;
use axum::{
	Router,
	body::Body,
	http::{HeaderMap, StatusCode, Uri},
	middleware,
	response::{Response, Result},
	routing::{get, post},
};
use dotenvy::dotenv;
use migration::{Migrator, MigratorTrait};
use reqwest::Client;
use routes::*;
use sea_orm::{Database, DatabaseConnection};
use std::env;
use std::sync::LazyLock;
use tokio::{net::TcpListener, sync::broadcast};
use tower_http::cors::{Any, CorsLayer};
use websocket::{WsEnvelope, WsState};

#[derive(Clone)]
pub struct AppState {
	pub conn: DatabaseConnection,
	pub ws_state: WsState,
}

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);

#[tokio::main]
async fn main() {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let api_host = env::var("API_HOST").expect("API_HOST must be set");
	let api_port = env::var("API_PORT").expect("API_PORT must be set");
	let app_host = env::var("APP_HOST").expect("APP_HOST must be set");
	let app_port = env::var("APP_PORT")
		.expect("APP_PORT must be set")
		.parse::<u16>()
		.unwrap();

	let conn = Database::connect(&database_url)
		.await
		.expect("Failed to connect to the database");
	Migrator::up(&conn, None).await.unwrap();

	let (tx, _) = broadcast::channel::<WsEnvelope>(1000);
	let ws_state = WsState::new(tx);

	let cors = CorsLayer::new()
		.allow_origin(Any)
		.allow_methods(Any)
		.allow_headers(Any);

	let app = Router::new()
		.route("/api/users", get(get_users))
		.route("/api/users/{username}", get(get_user))
		.route("/api/directory/{id}", get(get_directory))
		.route("/api/directory", post(create_directory))
		.route("/api/thread/{id}", get(get_message_thread))
		.route("/api/message/{id}", get(get_message))
		.route("/api/message", post(create_message))
		.route("/api/ws", get(ws_handler))
		.route_layer(middleware::from_fn(auth_middleware))
		.route("/api/signup", post(signup))
		.route("/api/login", post(login))
		.fallback(get(move |uri: Uri, headers: HeaderMap| {
			proxy(uri, app_host, app_port, headers)
		}))
		.layer(cors)
		.with_state(AppState { conn, ws_state });

	let listener = TcpListener::bind(format!("{api_host}:{api_port}"))
		.await
		.unwrap();
	println!("Server running on http://{api_host}:{api_port}");
	axum::serve(listener, app).await.unwrap();
}

async fn proxy(uri: Uri, host: String, port: u16, headers: HeaderMap) -> Result<Response> {
	let path_and_query = uri.path_and_query().map(|pq| pq.as_str()).unwrap_or("/");
	let proxy_url = format!("http://{host}:{port}{path_and_query}");

	let mut request = HTTP_CLIENT.get(&proxy_url);

	for (key, value) in headers.iter() {
		if key != "host"
			&& let Ok(header_value) = value.to_str()
		{
			request = request.header(key.as_str(), header_value);
		}
	}

	match request.send().await {
		Ok(response) => {
			let status = response.status();
			let response_headers = response.headers().clone();
			let body = response.bytes().await.unwrap_or_default();

			let mut builder = Response::builder().status(status.as_u16());

			for (key, value) in response_headers.iter() {
				if key != "content-length" && key != "transfer-encoding" {
					builder = builder.header(key, value);
				}
			}

			Ok(builder.body(Body::from(body)).unwrap())
		}
		Err(e) => {
			eprintln!("Proxy error for {}: {}", proxy_url, e);
			Err(StatusCode::BAD_GATEWAY.into())
		}
	}
}
