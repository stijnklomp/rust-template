// example — remove when building your service (notes demo slice)
//
// Domain steps for the notes capability. They speak in domain terms ("a note
// with an owner and a text"), not endpoint terms, so the feature files survive
// a rewrite of the route or the storage layer.

use cucumber::{given, then, when};
use serde_json::json;

use crate::driver::SpecRequest;
use crate::world::{SpecWorld, unique_label};

/// Namespaces every value this scenario writes.
#[given("a unique run label")]
async fn a_unique_run_label(world: &mut SpecWorld) {
    world.label = unique_label("notes");
}

/// Creates a note through the public API.
#[when(expr = "I create a note with owner {string} and text {string}")]
async fn create_note(world: &mut SpecWorld, owner: String, text: String) {
    let owner = world.unique(&owner);
    let text = world.unique(&text);
    world.created_owner = Some(owner.clone());
    world.created_text = Some(text.clone());
    world
        .send(SpecRequest::post(
            "/v1/notes",
            json!({ "owner": owner, "note": text }),
        ))
        .await;
}

/// Creates a note whose text exceeds the documented limit.
#[when(expr = "I create a note with owner {string} and a text of {int} characters")]
async fn create_note_with_text_of_length(world: &mut SpecWorld, owner: String, length: usize) {
    let owner = world.unique(&owner);
    let text = format!("{} {}", "x".repeat(length), world.label);
    world.created_owner = Some(owner.clone());
    world.created_text = Some(text.clone());
    world
        .send(SpecRequest::post(
            "/v1/notes",
            json!({ "owner": owner, "note": text }),
        ))
        .await;
}

/// Reads the list until the note created by this scenario is visible.
///
/// The database is shared, so previously stored notes may fill earlier pages.
#[when("I read the notes list")]
async fn read_notes_list(world: &mut SpecWorld) {
    let target = world
        .created_owner
        .clone()
        .expect("a note-creation step must run first");

    for page in 1..=50 {
        world
            .send(SpecRequest::get(format!(
                "/v1/notes?page={page}&perPage=100"
            )))
            .await;

        let notes = {
            let response = world.response();
            assert_eq!(response.status, 200, "GET page {page}: {}", response.text);
            response.body["notes"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        };

        if notes.is_empty() {
            break;
        }
        if notes.iter().any(|note| note["owner"] == target) {
            world.seen_notes = notes;
            return;
        }
    }

    panic!("the note owned by {target:?} was not visible within 50 pages");
}

/// Asserts the create response carries the stored note, camelCase and with
/// identical timestamps on a fresh row.
#[then("the response is the created note")]
async fn response_is_the_created_note(world: &mut SpecWorld) {
    let owner = world
        .created_owner
        .clone()
        .expect("a create step must run first");
    let text = world
        .created_text
        .clone()
        .expect("a create step must run first");

    let (id, created_at) = {
        let response = world.response();
        assert_eq!(
            response.body["message"], "Note Created",
            "{}",
            response.text
        );

        let note = &response.body["note"];
        assert_eq!(note["owner"], owner);
        assert_eq!(note["note"], text);
        assert!(
            note["createdAt"].as_str().is_some_and(|at| !at.is_empty()),
            "createdAt must be a timestamp string: {}",
            response.text
        );
        assert_eq!(
            note["createdAt"], note["updatedAt"],
            "a fresh note has identical timestamps: {}",
            response.text
        );
        assert!(
            note.get("created_at").is_none(),
            "JSON must be camelCase: {}",
            response.text
        );

        (
            note["id"].as_i64(),
            note["createdAt"].as_str().map(str::to_owned),
        )
    };

    world.created_id = id;
    world.created_at = created_at;
}

/// Asserts the list exposes the same row that the create response returned.
#[then("the note I created appears in the list")]
async fn created_note_appears_in_the_list(world: &mut SpecWorld) {
    let id = world
        .created_id
        .expect("the response step must run before this assertion");
    let text = world
        .created_text
        .clone()
        .expect("a create step must run first");
    let created_at = world
        .created_at
        .clone()
        .expect("the response step must run before this assertion");

    let found = world
        .seen_notes
        .iter()
        .find(|note| note["id"].as_i64() == Some(id))
        .unwrap_or_else(|| panic!("note {id} is not in the list"));

    assert_eq!(found["note"], text);
    assert_eq!(found["createdAt"], created_at);
}
