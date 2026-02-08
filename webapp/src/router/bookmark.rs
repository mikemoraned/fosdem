use askama::Template;
use axum::{extract::State, response::Html};

use shared::{model::Event, queryable::Queryable};

use crate::filters;
use crate::state::AppState;

#[derive(Template, Debug)]
#[template(path = "bookmarks.html")] // TODO: revert to bookmarks.html after testing
struct BookmarksTemplate {
    events: Vec<Event>,
    current_event: Option<Event>, // TODO: remove this
    current_fosdem: shared::model::CurrentFosdem,
}

#[tracing::instrument(skip(state))]
pub async fn bookmarks(State(state): State<AppState>) -> axum::response::Result<Html<String>> {
    let mut events = state.queryable.load_all_events().await.unwrap();
    events.sort_by_key(|e| e.starting_time());
    let page = BookmarksTemplate {
        events,
        current_event: None,
        current_fosdem: state.current_fosdem.clone(),
    };
    let html = page.render().unwrap();
    Ok(Html(html))
}
