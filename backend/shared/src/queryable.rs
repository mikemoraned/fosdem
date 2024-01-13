use chrono::{NaiveDate, NaiveTime};
use futures::future::join_all;
use openai_dive::v1::api::Client;
use pgvector::Vector;
use sqlx::{
    postgres::{PgPoolOptions, PgRow},
    Pool, Postgres, Row,
};
use tracing::debug;
use url::Url;

use crate::{
    openai::get_embedding,
    queryable_trait::{Event, QueryableTrait, SearchItem},
};

#[derive(Debug)]
pub struct Queryable {
    openai_client: Client,
    pool: Pool<Postgres>,
}

const BASE_URL_STRING: &str = "https://fosdem.org/2024/schedule/event/";

const MAX_POOL_CONNECTIONS: u32 = 10;
const MAX_RELATED_EVENTS: u8 = 5;

impl QueryableTrait for Queryable {
    #[tracing::instrument(skip(self))]
    async fn load_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        debug!("Running Query to find all events");
        let rows =
            sqlx::query("SELECT id, start, date, duration, title, slug, abstract FROM events_5")
                .fetch_all(&self.pool)
                .await?;
        let mut events = vec![];
        for row in rows {
            events.push(self.row_to_event(&row)?);
        }
        Ok(events)
    }
}

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
        let rows =
            sqlx::query("SELECT id, start, date, duration, title, slug, abstract FROM events_5")
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
            sqlx::query("SELECT embedding FROM embedding_1 WHERE title = $1")
                .bind(title)
                .fetch_one(&self.pool)
                .await?
                .try_get("embedding")?;

        debug!("Running Query to find Events similar to title");
        let sql = "
    SELECT ev.id, ev.start, ev.date, ev.duration, ev.title, ev.slug, ev.abstract, 
           em.embedding <-> ($2) AS distance
    FROM embedding_1 em JOIN events_5 ev ON ev.title = em.title
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
    SELECT ev.id, ev.date, ev.start, ev.duration, ev.title, ev.slug, ev.abstract, 
           em.embedding <-> ($1) AS distance
    FROM embedding_1 em JOIN events_5 ev ON ev.title = em.title
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
        let title: String = row.try_get("title")?;
        let slug: String = row.try_get("slug")?;
        let url = self.event_url(&slug)?;
        let r#abstract: String = row.try_get("abstract")?;

        Ok(Event {
            id: id as u32,
            date,
            start,
            duration: duration as u32,
            title,
            slug,
            url,
            r#abstract,
        })
    }

    fn event_url(&self, slug: &str) -> Result<Url, Box<dyn std::error::Error>> {
        Ok(Url::parse(BASE_URL_STRING)?.join(slug)?)
    }
}
