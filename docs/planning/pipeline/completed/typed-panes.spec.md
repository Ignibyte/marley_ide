---
pipeline_id: fe3b4081-3eb0-4e60-8c2e-d1afd860f19f
ticket: forge#107 (99ad0ddd-b6be-46a2-b6b5-5d987ac0fb08) · local docs/planning/tickets/open/TICKET-107-typed-panes.md
aar_id: c2c6b7cc-185d-4241-b21c-e4e9a375114c
status: Phase 5 — Complete PASS
title: typed panes (PaneKind)
type: feature
milestone: M5 — The Warp Workspace
references:
  - crates/marley_app/src/workspace.rs (PURE: PaneKind + PaneState.kind)
---

## Title
The foundation of the Warp pane grid: every pane carries a `PaneKind` (Terminal / FileTree / CodeView /
Git), defaulting to Terminal and preserved across split/close — so later panes can host files, code, or git
beside terminals.

## Scope
### In
- PURE `PaneKind` enum {Terminal, FileTree, CodeView, Git} (Copy, Default = Terminal) + `label()`.
- `PaneState<S>` gains `kind: PaneKind` (Terminal in `new`), preserved by the split/close algebra.

### Out
- Per-kind CONTENT (file tree / viewer / git in a pane) — seq-7/8/9. Per-kind title bars — seq-2.
- Making the session optional (non-terminal panes without an `S`) — a later refactor when a kind needs it.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — the kind lives on `PaneState` (per-pane state), NOT the `PaneGroup` layout tree (which stays pure geometry).
- D2 — every pane is Terminal in #107; `label()` (Terminal→"terminal"/FileTree→"files"/CodeView→"code"/Git→"git")
  keeps all variants live + MSI-testable and is the dispatch seq-2 builds the title bar on.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a pane is created, its `kind` shall default to `Terminal`. | unit |
| REQ-002 | WHEN `PaneKind::label` runs, it shall map each variant to its label. | unit |
| REQ-003 | WHEN a pane is split or a sibling closed, each surviving pane's `kind` shall be unchanged. | unit |
| REQ-004 | gate GREEN, cov/MSI 100 on PaneKind + the kind field. | gate |

## Phase Plan
- **P2** — PaneKind + label + the PaneState.kind field + the split/close preservation; test plan.
- **P3** — implement (workspace.rs).
- **P3.5** — 1 critic: PaneKind MSI (label/default/preservation); the layout tree stays geometry-only.
- **P4** — the PaneKind tests (cov/MSI 100) + gate GREEN.
- **P5** — docs, AAR, archive, close #107.
