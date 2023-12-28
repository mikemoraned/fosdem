use log::info;
use openai_dive::v1::api::Client;
use pgvector::Vector;
use serde::Serialize;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use url::Url;

use crate::openai::get_embedding;

#[derive(Debug)]
pub struct Queryable {
    openai_client: Client,
    pool: Pool<Postgres>,
}

#[derive(Serialize)]
pub struct SearchItem {
    pub event: Event,
    pub distance: f64,
}

#[derive(Serialize)]
pub struct Event {
    pub title: String,
    pub slug: String,
    pub url: Url,
    pub r#abstract: String,
}

const BASE_URL_STRING: &str = "https://fosdem.org/2024/schedule/event/";

impl Queryable {
    pub async fn connect(
        db_host: &str,
        db_password: &str,
        openai_api_key: &str,
    ) -> Result<Queryable, Box<dyn std::error::Error>> {
        info!("Creating OpenAI Client");
        let openai_client = Client::new(openai_api_key.into());

        info!("Connecting to DB");
        let db_url = format!("postgres://postgres:{}@{}/postgres", db_password, db_host);
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;
        Ok(Queryable {
            openai_client,
            pool,
        })
    }

    pub async fn find_all_events(&self) -> Result<Vec<Event>, Box<dyn std::error::Error>> {
        info!("Running Query");
        let sql = "
    SELECT ev.title, ev.slug, ev.abstract
    FROM events_2 ev
    ORDER BY ev.title ASC;
    ";
        let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
        let mut events = vec![];
        for row in rows {
            let title: String = row.try_get("title")?;
            let slug: String = row.try_get("slug")?;
            let url = self.event_url(&slug)?;
            let r#abstract: String = row.try_get("abstract")?;
            events.push(Event {
                title,
                slug,
                url,
                r#abstract,
            });
        }
        Ok(events)
    }

    pub async fn search(
        &self,
        query: &str,
        limit: u8,
    ) -> Result<Vec<SearchItem>, Box<dyn std::error::Error>> {
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

        info!("Running Query");
        let sql = "
    SELECT ev.title, ev.slug, ev.abstract, em.embedding <-> ($1) AS distance
    FROM embedding_1 em JOIN events_2 ev ON ev.title = em.title
    ORDER BY em.embedding <-> ($1) LIMIT $2;
    ";
        let rows = sqlx::query(sql)
            .bind(embedding)
            .bind(limit as i32)
            .fetch_all(&self.pool)
            .await?;
        let mut entries = vec![];
        for row in rows {
            let title: String = row.try_get("title")?;
            let distance: f64 = row.try_get("distance")?;
            let slug: String = row.try_get("slug")?;
            let url = self.event_url(&slug)?;
            let r#abstract: String = row.try_get("abstract")?;
            entries.push(SearchItem {
                event: Event {
                    title,
                    slug,
                    url,
                    r#abstract,
                },
                distance,
            });
        }
        Ok(entries)
    }

    fn event_url(&self, slug: &str) -> Result<Url, Box<dyn std::error::Error>> {
        Ok(Url::parse(BASE_URL_STRING)?.join(slug)?)
    }
}
