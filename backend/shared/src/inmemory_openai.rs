use std::path::Path;

use futures::future::join_all;
use nalgebra::DVector;
use openai_dive::v1::{api::Client, resources::embedding};
use tracing::debug;

use crate::{
    openai::get_embedding,
    queryable::{Event, Queryable, SearchItem, MAX_RELATED_EVENTS},
};

#[derive(Debug)]
pub struct InMemoryOpenAIQueryable {
    openai_client: Client,
    events: Vec<EmbeddedEvent>,
}

type OpenAIVector = DVector<f64>;

fn distance(lhs: &OpenAIVector, rhs: &OpenAIVector) -> f64 {
    lhs.metric_distance(rhs)
}

#[derive(Debug)]
struct EmbeddedEvent {
    event: Event,
    openai_embedding: OpenAIVector,
}

impl Queryable for InMemoryOpenAIQueryable {
    async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        Ok(self.events.iter().map(|e| e.event.clone()).collect())
    }

    async fn find_related_events(
        &self,
        title: &String,
        limit: u8,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        debug!("Finding embedding for title");
        let event = match self.events.iter().find(|e| e.event.title == *title) {
            Some(e) => e,
            None => return Err(format!("no embedding for \'{}\'", title).into()),
        };

        debug!("Finding all distances from embedding");
        let mut entries = vec![];
        for embedded_event in &self.events {
            if embedded_event.event.id != event.event.id {
                entries.push(SearchItem {
                    event: embedded_event.event.clone(),
                    distance: distance(&event.openai_embedding, &embedded_event.openai_embedding),
                    related: None,
                });
            }
        }
        debug!("Limiting to the top {}", limit);
        entries.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        entries.truncate(limit as usize);

        Ok(entries)
    }

    async fn search(
        &self,
        query: &str,
        limit: u8,
        find_related: bool,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        debug!("Getting embedding for query");
        let response = get_embedding(&self.openai_client, &query).await?;
        let embedding = OpenAIVector::from(response.data[0].embedding.clone());

        debug!("Finding all distances from embedding");
        let mut entries = vec![];
        for embedded_event in &self.events {
            entries.push(SearchItem {
                event: embedded_event.event.clone(),
                distance: distance(&embedding, &embedded_event.openai_embedding),
                related: None,
            });
        }
        debug!("Limiting to the top {}", limit);
        entries.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap());
        entries.truncate(limit as usize);

        if find_related {
            debug!("Running query to find related events");
            let jobs = entries.into_iter().map(|mut entry| async {
                entry.related = Some(
                    self.find_related_events(&entry.event.title, MAX_RELATED_EVENTS)
                        .await
                        .expect(&format!("find related items for {}", &entry.event.title)),
                );
                entry
            });
            let entries_with_related = join_all(jobs).await;
            debug!(
                "Found {} Events, with related Events",
                entries_with_related.len()
            );
            Ok(entries_with_related)
        } else {
            Ok(entries)
        }
    }
}

impl InMemoryOpenAIQueryable {
    pub async fn connect(
        csv_data_dir: &Path,
        openai_api_key: &str,
    ) -> Result<InMemoryOpenAIQueryable, Box<dyn std::error::Error>> {
        debug!("Creating OpenAI Client");
        let openai_client = Client::new(openai_api_key.into());

        debug!("Loading data from {:?}", csv_data_dir);
        Ok(InMemoryOpenAIQueryable {
            openai_client,
            events: parsing::parse_embedded_events(csv_data_dir)?,
        })
    }
}

mod parsing {
    use std::{fs::File, path::Path};

    use tracing::debug;

    use crate::queryable::Event;

    use super::{EmbeddedEvent, OpenAIVector};

    pub fn parse_embedded_events(
        csv_data_dir: &Path,
    ) -> Result<Vec<EmbeddedEvent>, Box<dyn std::error::Error>> {
        let events_path = csv_data_dir.join("event_6.csv");
        let events = parse_all_events(&events_path)?;
        let embeddings_path = csv_data_dir.join("embedding.csv");
        let embeddings: Vec<Embedding> = parse_all_embeddings(&embeddings_path)?;

        let mut embedded_events = vec![];
        for event in events {
            let result = embeddings.iter().find(|e| e.title == event.title);
            match result {
                Some(embedding) => embedded_events.push(EmbeddedEvent {
                    event,
                    openai_embedding: embedding.openai_embedding.clone(),
                }),
                None => {
                    return Err(
                        format!("failed to find embedding for title \'{}\'", event.title).into(),
                    );
                }
            }
        }
        Ok(embedded_events)
    }

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
                openai_embedding: parse_openai_embedding_vector(record.embedding)?,
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
            let parts: Vec<f64> = within
                .split(",")
                .into_iter()
                .map(|p| p.parse::<f64>().unwrap())
                .collect();
            let openaivector = OpenAIVector::from(parts);
            Ok(openaivector)
        } else {
            Err("not enclosed by []".into())
        }
    }
}
