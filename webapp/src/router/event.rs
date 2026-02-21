use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};
use tracing::warn;

use crate::filters;
use crate::state::AppState;
use serde::Deserialize;
use shared::model::{Event, SearchItem};
use shared::queryable::Queryable;
use shared::{inmemory_openai::InMemoryOpenAIQueryable, model};

#[derive(Template)]
#[template(source = "{{ content|safe }}", ext = "html")]
struct AbstractTemplate {
    content: String,
}

#[derive(Template)]
#[template(path = "event_card.html")]
struct EventCardTemplate {
    event: Event,
    current_event: Option<Event>,
}
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
    pub current_fosdem: shared::model::CurrentFosdem,
}

#[tracing::instrument(skip(state))]
pub async fn event_2025(
    State(state): State<AppState>,
    Path(event_in_year_id): Path<u32>,
) -> Result<Html<String>, StatusCode> {
    event(State(state), Path((2025, event_in_year_id))).await
}

#[tracing::instrument(skip(state))]
pub async fn event(
    State(state): State<AppState>,
    Path((year, event_in_year_id)): Path<(u32, u32)>,
) -> Result<Html<String>, StatusCode> {
    // TODO: this is all a bit contorted, for a couple of reasons:
    // - InMemoryOpenAIQueryable should really natively support finding related events, as opposed to
    // us having to find by event.title in `find_related_events`
    // - When we are doing the two calls we can't have a nested await as `dyn StdError` isn't
    // `Send`, which Rust thinks it needs to be on the second call
    // Best thing is to move more of the responsibility into `InMemoryOpenAIQueryable` out of here
    let event_id = model::EventId::new(year, event_in_year_id);
    let possible_event: Option<Event> =
        (state.queryable.find_event_by_id(event_id).await).unwrap_or_default();
    if let Some(event) = possible_event {
        let current_event = None;
        let related = find_related_events(&state.queryable, &event).await;
        let page = EventTemplate {
            event,
            related,
            current_event,
            current_fosdem: state.current_fosdem.clone(),
        };
        let html = page.render().unwrap();
        Ok(Html(html))
    } else {
        warn!("Could not find event: {}", event_id);
        Err(StatusCode::NOT_FOUND)
    }
}

async fn find_related_events(
    queryable: &InMemoryOpenAIQueryable,
    event: &Event,
) -> Option<Vec<SearchItem>> {
    (queryable.find_related_events(&event.title, 10, None).await).ok()
}

#[tracing::instrument(skip(state))]
pub async fn event_abstract(
    State(state): State<AppState>,
    Path((year, event_in_year_id)): Path<(u32, u32)>,
) -> Result<Html<String>, StatusCode> {
    let event_id = model::EventId::new(year, event_in_year_id);
    match state
        .queryable
        .find_event_by_id(event_id)
        .await
        .unwrap_or_default()
    {
        Some(event) => {
            let page = AbstractTemplate {
                content: event.r#abstract,
            };
            Ok(Html(page.render().unwrap()))
        }
        None => {
            warn!("Could not find event abstract: {}", event_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}

#[tracing::instrument(skip(state))]
pub async fn event_card(
    State(state): State<AppState>,
    Path((year, event_in_year_id)): Path<(u32, u32)>,
) -> Result<Html<String>, StatusCode> {
    let event_id = model::EventId::new(year, event_in_year_id);
    match state
        .queryable
        .find_event_by_id(event_id)
        .await
        .unwrap_or_default()
    {
        Some(event) => {
            let page = EventCardTemplate {
                event,
                current_event: None,
            };
            Ok(Html(page.render().unwrap()))
        }
        None => {
            warn!("Could not find event card: {}", event_id);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
