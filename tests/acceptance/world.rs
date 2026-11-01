//! The Cucumber `World`: per-scenario state and the driver seam.
//!
//! Every scenario gets a fresh world with a unique label; all data it writes
//! is namespaced with that label, so in-process scenarios can run concurrently
//! and black-box runs stay deterministic on a shared deployment (ADR-0011).

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::driver::{Api, SpecRequest, SpecResponse};

/// Shared state for one scenario.
#[derive(Debug, cucumber::World)]
pub struct SpecWorld {
    /// Transport for this run (in-process or black-box).
    pub api: Api,
    /// Unique label every value this scenario writes carries.
    pub label: String,
    /// The last response, set by a request step.
    pub last: Option<SpecResponse>,
    /// Owner used by the note-creation step, after namespacing.
    pub created_owner: Option<String>,
    /// Text used by the note-creation step, after namespacing.
    pub created_text: Option<String>,
    /// Id of the created note.
    pub created_id: Option<i64>,
    /// `createdAt` of the created note.
    pub created_at: Option<String>,
    /// The page of notes found by the list step.
    pub seen_notes: Vec<Value>,
}

impl Default for SpecWorld {
    fn default() -> Self {
        Self {
            api: Api::from_env(),
            label: unique_label("spec"),
            last: None,
            created_owner: None,
            created_text: None,
            created_id: None,
            created_at: None,
            seen_notes: Vec::new(),
        }
    }
}

impl SpecWorld {
    /// Sends a request and remembers the response.
    pub async fn send(&mut self, request: SpecRequest) {
        self.last = Some(self.api.send(request).await);
    }

    /// The last response; a `Then` step without a preceding request is a bug
    /// in the feature file.
    pub fn response(&self) -> &SpecResponse {
        self.last
            .as_ref()
            .expect("a request step must run before an assertion step")
    }

    /// Namespaces `value` so this scenario cannot see or collide with another.
    pub fn unique(&self, value: &str) -> String {
        format!("{value} {}", self.label)
    }
}

/// A label no other scenario in any run uses.
pub fn unique_label(prefix: &str) -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is set")
        .as_nanos();
    format!("{prefix}-{}-{nanos}", std::process::id())
}
