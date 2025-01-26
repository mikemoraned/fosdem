use std::{path::PathBuf, sync::Arc};


use axum::{http::Method, routing::get, Router};
use content::video_index::VideoIndex;
use shared::
    inmemory_openai::InMemoryOpenAIQueryable
;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::state::AppState;

pub mod related;
mod index;
mod search;
mod next;
mod video;
mod bookmark;
mod event;

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
        .route("/bookmarks", get(bookmark::bookmarks))
        .route("/connections/", get(related::related))
        .route("/next/", get(next::next))
        .route("/event/:event_id/", get(event::event))
        .route("/video/:event_id/", get(video::event_video))
        .route("/video/:event_id/captions.vtt", get(video::event_video_webvtt))
        .layer(cors)
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state)
}
