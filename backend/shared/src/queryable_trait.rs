use chrono::{NaiveDate, NaiveTime};
use url::Url;

#[derive(Debug, Clone)]
pub struct SearchItem {
    pub event: Event,
    pub distance: f64,
    pub related: Option<Vec<SearchItem>>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: u32,
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub duration: u32,
    pub title: String,
    pub slug: String,
    pub url: Url,
    pub r#abstract: String,
}

pub trait QueryableTrait {
    async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>>;
    async fn find_related_events(
        &self,
        title: &String,
        limit: u8,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>>;
}
