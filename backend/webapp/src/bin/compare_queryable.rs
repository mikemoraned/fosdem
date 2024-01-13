use std::path::PathBuf;

use clap::Parser;
use shared::{
    env::load_secret, inmemory_openai::InMemoryOpenAIQueryable,
    postgres_openai::PostgresOpenAIQueryable, queryable::Queryable,
};
use tracing::info;

/// load two Queryable implementations, and compare them
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// path to directory where CSV files are kept
    #[arg(short, long)]
    csv_data_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv()?;
    let args = Args::parse();

    info!("Creating PostgresOpenAIQueryable");
    let openai_api_key = load_secret("OPENAI_API_KEY");
    let db_host = load_secret("DB_HOST");
    let db_key = load_secret("DB_KEY");
    let queryable1 = PostgresOpenAIQueryable::connect(&db_host, &db_key, &openai_api_key).await?;

    info!("Creating InMemoryOpenAIQueryable");
    let queryable2 = InMemoryOpenAIQueryable::connect(&args.csv_data_dir, &openai_api_key).await?;

    compare_events(&queryable1, &queryable2).await?;
    compare_related(&queryable1, &queryable2, 5).await?;

    Ok(())
}

async fn compare_events<T1, T2>(
    queryable1: &T1,
    queryable2: &T2,
) -> Result<(), Box<dyn std::error::Error>>
where
    T1: Queryable,
    T2: Queryable,
{
    let q1_events = queryable1.load_all_events().await?;
    let q2_events = queryable2.load_all_events().await?;
    assert_eq!(q1_events, q2_events);

    Ok(())
}

async fn compare_related<T1, T2>(
    queryable1: &T1,
    queryable2: &T2,
    limit: u8,
) -> Result<(), Box<dyn std::error::Error>>
where
    T1: Queryable,
    T2: Queryable,
{
    let q1_events = queryable1.load_all_events().await?;
    let q2_events = queryable2.load_all_events().await?;

    for q1_event in q1_events {
        let q1_related = queryable1
            .find_related_events(&q1_event.title, limit)
            .await?;
        let q2_related = queryable2
            .find_related_events(&q1_event.title, limit)
            .await?;
        assert_eq!(q1_related, q2_related);
    }

    Ok(())
}
