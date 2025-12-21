use std::{path::PathBuf, sync::Arc};

use axum::{http::Method, routing::get, Router};
use content::video_index::VideoIndex;
use shared::inmemory_openai::InMemoryOpenAIQueryable;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};

use crate::state::AppState;

mod bookmark;
mod event;
mod index;
mod next;
pub mod related;
mod room;
mod search;
mod video;

pub async fn app_state(
    openai_api_key: &str,
    model_dir: &std::path::Path,
    video_content_dir: &Option<PathBuf>,
    current_year: u32,
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
        current_fosdem: shared::model::CurrentFosdem { year: current_year },
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
        .route("/event/{event_in_year_id}/", get(event::event_2025))
        .route("/{year}/event/{event_in_year_id}/", get(event::event))
        .route("/room/{room_id}/", get(room::room))
        .route("/{year}/video/{event_in_year_id}/", get(video::event_video))
        .route(
            "/{year}/video/{event_in_year_id}/captions.vtt",
            get(video::event_video_webvtt),
        )
        .layer(cors)
        .nest_service("/assets", ServeDir::new("assets"))
        .with_state(state)
}
