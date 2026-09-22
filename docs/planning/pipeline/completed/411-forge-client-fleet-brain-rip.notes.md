# 411 — Forge client + fleet-brain rip — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-411-forge-client-fleet-brain-rip.md
- **Pipeline spec:** 411-forge-client-fleet-brain-rip.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** `/work 411` (auto-approved through commit per session goal). Product-rip
  slice 2, runs after 410 (landed at def60cf). Intake: scrap-forge-pivot.md.
- **Classification / tier:** chore, M30, work pipeline, one shippable slice (+ the
  isolated CONSTITUTION amendment commit).
- **Recall (§18.3):**
  - PR-claude-shared-copy-closure-for-a-call-counted-probe-test-001 — probe-test moves
    must keep the shared-`Copy`-closure shape or gate:4 reds on a missed closure fn.
  - BF-claude-skip-detach-pump-fleet-live-001 — the INSERT-BETWEEN-attr-and-fn detach
    trap (5 strikes); every relocation keeps doc+attr+fn contiguous.
  - BF-373/BF-375 — the FleetSubscription reconnect/session semantics being deleted;
    the rail's promise shifts to quiet-Unconfigured (no transport at all).
  - From 410: inherited items (mcp_config deletion now orphan-proven; the
    OpenCockpit(Forge) vocabulary), the temporary same_web_origin duplicate (forge_client
    copy dies with the crate), T8 bearer property retires with the crate,
    L-claude-drive-gpui-menus-by-keyboard (the live-drive protocol),
    L-claude-rip-keeps-pure-seams-called-constant-input-001 (pattern precedent).
