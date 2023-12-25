use clap::{arg, Parser};

/// Load all content from a Pentabarf file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input Pentabarf xml path
    #[arg(short, long)]
    pentabarf: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let xml = std::fs::read_to_string(args.pentabarf)?;
    let doc = roxmltree::Document::parse(&xml)?;
    println!("doc: {:?}", doc);

    Ok(())
}
