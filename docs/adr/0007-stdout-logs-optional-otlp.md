# Logs on stdout; OTLP optional and env-gated

Logging is JSON on stdout only — identical in every environment, with no in-app shipping; whatever listens (a collector, `kubectl logs`, anything later) stays external and swappable. `REQUEST_LOG_LEVEL` and `APP_LOG_LEVEL` are independent per-layer filters (renamed from `CUSTOM_LOG_LEVEL`); sensitive headers are never logged.

Distributed traces and RED metrics cannot ride text logs, so an OTLP exporter activates only when `OTEL_EXPORTER_OTLP_ENDPOINT` is set — unset means nothing leaves the process. OTel Rust is 0.x with traces in Beta; versions are pinned exactly and bumped deliberately.
