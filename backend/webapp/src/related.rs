use askama::Template;
use axum::response::Html;
use axum::{extract::State, Json};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct D3Force {
    pub nodes: Vec<Node>,
    pub links: Vec<Link>,
}

#[derive(Serialize)]
pub struct Node {}

#[derive(Serialize)]
pub struct Link {}

#[derive(Template, Debug)]
#[template(path = "related.html")]
struct RelatedTemplate {}

#[tracing::instrument]
pub async fn related_data(State(state): State<AppState>) -> Json<D3Force> {
    Json(D3Force {
        nodes: vec![],
        links: vec![],
    })
}

#[tracing::instrument]
pub async fn related() -> Html<String> {
    let page: RelatedTemplate = RelatedTemplate {};
    let html = page.render().unwrap();
    Html(html)
}
