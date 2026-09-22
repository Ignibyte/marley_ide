---
pipeline_id: c9994f01-f083-42cb-a8fe-7afdfcbb723e
ticket: forge#213 (6c23355e-5688-404b-88e5-05fefa772aa9) · local docs/planning/tickets/open/TICKET-213-jump-to-failure.md
aar_id: ea72f711-876f-44db-803f-98b07dd18c30
status: Phase 5 — Complete PASS
title: A failing command → a "Jump to Failure" block action → open the editor at the failing line
type: feature
milestone: M18
references: [docs/planning/intake/editor-as-peer-and-terminal-fusion.md]
---

## Title
When a command block fails and its output references a source location, offer a
"Jump to Failure" action on the block that opens the editor at the primary failure
line — the run→fix loop, reusing #212's parser + open-at-line.

## Scope
### In
- `links.rs` (pure): `first_failure_ref(output: &str) -> Option<(PathBuf, usize, Option<usize>)>` —
  scan output lines with the #212 `scan_links`, return the FIRST `File{line: Some, ..}` (the primary
  failure location).
- `context_menu.rs` (pure): `MenuAction::JumpToFailure`; a 7-row `BLOCK_MENU_ITEMS_FAILED`
  (Jump-to-Failure FIRST, then the 6 existing); `MenuKind::Block` gains `has_failure: bool`;
  `items_for` returns the failed table when `has_failure`.
- `app.rs` (shim): opening a block menu computes `has_failure = block-failed && first_failure_ref(output).is_some()`;
  dispatching `JumpToFailure` → `first_failure_ref` → `open_file_at`.

### Out (explicitly deferred)
- The diagnostics gutter (#289), next/prev nav (#290), multi-frame traces (#291) — separate M18 tickets.
- An inline (non-menu) block affordance / a keybinding for jump-to-failure — the menu row is v1.
- Ranking failures beyond "first line-carrying ref" (a scored primary-error heuristic is a follow-up).

## Reference (§20)
Zed / an IDE's "jump to first error" (click a failed build → the editor opens at
the error). Marley matches by parsing the failed block's output and opening at the
primary ref. Clean-room §20: the ref-pick + menu-row logic are Marley's own
(reusing #212); no Zed source read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — "primary failure" = the FIRST output line carrying a `file:line` ref.**
  Compilers/tests emit the primary error first; a scored heuristic (error vs
  warning vs note) is a follow-up. `first_failure_ref` returns the first
  `scan_links` File with `line.is_some()` (a bare path with no line is not a jump target).
- **D2 — the row is CONDITIONAL** on `has_failure` (block failed AND a ref exists),
  carried on `MenuKind::Block { pane, block, has_failure }`. A succeeding block, or
  a failed block with no ref, shows the unchanged 6-row menu (no dead/disabled row).
- **D3 — "failed" = `exit_status_kind` is the danger/failure kind** (a finished
  block with a non-zero exit). Pending/Running never show the row.
- **D4 — reuse #212 end to end** — `scan_links` for the ref, `open_file_at` for the
  open+caret. No new parsing or open logic.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN output contains a `file:line` ref, `first_failure_ref` shall return the FIRST line-carrying ref (path, line, col?). | links.rs unit: multi-line output (a note line, then `src/a.rs:12:5: error`) → the a.rs ref; a bare-path-only output → None; empty → None. |
| REQ-002 | WHERE a block has failed AND has a failure ref, `items_for` shall include a "Jump to Failure" row (first); else the 6-row menu. | context_menu.rs unit: `Block{has_failure:true}` → 7 rows incl JumpToFailure; `false` → the 6-row table; `Split` → the split table. |
| REQ-003 | WHEN "Jump to Failure" is chosen, the editor shall open the file at the primary ref's line. | Driven capture (a failing `cargo`/`grep`, right-click → Jump to Failure → editor at the line); mechanism otherwise (reuses #212 `open_file_at`, unit-proven). |
| REQ-004 | `first_failure_ref` + `items_for` shall be pure at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — `first_failure_ref` + the `JumpToFailure`/`BLOCK_MENU_ITEMS_FAILED`/`has_failure` menu changes + the app.rs open + dispatch; the test plan.
- **P3 Implement** — the pure fns + the menu wiring + the shim.
- **P3.5 Inspect** — the primary-ref pick, the conditional row, the failed-detection, the MenuKind consumers.
- **P4 Validate** — `first_failure_ref` fixtures + `items_for` branches + a driven jump; gate green [diff].
- **P5 Complete** — CHANGELOG + app_shell doc, AAR, close #213.
