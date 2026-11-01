//! Application composition: the infrastructure clients and the state `main`
//! hands to the router.
//!
//! Extracted from `main` so the acceptance suite boots the *same* composition
//! the service runs (ADR-0011): the in-process spec driver calls
//! [`Infrastructure::build`] instead of re-assembling a subset of `AppState`.

use std::sync::Arc;

use thiserror::Error;

use crate::config::AppConfig;
use crate::database::Database;
use crate::infrastructure::cache::{CacheClient, CacheError};
use crate::repositories::notes::PgNotesRepository;
use crate::server::AppState;
use crate::services::notes::NotesService;

/// Infrastructure construction failure; `main` maps it to a startup error.
#[derive(Debug, Error)]
pub enum BuildError {
    /// The cache client could not connect.
    #[error("failed to initialize the cache client: {0}")]
    Cache(#[from] CacheError),
    /// `DATABASE_URL` is not a valid Postgres URL.
    #[error("invalid DATABASE_URL: {0}")]
    Database(#[from] sqlx::Error),
}

/// The clients behind [`AppState`], kept separately so `main` can register
/// their cleanup hooks.
pub struct Infrastructure {
    /// State the router serves from.
    pub state: AppState,
    /// Postgres pool (cloned into `state`).
    pub database: Database,
    /// Cache connection (cloned into `state`).
    pub cache: CacheClient,
}

impl Infrastructure {
    /// Connects the cache, configures the lazy database pool, registers the
    /// readiness check and wires the demo notes service.
    pub async fn build(config: &AppConfig) -> Result<Self, BuildError> {
        let cache = CacheClient::connect(&config.cache).await?;
        let database = Database::connect_lazy(&config.database_url)?;
        tracing::info!("database pool configured");

        let mut state = AppState {
            cache: cache.clone(),
            database: Some(database.clone()),
            ..AppState::default()
        };
        state.readiness.register(database.clone());
        // example — remove when building your service (notes demo slice)
        state.notes = Some(Arc::new(NotesService::new(Arc::new(
            PgNotesRepository::new(database.pool().clone()),
        ))));

        Ok(Self {
            state,
            database,
            cache,
        })
    }
}
