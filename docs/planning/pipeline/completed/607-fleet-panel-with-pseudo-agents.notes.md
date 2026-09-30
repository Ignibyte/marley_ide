# The fleet contract's types, a pseudo provider, and the Fleet panel's list — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-607-fleet-panel-with-pseudo-agents.md
- **Pipeline spec:** 607-fleet-panel-with-pseudo-agents.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones", after the fleet contract (docs/marley/fleet-contract.md, D20) was settled with him.
- **Batch:** wave 1 of `docs/planning/design-notes/remaining-work-2026-09-30.md`.
- **Recall (§18.3):**
  - AD-claude-456 and F-claude-456: the Marley layout moves the Agent Panel between docks and restores what each dock showed; a panel sharing the right dock is subject to it.
  - Dock activation priorities must be unique per dock (dock.rs:784-796); 0 to 7 are in use.
  - marley_fleet: the same six states, serde conventions to copy, no run/phase/gate/event/usage/host types.
  - Brain: the contract decisions are D20 and docs/marley/fleet-contract.md's Settled list.
- **Discovery:** one Explore sweep for the wave (2026-09-30): dock panels, center items, gpui
  drawing, marley_fleet, the MCP and HTTP clients, SSH seams, settings, and the Warp, Orca and Zed
  notes. The spec's Prior art cites what applies here; the Plan phase re-verifies each seam at
  promotion.

### Promotion (Opus, 2026-09-30)
- **Pre-flight:** no active pipeline, cargo idle, README marker present; the branch level with its
  origin.
- **Recall:** as queued above. Brain: nothing on this seam (the consultation is closed at
  Complete).
- **Seams re-verified:**
  - `workspace::Panel`'s required methods (`dock.rs:36-104`).
  - Priorities in use: 0, 1, 2, 3, 5, 6, 7, 200. The Fleet panel takes 20.
  - `Workspace::add_panel` (`workspace.rs:2888`).
  - The workbench's per-workspace hook (`cx.observe_new::<Workspace>`, `marley_workbench.rs:606`).
  - `marley_fleet`'s manifest and lint table to copy.
  - The gates find Marley crates by `crates/marley_*` (`gates.sh:99, 188`), so a new crate needs
    no list edit.
  - The `Cargo.toml` and `settings_content/src/marley.rs` touchpoint rows (47, 57) and
    `default.json`'s (64).

