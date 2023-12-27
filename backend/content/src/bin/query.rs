use clap::Parser;
use dotenv;
use sqlx::postgres::PgPoolOptions;

/// Run a query against a remote DB
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// host address of DB
    #[arg(long)]
    host: String,

    /// schema to use
    #[arg(long)]
    schema: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv()?;
    let args = Args::parse();

    let password_key_name = "DB_KEY";
    let password =
        dotenv::var(password_key_name).expect(&format!("{} is not set", password_key_name));

    let url = format!("postgres://postgres:{}@{}/postgres", password, args.host);
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&pool)
        .await?;

    assert_eq!(row.0, 150);

    Ok(())
}
