use axum::{http::StatusCode, routing::get, Router};

async fn index() -> &'static str {
    "Hello, world!"
}

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health));

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
