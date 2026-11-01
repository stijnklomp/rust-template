//! The acceptance driver: the only place the spec suite touches the service's
//! transport.
//!
//! Steps go through [`SpecWorld`](crate::world::SpecWorld), so the same
//! `.feature` files run in two modes (ADR-0011):
//!
//! - **in-process** (default): the axum router over `axum-test`, against the
//!   real Postgres/Valkey from the compose stack — fast and concurrent;
//! - **black-box** (`SPEC_TARGET` set): `reqwest` against a deployed service —
//!   serial, for the per-push pipeline gate.
//!
//! Nothing outside this file, `world.rs` and `runner.rs` may import
//! `rust_template::`.

use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};

use axum::http::header;
use axum_test::TestServer;
use serde_json::Value;

/// Base URL for black-box runs; unset selects the in-process driver.
pub const TARGET_ENV: &str = "SPEC_TARGET";

/// The shared in-process app, installed once by `runner::main`.
static IN_PROCESS: OnceLock<Arc<TestServer>> = OnceLock::new();

/// Installs the in-process app. Called once, before the suite starts.
pub fn install_in_process(server: TestServer) {
    if IN_PROCESS.set(Arc::new(server)).is_err() {
        panic!("the in-process app is installed once");
    }
}

/// One request the spec asks for.
#[derive(Debug, Clone)]
pub struct SpecRequest {
    /// HTTP method (`GET` or `POST`).
    pub method: &'static str,
    /// Path, including the query string.
    pub path: String,
    /// JSON body, when the method takes one.
    pub body: Option<Value>,
}

impl SpecRequest {
    /// A `GET` request.
    pub fn get(path: impl Into<String>) -> Self {
        Self {
            method: "GET",
            path: path.into(),
            body: None,
        }
    }

    /// A `POST` request with a JSON body.
    pub fn post(path: impl Into<String>, body: Value) -> Self {
        Self {
            method: "POST",
            path: path.into(),
            body: Some(body),
        }
    }
}

/// The normalized response both drivers produce.
#[derive(Debug, Clone)]
pub struct SpecResponse {
    /// HTTP status code.
    pub status: u16,
    /// `Content-Type` header, empty when absent.
    pub content_type: String,
    /// Parsed JSON body; `Null` for empty or non-JSON bodies.
    pub body: Value,
    /// Raw body, for failure messages.
    pub text: String,
}

/// Transport selected by `SPEC_TARGET`.
pub enum Api {
    /// Direct in-process router (fast, concurrent).
    InProcess(Arc<TestServer>),
    /// HTTP against a deployed service (serial).
    Http {
        /// Reused HTTP client.
        client: reqwest::Client,
        /// Service root URL, without a trailing slash.
        base_url: String,
    },
}

impl std::fmt::Debug for Api {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InProcess(_) => formatter.write_str("Api::InProcess"),
            Self::Http { base_url, .. } => formatter
                .debug_struct("Api::Http")
                .field("base_url", base_url)
                .finish(),
        }
    }
}

impl Api {
    /// The driver for this run.
    pub fn from_env() -> Self {
        match std::env::var(TARGET_ENV) {
            Ok(base_url) => Self::Http {
                client: reqwest::Client::builder()
                    .timeout(Duration::from_secs(30))
                    .build()
                    .expect("the HTTP client must build"),
                base_url: base_url.trim_end_matches('/').to_owned(),
            },
            Err(_) => Self::InProcess(
                IN_PROCESS
                    .get()
                    .expect("runner::main installs the in-process app before the suite starts")
                    .clone(),
            ),
        }
    }

    /// Sends one request and normalizes the response.
    pub async fn send(&self, request: SpecRequest) -> SpecResponse {
        match self {
            Self::InProcess(server) => {
                let mut pending = match request.method {
                    "GET" => server.get(&request.path),
                    "POST" => server.post(&request.path),
                    other => panic!("the spec driver does not support {other}"),
                };
                if let Some(body) = request.body {
                    pending = pending.json(&body);
                }

                let response = pending.await;
                let status = response.status_code().as_u16();
                let content_type = response
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_owned();
                normalize(status, content_type, response.text())
            }
            Self::Http { client, base_url } => {
                let method = reqwest::Method::from_bytes(request.method.as_bytes())
                    .expect("the spec driver only sends valid methods");
                let mut pending = client.request(method, format!("{base_url}{}", request.path));
                if let Some(body) = request.body {
                    pending = pending.json(&body);
                }

                let response = pending
                    .send()
                    .await
                    .expect("the deployed service must be reachable");
                let status = response.status().as_u16();
                let content_type = response
                    .headers()
                    .get(header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .unwrap_or_default()
                    .to_owned();
                let text = response
                    .text()
                    .await
                    .expect("the response body must be readable");
                normalize(status, content_type, text)
            }
        }
    }
}

fn normalize(status: u16, content_type: String, text: String) -> SpecResponse {
    let body = serde_json::from_str(&text).unwrap_or(Value::Null);
    SpecResponse {
        status,
        content_type,
        body,
        text,
    }
}

/// Blocks a black-box run until `/readyz` answers 204, bounded by a deadline.
///
/// The deadline is generous because the compose black-box profile builds the
/// app from source on a cold cache; a deployed service answers immediately.
pub async fn wait_until_ready(base_url: &str) {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("the readiness client must build");
    let url = format!("{}/readyz", base_url.trim_end_matches('/'));
    let deadline = Instant::now() + Duration::from_secs(180);

    loop {
        match client.get(&url).send().await {
            Ok(response) if response.status().as_u16() == 204 => return,
            other => {
                assert!(
                    Instant::now() < deadline,
                    "the deployed service at {base_url} never became ready: {other:?}"
                );
            }
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}
