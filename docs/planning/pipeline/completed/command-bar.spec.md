---
pipeline_id: 6bceb06b-acaf-4f5e-931f-b8c026ed451e
ticket: forge#117 (afc03377-f5ab-4391-94f4-61904d2f7ed9) · local docs/planning/tickets/open/TICKET-117-command-bar.md
aar_id: 5df08bc9-289d-4070-abef-99ee9a7da0c1
status: Phase 5 — Complete PASS
title: the global top command bar
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/command_bar.rs (NEW PURE: HitKind, SearchHit, search_everything)
  - crates/marley_app/src/lib.rs (mod command_bar)
  - crates/marley_app/src/app.rs (SHIM: the top bar + the mixed-results dropdown)
---

## Title
The Warp top bar: one "Search sessions, agents, files…" field that fuzzy-searches across sessions, files,
and palette actions at once — kind-tagged, ranked, capped.

## Scope
### In
- PURE `search_everything(query, sessions, files, actions, cap)` → `Vec<SearchHit{kind, label}>`.
- SHIM: a top-bar strip + a mixed-results dropdown; a file hit opens the viewer.

### Out
- Ranking across sources (order = sessions, then files, then actions; the matcher orders within a source).
- Full routing of session/action hits (files route; session/action are best-effort — noted).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `search_everything` fuzzy-matches each source in order (sessions → files → actions), kind-tagging each
  hit, then truncates to `cap`; empty query → no hits.
- D2 — the top-bar input mirrors the #112 session-search focus + key routing.
- D3 — file hits route (open the viewer); session/action routing is best-effort v1.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `search_everything` runs a non-empty query, it shall return kind-tagged hits from all 3 sources, capped. | unit |
| REQ-002 | WHEN the query is empty, `search_everything` shall return no hits. | unit |
| REQ-003 (visual) | WHEN the app renders, a "Search sessions, agents, files…" bar shall show atop the window. | live capture |
| REQ-004 | gate GREEN, cov/MSI 100 on search_everything; the shim masked. | gate |

## Phase Plan
- **P2** — HitKind/SearchHit + search_everything; the top-bar + dropdown; test plan.
- **P3** — implement (command_bar.rs + lib.rs + app.rs).
- **P3.5** — 1 critic: search_everything MSI (empty/per-source/kind/cap); the input + file route.
- **P4** — search_everything tests (cov/MSI 100) + a live capture + gate GREEN.
- **P5** — docs, AAR, archive, close #117.
