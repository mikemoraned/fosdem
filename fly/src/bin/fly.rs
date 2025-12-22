use std::path::PathBuf;

use axum::{http::StatusCode, routing::get};
use chrono::Utc;
use clap::Parser;
use fly::tracing::{init_opentelemetry_from_environment, init_safe_default_from_environment};
use shared::env::load_secret;
use tokio::net::TcpListener;
use tracing::info;
use webapp::router::{app_state, router};

async fn health() -> StatusCode {
    StatusCode::NO_CONTENT
}

/// start on fly.io
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// current year of the events
    #[arg(short, long)]
    current_year: u32,

    /// selectable years for filtering
    #[arg(short, long, value_delimiter = ' ')]
    selectable_years: Vec<u32>,

    /// path to directory where CSV files are kept
    #[arg(short, long)]
    model_dir: PathBuf,

    /// include video content at path
    #[arg(long)]
    include_video_content: Option<PathBuf>,

    /// enable opentelemetry
    #[arg(long)]
    opentelemetry: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match dotenvy::dotenv() {
        Ok(path) => println!("Loaded env file at {:?}", path),
        Err(e) => println!(
            "Failed to load env file, will use external env; error: {:?}",
            e
        ),
    }

    let args = Args::parse();

    if args.opentelemetry {
        match init_opentelemetry_from_environment() {
            Ok(_) => {
                info!("Opentelemetry initialised")
            }
            Err(e) => {
                println!(
                    "Failed to initialise Opentelemetry ('{:?}'), falling back to default",
                    e
                );
                init_safe_default_from_environment()?;
            }
        }
    } else {
        init_safe_default_from_environment()?;
    }

    let openai_api_key = load_secret("OPENAI_API_KEY")?;
    let app_state = app_state(
        &openai_api_key,
        &args.model_dir,
        &args.include_video_content,
        args.current_year,
        args.selectable_years,
        Utc::now(),
    )
    .await;

    let router = router(app_state).await;
    let app = router.route("/health", get(health));

    let listener = TcpListener::bind("0.0.0.0:8000").await?;
    info!("will listen on {:?}", listener);
    axum::serve(listener, app).await?;

    Ok(())
}
