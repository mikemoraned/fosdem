use std::fs::File;

use clap::{arg, Parser};
use content::pentabarf::Schedule;
use xmlserde::xml_deserialize_from_str;

/// Convert all content from a Pentabarf file into a CSV
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input Pentabarf xml path
    #[arg(short, long)]
    pentabarf: String,

    /// output csv path
    #[arg(short, long)]
    csv: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let xml = std::fs::read_to_string(args.pentabarf)?;
    let mut csv = csv::Writer::from_writer(File::create(args.csv)?);
    let schedule: Schedule = xml_deserialize_from_str(&xml)?;
    csv.write_record(&["id", "title", "slug", "abstract"])?;
    for day in schedule.days {
        for room in day.rooms {
            for event in room.events {
                csv.write_record(&[
                    event.id.to_string(),
                    event.title.value,
                    event.slug.value,
                    event.r#abstract.value,
                ])?;
            }
        }
    }

    Ok(())
}
