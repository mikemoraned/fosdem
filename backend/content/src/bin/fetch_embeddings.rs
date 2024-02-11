use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

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

    /// include slide content at path
    #[arg(long)]
    include_slide_content: Option<String>,

    /// output csv path
    #[arg(long)]
    embedding_csv: String,
}

#[derive(Debug, Deserialize)]
struct EventRecord {
    id: u32,
    title: String,
    track: String,
    r#abstract: String,
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

    let mut slide_content_for_event: HashMap<u32, String> = HashMap::new();
    if let Some(slides_content_path) = args.include_slide_content {
        println!("Fetching slide content from {} ... ", slides_content_path);
        let base_path = Path::new(&slides_content_path);
        let mut slide_content_count = 0;
        for event in events.iter() {
            let slide_content_path = base_path.join(event.id.to_string()).with_extension("txt");
            if slide_content_path.exists() {
                let mut file = File::open(slide_content_path)?;
                let mut slide_content = String::new();
                file.read_to_string(&mut slide_content)?;
                slide_content_for_event.insert(event.id, slide_content);
                slide_content_count += 1;
            }
        }
        println!("Read {} events with slide content ", slide_content_count);
    }

    println!(
        "Looking up and writing embeddings to {} ... ",
        args.embedding_csv
    );
    let mut embedding_writer = csv::Writer::from_writer(File::create(args.embedding_csv)?);
    embedding_writer.write_record(&["title", "embedding"])?;
    let progress = progress_bar(events.len() as u64);
    for event in events.iter() {
        let response = get_embedding(&client, &event, &slide_content_for_event).await?;
        let embedding = &response.data[0];
        embedding_writer.write_record(&[&event.title, &embedding_as_string(embedding)])?;
        progress.inc(1);
    }

    Ok(())
}

async fn get_embedding(
    client: &Client,
    event: &EventRecord,
    slide_content_for_event: &HashMap<u32, String>,
) -> Result<EmbeddingResponse, Box<dyn std::error::Error>> {
    let preferred_input = if let Some(slide_content) = slide_content_for_event.get(&event.id) {
        format!("{}\nSlides:{}", format_basic_input(event), slide_content)
    } else {
        format_basic_input(event)
    };
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

fn format_basic_input(event: &EventRecord) -> String {
    format!(
        "FOSDEM Conference Event 2024\nTitle: {}\nTrack: {}\nAbstract: {}",
        event.title, event.track, event.r#abstract
    )
}

fn trim_input(input: &String) -> String {
    use tiktoken_rs::cl100k_base;
    let max_tokens = 8192 - 100;
    let token_estimator = cl100k_base().unwrap();

    let tokens = token_estimator.split_by_token(&input, false).unwrap();
    let trimmed: Vec<_> = tokens.into_iter().take(max_tokens).collect();
    trimmed.join("")
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
