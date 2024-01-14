use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Query, State},
    http::Method,
    response::Html,
    routing::get,
    Router,
};
use axum_valid::Valid;
use chrono::{Duration, NaiveDate, NaiveDateTime};
use serde::Deserialize;
use shared::queryable::{Event, Queryable, SearchItem};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::{debug, info};
use validator::Validate;

use crate::related::related;
use crate::state::AppState;

#[derive(Deserialize, Validate, Debug)]
struct Params {
    #[validate(length(min = 2, max = 100))]
    q: String,
    #[validate(range(min = 1, max = 20))]
    limit: u8,
}

#[derive(Template, Debug)]
#[template(path = "search.html")]
struct SearchTemplate {
    query: String,
    items: Vec<SearchItem>,
}

mod filters {
    pub fn distance_similarity(distance: &f64) -> ::askama::Result<String> {
        let similarity = 1.0 - distance;
        Ok(format!("{:.2}", similarity).into())
    }

    pub fn distance_icon(distance: &f64) -> ::askama::Result<String> {
        let similarity = 1.0 - distance;
        let assumed_max_typical_similarity = 0.60;
        let opacity = (similarity / assumed_max_typical_similarity).min(1.0f64);
        Ok(format!(
            "<i class=\"fa-solid fa-circle\" style=\"opacity: {}\"></i>",
            opacity
        )
        .into())
    }
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

#[tracing::instrument]
async fn index() -> Html<String> {
    let page = IndexTemplate {};
    let html = page.render().unwrap();
    Html(html)
}

#[tracing::instrument(skip(state))]
async fn search(
    State(state): State<AppState>,
    Valid(Query(params)): Valid<Query<Params>>,
) -> axum::response::Result<Html<String>> {
    info!("search params: {:?}", params);
    match state.queryable.search(&params.q, params.limit, true).await {
        Ok(items) => {
            let page = SearchTemplate {
                query: params.q,
                items,
            };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("search failed".into()),
    }
}

#[derive(Template, Debug)]
#[template(path = "now_and_next.html")]
struct NowAndNextTemplate {
    now: NaiveDateTime,
    current_events: Vec<Event>,
}

#[tracing::instrument(skip(state))]
async fn now_and_next(State(state): State<AppState>) -> axum::response::Result<Html<String>> {
    match state.queryable.load_all_events().await {
        Ok(all_events) => {
            let current_day = NaiveDate::from_ymd_opt(2024, 2, 3).unwrap();
            let now = current_day.and_hms_opt(11, 0, 0).unwrap();
            let one_hour_from_now = now + Duration::hours(1);
            debug!("Current hour: {} -> {}", now, one_hour_from_now);
            let mut current_events = vec![];
            for event in all_events.iter() {
                let starting_time = event.date.and_time(event.start);
                let ending_time = starting_time + Duration::minutes(event.duration.into());
                debug!(
                    "event: {} -> {}, {}, {}",
                    starting_time,
                    ending_time,
                    now <= ending_time,
                    ending_time <= one_hour_from_now
                );
                if now <= ending_time && ending_time <= one_hour_from_now {
                    current_events.push(event.clone());
                }
            }
            debug!("Found {} current events", current_events.len());
            let page = NowAndNextTemplate {
                now,
                current_events,
            };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("failed".into()),
    }
}

pub async fn router(openai_api_key: &str, db_host: &str, db_key: &str) -> Router {
    let state = AppState {
        queryable: Arc::new(
            Queryable::connect(&db_host, &db_key, &openai_api_key)
                .await
                .unwrap(),
        ),
    };

    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    let router = Router::new()
        .route("/", get(index))
        .route("/search", get(search))
        .route("/connections/", get(related))
        .route("/now/", get(now_and_next))
        .layer(cors)
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state);

    router
}
