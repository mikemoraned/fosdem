use clap::Parser;
use dotenvy;

use shared::queryable::Queryable;

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

    dotenvy::dotenv()?;
    let args = Args::parse();

    let password_key_name = "DB_KEY";
    let password =
        dotenvy::var(password_key_name).expect(&format!("{} is not set", password_key_name));

    let api_key_name = "OPENAI_API_KEY";
    let api_key = dotenvy::var(api_key_name).expect(&format!("{} is not set", api_key_name));

    let queryable = Queryable::connect(&args.host, &password, &api_key).await?;

    let items = queryable.search(&args.query, args.limit, false).await?;
    for item in items.iter() {
        println!(
            "title: {} (distance: {:.3})",
            item.event.title, item.distance
        );
        println!("url: {}", item.event.url);
        if args.r#abstract {
            println!("{}", item.event.r#abstract);
            println!();
        }
    }

    Ok(())
}
