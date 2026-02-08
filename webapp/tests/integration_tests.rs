use reqwest::blocking::{Client, Response};
use reqwest::Result;
use shared::model::EventId;
use std::env;
use test_shared::{
    EVENT_ID_2025, EVENT_ID_2025_ABSTRACT_PATH, EVENT_ID_2025_BACKWARDS_COMPATIBLE_PATH,
    EVENT_ID_2025_CANONICAL_PATH, EVENT_ID_2025_CONTENT_SAMPLE, EVENT_ID_2026,
    EVENT_ID_2026_ABSTRACT_PATH, EVENT_ID_2026_CANONICAL_PATH, EVENT_ID_2026_CONTENT_SAMPLE,
    SEARCH_TERM,
};

fn get_base_url() -> String {
    env::var("TEST_BASE_URL").unwrap_or_else(|_| "http://localhost:8000".to_string())
}

fn client() -> Client {
    Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client")
}

fn exists_at_path(path: &str) -> Result<Response> {
    let url = format!("{}{}", get_base_url(), path);
    let response = client().get(url).send()?;

    assert_eq!(response.status(), 200);

    Ok(response)
}

#[test]
fn test_homepage_exists() {
    exists_at_path("/").expect("exists");
}

#[test]
fn test_sitemap_exists() {
    exists_at_path("/sitemap.xml").expect("exists");
}

#[test]
fn test_homepage_contains_expected_content() {
    let response = exists_at_path("/").expect("exists");

    let body = response.text().expect("Failed to read body");
    assert!(body.contains("<!DOCTYPE html>"), "doctype");
    assert!(body.contains("<title>FOSDEM 2026</title>"), "title");
    assert!(
        body.contains("href=\"https://fosdem.org/2026/\""),
        "2026 link"
    );
    assert!(
        body.contains(
            "All content such as talks and biographies is the sole responsibility of the speaker."
        ),
        "footer"
    );
}

fn event_id_as_anchor_text(event_id: EventId) -> String {
    format!(
        "<a name=\"{}-{}\"></a>",
        event_id.year(),
        event_id.event_in_year()
    )
}

fn assert_any_year_search(path_and_query: &str) {
    let response = client()
        .get(format!("{}{}", get_base_url(), path_and_query))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body = response.text().expect("Failed to read body");
    assert!(body.contains(&event_id_as_anchor_text(EVENT_ID_2025)));
    assert!(body.contains(&event_id_as_anchor_text(EVENT_ID_2026)));
}

#[test]
fn test_search_for_any_year_no_year_param_specified() {
    assert_any_year_search(&format!("/search?q={SEARCH_TERM}&limit=20"));
}

#[test]
fn test_search_for_any_year_with_year_param_as_empty_string() {
    assert_any_year_search(&format!("/search?q={SEARCH_TERM}&limit=20&year="));
}

#[test]
fn test_search_for_2025_only() {
    let response = client()
        .get(format!(
            "{}/search?q={}&limit=20&year=2025",
            get_base_url(),
            SEARCH_TERM
        ))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);

    let body = response.text().expect("Failed to read body");
    assert!(body.contains(&event_id_as_anchor_text(EVENT_ID_2025)));
    assert!(!body.contains(&event_id_as_anchor_text(EVENT_ID_2026)));
}

fn assert_2025_content(response: Response) {
    let body = response.text().expect("Failed to read body");
    // Abstract is lazy-loaded, so check for skeleton placeholder instead
    assert!(body.contains("skeleton-lines"));
    assert!(body.contains(EVENT_ID_2025_ABSTRACT_PATH));
}

#[test]
fn test_2025_content_in_backwards_compatible_place() {
    let response = exists_at_path(EVENT_ID_2025_BACKWARDS_COMPATIBLE_PATH).expect("exists");
    assert_2025_content(response);
}

#[test]
fn test_2025_content_in_canonical_place() {
    let response = exists_at_path(EVENT_ID_2025_CANONICAL_PATH).expect("exists");
    assert_2025_content(response);
}

fn assert_2026_content(response: Response) {
    let body = response.text().expect("Failed to read body");
    // Abstract is lazy-loaded, so check for skeleton placeholder instead
    assert!(body.contains("skeleton-lines"));
    assert!(body.contains(EVENT_ID_2026_ABSTRACT_PATH));
}

#[test]
fn test_2026_content_in_canonical_place() {
    let response = exists_at_path(EVENT_ID_2026_CANONICAL_PATH).expect("exists");
    assert_2026_content(response);
}

#[test]
fn test_2025_abstract_endpoint() {
    let response = exists_at_path(EVENT_ID_2025_ABSTRACT_PATH).expect("exists");
    let body = response.text().expect("Failed to read body");
    assert!(body.contains(EVENT_ID_2025_CONTENT_SAMPLE));
    // Abstract endpoint returns just the abstract HTML, not a full page
    assert!(!body.contains("<!DOCTYPE html>"));
}

