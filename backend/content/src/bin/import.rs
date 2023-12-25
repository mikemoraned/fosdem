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
    #[xmlserde(name = b"room", ty = "child")]
    rooms: Vec<Room>,
}

#[derive(XmlDeserialize, Default, Debug)]
struct Room {
    #[xmlserde(name = b"name", ty = "attr")]
    name: String,
    #[xmlserde(name = b"event", ty = "child")]
    events: Vec<crate::Event>,
}

#[derive(XmlDeserialize, Default, Debug)]
struct Event {
    #[xmlserde(name = b"title", ty = "child")]
    title: Title,
    #[xmlserde(name = b"abstract", ty = "child")]
    r#abstract: Abstract,
}

#[derive(XmlDeserialize, Default, Debug)]
struct Abstract {
    #[xmlserde(ty = "text")]
    value: String,
}

#[derive(XmlDeserialize, Default, Debug)]
struct Title {
    #[xmlserde(ty = "text")]
    value: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let xml = std::fs::read_to_string(args.pentabarf)?;
    let schedule: Schedule = xml_deserialize_from_str(&xml)?;
    for day in schedule.days {
        println!("day: {}", day.date);
        for room in day.rooms {
            for event in room.events {
                println!("\tTitle: {}", event.title.value);
                println!("\t{}\n", event.r#abstract.value);
            }
        }
    }

    Ok(())
}
