---
pipeline_id: 47fcaaeb-44fd-4ee7-96cc-0144cc64ba4b
ticket: forge#111 (f10d1452-200b-4efb-9480-01cf31197816) · local docs/planning/tickets/open/TICKET-111-session-row-chrome.md
aar_id: 3e93a25f-f0a2-4a43-b665-7a29ae662e59
status: Phase 5 — Complete PASS
title: session row chrome (icon/title/subtitle)
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/sessions.rs (PURE: session_row_icon; Session.running)
  - crates/marley_app/src/app.rs (SHIM: two-line rows; running from AgentStatus)
---

## Title
Give each sidebar row the Warp chrome: an icon reflecting an agent's running state (active ✳ / idle ✧ /
terminal ▸) and a two-line layout — a title with a muted subtitle beneath.

## Scope
### In
- PURE `session_row_icon(is_agent, running)` (3 distinct glyphs) + `Session.running`; `session_rows` uses it.
- SHIM: `running` derived from the real `AgentStatus`; the row renders `[icon | title / subtitle]` (two lines).

### Out
- A pure `session_subtitle(branch/cwd)` — deferred (no branch data source yet; the "main" placeholder renders).
- A running spinner/animation. Per-command titles.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `session_row_icon` returns 3 distinct glyphs (agent-active ✳ / agent-idle ✧ / terminal ▸); the color
  is the shim's job.
- D2 — `running` = the agent's `AgentStatus` is `Working` or `Waiting` (alive + active); a terminal → false.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `session_row_icon(is_agent, running)` runs, it shall map the 3 states to 3 distinct glyphs. | unit |
| REQ-002 | WHEN `session_rows` builds a row, its icon shall come from `session_row_icon(is_agent, running)`. | unit |
| REQ-003 (visual) | WHEN a session row renders, it shall show the icon + a title + a muted subtitle line. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on session_row_icon + session_rows; the shim masked. | gate |

## Phase Plan
- **P2** — session_row_icon + Session.running; the two-line row + running-from-AgentStatus; test plan.
- **P3** — implement (sessions.rs + app.rs; update the test helpers).
- **P3.5** — 1 critic: session_row_icon MSI (3 arms); running flows from AgentStatus; the two-line row.
- **P4** — the icon + session_rows tests (cov/MSI 100) + a live capture + gate GREEN.
- **P5** — docs, AAR, archive, close #111.
