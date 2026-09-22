---
pipeline_id: 01a188ae-6b4d-4e38-abfd-92343b193164
ticket: forge#175 (ad4f71a6-e985-4313-bb09-464811148316) · local docs/planning/tickets/open/TICKET-175-block-menu.md
aar_id: e21aca98-eb76-453b-b3a8-8576129d75c8
status: Phase 5 — Complete PASS
title: M11 — the block context menu (right-click a block → Copy cmd / Copy out / Rerun + split)
type: feature
milestone: M11 — Live everywhere + Warp blocks
references:
  - crates/marley_app/src/nav.rs (PURE: block_at_row)
  - crates/marley_app/src/context_menu.rs (PURE: MenuKind + per-kind item tables)
  - crates/marley_app/src/app.rs (SHIM: the right-click hit-test, the shared action helpers, the menu run)
---

## Title
Right-clicking INSIDE a command block opens a unified context menu — Copy command, Copy output, Rerun,
then the split actions — targeting THAT block; right-clicking elsewhere keeps the split-only menu. The
three block actions already exist as tiny header buttons (R50) — they get extracted into shared helpers,
NOT duplicated.

## Scope
### In
- PURE `nav.rs`: `block_at_row(output_line_counts, row) -> Option<usize>` (the block whose
  `[header, header+1+outputs)` range holds the row; the prompt/past-end → None).
- PURE `context_menu.rs`: `MenuKind { Split, Block }` on the state; `items_for(kind)` (3 split rows vs
  Copy command/Copy output/Rerun + the 3); MenuAction grows the 3 block variants; wrap/action over the
  kind's table.
- SHIM: the terminal right-click computes the content row (the selection math: pane_grid_pos + the
  viewport start) → block_at_row → a Block-kind menu carrying (pane, block_index), else Split-kind;
  `copy_block_text` / `rerun_block` helpers extracted from the header buttons and shared by both routes;
  the render sizes by `items_for` len.

### Out
- New block actions (delete/share); per-block hover menus; touching the header buttons' layout.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `block_at_row` shall map header + output rows to their block and the prompt/past-end to None (boundary mutants killed). | unit |
| REQ-002 | `items_for` shall yield the split table for Split and the 6-row table for Block; the selection shall wrap over the ACTIVE kind's length. | unit |
| REQ-003 (visual) | WHEN right-clicking a block's rows, the menu shall list Copy command/Copy output/Rerun + the split items; "Copy command" then ⌘V shall reproduce the command on the prompt. | driven capture |
| REQ-004 (visual) | Rerun shall re-execute the block's command (a fresh block appears). | driven capture |
| REQ-005 | gate GREEN; the pure fns cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 implement. P3.5 1 critic (row math vs the render walk incl. scroll; the shared-helper borrow
shapes; menu exclusivity #96). P4 tests + driven + gate. P5 docs.
