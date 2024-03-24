use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use clap::Parser;
use embedding::model::SubjectEmbedding;
use shared::model::{Event, OpenAIEmbedding};
use tracing::info;

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
    let subject_embeddings: Vec<SubjectEmbedding> = convert(&events, &embeddings);

    info!("Writing embeddings to {:?} ... ", args.embeddings_out);
    let embedding_file = File::create(args.embeddings_out)?;
    let mut writer = BufWriter::new(embedding_file);
    serde_json::to_writer(&mut writer, &subject_embeddings)?;
    writer.flush()?;

    Ok(())
}

fn convert(_events: &[Event], _embeddings: &[OpenAIEmbedding]) -> Vec<SubjectEmbedding> {
    vec![]
}
