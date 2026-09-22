---
pipeline_id: 9dfa2911-70ab-4d8a-96d5-2e8211d313af
ticket: docs/planning/tickets/open/TICKET-411-forge-client-fleet-brain-rip.md
status: Phase 5 — Complete PASS
title: Remove marley_forge_client + the fleet-brain forge wiring (product-rip slice 2)
type: chore
milestone: M30
references:
  - docs/planning/intake/scrap-forge-pivot.md
  - docs/planning/pipeline/completed/410-browser-forge-identity-rip.spec.md
  - docs/marley_architecture/fleet-control-plane.md
---

## Title
Second and final slice of the scrap-forge product rip: delete `marley_forge_client`
and every forge product surface — the ⌘⇧F sprint cockpit (`forge_view`,
`RightSection::Forge`, the status-bar sprint segment), the fleet-brain transport
(`FleetSubscription`, `endpoint_for_brain`, the four write paths, arm-c
"no forge client"), and the orphaned `mcp_config.rs` — while RE-HOMING the one
generic seam the crate still owns (the #406 browser-failure probe →
`marley_app/src/browser_probe.rs`, its 21 tests intact) and keeping the
forge-agnostic fleet RAIL (snapshot model + demo feed + quiet-Unconfigured). The
CONSTITUTION preamble's product words retire in an isolated amendment commit.

