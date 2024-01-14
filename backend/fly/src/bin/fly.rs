use std::path::PathBuf;

use axum::{http::StatusCode, routing::get};
use clap::{Arg, Parser};
use dotenvy;
use shared::env::load_secret;
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber;
use webapp::router::{app_state, router};

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

/// start on fly.io
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to directory where CSV files are kept
    #[arg(short, long)]
    csv_data_dir: PathBuf,
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
    let args = Args::parse();

    let openai_api_key = load_secret("OPENAI_API_KEY");
    // let db_host = load_secret("DB_HOST");
    // let db_key = load_secret("DB_KEY");
    // let app_state = app_state(&openai_api_key, &db_host, &db_key).await;
    let app_state = app_state(&openai_api_key, &args.csv_data_dir).await;

    let router = router(app_state).await;
    let app = router.route("/health", get(health));

    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    info!("will listen on {:?}", listener);
    axum::serve(listener, app).await?;

    Ok(())
}
