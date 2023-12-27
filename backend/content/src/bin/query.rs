use clap::Parser;
use dotenv;
use sqlx::{postgres::PgPoolOptions, Row};

/// Run a query against a remote DB
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// host address of DB
    #[arg(long)]
    host: String,
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

    let sql = "
    SELECT id, title FROM embedding_1 
    WHERE id != 20 
    ORDER BY embedding <-> (SELECT embedding FROM embedding_1 WHERE id = 20) LIMIT 5;
    ";
    let rows = sqlx::query(sql).fetch_all(&pool).await?;
    for row in rows {
        let title: &str = row.try_get("title")?;
        println!("title: {}", title);
    }

    Ok(())
}
