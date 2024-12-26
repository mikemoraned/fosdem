use std::{path::PathBuf, sync::Arc};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::{header, Method, StatusCode},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};

use content::video_index::VideoIndex;
use serde::Deserialize;
use shared::{
    inmemory_openai::InMemoryOpenAIQueryable,
    model::Event,
};
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use validator::Validate;

use crate::related::related;
use crate::state::AppState;
use shared::queryable::Queryable;

mod index;
mod search;
mod next;

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
        .route("/", get(index::index))
        .route("/search", get(search::search))
        .route("/connections/", get(related))
        .route("/next/", get(next::next))
        .route("/video/:event_id/", get(event_video))
        .route("/video/:event_id/captions.vtt", get(event_video_webvtt))
        .layer(cors)
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state)
}
