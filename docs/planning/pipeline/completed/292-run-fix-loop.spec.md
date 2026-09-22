---
pipeline_id: 5284a90a-bca5-432b-803c-846878b80b53
ticket: forge#292 (0db4535b-6237-4318-ae18-a86ba1ce80ca) · local docs/planning/tickets/open/TICKET-292-run-fix-loop.md
aar_id: 0e7958f7-089f-4c3c-9279-7ab62656de78
status: Phase 5 — Complete PASS
title: Close the run→fix loop — re-run the last failed command from the editor; markers clear on green
type: feature
milestone: M18
references: [docs/planning/intake/editor-as-peer-and-terminal-fusion.md]
---

## Title
Close the run→fix loop: a "Re-run Last Failed Command" palette command re-runs the
last Failure block (pure `last_failure_block_index` + the shipped #175 `rerun_block`),
and the #289 diagnostics gutter clears on green — inherently, because a re-run writes
a new last block and the gutter sources only a LAST block that failed.

## Scope
### In
- `block_status.rs` (pure): `last_failure_block_index(kinds) -> Option<usize>` — the HIGHEST index
  whose `StatusKind` is `Failure` (the most recent failed command); `None` when none failed.
- `app.rs` (shim): a cockpit palette command **"Re-run Last Failed Command"** (a free static
  `CommandId`, keywords `re-run/rerun/failed/retry`) → resolve the last-failed block in the
  `workspace()` terminal via `last_failure_block_index` → `rerun_block(pane, idx)` (the #175 path;
  a no-op when nothing failed or a command is already running).
- **Clear-on-green: INHERENT (verified, not re-implemented).** `rerun_block` writes the command to
  the PTY → a NEW block becomes the terminal's last; #289's `open_file_diagnostic_rows` scans only a
  LAST block that is a `Failure`, so a green re-run's Success last-block yields no rows → the gutter
  markers clear. A re-run that fails again re-marks from the new block.

### Out (explicitly deferred)
- **(b) the per-block "re-run to verify" affordance + `should_offer_rerun`** — a discoverability
  glyph on the block when the focused editor file is one that failed block referenced. Needs
  per-block referenced-file tracking + a block-header render/click; a follow-up (the loop-closer is
  the palette command + clear-on-green).
- A dedicated keybinding — the palette command is the slice; a chord is a cheap follow-up.
- Multi-terminal scope — `last_failure_block_index` runs on `workspace()`'s terminal (the same scope
  #289 uses); broadening to all terminal tabs is the shared #295 follow-up.

## Reference (§20)
The IDE run→fix→re-run loop (Zed / JetBrains "Re-run" + diagnostics that clear on a clean build).
Marley matches by re-running the last failed command from the palette (no hunting for the block) and
letting the #289 gutter self-clear on green. Clean-room §20: Marley's own Block model + palette (#204)
+ the shipped `rerun_block` (#175); no Zed/Warp source read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the pure seam is `last_failure_block_index`** (highest-index `Failure`). Re-run reuses the
  shipped `rerun_block` (#175) — no new run mechanism; the command's re-invocation is already tested.
- **D2 — clear-on-green is INHERENT, no new code.** `rerun_block` → a new last block; #289's
  Failure-only gutter excludes a green last block. This is D2 of #289 restated, not new logic — a
  Validate assertion, not an implementation.
- **D3 — scope to `workspace()`'s terminal** (consistent with #289/#290; #295 broadens both at once).
- **D4 — a palette command that resolves to a verb** like every #204 cockpit command (a free static
  `CommandId`), not a special-cased dispatch.
- **D5 — (b) the block affordance is OUT** (see Scope/Out) — the title is "re-run from the editor +
  clear on green", which is D1+D2; (b) is an enhancement.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a block's `StatusKind` list has ≥1 `Failure`, `last_failure_block_index` shall return the HIGHEST such index; WHEN none is `Failure`, it shall return `None`. | block_status.rs unit: `[Success,Failure,Success,Failure,Running]` → `Some(3)`; `[Success,Success]` → `None`; `[]` → `None`; `[Failure]` → `Some(0)`. |
| REQ-002 | WHEN "Re-run Last Failed Command" is invoked and `workspace()`'s terminal has a last-failed block, the shim shall re-run that block's command (via `rerun_block`). | Live drive: a failing cmd → invoke the palette command → the command re-runs (a new block appears). |
| REQ-003 | WHEN a re-run finishes with a non-`Failure` exit, the diagnostic gutter markers for that file shall clear. | Live drive: fail → fix+save → re-run → on green the gutter markers disappear (inherent via #289). |
| REQ-004 | `last_failure_block_index` shall be pure at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — `last_failure_block_index` (signature over `StatusKind`s) + the palette command wiring (the free `CommandId` + `action_for_command` verb + the dispatch calling `rerun_block`); test plan.
- **P3 Implement** — the pure fn + the palette command + dispatch.
- **P3.5 Inspect** — the highest-vs-first index, the empty/no-failure/running cases, the dispatch borrow, the no-op guards (nothing failed / command running).
- **P4 Validate** — pure unit (REQ-001) + a LIVE DRIVE (fail → re-run via palette → green → markers clear); gate green [diff].
- **P5 Complete** — CHANGELOG + app_shell.md, AAR, close #292.
