use log::info;
use openai_dive::v1::api::Client;
use pgvector::Vector;

use crate::openai::get_embedding;

pub struct Queryable {
    openai_client: Client,
}

impl Queryable {
    pub fn new(openai_api_key: &str) -> Queryable {
        let openai_client = Client::new(openai_api_key.into());
        Queryable { openai_client }
    }

    pub async fn find(&self, query: &str) -> Result<Vector, Box<dyn std::error::Error>> {
        info!("Getting embedding for query");
        let response = get_embedding(&self.openai_client, &query).await?;
        let embedding = Vector::from(
            response.data[0]
                .embedding
                .clone()
                .into_iter()
                .map(|f| f as f32)
                .collect::<Vec<_>>(),
        );
        Ok(embedding)
    }
}
