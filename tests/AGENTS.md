# AGENTS.md — tests

Scope: everything under `tests/` and the test recipes. The root `AGENTS.md` holds repo-wide rules; the closest `AGENTS.md` wins. Unit tests are **not** here — they live inline in `src/` (`#[cfg(test)] mod tests`).

## Overview

`tests/acceptance/specs/*.feature` is the executable specification (ADR-0011). Humans own and approve it; the Rust code is the implementation. Never edit a `.feature` without approval — after review, `just spec-approve` refreshes `specs.lock`, and `just spec-check` (in `just check` and the pre-commit hook) fails otherwise.

## Setup & environment

- Unit and feature suites need only the pinned Rust toolchain: `just test`.
- Host-run acceptance and component suites need the infra stack: `docker compose --profile infra up -d` (Postgres :5432, Valkey :6379, RabbitMQ :5672).
- Compose recipes (`test-acceptance`, `test-blackbox-compose`, `acceptance`) need only Docker; the toolchain and crates are cached in the gitignored `.acceptance/`.

## Suites

| Suite | Location | Proves | Command | Needs |
| --- | --- | --- | --- | --- |
| Unit | `src/**` (`#[cfg(test)] mod tests`) | pure logic with private access; the TDD loop | `just test` | nothing |
| Feature | `tests/feature/` | one capability through the real router and middleware, with fakes at repository seams | `just test` | nothing |
| Acceptance — in-process | `tests/acceptance/specs/` via `runner.rs` | the approved spec against the real router and real Postgres/Valkey; concurrent, unique data per scenario | `just test-spec` | Postgres |
| Acceptance — black-box | the same specs, `SPEC_TARGET` set | the same spec over HTTP against a deployed or compose-hosted service; serial, no retries | `just test-blackbox <url>` / `just test-blackbox-compose` | a deployment / Docker |
| Component | `tests/component/` | this service against real Postgres/Valkey/RabbitMQ, with no peer services | `just test-component` | infra stack |
| Integration | `tests/integration/` | cluster e2e with other deployed services; empty ignored example until one exists | manual (`E2E_BASE_URL`, `--run-ignored all`) | staging cluster |
| Contract | `tests/pact/` | consumer/provider pacts (ADR-0008) | `just test-pact` | Postgres |

`just check` is the fast gate (unit + feature + doctests, lint, sqlx cache, spec lock); `just integrity` adds the fast-suite coverage gate, component, spec, compose black-box, pact and acceptance gates.

## Coverage

`just coverage` runs the default nextest profile (unit + feature) and writes `lcov.info`; `just coverage-check` is the **100% fast-suite gate** (ADR-0013): every measured line must be covered. It runs in `just integrity`, not `just check` (coverage instrumentation is slower and needs `cargo-llvm-cov`).

- The measured surface is unit + feature code. `src/main.rs`, `src/app.rs`, `src/bin/`, `src/repositories/`, `src/database.rs`, `src/amqp.rs`, `src/consumer.rs` and `src/infrastructure/` are excluded in `justfile`: their tests are the component/acceptance suites, or they are entrypoints.
- For a line that genuinely cannot be exercised by the fast suite, add a reason-bearing marker:
  - `// coverage:ignore: <reason>` for one line;
  - `// coverage:ignore-start: <reason>` … `// coverage:ignore-end` for a block.
- `scripts/coverage-check.sh` filters those lines from the LCOV report. Stable Rust has no line-level coverage exclusion, so this is the deliberate seam; do not use a marker to hide a line that a fast-suite test could cover, and do not add a business module to `coverage_ignore`.
- This gate does not measure component/acceptance coverage. Those suites remain the evidence for real SQL, broker/cache and startup wiring.

## Key files

- `tests/acceptance/runner.rs` — boots the in-process app via `app::Infrastructure`, applies migrations once, and configures Cucumber (`fail_on_skipped`, concurrency, black-box serial).
- `tests/acceptance/driver.rs` — the only transport seam: `Api::{InProcess, Http}`; `SPEC_TARGET` selects black-box.
- `tests/acceptance/world.rs` — per-scenario state and the unique run label.
- `tests/acceptance/steps/` — step definitions: `http.rs` for transport assertions, one module per capability.
- `tests/acceptance/specs/` — the approved Gherkin files and `specs.lock`.
- `tests/feature/main.rs`, `tests/component/main.rs`, `tests/integration/main.rs` — one module tree per suite.
- `tests/common/mod.rs` — broker/unique helpers for the component suite.
- `tests/pact/` — consumer generation and provider verification.
- `scripts/coverage-check.sh` — filters `coverage:ignore` markers from LCOV and enforces zero uncovered lines.

