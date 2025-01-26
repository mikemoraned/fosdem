

use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
};

use serde::Deserialize;
use shared::model::{Event, SearchItem};
use validator::Validate;

use crate::filters;
use crate::state::AppState;
use shared::queryable::Queryable;

#[derive(Deserialize, Validate, Debug)]
pub struct EventParams {
    #[validate(range(min = 1, max = 20000))]
    id: Option<u32>,
}

#[derive(Template, Debug)]
#[template(path = "event.html")]
struct EventTemplate {
    pub event: Event,
    pub related: Option<Vec<SearchItem>>,
    pub current_event: Option<Event>, // TODO: remove this
}

#[derive(Deserialize, Debug)]
pub struct EventIdParam(u32);

#[tracing::instrument(skip(state))]
pub async fn event(
    State(state): State<AppState>,
    Path(EventIdParam(event_id)): Path<EventIdParam>,
) -> axum::response::Result<Html<String>> {
    match state.queryable.find_event_by_id(event_id).await {
        Ok(Some(event)) => {
            let related = None;
            let current_event = None;
            let page = EventTemplate { event, related, current_event };
            let html = page.render().unwrap();
            Ok(Html(html))
        }
        _ => Err("failed".into()),
    }
}

