use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use clap::Parser;

use content::slide_index::SlideIndex;
use content::video_index::VideoIndex;

use embedding::model::SubjectEmbedding;
use embedding::openai_ada2::get_event_embedding;
use openai_dive::v1::api::Client;

use shared::cli::progress_bar;
use shared::model::{Event, EventArtefact, EventId};

use tracing::info;

/// Fetch Embeddings
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// model area, where input events are, and where embeddings will be written
    #[arg(long)]
    model_dir: PathBuf,

    /// include slide content at path
    #[arg(long)]
    include_slide_content: Option<PathBuf>,

    /// include video content at path
    #[arg(long)]
    include_video_content: Option<PathBuf>,

    /// whether to write out combined embeddings
    #[arg(long)]
    write_combined_embeddings: bool,

    /// whether to write out video-only embeddings
    #[arg(long)]
    write_video_embeddings: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv()?;
    let args = Args::parse();

    let api_key_name = "OPENAI_API_KEY";
    let api_key =
        dotenvy::var(api_key_name).unwrap_or_else(|_| panic!("{} is not set", api_key_name));

    let client = Client::new(api_key);

    info!("Reading events ...");
    let events = Event::from_model_area(&args.model_dir)?;

    let slide_index = if let Some(base_path) = args.include_slide_content {
        let index = SlideIndex::from_content_area(&base_path, &events)?;
        info!("Read {} events with slide content ", index.entries.len());
        index
    } else {
        SlideIndex::empty_index()
    };

    let video_index = if let Some(base_path) = args.include_video_content {
        let index = VideoIndex::from_content_area(&base_path)?;
        info!("Read {} events with video content ", index.entries.len());
        index
    } else {
        VideoIndex::empty_index()
    };

    if args.write_combined_embeddings {
        write_combined_embeddings(
            &args.model_dir,
            &events,
            &client,
            &slide_index,
            &video_index,
        )
        .await?;
    }

    if args.write_video_embeddings {
        write_video_embeddings(&args.model_dir, &events, &client, &video_index).await?;
    }

    Ok(())
}

async fn write_combined_embeddings(
    model_dir: &Path,
    events: &[Event],
    client: &Client,
    slide_index: &SlideIndex,
    video_index: &VideoIndex,
) -> Result<(), Box<dyn std::error::Error>> {
    let embedding_path = model_dir
        .join("openai_combined_embeddings")
        .with_extension("json");

    info!(
        "Looking up and writing combined embeddings to {} ... ",
        embedding_path.to_str().unwrap()
    );
    let mut embeddings = vec![];
    let progress = progress_bar(events.len() as u64);
    for event in events.into_iter() {
        let subject = EventArtefact::Combined {
            event_id: EventId(event.id),
        };
        let embedding = get_event_embedding(&client, &event, &slide_index, &video_index).await?;
        let subject_embedding = SubjectEmbedding::new(subject, embedding);
        embeddings.push(subject_embedding);
        progress.inc(1);
    }

    let embedding_file = File::create(embedding_path)?;
    let mut writer = BufWriter::new(embedding_file);
    serde_json::to_writer_pretty(&mut writer, &embeddings)?;
    writer.flush()?;

    Ok(())
}

async fn write_video_embeddings(
    model_dir: &Path,
    events: &[Event],
    client: &Client,
    video_index: &VideoIndex,
) -> Result<(), Box<dyn std::error::Error>> {
    let embedding_path = model_dir
        .join("openai_video_embeddings")
        .with_extension("json");

    info!(
        "Looking up and writing video embeddings to {} ... ",
        embedding_path.to_str().unwrap()
    );
    let mut embeddings = vec![];
    let progress = progress_bar(events.len() as u64);
    for event in events.into_iter() {
        if let Some(video_file) = video_index.video_file_for_event_id(event.id) {
            let subject = EventArtefact::Video {
                event_id: EventId(event.id),
                file: video_file,
            };
            let embedding =
                get_event_embedding(&client, &event, &SlideIndex::empty_index(), &video_index)
                    .await?;
            let subject_embedding = SubjectEmbedding::new(subject, embedding);
            embeddings.push(subject_embedding);
        }
        progress.inc(1);
    }

    let embedding_file = File::create(embedding_path)?;
    let mut writer = BufWriter::new(embedding_file);
    serde_json::to_writer_pretty(&mut writer, &embeddings)?;
    writer.flush()?;

    Ok(())
}
