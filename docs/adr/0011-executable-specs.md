# Executable specs are the source of truth (BDD)

Humans approve behaviour, agents write code. The durable artifact is therefore the spec, not the implementation: `tests/acceptance/specs/*.feature` holds what the service promises, in Gherkin, and the same files run against two drivers — in-process (`axum-test` over the real router, real Postgres/Valkey) and black-box (`reqwest` against a deployed service). A rewrite is safe when the spec still passes.

Gherkin on `cucumber` 0.23 (cucumber-rs) is the chosen runner. It is the only mature Rust implementation with async steps, a step registry, tags, retries and `fail_on_skipped`; Gauge has no maintained Rust runner, `rstest-bdd` needs a hand-written Rust binding per scenario (so the spec alone is not sufficient), and `subplot`/`gherkin-cargo-test` either need an external toolchain or are sync-only and too young for a template. The spec text stays language-neutral: only step definitions and `driver.rs` are Rust.

Rules that keep the spec rewrite-proof and honest:

- Steps describe domain outcomes ("a note can be read back"), not route or function names. Transport-level vocabulary lives in `steps/http.rs`.
- Nothing under `tests/acceptance/steps/` may import `rust_template::`; `just spec-check` enforces it. Only `driver.rs`, `world.rs` and `runner.rs` know how the service is assembled.
- The in-process app is built by `rust_template::app::Infrastructure`, the same composition `main` uses, so the spec cannot drift from production wiring.
- `fail_on_skipped()` makes an unbound step fail the suite: rewording a step cannot silently un-test a feature.
- Every scenario namespaces the data it writes with a unique label. In-process scenarios therefore run concurrently; black-box scenarios run serially (`max_concurrent_scenarios(1)`), with no retries — a flake is a bug.
- In-process runs apply migrations once and use the real stack; black-box runs never touch the schema.
- `tests/acceptance/specs.lock` pins the approved `.feature` files by SHA-256. `just spec-check` runs in `just check`; changing a spec requires `just spec-approve`, a deliberately separate, reviewable step a human runs after approving the diff. This mirrors the committed `.sqlx` cache: the gate fails when the artifact is stale.

Unit tests stay next to the code (`#[cfg(test)] mod tests`), because Rust unit tests need private access and cargo compiles them into the library target; a `test/unit/` directory cannot do that. The acceptance suite is its own cargo target (`harness = false`, behind the `spec` feature) because nextest cannot list a custom harness; `just test-spec` and `just test-blackbox` are the entry points. The spec is deliberately not part of `just test`: it needs infrastructure, while the fast suite must not.

`tests/acceptance/specs/notes.feature` and its steps are the worked example, marked for removal like the rest of the demo slice.
