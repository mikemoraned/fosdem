use openai_dive::v1::api::Client;
use tracing::debug;

use crate::{
    openai::get_embedding,
    queryable::{Event, Queryable, SearchItem},
};

#[derive(Debug)]
pub struct InMemoryOpenAIQueryable {
    openai_client: Client,
}

impl InMemoryOpenAIQueryable {
    pub async fn connect(
        openai_api_key: &str,
    ) -> Result<InMemoryOpenAIQueryable, Box<dyn std::error::Error>> {
        debug!("Creating OpenAI Client");
        let openai_client = Client::new(openai_api_key.into());

        Ok(InMemoryOpenAIQueryable { openai_client })
    }
}

impl Queryable for InMemoryOpenAIQueryable {
    async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        todo!()
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