## Examples

Follow the demo slice: `tests/acceptance/specs/notes.feature` + `tests/acceptance/steps/notes.rs` (domain steps, problem+json), `tests/feature/notes.rs` (fake repository) and `tests/component/mq.rs` (real broker, DLQ). They are marked `example — remove when building your service`.

## Choosing a suite

| Change | Suite |
| --- | --- |
| Pure function or private helper | Unit, inline in `src/` |
| One capability through the router with a fake repository | Feature |
| Behaviour a human should approve, or the HTTP contract | Acceptance spec (in-process; also run black-box before deploying) |
| Real database, cache or broker semantics | Component |
| Flow across deployed services | Integration (staging only) |
| Consumer/provider contract | Pact |

## Conventions

- Specs describe domain outcomes, not routes or functions; transport vocabulary belongs in `tests/acceptance/steps/http.rs`.
- Steps must not import `rust_template::`; only `tests/acceptance/driver.rs`, `world.rs` and `runner.rs` know how the app is assembled.
- Every scenario namespaces the data it writes with a unique label, so in-process scenarios run concurrently and black-box runs stay deterministic.
- In-process applies migrations once; black-box never migrates — the compose `migrate` service (or the deployment pipeline) does it before the app starts.
- Feature tests use hand-written fakes behind repository traits — no infrastructure, real middleware pipeline.
- Component tests share one real stack: serial, unique queue/key names, and the shared `BROKER` lock for broker-mutating tests.
- Integration tests are `#[ignore]`d until a staging cluster exists; they assert downstream effects with bounded polling, never fixed sleeps.
- Use the API for assertions; query SQL only for asynchronous side effects that the API cannot observe yet.

## Security & safety

- Never commit real credentials, production data or captured traffic; fixtures are synthetic.
- The compose database is disposable: reset it with `docker compose --profile test down -v`. Never point the suite at a shared environment.

## Running tests

| Recipe | Runs |
| --- | --- |
| `just test` | Unit + feature tests (nextest) and doctests |
| `just test-component` | This service against real Postgres/Valkey/RabbitMQ (serial) |
| `just test-spec` | The spec in-process, concurrent, migrations once |
| `just test-blackbox <url>` | The same spec over HTTP against a deployment, serial |
| `just test-blackbox-compose` | The same spec against the compose-hosted app, serial |
| `just test-pact` | Pact consumer generation + provider verification |
| `just test-acceptance` | Component + in-process spec once against its own compose stack |
| `just acceptance` | Idle compose stack: `docker compose --profile acceptance exec ...` |
| `just coverage` | LCOV for the fast suite (`lcov.info`, gitignored), with coverage markers filtered |
| `just coverage-check` | 100% zero-uncovered fast-suite gate (runs `coverage` first) |
| `just spec-check` / `just spec-approve` | Verify / re-approve `specs.lock` after human review |

## Pitfalls

- Cucumber uses `harness = false`, which nextest cannot list: the spec target is behind `--features spec` and never runs under `cargo nextest`.
- `SPEC_TARGET` switches the same specs to black-box: serial, no retries — a flake is a bug, not a retry.
- The default nextest profile filters `binary(component)` out; component tests need `just test-component`.
- `coverage-check` fails with `file:line` for every uncovered measured line; add a real test or a justified marker, not a lower threshold.
- `coverage:ignore` markers are comments only; the filter script reads them from source, so an unmatched `ignore-start` fails the gate.
- `docker compose down` needs the same `--profile` flag as `up` (`test`, `acceptance`, `blackbox`); a bare `down` is a no-op.

## Commit checklist

- `just check` is green.
- New behaviour has the right suite, and spec changes were approved (`specs.lock` committed in the same change).
- Steps stayed domain-level and did not import `rust_template::`.
- Test data is namespaced; no real credentials are committed.

## When stuck

- A step is unbound → add it under `tests/acceptance/steps/` or fix the Gherkin; `fail_on_skipped` makes the run red by design.
- The suite cannot reach infrastructure → start `docker compose --profile infra up -d`, or use the compose recipes.
- Black-box runs time out → read `docker compose --profile blackbox logs app`; the runner polls `/readyz` for up to 180s.
- A spec lock check fails → review the `.feature` diff, then `just spec-approve`.
