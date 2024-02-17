use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use nalgebra::DVector;
use openai_dive::v1::resources::embedding::EmbeddingResponse;
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
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct Person {
    pub id: u32,
    pub name: String,
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
}

pub type OpenAIVector = DVector<f64>;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenAIEmbedding {
    pub title: String,
    pub embedding: OpenAIVector,
}

impl OpenAIEmbedding {
    pub fn embedding_from_response(response: &EmbeddingResponse) -> OpenAIVector {
        let parts = response.data[0].embedding.clone();
        OpenAIVector::from(parts)
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
