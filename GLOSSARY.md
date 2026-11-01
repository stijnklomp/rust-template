# Rust microservice template

Vocabulary for this template and the services built from it.

## Service shape

**Service app**:
The single binary this template builds; serves HTTP and starts no message consumers by default.
_Avoid_: api service, api mode

**Worker**:
A consumer-only process a service adds when it needs one (typically a small `src/bin/worker.rs`); not shipped by the template.
_Avoid_: worker mode, consumer service

**Consumer module**:
The library module that declares queues and runs the consume loop (always-ack, DLQ republish). Tested and shipped; wired only by services that consume.
_Avoid_: listener, subscriber

**Demo slice**:
The marked example code (notes routes, their tests, seed tooling) whose only purpose is to materialize the template's patterns; removed when building a real service.
_Avoid_: sample, boilerplate, scaffold

## Logging

**Request log**:
Per-request HTTP log line; verbosity controlled by `REQUEST_LOG_LEVEL`.

**App log**:
All non-request logs — business logic, lifecycle, infrastructure; verbosity controlled by `APP_LOG_LEVEL`. (Replaces the Fastify template's `CUSTOM_LOG_LEVEL`.)
_Avoid_: custom log, business log

## Testing

**Fast suite**:
The default nextest run — unit and feature tests, no infrastructure — plus doctests (`just test`). The component and spec suites are deliberately not in it.
_Avoid_: unit tests, default tests

**Feature test**:
A test in `tests/feature/` that drives one capability through the real router and middleware in-process, with hand-written fakes behind repository seams. No infrastructure.
_Avoid_: integration test, API test

**Component test**:
A test in `tests/component/` that runs this service against its real Postgres, Valkey and RabbitMQ, with no other services involved. Serial, because the stack is shared (`just test-component`).
_Avoid_: integration test

**Spec**:
A human-owned Gherkin file in `tests/acceptance/specs/` describing behaviour the service promises. The spec is the source of truth; code is the implementation. Changes need human approval and a `just spec-approve` lock update.
_Avoid_: test case, scenario file

**Spec suite**:
The executable spec: the Gherkin files plus their step definitions, run by the Cucumber harness (`just test-spec`, `just test-blackbox`).
_Avoid_: BDD tests, acceptance tests

**Step definition**:
The Rust function that implements one line of Gherkin. Lives in `tests/acceptance/steps/` and must not import `rust_template::`; only the driver knows the implementation.
_Avoid_: step, glue

**World**:
The per-scenario state Cucumber constructs (`tests/acceptance/world.rs`): the driver, the unique run label and the response under assertion.
_Avoid_: context, state

**In-process acceptance**:
The default spec mode: the real router over `axum-test` and the real stack, concurrent, migrations applied once (`just test-spec`).
_Avoid_: local acceptance

**Black-box acceptance**:
The spec mode that drives a deployed service over HTTP via `SPEC_TARGET`, serially and without retries, for the per-push gate (`just test-blackbox <url>`).
_Avoid_: e2e, end-to-end

**Integration test**:
A cluster end-to-end test in `tests/integration/`: this service plus other deployed services, against a staging environment. Empty in the template; ignored until a cluster exists.
_Avoid_: acceptance test, component test

## Compose profiles

**infra**: only Postgres, Valkey, RabbitMQ.
**dev**: source-mounted containers with hot reload.
**local**: builds the Dockerfile and runs it.
**publish**: runs a published image from the registry (no build).
**test**: full stack + acceptance tests, once, then exits.
**acceptance**: full stack up and idle; exec in to run all tests, one test, or debug.

**Published image**:
The multi-arch (amd64 + arm64) manifest image produced by `just docker-publish` and executed by the `publish` profile.
