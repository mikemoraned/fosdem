use reqwest::blocking::{Client, Response};
use reqwest::Result;
use shared::model::EventId;
use std::env;
use test_shared::{
    EVENT_ID_2025, EVENT_ID_2025_BACKWARDS_COMPATIBLE_PATH, EVENT_ID_2025_CANONICAL_PATH,
    EVENT_ID_2025_CONTENT_SAMPLE, EVENT_ID_2026, EVENT_ID_2026_CANONICAL_PATH,
    EVENT_ID_2026_CONTENT_SAMPLE, SEARCH_TERM,
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
    assert!(body.contains("<!DOCTYPE html>"));
    assert!(body.contains("<title>FOSDEM 2026</title>"));
    assert!(body.contains("<a href=\"https://fosdem.org/2026/\">FOSDEM 2026</a>"));
    assert!(body.contains(
        "All content such as talks and biographies is the sole responsibility of the speaker."
    ));
}

fn event_id_as_anchor_text(event_id: EventId) -> String {
    return format!(
        "<a name=\"{}-{}\"></a>",
        event_id.year(),
        event_id.event_in_year()
    );
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
    assert!(body.contains(EVENT_ID_2025_CONTENT_SAMPLE));
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
    assert!(body.contains(EVENT_ID_2026_CONTENT_SAMPLE));
}

#[test]
fn test_2026_content_in_canonical_place() {
    let response = exists_at_path(EVENT_ID_2026_CANONICAL_PATH).expect("exists");
    assert_2026_content(response);
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
