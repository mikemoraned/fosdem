use axum::{http::StatusCode, routing::get, Router};
use dotenvy;

async fn index() -> &'static str {
    "Hello, world!"
}

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

fn load_secret(name: &str) -> String {
    let secret = dotenvy::var(&name).expect(&format!("{} is not set", &name));
    let suffix = secret[(secret.len() - 3)..].to_string();
    println!(
        "Loaded secret with name '{}', ending with '{}'",
        name, suffix
    );
    secret
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    let openai_api_key = load_secret("OPENAI_API_KEY");
    let db_host = load_secret("DB_HOST");
    let db_key = load_secret("DB_KEY");

    let app = Router::new()
        .route("/", get(index))
        .route("/health", get(health));

    axum::Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();

    Ok(())
}
