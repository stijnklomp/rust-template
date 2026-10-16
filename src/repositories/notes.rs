// example — remove when building your service
//!
//! Notes repository. [`NotesRepository`] is the seam services depend on;
//! tests swap in hand-written fakes (the house idiom — no module mocking in
//! Rust), which is also the seam later tickets build on.

use std::future::Future;
use std::pin::Pin;

use sqlx::PgPool;
use thiserror::Error;

use crate::models::note::{CreateNoteBody, Note};

/// Boxed future returned by repository methods (mirrors `ReadinessFuture`).
pub type RepositoryFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, RepositoryError>> + Send + 'a>>;

/// Storage failure; the service layer maps it to a problem+json 500.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// The database refused or failed the operation.
    #[error("database operation failed")]
    Database(#[from] sqlx::Error),
}

/// What the notes service needs from storage.
pub trait NotesRepository: Send + Sync + 'static {
    /// One page of notes.
    fn find_page(&self, page: u32, per_page: u32) -> RepositoryFuture<'_, Vec<Note>>;

    /// Inserts a note and returns the stored row.
    fn create(&self, body: CreateNoteBody) -> RepositoryFuture<'_, Note>;
}

/// Postgres-backed notes repository (sqlx, compile-time checked against the
/// committed `.sqlx` cache).
#[derive(Debug, Clone)]
pub struct PgNotesRepository {
    pool: PgPool,
}

impl PgNotesRepository {
    /// Repository over the shared pool.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl NotesRepository for PgNotesRepository {
    fn find_page(&self, page: u32, per_page: u32) -> RepositoryFuture<'_, Vec<Note>> {
        Box::pin(async move {
            // Deliberate fix of the Fastify template's off-by-pagination bug
            // (parity index §5): its repository used `skip: page - 1` (one row
            // per page step) instead of `(page - 1) * perPage`, so page 2 with
            // perPage 10 started at row 1. `ORDER BY id` makes paging stable
            // (Prisma's `findMany` had no defined order).
            let offset = i64::from(page.saturating_sub(1)) * i64::from(per_page);
            let limit = i64::from(per_page);

            let notes = sqlx::query_as!(
                Note,
                r#"SELECT id, owner, note,
                          "createdAt" AS created_at, "updatedAt" AS updated_at
                   FROM note
                   ORDER BY id
                   LIMIT $1 OFFSET $2"#,
                limit,
                offset
            )
            .fetch_all(&self.pool)
            .await?;

            Ok(notes)
        })
    }

    fn create(&self, body: CreateNoteBody) -> RepositoryFuture<'_, Note> {
        Box::pin(async move {
            let note = sqlx::query_as!(
                Note,
                r#"INSERT INTO note (owner, note, "createdAt", "updatedAt")
                   VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
                   RETURNING id, owner, note,
                             "createdAt" AS created_at, "updatedAt" AS updated_at"#,
                body.owner,
                body.note
            )
            .fetch_one(&self.pool)
            .await?;

            Ok(note)
        })
    }
}