## Scope
### In (verified seam map at HEAD def60cf — full detail in notes discovery)
- **Probe re-home (MOVE):** `forge_client/src/probe.rs` (whole module: 6-arm
  `ProbeOutcome`, `PROBE_*_TIMEOUT_MS`, `split_origin`/`classify_io`/
  `connect_failure`/`probe_web_origin`, 8 units) → new
  `marley_app/src/browser_probe.rs`; the 13 livewire probe rows
  (`forge_client/tests/livewire.rs:1001-1218`) → new
  `marley_app/tests/browser_probe_livewire.rs`. Inline a status-line parser
  (probe.rs:177 leans on the crate's `http_status_code` — hidden dep). Repoint:
  `browser_state.rs:29` use, app.rs SpawnProbe :11031, the 3 app-linked
  instantiation rows + pump drive rows in headless_drive, `lib.rs` mod decl.
  gates.sh:233 drops the `marley_forge_client/src/adapter\.rs` exclude (+ prose
  :201-204).
- **Sprint UI (DELETE):** `forge_view.rs`; app.rs forge_client/forge_sprint/
  forge_pending/forge_open fields + boot build + pump drain + `refresh_forge` +
  `forge_ticket_row`/`claim_forge_ticket`/`comment_focused_on` + the ⌘⇧F overlay
  render + `toggle-forge` dispatch + `MARLEY_OWNER`; `OverlayStates.forge_open`
  (the 27-state inventory becomes 26 — browser.rs field/disjunct/fixtures/doc
  numbers); keymap `toggle-forge` rows (⌘⇧F stays Editor-scoped project-search);
  `RightSection::Forge` (variant/key/label/icon; `section_tabs` 3→2,
  `content.rs` cockpit `slots` 3→2, `SECTION_BROWSER_ITEMS` 4→3 with the #387
  row-0 default becoming Agents); status-bar `sprint_summary` + segment (the
  click-to-Agents affordance index SHIFTS 1→0 at app.rs:21236 — load-bearing);
  `Icon::Forge` + `icons/forge.svg` asset + registry rows.
- **Fleet transport (DELETE):** app.rs fleet_subscription/fleet_incoming/
  fleet_conn_last wiring + `FleetSubscription::start` + pump drains + test doors;
  `endpoint_for_brain` call; `fleet_live.rs` `forge_client_built` param + arm-c +
  `MisconfigArm::ForgeClientMissing`; `NO_FORGE_CLIENT_REASON` (fleet_rail :140-143
  + the byte-pin half of :798-799); the four write paths (claim/comment/answer/
  send + their app.rs guards + fail-records :8295/:8351);
  `tests/fleet_livewire.rs` (whole file, 4 tests). `demo_feed_permitted`'s
  producer input per F1/F2.
- **Crate deletion:** `crates/marley_forge_client/` (105 tests: 84 die, 21 move)
  + `examples/check_forge.rs`; Cargo edge (marley_app Cargo.toml:33; the url-dep
  comment :38-41 rewords); `mcp_config.rs` + its 7 path tests + lib.rs:72 decl +
  the app.rs:2472-2476 mcp_json read (orphan-PROVEN: its only two readers die
  here); prose sweep (marley_mcp ×6 + Cargo.toml, marley_fleet lib.rs:4,
  browser.rs :212/:386 "retires with 411" re-voice, agent_view.rs:3-4, keymap
  comments).
- **CONSTITUTION amendment (ISOLATED commit, per the amendment rule):** preamble
  lines 5-6 "+ Forge + ops" + the 11-14 "Forge MCP client" sentence + its
  409-documented exception parenthetical — all retire.
- **React POC (rip FIRST, per parity):** delete `ForgePane.tsx` +
  `overlays/ForgeOverlay.tsx`; Workspace.tsx mounts/⌘⇧F branch/state; TitleBar
  cockpit row; StatusBar sprint segment; FleetDock arm-c mock; ContextMenu
  SectionOpenForge; App.tsx forgeOpen + 'forge' tab unions; LeftRail/Icon/
  CommandPalette rows; WorkspaceHome openForge button; doc rewords. Parity docs
  (MARLEY-PARITY.md rows :592/:599/:601 + mentions, README, DEMO_FEATURES,
  STEP2-PORT-INSPECTION) swept at complete.
- **Arch docs (Phase 5):** delete `marley_forge_client.md`; sweep crate-map,
  00-overview, crate-triage, clean-build-plan, marley_agent, pane-composition-
  model, fleet-control-plane (the 411-falsified claims under its banner).

### Out (explicitly deferred)
- The fleet RAIL itself (`fleet_rail.rs` — forge-agnostic by design, 18/19 tests
  untouched), `crates/marley_fleet/` (prose-only), the future non-forge brain
  endpoint (`subscription_target`/`brain_endpoint` setting SURVIVE quiet).
- `RightSection::{Details, Agents}` and their whole cockpit machinery.
- The generic browser (410's shape) — only the probe path is touched, as a MOVE.
- Settings/grid migration — none needed: `right_section_from_key`'s catch-all
  already falls back to Details (right_dock.rs:37; grid `C=forge` same route).

## Reference (§20)
N/A — Marley-specific product removal (the scrap-forge pivot's second slice); no
reference-app behavior is matched. The surviving generic surfaces keep the
references their own specs carry; the probe move is an in-tree relocation of
Marley-authored code.

### Prior art
Swept 2026-08-09 (plan phase):
1. **Behavior maps** — no Warp/Zed analog for a sprint cockpit or fleet-brain
   transport (`docs/warp_architecture/`, `docs/zed_architecture/`: no hits on
   this seam); nothing to match for a deletion.
2. **Published material** — n/a for a rip; the probe's HTTP status-line grammar
   is RFC 9112 §4 (already embodied in the moving code's parser).
3. **Permissive deps** — the probe rides `std::net` only (TcpStream +
   read-timeout loops — already the adopted substrate; no in-tree crate owns a
   "TCP origin probe" seam better). The status-line parse to inline is ~10 lines
   over `str::split`; adding an HTTP-parser dep for it would be new supply chain
   against gate:8's lean posture — rejected. Checked gpui/ropey/regex/
   alacritty_terminal: no owner. Nothing to adopt; nothing reinvented.

## React-first (parity)
UI-AFFECTING — zones A + B (the frozen shell chrome loses the TitleBar cockpit
group's Forge tab and the status bar's sprint segment; the Browser-section ＋/
context menus lose their Forge row; the ⌘⇧F overlay and Forge cockpit pane
disappear; the FleetDock misconfig arm-c wording dies). marley-web files (port
map rows in `marley-web/docs/MARLEY-PARITY.md`): DELETE
`components/views/ForgePane.tsx` + `components/overlays/ForgeOverlay.tsx`; EDIT
`pages/Workspace.tsx`, `components/{TitleBar,StatusBar,FleetDock,ContextMenu,
LeftRail,Icon}.tsx`, `components/overlays/CommandPalette.tsx`, `App.tsx`,
`components/views/{BrowserView,WorkspaceHome,ManagerAgentSetup}.tsx` (per the
notes discovery table; design-ahead non-porting chrome — the kanban, the
manager-setup capability rows, BrowserView's simulated web half — stays). Plan
line: build & visually verify in marley-web first (`pnpm --filter
@workspace/marley-ide run dev` → localhost:5173, screenshot + READ: title bar
shows Details/Agents only, no sprint segment, ＋ menu 3 rows), then port 1:1.
Validate captures the React↔Marley parity pair on the post-rip chrome.

## Locked-In Decisions
- D1 — The probe moves VERBATIM (module doc re-voiced, `http_status_code`
  inlined) into covered, unmasked `browser_probe.rs`; its 21 tests move with it
  (8 units in-module, 13 livewire in a new integration file). The
  BF-claude-skip-detach trap rule applies to every relocation: doc + attr + fn
  stay one contiguous block.
- D2 — The fleet RAIL survives untouched in behavior: snapshot model, demo feed,
  chip tones, dispatch/answer state machines. Only the transport + arm-c die.
- D3 — Tolerant boot needs NO migration: stale `right_section = "forge"` /
  `C=forge` fall back to Details via the existing `right_section_from_key`
  catch-all (right_dock.rs:37, the #95 pattern) — pinned by a test at validate.
- D4 — The #387 Browser-section ＋-menu row-0 default becomes **Agents** (the
  next cockpit row up); `SECTION_BROWSER_ITEMS` 4→3, `section_tabs` 3→2,
  cockpit `slots` 3→2 — hard arities shrink, exhaustive matches stay total.
- D5 — The status-bar agent segment's click-to-Agents affordance moves with its
  index (1→0, app.rs:21236) — the four `cockpit_status_*` tests re-pin the new
  ordering.
- D6 — The CONSTITUTION amendment is its own isolated commit (only
  CONSTITUTION.md), landed BEFORE the code commit, per the amendment rule.
- D7 — React-first ordering: the POC rip lands + is visually verified before the
  Rust rip (M29 #403 lesson; the misclick lesson says drive menus by keyboard).

Design forks for Phase 2 (named, not locked):
- F1 — `ConnectionState`'s post-411 home: re-home a 2-variant enum into
  `fleet_live.rs` (keeps `fleet_header_title`'s Live/Reconnecting arms + their 6
  tests compilable for a future non-forge brain) vs collapse the header to the
  no-transport arm (leaner; the future feature re-adds state). Interacts with
  `demo_feed_permitted`'s `subscription_active` input (t376_req007).
- F2 — The dispatch/answer machines' no-transport terminal: with the write
  quartet gone, does confirm land a typed "no transport" Failed record (machines
  + drives REWORD; rail stays interactive-in-demo) or do the verbs disable
  entirely (machines simplify; drives DELETE)? The two headless drives
  (:10230/:10347) + app.rs :8281-8351 follow the pick.
- F3 — `browser_boot_ignores_mcp_json_headless`'s fate once `mcp_config` dies
  with the boot read: REWORD (keep as the standing negative pin — a `.mcp.json`
  on disk feeds NOTHING anywhere) vs DELETE (its premise evaporated). Leans
  REWORD: the pin outlives the module it guarded against.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The workspace shall contain no `crates/marley_forge_client` and no reference to it (code, Cargo, gates.sh regex) | `test ! -d crates/marley_forge_client`; `grep -rn 'marley_forge_client' crates/ scripts/ Cargo.*` = 0; build green |
| REQ-002 | The browser-failure probe shall live at `marley_app::browser_probe` with all 6 `ProbeOutcome` arms, both timeouts, and its 21 moved tests RUN green (8 unit + 13 livewire) | `cargo nextest run -p marley -E 'test(browser_probe)'` + the livewire file; `classify_probe` arms compile against the new path |
| REQ-003 | WHEN a settings file or persisted grid carries stale `forge` keys (`right_section = "forge"`, `C=forge`), the app shall boot with the Details fallback and no error | tolerant-restore unit/drive RUN green (D3 pin) |
| REQ-004 | The fleet rail shall classify setups without a ForgeClientMissing arm and render Unconfigured/demo cleanly; the four forge write paths shall be gone | `classify_fleet_setup` reworked units; fleet_rail :798 repinned; grep `NO_FORGE_CLIENT_REASON` = 0 |
| REQ-005 | The status bar shall render the agent segment at index 0 WITH its click-to-Agents affordance | repointed `cockpit_status_*` units + the index-0 click arm; headless/live check |
| REQ-006 | ⌘⇧F shall be bound only in the Editor scope (project-search); `toggle-forge` shall not exist | keymap units repointed; grep `toggle-forge` = 0 |
| REQ-007 | Live product code and the CONSTITUTION preamble shall carry zero forge product references (history/knowledge/completed-pipeline docs exempt) | grep sweep over `crates/ CONSTITUTION.md` (false-positive list from 410 respected); the amendment commit is isolated |
| REQ-008 | The React POC shall carry the same post-rip chrome (no Forge tab/overlay/pane/segment/menu row), visually verified, parity pair captured | POC typecheck + screenshots READ; MARLEY-PARITY.md rows updated at complete |
| REQ-009 | The full diff shall pass the delivery gate | `scripts/gates.sh --diff` exit 0 (receipt) |

## Phase Plan
- **P2 Design** — resolve F1/F2/F3; file manifest (React + Rust halves, the
  amendment commit split); regression test plan incl. the moved-probe lanes and
  the PR-shared-copy-closure shape; risks.
- **P3 Implement** — POC rip + visual verify FIRST; then the amendment commit;
  then the Rust rip per manifest.
- **P3.5 Inspect** — critics: correctness (arity shrinks, index shift, exhaustive
  matches), state-integrity (tolerant boot, fleet rail survival), provenance/
  secrets (bearer plumbing dies clean), simplification.
- **P4 Validate** — write/repoint planned tests; RUN full suites; live drive +
  parity pair; gate `--diff` green.
- **P5 Complete** — CHANGELOG; arch docs (delete marley_forge_client.md + the
  7-doc sweep); parity docs; ledger capture; archive; close the ticket.
