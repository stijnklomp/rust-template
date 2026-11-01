# Test layout: unit inline, suites under `tests/`

Tests are split by what they prove, not by which process they run in:

- **Unit** — `#[cfg(test)] mod tests` beside the code in `src/`. Private access, no I/O, the TDD loop.
- **Feature** — `tests/feature/`: one capability through the real router and middleware, in-process, with hand-written fakes at repository seams. No infrastructure.
- **Component** — `tests/component/`: this service against its real Postgres/Valkey/RabbitMQ, with no other services. Serial, because one stack is shared.
- **Acceptance** — `tests/acceptance/`: the executable spec (ADR-0011) — Gherkin, step definitions, and the two drivers.
- **Integration** — `tests/integration/`: cluster end-to-end tests against a staging environment and other deployed services. Empty in the template apart from an ignored example; nothing here can run until the service is deployed next to its peers.
- **Contract** — `tests/pact/`: consumer/provider pacts (ADR-0008).

The folder names mirror the Fastify template's `test/unit|feature|integration|acceptance` tree, with two Rust-driven differences. Unit tests cannot live outside `src/` without losing privacy, and a top-level `test/` (singular) is not a cargo test directory: using it requires `autotests = false` plus an explicit `[[test]]` entry for every file, so a new test file that nobody registers is silently never run. A false green contradicts the point of the suite, so the template keeps cargo's `tests/` convention, where discovery is automatic.

Two suites need a runner decision. `tests/feature`, `tests/component` and `tests/integration` are ordinary libtest targets and run under nextest (`just test`, `just test-component`). The acceptance spec uses Cucumber's `harness = false`, which nextest cannot list, so it is a separate target behind the `spec` feature and runs through `just test-spec`, `just test-blackbox` (deployed) or `just test-blackbox-compose` (local compose). `just check` covers formatting, clippy, the `.sqlx` cache, the spec lock and the fast suite; `just integrity` adds the component, spec, compose black-box, pact and acceptance gates.
