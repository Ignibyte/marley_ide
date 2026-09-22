---
pipeline_id: d7715cf1-d3ca-4311-bc0f-691c5eb0a15c
ticket: forge#50 (9cdc30e8-2f65-4a07-bbf1-fdc55b8ae936) · local docs/planning/tickets/open/TICKET-050-block-output-trim.md
aar_id: d60e56c1-ea66-4078-8a9c-0e6104a2e596
status: Phase 5 — Complete PASS
title: block output trims trailing blank rows (history stacks)
type: bug
milestone: M1.H
references:
  - crates/terminal_blocks/src/styled.rs (NEW trim_trailing_blank_rows)
  - crates/terminal_blocks/src/session.rs (apply it to the block-output path)
  - docs/specs/SPEC-terminal-blocks.spec.md (R19 amend)
---

## Title
CRITICAL (chad live testing + window capture + code read): "every command clears the previous" — command
blocks don't stack as scrollback. Each block captures the FULL screen-height grid (incl. trailing blank
rows), so one short command is a full-screen block that fills the viewport and shoves the previous off.

## Scope
### In
- `crates/terminal_blocks/src/styled.rs` (PURE — cov/MSI 100): `trim_trailing_blank_rows(rows:
  Vec<StyledLine>) -> Vec<StyledLine>` — drop trailing rows whose text is all trim-empty (`row.iter()
  .all(|r| r.text.trim().is_empty())`); stop at the first non-blank row from the end. A no-output
  command → `[]`; an all-blank grid → `[]`; interior blank rows are KEPT (only trailing trimmed).
- `crates/terminal_blocks/src/session.rs` — in `ingest`, wrap the block-output set:
  `self.model.set_current_output(trim_trailing_blank_rows(term_to_styled_rows(&self.term)))`.
- SPEC-terminal-blocks R19 note (block output is trailing-blank-row-trimmed). CHANGELOG + arch.

### Out (explicitly deferred)
- Any change to `grid_styled_rows()` (the alt-screen render) — it KEEPS the full untrimmed grid (a
  full-screen program owns its whole screen). Long-command output that exceeds the grid (only the last
  screen is captured) — a separate scrollback-capture concern, not this ticket. Tab-completion (#33).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — Trim TRAILING blank rows only (not interior): a command whose output has a blank line in the
  MIDDLE keeps it; only the empty grid rows BELOW the last real output line are dropped.
- D2 — Apply ONLY to the block-output path (`set_current_output`), NOT to `grid_styled_rows()`
  (alt-screen). A full-screen program (vim/top) fills its grid; its rows must not be trimmed.
- D3 — A blank row = every run's `text.trim()` is empty (covers no-runs, empty runs, whitespace-only —
  `coalesce_row` already trims trailing spaces, so a truly blank grid line coalesces to empty/whitespace).
- D4 — PURE surface = `trim_trailing_blank_rows` (cov/MSI 100). The `ingest` wiring is the session
  passthrough path.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `trim_trailing_blank_rows(rows)` is called, it shall drop every trailing all-trim-empty row and stop at the first non-blank from the end; interior blank rows shall be kept. | unit (trailing/interior/none) |
| REQ-002 | WHEN the input is all blank (or empty), it shall return `[]`; WHEN there are no trailing blanks, it shall return the input unchanged. | unit (all-blank; no-trailing) |
| REQ-003 | WHEN a command finishes, its block's `output_styled()` shall have NO trailing blank rows (only the real output lines) — so blocks are only as tall as their output and stack as scrollback. | session unit test (a command → block output = the real lines, no trailing blanks) |
| REQ-004 | WHEN a full-screen program is active, `grid_styled_rows()` shall still return the FULL grid (unaffected by the trim). | build + reasoning (separate path) |
| REQ-005 | WHEN `scripts/gates.sh` runs, every gate shall be GREEN with cov/MSI 100 on `trim_trailing_blank_rows`; a rebuild + window capture shall show two commands STACKED as separate blocks (chad-typed, I capture). | gate + capture |

## Phase Plan
- **P2 Design** — the exact `trim_trailing_blank_rows` + the `ingest` wiring; confirm alt-screen
  untouched; the SPEC note; the session test approach.
- **P3 Implement** — the pure fn + the `ingest` one-line wrap + spec + CHANGELOG.
- **P3.5 Inspect** — critic: the trailing-vs-interior distinction, the blank predicate (whitespace/
  empty runs), the alt-screen path stays full, no regression to output_text/the existing block tests.
- **P4 Validate** — the trim tests + the session test + gate GREEN + rebuild + capture (stacked blocks).
- **P5 Complete** — docs, AAR, archive, close #50.
