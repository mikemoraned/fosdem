use axum::{http::StatusCode, routing::get};
use dotenvy;
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber;
use webapp::router::router;

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

fn load_secret(name: &str) -> String {
    let secret = dotenvy::var(&name).expect(&format!("{} is not set", &name));
    let suffix = secret[(secret.len() - 3)..].to_string();
    info!(
        "Loaded secret with name '{}', ending with '{}'",
        name, suffix
    );
    secret
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    match dotenvy::dotenv() {
        Ok(path) => info!("Loaded env file at {:?}", path),
        Err(e) => warn!(
            "Failed to load env file, will use external env; error: {:?}",
            e
        ),
    }

    let openai_api_key = load_secret("OPENAI_API_KEY");
    let db_host = load_secret("DB_HOST");
    let db_key = load_secret("DB_KEY");

    let router = router(&openai_api_key, &db_host, &db_key).await;
    let app = router.route("/health", get(health));

    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    info!("will listen on {:?}", listener);
    axum::serve(listener, app).await?;

    Ok(())
}
