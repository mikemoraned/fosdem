use clap::{arg, Parser};
use sxd_document::parser;
use sxd_xpath::{evaluate_xpath, nodeset, Value};

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
    let package = parser::parse(&xml)?;
    let document = package.as_document();
    if let Value::Nodeset(days) = evaluate_xpath(&document, "/schedule/day")? {
        for day in days {
            if let nodeset::Node::Element(e) = day {
                println!("day: {:?}", e.attribute("date").unwrap().value());
            }
        }
    }

    Ok(())
}
