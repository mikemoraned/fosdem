use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;
use embedding::model::{Embedding, EventArtefact, EventId, SubjectEmbedding};
use shared::model::{Event, OpenAIEmbedding};
use tracing::{debug, info, warn};

/// Fetch Embeddings
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// model area, where input events are
    #[arg(long)]
    model_dir: PathBuf,

    /// embeddings in
    #[arg(long)]
    embeddings_in: PathBuf,

    /// embeddings out
    #[arg(long)]
    embeddings_out: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    info!("Loading embeddings data from {:?}", args.embeddings_in);
    let reader = BufReader::new(File::open(args.embeddings_in)?);
    let embeddings: Vec<OpenAIEmbedding> = serde_json::from_reader(reader)?;

    let events_path = args.model_dir.join("events").with_extension("json");
    info!("Loading events data from {:?}", events_path);
    let reader = BufReader::new(File::open(events_path)?);
    let events: Vec<Event> = serde_json::from_reader(reader)?;

    info!("Converting {} embeddings", embeddings.len());
    let subject_embeddings: Vec<SubjectEmbedding> = convert(&events, &embeddings)?;

    info!("Writing embeddings to {:?} ... ", args.embeddings_out);
    let embedding_file = File::create(args.embeddings_out)?;
    let mut writer = BufWriter::new(embedding_file);
    serde_json::to_writer_pretty(&mut writer, &subject_embeddings)?;
    writer.flush()?;

    Ok(())
}

fn convert(
    events: &[Event],
    embeddings: &[OpenAIEmbedding],
) -> Result<Vec<SubjectEmbedding>, Box<dyn std::error::Error>> {
    let mut title_index: HashMap<String, Vec<&Event>> = HashMap::new();
    for event in events {
        if let Some(events) = title_index.get_mut(&event.title) {
            warn!(
                "duplicate title key found: '{}' already mapped to {}",
                event.title, event.id
            );
            events.push(event);
        } else {
            title_index.insert(event.title.clone(), vec![event]);
        }
    }

    let mut converted = Vec::new();
    for embedding in embeddings {
        let title = &embedding.title;
        if let Some(events) = title_index.get_mut(title) {
            if let Some(event) = events.pop() {
                let event_id = EventId(event.id);
                debug!(
                    "Mapping embeddings with title: '{}' to event id {:?}",
                    embedding.title, event_id
                );

                let subject = EventArtefact::Combined {
                    event_id: event_id.clone(),
                };
                let converted_embedding = Embedding::OpenAIAda2 {
                    vector: embedding.embedding.clone(),
                };
                converted.push(SubjectEmbedding::new(subject, converted_embedding));
            } else {
                return Err(format!("no more events to map to for title {}", title).into());
            }
        } else {
            return Err(format!("Could not find event with title {}", title).into());
        }
    }

    Ok(converted)
}
