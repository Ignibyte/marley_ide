---
pipeline_id: 6f8b83cb-88ef-4412-8a97-be79916538c8
ticket: forge#139 (9736b3a8-268f-4b84-9f20-e9e1c99b306e) · local docs/planning/tickets/open/TICKET-139-pane-placement.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: deterministic file/pane placement (open to the right) [M8]
type: feature
milestone: M8 — Warp Chrome & Fidelity
references:
  - crates/marley_app/src/workspace.rs (PURE: insert_split_at, rightmost_pane, open_pane_rightmost)
  - crates/marley_app/src/app.rs (SHIM: open_kind_pane/open_code_pane route through open_pane_rightmost)
---

## Title
Files, code, and git panes always open on the right — consistent, not wherever the focus happens to be.

## Scope
### In
- PURE: extract `insert_split_at(target, …)`; `rightmost_pane()` (DFS-last leaf); `open_pane_rightmost(…)`.
- SHIM: the non-terminal openers (files/code/git) route their OpenNew through `open_pane_rightmost`.

### Out
- Terminal placement (unchanged — `split_focused`). Re-ordering existing panes. Drag-reorder.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `rightmost_pane` = `*group.panes().last().unwrap_or(&focused)` (DFS-last leaf; no `expect` — the
  fallback covers the impossible-empty case). `open_pane_rightmost` splits it (After) so the new pane lands
  rightmost.
- D2 — `insert_focused_split` and `open_pane_rightmost` share `insert_split_at(target, …)`; the focused-split
  behavior (terminals, `⌘D`) is unchanged.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `rightmost_pane` runs on a `[T,F,C]` grid, it shall return the DFS-last leaf (C). | unit |
| REQ-002 | WHEN `open_pane_rightmost` opens with a NON-rightmost pane focused, the new pane shall land LAST (rightmost). | unit |
| REQ-003 (visual) | WHEN a file/git pane is opened with a terminal focused, it shall appear to the right of the terminals. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on rightmost_pane + open_pane_rightmost; the openers masked. | gate |

## Phase Plan
- **P2** — insert_split_at extraction; rightmost_pane; open_pane_rightmost; the opener routing; test plan.
- **P3** — implement (workspace.rs + app.rs).
- **P3.5** — 1 self-review: the refactor keeps focused-split; rightmost picks last; the openers route right.
- **P4** — rightmost_pane + open_pane_rightmost tests (cov/MSI 100) + a LIVE capture + gate GREEN.
- **P5** — docs, AAR, archive, close #139.
