use std::collections::HashMap;
use std::path::Path;

use chrono::{Duration, FixedOffset, NaiveDateTime, Utc};
use embedding::model::distance;
use embedding::model::Embedding;
use embedding::openai_ada2::get_phrase_embedding;
use openai_dive::v1::api::Client;

use shared::model::Event;
use shared::model::EventId;
use shared::model::NextEvents;
use shared::model::NextEventsContext;
use shared::model::RoundedDistance;
use shared::model::SearchItem;
use tracing::{debug, span};

use crate::queryable::Queryable;
use crate::queryable::SearchKind;
use crate::queryable::MAX_RELATED_EVENTS;

#[derive(Debug)]
pub struct InMemoryOpenAIQueryable {
    openai_client: Client,
    events: Vec<Event>,
    index: HashMap<SearchKind, HashMap<EventId, EventEmbedding>>,
}

#[derive(Debug)]
struct EventEmbedding {
    event: Event,
    embedding: Embedding,
}

impl EventEmbedding {
    pub fn distance(&self, embedding: &Embedding) -> RoundedDistance {
        match &self.embedding {
            Embedding::OpenAIAda2 { vector } => {
                let self_vector = vector;
                match embedding {
                    Embedding::OpenAIAda2 { vector } => distance(self_vector, &vector).into(),
                }
            }
        }
    }
}

impl Queryable for InMemoryOpenAIQueryable {
    #[tracing::instrument(skip(self))]
    async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        Ok(self.events.clone())
    }

    #[tracing::instrument(skip(self))]
    async fn find_event_by_id(
        &self,
        event_id: u32,
    ) -> Result<Option<Event>, Box<dyn std::error::Error>> {
        Ok(match self.events.iter().find(|e| e.id == event_id) {
            Some(e) => Some(e.clone()),
            None => None,
        })
    }

    #[tracing::instrument(skip(self))]
    async fn find_related_events(
        &self,
        title: &str,
        kind: &SearchKind,
        limit: u8,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        debug!("Finding parent event for title");
        let parent_event = match self.events.iter().find(|e| e.title == *title) {
            Some(e) => e,
            None => return Err(format!("no event for \'{}\'", title).into()),
        };
        let parent_event_id = EventId(parent_event.id);

        if let Some(embedding_index) = self.index.get(&kind) {
            debug!("Finding embedding for parent event");
            if let Some(parent_event_embedding) = embedding_index.get(&parent_event_id) {
                debug!("Finding all distances from parent embedding");
                let mut entries = vec![];
                for (event_id, embedding) in embedding_index {
                    if event_id != &parent_event_id {
                        entries.push(SearchItem {
                            event: embedding.event.clone(),
                            distance: embedding.distance(&parent_event_embedding.embedding),
                            related: None,
                        });
                    }
                }
                debug!("Limiting to the top {}", limit);
                InMemoryOpenAIQueryable::normalised_sort(&mut entries);
                entries.truncate(limit as usize);

                Ok(entries)
            } else {
                Err(format!(
                    "no embedding for parent event {:?} in {:?}",
                    parent_event_id, kind
                )
                .into())
            }
        } else {
            Err(format!("no index for {:?}", kind).into())
        }
    }

    #[tracing::instrument(skip(self))]
    async fn search(
        &self,
        query: &str,
        kind: &SearchKind,
        limit: u8,
        find_related: bool,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        debug!("Getting embedding for query");
        let embedding = get_phrase_embedding(&self.openai_client, query).await?;

        let embedding_index = self
            .index
            .get(&kind)
            .ok_or(format!("no index for {:?}", kind))?;

        debug!("Finding all distances from embedding");
        let mut entries = vec![];
        for event_embedding in embedding_index.values() {
            entries.push(SearchItem {
                event: event_embedding.event.clone(),
                distance: event_embedding.distance(&embedding),
                related: None,
            });
        }
        debug!("Limiting to the top {}", limit);
        InMemoryOpenAIQueryable::normalised_sort(&mut entries);
        entries.truncate(limit as usize);

        if find_related {
            span!(tracing::Level::INFO, "find_related")
                .in_scope(|| async {
                    debug!("Running query to find related events");
                    let mut entries_with_related = vec![];
                    for mut entry in entries.into_iter() {
                        entry.related = Some(
                            self.find_related_events(&entry.event.title, kind, MAX_RELATED_EVENTS)
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
        let (events, index) = parsing::parse_embedded_events(model_dir)?;
        Ok(InMemoryOpenAIQueryable {
            openai_client,
            events,
            index,
        })
    }

    fn normalised_sort(items: &mut Vec<SearchItem>) {
        items.sort_by(|a, b| {
            if a.distance == b.distance {
                a.event.id.partial_cmp(&b.event.id).unwrap()
            } else {
                a.distance.partial_cmp(&b.distance).unwrap()
            }
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
    use std::{collections::HashMap, fs::File, io::BufReader, path::Path, vec};

    use embedding::parsing::parse_all_subject_embeddings;

    use shared::model::{Event, EventArtefact, EventId};
    use tracing::debug;

    use crate::queryable::SearchKind;

    use super::EventEmbedding;

    pub fn parse_embedded_events(
        model_dir: &Path,
    ) -> Result<
        (
            Vec<Event>,
            HashMap<SearchKind, HashMap<EventId, EventEmbedding>>,
        ),
        Box<dyn std::error::Error>,
    > {
        let events_path = model_dir.join("events").with_extension("json");
        let events_index = parse_all_events(&events_path)?;

        let embeddings_paths = vec![
            model_dir
                .join("openai_combined_embeddings")
                .with_extension("json"),
            model_dir
                .join("openai_video_embeddings")
                .with_extension("json"),
        ];
        let subject_embeddings = parse_all_subject_embeddings(&embeddings_paths)?;
        let mut index = HashMap::new();
        for search_kind in vec![SearchKind::Combined, SearchKind::VideoOnly] {
            index.insert(search_kind, HashMap::new());
        }
        for subject_embedding in subject_embeddings {
            let (event_id, search_kind) = match subject_embedding.subject {
                EventArtefact::Combined { event_id } => (event_id, SearchKind::Combined),
                EventArtefact::Video { event_id, file: _ } => (event_id, SearchKind::VideoOnly),
            };
            let embedding_index = index
                .get_mut(&search_kind)
                .ok_or(format!("no index for {:?}", search_kind))?;
            let event = events_index
                .get(&event_id)
                .ok_or(format!("could not find event for {:?}", event_id))?;
            let event_embedding = EventEmbedding {
                event: event.clone(),
                embedding: subject_embedding.embedding.clone(),
            };
            embedding_index.insert(event_id.clone(), event_embedding);
        }

        if let Some(embedding_index) = index.get(&SearchKind::Combined) {
            for (event_id, event) in &events_index {
                if !embedding_index.contains_key(&event_id) {
                    return Err(format!(
                        "failed to find combined embedding for title \'{}\' with id {}",
                        event.title, event.id
                    )
                    .into());
                }
            }
        }

        let mut events: Vec<Event> = events_index
            .values()
            .map(|e| e.clone())
            .into_iter()
            .collect();

        events.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Ok((events, index))
    }

    fn parse_all_events(
        events_path: &Path,
    ) -> Result<HashMap<EventId, Event>, Box<dyn std::error::Error>> {
        debug!("Loading events data from {:?}", events_path);

        let mut events_index = HashMap::new();

        let reader = BufReader::new(File::open(events_path)?);
        let events: Vec<Event> = serde_json::from_reader(reader)?;

        for event in events.into_iter() {
            events_index.insert(EventId(event.id), event);
        }

        Ok(events_index)
    }
}
