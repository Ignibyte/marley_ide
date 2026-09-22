---
pipeline_id: 1bd20d2e-5d62-46c2-8b9f-eace2ea0fc6c
ticket: forge#150 (ee02dfec-81e3-4549-83d3-abba18cf2cc1) · local docs/planning/tickets/open/TICKET-150-workspace-model.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: M9 seq-1 — pure Workspace/Project/Tab model + algebra
type: feature
milestone: M9 — Workspace / Project / Tab model
references:
  - crates/marley_app/src/workspace.rs (RENAME Workspace<S> → PaneGrid<S>; behavior unchanged)
  - crates/marley_app/src/tabs.rs (NEW, PURE: Workspace/Project/Tab/TabContent + algebra)
  - crates/marley_app/src/app.rs (mechanical: the rename's call sites; NO behavior change)
---

## Title
The foundation of the tab model — a pure `Workspace → Project → Tab` hierarchy with a fully-tested algebra,
where a terminal tab owns the (renamed) pane grid so all of M6's tiling survives inside it.

## Scope
### In
- NEW pure module (`tabs.rs`): `Workspace<S>`, `Project<S>`, `Tab<S>`, `TabContent<S>` + the add/close/switch
  algebra + invariants + a typed error.
- RENAME the existing `Workspace<S>` (the pane grid) → `PaneGrid<S>` (mechanical; behavior identical).

### Out
- ANY render / gpui wiring (seq-2/#151). Wiring the app to *use* the new top-level (seq-2). CodeView tab
  variant (seq-5). Cockpit-tab rendering (seq-4). Persistence of the hierarchy.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — **Naming:** the existing tiled-grid `Workspace<S>` is renamed `PaneGrid<S>` (it IS a grid of panes now);
  the NEW top-level container takes the name `Workspace<S>`. `TabContent::Terminal(PaneGrid<S>)`.
- D2 — **Hierarchy (chad 2026-07-07):** `Workspace` is a MULTI-PROJECT container; `Project` owns the repo root
  + tabs; a `Tab` is `Terminal(PaneGrid<S>)` or `Cockpit(RightSection)` (CodeView added seq-5).
- D3 — **Invariants (mirror M6 R28–R30):** a workspace always has ≥1 project; a project always has ≥1 tab;
  `active_project`/`active_tab` are always valid indices. Closing the last project/tab returns a typed error
  (never empties a level). Closing the ACTIVE item moves active to its predecessor (successor if it was
  first) — the M6 `close_pane` focus rule. `switch_*(idx)` out of range returns a typed error.
- D4 — generic over the session handle `S`, threaded through `TabContent::Terminal`.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `add_tab` appends a tab, the new tab shall become active; `add_project` likewise. | unit |
| REQ-002 | WHEN the ACTIVE tab is closed (not first), `active_tab` shall move to its predecessor; the same for projects. | unit |
| REQ-003 | WHEN the FIRST (active) tab is closed, `active_tab` shall move to the new first (old successor). | unit |
| REQ-004 | WHEN the LAST tab of a project is closed, it shall return `LastTab` and leave the project unchanged; the last project → `LastProject`. | unit |
| REQ-005 | WHEN `switch_project`/`switch_tab` is given an out-of-range index, it shall return `IndexOutOfRange` and not change active. | unit |
| REQ-006 | WHEN a Terminal tab is built, it shall wrap a `PaneGrid<S>` and expose it (so seq-2 renders/operates it). | unit |
| REQ-007 | The existing pane-grid behavior (split/close/resize/focus) shall be unchanged after the rename (all prior tests pass). | regression |
| REQ-008 | gate GREEN, cov/MSI 100 on the new algebra; the app.rs rename call-sites masked/unchanged. | gate |

## Phase Plan
- **P2** — the module layout, the exact types + error enum, the rename mechanics (seq-1 vs deferred), test plan.
- **P3** — implement (rename + `tabs.rs`).
- **P3.5** — 2 critics (correctness of the active-follows-close + invariants; the rename touches nothing else).
- **P4** — the algebra tests (cov/MSI 100) + the full existing suite green after the rename + gate GREEN.
- **P5** — docs, AAR, archive, close #150.
