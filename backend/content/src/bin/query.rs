use clap::Parser;
use content::openai::get_embedding;
use dotenv;
use log::info;
use openai_dive::v1::api::Client;
use pgvector::Vector;
use sqlx::{postgres::PgPoolOptions, Row};

/// Run a query against a remote DB
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// host address of DB
    #[arg(long)]
    host: String,

    /// query to search for
    #[arg(long)]
    query: String,
}

fn setup_logging_and_tracing() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logging_and_tracing()?;

    dotenv::dotenv()?;
    let args = Args::parse();

    let password_key_name = "DB_KEY";
    let password =
        dotenv::var(password_key_name).expect(&format!("{} is not set", password_key_name));

    info!("Connecting to DB");
    let url = format!("postgres://postgres:{}@{}/postgres", password, args.host);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    info!("Creating OpenAI Client");
    let api_key_name = "OPENAI_API_KEY";
    let api_key = dotenv::var(api_key_name).expect(&format!("{} is not set", api_key_name));

    let client = Client::new(api_key);

    info!("Getting embedding for query");
    let response = get_embedding(&client, &args.query).await?;
    let embedding = Vector::from(
        response.data[0]
            .embedding
            .clone()
            .into_iter()
            .map(|f| f as f32)
            .collect::<Vec<_>>(),
    );

    info!("Running Query");
    let sql = "
    SELECT id, title FROM embedding_1 
    ORDER BY embedding <-> ($1) LIMIT 5;
    ";
    let rows = sqlx::query(sql).bind(embedding).fetch_all(&pool).await?;
    for row in rows {
        let title: &str = row.try_get("title")?;
        println!("title: {}", title);
    }

    Ok(())
}
