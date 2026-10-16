//! Values shared by the two Pact example crates (`tests/pact/consumer.rs` and
//! `tests/pact/provider.rs`); each crate `#[path]`-includes this file because
//! integration-test crates cannot share modules through the library.

/// Owner under which every pact fixture row is created: the consumer example
/// uses it in its provider-state params and its create request, and the
/// provider test deletes exactly those rows during teardown
/// (`DELETE FROM note WHERE owner = $1`) — never hand-seeded rows.
pub const PACT_OWNER: &str = "pact-notes-client";
