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
The marked example code (notes routes, seed tooling) whose only purpose is to materialize the template's patterns; removed when building a real service.
_Avoid_: sample, boilerplate, scaffold

## Logging

**Request log**:
Per-request HTTP log line; verbosity controlled by `REQUEST_LOG_LEVEL`.

**App log**:
All non-request logs — business logic, lifecycle, infrastructure; verbosity controlled by `APP_LOG_LEVEL`. (Replaces the Fastify template's `CUSTOM_LOG_LEVEL`.)
_Avoid_: custom log, business log

## Compose profiles

**infra**: only Postgres, Valkey, RabbitMQ.
**dev**: source-mounted containers with hot reload.
**local**: builds the Dockerfile and runs it.
**publish**: runs a published image from the registry (no build).
**test**: full stack + acceptance tests, once, then exits.
**acceptance**: full stack up and idle; exec in to run all tests, one test, or debug.

**Published image**:
The multi-arch (amd64 + arm64) manifest image produced by `just docker-publish` and executed by the `publish` profile.
