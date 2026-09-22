---
pipeline_id: 7b26e83a-e0ce-4b96-af24-05dd395cdcb3
ticket: forge#112 (f883c3c8-09c2-42c4-a9af-8dbe417e9f34) · local docs/planning/tickets/open/TICKET-112-search-tabs.md
aar_id: cb33804b-e8f9-405d-b730-450607d10916
status: Phase 5 — Complete PASS
title: "Search tabs" sidebar filter
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/sessions.rs (PURE: filter_sessions)
  - crates/marley_app/src/app.rs (SHIM: the search box + key routing + filter-before-group)
---

## Title
A "Search tabs…" field atop the sessions sidebar that fuzzy-filters the sessions as you type (reusing the
shared matcher).

## Scope
### In
- PURE `filter_sessions(sessions, query)` — fuzzy match on title/subtitle, order preserved, empty → all.
- SHIM: a `session_filter` input box + focus/key routing; the filter applies before grouping.

### Out
- Re-ranking by score (order preserved to keep grouping stable). Highlighting the matched chars.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `filter_sessions` keeps input order (no re-rank) so groups stay stable; matches on title OR subtitle
  via `marley_search_core::fuzzy_score`.
- D2 — the search input mirrors the #51 find-bar pattern (click to focus, char/backspace/esc).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the query is empty, `filter_sessions` shall return all sessions (order kept). | unit |
| REQ-002 | WHEN a query is given, `filter_sessions` shall keep only sessions whose title or subtitle matches. | unit |
| REQ-003 (visual) | WHEN the sidebar renders, a "Search tabs" field shall show atop the session list. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on filter_sessions; the shim masked. | gate |

## Phase Plan
- **P2** — filter_sessions; the search box + key routing + filter-before-group; test plan.
- **P3** — implement (sessions.rs + app.rs).
- **P3.5** — 1 critic: filter_sessions MSI (empty/predicate/OR); the input routing; filter-before-group.
- **P4** — filter_sessions tests (cov/MSI 100) + a live capture + gate GREEN.
- **P5** — docs, AAR, archive, close #112.
