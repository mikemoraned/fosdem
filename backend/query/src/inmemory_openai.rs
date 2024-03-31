use std::path::Path;

use chrono::{Duration, FixedOffset, NaiveDateTime, Utc};
use embedding::model::distance;
use embedding::model::Embedding;
use embedding::model::OpenAIVector;
use embedding::openai_ada2::get_phrase_embedding;
use openai_dive::v1::api::Client;

use shared::model::Event;
use shared::model::NextEvents;
use shared::model::NextEventsContext;
use shared::model::SearchItem;
use tracing::{debug, span};

use crate::queryable::Queryable;
use crate::queryable::SearchKind;
use crate::queryable::MAX_RELATED_EVENTS;

#[derive(Debug)]
pub struct InMemoryOpenAIQueryable {
    openai_client: Client,
    events: Vec<EmbeddedEvent>,
}

#[derive(Debug)]
struct EmbeddedEvent {
    event: Event,
    openai_embedding: OpenAIVector,
}

impl EmbeddedEvent {
    pub fn distance(&self, embedding: &Embedding) -> f64 {
        let Embedding::OpenAIAda2 { vector } = embedding;
        distance(&self.openai_embedding, &vector)
    }
}

impl Queryable for InMemoryOpenAIQueryable {
    #[tracing::instrument(skip(self))]
    async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        Ok(self.events.iter().map(|e| e.event.clone()).collect())
    }

    #[tracing::instrument(skip(self))]
    async fn find_event_by_id(
        &self,
        event_id: u32,
    ) -> Result<Option<Event>, Box<dyn std::error::Error>> {
        Ok(match self.events.iter().find(|e| e.event.id == event_id) {
            Some(e) => Some(e.event.clone()),
            None => None,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn find_related_events(
        &self,
        title: &str,
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
                    distance: embedded_event.distance(&Embedding::OpenAIAda2 {
                        vector: event.openai_embedding.clone(),
                    }),
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
        kind: &SearchKind,
        find_related: bool,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        debug!("Getting embedding for query");
        let embedding = get_phrase_embedding(&self.openai_client, query).await?;

        debug!("Finding all distances from embedding");
        let mut entries = vec![];
        for embedded_event in &self.events {
            entries.push(SearchItem {
                event: embedded_event.event.clone(),
                distance: embedded_event.distance(&embedding),
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
                            self.find_related_events(&entry.event.title, MAX_RELATED_EVENTS)
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
    #[tracing::instrument(skip(self))]
    fn get_event_context(
        &self,
        context: NextEventsContext,
        all_events: &[Event],
    ) -> Result<(NaiveDateTime, Event, Vec<Event>), Box<dyn std::error::Error>> {
        let now_utc = Utc::now();
        let central_european_time = FixedOffset::east_opt(3600).unwrap();
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

    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self))]
    fn find_overlapping_events(
        &self,
        begin: NaiveDateTime,
        end: NaiveDateTime,
        all_events: &[Event],
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
    use std::{fs::File, io::BufReader, path::Path, vec};

    use embedding::{model::Embedding, parsing::parse_all_subject_embeddings_into_index};

    use shared::model::{Event, EventArtefact, EventId};
    use tracing::debug;

    use super::EmbeddedEvent;

    pub fn parse_embedded_events(
        model_dir: &Path,
    ) -> Result<Vec<EmbeddedEvent>, Box<dyn std::error::Error>> {
        let events_path = model_dir.join("events").with_extension("json");
        let events = parse_all_events(&events_path)?;
        let embeddings_paths = vec![
            model_dir
                .join("openai_combined_embeddings")
                .with_extension("json"),
            model_dir
                .join("openai_video_embeddings")
                .with_extension("json"),
        ];
        let embeddings_index = parse_all_subject_embeddings_into_index(&embeddings_paths)?;

        let mut embedded_events = vec![];
        for event in events {
            match embeddings_index.get(&EventId(event.id)) {
                Some(embeddings) => {
                    let possible_embedding = embeddings.iter().find(|e| {
                        EventArtefact::Combined {
                            event_id: EventId(event.id),
                        } == e.subject
                    });
                    match possible_embedding {
                        Some(subject_embedding) => {
                            let Embedding::OpenAIAda2 { vector } = &subject_embedding.embedding;
                            embedded_events.push(EmbeddedEvent {
                                event,
                                openai_embedding: vector.clone(),
                            });
                        }
                        None => {
                            return Err(format!(
                                "failed to find combined embedding for title \'{}\' with id {}",
                                event.title, event.id
                            )
                            .into());
                        }
                    }
                }
                None => {
                    return Err(format!(
                        "failed to find any embedding for title \'{}\' with id {}",
                        event.title, event.id
                    )
                    .into());
                }
            }
        }
        Ok(embedded_events)
    }

    fn parse_all_events(events_path: &Path) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        debug!("Loading events data from {:?}", events_path);

        let reader = BufReader::new(File::open(events_path)?);
        let mut events: Vec<Event> = serde_json::from_reader(reader)?;

        events.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(events)
    }
}
