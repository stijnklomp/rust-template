//! Transport-level steps shared by every feature.

use cucumber::{then, when};

use crate::driver::SpecRequest;
use crate::world::SpecWorld;

/// Issues a `GET`; useful for operational endpoints the spec names directly.
#[when(expr = "I request {string}")]
async fn i_request(world: &mut SpecWorld, path: String) {
    world.send(SpecRequest::get(path)).await;
}

/// Asserts the status of the last response.
#[then(expr = "the response status is {int}")]
async fn response_status_is(world: &mut SpecWorld, status: u16) {
    let response = world.response();
    assert_eq!(
        response.status, status,
        "expected status {status}, got {}: {}",
        response.status, response.text
    );
}

/// Asserts the RFC 9457 content type and that the document describes this
/// response rather than a different one.
#[then("the response is a problem document")]
async fn response_is_a_problem_document(world: &mut SpecWorld) {
    let response = world.response();
    assert!(
        response
            .content_type
            .starts_with("application/problem+json"),
        "expected application/problem+json, got {:?}: {}",
        response.content_type,
        response.text
    );
    assert_eq!(
        response.body["status"].as_u64(),
        Some(u64::from(response.status)),
        "the problem status must match the response status: {}",
        response.text
    );
    assert!(
        response.body["title"]
            .as_str()
            .is_some_and(|title| !title.is_empty()),
        "the problem must carry a title: {}",
        response.text
    );
}

/// Asserts the JSON pointer a validation error points at.
#[then(expr = "the problem points at {string}")]
async fn problem_points_at(world: &mut SpecWorld, pointer: String) {
    let response = world.response();
    let errors = response.body["errors"]
        .as_array()
        .unwrap_or_else(|| panic!("expected an errors extension: {}", response.text));
    assert_eq!(
        errors[0]["pointer"], pointer,
        "expected the first error to point at {pointer}: {}",
        response.text
    );
}
