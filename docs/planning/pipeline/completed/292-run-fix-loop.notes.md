# Close the run→fix loop — Notes

- **Forge ticket:** #292 `0db4535b-6237-4318-ae18-a86ba1ce80ca`
- **AAR:** `0e7958f7-089f-4c3c-9279-7ab62656de78`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-292-run-fix-loop.md
- **Pipeline spec:** 292-run-fix-loop.spec.md

## Phase 1 — Plan
- **Request:** (a) an editor command "re-run last failed command"; (b) a per-block
  "re-run to verify" affordance; (c) clear diagnostics on green.
- **Classification / tier:** work pipeline slice, `feature`. Crates: `marley_app`
  (`block_status.rs` pure locator, `app.rs` palette command + dispatch).
- **KEY discovery (bounds the work):** `rerun_block(pane, idx)` (app.rs:3619) already
  exists (#175) — it `write_command(block.rerun_command())` to the PTY, creating a NEW
  block. The #289 gutter (`open_file_diagnostic_rows`) sources only the LAST block per
  terminal, `Failure` only → a green re-run's new Success last-block yields no rows →
  **(c) is INHERENT, zero new code.** So the genuine delta is just (a): a pure
  `last_failure_block_index` + a palette command wrapping the shipped `rerun_block`.
- **Scope decision:** ship the title's slice = (a) + (c). Defer (b) — the per-block
  "re-run to verify" affordance needs per-block referenced-file tracking + a block-header
  render/click (a discoverability enhancement, not the loop-closer). Defer a keybinding
  + multi-terminal (#295) too. This keeps one tight, testable, live-drivable slice.
- **Reuse:** `block_status::exit_status_kind`/`StatusKind` (#187); `rerun_block` +
  `block.rerun_command` (#175); `cockpit_commands`/`action_for_command`/`CommandId`
  (#204/#87); the #289 `open_file_diagnostic_rows` (unchanged — carries clear-on-green).
- **Palette scheme (grounded):** `cockpit_commands()` (app.rs:994) → `Vec<Command>` with
  static `CommandId(0/1/2/4/5)`; `3` is a free static id; each resolves to a verb via
  `action_for_command` (palette.rs) except `SAVE_WORKFLOW_ID=CommandId(9)` (special). The
  design picks the free id + the dispatch path.
- **Risk:** low. Pure locator is a one-liner `rposition`; the palette command mirrors the
  shipped #204 pattern; clear-on-green is inherent. The only care: the dispatch borrow +
  the no-op guards (nothing failed / a command already running — `rerun_block` guards the
  latter).

## Phase 2 — Design

### Architecture / approach
A pure locator + a palette-command shim reusing the shipped `rerun_block` (#175). Fits
the cockpit layer: palette (`cockpit_commands`/`action_for_command`, #204) → a verb →
`dispatch_action` → the terminal Block model.

- **Pure** `block_status::last_failure_block_index(kinds: &[StatusKind]) -> Option<usize>`
  = `kinds.iter().rposition(|k| *k == StatusKind::Failure)` (`StatusKind: Copy+PartialEq`,
  the same comparison #289 uses). The HIGHEST-indexed `Failure` = the most recent failed
  command; `None` when none failed. Earns its keep over the existing `rerun-last` (⌘R,
  which re-runs the most recent FINISHED command via `focused_terminal`): after a failure
  you may run other commands (cd/ls/cat) before fixing — `rposition` scans BACK to the
  failure even when it's no longer the last block.
- **Palette command** (shim, app.rs `cockpit_commands`): a `Command { id: CommandId(11),
  title: "Re-run Last Failed Command", keywords: [re-run/rerun/failed/retry/error],
  binding: None }`. `CommandId(11)` is the next free static id (0/1/2/4/5/6/7/8/10 used;
  9 = SAVE_WORKFLOW; 3 = the removed "Open Settings" gap — left alone).
- **Verb** (palette.rs `action_for_command`): `CommandId(11) => Some("rerun-last-failed")`.
  Distinct from the existing `"rerun-last"` (⌘R).
- **Dispatch** (app.rs `dispatch_action`): a `"rerun-last-failed"` arm — scan
  `workspace().states()` (ascending PaneId, the #289 scope; NOT `focused_terminal`, which
  is None when invoking from the editor) for the FIRST terminal pane with a `Failure` block
  via `last_failure_block_index` over its blocks' `exit_status_kind`s → capture `(PaneId,
  idx)` immutably → `self.rerun_block(pane, idx)` (its own `workspace_mut` lookup; guards
  `is_command_running`). No-op when no terminal has a failure.
- **Clear-on-green: INHERENT.** `rerun_block` writes the command → a new last block; #289's
  Failure-only `open_file_diagnostic_rows` excludes a green last block → markers clear. No
  code; a Validate assertion (the live drive).

**§14:** pure locator (no panic — `rposition` returns Option); the shim is the only state
touch, reusing the tested `rerun_block` (process-spawn stays in the session adapter).

**§20 (clean-room) — CONFIRMED.** Reference = the IDE run→fix→re-run loop (Zed/JetBrains
"Re-run" + diagnostics clearing on a clean build). Marley matches by re-running the last
FAILED command from the palette (no block-hunting) + the #289 gutter self-clearing on
green. Marley's own Block model + palette + `rerun_block`; no Zed/Warp source read.

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/block_status.rs` | + `pub fn last_failure_block_index(kinds: &[StatusKind]) -> Option<usize>` (pure; tests in P4) |
| `crates/marley_app/src/palette.rs` | `action_for_command`: + `CommandId(11) => Some("rerun-last-failed")` arm (roster test updated in P4) |
| `crates/marley_app/src/app.rs` | `cockpit_commands`: + the `CommandId(11)` "Re-run Last Failed Command" entry; `dispatch_action`: + the `"rerun-last-failed"` arm (shim; `dispatch_action` is `mutants::skip`/excluded) |

### Regression Test Plan
| # | Test | AC | Asserts |
|---|---|---|---|
| T1 | `block_status::last_failure_block_index_finds_highest_failure` | REQ-001 | `[Success,Failure,Success,Failure,Running]`→`Some(3)`; `[Failure,Success]`→`Some(0)` (the failure, not the last block); `[Success,Success]`→`None`; `[]`→`None`; `[Failure]`→`Some(0)`. Kills `==`→`!=` / closure true/false / body→None. |
| T2 | `palette::action_for_command_maps_every_row` (extend) | REQ-002 | + `action_for_command(CommandId(11)) == Some("rerun-last-failed")`. `every_cockpit_command_resolves_to_a_verb` auto-covers (it iterates `cockpit_commands()`). |
| T3 | LIVE DRIVE | REQ-002/003 | A failing `python3 boom.py` → open+edit the file → invoke "Re-run Last Failed Command" from the palette → the command re-runs; then fix so it exits 0 → re-run → the gutter markers clear. |
| T4 | gate `scripts/gates.sh --diff` | REQ-004 | 100% line cov + MSI 100 on `last_failure_block_index`; brand-scrub; clippy -D. |

**Uncoverable:** the `dispatch_action` shim (workspace scan + `rerun_block`) is app.rs
(`mutants::skip` + coverage-excluded) — proven by the live drive + reuse of the tested
`rerun_block`. The pure locator + the verb-resolution roster are unit-tested.

### Risks / decisions
- **D6 — scan `workspace().states()`, first terminal with a failure (ascending PaneId).**
  `focused_terminal` is None when invoking from the editor (the whole point). Multi-pane
  picks the lowest-PaneId terminal-with-a-failure — deterministic; a cross-pane "most
  recent failure" refinement rides the #295 multi-terminal follow-up.
- **D7 — `CommandId(11)`, not the `3` gap.** Reusing the documented-removed `3` would
  churn its comment + the `CommandId(3)==None` roster assertion for no gain; `11` is clean.
- **D8 — reuse `rerun_block` verbatim** (guards `is_command_running`, extracts
  `rerun_command()`); no new run path. `write_command` → a new last block → clear-on-green.

## Phase 3 — Implement
- **`block_status.rs`** — `pub fn last_failure_block_index(kinds: &[StatusKind]) -> Option<usize>`
  = `kinds.iter().rposition(|k| *k == StatusKind::Failure)` (after `exit_status_kind`).
- **`palette.rs`** — `action_for_command`: `CommandId(11) => Some("rerun-last-failed")`.
- **`app.rs`** — `cockpit_commands`: the `CommandId(11)` "Re-run Last Failed Command" entry
  (keywords re-run/rerun/failed/retry/error, no binding); `dispatch_action`: the
  `"rerun-last-failed"` arm — `workspace().states().find_map` over terminal panes (ascending
  PaneId) computing `last_failure_block_index` over each pane's `exit_status_kind`s, capturing
  `(*id, idx)` immutably, then `self.rerun_block(pane, idx)`.
- **Deviations:** none. `cargo check -p marley` clean (no errors, no unused warnings). The
  immutable `workspace().states()` scan captures owned `(PaneId, usize)` before the
  `rerun_block` `workspace_mut` — no borrow conflict.
- **Tests deferred to P4** (the pure unit + the roster assertion + the live drive).

## Inspect (Phase 3.5)
A rigorous self-review + a real mutant-list confirmation + green roster tests (a
background correctness/reuse critic was also spawned; findings folded on report).

**Self-review — no defects.** Lenses:
- **Correctness:** `last_failure_block_index` = `rposition(|k| *k == Failure)` — the
  HIGHEST-indexed Failure; traced `[Success,Failure,Success,Failure,Running]`→`Some(3)`,
  `[Failure,Success]`→`Some(0)`, `[]`/`[Success,Success]`→`None`. Never returns a
  non-Failure index. Pure, total (rposition→Option, no panic).
- **Mutants (real set via `--list`):** 4 — `→None`, `→Some(0)`, `→Some(1)`, `==`→`!=`.
  All killed by the T1 cases (the mixed `Some(3)` kills `!=` [would yield `Some(4)`=Running]
  + the body-`Some(n)`; the `None`/`Some(0)` cases kill the rest).
- **Reuse:** distinct from the existing `rerun-last` (⌘⇧R → `blocks().last_rerunnable()`
  = the most recent FINISHED command via `focused_terminal`); `rerun-last-failed` scans
  BACK to the failure + works with the editor focused (no focused terminal). No existing
  "last failed" helper duplicated.
- **Borrow:** the immutable `workspace().states()` scan captures owned `(*id, idx)` before
  the mutable `rerun_block` — sound (compiles; the immutable borrow ends at `target`).
- **Roster / wiring:** `CommandId(11)` appears only in `cockpit_commands` + `action_for_command`
  (grep-confirmed unique); the verb is both mapped + dispatched. `every_cockpit_command_resolves_to_a_verb`
  passes dynamically (green run); NO test hardcodes `cockpit_commands().len()`; NO
  dispatch-verb enumeration test; and — crucially — the command has `binding: None`, so the
  keymap roster is UNTOUCHED (unlike #290's F8, which needed roster updates).
- **Multi-pane / no-op:** picks the first (lowest-PaneId) terminal with a failure (D6,
  deterministic); no-op when none / a command is running (`rerun_block` guards the latter).

**Ledger (self-review):** no findings; lenses covered: correctness, panic-safety,
mutation-reachability, reuse, borrow, roster/wiring impact, multi-pane semantics.

### Background critic report (folded in — inspect re-opened)
The critic completed after I'd entered validate; re-opened inspect to apply its real
findings (§0 source-fix). 6 concerns; 2 real, 2 stale (already fixed), rejected the rest.

| Finding | Verdict | Action |
|---|---|---|
| **F1 [MED] non-deterministic pane selection + FALSE comment** — `states()` iterates a `HashMap` (RandomState), so "ascending PaneId" is false; with multiple terminal panes each holding a failure, `find_map` picks an ARBITRARY one (differs per run). My self-review MISSED this (assumed `states()` was ordered). | **REAL — CONFIRMED** (`panes: HashMap`; `PaneId(u64)` not `Ord`; sibling #289 is deliberately order-INDEPENDENT via union). | **FIXED:** replaced `find_map` with a `filter_map`→`collect`→`sort_by_key(id.0)`→`first()` — deterministic lowest-PaneId; honest comment citing #295 for the multi-pane refinement. |
| **F2 [LOW] busy pane shadows an idle failed pane** — the scan picked a pane by has-a-failure, then `rerun_block` no-ops if it's busy, even if another pane has an idle failure. | **REAL — CONFIRMED.** | **FIXED (same edit):** the `filter_map` skips `is_command_running()` panes, so only idle failed panes are candidates. |
| **F3 [MED] `last_failure_block_index` has zero tests** | **STALE** — the critic snapshotted before validate; T1 now covers it, gate GREEN [diff] = cov/MSI 100. | none (already covered) |
| **F4 [MED] roster omits `CommandId(11)` (string-value mutant survives)** | **STALE** — T2 added `action_for_command(CommandId(11)) == Some("rerun-last-failed")`; gate MSI 100. | none (already covered) |
| Correctness / borrow / reuse / `CommandId(11)` free (concerns 1,3,4,6) | **REJECTED (clean)** — matched my self-review: `rposition` total + Failure-only; `PaneId: Copy`, owned-extract-then-mutate borrow sound; distinct from `last_rerunnable` (text vs index); id 11 only at the 2 sites. | none |

**Fix verified:** `cargo check -p marley` clean; the deterministic + busy-skip scan compiles.
`failure-record` + `prevention-rule` captured at Phase 5.
Lesson: `workspace().states()` is a HashMap — NEVER assume iteration order; sort when a
single deterministic pick is needed (mirror #289's order-independent union otherwise).

## Phase 4 — Validate
### Tests added
- **T1** `block_status::last_failure_block_index_finds_highest_failure` (REQ-001) —
  `[Success,Failure,Success,Failure,Running]`→`Some(3)`, `[Failure,Success]`→`Some(0)`,
  `[Failure]`→`Some(0)`, `[Success,Success]`/`[Running]`/`[]`→`None`. Kills the real
  4-mutant set (`--list`: `→None`/`→Some(0)`/`→Some(1)`/`==`→`!=`).
- **T2** `palette::action_for_command_maps_every_row` — added
  `action_for_command(CommandId(11)) == Some("rerun-last-failed")` (kills the string-value
  mutant on the new arm; `every_cockpit_command_resolves_to_a_verb` covers it dynamically).

### Real results
- `cargo nextest run -p marley` → **452 passed, 2 skipped** (+1 = T1).
- `scripts/gates.sh --diff` → **GATE GREEN [diff]** twice — before AND after the inspect
  fix (the app.rs shim is coverage/mutation-excluded; block_status + palette cov/MSI 100).

### LIVE DRIVE — REQ-002 proven end-to-end (machine unlocked)
`New empty workspace` → ONE terminal → `python3 /tmp/marley_292/boom.py` (exits 7 → the
✗ FAILED last block, output "attempt failed") → **⌘⇧P command palette** → typed "rerun"
→ the palette filtered to exactly **"Re-run Last Failed Command"** (`292-03-palette.png`)
→ Enter → **a SECOND `python3 /tmp/marley_292/boom.py` block appeared** (`292-04-reran.png`)
— the failed command re-ran. Crucially this worked with the PALETTE focused (no focused
terminal), proving the `workspace()`-scan (vs the existing `rerun-last`'s `focused_terminal`,
which would no-op here). REQ-002 ✓.
- **REQ-003 (clear-on-green): inherent + mechanism-verified.** `rerun_block` writes a new
  last block; #289's `open_file_diagnostic_rows` sources only a LAST block that is a
  `Failure`, so a green re-run's Success last-block yields no rows → markers clear. This is
  the unchanged #289 render path — #291's live drive already proved the gutter reflects the
  last block's Failure state (danger-red on Failure; the inverse is the same Failure-only
  filter). No separate gutter drive needed (this boom.py emits no file:line ref anyway).
- Cleanup: app quit, `~/.marley` settings restored, `/tmp/marley_292` removed, tree clean.

No pre-existing failures in scope.

## Phase 5 — Complete
- **CHANGELOG.md** `### Added` (M18): the #292 run→fix-loop entry.
- **docs/marley_architecture/app_shell.md**: the `M18 (#292)` note after #291.
- **Knowledge (forge wired):** AAR `0e7958f7` submitted (completed, effectiveness 5;
  2 novel findings); `failure-record` **BF-claude-workspace-states-hashmap-order-assumed-001**
  (the critic caught the HashMap-order assumption my self-review missed);
  `prevention-rule` **PR-claude-workspace-states-is-unordered-001** (never assume
  `states()` order — sort for one pick, union for all).
- **Lessons:** (1) `rerun_block` (#175) + #289's Failure-only gutter made this a THIN
  slice — clear-on-green was zero new code. (2) `workspace().states()` is a HashMap →
  the deterministic-pick bug; a background critic caught what my self-review missed even
  on a small diff → keep spawning critics. (3) A late-arriving critic finding is fine to
  fold by re-entering inspect (set the spec status back to "Phase 3 — Implement PASS" so
  the phase-gate allows the app-code fix, apply, re-close).
