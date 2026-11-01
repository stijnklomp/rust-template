//! Cluster end-to-end tests: this service plus every other deployed service.
//!
//! Deliberately empty in the template: nothing here can run until the service
//! is deployed next to its peers. Add tests once it is — the pattern is
//! "call service B, assert the accepted response, then poll the downstream
//! observable (API or database) until it appears, with a bounded deadline and
//! no fixed sleeps".
//!
//! Everything is `#[ignore]`d, so the default suite never tries to reach a
//! cluster:
//!
//! ```sh
//! E2E_BASE_URL=https://service-b.staging.example \
//!     cargo nextest run -E 'binary(integration)' --run-ignored all
//! ```

#[tokio::test]
#[ignore = "requires a staging cluster; set E2E_BASE_URL"]
async fn user_created_in_service_b_appears_in_the_database() {
    let base = std::env::var("E2E_BASE_URL")
        .expect("E2E_BASE_URL must point at the staging cluster's service B");

    // `reqwest` uses `rustls-no-provider`; install one before the client.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{base}/v1/users"))
        .json(&serde_json::json!({ "email": "e2e@example.com" }))
        .send()
        .await
        .expect("service B must be reachable");

    assert_eq!(
        response.status(),
        reqwest::StatusCode::ACCEPTED,
        "service B must accept the user-creation request"
    );

    // Replace this with the real downstream assertion: poll service A (or its
    // database) until the user shows up, bounded by a deadline.
    let _ = response;
}
