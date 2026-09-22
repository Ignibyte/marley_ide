---
pipeline_id: bee6eac2-7859-4d81-860b-2573956f9b79
ticket: forge#71 (1743b2b3-91e7-4fb8-9652-42f0821a08f3) · local docs/planning/tickets/open/TICKET-071-insert-guard.md
aar_id: fda397ee-2a23-4a1c-bbe6-645b34f68c7c
status: Phase 5 — Complete PASS
title: guard finder/file-click prompt-inserts on is_command_running
type: bug
milestone: Terminal Polish
references:
  - crates/marley_app/src/app.rs (SHIM: the finder-Enter #65 + file-tree-click #59 inserts)
---

## Title
The cmd-P finder Enter (#65) and the file-tree click (#59) insert the path UNCONDITIONALLY into the cooked
prompt buffer; while a foreground command runs, that lands in the invisible cooked buffer instead of the
program. Mirror the paste guard (#42).

## Scope
### In
- Both insert sites (app.rs finder-Enter ~830 + file-tree click ~1540): guard on
  `state.session.is_command_running()` — running → `write_bytes(text.as_bytes())` (reaches the program);
  else the existing `buffer.edit` + caret advance.

### Out
- Any pure-logic change (SHIM-only). Changing WHAT the finder/tree pick (unchanged). Bracketed-paste
  wrapping of the path (a plain write, like a typed path).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — mirror the PROVEN #42 paste split exactly: while a command runs, the picked path goes to the PTY
  (`write_bytes`), not Marley's cooked buffer; at the prompt, the cooked insert is unchanged.
- D2 — on the write path there is NO caret advance (the caret is Marley's cooked-buffer cursor; the
  running program owns its own).
- D3 — SHIM-only (app.rs, cov-excluded + masked) — no pure surface, no new unit tests.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a file is picked (finder Enter / tree click) at the PROMPT, its path shall be inserted into the cooked buffer (unchanged). | self-test/structural |
| REQ-002 | WHEN a file is picked WHILE a command runs, its path shall be written to the PTY (reaching the program), not the cooked buffer. | self-test (env-blocked → structural) |
| REQ-003 | `scripts/gates.sh` GREEN (the change is masked/cov-excluded). | gate |

## Phase Plan
- **P2** — the exact guarded blocks at both sites; no pure surface.
- **P3** — implement (2 edits).
- **P3.5** — self-review: the guard mirrors #42 at both sites; write_bytes vs buffer.edit; no caret advance on write.
- **P4** — gate GREEN + the self-test (env-blocked → structural: the guard mirrors the proven #42 paste split).
- **P5** — docs, AAR, archive, close #71.
