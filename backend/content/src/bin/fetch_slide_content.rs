use std::io::Write;
use std::path::Path;

use std::sync::Arc;
use std::{collections::VecDeque, fs::File};

use bytes::Bytes;
use clap::Parser;

use serde::Deserialize;
use shared::cli::progress_bar;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Fetch Embeddings
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input csv path
    #[arg(long)]
    event_csv: String,

    /// where to to put slide text content
    #[arg(long)]
    slides: String,
}

#[derive(Debug, Deserialize, Clone)]
struct Event {
    id: u32,
    slides: String,
}

#[derive(Debug)]
struct SlideWork {
    event: Event,
    raw_content: Option<Bytes>,
    text_content: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let mut event_reader = csv::Reader::from_reader(File::open(&args.event_csv)?);
    info!("Reading events from {} ... ", args.event_csv);

    let mut phase1 = vec![];
    for result in event_reader.deserialize() {
        let event: Event = result?;
        phase1.push(SlideWork {
            event,
            raw_content: None,
            text_content: None,
        });
    }
    info!("{}", summarise_status(&phase1));

    // let phase1: Vec<_> = phase1.into_iter().take(10).collect();

    info!("Fetching slide content");
    let mut phase2 = vec![];
    let phase1_progress = progress_bar(phase1.len() as u64);
    for work in phase1.into_iter() {
        if work.event.slides.len() > 0 {
            phase2.push(match fetch_content(&work.event.slides).await {
                Ok(content) => SlideWork {
                    raw_content: Some(content),
                    ..work
                },
                Err(e) => {
                    warn!(
                        "[{}]: got error fetching \'{}\': {}",
                        work.event.id, work.event.slides, e
                    );
                    work
                }
            });
        } else {
            phase2.push(work);
        }
        phase1_progress.inc(1);
    }
    info!("{}", summarise_status(&phase2));

    info!("Parsing slide content");
    let mut phase3 = vec![];
    debug!("Dispatching tasks");
    let max_concurrent = Arc::new(tokio::sync::Semaphore::new(4));
    let mut pending_tasks = VecDeque::new();
    for work in phase2.into_iter() {
        pending_tasks.push_back(tokio::spawn(parse_content_task(
            max_concurrent.clone(),
            work,
        )));
    }
    let phase2_progress = progress_bar(pending_tasks.len() as u64);
    debug!("Fetching task results");
    while pending_tasks.len() > 0 {
        let pending_task = pending_tasks.pop_front().unwrap();
        phase3.push(pending_task.await.unwrap());
        phase2_progress.inc(1);
    }
    info!("{}", summarise_status(&phase3));

    info!("Saving slide content");
    let phase3_progress = progress_bar(phase3.len() as u64);
    let base_path = Path::new(&args.slides);
    for work in phase3.into_iter() {
        if let Some(text) = &work.text_content {
            let file_path = base_path
                .join(work.event.id.to_string())
                .with_extension("txt");
            let mut file = File::create(file_path)?;
            file.write_all(text.as_bytes())?;
        }
        phase3_progress.inc(1);
    }

    Ok(())
}

async fn parse_content_task(limit: Arc<Semaphore>, work: SlideWork) -> SlideWork {
    if let Some(raw_content) = &work.raw_content {
        let permit = limit.acquire().await.unwrap();
        let result = parse_content(&raw_content);
        drop(permit);
        match result.await {
            Ok(text) => SlideWork {
                text_content: Some(text),
                ..work
            },
            Err(e) => {
                warn!(
                    "[{}]: got error parsing content for \'{}\': {}",
                    work.event.id, work.event.slides, e
                );
                work
            }
        }
    } else {
        work
    }
}

fn summarise_status(works: &Vec<SlideWork>) -> String {
    let with_slides: Vec<_> = works.iter().filter(|w| w.event.slides.len() > 0).collect();
    let with_raw_content: Vec<_> = works.iter().filter(|w| w.raw_content.is_some()).collect();
    let with_text_content: Vec<_> = works.iter().filter(|w| w.text_content.is_some()).collect();
    format!(
        "status: total: {}, with: slides: {}, raw: {}, text: {}",
        works.len(),
        with_slides.len(),
        with_raw_content.len(),
        with_text_content.len()
    )
}

async fn fetch_content(slide_url: &str) -> Result<Bytes, Box<dyn std::error::Error>> {
    let result = reqwest::get(slide_url).await?;
    if result.status().is_success() {
        Ok(result.bytes().await?)
    } else {
        Err(format!("non-success: {}", result.status()).into())
    }
}

async fn parse_content(raw_content: &Bytes) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let result = client
        .put("https://fosdem2024-tika.fly.dev/tika")
        .body(raw_content.clone())
        .header("Accept", "text/plain")
        .send()
        .await?;
    if result.status().is_success() {
        Ok(result.text().await?)
    } else {
        Err(format!("non-success: {}", result.status()).into())
    }
}
