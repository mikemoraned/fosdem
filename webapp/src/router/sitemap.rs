use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use axum_extra::extract::Host;
use shared::queryable::Queryable;
use sitemap_rs::url::{ChangeFrequency, Url};
use sitemap_rs::url_set::UrlSet;

use crate::state::AppState;

#[tracing::instrument(skip(state))]
pub async fn sitemap(Host(host): Host, State(state): State<AppState>) -> Response {
    let base_url = if let Ok(url) = base_url_from_host(&host) {
        url
    } else {
        return (StatusCode::BAD_REQUEST, "Invalid host").into_response();
    };
    let events = match state.queryable.load_all_events().await {
        Ok(events) => events,
        Err(_) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to load events").into_response()
        }
    };
    let event_urls = events.into_iter().map(|event| {
        let mut builder = Url::builder(format!(
            "{}/{}/event/{}/",
            base_url,
            event.id.year(),
            event.id.event_in_year()
        ));
        builder.last_modified(state.current_fosdem.updated_at.fixed_offset());

        if event.year == state.current_fosdem.year {
            builder
                .change_frequency(ChangeFrequency::Daily)
                .priority(1.0);
        } else {
            builder
                .change_frequency(ChangeFrequency::Weekly)
                .priority(0.5);
        }
        builder.build().unwrap()
    });

    let blog_urls = state.blog_index.all_posts().iter().map(|post| {
        Url::builder(format!("{}{}", base_url, post.url_path()))
            .change_frequency(ChangeFrequency::Monthly)
            .priority(0.6)
            .build()
            .unwrap()
    });

    let urls: Vec<Url> = event_urls.chain(blog_urls).collect();

    let url_set = match UrlSet::new(urls) {
        Ok(set) => set,
        Err(_) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to create sitemap",
            )
                .into_response()
        }
    };

    let mut buf = Vec::new();
    if url_set.write(&mut buf).is_err() {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to write sitemap").into_response();
    }

    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        buf,
    )
        .into_response()
}

fn base_url_from_host(host: &str) -> Result<String, Box<dyn std::error::Error>> {
    let allowlist = ["fosdem.houseofmoran.io", "fosdem2024-staging.fly.dev"];
    if host == "localhost" || host.starts_with("localhost:") {
        Ok(format!("http://{}", host))
    } else if allowlist.contains(&host) {
        Ok(format!("https://{}", host))
    } else {
        Err("Host not allowed".into())
    }
}
