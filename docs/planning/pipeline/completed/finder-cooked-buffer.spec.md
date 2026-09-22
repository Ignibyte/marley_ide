---
pipeline_id: 0175a5c2-9126-4fe2-8858-ea1975ab5a2a
ticket: forge#65 (e410d50c-a919-4544-b75c-50af2feb865e) · local docs/planning/tickets/open/TICKET-065-finder-cooked-buffer.md
aar_id: fc267cda-016c-4d30-bd64-8c46542eaad4
status: Phase 5 — Complete PASS
title: cmd-P finder Enter inserts into the cooked buffer
type: bug
milestone: M2.C
references:
  - crates/marley_app/src/app.rs (SHIM: handle_finder_key Enter → buffer.edit, not write_bytes)
---

## Title
Fix the #57 cmd-P finder's Enter: it `write_bytes` the chosen path to the PTY — invisible in cooked mode
(the #59 bug) — instead of inserting into Marley's rendered prompt buffer. Route it through `buffer.edit`
(mirroring #59's file-click). Closes the visual-verification gap #57 couldn't reach.

## Scope
### In (SHIM only, `app.rs` `handle_finder_key` — mutants::skip + cov-excluded; NO new pure surface)
- Replace the finder Enter's `write_bytes(chosen_path)` with `state.buffer.edit(caret..caret, path,
  EditOrigin::Human)` + `state.caret` advance — the #59 cooked-buffer insert (per
  PR-claude-insert-at-prompt-goes-to-cooked-buffer-not-raw-pty-001).

### Out
- Any new pure logic (reuses the marley_editor `buffer.edit`/`CharOffset`, already tested). The finder's
  ranking/selection (unchanged). Run-on-Enter (still just inserts, like #59/#60).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — insert into the cooked buffer (`buffer.edit`), NOT `write_bytes` — the #59 lesson: a raw PTY write
  reaches the shell's ZLE but not Marley's rendered line; in cooked mode Marley owns the line editor.
- D2 — mirror #59's file-click (app.rs ~1058) + the cmd-V paste verbatim (the established pattern).
- D3 — no new unit tests (SHIM-only); the SELF-TEST is the proof — and #65 finally makes the finder Enter
  self-test-verifiable (Enter is a keycode; #57's type-to-filter needed key_char which the harness can't drive).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 (visual) | WHEN a file is chosen in the cmd-P finder and Enter is pressed, its path shall APPEAR at the prompt (a visible cooked-buffer insert), not be written invisibly to the PTY. | self-test (cmd-P → Enter → the path shows) |
| REQ-002 | `scripts/gates.sh` GREEN (no regression); the app shim is masked (no new pure lines). | gate |

## Phase Plan
- **P2** — the one-block Enter replacement (write_bytes → buffer.edit), the test plan (no unit; self-test).
- **P3** — the app.rs handle_finder_key Enter change.
- **P3.5** — critic: the insert mirrors #59 (buffer.edit + caret advance), no write_bytes remains, no panic,
  all-masked.
- **P4** — `cargo nextest` (no regression) + the SELF-TEST (cmd-P → Enter → the visible path) + gate GREEN.
- **P5** — docs, AAR, archive, close #65 — **M2.C COMPLETE**.
