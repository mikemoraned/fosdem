use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use chrono::{NaiveDate, NaiveTime, Timelike};
use clap::{arg, Parser};
use content::pentabarf::{Attachment, Schedule};
use shared::model::Event;
use url::Url;
use xmlserde::xml_deserialize_from_str;

/// Convert all content from a Pentabarf file into a CSV
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// input Pentabarf xml path
    #[arg(short, long)]
    pentabarf: String,

    /// output model directory
    #[arg(short, long)]
    model_dir: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let xml = std::fs::read_to_string(args.pentabarf)?;
    let schedule: Schedule = xml_deserialize_from_str(&xml)?;
    let mut model_events = vec![];
    for day in schedule.days {
        for room in day.rooms {
            for event in room.events {
                let model_event = Event {
                    id: event.id,
                    date: NaiveDate::parse_from_str(&day.date, "%Y-%m-%d").unwrap(),
                    start: NaiveTime::parse_from_str(&event.start.value, "%H:%M").unwrap(),
                    duration: parse_into_minutes(&event.duration.value)?,
                    room: room.name.clone(),
                    track: event.track.value,
                    title: event.title.value,
                    slug: event.slug.value,
                    url: Url::parse(&event.url.value)?,
                    r#abstract: event.r#abstract.value,
                    slides: slides(&event.attachments),
                };
                model_events.push(model_event);
            }
        }
    }

    let event_path = args.model_dir.join("events").with_extension("json");
    let event_file = File::create(event_path)?;
    let mut writer = BufWriter::new(event_file);
    serde_json::to_writer(&mut writer, &model_events)?;
    writer.flush()?;

    Ok(())
}

fn slides(event: &content::pentabarf::Attachments) -> String {
    let slides: Vec<&Attachment> = event
        .attachments
        .iter()
        .filter(|a| a.r#type == "slides")
        .collect();
    if slides.len() >= 1 {
        slides[0].href.clone()
    } else {
        "".into()
    }
}

fn parse_into_minutes(value: &str) -> Result<u32, Box<dyn std::error::Error>> {
    let time = NaiveTime::parse_from_str(value, "%H:%M")?;
    Ok((time.hour() * 60) + time.minute())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_into_minutes_min_value() {
        assert_eq!(0, parse_into_minutes("00:00").unwrap());
    }

    #[test]
    fn test_parse_into_minutes_hour_and_minute_detail() {
        assert_eq!((2 * 60) + 22, parse_into_minutes("02:22").unwrap());
    }

    #[test]
    fn test_parse_into_minutes_max_value() {
        assert_eq!((23 * 60) + 59, parse_into_minutes("23:59").unwrap());
    }

    #[test]
    fn test_parse_into_minutes_invalid_values() {
        for example in vec!["foop", "24:00"] {
            assert!(parse_into_minutes(example).is_err());
        }
    }
}
