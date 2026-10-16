// example — remove when building your service
//!
//! The `v1` route surface (demo slice: notes).

pub mod notes;

use utoipa_axum::router::OpenApiRouter;

use crate::server::AppState;

/// Routes under `/v1` (each handler's `#[utoipa::path]` carries the absolute path).
pub fn routes() -> OpenApiRouter<AppState> {
    notes::routes()
}
