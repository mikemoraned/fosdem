use std::io::BufReader;

use std::time::Duration;
use std::{fs::File, path::PathBuf};

use clap::Parser;

use indicatif::{MultiProgress, ProgressBar};
use shared::cli::progress_bar;
use shared::model::Event;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::task::JoinSet;
use tracing::{debug, info, warn};

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
    Aborted,
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
        .map(|url| VideoDownload::Command(url.clone(), video_path(&args.video_dir, &url)))
        .collect();

    let pending_downloads = subset(pending_downloads, args.offset, args.limit);
    let total_pending_downloads = pending_downloads.len();

    let mut join_set = JoinSet::new();
    let (video_download_tx, video_download_rx) =
        mpsc::channel::<VideoDownload>(total_pending_downloads + 1);
    let (audio_extraction_tx, audio_extraction_rx) =
        mpsc::channel::<AudioExtraction>(total_pending_downloads + 1);

    let multi_progress = MultiProgress::new();

    info!(
        "Fetching {} events with video content, saving in {}",
        pending_downloads.len(),
        args.video_dir.to_str().unwrap()
    );
    for pending_download in pending_downloads {
        video_download_tx.send(pending_download).await?;
    }
    video_download_tx.send(VideoDownload::End).await?;
    join_set.spawn(download_video_stage(
        video_download_rx,
        audio_extraction_tx,
        multi_progress.add(progress_bar(total_pending_downloads as u64)),
        args.audio_dir.clone(),
    ));

    info!(
        "Extracting audio from videos, saving in {}",
        args.audio_dir.to_str().unwrap()
    );
    join_set.spawn(extraction_stage(
        audio_extraction_rx,
        multi_progress.add(progress_bar(total_pending_downloads as u64)),
    ));

    while let Some(result) = join_set.join_next().await {
        let stage_result = result?;
        info!("{}", stage_result?);
    }

    Ok(())
}

async fn download_video_stage(
    mut video_download_rx: Receiver<VideoDownload>,
    audio_extraction_tx: Sender<AudioExtraction>,
    progress: ProgressBar,
    audio_dir: PathBuf,
) -> Result<String, String> {
    debug!("download stage starting");
    progress.enable_steady_tick(Duration::from_secs(1));
    while let Some(pending_download) = video_download_rx.recv().await {
        use VideoDownload::*;

        match pending_download {
            Command(url, video_path) => {
                debug!("downloading {}", url);
                if video_path.exists() {
                    debug!("{:?} already downloaded, skipping", video_path);
                    audio_extraction_tx
                        .send(AudioExtraction::Command(
                            video_path.clone(),
                            audio_path(&audio_dir, &video_path),
                        ))
                        .await
                        .map_err(|e| format!("error sending: {}", e))?;
                } else {
                    match download_video(&url, &video_path).await {
                        Ok(_) => {
                            audio_extraction_tx
                                .send(AudioExtraction::Command(
                                    video_path.clone(),
                                    audio_path(&audio_dir, &video_path),
                                ))
                                .await
                                .map_err(|e| format!("error sending: {}", e))?;
                        }
                        Err(e) => {
                            warn!("download of {} failed, {}", url, e);
                            audio_extraction_tx
                                .send(AudioExtraction::Aborted)
                                .await
                                .map_err(|e| format!("error sending: {}", e))?;
                        }
                    }
                }

                progress.inc(1);
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

fn video_path(video_dir: &PathBuf, url: &Url) -> PathBuf {
    let url_path = PathBuf::from(url.path());
    video_dir.join(url_path.file_name().unwrap())
}

fn audio_path(audio_dir: &PathBuf, video_path: &PathBuf) -> PathBuf {
    let file_stem = video_path.file_stem().unwrap();
    audio_dir
        .join(format!("{}_audioonly", file_stem.to_str().unwrap()))
        .with_extension("mp4")
}

async fn download_video(url: &Url, video_path: &PathBuf) -> Result<(), String> {
    debug!("fetching {} -> {:?}", url, video_path);

    let tmp_video_path = video_path.with_extension("tmp");
    let tmp_file = TempFile::create(video_path.clone(), tmp_video_path.clone())
        .map_err(|e| format!("{}", e))?;
    debug!("starting download");
    let command = download_video_command(url, &tmp_video_path);
    let output = command.await.map_err(|e| format!("{}", e))?;
    if output.status.success() {
        tmp_file.commit().map_err(|e| format!("{}", e))?;
        Ok(())
    } else {
        tmp_file.abort().map_err(|e| format!("{}", e))?;
        Err(format!("download command failed: {}", output.status).into())
    }
}

struct TempFile {
    real_path: PathBuf,
    tmp_path: PathBuf,
}

impl TempFile {
    pub fn create(
        real_path: PathBuf,
        tmp_path: PathBuf,
    ) -> Result<TempFile, Box<dyn std::error::Error>> {
        debug!("using {:?} as tmp file", tmp_path);

        let temp_file = TempFile {
            real_path,
            tmp_path,
        };

        temp_file.cleanup_tmp_file()?;
        temp_file.cleanup_real_file()?;

        Ok(temp_file)
    }

    pub fn commit(self) -> Result<(), Box<dyn std::error::Error>> {
        debug!(
            "committing, renaming {:?} to {:?}",
            self.tmp_path, self.real_path
        );
        std::fs::rename(self.tmp_path, self.real_path)?;
        Ok(())
    }

    pub fn abort(self) -> Result<(), Box<dyn std::error::Error>> {
        self.cleanup_tmp_file()?;

        Ok(())
    }

    fn cleanup_tmp_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.tmp_path.exists() {
            debug!("removing tmp file");
            std::fs::remove_file(self.tmp_path.clone())?;
        }
        Ok(())
    }

    fn cleanup_real_file(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.real_path.exists() {
            debug!("removing real file");
            std::fs::remove_file(self.real_path.clone())?;
        }
        Ok(())
    }
}

fn download_video_command(
    url: &Url,
    video_path: &PathBuf,
) -> impl futures::Future<Output = futures::io::Result<async_process::Output>> {
    async_process::Command::new("wget")
        .arg(format!(
            "--output-document={}",
            video_path.to_str().unwrap()
        ))
        .arg(url.to_string())
        .output()
}

async fn extraction_stage(
    mut audio_extraction_rx: Receiver<AudioExtraction>,
    progress: ProgressBar,
) -> Result<String, String> {
    debug!("extraction stage starting");
    progress.enable_steady_tick(Duration::from_secs(1));
    while let Some(audio_extraction) = audio_extraction_rx.recv().await {
        use AudioExtraction::*;

        match audio_extraction {
            Command(from, to) => {
                debug!("extracting {:?} -> {:?}", from, to);
                tokio::time::sleep(Duration::from_secs(1)).await;
                progress.inc(1);
            }
            Aborted => {
                progress.inc(1);
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
