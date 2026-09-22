---
pipeline_id: 46014ad1-a327-4d8b-a470-03e58a52d6ce
ticket: forge#161 (7056f380-b37e-4cbc-b476-c66c7cc894ae) · local docs/planning/tickets/open/TICKET-161-tab-close.md
aar_id: 8ad2463e-a192-428f-b143-9af0a1e0460d
status: Phase 5 — Complete PASS
title: M10 — tab close (rail × + ⌘W tab semantics + the last-terminal guard)
type: feature
milestone: M10 — Warp polish + shell hardening
references:
  - crates/marley_app/src/tabs.rs (PURE: TabError::LastTerminal, the close_tab guard, Workspace::project_mut)
  - crates/marley_app/src/app.rs (SHIM: close_tab_at, the rail ×, the ⌘W branch)
---

## Title
Tabs can be closed — an × on each rail tab row, ⌘W closes the tab at its last pane (or a cockpit/code tab),
and closing the last terminal tab of a project is refused (absorbs #159's guard).

## Scope
### In
- PURE `tabs.rs`: `TabError::LastTerminal`; `close_tab` refuses removing the only TERMINAL tab;
  `Workspace::project_mut(idx)`.
- SHIM `app.rs`: `close_tab_at(project, tab)` (close + thread-drop reap + persist + a refusal flash);
  the rail Tab-row × affordance; the ⌘W "close-pane" branch (non-terminal active tab OR a 1-pane grid → close
  the TAB; else close the focused pane as today).

### Out
- Project close (the #162 ticket). The #159 vestigial right-dock cleanup (stays in #159). Undo/reopen a
  closed tab. Confirm-before-close.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — guard precedence: a single tab still errors `LastTab`; a multi-tab project closing its only terminal
  errors `LastTerminal` (checked before the remove). `workspace()`'s ≥1-terminal invariant stays enforced.
- D2 — ⌘W semantics (Warp-like): panes close first; the last pane closes the tab; a cockpit/code active tab
  closes immediately (it has no panes).
- D3 — the removed Tab is dropped on a spawned thread (the existing dead-pane reap pattern) so PTY teardown
  never blocks the UI frame.
- D4 — a refused close shows a status flash ("can't close the last terminal") — no silent no-op.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `close_tab` targets the only terminal tab of a multi-tab project, it shall return `Err(LastTerminal)` and leave the tabs unchanged; a cockpit/code tab in the same project shall close Ok. | unit |
| REQ-002 | WHEN a project has ≥2 terminal tabs, closing either shall succeed with the active index following (`adjust_active`); a single-tab project shall still return `Err(LastTab)`. | unit |
| REQ-003 | `Workspace::project_mut(idx)` shall return `Some` for a valid index and `None` past the end. | unit |
| REQ-004 (visual) | WHEN a rail tab row's × is clicked, that tab shall close (the rail shrinks, the center follows); WHEN the last terminal tab's × is clicked, a refusal flash shall show and nothing closes. | driven capture |
| REQ-005 (visual) | WHEN ⌘W is pressed with the active terminal tab at 1 pane, the TAB shall close; with >1 pane the focused PANE shall close (unchanged). | driven capture |
| REQ-006 | gate GREEN; the pure guard/accessor at cov/MSI 100; the shim masked. | gate |

## Phase Plan
- **P2** — the guard + project_mut; close_tab_at; the × render (bubbling note); the ⌘W branch; test plan.
- **P3** — implement (tabs.rs + app.rs).
- **P3.5** — 1-2 critics (guard precedence + invariant preservation; the × routing/bubbling + reap).
- **P4** — pure tests (cov/MSI 100) + driven captures (× closes; ⌘W tab-close; the refusal) + gate.
- **P5** — docs, AAR, close #161 + comment #159, archive.
