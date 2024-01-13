use std::{fs::File, path::Path};

use openai_dive::v1::api::Client;
use tracing::debug;

use crate::{
    openai::get_embedding,
    queryable::{Event, Queryable, SearchItem},
};

#[derive(Debug)]
pub struct InMemoryOpenAIQueryable {
    openai_client: Client,
    events: Vec<Event>,
}

impl InMemoryOpenAIQueryable {
    pub async fn connect(
        csv_data_dir: &Path,
        openai_api_key: &str,
    ) -> Result<InMemoryOpenAIQueryable, Box<dyn std::error::Error>> {
        debug!("Creating OpenAI Client");
        let openai_client = Client::new(openai_api_key.into());

        debug!("Loading data from {:?}", csv_data_dir);

        let events_path = csv_data_dir.join("event_6.csv");
        let events = Self::parse_all_events(&events_path)?;

        Ok(InMemoryOpenAIQueryable {
            openai_client,
            events,
        })
    }

    fn parse_all_events(events_path: &Path) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        debug!("Loading events data from {:?}", events_path);

        let mut rdr = csv::Reader::from_reader(File::open(events_path)?);
        let mut events = vec![];
        for result in rdr.deserialize() {
            let event: Event = result?;
            events.push(event);
        }
        events.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(events)
    }
}

impl Queryable for InMemoryOpenAIQueryable {
    async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        Ok(self.events.clone())
    }

    async fn find_related_events(
        &self,
        title: &String,
        limit: u8,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        todo!()
    }

    async fn search(
        &self,
        query: &str,
        limit: u8,
        find_related: bool,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        todo!()
    }
}
