# M11 #174 — agent jump + cross-tab send — Notes

- **Forge ticket:** #174 `ad82a4c7-16a7-48ba-b5a0-ed9827d8d10a` · **AAR:** `6d6aac68-38d2-4126-a393-a9615abcda7f`

## Phase 1 — Plan / Phase 2 — Design (folded)
- jump_to_pane mirrors the rail project-click (switch_project + sync + switch_tab + focus + persist);
  locate_pane is #173's tested helper. The send resolve swaps the active-grid terminal_mut for a grids_mut
  find_map. Validation exercises the #172-unblocked typed path (⌘⇧S with a composed line).

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- jump_to_pane: locate_pane → switch_project (+ sync_active_project only on a project CHANGE) → switch_tab
  → the owning grid's focus → persist_grid; false on a gone pane.
- The Agents cockpit row (⌘-click diff branch untouched) + the Fleet row (still closes the overlay) both
  jump-or-flash-gone. send-to-agent resolves via grids_mut().find_map(terminal_mut).
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: 1 background critic (ordering/context/send-borrow/flash lenses; full nextest 260/260 + clippy clean).

- **Critic verified the core:** jump ordering exact (project_changed BEFORE switch; sync reads the new root;
  the discarded Results are guaranteed-by-construction; persist captures the jumped-to shape); the cockpit
  closure holds only Copy+cloned data (no use-after); the same-project jump correctly SKIPS sync (preserving
  files_scroll); the send's compose-read + clear both target the ACTIVE grid's focused terminal with the
  grids_mut borrow ended between (clippy-proven); ids unique ⇒ find_map is exact.
- **[Medium → FIXED] the BROADCAST arm was the last active-grid-only agent resolve** — background agents
  were silently skipped while still being ticket-tagged (#80) as if delivered. Now grids_mut().find_map like
  the send.
- **[Low → FIXED] the rail Tab + Pane rows switched projects WITHOUT sync_active_project** (pre-existing,
  flagged adjacent) — the Files tree/cwd/branch stayed on the old project after a cross-project rail click.
  Both now use the project_changed sync gate.
- **[Notes, follow-ups]:** the ⌘-click agent diff is project-blind (runs in the ACTIVE root) — flag for the
  observe thread; a dead-but-present agent's ⌘⇧S still fails flash-less (pre-existing).
- **HARNESS FINDING (fixed in drive.swift, this diff):** synthetic chords LATCH the session's modifier state
  — a later nil-source "plain" click inherited ⌘ and hit the ⌘-click diff branch instead of the jump. Fix:
  click()/rightClick()/typeText()/key() now set flags EXPLICITLY ([] or the intended mask); a `clearmods`
  verb exists as a belt-and-suspenders reset. README-worthy class: deterministic flags beat state cleanup.

Lenses: mutation ordering, closure captures, borrow shape, delivery/tag honesty, harness determinism.

## Phase 4 — Validate
- **Tests:** none new (locate_pane was killed in #173; the shim is masked per the norm). Full suite 260/260.
- **Self-test (REQ-001, the jump):** aj_agents.png — the Agents tab full-screen with `● claude (working)`;
  aj_jump3_rail.png after the row click — the claude TAB is active showing its banner, pane 5 (the agent's)
  focused, "focus: claude". (Two earlier attempts exposed the latched-⌘ harness bug — the click opened the
  ⌘-click diff; fixed above and re-run clean.)
- **Self-test (REQ-002, the typed cross-tab send):** ⌘2 → type:"hello from tab two" → ⌘⇧S: the compose line
  CLEARED (sent=true — pre-#174 it stayed, the silent-fail). aj_delivered_pane.png (⌘1): the agent pane
  shows the line RECEIVED and the agent's live REPLY — delivered to a background tab, processed by the #173
  pump, typed via the #172-retested path.
- **Gate:** GREEN [diff] 15/15.

## Phase 5 — Complete
- CHANGELOG (Added + Fixed) + app_shell #174 note; forge #174 → done; failure-record synthetic-chords-latch-session-modifiers. **M11 3/8.** LESSONS: grep the arm FAMILY when fixing a resolve pattern (broadcast was the sibling); pin flags on EVERY synthetic event; when a driven click acts modified, suspect the harness first.
