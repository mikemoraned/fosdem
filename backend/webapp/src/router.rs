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
use chrono::{Duration, NaiveDate};
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
    use unicode_segmentation::UnicodeSegmentation;

    fn count_graphemes(s: &str) -> usize {
        UnicodeSegmentation::graphemes(s, true).into_iter().count()
    }

    pub fn truncate_title(title: &String, max_size: usize) -> ::askama::Result<String> {
        if count_graphemes(&title) <= max_size {
            return Ok(title.clone());
        }

        let suffix = " …";
        let suffix_length = count_graphemes(suffix);
        let mut available = max_size - suffix_length;

        let chunks = title.split_word_bounds().collect::<Vec<&str>>();
        let mut limited = vec![];
        for chunk in chunks {
            let chunk_length = count_graphemes(chunk);
            if available >= chunk_length {
                limited.push(chunk);
                available -= chunk_length;
            } else {
                break;
            }
        }
        limited.push(suffix);
        Ok(limited.join(""))
    }

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
    current_events: Vec<Event>,
    selected_event: Event,
    next_events: Vec<Event>,
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
                if now <= ending_time && ending_time <= one_hour_from_now {
                    current_events.push(event.clone());
                }
            }
            debug!("Found {} current events", current_events.len());
            let selected_event = current_events[0].clone();
            let selected_event_end_time = selected_event.date.and_time(selected_event.start)
                + Duration::minutes(selected_event.duration.into());
            let one_hour_after_selected_event_end_time =
                selected_event_end_time + Duration::hours(1);
            let mut next_events = vec![];
            for event in all_events.iter() {
                let starting_time = event.date.and_time(event.start);
                if selected_event_end_time <= starting_time
                    && starting_time <= one_hour_after_selected_event_end_time
                {
                    next_events.push(event.clone());
                }
            }
            let page = NowAndNextTemplate {
                current_events,
                selected_event,
                next_events,
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
