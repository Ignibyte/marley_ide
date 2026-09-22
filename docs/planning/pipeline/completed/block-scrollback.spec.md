---
pipeline_id: 6ee06cf1-9ffd-4576-a25b-43375bac1c54
ticket: forge#52 (bd410f09-cc72-4bb4-860f-4a23cea5fcb3) · local docs/planning/tickets/open/TICKET-052-block-scrollback.md
aar_id: 5b95f7a9-f442-4085-974a-ff0f49c7a764
status: Phase 5 — Complete PASS
title: capture full command output beyond the grid (block scrollback)
type: bug
milestone: Terminal Polish
references:
  - crates/terminal_blocks/src/session.rs (full_term_to_styled_rows + the Precmd full-capture in ingest)
---

## Title
A command whose output exceeds the screen height keeps only the LAST screen in its block (the earlier
lines scroll off the visible grid). alacritty already keeps them in history (Config `scrolling_history:
10000`) — read the FULL history+screen into the FINISHED block.

## Scope
### In
- `session.rs`: `full_term_to_styled_rows(term)` reading `-(history_size)..screen_lines`; the ingest, on
  the `Precmd` hook (before the block closes), captures the trimmed full output into the finished block.

### Out
- Reading history LIVE per-chunk while a command runs (a running block keeps the visible-screen snapshot —
  bounded, no quadratic re-read). alt-screen `grid_styled_rows` (unchanged). Raising the 10000 history cap.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the FULL capture happens ONCE, at Precmd (command finish), NOT per-Passthrough → bounded (no
  quadratic O(history×chunks) re-snapshot). A running block shows the live visible screen (unchanged).
- D2 — `full_term_to_styled_rows` reads the grid's history region (negative Line indices) + the visible
  screen: `-(grid.history_size() as i32)..(grid.screen_lines() as i32)`.
- D3 — the Precmd full-capture is applied BEFORE `apply_hook(Precmd)` closes the block (set_current_output
  writes the RUNNING block); trailing blanks trimmed (#50).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a command's output exceeds screen_lines and finishes, the finished block's `output_styled` shall contain ALL the output rows (incl. scrolled-off), trailing blanks trimmed. | integration |
| REQ-002 | WHEN a command's output fits the screen, the block's output shall be unchanged (no regression). | integration |
| REQ-003 | `scripts/gates.sh` GREEN, cov/MSI 100 on the new fn + the Precmd branch. | gate |

## Phase Plan
- **P2** — `full_term_to_styled_rows` (the history range) + the Precmd capture in ingest; mutation targets;
  the ingest integration test plan.
- **P3** — implement (session.rs).
- **P3.5** — 1 critic: the -history..screen range (off-by-one), the Precmd capture BEFORE close, the live
  path unchanged, no per-chunk full read, cov/MSI on the fn.
- **P4** — the ingest test (feed >screen_lines rows + Precmd → all rows present) + gate GREEN (cov/MSI 100).
- **P5** — docs, AAR, archive, close #52 (**closes M1.H's last open ticket alongside #51**).
