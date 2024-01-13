use std::fs::File;

use chrono::{NaiveTime, Timelike};
use clap::{arg, Parser};
use content::pentabarf::Schedule;
use url::Url;
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

const BASE_URL_STRING: &str = "https://fosdem.org/2024/schedule/event/";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let xml = std::fs::read_to_string(args.pentabarf)?;
    let mut csv = csv::Writer::from_writer(File::create(args.csv)?);
    let schedule: Schedule = xml_deserialize_from_str(&xml)?;
    csv.write_record(&[
        "id", "date", "start", "duration", "title", "slug", "url", "abstract",
    ])?;
    for day in schedule.days {
        for room in day.rooms {
            for event in room.events {
                csv.write_record(&[
                    event.id.to_string(),
                    day.date.clone(),
                    event.start.value,
                    parse_into_minutes(&event.duration.value)?.to_string(),
                    event.title.value,
                    event.slug.value.clone(),
                    event_url(&event.slug.value)?.to_string(),
                    event.r#abstract.value,
                ])?;
            }
        }
    }

    Ok(())
}

fn event_url(slug: &str) -> Result<Url, Box<dyn std::error::Error>> {
    Ok(Url::parse(BASE_URL_STRING)?.join(slug)?)
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
