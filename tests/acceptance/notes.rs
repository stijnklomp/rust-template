// example — remove when building your service
//
// The notes demo slice against real Postgres: create through the API, read
// the note back through the API, and rejection behavior — through the real
// router over [`PgNotesRepository`], exactly as `main` wires it.

use std::sync::Arc;

use axum::http::{StatusCode, header};
use axum_test::TestServer;
use serde_json::json;

use rust_template::database::Database;
use rust_template::problem::PROBLEM_JSON_CONTENT_TYPE;
use rust_template::repositories::notes::PgNotesRepository;
use rust_template::server::{AppState, build_router};
use rust_template::services::notes::NotesService;

use crate::migrated_database;

/// The app as `main` assembles it: database handle, readiness check and the
/// Postgres-backed notes service.
fn app(database: &Database) -> TestServer {
    let mut state = AppState {
        database: Some(database.clone()),
        ..AppState::default()
    };
    state.readiness.register(database.clone());
    // example — remove when building your service (notes demo slice)
    state.notes = Some(Arc::new(NotesService::new(Arc::new(
        PgNotesRepository::new(database.pool().clone()),
    ))));
    TestServer::new(build_router(state))
}

/// Finds `id` by paging through `GET /v1/notes` (the database is shared, so
/// previously stored notes may fill earlier pages).
async fn find_note(server: &TestServer, id: i64) -> serde_json::Value {
    for page in 1..=50 {
        let response = server
            .get(&format!("/v1/notes?page={page}&perPage=100"))
            .await;
        assert_eq!(response.status_code(), StatusCode::OK, "GET page {page}");
        let body = response.json::<serde_json::Value>();
        let notes = body["notes"].as_array().expect("notes must be an array");
        if notes.is_empty() {
            break;
        }
        if let Some(note) = notes.iter().find(|note| note["id"].as_i64() == Some(id)) {
            return note.clone();
        }
    }
    panic!("note {id} was not visible through GET /v1/notes within 50 pages");
}

#[tokio::test]
async fn notes_flow_round_trips_through_real_postgres() {
    let server = app(&migrated_database().await);
    let label = crate::unique_label("notes");

    let created = server
        .post("/v1/notes")
        .json(&json!({
            "note": format!("acceptance note {label}"),
            "owner": format!("acceptance {label}"),
        }))
        .await;
    assert_eq!(created.status_code(), StatusCode::CREATED);
    let created = created.json::<serde_json::Value>();
    assert_eq!(created["message"], "Note Created");
    let note = &created["note"];
    assert_eq!(note["owner"], format!("acceptance {label}"));
    assert_eq!(note["note"], format!("acceptance note {label}"));
    assert!(
        note["createdAt"].as_str().is_some_and(|at| !at.is_empty()),
        "createdAt must be a timestamp string: {created}"
    );
    assert_eq!(
        note["createdAt"], note["updatedAt"],
        "a fresh note has identical timestamps: {created}"
    );
    assert!(
        note.get("created_at").is_none(),
        "JSON must be camelCase: {created}"
    );

    // Read the same row back through the list endpoint.
    let id = note["id"].as_i64().expect("id");
    let fetched = find_note(&server, id).await;
    assert_eq!(fetched["owner"], note["owner"]);
    assert_eq!(fetched["note"], note["note"]);
    assert_eq!(fetched["createdAt"], note["createdAt"]);
}

#[tokio::test]
async fn notes_validation_error_is_problem_json_through_the_real_router() {
    let server = app(&migrated_database().await);

    let response = server
        .post("/v1/notes")
        .json(&json!({
            "note": "x".repeat(301),
            "owner": "acceptance",
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(
        response
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some(PROBLEM_JSON_CONTENT_TYPE)
    );
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["title"], "Bad Request");
    assert_eq!(body["status"], 400);
    let errors = body["errors"].as_array().expect("errors extension");
    assert_eq!(errors[0]["pointer"], "/note");
}
