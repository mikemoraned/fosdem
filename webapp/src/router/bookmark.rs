
use askama::Template;
use axum::{
    extract::State,
    response::Html,
};

use shared::model::Event;

use crate::state::AppState;

#[derive(Template, Debug)]
#[template(path = "bookmarks.html")]
struct BookmarksTemplate {
    events: Vec<Event>,
    current_event: Option<Event>, // TODO: remove this
}

#[tracing::instrument(skip(_state))]
pub async fn bookmarks(
    State(_state): State<AppState>
) -> axum::response::Result<Html<String>> {
    let page = BookmarksTemplate {
        events: vec![],
        current_event: None,
    };
    let html = page.render().unwrap();
    Ok(Html(html))
}
