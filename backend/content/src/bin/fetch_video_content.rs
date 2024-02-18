use std::io::BufReader;

use std::{fs::File, path::PathBuf};

use clap::Parser;

use shared::cli::progress_bar;
use shared::model::Event;
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
    // /// where to store converted audio
    // #[arg(long)]
    // audio_dir: PathBuf,

    // /// where to to put video text content in webvtt format
    // #[arg(long)]
    // webvtt_dir: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let events_path = args.model_dir.join("events").with_extension("json");

    info!("Reading events from {} ... ", events_path.to_str().unwrap());
    let reader = BufReader::new(File::open(events_path)?);
    let events: Vec<Event> = serde_json::from_reader(reader)?;

    let events_with_videos: Vec<Event> = events
        .into_iter()
        .filter(|e| e.mp4_video_link().is_some())
        .collect();

    info!(
        "Fetching {} events with video content, saving in {}",
        events_with_videos.len(),
        args.video_dir.to_str().unwrap()
    );
    let progress = progress_bar(events_with_videos.len() as u64);
    let mut video_paths = vec![];
    for event in events_with_videos {
        if let Some(url) = event.mp4_video_link() {
            let video_path = video_path(&args.video_dir, &url);
            if video_path.exists() {
                debug!("{:?} already downloaded, skipping", video_path);
            } else {
                fetch_video(&url, &video_path).await?;
            }
            video_paths.push(video_path);
        }
        progress.inc(1);
    }

    Ok(())
}

async fn fetch_video(url: &Url, video_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    debug!("fetching {} -> {:?}", url, video_path);
    let tmp_video_path = video_path.with_extension("tmp");
    debug!("using {:?} as tmp file", tmp_video_path);
    if tmp_video_path.exists() {
        debug!("removing existing tmp file");
        fs::remove_file(tmp_video_path.clone())?;
    }
    if video_path.exists() {
        debug!("removing existing video file");
        fs::remove_file(video_path.clone())?;
    }
    debug!("starting download");
    let command = async_process::Command::new("wget")
        .arg(format!(
            "--output-document={}",
            tmp_video_path.to_str().unwrap()
        ))
        .arg(url.to_string())
        .output();
    let output = command.await?;
    if output.status.success() {
        debug!(
            "download succeeded, renaming {:?} to {:?}",
            tmp_video_path, video_path
        );
        fs::rename(tmp_video_path, video_path)?;
        Ok(())
    } else {
        Err(format!("download command failed: {}", output.status).into())
    }
}

fn video_path(video_dir: &PathBuf, url: &Url) -> PathBuf {
    let url_path = PathBuf::from(url.path());
    video_dir.join(url_path.file_name().unwrap())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use url::Url;

    use crate::video_path;

    #[test]
    fn test_video_path() {
        let video_dir = PathBuf::from("/some/dir");
        let url = Url::parse("http://foo.com/foop/file.mp4").unwrap();
        let video_path = video_path(&video_dir, &url);
        assert_eq!(PathBuf::from("/some/dir/file.mp4"), video_path);
    }
}
