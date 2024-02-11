use std::fs::File;

use clap::Parser;
use dotenvy;
use futures::future::join_all;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::embedding::{Embedding, EmbeddingParameters, EmbeddingResponse};
use serde::Deserialize;
use shared::cli::progress_bar;
use tracing::{info, warn};

/// Fetch Embeddings
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input csv path
    #[arg(long)]
    event_csv: String,
}

#[derive(Debug, Deserialize)]
struct SlideWork {
    id: u32,
    slides: String,
    raw_content: Option<String>,
    text_content: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut event_reader = csv::Reader::from_reader(File::open(&args.event_csv)?);
    info!("Reading events from {} ... ", args.event_csv);

    let mut works = vec![];
    for result in event_reader.deserialize() {
        let work: SlideWork = result?;
        works.push(work);
    }
    summarise_status(&works);

    info!("Fetching slide content");
    let raw_jobs = works
        .into_iter()
        .filter(|w| w.slides.len() > 0)
        .map(|work| async {
            match fetch_content(&work.slides).await {
                Ok(content) => SlideWork {
                    raw_content: Some(content),
                    ..work
                },
                Err(e) => {
                    warn!(
                        "[{}]: got error fetching \'{}\': {}",
                        work.id, work.slides, e
                    );
                    work
                }
            }
        });
    let works = join_all(raw_jobs).await;
    summarise_status(&works);

    Ok(())
}

fn summarise_status(works: &Vec<SlideWork>) {
    let with_slides: Vec<_> = works.iter().filter(|w| w.slides.len() > 0).collect();
    let with_raw_content: Vec<_> = works.iter().filter(|w| w.raw_content.is_some()).collect();
    let with_text_content: Vec<_> = works.iter().filter(|w| w.text_content.is_some()).collect();
    info!(
        "total: {}, with: slides: {}, raw: {}, text: {}",
        works.len(),
        with_slides.len(),
        with_raw_content.len(),
        with_text_content.len()
    )
}

async fn fetch_content(slide_url: &str) -> Result<String, Box<dyn std::error::Error>> {
    let result = reqwest::get(slide_url).await?;
    if result.status().is_success() {
        Ok(result.text().await?)
    } else {
        Err(format!("non-success: {}", result.status()).into())
    }
}