### Design
- **Approach.**
  - *`marley_sdk`* (pure: `serde`, `serde_json`; the Marley lint table):
    - `work` (`Handshake`, `AgentList`, `AgentSummary`, `AgentDetail`, `AgentInfo`, `WorkItem`,
      `PhaseSummary`, `Run`, `PhaseRun`, `PhaseState`, `Gate`, `GateState`, `Event`, `Usage`,
      `Tokens`, `Question`, `Changes`, `Attention`, `State`);
    - `host` (`HostSnapshot`, `HostInfo`, `Memory`, `Disk`, `Network`, `AgentProcess`);
    - `stale::is_stale(last_seen_ms, now_ms, poll_s, stale_after_s)`;
    - `pseudo`: fixtures under `crates/marley_sdk/fixtures/*.json`, `include_str!`'d, and
      `Pseudo::at(now_ms) -> (AgentList, Vec<AgentDetail>, Vec<HostSnapshot>)`, moved by time.
      CPU from a slow sine per host, an event every 8 s, a phase advanced each 60 s cycle, and
      the third agent's `last_seen_ms` stopping 20 s after the start.
    - Serde follows `marley_fleet`: `rename_all = "lowercase"` on the enums, `default` on
      optional fields, no `deny_unknown_fields`.
  - *Settings:* `MarleySettingsContent::fleet: Option<MarleyFleetSettingsContent { providers:
    Option<Vec<FleetProviderContent>> }>`, `FleetProviderContent` tagged by `kind` (`pseudo` for
    now; #610 and #611 add theirs). `MarleySettings::fleet_providers`. `default.json` gets
    `"fleet": { "providers": [] }`.
  - *`fleet.rs`* (workbench):
    - `Fleet`, a global entity: its providers from the settings, the last list and details per
      provider, a watcher count, and a poll task every `poll_s` while a panel shows (as `Ports`
      does, F-claude-521). It notifies only on change.
    - `FleetPanel`, a `Panel` on the right (valid only on the right), persistent name
      `MarleyFleetPanel`, icon `IconName::Server`, toggle action `marley::ToggleFleet`, priority 20.
      It observes the `Fleet` and renders host headers with agent rows: runtime icon, name, state
      chip (stale from `is_stale`), work item key, phase `n/m`, attention mark. With no provider
      it shows the not-set-up message.
    - Each workspace gets its panel in the workbench's `observe_new` hook.
- **File manifest:**
  - `crates/marley_sdk/{Cargo.toml, src/marley_sdk.rs, src/work.rs, src/host.rs, src/stale.rs,
    src/pseudo.rs, fixtures/*.json}` (Marley, new crate);
  - `Cargo.toml` (workspace member and dependency; Zed file, row 47 updated);
  - `crates/marley_workbench/{Cargo.toml, src/fleet.rs, src/marley_workbench.rs}` (Marley);
  - `crates/settings_content/src/marley.rs` (Marley file in a Zed crate, row 57 updated);
  - `assets/settings/default.json` (Zed file, row 64 updated);
  - `script/e2e/607-fleet-panel-with-pseudo-agents.sh` (Test).
- **Visual check plan:**
  - The scenario writes the pseudo provider into the run's settings and toggles the panel from
    the palette. Shots: `list.png` (REQ-001), `moved.png` after 25 s (REQ-002: an event or
    phase moved, and the third agent stale), and `not-set-up.png` with the setting removed
    through the settings file (REQ-003).
  - REQ-004 and REQ-005 are checked by review; REQ-005 is also watched in the scenario by
    switching layouts once.
- **Risks:**
  - Adding a panel from `observe_new` before Zed's own panels load: if the dock's saved state
    races it, add it after the workspace's first frame (`defer_in`).
  - One `Fleet` for every window: panels observe it and never update it inside their own
    render.

## Phase 2 — Code (2026-09-30)
- **Built:**
  - `marley_sdk`, a new pure crate:
    - `work` and `host`, the `v1` types. Enums fall back to `Unknown` on a value this version
      does not name (`serde(other)`), and optional fields take `default`.
    - `stale::is_stale`.
    - `pseudo::Pseudo`: the fixtures `handshake.json`, `details.json` and `hosts.json`, every
      `_ms` rebased to the reading's moment by a walk over the JSON, then moved:
      - the working agent's phase a minute and an event every 8 s, with its tokens;
      - the hosts' processor and network by an integer wobble table, so nothing casts a float;
      - the third agent's `last_seen_ms` stopping 12 s after the start.
  - Settings: `MarleyFleetSettingsContent { providers }` and `FleetProviderContent::Pseudo` in
    `settings_content/src/marley.rs`, `MarleySettings::fleet_providers`, and
    `"fleet": { "providers": [] }` in `default.json`. The touchpoint rows (Cargo.toml,
    settings_content, default.json) were written first.
  - `fleet.rs`:
    - the `Fleet` global, polled by one task while a panel is active (`watch` and `unwatch`
      from `Panel::set_active` and the panel's release), written only on change;
    - `FleetPanel` (right dock only, priority 20, `IconName::Server`, `marley: toggle fleet`);
    - the not-set-up message, and each source's header with its failure.
  - Each workspace gets its panel in `fleet::init`'s `observe_new`.
- **Deviations:**
  - `TokenUse`'s fields are `input`, `output` and `cache_read` in Rust, with `serde(rename)` to
    the contract's `input_tokens`, `output_tokens` and `cache_read_tokens`: clippy's
    `struct_field_names` refused the shared postfix, and the wire names are fixed.
  - The pseudo handshake polls every 2 s, so the demo moves; the third agent reads stale after
    about 18 s.
- **Review of the diff:**
  - REQ-001: the list, grouped by host, the chips, keys and phases, the attention marks.
  - REQ-002: `read_providers` with the stale rule measured from the reading's time.
  - REQ-003: the not-set-up message.
  - REQ-004: no `deny_unknown_fields`, `default` everywhere optional, `serde(other)` on enums.
  - REQ-005: the panel is valid only on the right, so the layout's move of the Agent Panel
    leaves it.
  - A failure of the fixtures is shown as the source's failure, never a panic.
  - Nothing reads or updates the global inside another update: the poll runs in its own task,
    and the panel only observes.
- **Gate:** `just gate-diff` 17 PASS, 0 FAIL (`GATE GREEN [diff]`).

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/607-fleet-panel-with-pseudo-agents.sh` under sway, run with
  `just e2e`. Its settings name `{ "kind": "pseudo" }`, the run trusts the scratch project, and
  `marley: toggle fleet` opens the panel.
- **Shots, each read:**
  - `list.png` (REQ-001): FLEET, the header "Pseudo provider", the hosts build-1 and vps-2 with
    the server icon. Under build-1: build-1 with the Claude mark, "RB-142 · code 2/4" and a blue
    `working` chip; review-1 with the OpenAI mark, "RB-139 · test 3/4", the warning mark and a
    yellow `waiting` chip. Under vps-2: docs-1, "RB-151 · code 2/4", the failed mark and a red
    `error` chip. Cropped at 2x to read the small text.
  - `moved.png` (REQ-002), 62 s on: build-1 reads "test 3/4", so the run moved a phase; docs-1's
    chip reads `stale`, with the failed mark kept.
  - `not-set-up.png` (REQ-003): with `providers` set to `[ ]`, the panel says "The fleet is not
    set up." and names `marley.fleet.providers` and `{ "kind": "pseudo" }`.
  - `layout.png` (REQ-005 and REQ-002 again): with the provider back, a round trip through Zed's
    layout and the toggle, the panel is in the right dock and build-1 reads "complete 4/4": the
    reads carried on after the round trip.
- **Reds found and fixed:**
  - The first runs ended with the panel showing a stale list after the layout round trip. Two
    causes, one ours:
    - Ours: polling ran while a panel had said `set_active(true)` and not `false`. Zed's docks
      call `set_active` for a panel activated in a closed dock too, and a layout switch moves
      panels between docks, so that count does not say whether a panel shows. `fleet.rs` now
      asks the docks: the loop runs while a Fleet panel is the visible panel of an open right
      dock in some window's shown workspace, checked at each read, and a panel that draws starts
      the loop with a deferred call when it is not running. `set_active` and the watcher count
      are gone.
    - Not ours: after the fix, a probe logged every read and showed the loop running with the
      provider count still 1 after the file said `[ ]`. A second probe changed `ui_font_size`
      before and after a layout round trip: the first edit applied and the second did not.
      A hand edit to settings.json after Marley has written the file (here the layout switch's
      write) does not reload. Pre-existing, not in scope: TICKET-612. The scenario now edits
      the settings before the round trip.
  - The probes' log lines were removed before the last gate.
- **Gate after the fix:** `just gate-diff` 17 PASS, 0 FAIL.
- **Focus:** the scenario never touched the user's session: sway stopped with the run's Marley,
  and Hyprland had no Marley windows before or after.

## Phase 4 — Complete (2026-09-30)
- **Documented:**
  - `CHANGELOG.md` (Added: the Fleet panel, on a pseudo provider);
  - `docs/marley/three-prong-plan.md` (D20: wave 1, #607 shipped) and
    `docs/marley/fleet-contract.md` (the pseudo data now lives in `marley_sdk`);
  - `docs/marley_architecture/marley_sdk.md` (new), `marley_workbench.md` (the Fleet panel
    section), and the docs map `docs/marley/README.md`;
  - the guide (`docs/marley/guide.md`, a Fleet panel section and its setting), the guide page
    (`crates/marley_workbench/guide/index.html`: the fleet section, the command and the setting)
    and the walkthrough (Part 12, and the command in Appendix B);
  - `docs/marley/zed-touchpoints.md`: the Cargo.toml, `settings_content/src/marley.rs` and
    `default.json` rows describe what shipped.
- **Knowledge:** F-claude-607-polling-followed-set-active-not-what-shows-001,
  PR-claude-607-ask-the-dock-whether-a-panel-shows-001,
  L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001,
  AD-claude-607-the-fleet-contract-lives-in-marley-sdk-001.
- **Brain:** consultation f4b5edf70283414cad65254ab15f9930 closed with `brain decide`
  (`decisions/marleys-fleet-contract-lives-in-marley-sdk-drawn-by-a-right-dock-fleet-panel`,
  follow-up by 2026-10-14).
- **Follow-up minted:** TICKET-612, the settings reload after Marley's own write (bug, queued).

