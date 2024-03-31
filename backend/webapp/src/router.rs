use std::{path::PathBuf, sync::Arc};

use askama::Template;
use axum::{
    extract::{Path, Query, State},
    http::{header, Method, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use axum_valid::Valid;

use content::video_index::VideoIndex;
use query::queryable::Queryable;
use query::{inmemory_openai::InMemoryOpenAIQueryable, queryable::SearchKind};
use serde::Deserialize;
use shared::model::{Event, NextEvents, NextEventsContext, SearchItem};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::info;
use validator::Validate;

use crate::filters;
use crate::related::related;
use crate::state::AppState;

#[derive(Deserialize, Validate, Debug)]
struct SearchParams {
    #[validate(length(min = 2, max = 100))]
    q: String,
    #[validate(range(min = 1, max = 20))]
    limit: u8,
    #[serde(default)]
    kind: SearchKind,
}

#[derive(Template, Debug)]
#[template(path = "search.html")]
struct SearchTemplate {
    query: String,
    items: Vec<SearchItem>,
    current_event: Option<Event>,
    kind: SearchKind,
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    kind: SearchKind,
}

#[tracing::instrument]
async fn index() -> Html<String> {
    let page = IndexTemplate {
        kind: SearchKind::default(),
    };
    let html = page.render().unwrap();
    Html(html)
}

#[tracing::instrument(skip(state))]
async fn search(
    State(state): State<AppState>,
    Valid(Query(params)): Valid<Query<SearchParams>>,
) -> axum::response::Result<Html<String>> {
    info!("search params: {:?}", params);
    match state
        .queryable
        .search(&params.q, params.limit, &params.kind, true)
        .await
    {
        Ok(items) => {
            let page = SearchTemplate {
                query: params.q,
                items,
                current_event: None,
                kind: params.kind.clone(),
            };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("search failed".into()),
    }
}

#[derive(Deserialize, Validate, Debug)]
struct NextParams {
    #[validate(range(min = 1, max = 20000))]
    id: Option<u32>,
}

#[derive(Template, Debug)]
#[template(path = "now_and_next.html")]
struct NowAndNextTemplate {
    next: NextEvents,
    current_event: Option<Event>,
}

#[tracing::instrument(skip(state))]
async fn next(
    State(state): State<AppState>,
    Valid(Query(params)): Valid<Query<NextParams>>,
) -> axum::response::Result<Html<String>> {
    let context = match params.id {
        Some(event_id) => NextEventsContext::EventId(event_id),
        None => NextEventsContext::Now,
    };
    match state.queryable.find_next_events(context).await {
        Ok(next) => {
            let page = NowAndNextTemplate {
                next: next.clone(),
                current_event: Some(next.selected.clone()),
            };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("failed".into()),
    }
}

#[tracing::instrument(skip(state))]
async fn next_after_event(
    State(state): State<AppState>,
    Path(event_id): Path<u32>,
) -> axum::response::Result<Html<String>> {
    match state
        .queryable
        .find_next_events(NextEventsContext::Now)
        .await
    {
        Ok(next) => {
            let page = NowAndNextTemplate {
                next: next.clone(),
                current_event: Some(next.selected.clone()),
            };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        Err(_) => Err("failed".into()),
    }
}

#[derive(Deserialize, Validate, Debug)]
struct EventVideoParams {
    #[validate(range(min = 1, max = 20000))]
    id: Option<u32>,
}

#[derive(Template, Debug)]
#[template(path = "event_video.html")]
struct EventVideoTemplate {
    event: Event,
}

#[derive(Deserialize, Debug)]
struct EventIdParam(u32);

#[tracing::instrument(skip(state))]
async fn event_video(
    State(state): State<AppState>,
    Path(EventIdParam(event_id)): Path<EventIdParam>,
) -> axum::response::Result<Html<String>> {
    match state.queryable.find_event_by_id(event_id).await {
        Ok(Some(event)) => {
            let page = EventVideoTemplate { event };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        _ => Err("failed".into()),
    }
}

#[tracing::instrument(skip(state))]
async fn event_video_webvtt(
    State(state): State<AppState>,
    Path(EventIdParam(event_id)): Path<EventIdParam>,
) -> impl IntoResponse {
    match state.video_index.webvtt_for_event_id(event_id) {
        Some(webvtt) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/vtt")],
            webvtt.render(),
        ),
        None => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            "missing".into(),
        ),
    }
}

pub async fn app_state(
    openai_api_key: &str,
    model_dir: &std::path::Path,
    video_content_dir: &Option<PathBuf>,
) -> AppState {
    AppState {
        queryable: Arc::new(
            InMemoryOpenAIQueryable::connect(model_dir, openai_api_key)
                .await
                .unwrap(),
        ),
        video_index: Arc::new(if let Some(base_path) = video_content_dir {
            VideoIndex::from_content_area(base_path).unwrap()
        } else {
            VideoIndex::empty_index()
        }),
    }
}

pub async fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        // allow requests from any origin
        .allow_origin(Any);

    Router::new()
        .route("/", get(index))
        .route("/search", get(search))
        .route("/connections/", get(related))
        .route("/next/", get(next))
        .route("/video/:event_id/", get(event_video))
        .route("/video/:event_id/captions.vtt", get(event_video_webvtt))
        .layer(cors)
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state)
}
