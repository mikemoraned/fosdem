use std::{fmt::Display, fs::File, io::BufReader, path::Path};

use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use tracing::trace;
use url::Url;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SearchItem {
    pub event: Event,
    pub distance: RoundedDistance,
    pub related: Option<Vec<SearchItem>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, PartialOrd)]
pub struct RoundedDistance {
    units: u8,
}

impl RoundedDistance {
    pub fn from_units(units: u8) -> RoundedDistance {
        RoundedDistance { units }
    }
}

impl From<f64> for RoundedDistance {
    fn from(distance: f64) -> Self {
        let units = (distance * 100.0).round() as u8;
        RoundedDistance { units }
    }
}

impl Into<f64> for RoundedDistance {
    fn into(self) -> f64 {
        (self.units as f64) / 100.0
    }
}

impl RoundedDistance {
    pub fn to_similarity(&self) -> RoundedSimilarity {
        let units = 100 - self.units;
        RoundedSimilarity { units }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, PartialOrd)]
pub struct RoundedSimilarity {
    units: u8,
}

impl RoundedSimilarity {
    pub fn from_units(units: u8) -> RoundedSimilarity {
        RoundedSimilarity { units }
    }
}

impl Into<f64> for RoundedSimilarity {
    fn into(self) -> f64 {
        (self.units as f64) / 100.0
    }
}

impl Into<f64> for &RoundedSimilarity {
    fn into(self) -> f64 {
        (self.units as f64) / 100.0
    }
}

impl Display for RoundedSimilarity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let converted: f64 = self.into();
        write!(f, "{:.2}", converted)
    }
}

#[cfg(test)]
mod test {
    use super::{RoundedDistance, RoundedSimilarity};

    #[test]
    fn test_rounded_distance_from_f64() {
        let inputs = vec![0.64, 0.28, 0.0, 1.0];
        let expected_outputs = vec![
            RoundedDistance::from_units(64u8),
            RoundedDistance::from_units(28u8),
            RoundedDistance::from_units(0u8),
            RoundedDistance::from_units(100u8),
        ];

        for (input, expected) in inputs.into_iter().zip(expected_outputs.into_iter()) {
            let actual: RoundedDistance = input.into();
            assert_eq!(expected.units, actual.units);
        }
    }

    #[test]
    fn test_rounded_distance_into_f64() {
        let inputs = vec![
            RoundedDistance::from_units(64u8),
            RoundedDistance::from_units(28u8),
            RoundedDistance::from_units(0u8),
            RoundedDistance::from_units(100u8),
        ];
        let expected_outputs = vec![0.64, 0.28, 0.0, 1.0];

        for (input, expected) in inputs.into_iter().zip(expected_outputs.into_iter()) {
            let actual: f64 = input.into();
            assert_eq!(expected, actual);
        }
    }

    #[test]
    fn test_rounded_similarity_into_f64() {
        let inputs = vec![
            RoundedSimilarity::from_units(64u8),
            RoundedSimilarity::from_units(28u8),
            RoundedSimilarity::from_units(0u8),
            RoundedSimilarity::from_units(100u8),
        ];
        let expected_outputs = vec![0.64, 0.28, 0.0, 1.0];

        for (input, expected) in inputs.into_iter().zip(expected_outputs.into_iter()) {
            let actual: f64 = input.into();
            assert_eq!(expected, actual);
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub enum EventArtefact {
    Combined { event_id: EventId },
    Video { event_id: EventId, file: VideoFile },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct VideoFile {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Hash, Eq, PartialOrd)]
pub struct EventId(pub u32);

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Event {
    pub id: u32,
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub duration: u32,
    pub room: String,
    pub track: String,
    pub title: String,
    pub slug: String,
    pub url: Url,
    pub r#abstract: String,
    pub slides: Vec<Url>,
    pub presenters: Vec<Person>,
    pub links: Vec<Link>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Person {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, PartialOrd)]
pub struct Link {
    pub url: Url,
    pub name: String,
}

impl Event {
    pub fn from_model_area(model_dir: &Path) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        let events_path = model_dir.join("events").with_extension("json");

        trace!("Reading events from {} ... ", events_path.to_str().unwrap());
        let reader = BufReader::new(File::open(events_path)?);
        Ok(serde_json::from_reader(reader)?)
    }
}

impl Event {
    pub fn starting_time(&self) -> NaiveDateTime {
        self.date.and_time(self.start)
    }

    pub fn ending_time(&self) -> NaiveDateTime {
        self.starting_time() + Duration::minutes(self.duration.into())
    }

    pub fn sojourner_url(&self) -> Url {
        let base_url = Url::parse("https://fosdem.sojourner.rocks/2024/event/").unwrap();
        base_url.join(&self.id.to_string()).unwrap()
    }

    pub fn nav_url(&self, current_event: &Option<Event>) -> Url {
        match current_event {
            Some(event) => Url::parse(&format!(
                "https://nav.fosdem.org/r/{}/{}",
                event.nav_room(),
                self.nav_room()
            ))
            .unwrap(),
            None => {
                let location_base_url = Url::parse("https://nav.fosdem.org/l/").unwrap();
                location_base_url.join(&self.nav_room()).unwrap()
            }
        }
    }

    pub fn nav_room(&self) -> String {
        if self.room.contains(' ') {
            let parts: Vec<_> = self.room.split(' ').collect();
            let start = parts[0];
            start.to_lowercase().replace('.', "")
        } else {
            self.room.to_lowercase().replace('.', "")
        }
    }

    pub fn mp4_video_link(&self) -> Option<Url> {
        let video_links: Vec<Url> = self
            .links
            .iter()
            .filter(|l| l.name == "Video recording (mp4)" && l.url.to_string().ends_with(".mp4"))
            .map(|l| l.url.clone())
            .collect();
        video_links.first().cloned()
    }
}

#[derive(Debug, Clone)]
pub struct NextEvents {
    pub now: NaiveDateTime,
    pub current: Vec<Event>,
    pub selected: Event,
    pub next: Vec<Event>,
}

#[derive(Debug)]
pub enum NextEventsContext {
    Now,
    EventId(u32),
}
