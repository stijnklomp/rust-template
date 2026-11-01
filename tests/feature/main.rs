//! Feature tests: one feature end-to-end through the real router, in-process.
//!
//! These are the fast tests: no socket and no infrastructure. Services run
//! against hand-written fakes at their repository seams (the house idiom),
//! which is what lets a feature test prove "A calls B calls C" without a
//! database. Real-datastore behaviour lives in `tests/component/`, and the
//! executable spec lives in `tests/acceptance/` (ADR-0012).

mod health;
mod notes;
mod openapi;
mod request_logging;
mod shutdown;
mod smoke;
