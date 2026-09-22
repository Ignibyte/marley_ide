---
pipeline_id: 8b62779a-bb92-4291-be37-e0678371ffd2
ticket: forge#109 (4015210d-112e-451b-8a0b-ae2035922d1b) · local docs/planning/tickets/open/TICKET-109-sessions-sidebar.md
aar_id: ad701083-ef81-4427-86b3-d571ac75b3a2
status: Phase 5 — Complete PASS
title: the sessions sidebar (persistent left)
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/sessions.rs (NEW PURE: Session, SessionRow, session_rows)
  - crates/marley_app/src/lib.rs (mod sessions)
  - crates/marley_app/src/app.rs (SHIM: the left-dock sessions list)
---

## Title
The Warp "active sessions on the left": a persistent sidebar listing every terminal/agent session, each a
clickable row that focuses its pane — promoting the ⌘⇧E Fleet overlay into always-on chrome.

## Scope
### In
- NEW pure `sessions.rs`: `Session` / `SessionRow` + `session_rows(sessions, focused)`.
- SHIM: the left dock shows a "Sessions" list (built from the workspace panes) above the Files tree; a row
  click focuses the pane; the active row is highlighted.

### Out
- Workspace grouping (seq-4). Rich icons/titles/branch subtitles (seq-5). The "Search tabs" filter (seq-6).
- Moving Files out of the left dock (seq-7) — Files stays below the sessions for now.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — a NEW `sessions.rs` module (seq-4/5/6 extend it) holds the pure surface.
- D2 — `session_rows` maps each `Session` → a `SessionRow` (icon by is_agent, active = id==focused, order kept).
- D3 — Files stays reachable (below the sessions) until seq-7 makes it a pane — no access gap.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `session_rows(sessions, focused)` runs, it shall map each session to a row (icon by kind, order kept). | unit |
| REQ-002 | WHEN a session's id equals `focused`, its row's `active` shall be true (else false). | unit |
| REQ-003 (visual) | WHEN the app renders, the left dock shall list the sessions; the focused row highlighted. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on session_rows; the shim masked. | gate |

## Phase Plan
- **P2** — Session/SessionRow + session_rows; the left-dock list + click-to-focus; test plan.
- **P3** — implement (sessions.rs + lib.rs + app.rs).
- **P3.5** — 1 critic: session_rows MSI (icon/active/order); the click focuses the right pane; Files reachable.
- **P4** — session_rows tests (cov/MSI 100) + a live capture + gate GREEN.
- **P5** — docs, AAR, archive, close #109.
