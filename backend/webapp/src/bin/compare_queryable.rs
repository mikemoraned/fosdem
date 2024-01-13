use shared::{
    env::load_secret, inmemory_openai::InMemoryOpenAIQueryable,
    postgres_openai::PostgresOpenAIQueryable, queryable::Queryable,
};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv()?;

    info!("Creating PostgresOpenAIQueryable");
    let openai_api_key = load_secret("OPENAI_API_KEY");
    let db_host = load_secret("DB_HOST");
    let db_key = load_secret("DB_KEY");
    let queryable1 = PostgresOpenAIQueryable::connect(&db_host, &db_key, &openai_api_key).await?;

    info!("Creating InMemoryOpenAIQueryable");
    let queryable2 = InMemoryOpenAIQueryable::connect(&openai_api_key).await?;

    compare_events(&queryable1, &queryable2);

    Ok(())
}

fn compare_events<T1, T2>(queryable1: &T1, queryable2: &T2)
where
    T1: Queryable,
    T2: Queryable,
{
    todo!()
}
