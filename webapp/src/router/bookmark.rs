
use askama::Template;
use axum::{
    extract::State,
    response::Html,
};

use shared::{model::Event, queryable::Queryable};

use crate::state::AppState;

#[derive(Template, Debug)]
#[template(path = "bookmarks.html")]
struct BookmarksTemplate {
    events: Vec<Event>,
    current_event: Option<Event>, // TODO: remove this
}

#[tracing::instrument(skip(state))]
pub async fn bookmarks(
    State(state): State<AppState>
) -> axum::response::Result<Html<String>> {
    let mut events = state.queryable.load_all_events().await.unwrap();
    events.sort_by_key(|e| e.starting_time());
    let page = BookmarksTemplate {
        events,
        current_event: None,
    };
    let html = page.render().unwrap();
    Ok(Html(html))
}
