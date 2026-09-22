# M10 — rail agent icons + unique PaneIds — Notes

- **Forge ticket:** #167 `409beb81-022d-428d-b68e-7ccda8886411` · **AAR:** `96ea54f1-f7e3-4eca-8705-82c342780fef`
- Absorbs #158 (`e115c821-3483-492d-8ba4-a5d91fdec041`).

## Phase 1 — Plan / Phase 2 — Design (folded)
- Id blocks (1<<32) per grid kill the aliasing with ZERO changes to the 26 map sites; new_with_base keeps
  new()=base-0 so every deterministic test stands. fleet_status_for aggregates a tab's agent statuses
  (Working > Waiting > Idle > Exited). The rail Tab arm prefixes the glyph. Close paths retain-remove the
  closed grids' ids exactly (replacing the blanket last_agent clears).
- **AAR id:** `96ea54f1-f7e3-4eca-8705-82c342780fef`.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- workspace.rs: PANE_ID_BLOCK (1<<32); PaneGrid::new_with_base(initial, base) — new() = base 0 (all existing tests untouched).
- agent_view.rs: fleet_status_for(pane_ids, agents) — max_by_key over rank(Working 3 > Waiting 2 > Idle 1 > Exited 0).
- app.rs: pane_blocks counter + next_pane_block() (blocks * PANE_ID_BLOCK); ⌘T + open_project_path grids mint new_with_base (the boot grid stays base 0); close_tab_at/close_project_at now retain-remove EXACTLY the dead grids ids from agents/remotes + targeted last_agent clears (replacing the #161/#162 blanket clears); the rail Tab arm prefixes agent_status_glyph(fleet_status_for(...)) (accent while Working, muted otherwise); AgentStatus imported.
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: 1 background critic (uniqueness end-to-end incl. a cargo-mutants --in-diff --list dry run) + the
driven capture.

- **Critic verified the core:** every non-test grid-creation site enumerated (boot base-0; ⌘T + open_project
  via next_pane_block; restore builds within block 0; persistence is kinds-only — ids never serialized); NO
  PaneId narrowing casts anywhere; the #153 send-to-agent stale-handle assumption is now exactly TRUE on all
  close paths; the #158 class structurally dead. nextest 55/55; clippy clean.
- **[MAJOR → FIXED] the `mutants::skip` attr was STOLEN:** inserting next_pane_block between
  new_terminal_pane's doc+attr and body re-attached the attr to the NEW fn — new_terminal_pane went unmasked
  (3 unkillable mutants, a guaranteed gate-5 RED, proven via --list). Each fn now carries its own doc+attr.
  LESSON: never insert an item between another item's attributes and body — outer attrs bind to the next item.
- **[MAJOR → FIXED] PANE_ID_BLOCK's `<<`→`>>` mutant was viable** (the test used the const symbolically, so
  0 passed it). Pinned: `assert_eq!(PANE_ID_BLOCK, 1u64 << 32)`.
- **[minor → FIXED] the pane-header × bypassed map cleanup + dropped inline** — now mirrors the ⌘W triple
  (both maps + targeted last_agent + threaded reap).
- **[minor → FIXED] the Working↔Waiting rank swap was unproven** — added the {Waiting, Working} assert.
- **[minor → AAR note] background-tab glyphs freeze** — the pump refreshes only the ACTIVE grid (pre-existing
  #151/#153 class; ties into the M3/M5 observe work). REQ-003 promises presence, not liveness; noted.

Lenses: creation-site enumeration, mutant viability (via --list), attr binding, map-lifecycle exactness,
rank-order proof, narrowing casts.

## Phase 4 — Validate
- **Tests:** new_with_base_mints_from_base (base mint + split continuation + new()=0 + the const PINNED against <<→>>); fleet_status_for_priority (None/empty; Working>Waiting PINNED; Waiting>Idle+Exited; Idle>Exited; Exited-alone; no unlisted-pane leak). 2/2 + workspace/agent 55/55.
- **Self-test:** ar_glyph.png — after ⌘⇧A + ⌘T: tab 1 shows ● claude (the accent Working glyph + the live agent title) with its split panes; terminal 2 (active) shows NO glyph; the footer reads "1 agent · 1 working" — the cross-tab correctness #158 broke, pixel-proven.
- **Gate:** first RED gate:14 (a doc link to the crate-private PANE_ID_BLOCK — the #150 class; de-linked) → GREEN [diff] 15/15, MSI 100 (the two critic majors verified killed).

## Phase 5 — Complete
- CHANGELOG + app_shell #167 note; forge #167 → done, #158 → done (structurally fixed). **The goal batch 161-170 is COMPLETE (10/10 — #163 remains in the sprint).** LESSONS: fix the KEYSPACE not the storage for aliasing bugs; attr binding steals masks; pin consts in tests; rustdoc private-links is a 2×-recurring trap.
