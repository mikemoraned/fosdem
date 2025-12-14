use std::io::BufReader;

use std::{fs::File, path::PathBuf};

use clap::Parser;

use reqwest::StatusCode;
use shared::cli::progress_bar;
use shared::model::Event;
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

    /// skip checking those where no work is required
    #[arg(long)]
    skip_completed: bool,

    /// only verify whether the files exist
    #[arg(long)]
    verify_only: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let events_path = args.model_dir.join("events").with_extension("json");

    info!("Reading events from {} ... ", events_path.to_str().unwrap());
    let reader = BufReader::new(File::open(events_path)?);
    let events: Vec<Event> = serde_json::from_reader(reader)?;

    if args.verify_only {
        info!("Shall verify only");
    }

    let events_with_videos: Vec<Event> = events
        .into_iter()
        .filter(|e| e.mp4_video_link().is_some())
        .collect();

    let events_with_videos: Vec<Event> = if let Some(n) = args.offset {
        events_with_videos.into_iter().skip(n).collect()
    } else {
        events_with_videos
    };
    let events_with_videos: Vec<Event> = if let Some(n) = args.limit {
        events_with_videos.into_iter().take(n).collect()
    } else {
        events_with_videos
    };
    let events_with_videos: Vec<Event> = if args.skip_completed {
        info!(
            "Checking {} events with video content, to see if any work is required",
            events_with_videos.len()
        );
        events_with_videos
            .into_iter()
            .filter(|e| {
                if let Some(url) = e.mp4_video_link() {
                    let video_path = video_path(&args.video_dir, &url);
                    let audio_path = audio_path(&args.audio_dir, &video_path);
                    let wav_path = wav_path(&audio_path);
                    let webvtt_path = webvtt_path(&args.webvtt_dir, &wav_path);
                    let all_completed = video_path.exists()
                        && audio_path.exists()
                        && wav_path.exists()
                        && webvtt_path.exists();
                    !all_completed
                } else {
                    false
                }
            })
            .collect()
    } else {
        events_with_videos
    };
    let events_with_videos_total = events_with_videos.len();

    info!(
        "Fetching {} events with video content, saving in {}",
        events_with_videos.len(),
        args.video_dir.to_str().unwrap()
    );
    let progress = progress_bar(events_with_videos.len() as u64);
    let mut video_paths = vec![];
    let mut video_paths_missing = 0;
    for event in events_with_videos {
        if let Some(url) = event.mp4_video_link() {
            let video_path = video_path(&args.video_dir, &url);
            if video_path.exists() {
                debug!("{:?} already downloaded, skipping", video_path);
                progress.inc(1);
                video_paths.push(video_path);
            } else if args.verify_only {
                debug!("{:?} verify only, skipping", video_path);
                video_paths_missing += 1;
                video_paths.push(video_path);
            } else if url_reachable(&url).await? {
                fetch_video(&url, &video_path).await?;
                progress.inc(1);
                video_paths.push(video_path);
            }
        }
    }
    if args.verify_only {
        info!(
            "Video paths missing: {}/{}",
            video_paths_missing, events_with_videos_total
        );
    }

    info!(
        "Extracting audio from {} videos, saving in {}",
        video_paths.len(),
        args.audio_dir.to_str().unwrap()
    );
    let progress = progress_bar(video_paths.len() as u64);
    let mut audio_paths = vec![];
    let mut audio_paths_missing = 0;
    for video_path in video_paths {
        let audio_path = audio_path(&args.audio_dir, &video_path);
        if audio_path.exists() {
            debug!("{:?} already extracted, skipping", audio_path);
            progress.inc(1);
        } else if args.verify_only {
            debug!("{:?} verify only, skipping", audio_path);
            audio_paths_missing += 1;
        } else {
            extract_audio(&video_path, &audio_path).await?;
            progress.inc(1);
        }
        audio_paths.push(audio_path);
    }
    if args.verify_only {
        info!(
            "Audio paths missing: {}/{}",
            audio_paths_missing, events_with_videos_total
        );
    }

    info!(
        "Extracting WAV from {} audio files, saving in {}",
        audio_paths.len(),
        args.audio_dir.to_str().unwrap()
    );
    let progress = progress_bar(audio_paths.len() as u64);
    let mut wav_paths = vec![];
    let mut wav_paths_missing = 0;
    for audio_path in audio_paths {
        let wav_path = wav_path(&audio_path);
        if wav_path.exists() {
            debug!("{:?} already extracted, skipping", wav_path);
            progress.inc(1);
        } else if args.verify_only {
            debug!("{:?} verify only, skipping", wav_path);
            wav_paths_missing += 1;
        } else {
            extract_wav(&audio_path, &wav_path).await?;
            progress.inc(1);
        }
        wav_paths.push(wav_path);
    }
    if args.verify_only {
        info!(
            "WAV paths missing: {}/{}",
            wav_paths_missing, events_with_videos_total
        );
    }

    info!(
        "Extracting text from {} WAV files, saving in {}",
        wav_paths.len(),
        args.webvtt_dir.to_str().unwrap()
    );
    let progress = progress_bar(wav_paths.len() as u64);
    let mut webvtt_paths_missing = 0;
    for wav_path in wav_paths {
        let webvtt_path = webvtt_path(&args.webvtt_dir, &wav_path);
        if webvtt_path.exists() {
            debug!("{:?} already extracted, skipping", webvtt_path);
            progress.inc(1);
        } else if args.verify_only {
            debug!("{:?} verify only, skipping", webvtt_path);
            webvtt_paths_missing += 1;
        } else {
            extract_webvtt(&wav_path, &webvtt_path).await?;
            progress.inc(1);
        }
    }
    if args.verify_only {
        info!(
            "WEBVTT paths missing: {}/{}",
            webvtt_paths_missing, events_with_videos_total
        );
    }

    Ok(())
}

