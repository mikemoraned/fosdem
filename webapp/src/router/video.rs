use askama::Template;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse},
};

use serde::Deserialize;
use shared::model::{self, Event};
use validator::Validate;

use crate::state::AppState;
use shared::queryable::Queryable;

#[derive(Deserialize, Validate, Debug)]
#[allow(dead_code)]
pub struct EventVideoParams {
    #[validate(range(min = 1, max = 20000))]
    id: Option<u32>,
}

#[derive(Template, Debug)]
#[template(path = "event_video.html")]
struct EventVideoTemplate {
    event: Event,
}

#[tracing::instrument(skip(state))]
pub async fn event_video(
    State(state): State<AppState>,
    Path((year, event_in_year_id)): Path<(u32, u32)>,
) -> axum::response::Result<Html<String>> {
    match state
        .queryable
        .find_event_by_id(model::EventId::new(year, event_in_year_id))
        .await
    {
        Ok(Some(event)) => {
            let page = EventVideoTemplate { event };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        _ => Err("failed".into()),
    }
}

#[tracing::instrument(skip(state))]
pub async fn event_video_webvtt(
    State(state): State<AppState>,
    Path((year, event_in_year_id)): Path<(u32, u32)>,
) -> impl IntoResponse {
    match state
        .video_index
        .webvtt_for_event_id(model::EventId::new(year, event_in_year_id))
    {
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
