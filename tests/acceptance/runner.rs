//! Acceptance spec runner (BDD): Cucumber over `specs/`.
//!
//! The suite is the executable specification: humans own `specs/*.feature`,
//! step definitions implement the vocabulary, and the runner fails on an
//! unbound step (`fail_on_skipped`) — a spec never silently stops being
//! enforced (ADR-0011).
//!
//! Two modes, selected by `SPEC_TARGET`:
//!
//! - unset: in-process, concurrent, migrations applied once — `just test-spec`;
//! - set: black-box, serial, against the deployed URL — `just test-blackbox`.
//!
//! The in-process app is built by [`rust_template::app::Infrastructure`], the
//! same composition `main` uses, so the spec cannot drift from production
//! wiring.

mod driver;
mod steps;
mod world;

use cucumber::World as _;
use rust_template::app::Infrastructure;
use rust_template::config::AppConfig;
use rust_template::server::build_router;

use world::SpecWorld;

/// Upper bound on concurrent in-process scenarios (the database pool and the
/// shared broker tolerate this comfortably; black-box runs are serial).
const MAX_IN_PROCESS_CONCURRENCY: usize = 8;

#[tokio::main]
async fn main() {
    // `reqwest` is built with `rustls-no-provider` (shared with the pact
    // crates), so install a crypto provider before any client is constructed.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let target = std::env::var(driver::TARGET_ENV).ok();
    let black_box = target.is_some();

    if black_box {
        // Never start the suite before the deployment is actually serving.
        driver::wait_until_ready(target.as_deref().expect("target checked above")).await;
    } else {
        boot_in_process().await;
    }

    SpecWorld::cucumber()
        .max_concurrent_scenarios(if black_box {
            1
        } else {
            in_process_concurrency()
        })
        .fail_on_skipped()
        .run_and_exit("tests/acceptance/specs")
        .await;
}

/// Builds the app exactly as `main` does and installs it in the driver.
async fn boot_in_process() {
    let config = AppConfig::load().expect("the acceptance configuration must load");
    let Infrastructure {
        state, database, ..
    } = Infrastructure::build(&config)
        .await
        .expect("the acceptance infrastructure must be reachable");

    // `sqlx::migrate!` applies only pending migrations, so repeated runs and
    // parallel stacks stay deterministic.
    sqlx::migrate!("./migrations")
        .run(database.pool())
        .await
        .expect("the acceptance migrations must apply");

    driver::install_in_process(axum_test::TestServer::new(build_router(state)));
}

/// How many scenarios the in-process mode runs at once.
fn in_process_concurrency() -> usize {
    std::thread::available_parallelism()
        .map_or(4, |parallelism| parallelism.get())
        .min(MAX_IN_PROCESS_CONCURRENCY)
}
