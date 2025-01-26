use askama::Template;
use axum::response::Html;

use serde::Serialize;
use url::Url;

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
struct RelatedTemplate {}

#[tracing::instrument]
pub async fn related() -> Html<String> {
    let page: RelatedTemplate = RelatedTemplate {};
    let html = page.render().unwrap();
    Html(html)
}
