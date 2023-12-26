use std::fs::File;

use clap::Parser;
use dotenv;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::embedding::{Embedding, EmbeddingParameters, EmbeddingResponse};
use serde::Deserialize;

/// Fetch Embeddings
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input csv path
    #[arg(long)]
    event_csv: String,

    /// output csv path
    #[arg(long)]
    embedding_csv: String,
}

#[derive(Debug, Deserialize)]
struct EventRecord {
    title: String,
    r#abstract: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let args = Args::parse();

    let api_key_name = "OPENAI_API_KEY";
    let api_key = dotenv::var(api_key_name).expect(&format!("{} is not set", api_key_name));

    let client = Client::new(api_key);

    let mut event_reader = csv::Reader::from_reader(File::open(args.event_csv)?);
    let mut embedding_writer = csv::Writer::from_writer(File::create(args.embedding_csv)?);
    embedding_writer.write_record(&["title", "embedding"])?;
    for result in event_reader.deserialize() {
        let event: EventRecord = result?;
        let response = get_embedding(&client, &event).await?;
        let embedding = &response.data[0];
        embedding_writer.write_record(&[event.title, embedding_as_string(embedding)])?;
    }

    Ok(())
}

async fn get_embedding(
    client: &Client,
    event: &EventRecord,
) -> Result<EmbeddingResponse, Box<dyn std::error::Error>> {
    let input = format!("{} {}", event.title, event.r#abstract);

    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: input.to_string(),
        encoding_format: None,
        user: None,
    };

    let response = client.embeddings().create(parameters).await.unwrap();

    Ok(response)
}

fn embedding_as_string(embedding: &Embedding) -> String {
    embedding
        .embedding
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(",")
}
