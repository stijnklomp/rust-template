// example — remove when building your service
//
// Feature tests for the notes demo slice: the endpoints through the real
// router, driven in-process by axum-test, with a hand-written fake repository
// behind the service (the house idiom — no database).

use std::sync::Arc;

use axum::http::{StatusCode, header};
use axum_test::TestServer;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use rust_template::models::note::{CreateNoteBody, Note};
use rust_template::problem::PROBLEM_JSON_CONTENT_TYPE;
use rust_template::repositories::notes::{NotesRepository, RepositoryFuture};
use rust_template::server::{AppState, build_router};
use rust_template::services::notes::NotesService;

/// Returns the configured notes verbatim for every page (page bounds are not
/// what these tests exercise; the live SQL is verified against Postgres in
/// the acceptance suite).
struct FakeNotesRepository {
    notes: Vec<Note>,
}

impl NotesRepository for FakeNotesRepository {
    fn find_page(&self, _page: u32, _per_page: u32) -> RepositoryFuture<'_, Vec<Note>> {
        let notes = self.notes.clone();
        Box::pin(async move { Ok(notes) })
    }

    fn create(&self, body: CreateNoteBody) -> RepositoryFuture<'_, Note> {
        Box::pin(async move {
            let mut note = test_note(1);
            note.owner = body.owner;
            note.note = body.note;
            Ok(note)
        })
    }
}

fn test_note(id: i32) -> Note {
    let created_at = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
        NaiveTime::from_hms_micro_opt(12, 0, 0, 123_456).unwrap(),
    );
    Note {
        id,
        owner: "ada".to_owned(),
        note: "port the slice".to_owned(),
        created_at,
        updated_at: created_at,
    }
}

fn app_with_notes(notes: Vec<Note>) -> TestServer {
    let state = AppState {
        notes: Some(Arc::new(NotesService::new(Arc::new(FakeNotesRepository {
            notes,
        })))),
        ..AppState::default()
    };
    TestServer::new(build_router(state))
}

fn content_type(response: &axum_test::TestResponse) -> Option<&str> {
    response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
}

#[tokio::test]
async fn get_notes_returns_the_page_with_camel_case_timestamps() {
    let server = app_with_notes(vec![test_note(1), test_note(2)]);

    let response = server.get("/v1/notes?page=1&perPage=10").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    let body = response.json::<serde_json::Value>();
    let notes = body["notes"].as_array().expect("notes must be an array");
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0]["id"], 1);
    assert_eq!(notes[0]["owner"], "ada");
    assert_eq!(notes[0]["note"], "port the slice");
    assert_eq!(notes[0]["createdAt"], "2026-10-16T12:00:00.123456");
    assert_eq!(notes[0]["updatedAt"], "2026-10-16T12:00:00.123456");
    assert!(
        notes[0].get("created_at").is_none(),
        "JSON must be camelCase: {body}"
    );
}

#[tokio::test]
async fn get_notes_rejects_page_below_one_with_problem_json_and_errors() {
    let server = app_with_notes(vec![]);

    let response = server.get("/v1/notes?page=0&perPage=10").await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(content_type(&response), Some(PROBLEM_JSON_CONTENT_TYPE));
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["title"], "Bad Request");
    assert_eq!(body["status"], 400);
    let errors = body["errors"].as_array().expect("errors extension");
    assert_eq!(errors[0]["pointer"], "/page");
    assert!(errors[0]["detail"].as_str().is_some_and(|d| !d.is_empty()));
}

#[tokio::test]
async fn get_notes_rejects_missing_query_parameters() {
    let server = app_with_notes(vec![]);

    let response = server.get("/v1/notes").await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(content_type(&response), Some(PROBLEM_JSON_CONTENT_TYPE));
    let body = response.json::<serde_json::Value>();
    let errors = body["errors"].as_array().expect("errors extension");
    assert!(!errors.is_empty());
}

#[tokio::test]
async fn get_notes_reports_service_unavailable_without_a_repository() {
    let server = TestServer::new(build_router(AppState::default()));

    let response = server.get("/v1/notes?page=1&perPage=10").await;

    assert_eq!(response.status_code(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(content_type(&response), Some(PROBLEM_JSON_CONTENT_TYPE));
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["title"], "Service Unavailable");
}

#[tokio::test]
async fn create_note_returns_201_with_the_created_note() {
    let server = app_with_notes(vec![]);

    let response = server
        .post("/v1/notes")
        .json(&serde_json::json!({
            "note": "write the slice",
            "owner": "grace",
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::CREATED);
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["message"], "Note Created");
    assert_eq!(body["note"]["owner"], "grace");
    assert_eq!(body["note"]["note"], "write the slice");
    assert_eq!(body["note"]["createdAt"], "2026-10-16T12:00:00.123456");
}

#[tokio::test]
async fn create_note_rejects_an_overlong_note_with_problem_json_and_errors() {
    let server = app_with_notes(vec![]);

    let response = server
        .post("/v1/notes")
        .json(&serde_json::json!({
            "note": "x".repeat(301),
            "owner": "ada",
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(content_type(&response), Some(PROBLEM_JSON_CONTENT_TYPE));
    let body = response.json::<serde_json::Value>();
    assert_eq!(body["title"], "Bad Request");
    let errors = body["errors"].as_array().expect("errors extension");
    assert_eq!(errors[0]["pointer"], "/note");
}

#[tokio::test]
async fn create_note_rejects_unknown_fields() {
    // Deliberate divergence from Fastify (which silently strips extra
    // properties): request DTOs are strict (`deny_unknown_fields`).
    let server = app_with_notes(vec![]);

    let response = server
        .post("/v1/notes")
        .json(&serde_json::json!({
            "note": "fine",
            "owner": "ada",
            "admin": true,
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(content_type(&response), Some(PROBLEM_JSON_CONTENT_TYPE));
    let body = response.json::<serde_json::Value>();
    let errors = body["errors"].as_array().expect("errors extension");
    assert!(!errors.is_empty());
}

#[tokio::test]
async fn create_note_rejects_malformed_json() {
    let server = app_with_notes(vec![]);

    let response = server
        .post("/v1/notes")
        // `text()` sets `text/plain`; the JSON content type goes on after.
        .text("{")
        .content_type("application/json")
        .await;

    assert_eq!(response.status_code(), StatusCode::BAD_REQUEST);
    assert_eq!(content_type(&response), Some(PROBLEM_JSON_CONTENT_TYPE));
    let body = response.json::<serde_json::Value>();
    let errors = body["errors"].as_array().expect("errors extension");
    assert!(!errors.is_empty());
    assert!(body["detail"].as_str().is_some());
}
