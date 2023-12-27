use log::info;
use openai_dive::v1::api::Client;
use pgvector::Vector;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres, Row};
use url::Url;

use crate::openai::get_embedding;

pub struct Queryable {
    openai_client: Client,
    pool: Pool<Postgres>,
}

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

    pub async fn find(&self, query: &str, limit: u8) -> Result<(), Box<dyn std::error::Error>> {
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

        let base_url = Url::parse("https://fosdem.org/2024/schedule/event/")?;

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
        for row in rows {
            let title: &str = row.try_get("title")?;
            let distance: f64 = row.try_get("distance")?;
            let slug: &str = row.try_get("slug")?;
            let url = base_url.join(slug)?;
            let r#abstract: &str = row.try_get("abstract")?;
            println!("title: {} (distance: {:.3})", title, distance);
            println!("url: {}", url);
            // if args.r#abstract {
            println!("{}", r#abstract);
            println!();
            // }
        }
        Ok(())
    }
}
