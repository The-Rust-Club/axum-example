use std::sync::{Arc, Mutex};

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::State, Router};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;
use axum::Json;
use tower_http::services::ServeDir;

const DATABASE_URL: &str = "postgres://example:rust@165.227.252.146/example";

#[derive(Serialize, Deserialize, Debug, FromRow)]
struct User {
    username: String,
    password: String,
}

#[derive(Debug, Clone)]
struct App {
    pub db_pool: sqlx::PgPool,
}

type AppState = Arc<Mutex<App>>;

impl User {
    pub fn from_unhashed(username: &str, password: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(password);
        let hashed_password = format!("{:x}", hasher.finalize());
        User {
            username: username.to_string(),
            password: hashed_password,
        }
    }
}

async fn get_users(State(app): State<AppState>) -> impl IntoResponse {
    let dbpool = &app.lock().unwrap().clone().db_pool;
       
    let users: Vec<User> = sqlx::query_as!(User,"SELECT * from USERS;").fetch_all(
            dbpool,
        ).await.unwrap();

    Json(users)
}

async fn add_user(State(app): State<AppState>, Json(body): Json<User>) -> impl IntoResponse {
    let user = User::from_unhashed(&body.username, &body.password);
    let dbpool = &app.lock().unwrap().clone().db_pool;
    let result = sqlx::query!("INSERT INTO USERS (username, password) VALUES ($1, $2);", &user.username, &user.password)
        .execute(dbpool)
        .await
        .unwrap();

    if result.rows_affected() == 1 {
        (
            StatusCode::OK,
            "User added successfully".to_string(),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to add user".to_string(),
        )
    }
}

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000")
        .await
        .unwrap();
    println!("Listening on: 127.0.0.1:5000");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(DATABASE_URL)
        .await
        .unwrap();
    println!("Connected to database");
    let app_state = Arc::new(Mutex::new(App { db_pool: pool }));
    axum::serve(
        listener,
        Router::new()
        .route("/users/get", get(get_users))
        .route("/users/add", post(add_user))
        .nest_service("/", ServeDir::new("public/"))
        .with_state(app_state)
        ,
    )
    .await
    .unwrap();
}
