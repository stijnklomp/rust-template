// example — remove when building your service
//
// Removing the demo slice also deletes the checked-in
// `pacts/notes-client-rust-template.json` and the notes provider states below
// (`PactNotesStates` and its fixtures).
//
// Provider side of the Pact example (ADR-0008).
//
// Boots the real app on an ephemeral port against the real Postgres from the
// app's own configuration, then verifies **every checked-in pact in
// `pacts/`** (drop a new file in and it is picked up). Provider states run
// through [`PactNotesStates`]: setup creates rows through the app's
// repository, teardown deletes the pact-owned rows it created.
//
// Needs Postgres on `localhost:5432` (`.env.development` `DATABASE_URL`); run
// via `just test-pact`.

#[path = "common.rs"]
mod common;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use pact_models::provider_states::ProviderState;
use pact_verifier::callback_executors::ProviderStateExecutor;
use pact_verifier::{
    FilterInfo, NullRequestFilterExecutor, PactSource, ProviderInfo, ProviderTransport,
    PublishOptions, VerificationOptions, verify_provider_async,
};
use rust_template::config::AppConfig;
use rust_template::database::Database;
use rust_template::models::note::CreateNoteBody;
use rust_template::repositories::notes::{NotesRepository, PgNotesRepository};
use rust_template::server::{self, AppState};
use rust_template::services::notes::NotesService;
use rust_template::shutdown;
use serde_json::{Value, json};
use sqlx::PgPool;

use common::PACT_OWNER;

#[tokio::test]
async fn verifies_every_checked_in_pact_against_the_real_app() {
    let config = AppConfig::load()
        .expect("app config (run from the repo root; `.env.development` needs DATABASE_URL)");
    let database = Database::connect_lazy(&config.database_url).expect("valid DATABASE_URL");
    let pool = database.pool().clone();
    let repository = Arc::new(PgNotesRepository::new(pool.clone()));

    // Rows a crashed run may have left behind would break the checks below.
    delete_pact_rows(&pool).await.expect("pre-run cleanup");

    // The real app — same router, pool and repository as `main` — on an
    // ephemeral port.
    let listener = tokio::net::TcpListener::bind(("127.0.0.1", 0))
        .await
        .expect("bind an ephemeral port");
    let port = listener.local_addr().expect("listener address").port();
    let state = AppState {
        database: Some(database.clone()),
        notes: Some(Arc::new(NotesService::new(repository.clone()))),
        ..AppState::default()
    };
    let (trigger, watch) = shutdown::channel();
    let app = tokio::spawn(server::serve(
        listener,
        server::build_router(state),
        watch,
        Duration::from_secs(5),
    ));

    let pact_files = pact_files();
    assert!(
        !pact_files.is_empty(),
        "no pacts in ./pacts — `just test-pact` generates the consumer pact first"
    );

    let publish = publish_options();
    let states = Arc::new(PactNotesStates::new(repository, pool.clone()));
    let result = verify_provider_async(
        provider_info(port),
        pact_sources(&pact_files),
        FilterInfo::None,
        vec![],
        &VerificationOptions {
            coloured_output: false,
            ..VerificationOptions::<NullRequestFilterExecutor>::default()
        },
        publish.as_ref(),
        &states,
        None,
    )
    .await
    .expect("run provider verification");

    delete_pact_rows(&pool).await.expect("post-run cleanup");
    trigger.trigger();
    app.await.expect("server task").expect("clean shutdown");
    database.close().await;

    for file in &pact_files {
        println!("verified {}", file.display());
    }
    for interaction in &result.interaction_results {
        let outcome = if interaction.result.is_ok() {
            "OK"
        } else {
            "FAILED"
        };
        println!("  - {} [{outcome}]", interaction.interaction_description);
    }

    assert!(
        result.result,
        "pact verification failed:\n{}\n{:#?}",
        result.output.join("\n"),
        result.errors
    );
    assert!(
        result.interaction_results.len() >= pact_files.len(),
        "every pact file must contribute at least one interaction"
    );
}