#[test]
fn test_2026_abstract_endpoint() {
    let response = exists_at_path(EVENT_ID_2026_ABSTRACT_PATH).expect("exists");
    let body = response.text().expect("Failed to read body");
    assert!(body.contains(EVENT_ID_2026_CONTENT_SAMPLE));
    // Abstract endpoint returns just the abstract HTML, not a full page
    assert!(!body.contains("<!DOCTYPE html>"));
}

#[test]
fn test_event_404_for_nonexistent_event() {
    let response = client()
        .get(format!("{}/2026/event/99999/", get_base_url()))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[test]
fn test_abstract_404_for_nonexistent_event() {
    let response = client()
        .get(format!("{}/2026/event/99999/abstract/", get_base_url()))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[test]
fn test_health_endpoint() {
    let response = client()
        .get(format!("{}/health", get_base_url()))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 204);
}

#[test]
fn test_404_for_nonexistent_route() {
    let response = client()
        .get(format!(
            "{}/this-route-should-not-exist-12345",
            get_base_url()
        ))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

fn assert_generic_timetable_content(body: &str) {
    // Should contain day headings (Saturday/Sunday for FOSDEM)
    assert!(body.contains("Saturday"));
    assert!(body.contains("Sunday"));
    // Should contain table structure
    assert!(body.contains("<table"));
    assert!(body.contains("Time"));
}

#[test]
fn test_timetable_2025_exists() {
    let response = exists_at_path("/2025/timetable/").expect("exists");
    let body = response.text().expect("Failed to read body");
    assert!(body.contains("2025"));
    assert_generic_timetable_content(&body);
}

#[test]
fn test_timetable_2026_exists() {
    let response = exists_at_path("/2026/timetable/").expect("exists");
    let body = response.text().expect("Failed to read body");
    assert!(body.contains("2026"));
    assert_generic_timetable_content(&body);
}

#[test]
fn test_blog_list_exists() {
    let response = exists_at_path("/blog/").expect("exists");
    let body = response.text().expect("Failed to read body");
    assert!(body.contains("Blog"));
    assert!(body.contains("Data Update"));
}

#[test]
fn test_blog_post_exists() {
    let response = exists_at_path("/blog/2026-02-02/").expect("exists");
    let body = response.text().expect("Failed to read body");
    assert!(body.contains("Data Update"));
    assert!(body.contains("Updated event data."));
}

#[test]
fn test_blog_post_404_for_nonexistent() {
    let response = client()
        .get(format!("{}/blog/1999-01-01/", get_base_url()))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}

#[test]
fn test_rss_feed_exists() {
    let response = exists_at_path("/rss.xml").expect("exists");
    let body = response.text().expect("Failed to read body");
    assert!(body.contains("<rss"));
    assert!(body.contains("FOSDEM 2026"));
    assert!(body.contains("Data Update"));
    assert!(body.contains("/blog/2026-02-02/"));
}

#[test]
fn test_bookmarks_uses_lazy_loading() {
    let response = exists_at_path("/bookmarks").expect("exists");
    let body = response.text().expect("Failed to read body");

    // Should contain abstract URLs for lazy loading
    assert!(body.contains(EVENT_ID_2025_ABSTRACT_PATH));
    assert!(body.contains(EVENT_ID_2026_ABSTRACT_PATH));

    // Should NOT contain the actual abstract content (it's lazy loaded)
    assert!(!body.contains(EVENT_ID_2025_CONTENT_SAMPLE));
    assert!(!body.contains(EVENT_ID_2026_CONTENT_SAMPLE));

    // Should contain skeleton placeholders
    assert!(body.contains("skeleton-lines"));
}

#[test]
fn test_compression_enabled_when_client_supports_gzip() {
    let response = client()
        .get(format!("{}/bookmarks", get_base_url()))
        .header("Accept-Encoding", "gzip")
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);
    let content_encoding = response.headers().get("content-encoding");
    assert!(
        content_encoding.is_some(),
        "Expected content-encoding header when client accepts gzip"
    );
    assert_eq!(
        content_encoding.unwrap().to_str().unwrap(),
        "gzip",
        "Expected gzip content-encoding"
    );
}

#[test]
fn test_no_compression_when_client_does_not_support_it() {
    let no_compression_client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .no_gzip()
        .build()
        .expect("Failed to create HTTP client");

    let response = no_compression_client
        .get(format!("{}/bookmarks", get_base_url()))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);
    let content_encoding = response.headers().get("content-encoding");
    assert!(
        content_encoding.is_none(),
        "Expected no content-encoding header when client doesn't accept compression"
    );
}

#[test]
fn test_next_redirects_to_current_year_timetable() {
    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("Failed to create HTTP client");

    let response = client
        .get(format!("{}/next/", get_base_url()))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 307); // Temporary Redirect
    let location = response
        .headers()
        .get("location")
        .expect("Missing Location header");
    assert_eq!(location, "/2026/timetable/");
}
