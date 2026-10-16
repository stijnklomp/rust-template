// example — remove when building your service
//!
//! `just db-seed`: destructive, dev-only seeding of the notes demo table.
//! Refuses to run with `APP_ENV=production`; truncates `note` (restarting the
//! id sequence) and inserts mock rows so the endpoints have data to serve.
//! Uses the same config/pool/query stack as the service app.

use std::process::ExitCode;

use rust_template::config::AppConfig;
use rust_template::database::Database;

/// Mock rows inserted by the seed (owner, note).
const MOCK_NOTES: [(&str, &str); 25] = [
    ("ada", "Seed note 1"),
    ("ada", "Seed note 2"),
    ("ada", "Seed note 3"),
    ("ada", "Seed note 4"),
    ("ada", "Seed note 5"),
    ("grace", "Seed note 6"),
    ("grace", "Seed note 7"),
    ("grace", "Seed note 8"),
    ("grace", "Seed note 9"),
    ("grace", "Seed note 10"),
    ("linus", "Seed note 11"),
    ("linus", "Seed note 12"),
    ("linus", "Seed note 13"),
    ("linus", "Seed note 14"),
    ("linus", "Seed note 15"),
    ("margaret", "Seed note 16"),
    ("margaret", "Seed note 17"),
    ("margaret", "Seed note 18"),
    ("margaret", "Seed note 19"),
    ("margaret", "Seed note 20"),
    ("turing", "Seed note 21"),
    ("turing", "Seed note 22"),
    ("turing", "Seed note 23"),
    ("turing", "Seed note 24"),
    ("turing", "Seed note 25"),
];

#[tokio::main]
async fn main() -> ExitCode {
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("failed to load configuration: {error}");
            return ExitCode::FAILURE;
        }
    };

    if config.app_env.is_production() {
        eprintln!("db-seed is destructive and refuses to run with APP_ENV=production");
        return ExitCode::FAILURE;
    }

    let database = match Database::connect_lazy(&config.database_url) {
        Ok(database) => database,
        Err(error) => {
            eprintln!("invalid DATABASE_URL: {error}");
            return ExitCode::FAILURE;
        }
    };

    match seed(&database).await {
        Ok(inserted) => {
            println!("seeded {inserted} notes into the `note` table");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("seeding failed: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Truncates `note` and inserts [`MOCK_NOTES`]; returns the number inserted.
async fn seed(database: &Database) -> Result<usize, sqlx::Error> {
    sqlx::query!("TRUNCATE TABLE note RESTART IDENTITY")
        .execute(database.pool())
        .await?;

    for (owner, note) in MOCK_NOTES {
        sqlx::query!(
            r#"INSERT INTO note (owner, note, "createdAt", "updatedAt")
               VALUES ($1, $2, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)"#,
            owner,
            note
        )
        .execute(database.pool())
        .await?;
    }

    Ok(MOCK_NOTES.len())
}
