use clap::Parser;
use dotenvy;

use shared::{cli::progress_bar, env::load_secret, queryable::Queryable};

/// generate related items
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv()?;
    let args = Args::parse();

    let openai_api_key = load_secret("OPENAI_API_KEY");
    let db_host = load_secret("DB_HOST");
    let db_key = load_secret("DB_KEY");

    let queryable = Queryable::connect(&db_host, &db_key, &openai_api_key).await?;
    let titles = queryable.load_all_titles().await?;
    let progress = progress_bar(titles.len() as u64);
    for title in titles.iter() {
        queryable.find_related_events(&title, 5).await?;
        progress.inc(1);
    }

    Ok(())
}
