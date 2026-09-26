---
pipeline_id: 2c315284-850d-4755-8a08-32e253780624
ticket: docs/planning/tickets/closed/TICKET-514-telemetry-off-by-default.md
status: Phase 4 — Complete PASS
title: "Telemetry off by default"
type: chore
slice: privacy, cross-cutting (the Orca survey's open question 1)
references: [docs/orca_architecture/README.md, docs/orca_architecture/07-engineering-and-changelog.md]
---

## Title
Marley's defaults turn Zed's telemetry off: `telemetry.metrics` and `telemetry.diagnostics` are
false in `assets/settings/default.json`, so no usage event is queued, logged or posted unless the
user turns one on.

## Scope
### In
- `assets/settings/default.json`: `"diagnostics": false` and `"metrics": false` in the
  `telemetry` block, with a `Marley:` comment. The defaults file is read before `telemetry.start`
  in `crates/zed/src/main.rs`, so even the first event of a launch ("App Opened") is never queued;
  a runtime `update_default_settings` from `marley_workbench::init` would come after it.
- The row for `assets/settings/default.json` in `docs/marley/zed-touchpoints.md`.

### Out (explicitly deferred)
- The Marley settings page (#515), which shows these two toggles among Marley's own settings.
  Zed's Settings window already has them, on its General page under Privacy.
- Other requests to zed.dev that are not telemetry: sign-in, collaboration, the extension
  registry, Zed's hosted AI and edit predictions. They happen only when the user uses those
  features; each is its own setting in Zed.

## Reference (§20)
Upstream Zed (`client::telemetry`): `report_event` returns early when `settings.metrics` is false,
so nothing is queued, written to `telemetry.log` or posted; `diagnostics` gates crash and hang
reports. Marley keeps Zed's behavior and flips the defaults. Warp: N/A. Orca, for comparison,
keeps telemetry on by default with an opt-out (report 07 §A11).

### Prior art
- **Code we already ship.** `crates/client/src/telemetry.rs`: `report_event` (the `metrics` check),
  `flush_events_inner` (writes `telemetry.log`, then posts to `build_zed_api_url("/telemetry/events")`),
  `FLUSH_INTERVAL` (one second in a debug build, five minutes in release); the settings UI's
  Privacy items in `crates/settings_ui/src/page_data.rs` (`telemetry.diagnostics`,
  `telemetry.metrics`), which read the merged value and so show the new default.
- **Reports.** docs/orca_architecture/README.md (the correction that Marley does send telemetry),
  report 07 §A11.

## UI proof
UI-AFFECTING (a setting's default, visible in the Settings window).
`script/e2e/514-telemetry-off-by-default.sh` (`compositor sway`): the profile copy's settings get
`"server_url": "http://127.0.0.1:9"` so nothing can leave the box in any part of the run. Part one,
no telemetry setting: Marley opens, the palette and a panel are used, Marley quits; the profile's
`logs/telemetry.log` holds no event (run log). The Settings window's General page shows both
toggles off (`514-01-privacy-off`). Part two: `telemetry.metrics: true` added, Marley relaunched and
used; `telemetry.log` now holds events (run log), and the posts fail against the dead port.

## Locked-In Decisions
- D1 — The defaults file, not a runtime default: it is the only place that is in force before
  the first event of a launch.
- D2 — Both flags off, not only metrics: diagnostics sends crash and hang reports, which carry
  more about the machine than usage events do.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN no telemetry setting is set, Marley shall queue, log and send no telemetry event. | Part one's `telemetry.log`, empty |
| REQ-002 | WHEN no telemetry setting is set, the Settings window shall show Telemetry Metrics and Telemetry Diagnostics off. | Shot `514-01-privacy-off` |
| REQ-003 | WHEN `telemetry.metrics` is set true, Marley shall record telemetry events again. | Part two's `telemetry.log`, with events |

## Phase Plan
- **P1 Plan** — this spec.
- **P2 Code** — the ledger row, the two defaults.
- **P3 Test** — the scenario, the shot, the logs; the gate.
- **P4 Complete** — docs, knowledge, close, archive, commit.
