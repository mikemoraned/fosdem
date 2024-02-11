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
    let max_tokens = 8192;
    let input = if let Some(slide_content) = slide_content_for_event.get(&event.id) {
        append_slide_content(&format_basic_input(event), slide_content, max_tokens, 3)
    } else {
        format_basic_input(event)
    };

    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: input.to_string(),
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

fn append_slide_content(
    existing_content: &String,
    slide_content: &String,
    max_tokens: usize,
    tokens_per_word_estimate: usize,
) -> String {
    let existing_content_split: Vec<_> = existing_content.split(" ").collect();
    let slide_content_split: Vec<_> = slide_content.split(" ").collect();

    let existing_content_estimate = existing_content_split.len() * tokens_per_word_estimate;
    let slide_content_estimate = slide_content_split.len() * tokens_per_word_estimate;

    if existing_content_estimate + slide_content_estimate <= max_tokens {
        return format!("{}\n{}", existing_content.clone(), slide_content.clone());
    } else {
        let max_additional = max_tokens - existing_content_estimate;
        if max_additional > 0 {
            let minimized_slide_content_tokens: Vec<_> = slide_content_split
                .into_iter()
                .take(max_additional / tokens_per_word_estimate)
                .collect();
            return format!(
                "{}\n{}",
                existing_content.clone(),
                minimized_slide_content_tokens.join(" ")
            );
        } else {
            return existing_content.clone();
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

#[cfg(test)]
mod test {
    use crate::append_slide_content;

    #[test]
    fn test_append_slide_content_under_limit() {
        let existing = "aa bb cc".to_string();
        let extra = "xx yy".to_string();
        let max_tokens = 6;
        let actual = append_slide_content(&existing, &extra, max_tokens, 1);
        assert_eq!("aa bb cc\nxx yy", actual);
    }

    #[test]
    fn test_append_slide_content_needs_truncated() {
        let existing = "aa bb cc".to_string();
        let extra = "xx yy".to_string();
        let max_tokens = 4;
        let actual = append_slide_content(&existing, &extra, max_tokens, 1);
        assert_eq!("aa bb cc\nxx", actual);
    }

    #[test]
    fn test_append_slide_content_cannot_add_anymore() {
        let existing = "aa bb cc".to_string();
        let extra = "xx yy".to_string();
        let max_tokens = 3;
        let actual = append_slide_content(&existing, &extra, max_tokens, 1);
        assert_eq!(existing, actual);
    }
}
