use std::io::BufReader;

use std::time::Duration;
use std::{fs::File, path::PathBuf};

use clap::Parser;

use content::temp_file::TempFile;
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

#[derive(Debug)]
enum WAVExtraction {
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
    let (wav_extraction_tx, wav_extraction_rx) =
        mpsc::channel::<WAVExtraction>(total_pending_downloads + 1);

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
    join_set.spawn(audio_extraction_stage(
        audio_extraction_rx,
        wav_extraction_tx,
        multi_progress.add(progress_bar(total_pending_downloads as u64)),
    ));

    info!(
        "Extracting WAV from audio files, saving in {}",
        args.audio_dir.to_str().unwrap()
    );
    join_set.spawn(wav_extraction_stage(
        wav_extraction_rx,
        multi_progress.add(progress_bar(total_pending_downloads as u64)),
    ));

    while let Some(result) = join_set.join_next().await {
        let stage_result = result?;
        info!("{}", stage_result?);
    }

    Ok(())
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

fn wav_path(audio_path: &PathBuf) -> PathBuf {
    audio_path.with_extension("wav")
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
                let output = if video_path.exists() {
                    debug!("{:?} already downloaded, skipping", video_path);
                    AudioExtraction::Command(
                        video_path.clone(),
                        audio_path(&audio_dir, &video_path),
                    )
                } else {
                    match download_video(&url, &video_path).await {
                        Ok(_) => AudioExtraction::Command(
                            video_path.clone(),
                            audio_path(&audio_dir, &video_path),
                        ),
                        Err(e) => {
                            warn!("download of {} failed, {}", url, e);
                            AudioExtraction::Aborted
                        }
                    }
                };
                audio_extraction_tx
                    .send(output)
                    .await
                    .map_err(|e| format!("error sending: {}", e))?;

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

async fn audio_extraction_stage(
    mut audio_extraction_rx: Receiver<AudioExtraction>,
    wav_extraction_tx: Sender<WAVExtraction>,
    progress: ProgressBar,
) -> Result<String, String> {
    debug!("audio extraction stage starting");
    progress.enable_steady_tick(Duration::from_secs(1));
    while let Some(audio_extraction) = audio_extraction_rx.recv().await {
        use AudioExtraction::*;

        match audio_extraction {
            Command(video_path, audio_path) => {
                let output = if audio_path.exists() {
                    debug!("{:?} already extracted, skipping", audio_path);
                    WAVExtraction::Command(audio_path.clone(), wav_path(&audio_path))
                } else {
                    match extract_audio(&video_path, &audio_path).await {
                        Ok(_) => WAVExtraction::Command(audio_path.clone(), wav_path(&audio_path)),
                        Err(e) => {
                            warn!("extract of audio from {:?} failed, {}", video_path, e);
                            WAVExtraction::Aborted
                        }
                    }
                };
                wav_extraction_tx
                    .send(output)
                    .await
                    .map_err(|e| format!("error sending: {}", e))?;

                progress.inc(1);
            }
            Aborted => {
                wav_extraction_tx
                    .send(WAVExtraction::Aborted)
                    .await
                    .map_err(|e| format!("error sending: {}", e))?;

                progress.inc(1);
            }
            End => {
                wav_extraction_tx
                    .send(WAVExtraction::End)
                    .await
                    .map_err(|e| format!("error sending: {}", e))?;
                debug!("finished extraction");
                break;
            }
        }
    }

    Ok("audio extraction stage completed".into())
}

async fn extract_audio(video_path: &PathBuf, audio_path: &PathBuf) -> Result<(), String> {
    debug!("extracting {:?} -> {:?}", video_path, audio_path);
    let tmp_audio_path = audio_path.with_extension("tmp.mp4");
    let tmp_file = TempFile::create(audio_path.clone(), tmp_audio_path.clone())
        .map_err(|e| format!("{}", e))?;
    let command = extract_audio_command(&video_path, &tmp_audio_path);
    let output = command.await.map_err(|e| format!("{}", e))?;
    if output.status.success() {
        tmp_file.commit().map_err(|e| format!("{}", e))?;
        Ok(())
    } else {
        tmp_file.abort().map_err(|e| format!("{}", e))?;
        Err(format!(
            "extract command failed: {}, stdout: {}, stderr: {}",
            output.status,
            String::from_utf8(output.stdout).map_err(|e| format!("{}", e))?,
            String::from_utf8(output.stderr).map_err(|e| format!("{}", e))?
        )
        .into())
    }
}

fn extract_audio_command(
    video_path: &PathBuf,
    audio_path: &PathBuf,
) -> impl futures::Future<Output = futures::io::Result<async_process::Output>> {
    let mut command = async_process::Command::new("/opt/homebrew/bin/ffmpeg");
    command
        .arg("-i")
        .arg(video_path.to_str().unwrap())
        .arg("-map")
        .arg("0:a")
        .arg("-acodec")
        .arg("copy")
        .arg(audio_path.to_str().unwrap())
        .output()
}

async fn wav_extraction_stage(
    mut wav_extraction_rx: Receiver<WAVExtraction>,
    progress: ProgressBar,
) -> Result<String, String> {
    debug!("wav extraction stage starting");
    progress.enable_steady_tick(Duration::from_secs(1));
    while let Some(wav_extraction) = wav_extraction_rx.recv().await {
        use WAVExtraction::*;

        match wav_extraction {
            Command(audio_path, wav_path) => {
                if wav_path.exists() {
                    debug!("{:?} wav already extracted, skipping", wav_path);
                } else {
                    match extract_wav(&audio_path, &wav_path).await {
                        Ok(_) => (),
                        Err(e) => {
                            warn!("extract of wav from {:?} failed, {}", wav_path, e);
                        }
                    }
                };

                progress.inc(1);
            }
            Aborted => {
                progress.inc(1);
            }
            End => {
                debug!("finished wav extraction");
                break;
            }
        }
    }

    Ok("wav extraction stage completed".into())
}

async fn extract_wav(
    audio_path: &PathBuf,
    wav_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    debug!("extracting {:?} -> {:?}", audio_path, wav_path);
    let tmp_wav_path = audio_path.with_extension("tmp.wav");
    debug!("using {:?} as tmp file", tmp_wav_path);
    if tmp_wav_path.exists() {
        debug!("removing existing tmp file");
        fs::remove_file(tmp_wav_path.clone())?;
    }
    if wav_path.exists() {
        debug!("removing existing wav file");
        fs::remove_file(wav_path.clone())?;
    }
    let mut command = async_process::Command::new("/opt/homebrew/bin/ffmpeg");
    let command = command
        .arg("-i")
        .arg(audio_path.to_str().unwrap())
        .arg("-ar")
        .arg("16000")
        .arg("-ac")
        .arg("1")
        .arg("-c:a")
        .arg("pcm_s16le")
        .arg(tmp_wav_path.to_str().unwrap());
    debug!("starting extract using command: \'{:?}\'", command);
    let output = command.output().await?;
    if output.status.success() {
        debug!(
            "extract succeeded, renaming {:?} to {:?}",
            tmp_wav_path, wav_path
        );
        fs::rename(tmp_wav_path, wav_path)?;
        Ok(())
    } else {
        Err(format!(
            "extract command failed: {}, stdout: {}, stderr: {}",
            output.status,
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?
        )
        .into())
    }
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
