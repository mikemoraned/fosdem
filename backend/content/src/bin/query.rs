use clap::Parser;
use dotenv;
use log::{debug, error, info, log_enabled, Level};
use sqlx::{postgres::PgPoolOptions, Row};

/// Run a query against a remote DB
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// host address of DB
    #[arg(long)]
    host: String,

    /// id of title to lookup
    #[arg(long)]
    id: i64,
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

    info!("Running Query");
    let sql = "
    SELECT id, title FROM embedding_1 
    WHERE id != $1
    ORDER BY embedding <-> (SELECT embedding FROM embedding_1 WHERE id = $2) LIMIT 5;
    ";
    let rows = sqlx::query(sql)
        .bind(args.id)
        .bind(args.id)
        .fetch_all(&pool)
        .await?;
    for row in rows {
        let title: &str = row.try_get("title")?;
        println!("title: {}", title);
    }

    Ok(())
}
