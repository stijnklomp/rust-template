//! Component tests: this service against its real Postgres, Valkey and
//! RabbitMQ, with no other services involved.
//!
//! Run them with `just test-component` (needs the compose `infra` profile).
//! The default nextest profile excludes this binary, so `just test` stays
//! infrastructure-free. Serial by design: the tests share one broker and one
//! database.

#[path = "../common/mod.rs"]
mod common;

mod amqp;
mod cache;
mod consumer;
mod mq;

use rust_template::config::{CacheConfig, RabbitConfig, RabbitTransport};

/// Serializes tests that declare, mutate or observe the shared broker
/// topology. Shared by every module so plain `cargo test --test component`
/// (libtest, parallel) is safe; the nextest component profile is serial anyway.
pub(crate) static BROKER: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Reads a required environment variable, with a message pointing at the
/// intended entry point.
pub(crate) fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| {
        panic!(
            "{name} is not set; start the infrastructure stack \
             (`docker compose --profile infra up -d`) and run `just test-component`"
        )
    })
}

/// RabbitMQ settings from the environment, always enabled (the component
/// suite has a real broker).
pub(crate) fn rabbit_config() -> RabbitConfig {
    RabbitConfig {
        enabled: true,
        host: required_env("RABBIT_HOST"),
        port: required_env("RABBIT_PORT")
            .parse()
            .expect("RABBIT_PORT must be a port number"),
        user: required_env("RABBIT_USER"),
        password: required_env("RABBIT_PASSWORD"),
        transport: RabbitTransport::Amqp,
    }
}

/// Valkey settings from the environment, always enabled.
pub(crate) fn cache_config() -> CacheConfig {
    CacheConfig {
        disabled: false,
        host: required_env("CACHE_HOST"),
        port: required_env("CACHE_PORT")
            .parse()
            .expect("CACHE_PORT must be a port number"),
        password: required_env("CACHE_PASSWORD"),
    }
}