- **Discovery (Explore sweep vs HEAD def60cf — 12 ticket corrections):**
  1. Probe move set = **21 tests** (8 probe.rs units + **13** livewire rows :1001-1218),
     not 20; destination pair: `src/browser_probe.rs` + `tests/browser_probe_livewire.rs`.
  2. Hidden dep: probe.rs:177 → `lib.rs:347 http_status_code` — inline a status-line
     parser in browser_probe.rs.
  3. mcp_config.rs = **7** units (not 9); ORPHAN-PROVEN: mcp_json read app.rs:2472-2476,
     readers :2477-2480 (forge_client) + :2502 (endpoint_for_brain) — both die here;
     `marley_core::marley_mcp_config_file_path` is the unrelated survivor.
  4. NOT in ticket: gates.sh:233 adapter.rs coverage-exclude + prose :201-204 must go.
  5. NOT in ticket: `OverlayStates.forge_open` (browser.rs:77-78/:139/:280/:312 +
     app.rs:10835) — the 27-state inventory becomes **26**.
  6. NOT in ticket: app.rs:679 `MARLEY_OWNER` orphans (sole user claim_forge_ticket).
  7. NOT in ticket: the status-bar **index shift** — app.rs:21236 `if i == 1` →
     `i == 0` or the click-to-Agents affordance misses; status_bar :123 segment vec;
     4 cockpit_status_* tests re-pin.
  8. Hard arities: content.rs:275 `slots: [_; 3]`→2; context_menu.rs:163 `[_; 4]`→3;
     right_dock section_tabs 3→2; #387 row-0 default → Agents (test :535 repins).
  9. `ConnectionState` orphan risk → fork F1 (fleet_live:136-155 header arms + 6 tests).
  10. tests/fleet_livewire.rs: ALL 4 die (every one drives FleetSubscription).
  11. Headless rows = **9 fns** (not 14): 1 DELETE (arm-c wire drive :10065),
      2 DELETE-or-REWORD (question/dispatch machines — fork F2), 2 REWORD (C=forge
      cockpit restore :11312; browser_boot_ignores_mcp_json :11685 — fork F3),
      4 REPOINT (pump probe rows + 3 instantiation rows :12126-12172, banner re-voice).
  12. Crate ledger: 105 tests total = 84 die + 21 move; marley_app: 19 delete
      (forge_view 6, status_bar 1, fleet_live 1, fleet_livewire 4, mcp_config 7),
      ~26 reword/repoint (status_bar 4, right_dock 5, icons 1, content 2,
      context_menu 2, tabs 1, keymap 2, fleet_live 6, fleet_rail 1, browser 1,
      app 1); browser_state 19 survive (use-path repoint only).
  - Settings/grid tolerance CONFIRMED (right_section_from_key catch-all → Details;
    grid C=forge same route; grid_layout:476 documents the #95 pattern). Key:
    `"cockpit.right_section"` settings.rs:136-138; write :592-597; read :452.
  - Sprint UI kill map (current lines): forge_view.rs whole; app.rs :31-34 imports,
    :406-414 fields, :1823-1828 pump, :2477-2480 build, :2507 classify feed,
    :2669-2672 init, :4124-4137 refresh, :6142-6160 dock arm, :8850-8931 row/claim/
    comment, :10012-10018 dispatch, :20703-20733 overlay, :21223-21231 sprint render,
    :21917 icon test, :749 asset row.
  - Fleet transport kill map: app.rs :424/:427/:430 fields, :2486-2532 boot wiring,
    :2675-2677 init, :2821-2828 test doors, :8385-8453 pump drains, :10189 demo gate;
    fleet_live :10/:69-80/:102/:120-127/:136-155; fleet_rail :140-143/:798-799;
    write quartet adapter.rs:62/71/84/103 ↔ app.rs guards :8290/:8345/:8895/:8917 +
    fail records :8295/:8351.
  - CONSTITUTION preamble: lines 5-6 "+ Forge + ops" + 11-14 client sentence + the
    409 exception parenthetical.
  - marley_mcp prose ×6 (lib.rs:5/:51, transport.rs:3/:390, jsonrpc.rs:2, auth.rs:6)
    + Cargo.toml:8; marley_fleet lib.rs:4.
  - React POC: full DELETE/REWORD table in the sweep (ForgePane + ForgeOverlay whole;
    Workspace/TitleBar/StatusBar/FleetDock/ContextMenu/App/LeftRail/Icon/
    CommandPalette/BrowserView edits; WorkspaceHome openForge; design-ahead kanban/
    manager-setup/simulated-web stays). Parity docs sweep list captured for P5.
  - Arch docs: marley_forge_client.md DELETE; crate-map/00-overview/crate-triage/
    clean-build-plan/marley_agent/pane-composition-model/fleet-control-plane line
    lists captured for P5 (sweep by grep at close).
- **Decisions:** D1-D7 locked in spec; forks F1 (ConnectionState home), F2
  (dispatch/answer no-transport terminal), F3 (mcp_json negative-pin fate) → Phase 2.

## Phase 2 — Design

### Fork resolutions
- **F1 — `ConnectionState` re-homes into `fleet_live.rs` (2-variant Copy enum,
  Live/Reconnecting); the header survives whole; the classifier reshapes to
  (target, config_dir).** The header's live/reconnecting suffixes are GENERIC
  transport-display vocabulary (any future brain reuses them) — kept total +
  tested, called with `None` (the 410 constant-input pattern; future-seam
  comment). `classify_fleet_setup(target, config_dir_available)` drops BOTH the
  forge param/arm-c AND `EndpointRejected` (its constructor `endpoint_for_brain`
  dies; a future brain feature re-adds its own validation arm) —
  `MisconfigArm::ConfigDirUnavailable` remains the one (structurally-unreachable,
  documented) misconfig arm. The charter's "quiet-Unconfigured" falls out of the
  EXISTING contract: a configured target classifies `Configured`, and
  `(Configured, None)` already renders bare `"Fleet"` (#376's no-subscription
  line) — no lying reason, no new state. `demo_feed_permitted` stays called with
  constant `false` (no subscription can exist) + future-seam comment; t376_req007
  keeps its producer.
- **F2 — the dispatch/answer machines SURVIVE with a typed no-transport
  terminal.** New shared const `fleet_rail::NO_TRANSPORT_REASON = "no fleet
  transport"` replaces `NO_FORGE_CLIENT_REASON` (same shared-literal discipline:
  header/card/records tell one story); app.rs answer/dispatch confirm arms drop
  the client guard + write call and land the Failed record with the new reason;
  the :798 byte-pin repins; both headless machine drives REWORD their premise
  (was "no forge client → immediate Failed", now "no transport → immediate
  Failed" — same assertions).
- **F3 — `browser_boot_ignores_mcp_json_headless` REWORDS** into the standing
  tombstone: with `mcp_config` gone nothing anywhere reads `.mcp.json` — the
  drive's fixture-then-assert-nothing-derives pin now guards against ANY future
  re-wiring, not just the browser's. Comment re-voiced only.

### Architecture notes
- `browser_probe.rs` = probe.rs verbatim (module doc re-voiced: drop the
  adapter.rs-masking references; the unmasked/in-denominator posture STAYS) +
  a private `status_line_code(&str) -> Option<u16>` inlined from the crate's
  `http_status_code` shape + a direct arms unit (its old exercisers die with the
  crate; MSI needs a local killer). BF-skip-detach rule: every moved block stays
  doc+attr+fn contiguous; probe.rs carries zero mutants::skip — keep it that way.
- `tests/browser_probe_livewire.rs` = livewire.rs:1001-1218 verbatim (banner
  re-voiced, its own serve_once fixture travels; imports repoint to
  `marley_app::browser_probe` — requires `ProbeOutcome`/`probe_web_origin` to be
  `pub` in the new module and re-exported from lib.rs — YES: pub mod, since an
  integration test links the lib crate).
- Deletion order at implement: React POC first (visual verify) → CONSTITUTION
  amendment (ISOLATED commit, no .rs — no receipt needed) → Rust rip (one
  changeset: crate dir rm, app-side edits, gates.sh exclude removal — gates.sh
  is receipt-fingerprinted, so the P4 gate run post-edit mints the valid
  receipt).
- Status bar: `cockpit_status(agents)` (sprint param dies), segments =
  `[agent_summary(agents)]`; app.rs:21236 click arm `i == 1` → `i == 0`.
- Overlay inventory 27 → 26: `OverlayStates.forge_open` field + disjunct +
  builder line die; the flip-table test renames
  (`overlay_flip_table_each_of_26_states`) and drops the row; browser.rs doc
  numbers re-count.

### File manifest
**React half (FIRST, visual verify at 5173):** DELETE ForgePane.tsx,
overlays/ForgeOverlay.tsx; EDIT Workspace.tsx (import/mount/Esc/⌘⇧F-collapse/
dep-array/unions/comment), TitleBar.tsx (COCKPIT_TABS row + docs + union),
StatusBar.tsx (sprint segment + showSprint + 'no sprint' span die), FleetDock.tsx
(arm-c mock + doc), ContextMenu.tsx (SectionOpenForge + row + docs),
CommandPalette.tsx ('Open Forge' row + case + cycle 'client' step + union),
App.tsx (forgeOpen + 'forge' unions), LeftRail.tsx (label key + union), Icon.tsx
('forge' + glyph), BrowserView.tsx (import + ⚡ + mount), WorkspaceHome.tsx
(openForge + button + doc rewords), ManagerAgentSetup.tsx (doc + permission row
reword). Typecheck green; screenshots READ (title bar 2 cockpit tabs, no sprint
segment, ＋ menu 3 rows).

**Amendment commit:** CONSTITUTION.md only — preamble lines 5-6 + 11-14.

**Rust half:** `git rm -r crates/marley_forge_client`; NEW
src/browser_probe.rs + tests/browser_probe_livewire.rs; app.rs (imports :31-34 +
forge_view import :73-75, fields :406-414/:424-430, MARLEY_OWNER :679, asset row
:749, pump drains :1823-1828/:8385-8453, boot :2455-2532 collapse (keep
fleet_config_dir for the future-seam? — NO: cursor dir feeds only the
subscription; dies; `fleet_target`+classify(2-param) stay), init :2669-2677,
refresh/claim/comment/row fns, dock arm :6142-6160, dispatch :10012-10018, demo
gate :10189 → `demo_feed_permitted(false)`, SpawnProbe :11031 repoint, overlay
snapshot :10835, ⌘⇧F overlay :20703-20733, sprint render :21223-21231 + click
index :21236, icon test :21917, test doors :2821-2828); DELETE forge_view.rs +
tests/fleet_livewire.rs + mcp_config.rs (+ lib.rs decls :58/:72, + new
`pub mod browser_probe;`); browser.rs (OverlayStates forge_open + disjunct +
fixtures + 26-count docs + :212/:386 re-voice); browser_state.rs (:29 use
repoint); fleet_live.rs (ConnectionState re-home, classifier reshape, header
kept, 6 tests + t384_req002 rework); fleet_rail.rs (NO_TRANSPORT_REASON replaces
:140-143; :798 repin); status_bar.rs (sprint_summary + param + segment + test);
right_dock.rs (variant/key/label/icon/tabs + 5 tests + docs); content.rs (slots
3→2, slot() match, docs, 2 tests); context_menu.rs (SECTION_BROWSER_ITEMS 4→3,
#387 default → Agents, docs, 2 tests); tabs.rs (docs + 1 test); icons.rs
(variant + path + 2 test rows); keymap.rs (row :160-163 + asserts :652/:1293 +
comment rewords); headless_drive.rs (arm-c drive DELETE :10065; 2 machine drives
reword; C=forge restore reword :11312; mcp_json pin re-voice :11685; pump probe
rows + 3 instantiation rows repoint + banner); assets/icons/forge.svg DELETE;
Cargo.toml (dep :33 out; url comment reword); gates.sh (:233 regex fragment +
:201-204 prose); prose (marley_mcp ×6 + Cargo.toml:8, marley_fleet lib.rs:4,
agent_view.rs:3-4, grid_layout/orchestration light re-voice).

### Regression test plan
| REQ | Test(s) | Kind |
|---|---|---|
| REQ-001 | dir-absent + grep=0 + workspace build/tests green | validate greps + compile |
| REQ-002 | 8 moved probe units + inlined `status_line_code` arms unit + 13 moved livewire rows RUN; browser_state 19 compile on new path | units + integration |
| REQ-003 | NEW/reworked tolerant-boot pin: settings `right_section = "forge"` + grid `C=forge` → Details (the reworked cockpit-restore drive covers the grid half; a right_dock/settings unit covers the key half) | unit + drive |
| REQ-004 | classify_fleet_setup reshaped arms unit; t384_req002 reworked (arm-c gone); fleet_rail :798 NO_TRANSPORT repin; the 2 machine drives reworded; grep NO_FORGE_CLIENT_REASON = 0 | units + drives + grep |
| REQ-005 | 4 cockpit_status_* re-pins (single-segment ordering) + the i==0 click arm (headless where reachable; live drive) | units + drive |
| REQ-006 | keymap asserts: no toggle-forge; editor-scope ⌘⇧F intact | units + grep |
| REQ-007 | grep sweep (410's false-positive list respected: forge/forgery English verbs); amendment commit isolation checked at /commit | greps + review |
| REQ-008 | POC typecheck + screenshots READ; parity pair at validate (post-rip chrome: title bar, status bar, ＋ menu) | visual |
| REQ-009 | scripts/gates.sh --diff exit 0 | gate |
| moved-probe MSI | mutation --diff covers browser_probe.rs bodies; PR-shared-copy-closure shape preserved in moved tests | mutants |
| trybuild | none — deletions + a module move; no new type contract (recorded) | n/a |
| uncoverable | none NEW; the adapter.rs exclude LEAVES gates.sh with the crate | n/a |

### Risks
1. The boot-collapse region (app.rs :2455-2532) interleaves fleet + forge wiring
   with SURVIVING neighbors (browser signal channel, fleet_config_dir feeds the
   cursor dir which dies WITH the subscription — verify nothing else reads
   fleet_config_dir) — implement reads the full region before cutting.
2. Hard-arity shrinks (slots 3→2, menu 4→3, tabs 3→2) ripple into exhaustive
   matches — compiler-led, but the #387 default-flip (row-0 → Agents) is a
   BEHAVIOR change pinned by a repointed test.
3. The gates.sh edit invalidates any pre-edit receipt — P4's gate run comes
   after every edit (normal flow; noted so nobody trusts an early green).
4. The keymap's ⌘⇧F story flips (global binding gone; editor-scoped survives) —
   the two keymap tests + comments must tell the new story coherently.
5. marley_fleet is untouched BUT forge_client depended on it — deleting the
   consumer must not orphan marley_fleet types used elsewhere (fleet_rail uses
   marley_fleet::FleetSnapshot etc. — verified by compile).

## Phase 3 — Implement
- **React-first (executed, delegated batch):** ForgePane.tsx + ForgeOverlay.tsx deleted;
  12 files edited (Workspace/TitleBar/StatusBar/FleetDock/ContextMenu/CommandPalette/
  App/LeftRail/Icon/BrowserView/WorkspaceHome/ManagerAgentSetup) per the sweep table;
  one type-forced addition (App.tsx fleetDockMock union drops 'client'). tsc green.
  Visual verify (411-poc-after.png, READ): title bar has NO Forge cockpit tab (wrench
  icon gone), status bar reads "no agents · lsp: ready · focus: browser" (sprint
  segment dead, agents first), Browser section carries only the web tab.
- **CONSTITUTION amendment:** isolated commit `12a775e` (preamble thesis + crate
  sentence de-forged; only CONSTITUTION.md in the changeset).
- **Probe re-home:** `src/browser_probe.rs` (verbatim move + inlined
  `status_line_code` with a new arms unit; module doc re-voiced; zero mutants::skip
  kept) + `tests/browser_probe_livewire.rs` (13 rows verbatim, imports repointed);
  `pub mod browser_probe` in lib.rs; browser_state/app/headless repoints.
- **Rust rip (mine + a delegated compile-to-green batch):** crate dir + forge_view.rs
  + mcp_config.rs + tests/fleet_livewire.rs + forge.svg deleted; app.rs
  sprint/transport/boot surgery per manifest (boot collapsed to
  `classify_fleet_setup(target, config_dir)`; confirm arms send the no-transport
  failure through the STANDING result channels; pump keeps folds + scrubs, drops
  incoming/state/echo drains; demo gate `demo_feed_permitted(false)`; click index
  1→0); fleet_live reshaped (2-arg classifier, 1-arg header, MisconfigArm single-
  variant); fleet_rail (`NO_TRANSPORT_REASON` "no fleet transport"; observe fn +
  its matrix test deleted; `pending` kept — consumed by the receipt fold); status_bar
  (sprint segment out, tests re-pinned); right_dock/content/context_menu/tabs/icons/
  keymap arity+vocabulary shrinks (#387 default → Agents; ⌘⇧F global row out, roster
  82→81); browser.rs OverlayStates 27→26; headless drives per plan (arm-c deleted;
  machine drives fold to Failed(no-transport) same-tick; C=agents restore fixture;
  mcp_json tombstone re-banner); gates.sh adapter-exclude out; marley_mcp ×7 +
  marley_fleet + agent_view + grid_layout + settings prose.
- **Deviations from design (recorded):**
  1. **F1 AMENDED — ConnectionState COLLAPSED, not re-homed:** a re-homed 2-variant
     enum would have NO non-test constructor → `variant never constructed` at
     `-D warnings`. The header is 1-arg; the wire-suffix display retired with its
     producer; 6 fleet_live title tests reshaped accordingly.
  2. **F2 CASCADE (dead-code-forced):** with the async sends gone, keeping the fold
     fns CALLED required the confirm sites to send the typed failure through the
     standing channels (the constant-input lesson applied to a channel); the
     forge-SSE-specific echo layer (`dispatch_observe_record` + its matrix test)
     died with its producer — `DispatchPhase::Delivery` + `pending` SURVIVE via the
     receipt fold's construction. The two machine drives needed one extra pump call
     (the fold now lands same-tick).
  3. The Explore-missed `boot_configured_root...quiet` drive: the old
     misconfigured-endpoint wire drive re-pinned to the new quiet-Configured truth
     (a configured root titles bare "Fleet").
  4. mcp_config.rs deleted WITH its 7 path tests (the 410-deferred module falls
     here as designed — no deviation, noted for the ledger count).
- **Compile state:** `cargo check --workspace --all-targets` green; clippy
  `-D warnings` green; fmt applied; `cargo nextest run -p marley --lib` →
  **950/950 passed**. Final grep: remaining forge hits are tombstone comments,
  deliberate tolerant-boot pins, and the English verb (classified list in
  transcript). Cargo.lock: zero forge_client entries.

## Phase 3.5 — Inspect
- **Critics run (2, parallel, general-purpose; 4 lenses each):** correctness/AC (with
  the two design amendments as the primary targets) · state-integrity + security/
  provenance + simplification. Both verified concretely: byte-compares of the moved
  probe, full-suite RUNS (workspace 2047/2047; targeted lanes), mechanical 26/26/26/26
  overlay count, persisted-vocabulary greps, `bash -n`/shellcheck on gates.sh,
  `git show --stat 12a775e` (amendment isolation confirmed).
- **Findings ledger:**

| # | sev | finding | verdict | action |
|---|---|---|---|---|
| I1 | low | The nonce-supersede drop arm (the exact path the F2 no-transport sends lean on) had no test — two-queued-sends-one-record is now deterministically constructible | REAL (coverage gap on a newly-load-bearing arm) | FIXED — the dispatch machine drive gained the supersede row (third confirm before any fold; stale nonce drops, newest brief survives); RUN green |
| I2 | low | Stale `C=forge` grid blob claimed tolerant in a doc but pinned by no test (composition-only) | REAL (promise without a mutant-real pin) | FIXED — `restore_shell_stale_forge_cockpit_falls_back_to_details` added; RUN green |
| I3 | low/nit | 4 stale comments (config-dir copy purpose, footer #94 header, spacer segment list, fleet_cursor_dir's deleted-fn name-drop) | REAL (prose drift) | FIXED — all re-voiced |
| I4 | low | `fleet_cursor_dir` + `fnv1a` have zero production callers (dead_code shielded by the lib.rs pub re-export) | ACCEPTED — the documented kept-seam for the future transport (module doc names it); flagged for a future rip if the seam stays cold | none now |
| I5 | info | `DeliveryState` delivery arms production-unreachable (pending's Some-writers are test-only) | ACCEPTED — documented future-brain seam, unit-pinned | none |
| I6 | info | `marley_core::marley_mcp_config_file_path` purpose-dead (pre-existing orphan; this diff removed its last textual reference) | ACCEPTED — pre-existing; recorded as a follow-up rip candidate (Phase 5 notes) | carried |

- **Provenance:** browser_probe.rs = 70/346 changed lines vs the deleted probe.rs, all
  doc rewording + the byte-identical `status_line_code` inline + 1 new unit; POC
  deletions are Marley's own files. **Secrets:** zero bearer/token material in added
  lines; the tombstone fixture's sentinel pre-exists. **No high/medium findings.**
- **No F-/PR- appends:** no shipped-code bug (I1/I2 are test-strength fixes landed
  in-phase); Phase 5 owns the lessons.

## Phase 4 — Validate
- **Planned tests — all landed by this phase's close:** the 21 moved probe tests
  (9 unit incl. the new `status_line_code_arms_exact` + 13 livewire) RUN green; the
  reworked fleet_live/status_bar/right_dock/content/context_menu/tabs/keymap/browser
  re-pins RUN green; the two inspect-added pins
  (`restore_shell_stale_forge_cockpit_falls_back_to_details`,
  the dispatch drive's nonce-supersede row) RUN green.
- **Suites RUN (real output):** `cargo nextest run --workspace` →
  **2048 tests run: 2048 passed, 5 skipped** (the 5 = pre-existing documented lane
  skips). `cargo test --workspace --doc` → ok, 0 failed.
- **Acceptance greps:** crate dir GONE; `marley_forge_client` refs = 1 (the probe's
  re-home attribution comment — exemption class); `toggle-forge` refs = 4 (keymap
  history comments + the roster guard's rationale); `NO_FORGE_CLIENT` = 0;
  CONSTITUTION `Forge` count = 0.
- **Live drive + parity pair (rebuilt binary, direct-exec relaunch; captures in
  `.playwright-mcp/`, all READ):**
  - `411-live-boot.png` — title bar carries NO Forge cockpit affordance (the wrench
    icon gone); status bar reads "no agents · focus: terminal" — the sprint segment
    dead, agents at index 0.
  - `411-live-menu.png` — the Browser ＋ menu is EXACTLY 3 rows, **Agents / Details /
    Browser**, Agents seated as the row-0 default (the re-seated #387 default),
    keyboard-selected on open. Esc closed it; workspace left as found.
  - React pair `411-poc-after.png` (READ at implement): the same three structural
    absences on the POC side (no Forge tab, no sprint segment, no forge rows).
    The 411 deltas are REMOVALS — presence/absence verified on both captures; the
    surviving chrome's pixel tiers were pinned by 410's magick pair and are
    unchanged. **PARITY: PASS — no Rust fix, no marley-web fix.**
- **Pre-existing notes:** `block v0.1.6` future-incompat (upstream, unchanged). The
  2 "ignored headed" tests in `cargo test -p marley` are the documented headed lane.
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15 passed / 0 failed,
  exit 0. gate:4 lines 45298/45298 = 100.00% (the shrunken denominator reflects the
  deleted crate; browser_probe.rs fully in); gate:5 mutation on the diff — **19
  caught / 0 missed → MSI 100.0%**; gate:15 158/158. Receipt written.

## Phase 5 — Complete
- **CHANGELOG:** Unreleased/Changed slice-2 entry added above slice 1's.
- **Architecture docs (delegated grep-sweep):** `marley_forge_client.md` deleted;
  crate-map / 00-overview / crate-triage / clean-build-plan / marley_agent /
  pane-composition-model / fleet-control-plane swept — live claims updated to the
  as-ripped shape, history annotated (result recorded by the sweep agent; residue
  classified). Parity docs (MARLEY-PARITY.md rows, README, DEMO_FEATURES,
  STEP2-PORT-INSPECTION banner) updated on the marley-web side.
- **Ledger appends (codes):**
  `AD-claude-fleet-rail-quiet-no-transport-until-brain-001` (architecture-decisions);
  `PR-claude-dead-code-analysis-designs-the-rip-keep-list-001` (prevention-rules —
  the F1-flip lesson as a rule). Slice-1 siblings cross-linked.
- **Follow-up candidates recorded (not ticketed — below ticket-grain):**
  `marley_core::marley_mcp_config_file_path` purpose-dead orphan (inspect I6);
  `fleet_cursor_dir` cold-seam roster review if the brain transport stays unbuilt
  (inspect I4).
- **Ticket:** TICKET-411 → `tickets/closed/`, status closed. BACKLOG sweep: no stale
  row (left at promotion; TICKET-319 now tops the Queue).
- **Archive:** spec+notes pair → `docs/planning/pipeline/completed/`.
- **Harness note:** the direct-exec Marley instance from the P4 drive was left
  running (workspace as found); the marley-web dev server likewise.
