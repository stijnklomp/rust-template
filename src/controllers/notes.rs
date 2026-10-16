// example — remove when building your service
//!
//! Notes controllers. Input extraction lives here (`ValidatedQuery` /
//! `ValidatedJson`, see [`crate::validation`]); invalid input never reaches the
//! service and comes back as problem+json 400 (ADR-0005).

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::models::note::{CreateNoteBody, CreateNoteResponse, GetNotesQuery, NotesResponse};
use crate::problem::{ApiError, Problem};
use crate::server::AppState;
use crate::validation::{ValidatedJson, ValidatedQuery};

/// `GET /v1/notes`: one page of notes.
#[utoipa::path(
    get,
    path = "/v1/notes",
    tag = "notes",
    params(GetNotesQuery),
    responses(
        (status = 200, description = "A page of notes", body = NotesResponse),
        (status = 400, description = "Invalid query parameters", body = Problem),
        (status = 500, description = "Internal server error", body = Problem),
    )
)]
pub async fn get_notes(
    State(state): State<AppState>,
    ValidatedQuery(query): ValidatedQuery<GetNotesQuery>,
) -> Result<Json<NotesResponse>, ApiError> {
    let notes = state.notes()?.get_notes(&query).await?;

    Ok(Json(NotesResponse { notes }))
}

/// `POST /v1/notes`: create a note.
#[utoipa::path(
    post,
    path = "/v1/notes",
    tag = "notes",
    request_body = CreateNoteBody,
    responses(
        (status = 201, description = "Note created", body = CreateNoteResponse),
        (status = 400, description = "Invalid note", body = Problem),
        (status = 500, description = "Internal server error", body = Problem),
    )
)]
pub async fn create_note(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateNoteBody>,
) -> Result<(StatusCode, Json<CreateNoteResponse>), ApiError> {
    let note = state.notes()?.create_note(body).await?;

    Ok((
        StatusCode::CREATED,
        Json(CreateNoteResponse {
            message: "Note Created".to_owned(),
            note,
        }),
    ))
}
