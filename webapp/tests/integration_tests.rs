use reqwest::blocking::Client;
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

#[test]
fn test_homepage_exists() {
    let response = client()
        .get(format!("{}/", get_base_url()))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 200);
}

#[test]
fn test_homepage_contains_expected_content() {
    let response = client()
        .get(format!("{}/", get_base_url()))
        .send()
        .expect("Failed to send request");

    let body = response.text().expect("Failed to read body");
    assert!(body.contains("<!DOCTYPE html>"));
    assert!(body.contains("<title>FOSDEM 2025</title>"));
    assert!(body.contains("<a href=\"https://fosdem.org/2025/\">FOSDEM 2025</a>"));
    assert!(body.contains("All content such as talks and biographies is the sole responsibility of the speaker."));
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
        .get(format!("{}/this-route-should-not-exist-12345", get_base_url()))
        .send()
        .expect("Failed to send request");

    assert_eq!(response.status(), 404);
}
