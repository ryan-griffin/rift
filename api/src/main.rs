mod auth;
mod db;
mod entity;
mod error;
mod routes;
mod service;
mod websocket;
use anyhow::{Context, Result};
use auth::auth_middleware;
use axum::{
	Router,
	body::Body,
	http::{HeaderMap, StatusCode, Uri},
	middleware,
	response::Response,
	routing::{get, post},
};
use dotenvy::dotenv;
use error::ServiceError;
use migration::{Migrator, MigratorTrait};
use reqwest::Client;
use routes::*;
use sea_orm::{Database, DatabaseConnection};
use std::{env, future::IntoFuture, sync::LazyLock, time::Duration};
use tokio::{net::TcpListener, sync::oneshot};
use tower_http::cors::{Any, CorsLayer};
use websocket::WsState;

#[derive(Clone)]
pub struct AppState {
	pub conn: DatabaseConnection,
	pub ws_state: WsState,
}

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(Client::new);
const SHUTDOWN_GRACE_PERIOD: Duration = Duration::from_secs(8);

#[tokio::main]
async fn main() -> Result<()> {
	dotenv().ok();

	let database_url = env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
	let api_host = env::var("API_HOST").context("API_HOST must be set")?;
	let api_port = env::var("API_PORT").context("API_PORT must be set")?;
	let app_host = env::var("APP_HOST").context("APP_HOST must be set")?;
	let app_port = env::var("APP_PORT")
		.context("APP_PORT must be set")?
		.parse::<u16>()?;

	let conn = Database::connect(&database_url)
		.await
		.context("Failed to connect to the database")?;
	Migrator::up(&conn, None)
		.await
		.context("Failed to run database migrations")?;

	let ws_state = WsState::new(1000);

	let cors = CorsLayer::new()
		.allow_origin(Any)
		.allow_methods(Any)
		.allow_headers(Any);

	let public = Router::new()
		.route("/signup", post(signup))
		.route("/login", post(login));

	let authed = Router::new()
		.route("/users", get(get_users))
		.route("/users/{username}", get(get_user).delete(delete_user))
		.route(
			"/directory/{id}",
			get(get_directory).delete(delete_directory),
		)
		.route("/directory", post(create_directory))
		.route("/thread/{id}", get(get_message_thread))
		.route("/message/{id}", get(get_message).delete(delete_message))
		.route("/message", post(create_message))
		.route("/ws", get(ws_handler))
		.route_layer(middleware::from_fn(auth_middleware));

	let api = public
		.merge(authed)
		.route_layer(middleware::from_fn(normalize_rejections))
		.fallback(api_not_found);

	let app = Router::new()
		.nest("/api", api)
		.fallback(get(move |uri: Uri, headers: HeaderMap| {
			proxy(uri, app_host, app_port, headers)
		}))
		.layer(cors)
		.with_state(AppState { conn, ws_state });

	let listener = TcpListener::bind(format!("{api_host}:{api_port}")).await?;
	println!("Server running on http://{api_host}:{api_port}");

	let (shutdown_tx, shutdown_rx) = oneshot::channel();
	let server = axum::serve(listener, app)
		.with_graceful_shutdown(async {
			let _ = shutdown_rx.await;
		})
		.into_future();
	tokio::pin!(server);

	tokio::select! {
		result = &mut server => {
			result?;
			println!("Server stopped");
		}
		_ = shutdown_signal() => {
			let _ = shutdown_tx.send(());

			match tokio::time::timeout(SHUTDOWN_GRACE_PERIOD, &mut server).await {
				Ok(result) => {
					result?;
					println!("Server stopped");
				}
				Err(_) => {
					eprintln!("Shutdown deadline exceeded; forcing server stop");
				}
			}
		}
	}

	Ok(())
}

async fn api_not_found() -> ServiceError {
	ServiceError::NotFound("API route not found".into())
}

async fn shutdown_signal() {
	let ctrl_c = async {
		tokio::signal::ctrl_c()
			.await
			.expect("Failed to install Ctrl-C handler");
	};

	#[cfg(unix)]
	let terminate = async {
		tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
			.expect("Failed to install SIGTERM handler")
			.recv()
			.await;
	};

	#[cfg(not(unix))]
	let terminate = std::future::pending::<()>();

	tokio::select! {
		_ = ctrl_c => {},
		_ = terminate => {},
	}

	println!("\nStopping server...");
}

async fn proxy(
	uri: Uri,
	host: String,
	port: u16,
	headers: HeaderMap,
) -> Result<Response, StatusCode> {
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

			Ok(builder.body(Body::from(body)).map_err(|err| {
				eprintln!("Failed to build response body: {err}");
				StatusCode::INTERNAL_SERVER_ERROR
			})?)
		}
		Err(err) => {
			eprintln!("Proxy error for {}: {}", proxy_url, err);
			Err(StatusCode::BAD_GATEWAY)
		}
	}
}
