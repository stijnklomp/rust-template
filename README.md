# rust-template

A Rust microservice template: a single service app (axum + tokio) that ports the patterns of the Fastify template — layered HTTP, typed config, sqlx with a committed offline cache, optional Valkey and RabbitMQ clients, one error contract, generated OpenAPI, real acceptance tests — with `just` recipes as the only command surface (ADR-0010).

## Features

- **HTTP** — axum + tower-http: request IDs, structured request logs, response compression, 1 MiB body limit, request timeout, sensitive-header redaction; CORS present but off by default. Every non-2xx is RFC 9457 `application/problem+json` (ADR-0005).
- **OpenAPI 3.1** — generated from the code (DTO annotations + route macros) on non-production startup and by `just openapi`; never hand-edited (ADR-0004).
- **Postgres** — sqlx with compile-time-checked SQL against a committed `.sqlx` offline cache, so builds, tests and image builds never need a live database (ADR-0003). Migrations and seed tooling included.
- **Cache** — Valkey/Redis client, gated by `CACHE_DISABLED`.
- **AMQP** — lapin publisher and a tested consumer module with the frozen always-ack/DLQ semantics (ADR-0006). Dedicated consumer deployments are a per-service recipe (ADR-0002).
- **Logging** — JSON on stdout in every environment, with independent `REQUEST_LOG_LEVEL` and `APP_LOG_LEVEL` filters (ADR-0007).
- **Lifecycle** — graceful shutdown drains HTTP, cancels consumers and closes pools; `GET /healthz` (liveness) and `GET /readyz` (readiness with a live database check).
- **Testing** — unit/feature tests (nextest + axum-test), pact contract tests on both sides (ADR-0008), acceptance tests against real Postgres/Valkey/RabbitMQ.
- **Containers** — cargo-chef build → distroless `nonroot` runtime (ADR-0009); compose profiles for every flow; multi-arch publishes; plain `k8s/` manifests.
- **Supply chain** — cargo-deny (licenses, advisories, bans, sources), CycloneDX SBOM, cargo-auditable release binaries.
- **Demo slice** — the notes routes, their tests and the seed tooling are marked `example — remove when building your service`; they show the full layering end-to-end and are meant to be deleted.

## Getting started

Prerequisites: Docker with Compose v2. The `just` recipes additionally need the pinned Rust toolchain (`rust-toolchain.toml`, 1.99.1), `just`, and the cargo tools the recipes invoke (`cargo-nextest`, `sqlx-cli`, `cargo-deny`, `cargo-cyclonedx`, `cargo-auditable`, `cargo-llvm-cov`).

Infrastructure in Docker, app on the host:

```bash
docker compose --profile infra up -d   # Postgres, Valkey, RabbitMQ
just migrate                           # apply migrations
just db-seed                           # optional: mock notes (destructive, dev-only)
just dev                               # serve on http://localhost:3000
```

Or the app from mounted source inside Docker, with hot reload and no host toolchain:

```bash
docker compose --profile dev up        # bacon streams rebuilds to the logs
docker compose --profile dev down
```

Both read the committed `.env.development`; the dev container only rewrites the endpoints to its in-network services.

## Configuration

`APP_ENV` (default `development`) picks the committed env file: `.env.development`, `.env.production` or `.env.tests`. The gitignored `.env` (personal overrides) wins over it, and the real process environment wins over both. The typed config (`src/config.rs`) validates everything at startup and fails fast with a readable error.

Logging is JSON on stdout in every environment (ADR-0007), with two independent verbosity filters:

| Variable | Controls |
| --- | --- |
| `REQUEST_LOG_LEVEL` | One line per HTTP request: method, path, status, latency, request ID. `authorization` is never logged. |
| `APP_LOG_LEVEL` | Everything else: business logic, lifecycle, infrastructure. |

Health: `GET /healthz` → 204; `GET /readyz` → 204, or 503 while Postgres is unreachable.

## Testing

| Command | Runs | Needs |
| --- | --- | --- |
| `just test` | Fast suite: unit + feature tests (nextest) and doctests | nothing |
| `just test-pact` | Pact consumer generation + provider verification | Postgres on :5432 |
| `just test-acceptance` | Acceptance suite once, against its own compose stack; exits with the suite's status | Docker |
| `just acceptance` | Idle acceptance stack for focused runs | Docker |
| `just coverage` | LCOV report for the fast suite (`lcov.info`) | host toolchain |

`just check` runs the fast gates — formatting, clippy with warnings denied, the sqlx offline-cache check and the fast suite. `just integrity` runs every gate end-to-end and stops at the first failure.

Focused acceptance runs, against the idle stack:

```bash
just acceptance
docker compose --profile acceptance exec acceptance \
  cargo nextest run --features acceptance --profile acceptance -E 'test(cache_round_trips)'
docker compose --profile acceptance down -v
```

## Containers

`just docker-build` produces `rust-template:local`: a cargo-chef dependency layer on the pinned Rust slim image, then the binary in a distroless `cc-debian13:nonroot` runtime — no shell, non-root, glibc and CA certificates only (ADR-0009). The build is hermetic: `--locked`, sqlx resolves from the committed offline cache.

Compose profiles cover every flow:

| Profile | Start with | What runs |
| --- | --- | --- |
| `infra` | `docker compose --profile infra up -d` | Postgres, Valkey, RabbitMQ (management UI on :15672) |
| `dev` | `docker compose --profile dev up` | infra + the app from mounted source, hot reload |
| `local` | `docker compose --profile local up --build` | infra + the app built from the Dockerfile |
| `publish` | `IMAGE=registry/app:tag docker compose --profile publish up` | infra + a published image, no build |
| `test` | `just test-acceptance` | infra + a one-shot acceptance runner |
| `acceptance` | `just acceptance` | infra + an idle runner for exec-driven runs |

Every service is profile-gated, so `down` takes the same `--profile` flag as `up` (a bare `docker compose down` is a no-op here). Published ports are remappable through the environment (`API_PORT`, `DATABASE_PORT`, `CACHE_PORT`, `RABBIT_PORT`, `RABBIT_MGMT_PORT`), so this stack can run next to another one.

`just docker-publish` builds and pushes an amd64 + arm64 manifest for `$IMAGE` (default `rust-template:latest`); it needs a buildx builder with QEMU and a registry login. `k8s/` ships plain `Deployment`/`Service`/`ConfigMap` manifests — hardened pod, `/healthz` + `/readyz` probes, `APP_ENV=production`, and an image tag to replace with the published one.

## Documentation

- `AGENTS.md` — commands, rules and crate map for agents working in the repo.
- `GLOSSARY.md` — the vocabulary this template and the services built from it use.
- `docs/PORTING.md` — the Fastify template feature map, including deliberate divergences.
- `docs/adr/` — architecture decisions, one file per decision.
- `docs/research/rust-ecosystem-2026-10.md` — the ecosystem research behind the version picks.

## License

MIT — see `LICENSE`.
