//! Apply pending migrations without the `sqlx` CLI.
//!
//! Used by `just migrate` and by the compose black-box/acceptance profiles'
//! one-shot migration service, so neither needs `sqlx-cli` installed. The
//! migrations are embedded from `migrations/` at compile time (the same macro
//! the acceptance runner uses), so this works with the committed offline cache.

use std::process::ExitCode;

use rust_template::config::AppConfig;
use rust_template::database::Database;

#[tokio::main]
async fn main() -> ExitCode {
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("failed to load configuration: {error}");
            return ExitCode::FAILURE;
        }
    };

    let database = match Database::connect_lazy(&config.database_url) {
        Ok(database) => database,
        Err(error) => {
            eprintln!("invalid DATABASE_URL: {error}");
            return ExitCode::FAILURE;
        }
    };

    match sqlx::migrate!("./migrations").run(database.pool()).await {
        Ok(()) => {
            println!("migrations applied");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("migrations failed: {error}");
            ExitCode::FAILURE
        }
    }
}
