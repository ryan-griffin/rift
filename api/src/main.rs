use axum::{Json, Router, extract::Path, routing::get};
use serde::Serialize;
use std::sync::LazyLock;

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

#[derive(Serialize, Clone)]
struct Node {
    id: u32,
    name: &'static str,
    children: Option<Vec<Node>>,
}

static DIRECTORY: LazyLock<Node> = LazyLock::new(|| Node {
    id: 0,
    name: "Root",
    children: Some(vec![
        Node {
            id: 1,
            name: "General",
            children: None,
        },
        Node {
            id: 2,
            name: "Gaming",
            children: Some(vec![
                Node {
                    id: 3,
                    name: "Roblox",
                    children: None,
                },
                Node {
                    id: 4,
                    name: "Fortnite",
                    children: None,
                },
                Node {
                    id: 5,
                    name: "Minecraft",
                    children: Some(vec![
                        Node {
                            id: 6,
                            name: "Mods",
                            children: None,
                        },
                        Node {
                            id: 7,
                            name: "Modpacks",
                            children: None,
                        },
                    ]),
                },
            ]),
        },
        Node {
            id: 8,
            name: "Programming",
            children: Some(vec![
                Node {
                    id: 9,
                    name: "TypeScript",
                    children: None,
                },
                Node {
                    id: 10,
                    name: "Rust",
                    children: None,
                },
            ]),
        },
        Node {
            id: 11,
            name: "Announcements",
            children: None,
        },
        Node {
            id: 12,
            name: "Memes",
            children: None,
        },
        Node {
            id: 13,
            name: "Help",
            children: None,
        },
    ]),
});

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/users", get(get_users))
        .route("/users/{username}", get(get_user))
        .route("/directory", get(get_directory));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    println!("Server running on http://localhost:8080");
    axum::serve(listener, app).await.unwrap();
}

async fn get_users() -> Json<Vec<User>> {
    Json(USERS.to_vec())
}

async fn get_user(Path(username): Path<String>) -> Json<Option<User>> {
    let user = USERS.iter().find(|user| user.user_name == username);
    Json(user.cloned())
}

async fn get_directory() -> Json<Node> {
    Json(DIRECTORY.clone())
}
