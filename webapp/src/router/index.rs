use askama::Template;
use axum::{extract::State, response::Html};
use shared::summary::{load_summary, DataSummary};
use tracing::error;

use crate::state::AppState;

const RECENT_POSTS_COUNT: usize = 3;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    current_fosdem: shared::model::CurrentFosdem,
    default_year: Option<u32>,
    recent_posts: Vec<&'a ::blog::Post>,
    summary: DataSummary,
}

#[tracing::instrument(skip(state))]
pub async fn index(State(state): State<AppState>) -> Html<String> {
    let summary = load_summary(state.queryable.as_ref()).await.unwrap_or_else(|e| {
        error!("Failed to load summary: {}", e);
        DataSummary {
            by_year: std::collections::BTreeMap::new(),
        }
    });
    let page = IndexTemplate {
        current_fosdem: state.current_fosdem.clone(),
        default_year: None,
        recent_posts: state.blog_index.recent_posts(RECENT_POSTS_COUNT),
        summary,
    };
    let html = page.render().unwrap();
    Html(html)
}
