// example — remove when building your service
//
// Removing the demo slice also deletes the checked-in
// `pacts/notes-client-rust-template.json` this test writes, the notes
// provider states in `provider.rs`, and the `notes` tag in `src/openapi.rs`.
//
// Consumer side of the Pact example (ADR-0008).
//
// This file is the documentation for the consumer DSL: it states what a
// "notes client" expects from the service, drives the `pact_consumer` mock
// server with real requests, and — when the mock server shuts down — writes
// the contract to [`pact_file`], which is checked in. A mismatch between the
// requests the client makes and the contract fails this test; a mismatch
// between the contract and the service fails `tests/pact/provider.rs`.
//
// Run both sides with `just test-pact`.

#[path = "common.rs"]
mod common;

use std::path::{Path, PathBuf};

use pact_consumer::mock_server::StartMockServerAsync;
use pact_consumer::prelude::*;
use serde_json::json;

use common::PACT_OWNER;

/// Consumer name: the first half of the pact file name.
const CONSUMER: &str = "notes-client";
/// Note text the provider state has to create before the read interaction.
const EXISTING_NOTE: &str = "written by the pact provider state";
/// Note text the create interaction sends.
const CREATED_NOTE: &str = "created by the notes-client example";

/// The checked-in contract: `pacts/<consumer>-<provider>.json`.
fn pact_file() -> PathBuf {
    Path::new("pacts").join(format!("{CONSUMER}-{}.json", rust_template::APP_NAME))
}

#[tokio::test]
async fn notes_client_contract() {
    // The mock server merges into an existing pact file; remove it first so
    // interactions deleted here cannot survive in the checked-in contract.
    let _ = std::fs::remove_file(pact_file());

    let mock = PactBuilderAsync::new_v4(CONSUMER, rust_template::APP_NAME)
        .interaction("a request for a page of notes", "", |mut i| async move {
            // Provider states say what must be true before the request runs;
            // the params carry the fixture data the provider side has to set up.
            i.given_with_params(
                "a note exists",
                &json!({ "owner": PACT_OWNER, "note": EXISTING_NOTE }),
            );
            i.request
                .get()
                .path("/v1/notes")
                .query_param("page", "1")
                .query_param("perPage", "10");
            i.response.ok().json_body(json_pattern!({
                "notes": each_like!({
                    "id": like!(0),
                    "owner": like!(PACT_OWNER),
                    "note": like!(EXISTING_NOTE),
                    "createdAt": like!("2026-10-16T12:00:00.123456"),
                    "updatedAt": like!("2026-10-16T12:00:00.123456"),
                }),
            }));
            i
        })
        .await
        .interaction("a request to create a note", "", |mut i| async move {
            i.request
                .post()
                .path("/v1/notes")
                // Explicit header (the app's JSON extractor requires it; the
                // verifier sends pact headers verbatim).
                .content_type("application/json")
                .json_body(json!({ "note": CREATED_NOTE, "owner": PACT_OWNER }));
            i.response.created().json_body(json_pattern!({
                "message": "Note Created",
                "note": {
                    "id": like!(0),
                    "owner": PACT_OWNER,
                    "note": CREATED_NOTE,
                    "createdAt": like!("2026-10-16T12:00:00.123456"),
                    "updatedAt": like!("2026-10-16T12:00:00.123456"),
                },
            }));
            i
        })
        .await
        .output_dir("pacts")
        .start_mock_server_async(None, None)
        .await;

    // A stand-in for the real client: exercising the mock server is what
    // proves these are the requests the contract describes. `reqwest` is
    // built with `rustls-no-provider`, so install the ring provider first
    // (`PactBuilderAsync::new_v4` does not).
    let _ = rustls::crypto::ring::default_provider().install_default();
    let client = reqwest::Client::new();

    let response = client
        .get(mock.path("/v1/notes?page=1&perPage=10"))
        .send()
        .await
        .expect("GET the mocked notes page");
    assert_eq!(response.status(), 200);
    let page: serde_json::Value = response.json().await.expect("notes page JSON");
    assert_eq!(page["notes"][0]["owner"], PACT_OWNER);

    let response = client
        .post(mock.path("/v1/notes"))
        .json(&json!({ "note": CREATED_NOTE, "owner": PACT_OWNER }))
        .send()
        .await
        .expect("POST the mocked note");
    assert_eq!(response.status(), 201);
    let created: serde_json::Value = response.json().await.expect("created note JSON");
    assert_eq!(created["note"]["note"], CREATED_NOTE);

    // Dropping the mock server verifies both interactions and writes
    // `pact_file()`, so the contract is regenerated on every run. Drop it
    // explicitly to stamp the demo-slice removal marker below: the marker has
    // to live in `metadata` and be written here, or the next regeneration
    // would silently drop it.
    drop(mock);

    let contents = std::fs::read_to_string(pact_file()).expect("the mock server wrote the pact");
    let mut pact: serde_json::Value = serde_json::from_str(&contents).expect("the pact is JSON");
    pact["metadata"]["removeWhen"] =
        serde_json::json!("example — remove when building your service");
    std::fs::write(
        pact_file(),
        serde_json::to_string_pretty(&pact).expect("the pact serializes"),
    )
    .expect("write the marked pact");
}
