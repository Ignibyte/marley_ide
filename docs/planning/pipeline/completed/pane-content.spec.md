---
pipeline_id: 6e2aed87-5528-47e7-94bc-609bcde7e830
ticket: forge#120 (ac00c9c1-3da7-4053-92a6-911fbdc340bb) · local docs/planning/tickets/open/TICKET-120-pane-content.md
aar_id: 4719c07e-3767-42f0-b74b-46dc231a0c8d
status: Phase 5 — Complete PASS
title: PaneContent — typed pane content (M6 FOUNDATION)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/workspace.rs (PURE: TerminalPane, PaneContent, PaneState refactor, open_pane)
  - crates/marley_app/src/app.rs (SHIM: mechanical migration of ~117 field accesses / ~42 getter sites)
---

## Title
The Warp-layout foundation: a pane can hold a terminal **or** a session-less panel (files / code / git).
Refactor `PaneState<S>` to a typed `PaneContent<S>` so Files/Code/Git can become real grid panes — the
structural blocker behind "the layout isn't Warp".

## Scope
### In
- PURE: `TerminalPane<S>` (the 8 terminal fields), `PaneContent<S>` enum, `PaneState<S> { content }` +
  `kind()` / `terminal()` / `terminal_mut()` accessors, `open_pane(content)`.
- SHIM: mechanical migration of the terminal field accesses to the accessors; the terminal keeps working.

### Out
- Rendering the new pane types (seq-2). Opening files/git as panes from the UI (seq-4/5/6). Any visual change.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — `PaneContent<S> { Terminal(TerminalPane<S>), FileTree, CodeView(CodeViewState), Git }`; `CodeViewState`
  is the existing code_view.rs type (reused). The split/close/focus/neighbor algebra is UNCHANGED (it works
  on the `PaneGroup` tree + `PaneId`, never the content).
- D2 — `PaneState::new(session)` = Terminal; `open_pane(content)` inserts a session-less pane as a split of
  the focused leaf. `kind()` derives `PaneKind` from the content variant.
- D3 — the shim narrows once per getter site: `if let Some(term) = state.terminal_mut()` then `term.session`
  etc. NO panic on the non-terminal path (seq-1 still only creates terminals, but the guard is correct now).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `PaneState::new(session)` runs, `kind()` shall be Terminal and `terminal()` shall be Some. | unit |
| REQ-002 | WHEN `open_pane(FileTree)` runs, the new pane's `kind()` shall be FileTree and `terminal()` None. | unit |
| REQ-003 | WHEN a pane is split or a sibling closed, each surviving pane shall keep its content + kind. | unit |
| REQ-004 | WHEN `open_pane(CodeView(state))` runs, the pane shall hold that CodeViewState. | unit |
| REQ-005 | the existing split/close/focus/neighbor tests shall still pass; the terminal still renders live. | unit + capture |
| REQ-006 | gate GREEN, cov/MSI 100 on the new workspace surface; the shim masked. | gate |

## Phase Plan
- **P2** — the TerminalPane/PaneContent/PaneState design + the accessor + open_pane + the app.rs migration map; test plan.
- **P3** — implement (workspace.rs refactor + app.rs migration).
- **P3.5** — 3 critics (correctness of the migration; the algebra invariants; simplification) — scaled up (LARGE).
- **P4** — the workspace tests (cov/MSI 100) + a LIVE terminal capture (still works) + gate GREEN.
- **P5** — docs, AAR, architecture doc, archive, close #120.
