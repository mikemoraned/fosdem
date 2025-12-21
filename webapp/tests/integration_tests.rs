use reqwest::blocking::{Client, Response};
use reqwest::Result;
use std::env;

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
    let response = client().get(format!("{}{}", get_base_url(), path)).send()?;

    assert_eq!(response.status(), 200);

    Ok(response)
}

#[test]
fn test_homepage_exists() {
    exists_at_path("/").expect("exists");
}

#[test]
fn test_homepage_contains_expected_content() {
    let response = exists_at_path("/").expect("exists");

    let body = response.text().expect("Failed to read body");
    assert!(body.contains("<!DOCTYPE html>"));
    assert!(body.contains("<title>FOSDEM 2025</title>"));
    assert!(body.contains("<a href=\"https://fosdem.org/2025/\">FOSDEM 2025</a>"));
    assert!(body.contains(
        "All content such as talks and biographies is the sole responsibility of the speaker."
    ));
}

fn assert_2025_content(response: Response) {
    let body = response.text().expect("Failed to read body");
    assert!(body.contains("Using composefs and fs-verity"));
}

#[test]
fn test_2025_content_in_backwards_compatible_place() {
    let response = exists_at_path("/event/5191/").expect("exists");
    assert_2025_content(response);
}

#[test]
fn test_2025_content_in_canonical_place() {
    let response = exists_at_path("/2025/event/5191/").expect("exists");
    assert_2025_content(response);
}

fn assert_2026_content(response: Response) {
    let body = response.text().expect("Failed to read body");
    assert!(body.contains("Open source represents 70% to 90% of modern software codebases"));
}

#[test]
fn test_2026_content_in_canonical_place() {
    let response = exists_at_path("/2026/event/7910/").expect("exists");
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
