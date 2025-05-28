mod auth;
mod db;
mod entity;
use auth::{Credentials, auth_middleware, authenticate_user, generate_token};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::Result,
    routing::{get, post},
};
use dotenvy::dotenv;
use entity::{directory::Model as Directory, messages::Model as Message, users::Model as User};
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use serde_json::Value;
use std::env;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenv().expect("Failed to load .env file");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let api_port = env::var("API_PORT").expect("API_PORT must be set");

    let conn = Database::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    Migrator::up(&conn, None).await.unwrap();

    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users/{username}", get(get_user))
        .route("/directory/{id}", get(get_directory))
        .route("/thread/{id}", get(get_thread))
        .route_layer(middleware::from_fn(auth_middleware))
        .route("/login", post(login))
        .with_state(conn);

    let listener = TcpListener::bind(format!("0.0.0.0:{api_port}"))
        .await
        .unwrap();
    println!("Server running on http://localhost:{api_port}");
    axum::serve(listener, app).await.unwrap();
}

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
