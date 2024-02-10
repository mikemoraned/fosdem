use std::fs::File;

use clap::Parser;
use dotenvy;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::embedding::{Embedding, EmbeddingParameters, EmbeddingResponse};
use serde::Deserialize;
use shared::cli::progress_bar;

/// Fetch Embeddings
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input csv path
    #[arg(long)]
    event_csv: String,

    /// parse and include slide content
    #[arg(long, action)]
    include_slides: bool,

    /// output csv path
    #[arg(long)]
    embedding_csv: String,
}

#[derive(Debug, Deserialize)]
struct EventRecord {
    title: String,
    track: String,
    r#abstract: String,
    slides: String,
}

struct EmbeddingInput {
    title: String,
    track: String,
    r#abstract: String,
    slide_content: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;
    let args = Args::parse();

    let api_key_name = "OPENAI_API_KEY";
    let api_key = dotenvy::var(api_key_name).expect(&format!("{} is not set", api_key_name));

    let client = Client::new(api_key);

    let mut event_reader = csv::Reader::from_reader(File::open(&args.event_csv)?);
    print!("Reading events from {} ... ", args.event_csv);
    let mut events = vec![];
    for result in event_reader.deserialize() {
        let event: EventRecord = result?;
        events.push(event);
    }
    println!("done ");

    let input = fetch_input(&events)?;

    println!(
        "Looking up and writing embeddings to {} ... ",
        args.embedding_csv
    );
    let mut embedding_writer = csv::Writer::from_writer(File::create(args.embedding_csv)?);
    embedding_writer.write_record(&["title", "embedding"])?;
    let progress = progress_bar(input.len() as u64);
    for event in input.iter() {
        let response = get_embedding(&client, &event).await?;
        let embedding = &response.data[0];
        embedding_writer.write_record(&[&event.title, &embedding_as_string(embedding)])?;
        progress.inc(1);
    }

    Ok(())
}

fn fetch_input(
    events: &Vec<EventRecord>,
) -> Result<Vec<EmbeddingInput>, Box<dyn std::error::Error>> {
    let mut inputs = vec![];
    for event in events {
        inputs.push(EmbeddingInput {
            title: event.title.clone(),
            track: event.track.clone(),
            r#abstract: event.r#abstract.clone(),
            slide_content: None,
        });
    }
    Ok(inputs)
}

async fn get_embedding(
    client: &Client,
    input: &EmbeddingInput,
) -> Result<EmbeddingResponse, Box<dyn std::error::Error>> {
    let input = format_input(input);

    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: input.to_string(),
        encoding_format: None,
        user: None,
    };

    let response = client.embeddings().create(parameters).await.unwrap();

    Ok(response)
}

fn format_input(input: &EmbeddingInput) -> String {
    match input.slide_content.clone() {
        None => {
            format!(
                "FOSDEM Conference Event 2024\nTitle: {}\nTrack: {}\nAbstract: {}",
                input.title, input.track, input.r#abstract
            )
        }
        Some(content) => {
            format!(
                "FOSDEM Conference Event 2024\nTitle: {}\nTrack: {}\nAbstract: {}\nSlide Content:{}",
                input.title, input.track, input.r#abstract, content
            )
        }
    }
}

fn embedding_as_string(embedding: &Embedding) -> String {
    format!(
        "[{}]",
        embedding
            .embedding
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join(",")
    )
}
