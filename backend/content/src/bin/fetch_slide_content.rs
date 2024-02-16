use std::io::{BufReader, Write};
use std::path::Path;
use std::{fs::File, path::PathBuf};

use bytes::Bytes;
use clap::Parser;

use shared::cli::progress_bar;
use shared::model::Event;
use tracing::{info, warn};
use url::Url;

/// Fetch Slide Content
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input csv path
    #[arg(long)]
    model_dir: PathBuf,

    /// where to to put slide text content
    #[arg(long)]
    slides: String,
}

#[derive(Debug)]
struct SlideWork {
    event: Event,
    url: Option<url::Url>,
    raw_content: Option<Bytes>,
    text_content: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let events_path = args.model_dir.join("events").with_extension("json");

    info!("Reading events from {} ... ", events_path.to_str().unwrap());
    let reader = BufReader::new(File::open(events_path)?);
    let events: Vec<Event> = serde_json::from_reader(reader)?;
    println!("done ");

    let mut phase1 = vec![];
    for event in events {
        let url = event.slides.first().cloned();
        phase1.push(SlideWork {
            event,
            url,
            raw_content: None,
            text_content: None,
        });
    }
    info!("{}", summarise_status(&phase1));

    info!("Fetching slide content");
    let mut phase2 = vec![];
    let phase1_progress = progress_bar(phase1.len() as u64);
    for work in phase1.into_iter() {
        if let Some(url) = &work.url {
            phase2.push(match fetch_content(url).await {
                Ok(content) => SlideWork {
                    raw_content: Some(content),
                    ..work
                },
                Err(e) => {
                    warn!("[{}]: got error fetching \'{}\': {}", work.event.id, url, e);
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
    let phase2_progress = progress_bar(phase2.len() as u64);
    for work in phase2.into_iter() {
        if let Some(raw_content) = &work.raw_content {
            phase3.push(match parse_content(&raw_content).await {
                Ok(text) => SlideWork {
                    text_content: Some(text),
                    ..work
                },
                Err(e) => {
                    warn!(
                        "[{}]: got error parsing content for \'{:?}\': {}",
                        work.event.id, work.url, e
                    );
                    work
                }
            });
        } else {
            phase3.push(work);
        }
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

async fn fetch_content(slide_url: &Url) -> Result<Bytes, Box<dyn std::error::Error>> {
    let result = reqwest::get(slide_url.to_string()).await?;
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
        .header("X-Tika-OCRskipOcr", "true")
        .header("X-Tika-Skip-Embedded", "true")
        .send()
        .await?;
    if result.status().is_success() {
        Ok(result.text().await?)
    } else {
        Err(format!("non-success: {}", result.status()).into())
    }
}
