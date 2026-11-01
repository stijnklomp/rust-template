# AGENTS.md

Rust microservice template: one service app (axum + tokio) plus tested library modules. Vocabulary: `GLOSSARY.md`. Decisions: `docs/adr/`. Fastify → Rust feature map: `docs/PORTING.md`.

## Commands

Everything runs through `just` (ADR-0010: the recipes are the gates; there are no CI files). `just` alone lists them.

| Recipe | Purpose |
| --- | --- |
| `just check` | The gate: `fmt-check`, `lint`, `sqlx-check`, `spec-check`, `test`. Needs Postgres on localhost:5432. |
| `just test` | Fast suite: unit + feature tests (nextest) + doctests. No infrastructure. |
| `just test-component` | Component suite: this service against real Postgres/Valkey/RabbitMQ. Needs the infra stack. |
| `just test-spec` | Acceptance spec, in-process and concurrent. Needs Postgres. |
| `just test-blackbox <url>` | The same spec, black-box against a deployed service (serial). |
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
- **Specs are human-owned**: `.feature` files under `tests/acceptance/specs/` are the executable spec (ADR-0011). Never edit one without human approval; after review run `just spec-approve` to refresh `specs.lock`. `just spec-check` (part of `just check`) fails on an unapproved change. Step definitions must not import `rust_template::`.
- **Unit tests stay inline**: `#[cfg(test)] mod tests` in `src/`; the suites under `tests/` are feature/component/acceptance/integration (ADR-0012).
- **Run `just check` before declaring work done.**

## Crate map

- `src/main.rs` — wiring only: config → logging → `app::Infrastructure` → router → serve → cleanup.
- `src/app.rs` — `Infrastructure::build`, the composition shared by `main` and the in-process spec driver.
- `src/lib.rs` — `routes` → `controllers` → `services` → `repositories` (+ `models` DTOs), `infrastructure/` (cache), `amqp.rs` + `consumer.rs`, `config.rs`, `logging.rs`, `problem.rs` (RFC 9457), `health.rs`, `server.rs`, `shutdown.rs`, `openapi.rs`.
- `src/bin/` — `openapi` (writes `openapi.yaml`), `db_seed` (demo tooling).
- `tests/feature/` — in-process feature tests (fakes, no infra); `tests/component/` — real datastores/broker; `tests/acceptance/` — the Cucumber spec (`runner.rs`, `driver.rs`, `world.rs`, `steps/`, `specs/`); `tests/integration/` — cluster e2e example (ignored); `tests/pact/` — contracts.
- `migrations/`, `docker-compose.yml` (profiles: infra/dev/local/publish/test/acceptance), `k8s/` (plain manifests), `pacts/` (checked-in contracts), `examples/`.

## Pitfalls

- nextest has no doctest support; `just test` chains `cargo test --doc` for them — don't drop it.
- Cucumber uses `harness = false`, which nextest cannot list: the spec target is behind `--features spec` and runs through `just test-spec` / `just test-blackbox`, never `cargo nextest`.
- `just test-component` needs the `infra` stack; the default nextest profile filters `binary(component)` out.
- `SPEC_TARGET` switches the spec from in-process (concurrent, unique labels) to black-box (serial, no retries).
- `docker compose down` needs the same `--profile` flag as `up` — every service is profile-gated, so a bare `down` is a no-op.

## Agent docs

- Issue tracker: GitHub issues via `gh` — `docs/agents/issue-tracker.md`.
- Triage labels — `docs/agents/triage-labels.md`.
- Domain docs: single context — `GLOSSARY.md` + `docs/adr/` — `docs/agents/domain.md`.
- Skills: `.opencode/skills/` (architecture, development, testing, dependency-management, succinct).
