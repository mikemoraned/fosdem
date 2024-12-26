
use askama::Template;
use axum::{
    extract::{Query, State},
    response::Html,
};
use axum_valid::Valid;

use serde::Deserialize;
use shared::
    model::{Event, NextEvents, NextEventsContext}
;
use validator::Validate;

use crate::filters;
use crate::state::AppState;
use shared::queryable::Queryable;

#[derive(Deserialize, Validate, Debug)]
pub struct NextParams {
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
pub async fn next(
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

