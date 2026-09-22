---
pipeline_id: 2b30ecf1-99df-4ee6-822a-80958dccadd9
ticket: forge#76 (4fdd974f-20c7-40bc-bd3e-a315afd7fcad) · local docs/planning/tickets/open/TICKET-076-comment-from-app.md
aar_id: 695c166c-f900-4b4e-b599-737be9c63f63
status: Phase 5 — Complete PASS
title: comment on a ticket from the app
type: feature
milestone: M2.D
security_sensitive: true
references:
  - crates/marley_app/src/forge_view.rs (PURE: prepare_comment)
  - crates/marley_app/src/app.rs (SHIM: comment_focused_on + the shift-cmd-click branch)
  - scripts/selftest/drive.swift (HARNESS: cmdshiftclick)
---

## Title
Compose a comment at the prompt, then shift+cmd+left-click a forge ticket to POST it — a forge write from
the cockpit (the third modifier on the row: plain=copy #70, cmd=claim #75, shift+cmd=comment #76).

## Scope
### In
- PURE (`forge_view.rs`, cov/MSI 100): `prepare_comment(text) -> Option<String>` (trim; None if blank).
- SHIM (app.rs, masked): `comment_focused_on(id)` — the focused prompt line → prepare_comment → (if Some +
  client) a BG-thread `comment_ticket` + re-fetch (#69) + clear the prompt; the forge-row handler's third
  branch (shift+cmd).
- HARNESS: a `cmdshiftclick:fx,fy` verb (cmd+shift left-click).

### Out
- A dedicated comment-input overlay (compose at the prompt — the #72 drivable pattern). Editing/deleting a
  comment (forge is append-only). author_id attribution (#74's comment omits it — a later option).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — compose at the PROMPT (drivable, the #72 pattern), NOT a key_char overlay (not self-testable).
- D2 — shift+cmd+left-click posts (distinct from cmd=claim #75, plain=copy #70) — a deliberate,
  unambiguous write gesture.
- D3 — `prepare_comment` blocks a blank comment (trim → None if empty).
- D4 — clear-on-valid-attempt: the async comment can't sync-confirm delivery (unlike #72's write_bytes),
  so the prompt clears when the body is valid + the client exists (a failed local-forge comment is rare).
- D5 — SECURITY: reuses #74's `comment_ticket` (closed write set, localhost, bearer never logged).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `prepare_comment(text)` is called, it shall return `Some(trimmed)` for non-blank text and `None` for blank/whitespace. | unit |
| REQ-002 (visual/security) | WHEN a comment is composed and a ticket is shift+cmd+clicked, that comment shall be posted to the ticket (and a plain/cmd click does NOT comment). | self-test (raw forge read) |
| REQ-003 (security) | The comment shall reuse the #74 closed write set (localhost, no arbitrary tool, bearer never logged). | review |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on prepare_comment; the app shim masked. | gate |

## Phase Plan
- **P2** — prepare_comment + comment_focused_on + the shift-cmd-click branch + the cmdshiftclick verb,
  mutation targets, test plan.
- **P3** — implement the pure + the shim + the harness verb.
- **P3.5** — 2 critics (correctness + SECURITY): prepare_comment MSI; the modifier order (shift+cmd ≠
  cmd ≠ plain — no cross-fire); the clear-on-attempt; bearer/closed-set inheritance.
- **P4** — prepare_comment tests (cov/MSI 100) + the SELF-TEST (compose → shift-cmd-click → forge read
  shows the comment) + gate GREEN.
- **P5** — docs, AAR, archive, close #76.
