//! Step definitions: the shared vocabulary specs are written in.
//!
//! `http.rs` holds transport-level steps reused by every feature; one module
//! per capability holds its domain steps. Nothing here may import the
//! application crate — the spec must survive an implementation rewrite
//! (ADR-0011).

mod http;
mod notes;
