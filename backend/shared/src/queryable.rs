use chrono::{Duration, FixedOffset, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use futures::future::join_all;
use openai_dive::v1::api::Client;
use pgvector::Vector;
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    Pool, Postgres, Row,
};
use tracing::debug;
use url::Url;

use crate::openai::get_embedding;

#[derive(Debug)]
pub struct Queryable {
    openai_client: Client,
    pool: Pool<Postgres>,
}

#[derive(Debug, Clone)]
pub struct SearchItem {
    pub event: Event,
    pub distance: f64,
    pub related: Option<Vec<SearchItem>>,
}

#[derive(Debug, Clone)]
pub struct Event {
    pub id: u32,
    pub date: NaiveDate,
    pub start: NaiveTime,
    pub duration: u32,
    pub room: String,
    pub track: String,
    pub title: String,
    pub slug: String,
    pub url: Url,
    pub r#abstract: String,
}

impl Event {
    fn starting_time(&self) -> NaiveDateTime {
        self.date.and_time(self.start)
    }

    fn ending_time(&self) -> NaiveDateTime {
        self.starting_time() + Duration::minutes(self.duration.into())
    }
}

#[derive(Debug, Clone)]
pub struct NextEvents {
    pub now: NaiveDateTime,
    pub current: Vec<Event>,
    pub selected: Event,
    pub next: Vec<Event>,
}

#[derive(Debug)]
pub enum NextEventsContext {
    Now,
    EventId(u32),
}

const MAX_POOL_CONNECTIONS: u32 = 10;
const MAX_RELATED_EVENTS: u8 = 5;

impl Queryable {
    pub async fn connect(
        db_host: &str,
        db_password: &str,
        openai_api_key: &str,
    ) -> Result<Queryable, Box<dyn std::error::Error>> {
        debug!("Creating OpenAI Client");
        let openai_client = Client::new(openai_api_key.into());

        debug!("Connecting to DB");
        let db_url = format!("postgres://postgres:{}@{}/postgres", db_password, db_host);
        let pool = PgPoolOptions::new()
            .max_connections(MAX_POOL_CONNECTIONS)
            .connect(&db_url)
            .await?;
        Ok(Queryable {
            openai_client,
            pool,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        debug!("Running Query to find all events");
        let rows = sqlx::query(
            "SELECT id, start, date, duration, room, track, title, slug, url, abstract FROM events_8",
        )
        .fetch_all(&self.pool)
        .await?;
        let mut events = vec![];
        for row in rows {
            events.push(self.row_to_event(&row)?);
        }
        Ok(events)
    }

    #[tracing::instrument(skip(self))]
    pub async fn find_related_events(
        &self,
        title: &String,
        limit: u8,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        debug!("Running Query to find embedding for title");
        let embedding: pgvector::Vector =
            sqlx::query("SELECT embedding FROM embedding_3 WHERE title = $1")
                .bind(title)
                .fetch_one(&self.pool)
                .await?
                .try_get("embedding")?;

        debug!("Running Query to find Events similar to title");
        let sql = "
    SELECT ev.id, ev.start, ev.date, ev.duration, ev.room, ev.track, ev.title, ev.slug, ev.url, ev.abstract, 
           em.embedding <-> ($2) AS distance
    FROM embedding_3 em JOIN events_8 ev ON ev.title = em.title
    WHERE ev.title != $1
    ORDER BY em.embedding <-> ($2) LIMIT $3;
    ";
        let rows = sqlx::query(sql)
            .bind(title)
            .bind(embedding)
            .bind(limit as i32)
            .fetch_all(&self.pool)
            .await?;
        let mut entries = vec![];
        for row in rows {
            entries.push(self.row_to_search_item(&row)?);
        }
        debug!("Found {} Events similar to title", entries.len());
        Ok(entries)
    }

    #[tracing::instrument(skip(self))]
    pub async fn search(
        &self,
        query: &str,
        limit: u8,
        find_related: bool,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
        debug!("Getting embedding for query");
        let response = get_embedding(&self.openai_client, &query).await?;
        let embedding = Vector::from(
            response.data[0]
                .embedding
                .clone()
                .into_iter()
                .map(|f| f as f32)
                .collect::<Vec<_>>(),
        );

        debug!("Running query to find similar events");
        let sql = "
    SELECT ev.id, ev.date, ev.start, ev.duration, ev.room, ev.track, ev.title, ev.slug, ev.url, ev.abstract, 
           em.embedding <-> ($1) AS distance
    FROM embedding_3 em JOIN events_8 ev ON ev.title = em.title
    ORDER BY em.embedding <-> ($1) LIMIT $2;
    ";
        let rows = sqlx::query(sql)
            .bind(embedding)
            .bind(limit as i32)
            .fetch_all(&self.pool)
            .await?;
        let mut entries = vec![];
        for row in rows {
            entries.push(self.row_to_search_item(&row)?);
        }
        debug!("Found {} Events", entries.len());

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
    pub async fn find_next_events(
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

    fn row_to_search_item(&self, row: &PgRow) -> Result<SearchItem, Box<dyn std::error::Error>> {
        let distance: f64 = row.try_get("distance")?;
        Ok(SearchItem {
            event: self.row_to_event(&row)?,
            distance,
            related: None,
        })
    }

    fn row_to_event(&self, row: &PgRow) -> Result<Event, Box<dyn std::error::Error>> {
        let id: i64 = row.try_get("id")?;
        let date: NaiveDate = row.try_get("date")?;
        let start: NaiveTime = row.try_get("start")?;
        let duration: i64 = row.try_get("duration")?;
        let track: String = row.try_get("track")?;
        let room: String = row.try_get("room")?;
        let title: String = row.try_get("title")?;
        let slug: String = row.try_get("slug")?;
        let url = Url::parse(row.try_get("url")?)?;
        let r#abstract: String = row.try_get("abstract")?;

        Ok(Event {
            id: id as u32,
            date,
            start,
            duration: duration as u32,
            track,
            room,
            title,
            slug,
            url,
            r#abstract,
        })
    }
}
