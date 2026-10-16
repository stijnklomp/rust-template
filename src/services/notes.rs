// example — remove when building your service
//!
//! Notes service: the pass-through layer between controllers and the notes
//! repository (Fastify parity: `services/notes.ts` just calls the repository).
//! Storage errors become problem+json 500s; the source is logged, never
//! exposed (ADR-0005).

use std::sync::Arc;

use crate::models::note::{CreateNoteBody, GetNotesQuery, Note};
use crate::problem::ApiError;
use crate::repositories::notes::{NotesRepository, RepositoryError};

/// Notes operations over the [`NotesRepository`] seam.
#[derive(Clone)]
pub struct NotesService {
    repository: Arc<dyn NotesRepository>,
}

impl NotesService {
    /// Service over any notes repository (Postgres in `main`, fakes in tests).
    pub fn new(repository: Arc<dyn NotesRepository>) -> Self {
        Self { repository }
    }

    /// One page of notes.
    pub async fn get_notes(&self, query: &GetNotesQuery) -> Result<Vec<Note>, ApiError> {
        self.repository
            .find_page(query.page, query.per_page)
            .await
            .map_err(internal_error)
    }

    /// Creates a note and returns the stored row.
    pub async fn create_note(&self, body: CreateNoteBody) -> Result<Note, ApiError> {
        self.repository.create(body).await.map_err(internal_error)
    }
}

/// Storage failures are 500s, Fastify parity.
fn internal_error(error: RepositoryError) -> ApiError {
    ApiError::Internal {
        source: Box::new(error),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use axum::http::StatusCode;
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    use super::*;
    use crate::repositories::notes::RepositoryFuture;

    #[derive(Default)]
    struct FakeNotesRepository {
        /// Every `(page, per_page)` pair the service passed through.
        pages: Mutex<Vec<(u32, u32)>>,
        /// Every body the service passed through.
        created: Mutex<Vec<CreateNoteBody>>,
        /// When set, every call fails with this error.
        fail: bool,
    }

    impl FakeNotesRepository {
        fn failing() -> Self {
            Self {
                fail: true,
                ..Self::default()
            }
        }
    }

    impl NotesRepository for FakeNotesRepository {
        fn find_page(&self, page: u32, per_page: u32) -> RepositoryFuture<'_, Vec<Note>> {
            self.pages.lock().unwrap().push((page, per_page));
            let fail = self.fail;
            Box::pin(async move {
                if fail {
                    return Err(RepositoryError::Database(sqlx::Error::RowNotFound));
                }
                Ok(vec![test_note()])
            })
        }

        fn create(&self, body: CreateNoteBody) -> RepositoryFuture<'_, Note> {
            self.created.lock().unwrap().push(body.clone());
            let fail = self.fail;
            Box::pin(async move {
                if fail {
                    return Err(RepositoryError::Database(sqlx::Error::RowNotFound));
                }
                let mut note = test_note();
                note.owner = body.owner;
                note.note = body.note;
                Ok(note)
            })
        }
    }

    fn test_note() -> Note {
        let at = NaiveDateTime::new(
            NaiveDate::from_ymd_opt(2026, 10, 16).unwrap(),
            NaiveTime::from_hms_micro_opt(12, 0, 0, 123_456).unwrap(),
        );
        Note {
            id: 7,
            owner: "ada".to_owned(),
            note: "port the slice".to_owned(),
            created_at: at,
            updated_at: at,
        }
    }

    fn service(repository: &Arc<FakeNotesRepository>) -> NotesService {
        NotesService::new(repository.clone())
    }

    #[tokio::test]
    async fn get_notes_forwards_page_and_per_page_and_returns_the_rows() {
        let repository = Arc::new(FakeNotesRepository::default());

        let notes = service(&repository)
            .get_notes(&GetNotesQuery {
                page: 3,
                per_page: 25,
            })
            .await
            .unwrap();

        assert_eq!(*repository.pages.lock().unwrap(), vec![(3, 25)]);
        assert_eq!(notes, vec![test_note()]);
    }

    #[tokio::test]
    async fn create_note_forwards_the_body_and_returns_the_stored_row() {
        let repository = Arc::new(FakeNotesRepository::default());

        let note = service(&repository)
            .create_note(CreateNoteBody {
                note: "written first".to_owned(),
                owner: "grace".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(
            *repository.created.lock().unwrap(),
            vec![CreateNoteBody {
                note: "written first".to_owned(),
                owner: "grace".to_owned(),
            }]
        );
        assert_eq!(note.owner, "grace");
        assert_eq!(note.note, "written first");
    }

    #[tokio::test]
    async fn repository_failures_become_internal_errors() {
        let repository = Arc::new(FakeNotesRepository::failing());

        let error = service(&repository)
            .get_notes(&GetNotesQuery {
                page: 1,
                per_page: 10,
            })
            .await
            .unwrap_err();
        assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let error = service(&repository)
            .create_note(CreateNoteBody {
                note: "x".to_owned(),
                owner: "ada".to_owned(),
            })
            .await
            .unwrap_err();
        assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
