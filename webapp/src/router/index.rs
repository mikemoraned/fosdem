use askama::Template;
use axum::{extract::State, response::Html};

use crate::state::AppState;

const RECENT_POSTS_COUNT: usize = 3;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    current_fosdem: shared::model::CurrentFosdem,
    default_year: Option<u32>,
    recent_posts: Vec<&'a ::blog::Post>,
}

#[tracing::instrument(skip(state))]
pub async fn index(State(state): State<AppState>) -> Html<String> {
    let page = IndexTemplate {
        current_fosdem: state.current_fosdem.clone(),
        default_year: None,
        recent_posts: state.blog_index.recent_posts(RECENT_POSTS_COUNT),
    };
    let html = page.render().unwrap();
    Html(html)
}
