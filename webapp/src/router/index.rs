
use askama::Template;
use axum::response::Html;

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {}

#[tracing::instrument]
pub async fn index() -> Html<String> {
    let page = IndexTemplate {};
    let html = page.render().unwrap();
    Html(html)
}