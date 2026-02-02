use askama::Template;
use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
};
use axum_extra::extract::Host;

use crate::state::AppState;

#[derive(Template)]
#[template(path = "blog_list.html")]
struct BlogListTemplate<'a> {
    current_fosdem: shared::model::CurrentFosdem,
    posts: &'a [::blog::Post],
}

#[derive(Template)]
#[template(path = "blog_post.html")]
struct BlogPostTemplate<'a> {
    current_fosdem: shared::model::CurrentFosdem,
    post: &'a ::blog::Post,
}

#[tracing::instrument(skip(state))]
pub async fn blog_list(State(state): State<AppState>) -> Html<String> {
    let page = BlogListTemplate {
        current_fosdem: state.current_fosdem.clone(),
        posts: state.blog_index.all_posts(),
    };
    Html(page.render().unwrap())
}

#[tracing::instrument(skip(state))]
pub async fn blog_post(Path(date): Path<String>, State(state): State<AppState>) -> Response {
    match state.blog_index.find_by_slug(&date) {
        Some(post) => {
            let page = BlogPostTemplate {
                current_fosdem: state.current_fosdem.clone(),
                post,
            };
            Html(page.render().unwrap()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "Post not found").into_response(),
    }
}

#[tracing::instrument(skip(state))]
pub async fn rss_feed(Host(host): Host, State(state): State<AppState>) -> Response {
    let base_url = base_url_from_host(&host);

    let mut channel = rss::ChannelBuilder::default()
        .title(format!("FOSDEM {}", state.current_fosdem.year))
        .link(format!("{}/blog/", base_url))
        .description("Updates on fosdem.houseofmoran.io")
        .build();

    let items: Vec<rss::Item> = state
        .blog_index
        .all_posts()
        .iter()
        .map(|post| {
            rss::ItemBuilder::default()
                .title(Some(post.title.clone()))
                .link(Some(format!("{}{}", base_url, post.url_path())))
                .pub_date(Some(
                    post.date
                        .and_hms_opt(12, 0, 0)
                        .unwrap()
                        .and_utc()
                        .to_rfc2822(),
                ))
                .content(Some(post.content_html.clone()))
                .build()
        })
        .collect();

    channel.set_items(items);

    let xml = channel.to_string();

    (
        [(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")],
        xml,
    )
        .into_response()
}

fn base_url_from_host(host: &str) -> String {
    if host == "localhost" || host.starts_with("localhost:") {
        format!("http://{}", host)
    } else {
        format!("https://{}", host)
    }
}
