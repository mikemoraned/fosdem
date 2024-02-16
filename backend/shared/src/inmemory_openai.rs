use std::path::Path;

use chrono::{Duration, FixedOffset, NaiveDateTime, Utc};
use futures::future::join_all;
use nalgebra::DVector;
use openai_dive::v1::api::Client;

use tracing::debug;

use crate::model::{Event, NextEvents, NextEventsContext, SearchItem};
use crate::queryable::Queryable;
use crate::{openai::get_embedding, queryable::MAX_RELATED_EVENTS};

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

    #[tracing::instrument(skip(self))]
    async fn find_next_events(
        &self,
        context: NextEventsContext,
    ) -> Result<NextEvents, Box<dyn std::error::Error>> {
        let all_events = self.load_all_events().await?;

        let (now, selected, current) = self.get_event_context(context, &all_events)?;

        let selected_end_time = selected.ending_time();
        let one_hour_after_selected_end_time = selected_end_time + Duration::hours(1);
        let next = self
            .find_overlapping_events(
                selected_end_time,
                one_hour_after_selected_end_time,
                &all_events,
            )
            .into_iter()
            .filter(|e| e.id != selected.id)
            .filter(|e| e.starting_time() >= selected.ending_time())
            .collect();

        Ok(NextEvents {
            now,
            current,
            selected,
            next,
        })
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

impl InMemoryOpenAIQueryable {
    fn get_event_context(
        &self,
        context: NextEventsContext,
        all_events: &Vec<Event>,
    ) -> Result<(NaiveDateTime, Event, Vec<Event>), Box<dyn std::error::Error>> {
        let now_utc = Utc::now();
        let central_european_time = FixedOffset::east_opt(1 * 3600).unwrap();
        let now_belgium = now_utc.with_timezone(&central_european_time);
        let now = now_belgium.naive_local();

        match context {
            NextEventsContext::Now => {
                let nearest_event = self.find_nearest_event(&now, all_events).unwrap();

                let current = self.find_overlapping_events(
                    nearest_event.starting_time(),
                    nearest_event.ending_time(),
                    all_events,
                );
                debug!("Found {} current events", current.len());
                let selected = current[0].clone();

                Ok((now, selected, current))
            }
            NextEventsContext::EventId(event_id) => {
                let mut found = None;
                for event in all_events.iter() {
                    if event.id == event_id {
                        found = Some(event.clone());
                        break;
                    }
                }
                if let Some(selected) = found {
                    let current = self.find_overlapping_events(
                        selected.starting_time(),
                        selected.ending_time(),
                        all_events,
                    );

                    debug!("Found {} current events", current.len());
                    Ok((now, selected, current))
                } else {
                    Err(format!("could not find event with id {}", event_id).into())
                }
            }
        }
    }

    fn find_nearest_event(&self, now: &NaiveDateTime, all_events: &[Event]) -> Option<Event> {
        let mut nearest = None;
        for event in all_events {
            let diff = event.starting_time().signed_duration_since(*now);
            match nearest {
                None => nearest = Some(event.clone()),
                Some(ref e) => {
                    let e_diff = e.starting_time().signed_duration_since(*now);
                    if diff.abs() < e_diff.abs() {
                        nearest = Some(event.clone())
                    }
                }
            }
        }
        nearest
    }

    fn find_overlapping_events(
        &self,
        begin: NaiveDateTime,
        end: NaiveDateTime,
        all_events: &Vec<Event>,
    ) -> Vec<Event> {
        let mut overlapping = vec![];
        for event in all_events.iter() {
            let starting_time = event.date.and_time(event.start);
            let ending_time = starting_time + Duration::minutes(event.duration.into());
            if begin <= ending_time && ending_time <= end {
                overlapping.push(event.clone());
            }
        }
        overlapping.sort_by(|a, b| a.start.cmp(&b.start));
        overlapping
    }
}

mod parsing {
    use std::{fs::File, io::BufReader, path::Path};

    use tracing::debug;

    use crate::model::Event;

    use super::{EmbeddedEvent, OpenAIVector};

    pub fn parse_embedded_events(
        model_dir: &Path,
    ) -> Result<Vec<EmbeddedEvent>, Box<dyn std::error::Error>> {
        let events_path = model_dir.join("events").with_extension("json");
        let events = parse_all_events(&events_path)?;
        let embeddings_path = model_dir.join("embedding").with_extension("csv");
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

    // fn parse_all_events(events_path: &Path) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
    //     debug!("Loading events data from {:?}", events_path);

    //     let mut rdr = csv::Reader::from_reader(File::open(events_path)?);
    //     let mut events = vec![];
    //     for result in rdr.deserialize() {
    //         let event: Event = result?;
    //         events.push(event);
    //     }
    //     events.sort_by(|a, b| a.id.cmp(&b.id));
    //     Ok(events)
    // }

    fn parse_all_events(events_path: &Path) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        debug!("Loading events data from {:?}", events_path);

        // let mut rdr = csv::Reader::from_reader(File::open(events_path)?);
        // let mut events = vec![];
        // for result in rdr.deserialize() {
        //     let event: Event = result?;
        //     events.push(event);
        // }

        let reader = BufReader::new(File::open(events_path)?);
        let mut events: Vec<Event> = serde_json::from_reader(reader)?;

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