/// Provider-state handlers: a map from state name to real setup/teardown work.
///
/// Setup inserts through the app's repository (the same create path the
/// service uses); an unknown state is an error, so a pact that names a state
/// no handler knows cannot pass silently.
#[derive(Debug)]
struct PactNotesStates {
    repository: Arc<PgNotesRepository>,
    pool: PgPool,
}

impl PactNotesStates {
    fn new(repository: Arc<PgNotesRepository>, pool: PgPool) -> Self {
        Self { repository, pool }
    }
}

#[async_trait]
impl ProviderStateExecutor for PactNotesStates {
    async fn call(
        self: Arc<Self>,
        _interaction_id: Option<String>,
        provider_state: &ProviderState,
        setup: bool,
        _client: Option<&reqwest::Client>,
    ) -> anyhow::Result<HashMap<String, Value>> {
        if !setup {
            // Teardown runs after every interaction: drop the fixture rows.
            delete_pact_rows(&self.pool).await?;
            return Ok(HashMap::new());
        }

        match provider_state.name.as_str() {
            // Interactions without a `given` still receive an empty state.
            "" => Ok(HashMap::new()),
            "a note exists" => {
                let owner = state_param(provider_state, "owner")?;
                let note = state_param(provider_state, "note")?;
                let created = self
                    .repository
                    .create(CreateNoteBody { owner, note })
                    .await?;
                Ok(HashMap::from([("noteId".to_owned(), json!(created.id))]))
            }
            unknown => Err(anyhow::anyhow!(
                "no handler for provider state '{unknown}' — add it to PactNotesStates"
            )),
        }
    }

    fn teardown(&self) -> bool {
        true
    }
}

/// The provider under verification: the app on the ephemeral port.
fn provider_info(port: u16) -> ProviderInfo {
    #[allow(deprecated)] // the struct still carries legacy host/port fields
    ProviderInfo {
        name: rust_template::APP_NAME.to_owned(),
        host: "127.0.0.1".to_owned(),
        transports: vec![ProviderTransport {
            transport: "http".to_owned(),
            port: Some(port),
            path: None,
            scheme: Some("http".to_owned()),
        }],
        ..ProviderInfo::default()
    }
}

/// Every `pacts/*.json`: dropping a new pact in is picked up automatically.
fn pact_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir("pacts")
        .expect("the `pacts` directory is checked in")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .collect();
    files.sort();
    files
}

/// Where the pacts to verify come from. Broker hook (ADR-0008): the template
/// has no broker, so the checked-in files are the source. A broker deployment
/// replaces this with `PactSource::BrokerUrl` / `BrokerWithDynamicConfiguration`.
fn pact_sources(pact_files: &[PathBuf]) -> Vec<PactSource> {
    pact_files
        .iter()
        .map(|path| PactSource::File(path.display().to_string()))
        .collect()
}

/// Whether verification results are published. Broker hook (ADR-0008): no
/// broker exists, so nothing is published. With broker sources returned by
/// [`pact_sources`], build `PublishOptions { provider_version: Some(<build>),
/// .. }` here and the verifier publishes the results itself.
fn publish_options() -> Option<PublishOptions> {
    None
}

/// Deletes every row this verification owns. Test-only cleanup, so a plain
/// (non-macro) query keeps the feature-gated pact target out of the committed
/// `.sqlx` offline cache.
async fn delete_pact_rows(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM note WHERE owner = $1")
        .bind(PACT_OWNER)
        .execute(pool)
        .await?;
    Ok(())
}

/// Reads a string parameter from a provider state.
fn state_param(provider_state: &ProviderState, key: &str) -> anyhow::Result<String> {
    provider_state
        .params
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "provider state '{}' must pass a '{key}' param",
                provider_state.name
            )
        })
}
