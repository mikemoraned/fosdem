use std::collections::HashMap;
use std::fs::{DirEntry, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use clap::Parser;
use openai_dive::v1::api::Client;

use openai_dive::v1::resources::embedding::{EmbeddingParameters, EmbeddingResponse};
use regex::Regex;
use shared::cli::progress_bar;
use shared::model::{Event, OpenAIEmbedding};

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
    dotenvy::dotenv()?;
    let args = Args::parse();

    let api_key_name = "OPENAI_API_KEY";
    let api_key =
        dotenvy::var(api_key_name).unwrap_or_else(|_| panic!("{} is not set", api_key_name));

    let client = Client::new(api_key);

    let events_path = args.model_dir.join("events").with_extension("json");

    print!("Reading events from {} ... ", events_path.to_str().unwrap());
    let reader = BufReader::new(File::open(events_path)?);
    let events: Vec<Event> = serde_json::from_reader(reader)?;
    println!("done ");

    let mut slide_content_for_event: HashMap<u32, String> = HashMap::new();
    if let Some(base_path) = args.include_slide_content {
        println!("Fetching slide content from {:?} ... ", base_path);
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

    let mut video_content_for_event: HashMap<u32, String> = HashMap::new();
    if let Some(base_path) = args.include_video_content {
        println!("Fetching video content from {:?} ... ", base_path);
        let mut video_content_count = 0;
        let dir_entries: Result<Vec<DirEntry>, _> = std::fs::read_dir(base_path)?.collect();
        let file_regex = Regex::new(r"fosdem-2024-(?<event_id>\d+)-")?;
        for entry in dir_entries? {
            let file_name = entry.file_name();
            if let Some(c) = file_regex.captures(file_name.to_str().unwrap()) {
                if let Some(m) = c.name("event_id") {
                    let event_id: u32 = m.as_str().parse().unwrap();
                    let mut file = File::open(entry.path())?;
                    let mut content = String::new();
                    file.read_to_string(&mut content)?;
                    video_content_for_event.insert(event_id, content);
                    video_content_count += 1;
                }
            }
        }
        println!("Read {} events with video content ", video_content_count);
    }

    let embedding_path = args.model_dir.join("embeddings").with_extension("json");

    println!(
        "Looking up and writing embeddings to {} ... ",
        embedding_path.to_str().unwrap()
    );
    let mut embeddings = vec![];
    let progress = progress_bar(events.len() as u64);
    for event in events.into_iter() {
        let response = get_embedding(&client, &event, &slide_content_for_event).await?;
        let embedding = OpenAIEmbedding {
            title: event.title,
            embedding: OpenAIEmbedding::embedding_from_response(&response),
        };
        embeddings.push(embedding);
        progress.inc(1);
    }

    let embedding_file = File::create(embedding_path)?;
    let mut writer = BufWriter::new(embedding_file);
    serde_json::to_writer(&mut writer, &embeddings)?;
    writer.flush()?;

    Ok(())
}

async fn get_embedding(
    client: &Client,
    event: &Event,
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

fn format_basic_input(event: &Event) -> String {
    let lines: Vec<String> = vec![
        "FOSDEM Conference Event 2024".into(),
        format!("Title: {}", event.title),
        format!("Track: {}", event.track),
        format!("Abstract: {}", event.r#abstract),
        format!(
            "Presenter: {}",
            event
                .presenters
                .iter()
                .map(|p| p.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    ];
    lines.join("\n")
}

fn trim_input(input: &str) -> String {
    use tiktoken_rs::cl100k_base;
    let max_tokens = 8192 - 100;
    let token_estimator = cl100k_base().unwrap();

    let tokens = token_estimator.split_by_token(input, false).unwrap();
    let trimmed: Vec<_> = tokens.into_iter().take(max_tokens).collect();
    trimmed.join("")
}
