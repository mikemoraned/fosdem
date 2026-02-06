use std::path::PathBuf;

use chrono::Utc;
use clap::Parser;
use shared::{env::load_secret, inmemory_openai::InMemoryOpenAIQueryable, summary::load_summary};
use tracing::info;

#[derive(Parser, Debug)]
#[command(author, version, about = "Create a blog post summarizing current data")]
struct Args {
    /// Path to model directory
    #[arg(short, long)]
    model_dir: PathBuf,

    /// Path to blog posts directory
    #[arg(short, long)]
    blog_content_dir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let openai_api_key = load_secret("OPENAI_API_KEY")?;
    let queryable = InMemoryOpenAIQueryable::connect(&args.model_dir, &openai_api_key).await?;

    let summary = load_summary(&queryable).await?;

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let post_path = args.blog_content_dir.join(format!("{}.md", today));

    let mut content = String::new();
    content.push_str("---\n");
    content.push_str("title: Data Update\n");
    content.push_str("tags:\n");
    content.push_str("  - data\n");
    content.push_str("---\n\n");
    content.push_str("Updated event data.\n\n");
    content.push_str("| Year | Events | Presenters | Rooms | Tracks | Videos | Slides | Links |\n");
    content.push_str("|------|--------|--------|-------|--------|--------|--------|-------|\n");
    for (year, s) in &summary.by_year {
        let video_hours = s.video_duration.num_hours();
        content.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} (~{}h) | {} | {} |\n",
            year, s.events, s.people, s.rooms, s.tracks, s.videos, video_hours, s.slides, s.links
        ));
    }

    std::fs::write(&post_path, content)?;
    info!("Created post: {}", post_path.display());

    Ok(())
}
