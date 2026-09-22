# persist + boot the pane grid (M6 seq-3) — Notes

- **Forge ticket:** #122 `43b3a888-195a-4af0-bb61-b97b180678a1` · **AAR:** `f4f5806d-af25-44c5-9396-0475125e7d7b`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-122-grid-layout.md

## Phase 1 — Plan
- **Request:** forge #122 (M6 run 1/10) — serialize/restore the pane grid + boot into it; retire #121 TEMP.
- **Pre-flight:** PaneGroup = Leaf|Split (binary); Workspace::new = Terminal pane 0; #118 settings round-trip pattern.
- **Decisions:** D1 flat-row model (axis+kinds), malformed→[Terminal]; D2 workspace.grid + boot rebuild.
- **AAR id:** `f4f5806d-af25-44c5-9396-0475125e7d7b`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **grid_layout.rs (NEW PURE):** GridLayout{axis:PaneAxis, kinds:Vec<PaneKind>}; kind_char(t/f/c/g)+char_kind(→Option); flatten(group,kinds,out) DFS leaves; serialize_grid → single leaf="t" / split="H:t,f,c"; restore_grid → split_once(:) → axis H/V + comma kinds (each a single valid char) else default {H,[Terminal]}; single char → {H,[kind]}; empty/multichar/unknown/bad-axis → default. mod grid_layout in lib.rs.
- **settings.rs:** define_setting WorkspaceGrid: String=""; AppliedSettings.grid; applied_defaults/applied_from; persist_grid(manager,&str). Update ALL AppliedSettings literals.
- **app.rs SHIM:** new() drops the #121 TEMP block → `let grid = restore_grid(&applied.grid); let mut workspace = Workspace::new(session); for k in grid.kinds.iter().skip(1) { match k { Terminal→split_focused(spawn), FileTree/CodeView/Git→open_pane(H,After,PaneContent::_) } } workspace.focus(PaneId(0))`. self.persist_grid() helper (serialize_grid(group,kinds-map) → persist_grid) called after split/close/open. NOTE kinds[0]=Terminal assumed.
- **Mutation targets:** kind_char/char_kind arms, axis H/V both ways, malformed defaults, single-vs-split, applied_from grid, persist_grid.
- **Test plan:** grid_serialize_split ([T|F]→"H:t,f"), grid_round_trip (restore∘serialize preserves axis+kinds; all 4 kinds), grid_single ("t"→[Terminal]; V:t,f→Vertical), grid_malformed (""/"garbage"/"H:x"/"X:t"→[Terminal]), settings_grid_round_trip (persist_grid→reload→applied.grid restores).
- **Risks:** kinds-map for serialize built from workspace pane_ids→kind(); the boot rebuild order matches serialize DFS order; every AppliedSettings literal +grid (compiler-enforced).

## Phase 3 — Implement
- **Built:** grid_layout.rs (GridLayout + serialize_grid/restore_grid + kind_char/char_kind/flatten/one_char_kind, cov/MSI 100 target); mod; settings WorkspaceGrid + applied.grid + persist_grid + all AppliedSettings literals; app.rs boot rebuild (restore_grid → Workspace::new + open each kinds[1..] pane → refocus pane 0) REPLACING the #121 TEMP default + a persist_grid() helper called after split-pane + × close.
- **DEVIATION:** a boot CodeView pane opens with a placeholder CodeViewState ("// open a file…") since no file is bound yet (seq-5 opens real files). persist_grid wired to split-pane + × close (the ssh/new-agent splits persist on the next layout action — minor).
- **Verification:** fmt; check 0 err; clippy OK; 5 grid tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review of a small parser + a masked boot rebuild; the gate cargo-mutants is the adversarial authority on serialize/restore, + a thorough malformed-input test.
- **Lenses — no findings:** serialize_grid: single-leaf→bare char, split→"axis:kinds" DFS order; restore_grid: split_once(:) → axis H/V (bad→default) + each token exactly one valid char (empty/multichar/unknown→default), no-colon→single char; empty→default. kind_char↔char_kind are exhaustive inverses. The malformed test covers ""/whitespace/no-colon/multichar/unknown-char/trailing-comma/bad-axis. No unwrap/panic (all Option-guarded). The boot rebuild opens kinds[1..] (pane 0 = the initial terminal) + refocuses pane 0; a malformed grid → one terminal (safe). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** grid_serialize_split, grid_round_trip_all_kinds, grid_single_and_vertical, grid_malformed_defaults_to_one_terminal (grid_layout) + settings_round_trip extended with persist_grid("H:t,f")→applied.grid. 6 pass.
- **Self-test:** LIVE capture (grid122.png) — seeded workspace.grid="H:t,f,c,g" → app BOOTS into 4 panes tiled in a row: terminal (cyan focus+prompt) | Files tree | Code pane ("1 // open…") | ⎇ Source Control. REQ-004 PASS. (settings.toml seeded then restored.) Panes are narrow — the old dock/rail still present; #123-126 retire them for room. Sidebar shows terminal 1-4 (the phantom-label bug → #129).
- **Gate:** GREEN [diff] 15/15. Detour: (1) gate:7/8 — a NEW RustSec advisory RUSTSEC-2026-0204 (crossbeam-epoch unmaintained, gpui transitive) → added to deny.toml + .cargo/audit.toml per the established per-id triage. (2) gate:4/5 — three dead/equivalent branches flagged: the Vertical serialize arm (added a V-serialize test), a dead `if kinds.is_empty()` (removed), a dead `unwrap_or_else` closure in serialize_grid + a dead `?` in code_view parse_file_ref (→ eager unwrap_or). All were Option-always-Some unreachable arms that the 100%-line gate flags (flakily across cov invocations).

## Phase 5 — Complete
- CHANGELOG; forge #122 → done. **M6 3/10.** serialize/restore_grid + workspace.grid + boot rebuild (retires #121 TEMP); live-proven boot into [terminal|files|code|git]. cov/MSI 100. Lessons: advisory triage + the dead-Option-branch coverage trap (PR recorded).
