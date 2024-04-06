use serde::Deserialize;
use shared::model::{Event, NextEvents, NextEventsContext, SearchItem};

pub const MAX_RELATED_EVENTS: u8 = 5;

#[derive(Debug, Deserialize, Clone, Default, Eq, PartialEq, Hash)]
pub enum SearchKind {
    #[serde(rename = "combined")]
    #[default]
    Combined,
    #[serde(rename = "video_only")]
    VideoOnly,
}

impl SearchKind {
    pub fn form_value(&self) -> String {
        match self {
            Self::Combined => "combined",
            Self::VideoOnly => "video_only",
        }
        .into()
    }
}

#[allow(async_fn_in_trait)]
pub trait Queryable {
    async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>>;

    async fn find_event_by_id(
        &self,
        event_id: u32,
    ) -> Result<Option<Event>, Box<dyn std::error::Error>>;

    async fn find_related_events(
        &self,
        title: &str,
        limit: u8,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>>;

    async fn search(
        &self,
        query: &str,
        limit: u8,
        kind: &SearchKind,
        find_related: bool,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>>;

    async fn find_next_events(
        &self,
        context: NextEventsContext,
    ) -> Result<NextEvents, Box<dyn std::error::Error>>;
}
