use std::{fs::File, path::Path};

use nalgebra::{DVector, Point};
use openai_dive::v1::{api::Client, endpoints::embeddings, resources::embedding};
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

type OpenAIVector = DVector<f32>;
#[derive(Debug)]
struct Embedding {
    title: String,
    openai_embedding: OpenAIVector,
}

#[derive(Debug, serde::Deserialize)]
struct EmbeddingRecord {
    title: String,
    embedding: String,
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
        let embeddings_path = csv_data_dir.join("embedding.csv");
        let embeddings: Vec<Embedding> = Self::parse_all_embeddings(&embeddings_path)?;
        println!("embeddings: {:?}", embeddings);

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

    fn parse_all_embeddings(
        embeddings_path: &Path,
    ) -> Result<Vec<Embedding>, Box<dyn std::error::Error>> {
        debug!("Loading embeddings data from {:?}", embeddings_path);

        let mut rdr = csv::Reader::from_reader(File::open(embeddings_path)?);
        let mut embeddings = vec![];
        for result in rdr.deserialize() {
            let record: EmbeddingRecord = result?;
            let embedding = Embedding {
                title: record.title,
                openai_embedding: Self::parse_openai_embedding_vector(record.embedding)?,
            };
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    fn parse_openai_embedding_vector(
        embedding: String,
    ) -> Result<OpenAIVector, Box<dyn std::error::Error>> {
        if embedding.starts_with("[") && embedding.ends_with("]") {
            let within = &embedding[1..&embedding.len() - 1];
            let parts: Vec<f32> = within
                .split(",")
                .into_iter()
                .map(|p| p.parse::<f32>().unwrap())
                .collect();
            let openaivector = OpenAIVector::from(parts);
            Ok(openaivector)
        } else {
            Err("not enclosed by []".into())
        }
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
