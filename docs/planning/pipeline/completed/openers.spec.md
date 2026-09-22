---
pipeline_id: 9b36278d-9c7b-4bae-874e-ff4ecf5ad32b
ticket: forge#128 (3671a317-44bc-4677-b9c1-1a245cb9958c) · local docs/planning/tickets/open/TICKET-128-openers.md
aar_id: recorded-in-notes
status: Phase 5 — Complete PASS
title: open Files/Code/Git panes on demand (M6 openers)
type: feature
milestone: M6 — The Warp Layout
references:
  - crates/marley_app/src/workspace.rs (PURE: OpenAction, open_or_focus)
  - crates/marley_app/src/app.rs (SHIM: open_files_pane; DRY the openers; a "Files" command)
---

## Title
Every pane kind is openable on demand — a "Files" palette command opens the file explorer, joining the
⌘-click code opener (#124) and ⌘⇧C git opener (#125). The open-or-focus decision is now one pure fn.

## Scope
### In
- PURE `OpenAction` + `open_or_focus(target)` (focus an existing pane of the kind, else open a new one).
- SHIM: `open_files_pane`; refactor the openers through `open_or_focus`; a "Files" cockpit command.

### Out
- The finder ⌘↵ (already routes to code panes via #124). New kinds.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — Files/Git are singletons (open-or-FOCUS via `open_or_focus`); Code opens-or-updates (already #124).
- D2 — the "Files" command opens/focuses a FileTree pane, persisting the grid.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `open_or_focus(k)` runs with a pane of kind k open, it shall return FocusExisting(that id). | unit |
| REQ-002 | WHEN no pane of kind k is open, `open_or_focus(k)` shall return OpenNew. | unit |
| REQ-003 | gate GREEN, cov/MSI 100 on open_or_focus; the openers masked (palette needs env-blocked input). | gate |

## Phase Plan
- **P2** — OpenAction/open_or_focus; open_files_pane + the "Files" command + the DRY; test plan.
- **P3** — implement (workspace.rs + app.rs).
- **P3.5** — 1 self-review: open_or_focus arms; the command wires to open_files_pane.
- **P4** — open_or_focus tests (cov/MSI 100) + gate GREEN (opener code-reviewed; render already proven).
- **P5** — docs, AAR, archive, close #128.
