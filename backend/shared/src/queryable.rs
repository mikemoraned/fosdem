use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
pub struct SearchItem {
    pub event: Event,
    pub distance: f64,
    pub related: Option<Vec<SearchItem>>,
}

#[derive(Debug, Clone, serde::Deserialize, PartialEq)]
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

    pub fn nav_url(&self) -> Url {
        let location_base_url = Url::parse("https://nav.fosdem.org/l/").unwrap();
        let nav_room = if self.room.contains(" ") {
            let parts: Vec<_> = self.room.split(" ").collect();
            let start = parts[0];
            start.to_lowercase().replace(".", "")
        } else {
            self.room.to_lowercase().replace(".", "")
        };
        location_base_url.join(&nav_room).unwrap()
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

pub const MAX_RELATED_EVENTS: u8 = 5;

pub trait Queryable {
    async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>>;

    async fn find_related_events(
        &self,
        title: &String,
        limit: u8,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>>;

    async fn search(
        &self,
        query: &str,
        limit: u8,
        find_related: bool,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>>;

    async fn find_next_events(
        &self,
        context: NextEventsContext,
    ) -> Result<NextEvents, Box<dyn std::error::Error>>;
}
