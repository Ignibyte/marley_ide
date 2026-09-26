# TICKET-514 — Telemetry off by default

- **Ticket:** LOCAL #514 (chore, privacy, cross-cutting)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/514-telemetry-off-by-default.spec.md
- **Source ticket:** Chad, 2026-09-25, on the Orca survey's first question (Zed's telemetry is on and posted to `api.zed.dev`): "Lets have it off by default and configurable." (docs/orca_architecture/README.md, open question 1; report 07 §A11)
- **Status:** closed

## Summary
Marley inherits Zed's telemetry defaults, `telemetry.metrics` and `telemetry.diagnostics` both
true, and the settings on the dev box do not override them, so Marley queues usage events and
posts them to `api.zed.dev/telemetry/events`. Both default to false in Marley. They stay
settings, so turning either on in the settings file or the Settings window works as before.

## Acceptance
With no telemetry setting, Marley records and sends no telemetry event; with `telemetry.metrics`
set true, it records them again.
