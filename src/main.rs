use axum::Router;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();
    axum::serve(
        listener,
        Router::new()
            .nest_service("/", ServeDir::new("public/"))
    )
    .await
    .unwrap();
}

