use std::io::BufReader;

use std::{fs::File, path::PathBuf};

use clap::Parser;

use shared::model::Event;
use tokio::sync::mpsc::{self, Receiver, Sender};
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
enum VideoDownload {
    Command(Url, PathBuf),
    End,
}

#[derive(Debug)]
enum AudioExtraction {
    Command(PathBuf, PathBuf),
    End,
}

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
        .map(|url| VideoDownload::Command(url, args.video_dir.clone()))
        .collect();

    let pending_downloads = subset(pending_downloads, args.offset, args.limit);
    let total_pending_downloads = pending_downloads.len();

    let mut join_set = JoinSet::new();
    let (video_download_tx, video_download_rx) =
        mpsc::channel::<VideoDownload>(total_pending_downloads + 1);
    let (audio_extraction_tx, audio_extraction_rx) =
        mpsc::channel::<AudioExtraction>(total_pending_downloads + 1);

    info!(
        "Fetching {} events with video content, saving in {}",
        pending_downloads.len(),
        args.video_dir.to_str().unwrap()
    );
    for pending_download in pending_downloads {
        video_download_tx.send(pending_download).await?;
    }
    video_download_tx.send(VideoDownload::End).await?;
    join_set.spawn(download_stage(video_download_rx, audio_extraction_tx));

    info!(
        "Extracting audio from videos, saving in {}",
        args.audio_dir.to_str().unwrap()
    );
    join_set.spawn(extraction_stage(audio_extraction_rx));

    while let Some(result) = join_set.join_next().await {
        let stage_result = result?;
        info!("{}", stage_result?);
    }

    Ok(())
}

async fn download_stage(
    mut video_download_rx: Receiver<VideoDownload>,
    audio_extraction_tx: Sender<AudioExtraction>,
) -> Result<String, String> {
    debug!("download stage starting");
    while let Some(pending_download) = video_download_rx.recv().await {
        use VideoDownload::*;

        match pending_download {
            Command(url, path) => {
                debug!("downloading {}", url);
                audio_extraction_tx
                    .send(AudioExtraction::Command(path.clone(), path.clone()))
                    .await
                    .map_err(|e| format!("error sending: {}", e))?;
            }
            End => {
                debug!("finished downloads");
                audio_extraction_tx
                    .send(AudioExtraction::End)
                    .await
                    .map_err(|e| format!("error sending: {}", e))?;
                break;
            }
        }
    }

    Ok("download stage completed".into())
}

async fn extraction_stage(
    mut audio_extraction_rx: Receiver<AudioExtraction>,
) -> Result<String, String> {
    while let Some(audio_extraction) = audio_extraction_rx.recv().await {
        use AudioExtraction::*;

        match audio_extraction {
            Command(from, to) => {
                debug!("extracting {:?} -> {:?}", from, to);
            }
            End => {
                debug!("finished extraction");
                break;
            }
        }
    }

    Ok("extraction stage completed".into())
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
