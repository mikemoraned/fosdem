use std::fmt::format;
use std::fs::File;
use std::io::Cursor;

use clap::Parser;
use dotenv;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::embedding::{Embedding, EmbeddingParameters, EmbeddingResponse};

/// Fetch Embeddings
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// output csv path
    #[arg(short, long)]
    embedding_csv: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let args = Args::parse();

    let api_key_name = "OPENAI_API_KEY";
    let api_key = dotenv::var(api_key_name).expect(&format!("{} is not set", api_key_name));

    let client = Client::new(api_key);

    let title = "Creative Coding with TurtleStitch";
    let r#abstract = "<p>Turtlestitch is based on a browser-based educational programming language (Snap!) to generate patterns for embroidery machines. It is easy to use, requiring no prior knowledge in programming, yet powerful in creating nowels patterns for embroidery. It is useful for designers to experiment with generative aesthetics and precision embroidery as well as a tool for innovative workshops combining an introduction to programming with a haptic output.</p>";
    let input = format!("{} {}", title, r#abstract);

    let parameters = EmbeddingParameters {
        model: "text-embedding-ada-002".to_string(),
        input: input.to_string(),
        encoding_format: None,
        user: None,
    };

    let result = client.embeddings().create(parameters).await.unwrap();
    println!("{:#?}", result);
    let json = single_line_json(&result.data[0])?;
    println!("{:#?}", json);

    let mut csv = csv::Writer::from_writer(File::create(args.embedding_csv)?);
    csv.write_record(&["title", "raw"])?;
    csv.write_record(&[title, &json])?;

    Ok(())
}

fn single_line_json(embedding: &Embedding) -> Result<String, Box<dyn std::error::Error>> {
    use serde_jsonlines::WriteExt;
    let mut buf = vec![];
    let mut writer = Cursor::new(&mut buf);
    writer.write_json_lines(vec![embedding.embedding.clone()])?;
    let json_with_newlines = String::from_utf8(buf)?;
    let json_on_single_line = json_with_newlines.trim_end();
    Ok(String::from(json_on_single_line))
}
