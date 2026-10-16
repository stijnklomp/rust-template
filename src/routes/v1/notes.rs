// example — remove when building your service
//!
//! Notes routes: registers the notes controllers with axum and feeds their
//! `#[utoipa::path]` documentation into the generated OpenAPI document.

use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

// utoipa generates one `__path_<handler>` item per annotated handler; the
// `routes!` macro resolves them by name, so both must be in scope here.
use crate::controllers::notes::{__path_create_note, __path_get_notes, create_note, get_notes};
use crate::server::AppState;

/// `GET` and `POST` on `/v1/notes`.
pub fn routes() -> OpenApiRouter<AppState> {
    OpenApiRouter::new().routes(routes!(get_notes, create_note))
}