async fn url_reachable(url: &Url) -> Result<bool, Box<dyn std::error::Error>> {
    let status = reqwest::get(url.to_string()).await?.status();
    if status == StatusCode::OK {
        Ok(true)
    } else {
        warn!("failed to get {}, status: {:?}", url, status);
        Ok(false)
    }
}

async fn extract_webvtt(
    wav_path: &PathBuf,
    webvtt_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    debug!("extracting {:?} -> {:?}", wav_path, webvtt_path);
    let tmp_webvtt_path = wav_path.with_extension("wav.vtt");
    debug!("using {:?} as tmp file", tmp_webvtt_path);
    if tmp_webvtt_path.exists() {
        debug!("removing existing tmp file");
        fs::remove_file(tmp_webvtt_path.clone())?;
    }
    if webvtt_path.exists() {
        debug!("removing existing webvtt file");
        fs::remove_file(webvtt_path.clone())?;
    }
    let mut command = async_process::Command::new("/Users/mxm/Code/github/whisper.cpp/main");
    let command = command
        .arg("-m")
        .arg("/Users/mxm/Code/github/whisper.cpp/models/ggml-large-v3.bin")
        .arg("--output-vtt")
        .arg(wav_path.to_str().unwrap());
    debug!("starting webvtt extract using command: \'{:?}\'", command);
    let output = command.output().await?;
    if output.status.success() {
        debug!(
            "extract succeeded, copying {:?} to {:?}",
            tmp_webvtt_path, webvtt_path
        );
        fs::copy(tmp_webvtt_path, webvtt_path)?;
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

async fn extract_audio(
    video_path: &PathBuf,
    audio_path: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs;
    debug!("extracting {:?} -> {:?}", video_path, audio_path);
    let tmp_audio_path = audio_path.with_extension("tmp.mp4");
    debug!("using {:?} as tmp file", tmp_audio_path);
    if tmp_audio_path.exists() {
        debug!("removing existing tmp file");
        fs::remove_file(tmp_audio_path.clone())?;
    }
    if audio_path.exists() {
        debug!("removing existing audio file");
        fs::remove_file(audio_path.clone())?;
    }
    let mut command = async_process::Command::new("/opt/homebrew/bin/ffmpeg");
    let command = command
        .arg("-i")
        .arg(video_path.to_str().unwrap())
        .arg("-map")
        .arg("0:a")
        .arg("-acodec")
        .arg("copy")
        .arg(tmp_audio_path.to_str().unwrap());
    // .output();
    debug!("starting extract using command: \'{:?}\'", command);
    let output = command.output().await?;
    if output.status.success() {
        debug!(
            "extract succeeded, renaming {:?} to {:?}",
            tmp_audio_path, audio_path
        );
        fs::rename(tmp_audio_path, audio_path)?;
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

fn webvtt_path(webvtt_dir: &PathBuf, wav_path: &PathBuf) -> PathBuf {
    let file_stem = wav_path.file_stem().unwrap();
    webvtt_dir
        .join(file_stem.to_str().unwrap())
        .with_extension("vtt")
}

fn wav_path(audio_path: &PathBuf) -> PathBuf {
    audio_path.with_extension("wav")
}

fn audio_path(audio_dir: &PathBuf, video_path: &PathBuf) -> PathBuf {
    let file_stem = video_path.file_stem().unwrap();
    audio_dir
        .join(format!("{}_audioonly", file_stem.to_str().unwrap()))
        .with_extension("mp4")
}

fn video_path(video_dir: &PathBuf, url: &Url) -> PathBuf {
    let url_path = PathBuf::from(url.path());
    video_dir.join(url_path.file_name().unwrap())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use url::Url;

    use crate::{audio_path, video_path};

    #[test]
    fn test_video_path() {
        let video_dir = PathBuf::from("/some/dir");
        let url = Url::parse("http://foo.com/foop/file.mp4").unwrap();
        let video_path = video_path(&video_dir, &url);
        assert_eq!(PathBuf::from("/some/dir/file.mp4"), video_path);
    }

    #[test]
    fn test_audio_path() {
        let audio_dir = PathBuf::from("/some/dir");
        let video_path = PathBuf::from("/some/dir/file.mp4");
        let audio_path = audio_path(&audio_dir, &video_path);
        assert_eq!(PathBuf::from("/some/dir/file_audioonly.mp4"), audio_path);
    }
}
