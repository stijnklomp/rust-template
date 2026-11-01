# AGENTS.md — migrations

Scope: `migrations/` and schema changes. The root `AGENTS.md` holds repo-wide rules; the closest `AGENTS.md` wins.

## Overview

sqlx migrations are owned by this service (ADR-0003) and tracked in Postgres's `_sqlx_migrations` table. `src/bin/migrate.rs` and the acceptance runner embed them at compile time. The service app never migrates on startup.

## Setup & environment

- Postgres: `docker compose --profile infra up -d` (localhost:5432, `.env.development` credentials).
- `just migrate` applies pending migrations; `just db-reset` drops and recreates the database (destructive); `just db-seed` loads demo notes (destructive).
- The compose flows (`just test-blackbox-compose`, `just test-acceptance`, `just acceptance`) run the one-shot `migrate` service before the app.

## Conventions

- One change per file, named `<UTC timestamp>_<verb>_<noun>.sql` (e.g. `20261016171227_create_note.sql`). Never edit or rename a migration that has run anywhere — add a new one.
- Keep migrations forward-only (no down files). Correct a mistake with a new migration.
- Guard destructive DDL (drop, rename, type change) and call it out in the PR.
- This service owns its tables. Do not add DDL for tables owned by another service; coordinate instead (ADR-0003).
- `20261016171227_create_note.sql` is the demo slice (`note` table). Remove it with the rest when building a real service.
- After adding or changing a migration: `just migrate && just sqlx-prepare`, then commit `migrations/` and `.sqlx/` together. `just check` runs `cargo sqlx prepare --check` and fails on a stale cache.

## Running migrations

| Recipe | Does |
| --- | --- |
| `just migrate` | Applies pending migrations |
| `just db-reset` | Drops and recreates the database, then runs every migration (destructive) |
| `just db-seed` | Loads demo notes; refuses `APP_ENV=production` (destructive) |
| `just sqlx-prepare` / `just sqlx-check` | Refresh / verify the committed `.sqlx` offline cache |

In deployment, run migrations as a separate step before rolling the app: the image ships only the service binary. Locally, `just migrate` is that step.

## Security & safety

- Never put credentials, production data or PII in migrations or seed data; fixtures are synthetic.
- `just db-reset` and `just db-seed` are destructive. Never point them at a shared environment.
- Review destructive DDL explicitly; there is no down migration to undo it.

## Examples

- `20261016171227_create_note.sql` — the timestamped `CREATE TABLE` format and the demo marker.
- For a real change, add a new file (never edit that one), run `just migrate && just sqlx-prepare`, and commit both trees.

## Checklist

- [ ] New timestamped file; no applied migration was edited.
- [ ] The migration applies on a fresh `just db-reset`.
- [ ] `.sqlx/` refreshed and committed with the migration.
- [ ] The demo marker is present only on the demo slice.
- [ ] `just check` is green.

## When stuck

- "migration was previously applied but has been modified" → restore the file from git and add a new migration instead.
- Stale `.sqlx` cache → run `just sqlx-prepare` and commit `.sqlx/`.
- Database unreachable → start `docker compose --profile infra up -d` and check `DATABASE_URL`.
- `db-reset` cannot drop the database → stop the app/containers using it, or reset the volume with `docker compose --profile infra down -v` (destructive) and start it again.
