use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;

use content::slide_index::SlideIndex;
use content::video_index::VideoIndex;
use embedding::input::{format_basic_input, trim_input};
use embedding::model::{Embedding, OpenAIVector, SubjectEmbedding};
use openai_dive::v1::api::Client;

use openai_dive::v1::resources::embedding::{EmbeddingParameters, EmbeddingResponse};

use shared::cli::progress_bar;
use shared::model::{Event, EventArtefact, EventId};
use subtp::vtt::VttBlock;
use tracing::{debug, info};

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

    let embedding_path = args
        .model_dir
        .join("openai_combined_embeddings")
        .with_extension("json");

    info!(
        "Looking up and writing embeddings to {} ... ",
        embedding_path.to_str().unwrap()
    );
    let mut embeddings = vec![];
    let progress = progress_bar(events.len() as u64);
    for event in events.into_iter() {
        let response = get_embedding(&client, &event, &slide_index, &video_index).await?;
        let subject = EventArtefact::Combined {
            event_id: EventId(event.id),
        };
        let embedding = Embedding::OpenAIAda2 {
            vector: OpenAIVector::from(response.data[0].embedding.clone()),
        };
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

async fn get_embedding(
    client: &Client,
    event: &Event,
    slide_index: &SlideIndex,
    video_index: &VideoIndex,
) -> Result<EmbeddingResponse, Box<dyn std::error::Error>> {
    let mut preferred_input = String::new();
    use std::fmt::Write;

    writeln!(preferred_input, "{}", format_basic_input(event))?;
    if let Some(slide_content) = slide_index.entries.get(&event.id) {
        writeln!(preferred_input, "Slides:{}", slide_content)?;
    }
    if let Some(video_content) = video_index.webvtt_for_event_id(event.id) {
        let mut block_content: Vec<_> = video_content
            .blocks
            .iter()
            .map(|b| match b {
                VttBlock::Que(cue) => cue.payload.join("\n"),
                _ => "".into(),
            })
            .collect();
        block_content.dedup();
        debug!("[{}] blocks: {:?}", event.id, block_content);
        writeln!(preferred_input, "Subtitles:{}", block_content.join("\n"))?;
    }

    let trimmed_input = trim_input(&preferred_input);

    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: trimmed_input,
        encoding_format: None,
        user: None,
    };

    match client.embeddings().create(parameters).await {
        Ok(response) => Ok(response),
        Err(e) => Err(format!("[{}] error: \'{}\'", event.id, e).into()),
    }
}
