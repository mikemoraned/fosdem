use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
    vec,
};

use chrono::{NaiveDate, NaiveTime, Timelike};
use clap::Parser;
use content::pentabarf::{Attachment, Schedule};
use shared::model::{self, Event};
use tracing::{debug, info, warn};
use url::Url;
use xmlserde::xml_deserialize_from_str;

/// Convert all content from a Pentabarf file into a JSON file
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// dir where Pentabarf xml files are located
    #[arg(short, long)]
    pentabarf_dir: String,

    /// years to import
    #[arg(short, long, value_delimiter = ' ')]
    years: Vec<u32>,

    /// output model directory
    #[arg(short, long)]
    model_dir: PathBuf,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let mut model_events = vec![];
    for year in args.years.iter() {
        let pentabarf_path = PathBuf::from(&args.pentabarf_dir).join(format!("{}.xml", year));
        info!("Importing Pentabarf file from {:?}", pentabarf_path);
        let mut events_added_count = 0;
        let xml = std::fs::read_to_string(pentabarf_path.clone())?;
        let schedule: Schedule = xml_deserialize_from_str(&xml)?;
        debug!("{} days of content to read", schedule.days.len());
        for day in schedule.days {
            for room in day.rooms {
                for event in room.events {
                    let mut model_event = Event {
                        id: model::EventId::new(*year, event.id),
                        year: *year,
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
                        presenters: presenters(*year, event.persons),
                        links: links(event.links)?,
                    };
                    apply_fixups(&mut model_event, *year)?;
                    model_events.push(model_event);
                    events_added_count += 1;
                }
            }
        }
        info!("Imported {events_added_count} events from {pentabarf_path:?}");
        if events_added_count == 0 {
            warn!("did not import any events from {pentabarf_path:?}");
        }
    }

    let event_path = args.model_dir.join("events").with_extension("json");
    let event_file = File::create(event_path)?;
    let mut writer = BufWriter::new(event_file);
    serde_json::to_writer_pretty(&mut writer, &model_events)?;
    writer.flush()?;

    Ok(())
}

fn apply_fixups(event: &mut Event, year: u32) -> Result<(), Box<dyn std::error::Error>> {
    // Fixup URL year segment if needed
    if let Some(segments) = event.url.path_segments() {
        let segments: Vec<&str> = segments.collect();
        if !segments.is_empty() {
            let expected_year_segment = format!("{}", year);
            if segments[0] != expected_year_segment {
                let mut new_segments = segments.clone();
                new_segments[0] = &expected_year_segment;
                let mut new_url = event.url.clone();
                new_url
                    .path_segments_mut()
                    .map_err(|_| "Failed to get mutable path segments")?
                    .clear()
                    .extend(new_segments);
                warn!(
                    "Fixing up event URL year segment for event id {}, {} -> {}",
                    event.id, event.url, new_url
                );
                event.url = new_url;
            }
        }
    }
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

fn presenters(year: u32, persons: content::pentabarf::Persons) -> Vec<shared::model::Person> {
    persons
        .persons
        .into_iter()
        .map(|p| shared::model::Person {
            id: shared::model::PersonId::new(year, p.id),
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
