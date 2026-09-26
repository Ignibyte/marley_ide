# Telemetry off by default — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-514-telemetry-off-by-default.md
- **Pipeline spec:** 514-telemetry-off-by-default.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: "Lets have it off by default and configurable. WE need a marley
  settings pane" (the pane is #515).
- **Classification / tier:** chore; one Zed asset file.
- **Recall (§18.3):** nothing in the ledgers on telemetry. The survey's report 07 §A11 found the
  defaults on and the settings on the dev box silent; this session read `report_event` and
  `flush_events_inner` and confirmed events are posted to `api.zed.dev` when metrics are on.
  `telemetry.log` in Marley's data directory holds such events from 2026-09-25 (App Closed, Hang
  Incidents, Panel Button Clicked).
- **Discovery:** `assets/settings/default.json` lines 1665 to 1672 (the `telemetry` block);
  `crates/client/src/telemetry.rs` (`report_event`, `flush_events_inner`, `FLUSH_INTERVAL`);
  `crates/zed/src/main.rs` (`telemetry.start` runs before `marley_workbench::init`);
  `crates/settings_ui/src/page_data.rs` (`privacy_section`).
- **Checklist (no task tool in this session):** pick · pre-flight · recall · mint · prior art ·
  spec · design — done.

### Design
- Two values and a comment in `assets/settings/default.json`; the ledger row first. JSON with
  comments is Zed's format for the file, so a `// Marley:` line sits inside the block.
- **E2E plan:**

| REQ | Scenario part | Proof |
|---|---|---|
| REQ-001 | setup: `server_url` to `http://127.0.0.1:9` in the profile copy; steps: open the palette, open and close the project panel, quit | `telemetry.log` absent or empty (run log) |
| REQ-002 | relaunch, `zed: open settings`, the General page's Privacy section | `514-01-privacy-off` |
| REQ-003 | quit, add `telemetry.metrics: true`, relaunch, open the palette, quit | `telemetry.log` with events (run log) |

- **Risks:** none beyond the upstream merge, where the two values are re-applied.

## Phase 2 — Code
- **Built.** The ledger row for `assets/settings/default.json`, then `"diagnostics": false` and
  `"metrics": false` in its `telemetry` block under a `// Marley:` line. No Rust.
- **Review.** The settings UI's Privacy toggles read the merged value, so they show off; a user
  setting of either key wins over the default, which is the "configurable" half. Nothing else in
  the tree reads the two defaults.

## Phase 3 — Test
- **Scenario:** `script/e2e/514-telemetry-off-by-default.sh` (`compositor sway`), on the rebuilt
  debug build; the profile copy's `server_url` points at `http://127.0.0.1:9` for the whole run.
- **Part one** (no telemetry setting): after the launch, the trust prompt and the palette,
  "telemetry events recorded: 0". REQ-001.
- **Shot `514-01-privacy-off`, read:** the Settings window (tiled beside the main window in the
  sway), searched for "telemetry": General, Privacy, "Telemetry Diagnostics" and "Telemetry Metrics",
  both toggles off. REQ-002.
- **Part two** (`telemetry.metrics: true`, relaunched): "telemetry events recorded: 4 (App Opened,
  Settings Changed, Sidebar Toggled)". REQ-003; the posts went to the dead port.
- **Focus:** sway; nothing on Hyprland.
- **Gate:** `just gate-fast` (no Rust): `GATE GREEN [fast]`.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Changed: telemetry is off by default); the ledger row for
  `assets/settings/default.json` describes what shipped.
- **Knowledge:** nothing new to record; the decision is Chad's and is in the brain below.
- **Brain:** no consultation opened for this chore; the decision is recorded directly.
- **Ticket:** closed; the pair archived.
