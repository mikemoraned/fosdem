use askama::Template;
use axum::{extract::State, response::Html};

use serde::Serialize;
use url::Url;

use crate::state::AppState;

#[derive(Serialize)]
pub struct D3Force {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

#[derive(Serialize)]
pub struct Node {
    pub index: usize,
    pub title: String,
    pub local_path: String,
    pub url: Url,
    pub sojourner_url: Url,
    pub time_slot: usize,
    pub day: String,
    pub start: String,
}

#[derive(Serialize)]
pub struct Link {
    pub source: usize,
    pub target: usize,
    pub distance: f64,
}

#[derive(Template, Debug)]
#[template(path = "connections.html")]
struct RelatedTemplate {
    current_fosdem: shared::model::CurrentFosdem,
}

#[tracing::instrument]
pub async fn related(State(state): State<AppState>) -> Html<String> {
    let page: RelatedTemplate = RelatedTemplate {
        current_fosdem: state.current_fosdem.clone(),
    };
    let html = page.render().unwrap();
    Html(html)
}
