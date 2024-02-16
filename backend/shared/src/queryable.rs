use crate::model::{Event, NextEvents, NextEventsContext, SearchItem};

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
