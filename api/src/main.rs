mod db;
use axum::{Json, Router, extract::State, http::StatusCode, response::Result, routing::get};
use entity::directory::Model as Directory;
use entity::users::Model as User;
use migration::{Migrator, MigratorTrait};
use sea_orm::{Database, DatabaseConnection};
use std::env;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().expect("Failed to load .env file");

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let api_port = env::var("API_PORT").expect("API_PORT must be set");

    let conn = Database::connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    Migrator::up(&conn, None).await.unwrap();
    // let migrations = Migrator::get_applied_migrations(&conn).await.unwrap();
    // assert_eq!(migrations.len(), 2);

    let app = Router::new()
        .route("/users", get(get_users))
        // .route("/users/{username}", get(get_user))
        .route("/directory", get(get_directory))
        .with_state(conn);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", api_port))
        .await
        .unwrap();
    println!("Server running on http://localhost:{}", api_port);
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

// async fn get_user(Path(username): Path<String>) -> Json<Option<User>> {
//     let user = USERS.iter().find(|user| user.username == username);
//     Json(user.cloned())
// }

async fn get_directory(State(conn): State<DatabaseConnection>) -> Result<Json<Vec<Directory>>> {
    match db::get_directory(&conn).await {
        Ok(directories) => Ok(Json(directories)),
        Err(err) => {
            eprintln!("{err}");
            Err(StatusCode::INTERNAL_SERVER_ERROR.into())
        }
    }
}
