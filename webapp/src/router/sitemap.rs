use axum::{
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sitemap_rs::url::{ChangeFrequency, Url};
use sitemap_rs::url_set::UrlSet;

use crate::state::AppState;

#[tracing::instrument(skip(state))]
pub async fn sitemap(State(state): State<AppState>) -> Response {
    let urls = vec![
        Url::builder("https://example.com/".to_string())
            .last_modified(Utc::now().fixed_offset())
            .change_frequency(ChangeFrequency::Daily)
            .priority(1.0)
            .build()
            .unwrap(),
        Url::builder("https://example.com/about".to_string())
            .change_frequency(ChangeFrequency::Monthly)
            .priority(0.8)
            .build()
            .unwrap(),
        Url::builder("https://example.com/blog".to_string())
            .change_frequency(ChangeFrequency::Weekly)
            .priority(0.9)
            .build()
            .unwrap(),
    ];

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
