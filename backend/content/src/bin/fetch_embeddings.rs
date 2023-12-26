use std::fs::File;

use clap::Parser;
use dotenv;
use openai_dive::v1::api::Client;
use openai_dive::v1::resources::embedding::EmbeddingParameters;

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
    let embedding_string = format!(
        "[{}]",
        result.data[0]
            .embedding
            .iter()
            .map(|f| f.to_string())
            .collect::<Vec<String>>()
            .join(",")
    );

    let mut csv = csv::Writer::from_writer(File::create(args.embedding_csv)?);
    csv.write_record(&["title", "embedding"])?;
    csv.write_record(&[title, &embedding_string])?;

    Ok(())
}
