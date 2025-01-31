use std::fmt::{Display, Formatter};
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use nalgebra::DVector;
use openai_dive::v1::resources::embedding::{EmbeddingOutput, EmbeddingResponse};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchItem {
    pub event: Event,
    pub distance: f64,
    pub related: Option<Vec<SearchItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Event {
    pub id: u32,
    pub guid: String,
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

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Person {
    pub id: u32,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Link {
    pub url: Url,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RoomId(String); // TODO: use RoomId instead of String in Event

impl RoomId {
    pub fn new(id: String) -> RoomId {
        RoomId(id)
    }

    pub fn nav_url(&self) -> Url {
        let location_base_url = Url::parse("https://nav.fosdem.org/l/").unwrap();
        location_base_url.join(&self.nav_room()).unwrap()
    }

    pub fn nav_room(&self) -> String {
        if self.0.contains(' ') {
            let parts: Vec<_> = self.0.split(' ').collect();
            let start = parts[0];
            start.to_lowercase().replace('.', "")
        } else {
            self.0.to_lowercase().replace('.', "")
        }
    }
}

impl Display for RoomId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
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
        let base_url = Url::parse("https://fosdem.sojourner.rocks/2025/event/").unwrap();
        base_url.join(&self.guid.to_string()).unwrap()
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

pub type OpenAIVector = DVector<f64>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIEmbedding {
    pub title: String,
    pub embedding: OpenAIVector,
}

impl OpenAIEmbedding {
    pub fn embedding_from_response(response: &EmbeddingResponse) -> Result<OpenAIVector, Box<dyn std::error::Error>> {
        let output = response.data[0].embedding.clone();
        match output {
            EmbeddingOutput::Float(parts) => Ok(OpenAIVector::from(parts)),
            EmbeddingOutput::Base64(base64) => {
                Err(format!("Base64 encoding not supported: {}", base64).into())
            }
        }
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
