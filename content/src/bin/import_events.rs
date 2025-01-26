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

/// Convert all content from a Pentabarf file into a JSON file
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
                    guid: event.guid,
                    date: NaiveDate::parse_from_str(&day.date, "%Y-%m-%d").unwrap(),
                    start: NaiveTime::parse_from_str(&event.start.value, "%H:%M").unwrap(),
                    duration: parse_into_minutes(&event.duration.value)?,
                    room: room.name.clone(),
                    track: event.track.value,
                    title: event.title.value,
                    slug: event.slug.value,
                    url: Url::parse(&event.url.value)?,
                    r#abstract: event.r#abstract.value,
                    slides: slides(&event.attachments)?,
                    presenters: presenters(event.persons),
                    links: links(event.links)?,
                };
                model_events.push(model_event);
            }
        }
    }

    let event_path = args.model_dir.join("events").with_extension("json");
    let event_file = File::create(event_path)?;
    let mut writer = BufWriter::new(event_file);
    serde_json::to_writer_pretty(&mut writer, &model_events)?;
    writer.flush()?;

    Ok(())
}

fn links(
    links: content::pentabarf::Links,
) -> Result<Vec<shared::model::Link>, Box<dyn std::error::Error>> {
    links
        .links
        .into_iter()
        .map(|l| {
            Ok(shared::model::Link {
                url: Url::parse(&l.href)?,
                name: l.name,
            })
        })
        .collect()
}

fn presenters(persons: content::pentabarf::Persons) -> Vec<shared::model::Person> {
    persons
        .persons
        .into_iter()
        .map(|p| shared::model::Person {
            id: p.id,
            name: p.name,
        })
        .collect()
}

fn slides(event: &content::pentabarf::Attachments) -> Result<Vec<Url>, Box<dyn std::error::Error>> {
    let slide_attachments: Vec<&Attachment> = event
        .attachments
        .iter()
        .filter(|a| a.r#type == "slides")
        .collect();
    let mut slides = vec![];
    for slide_attachment in slide_attachments {
        slides.push(Url::parse(&slide_attachment.href)?);
    }
    Ok(slides)
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
        for example in ["foop", "24:00"] {
            assert!(parse_into_minutes(example).is_err());
        }
    }
}
