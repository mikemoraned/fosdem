use std::io::BufReader;

use std::{fs::File, path::PathBuf};

use clap::Parser;

use shared::cli::progress_bar;
use shared::model::Event;
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tracing::{debug, info};

use url::Url;

/// Fetch Slide Content
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input csv path
    #[arg(long)]
    model_dir: PathBuf,

    /// where to download videos
    #[arg(long)]
    video_dir: PathBuf,

    /// where to store converted audio
    #[arg(long)]
    audio_dir: PathBuf,

    /// where to to put video text content in webvtt format
    #[arg(long)]
    webvtt_dir: PathBuf,

    /// optionally skip first N videos
    #[arg(long)]
    offset: Option<usize>,

    /// optionally restrict to only N videos
    #[arg(long)]
    limit: Option<usize>,
}

#[derive(Debug)]
struct VideoDownload(Url, PathBuf);

#[derive(Debug)]
struct AudioExtraction(PathBuf, PathBuf);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let events_path = args.model_dir.join("events").with_extension("json");

    info!("Reading events from {} ... ", events_path.to_str().unwrap());
    let reader = BufReader::new(File::open(events_path)?);
    let events: Vec<Event> = serde_json::from_reader(reader)?;

    let pending_downloads: Vec<VideoDownload> = events
        .into_iter()
        .map(|e| e.mp4_video_link())
        .flatten()
        .map(|url| VideoDownload(url, args.video_dir.clone()))
        .collect();

    let pending_downloads = subset(pending_downloads, args.offset, args.limit);
    let total_pending_downloads = pending_downloads.len();

    let mut join_set = JoinSet::new();
    let (video_download_tx, mut video_download_rx) =
        mpsc::channel::<VideoDownload>(total_pending_downloads);
    let (audio_extraction_tx, mut audio_extraction_rx) =
        mpsc::channel::<AudioExtraction>(total_pending_downloads);

    info!(
        "Fetching {} events with video content, saving in {}",
        pending_downloads.len(),
        args.video_dir.to_str().unwrap()
    );
    for pending_download in pending_downloads {
        video_download_tx.send(pending_download).await?;
    }
    join_set.spawn(async move {
        while let Some(pending_download) = video_download_rx.recv().await {
            let VideoDownload(url, path) = pending_download;
            debug!("downloading {}", url);
            audio_extraction_tx
                .send(AudioExtraction(path.clone(), path.clone()))
                .await;
        }
    });

    info!(
        "Extracting audio from videos, saving in {}",
        args.audio_dir.to_str().unwrap()
    );
    join_set.spawn(async move {
        while let Some(audio_extraction) = audio_extraction_rx.recv().await {
            debug!("extracting {:?}", audio_extraction);
        }
    });

    while let Some(result) = join_set.join_next().await {
        result?;
    }

    Ok(())
}

fn subset<T>(events: Vec<T>, offset: Option<usize>, limit: Option<usize>) -> Vec<T> {
    let offsetted: Vec<T> = if let Some(n) = offset {
        events.into_iter().skip(n).collect()
    } else {
        events
    };
    if let Some(n) = limit {
        offsetted.into_iter().take(n).collect()
    } else {
        offsetted
    }
}
