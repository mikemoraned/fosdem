use askama::Template;
use axum::{extract::State, response::Html};

use crate::state::AppState;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    current_fosdem: shared::model::CurrentFosdem,
    default_year: Option<u32>,
}

#[tracing::instrument]
pub async fn index(State(state): State<AppState>) -> Html<String> {
    let page = IndexTemplate {
        current_fosdem: state.current_fosdem.clone(),
        default_year: None,
    };
    let html = page.render().unwrap();
    Html(html)
}
