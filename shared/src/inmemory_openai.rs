use std::path::Path;

use openai_dive::v1::api::Client;

use tracing::{debug, span};

use crate::model::{Event, EventId, OpenAIEmbedding, OpenAIVector, SearchItem};
use crate::queryable::Queryable;
use crate::{openai::get_embedding, queryable::MAX_RELATED_EVENTS};

#[derive(Debug)]
pub struct InMemoryOpenAIQueryable {
    openai_client: Client,
    events: Vec<EmbeddedEvent>,
}

fn distance(lhs: &OpenAIVector, rhs: &OpenAIVector) -> f64 {
    lhs.metric_distance(rhs)
}

#[derive(Debug)]
struct EmbeddedEvent {
    event: Event,
    openai_embedding: OpenAIVector,
}

impl Queryable for InMemoryOpenAIQueryable {
    #[tracing::instrument(skip(self))]
    async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        Ok(self.events.iter().map(|e| e.event.clone()).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn find_event_by_id(
        &self,
        event_id: EventId,
    ) -> Result<Option<Event>, Box<dyn std::error::Error>> {
        Ok(self
            .events
            .iter()
            .find(|e| e.event.id == event_id)
            .map(|e| e.event.clone()))
    }

    #[tracing::instrument(skip(self))]
    async fn find_related_events(
        &self,
        title: &str,
        limit: u8,
        year_filter: Option<u32>,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        debug!("Finding embedding for title");
        let event = match self.events.iter().find(|e| e.event.title == *title) {
            Some(e) => e,
            None => return Err(format!("no embedding for \'{}\'", title).into()),
        };

        debug!("Finding all distances from embedding");
        let mut entries = vec![];
        for embedded_event in &self.events {
            if let Some(year) = year_filter {
                if embedded_event.event.year != year {
                    continue;
                }
            }
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

    #[tracing::instrument(skip(self))]
    async fn search(
        &self,
        query: &str,
        limit: u8,
        find_related: bool,
        year_filter: Option<u32>,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        debug!("Getting embedding for query");
        let response = get_embedding(&self.openai_client, query).await?;
        let embedding = OpenAIEmbedding::embedding_from_response(&response)?;

        debug!("Finding all distances from embedding");
        let mut entries = vec![];
        for embedded_event in &self.events {
            if let Some(year) = year_filter {
                if embedded_event.event.year != year {
                    continue;
                }
            }
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
            span!(tracing::Level::INFO, "find_related")
                .in_scope(|| async {
                    debug!("Running query to find related events");
                    let mut entries_with_related = vec![];
                    for mut entry in entries.into_iter() {
                        entry.related = Some(
                            self.find_related_events(
                                &entry.event.title,
                                MAX_RELATED_EVENTS,
                                year_filter,
                            )
                            .await?,
                        );
                        entries_with_related.push(entry);
                    }
                    debug!(
                        "Found {} Events, with related Events",
                        entries_with_related.len()
                    );
                    Ok(entries_with_related)
                })
                .await
        } else {
            Ok(entries)
        }
    }
}

impl InMemoryOpenAIQueryable {
    pub async fn connect(
        model_dir: &Path,
        openai_api_key: &str,
    ) -> Result<InMemoryOpenAIQueryable, Box<dyn std::error::Error>> {
        debug!("Creating OpenAI Client");
        let openai_client = Client::new(openai_api_key.into());

        debug!("Loading data from {:?}", model_dir);
        Ok(InMemoryOpenAIQueryable {
            openai_client,
            events: parsing::parse_embedded_events(model_dir)?,
        })
    }
}

mod parsing {
    use std::{
        fs::File,
        io::{BufReader, Read},
        path::Path,
    };

    use flate2::read::GzDecoder;
    use tracing::{error, info};

    use crate::model::{Event, OpenAIEmbedding};

    use super::EmbeddedEvent;

    pub fn parse_embedded_events(
        model_dir: &Path,
    ) -> Result<Vec<EmbeddedEvent>, Box<dyn std::error::Error>> {
        let events_path = model_dir.join("events").with_extension("json");
        let events = parse_all_events(&events_path)?;
        let embeddings_path = model_dir.join("embeddings").with_extension("json.gz");
        let embeddings: Vec<OpenAIEmbedding> = parse_all_embeddings(&embeddings_path)?;

        info!(
            "Parsing embeddings and events from {:?} and {:?}",
            events_path, embeddings_path
        );

        let mut embedded_events = vec![];
        for event in events {
            let result = embeddings.iter().find(|e| e.title == event.title);
            match result {
                Some(embedding) => embedded_events.push(EmbeddedEvent {
                    event,
                    openai_embedding: embedding.embedding.clone(),
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

    fn parse_all_events(events_path: &Path) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        info!("Loading events data from {:?}", events_path);

        let reader = reader_for_path(events_path)?;
        match serde_json::from_reader::<BufReader<Box<dyn std::io::Read>>, Vec<Event>>(reader) {
            Ok(mut events) => {
                events.sort_by(|a, b| a.id.cmp(&b.id));
                info!("Loaded events data from {:?}", events_path);
                Ok(events)
            }
            Err(e) => {
                error!("error: {}", e);
                Err(format!("Could not parse_all_events: {}", e).into())
            }
        }
    }

    fn parse_all_embeddings(
        embeddings_path: &Path,
    ) -> Result<Vec<OpenAIEmbedding>, Box<dyn std::error::Error>> {
        info!("Loading embeddings data from {:?}", embeddings_path);

        let reader = reader_for_path(embeddings_path)?;

        match serde_json::from_reader::<BufReader<Box<dyn std::io::Read>>, Vec<OpenAIEmbedding>>(
            reader,
        ) {
            Ok(embeddings) => {
                info!("Loaded embeddings data from {:?}", embeddings_path);
                Ok(embeddings)
            }
            Err(e) => {
                error!("error: {}", e);
                Err(format!("Could not parse_all_embeddings: {}", e).into())
            }
        }
    }

    fn reader_for_path(
        path: &Path,
    ) -> Result<BufReader<Box<dyn Read>>, Box<dyn std::error::Error>> {
        match File::open(path) {
            Ok(file) => {
                let reader: BufReader<Box<dyn Read>> =
                    BufReader::new(if path.extension().is_some_and(|ext| ext == "gz") {
                        Box::new(GzDecoder::new(file))
                    } else {
                        Box::new(file)
                    });
                Ok(reader)
            }
            Err(e) => {
                error!("error: {}", e);
                Err(format!("Could not choose reader_for_path: {}", e).into())
            }
        }
    }
}
