# AGENTS.md

Rust microservice template: one service app (axum + tokio) plus tested library modules. The approved Gherkin spec in `tests/acceptance/specs/` is the source of truth for behaviour; the Rust code is its implementation. Vocabulary: `GLOSSARY.md`. Decisions: `docs/adr/`. Fastify → Rust feature map: `docs/PORTING.md`.

## Source of truth

`tests/acceptance/specs/*.feature` is the executable specification (ADR-0011). Humans own and approve it; a change is complete when the specs pass, and rewriting internals is safe if they still do.

- Never edit a `.feature` without human approval. After review, `just spec-approve` refreshes `tests/acceptance/specs.lock`; `just spec-check` (in `just check` and the pre-commit hook) fails on an unapproved change.
- Step definitions (`tests/acceptance/steps/`) implement the vocabulary and must not import `rust_template::`; only `driver.rs` / `world.rs` / `runner.rs` know how the app is assembled.
- The suite fails on an unbound step (`fail_on_skipped`), so rewording a spec cannot silently stop enforcing it.

## Tests

| Suite | Location | Proves | Command | Needs |
| --- | --- | --- | --- | --- |
| Unit | `src/**` (`#[cfg(test)] mod tests`) | pure logic with private access; the TDD loop | `just test` | nothing |
| Feature | `tests/feature/` | one capability through the real router and middleware, with fakes at repository seams | `just test` | nothing |
| Acceptance — in-process | `tests/acceptance/specs/` via `runner.rs` | the approved spec against the real router and real Postgres/Valkey; concurrent, unique data per scenario | `just test-spec` | Postgres |
| Acceptance — black-box | the same specs, `SPEC_TARGET` set | the same spec over HTTP against a deployed or compose-hosted service; serial, no retries | `just test-blackbox <url>` / `just test-blackbox-compose` | a deployment / Docker |
| Component | `tests/component/` | this service against real Postgres/Valkey/RabbitMQ, with no peer services | `just test-component` | infra stack |
| Integration | `tests/integration/` | cluster e2e with other deployed services; empty ignored example until one exists | — | staging cluster |
| Contract | `tests/pact/` | consumer/provider pacts (ADR-0008) | `just test-pact` | Postgres |

`just check` is the fast gate (unit + feature + doctests, lint, sqlx cache, spec lock); `just integrity` adds the component, spec, compose black-box, pact and acceptance gates.

## Commands

Everything runs through `just` (ADR-0010: the recipes are the gates; there are no CI files). `just` alone lists them.

| Recipe | Purpose |
| --- | --- |
| `just check` | The gate: `fmt-check`, `lint`, `sqlx-check`, `spec-check`, `test`. Needs Postgres on localhost:5432. |
| `just test` | Fast suite: unit + feature tests (nextest) + doctests. No infrastructure. |
| `just test-component` | Component suite: this service against real Postgres/Valkey/RabbitMQ. Needs the infra stack. |
| `just test-spec` | Acceptance spec, in-process and concurrent. Needs Postgres. |
| `just test-blackbox <url>` | The same spec, black-box against a deployed service (serial). |
| `just test-blackbox-compose` | The same spec, black-box against the compose-hosted app (serial). |
| `just spec-check` / `just spec-approve` | Verify / re-approve `tests/acceptance/specs.lock` after a human reviewed the spec diff. |
| `just fmt` / `just fmt-check` | Format the crate / check formatting. |
| `just lint` | Clippy with `-D warnings`. |
| `just build` / `just dev` | Build / run the service app. |
| `just test-pact` | Pact consumer generation + provider verification. Needs Postgres. |
| `just test-acceptance` | Component suite + in-process spec once against its own compose stack; exits with the suite's status. |
| `just acceptance` | Idle acceptance stack for `docker compose --profile acceptance exec ...` runs. |
| `just coverage` | LCOV report for the fast suite (`lcov.info`, gitignored). |
| `just migrate` / `just db-reset` / `just db-seed` | Migrations / drop-and-recreate / demo data. Need Postgres; reset and seed are destructive. |
| `just sqlx-prepare` / `just sqlx-check` | Refresh / verify the committed `.sqlx` offline cache. |
| `just openapi` | Regenerate the committed `openapi.yaml` from the code. |
| `just licenses` / `just sbom` | cargo-deny checks / CycloneDX SBOM + auditable release build. |
| `just integrity` | Every gate end-to-end, aborting at the first failure. Heavy; needs infrastructure. |
| `just docker-build` / `just docker-publish` | Local image / multi-arch manifest build-and-push. |

