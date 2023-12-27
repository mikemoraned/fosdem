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

    /// how many to show
    #[arg(long)]
    limit: u8,

    /// whether to show abstract
    #[arg(long)]
    r#abstract: bool,
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
    SELECT ev.title, ev.abstract, em.embedding <-> ($1) AS distance
    FROM embedding_1 em JOIN events_1 ev ON ev.title = em.title
    ORDER BY em.embedding <-> ($1) LIMIT $2;
    ";
    let rows = sqlx::query(sql)
        .bind(embedding)
        .bind(args.limit as i32)
        .fetch_all(&pool)
        .await?;
    for row in rows {
        let title: &str = row.try_get("title")?;
        let distance: f64 = row.try_get("distance")?;
        let r#abstract: &str = row.try_get("abstract")?;
        println!("title: {} (distance: {:.3})", title, distance);
        if args.r#abstract {
            println!("{}", r#abstract);
            println!();
        }
    }

    Ok(())
}
