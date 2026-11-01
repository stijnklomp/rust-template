---
name: testing
description: Use when writing or running tests, or when deciding whether a test needs real infrastructure.
---

# Testing

- `just test` — fast suite: unit + feature tests (axum-test, in-process) plus doctests. No infrastructure.
- `just test-component` — this service against real Postgres/Valkey/RabbitMQ (serial). Needs the compose `infra` profile.
- `just test-spec` — the executable spec (`tests/acceptance/specs/*.feature`), in-process and concurrent. Needs Postgres.
- `just test-blackbox <url>` — the same spec against a deployed service, serial, no retries.
- `just test-pact` — contract tests, both sides. Needs Postgres.
- `just test-acceptance` — component + in-process spec through the compose `test` profile, once. `just acceptance` leaves that stack idle for focused exec runs.
- `just check` is the pre-finish gate; `just coverage` writes LCOV.

Specs are human-owned: never edit `tests/acceptance/specs/*.feature` without approval; `just spec-approve` refreshes the lock after review, and `just spec-check` fails the gate otherwise. Unit tests stay inline in `src/` (ADR-0012); step definitions live in `tests/acceptance/steps/` and must not import `rust_template::` (ADR-0011).