## Rules

- **sqlx offline**: `.sqlx/` is committed and `.cargo/config.toml` sets `SQLX_OFFLINE=true`, so builds, tests and image builds never need a live database. After changing queries or migrations: `just migrate && just sqlx-prepare`, commit `.sqlx/` in the same change. `just check` runs `cargo sqlx prepare --check`, so a stale cache fails the gate. Never hand-edit `.sqlx/`.
- **OpenAPI is generated** (`just openapi`, ADR-0004): change the routes/DTOs and regenerate; never hand-edit `openapi.yaml`.
- **Demo slice**: files and lines marked `example — remove when building your service` (the notes routes, their tests, the seed tooling, the pact example plus its checked-in `pacts/` contract, and the `notes` OpenAPI tag) exist to materialize the template's patterns end-to-end; delete them when building a real service. Keep the markers until then, and don't mark new service code.
- **One binary** (ADR-0002): the app starts no consumers. A service that needs a dedicated consumer adds `src/bin/worker.rs` — the recipe and the frozen ack/DLQ semantics are in the `src/consumer.rs` module docs.
- **Specs are human-owned**: never edit a `.feature` without approval — see [Source of truth](#source-of-truth).
- **Tests stay in their layer**: unit tests inline in `src/`, suites under `tests/` — see [Tests](#tests) and ADR-0012.
- **Run `just check` before declaring work done.**

## Crate map

- `src/main.rs` — wiring only: config → logging → `app::Infrastructure` → router → serve → cleanup.
- `src/app.rs` — `Infrastructure::build`, the composition shared by `main` and the in-process spec driver.
- `src/lib.rs` — `routes` → `controllers` → `services` → `repositories` (+ `models` DTOs), `infrastructure/` (cache), `amqp.rs` + `consumer.rs`, `config.rs`, `logging.rs`, `problem.rs` (RFC 9457), `health.rs`, `server.rs`, `shutdown.rs`, `openapi.rs`.
- `src/bin/` — `openapi` (writes `openapi.yaml`), `migrate` (applies embedded migrations, no sqlx CLI), `db_seed` (demo tooling).
- `tests/` — suites per [Tests](#tests): `feature/`, `component/`, `acceptance/` (`runner.rs`, `driver.rs`, `world.rs`, `steps/`, `specs/`), `integration/`, `pact/`.
- `migrations/`, `docker-compose.yml` (profiles: infra/dev/local/publish/test/acceptance/blackbox), `k8s/` (plain manifests), `pacts/` (checked-in contracts), `examples/`.

## Pitfalls

- nextest has no doctest support; `just test` chains `cargo test --doc` for them — don't drop it.
- Cucumber uses `harness = false`, which nextest cannot list: the spec target is behind `--features spec` and runs through `just test-spec` / `just test-blackbox` / `just test-blackbox-compose`, never `cargo nextest`.
- `just test-component` needs the `infra` stack; the default nextest profile filters `binary(component)` out.
- `SPEC_TARGET` switches the spec from in-process (concurrent, unique labels) to black-box (serial, no retries).
- `docker compose down` needs the same `--profile` flag as `up` — every service is profile-gated, so a bare `down` is a no-op.

## Agent docs

- Issue tracker: GitHub issues via `gh` — `docs/agents/issue-tracker.md`.
- Triage labels — `docs/agents/triage-labels.md`.
- Domain docs: single context — `GLOSSARY.md` + `docs/adr/` — `docs/agents/domain.md`.
- Skills: `.opencode/skills/` (architecture, development, testing, dependency-management, succinct).
