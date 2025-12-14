use askama::Template;
use axum::{
    extract::{Path, State},
    response::Html,
};

use crate::filters;
use crate::state::AppState;
use serde::Deserialize;
use shared::inmemory_openai::InMemoryOpenAIQueryable;
use shared::model::{Event, SearchItem};
use shared::queryable::Queryable;
use validator::Validate;

#[derive(Deserialize, Validate, Debug)]
#[allow(dead_code)]
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
    // TODO: this is all a bit contorted, for a couple of reasons:
    // - InMemoryOpenAIQueryable should really natively support finding related events, as opposed to
    // us having to find by event.title in `find_related_events`
    // - When we are doing the two calls we can't have a nested await as `dyn StdError` isn't
    // `Send`, which Rust thinks it needs to be on the second call
    // Best thing is to move more of the responsibility into `InMemoryOpenAIQueryable` out of here
    let possible_event: Option<Event> =
        (state.queryable.find_event_by_id(event_id).await).unwrap_or_default();
    if let Some(event) = possible_event {
        let current_event = None;
        let related = find_related_events(&state.queryable, &event).await;
        let page = EventTemplate {
            event,
            related,
            current_event,
        };
        let html = page.render().unwrap();
        Ok(Html(html))
    } else {
        Err("failed".into())
    }
}

async fn find_related_events(
    queryable: &InMemoryOpenAIQueryable,
    event: &Event,
) -> Option<Vec<SearchItem>> {
    (queryable.find_related_events(&event.title, 10).await).ok()
}
