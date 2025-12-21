use askama::Template;
use axum::{extract::State, response::Html};

use crate::state::AppState;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    current_fosdem: shared::model::CurrentFosdem,
}

#[tracing::instrument]
pub async fn index(State(state): State<AppState>) -> Html<String> {
    let page = IndexTemplate {
        current_fosdem: state.current_fosdem.clone(),
    };
    let html = page.render().unwrap();
    Html(html)
}
