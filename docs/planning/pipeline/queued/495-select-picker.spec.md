---
pipeline_id: 2b706ad0-8ab8-47fd-a59b-3c97c07ec0fc
ticket: docs/planning/tickets/open/TICKET-495-select-picker.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "B1d: Marley draws the page's select lists"
type: feature
slice: prong 3 B1d (split from #493)
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/489-browser-input.spec.md]
---

## Title
A press on a `<select>` in the Browser tab shows its options in a list Marley draws, and the
chosen one is set in the page.

## Scope
### In
- On a left press in the page, before the press goes to the page, Marley asks what is under it
  (`DOM.getNodeForLocation`, then `DOM.describeNode`); for a `<select>` that is not `multiple`
  or disabled, it reads the options (text, value, disabled, selected, and their groups) in an
  isolated world and shows them in a list under the select, the current one marked, instead of
  sending the press.
- Choosing an option (the mouse, or the arrows and Enter) sets the select's `value` in the
  isolated world and dispatches `input` and `change` on it, so the page's own handlers run;
  Escape or a press elsewhere closes the list and the page keeps its value.
- A focused select still takes the keyboard in the page, as Chromium's own does.

### Out (explicitly deferred)
- `<select multiple>`, `<datalist>`, and a select inside a cross-site iframe.

## Reference (§20)
Chromium's own select popup for the behavior: the options under the select, the current one
marked, arrows and Enter to choose, Escape to close, `input` then `change` on a choice. Zed's
`ContextMenu` for the list's look. Warp: N/A.

### Prior art
- **Observed (the 2026-09-24 probe).** A `<select>` popup never reaches a headless frame, and no
  target appears for it.
- **Published material.** CDP's `DOM.getNodeForLocation`, `DOM.describeNode`, `DOM.resolveNode`,
  `Runtime.callFunctionOn`, `Page.createIsolatedWorld`.
- **Code we already ship.** #489's press path in `PageElement`; `Page::isolated_context`
  (#492); `ui::ContextMenu`.

## UI proof
UI-AFFECTING. `script/e2e/495-select-picker.sh` (`compositor sway`): a fixture page with a
`<select>` that prints its value and each `change`; a click on it, the list, a click on an
option; then Escape on a second opening. Shots: `495-01-open`, `495-02-chosen`,
`495-03-escaped`.

## Locked-In Decisions
- D1 — Marley draws the list and sets the value in an isolated world, so the page's handlers see
  an ordinary change.
- D2 — Only single selects, in the main frame or a same-site frame, for now.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses on a `<select>`, Marley shall show its options under it with the current one marked. | Shot `495-01-open` |
| REQ-002 | WHEN the user chooses an option, the page's select shall take its value and the page shall get `input` and `change`. | Shot `495-02-chosen` |
| REQ-003 | WHEN the user presses Escape in the list, it shall close and the page's value shall stay. | Shot `495-03-escaped` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the hit test, the options read, the list, the choice; fmt and clippy clean.
- **P3 Test** — the scenario, every shot; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
