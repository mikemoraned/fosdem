use clap::{arg, Parser};
use xmlserde::xml_deserialize_from_str;
use xmlserde_derives::XmlDeserialize;

/// Load all content from a Pentabarf file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input Pentabarf xml path
    #[arg(short, long)]
    pentabarf: String,
}

#[derive(XmlDeserialize, Default, Debug)]
#[xmlserde(root = b"schedule")]
struct Schedule {
    #[xmlserde(name = b"day", ty = "child")]
    days: Vec<Day>,
}

#[derive(XmlDeserialize, Default, Debug)]
struct Day {
    #[xmlserde(name = b"date", ty = "attr")]
    date: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let xml = std::fs::read_to_string(args.pentabarf)?;
    let schedule: Schedule = xml_deserialize_from_str(&xml)?;
    println!("schedule: {:?}", schedule);

    Ok(())
}
