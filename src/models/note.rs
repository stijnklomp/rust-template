// example — remove when building your service
//!
//! DTOs for the notes demo slice. One struct per DTO, carrying `serde`
//! (de/Serialization), `utoipa::ToSchema` (documentation) and `garde::Validate`
//! (runtime validation via the extractors in [`crate::validation`]); the
//! `#[garde]`/`#[schema]` duplication is the accepted cost of ADR-0004.
//!
//! Divergence from the Fastify template, kept deliberately: timestamps
//! serialize as `chrono` `NaiveDateTime` (e.g. `2026-10-16T12:00:00.123456`,
//! no timezone suffix) instead of Prisma's `toISOString()` `...Z` form.

use chrono::NaiveDateTime;
use garde::Validate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::{IntoParams, ToSchema};

/// A note (the `note` table row; Fastify's `note` Prisma model).
///
/// Field names are snake_case (the query macros map columns to fields by
/// name); the SQL aliases the camelCase columns (`"createdAt" AS created_at`).
#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    /// Autoincrement id.
    pub id: i32,
    /// Owner of the note.
    pub owner: String,
    /// Note text.
    pub note: String,
    /// Creation timestamp.
    pub created_at: NaiveDateTime,
    /// Last-update timestamp.
    pub updated_at: NaiveDateTime,
}

/// `GET /v1/notes` query parameters.
///
/// TypeBox parity: `page` minimum 1, `perPage` maximum 100, both required.
#[derive(Debug, Clone, Deserialize, Validate, IntoParams, ToSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[into_params(parameter_in = Query)]
pub struct GetNotesQuery {
    /// Page number, starting at 1.
    #[garde(range(min = 1))]
    #[param(minimum = 1)]
    pub page: u32,
    /// Notes per page, at most 100 (0 returns an empty page, like Fastify).
    #[garde(range(max = 100))]
    #[param(minimum = 0, maximum = 100)]
    pub per_page: u32,
}

/// `POST /v1/notes` request body.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Validate, ToSchema)]
#[serde(deny_unknown_fields)]
pub struct CreateNoteBody {
    /// Note text; JSON Schema `maxLength` counts code points (AJV parity), so
    /// the rule uses garde's `chars` mode — `simple` would count bytes.
    #[garde(length(chars, max = 300))]
    #[schema(max_length = 300)]
    pub note: String,
    /// Owner of the note.
    #[garde(length(chars, max = 100))]
    #[schema(max_length = 100)]
    pub owner: String,
}

/// `GET /v1/notes` response body.
#[derive(Debug, Serialize, ToSchema)]
pub struct NotesResponse {
    /// The requested page of notes.
    pub notes: Vec<Note>,
}

/// `POST /v1/notes` response body.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreateNoteResponse {
    /// Human-readable creation message (Fastify parity).
    pub message: String,
    /// The stored note.
    pub note: Note,
}
