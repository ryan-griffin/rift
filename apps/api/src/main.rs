use axum::{Json, Router, extract::Path, routing::get};
use serde::Serialize;

#[derive(Serialize, Clone)]
struct User {
    user_name: &'static str,
    name: &'static str,
}

static USERS: &[User] = &[
    User {
        user_name: "alice",
        name: "Alice",
    },
    User {
        user_name: "bob",
        name: "Bob",
    },
];

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/users", get(users))
        .route("/users/{username}", get(get_user));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn users() -> Json<Vec<User>> {
    Json(USERS.to_vec())
}

async fn get_user(Path(username): Path<String>) -> Json<Option<User>> {
    let user = USERS.iter().find(|user| user.user_name == username);
    Json(user.cloned())
}
