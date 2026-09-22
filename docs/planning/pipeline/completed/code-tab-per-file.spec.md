---
pipeline_id: 390c1f46-6617-4e82-8deb-638e48fbfaa6
ticket: forge#164 (72e7d16f-9565-415f-87b0-a4b419dcc2d4) · local docs/planning/tickets/open/TICKET-164-code-tab-per-file.md
aar_id: 320ef2f6-95d2-4d51-abcd-b3937f6204ae
status: Phase 5 — Complete PASS
title: M10 — a code tab per file (dedup by path)
type: feature
milestone: M10 — Warp polish + shell hardening
references:
  - crates/marley_app/src/tabs.rs (PURE: the open_or_switch_code path predicate + tests)
---

## Title
Each file opens its own code tab — re-opening a file switches to (and refreshes) its existing tab; a new file
appends a new tab. Replaces #154's single-reused-tab decision (tab close #161 makes per-file tabs cleanable).

## Scope
### In — PURE `tabs.rs` only
- `open_or_switch_code` matches `code_view().is_some_and(|cv| cv.path == state.path)`; the matched arm still
  replaces (a fresh read refreshes stale content) + switches; the else arm appends.

### Out
- Any shim change (open_file_in_viewer already routes here). An unbounded-tabs cap. Dirty-file indicators.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — re-open REFRESHES (replace the tab's state with the fresh read) — the old reuse semantic, now per-file.
- D2 — no tab cap: chad closes via × / ⌘W (#161).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a code tab for `state.path` exists, `open_or_switch_code` shall replace its content and switch to it WITHOUT changing the tab count. | unit |
| REQ-002 | WHEN no code tab for that path exists, it shall append a new code tab (title = file name) and activate it — existing code tabs untouched. | unit |
| REQ-003 (visual) | WHEN two different files are opened from the Files panel, the rail shall show two code tabs; re-opening the first shall not add a third. | driven capture |
| REQ-004 | gate GREEN; the predicate branch mutation-killed (path-eq vs any-code-tab). | gate |

## Phase Plan
P2 folded (a one-predicate change + tests). P3 implement. P3.5 self-review + the mutation gate as the
adversary (a critic on a 1-line pure diff adds nothing beyond gate:5). P4 tests + driven + gate. P5 docs/AAR/archive.
