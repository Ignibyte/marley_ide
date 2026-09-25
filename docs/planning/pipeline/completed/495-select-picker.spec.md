---
pipeline_id: 2b706ad0-8ab8-47fd-a59b-3c97c07ec0fc
ticket: docs/planning/tickets/open/TICKET-495-select-picker.md
status: Phase 4 — Complete PASS
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
- A listener Marley installs in an isolated world of each frame (the page's, its same-site
  frames', and each cross-site iframe's session) takes the press that would open a `<select>`
  that is neither `multiple`, sized as a list, nor disabled: a primary-button `mousedown`, and on
  a focused select Alt+Down, Alt+Up, F4 or Space. It cancels the press, so Chromium's own popup,
  which no headless frame shows, never opens, gives the select the focus, and reports the
  select's box, where the press landed, the options (text, disabled, group) and the selected
  one to Marley through a CDP binding.
- Marley shows the options in a list under the select, the current one marked, a group's label
  over its options, a disabled option greyed and not choosable. Choosing an option (the mouse,
  or the arrows and Enter) sets the select's index in the isolated world and dispatches `input`
  and `change` on it when the choice differs, so the page's own handlers run; Escape or a press
  elsewhere closes the list and the page keeps its value.
- A focused select still takes the arrow keys in the page, which change its value there as
  Chromium's own closed select does.
- The list opens only for the user: when the Browser tab has the focus or the user pressed in
  it a moment before. A select an agent clicks stays closed and focused, for keys.

### Out (explicitly deferred)
- `<select multiple>` and list-sized selects (the page draws those itself), `<datalist>`, and an
  agent tool that picks an option.

## Reference (§20)
Chromium's own select popup for the behavior: the options under the select, the current one
marked, arrows and Enter to choose, Escape to close, `input` then `change` on a choice. Zed's
`ContextMenu` for the list's look. Warp: N/A.

### Prior art
- **Observed (the 2026-09-24 probe).** A `<select>` popup never reaches a headless frame, and no
  target appears for it.
- **Published material.** CDP's `Runtime.addBinding` (with `executionContextName`),
  `Runtime.bindingCalled`, `Page.addScriptToEvaluateOnNewDocument` (with `worldName`),
  `Page.createIsolatedWorld`, `Runtime.evaluate`; `DOM.getNodeForLocation` and
  `DOM.describeNode`, which a hit test before each press would need.
- **Observed (the probe, 2026-09-25).** A listener in an isolated world named for the binding
  gets the page's `mousedown` on a select; `preventDefault` there keeps Chromium's popup shut
  (ArrowDown then changes the value in the page, with a trusted `input`, which an open popup
  would not do); the binding reports to the session, from a world made by
  `addScriptToEvaluateOnNewDocument` or, for a document loaded before, by
  `Page.createIsolatedWorld`; setting `selectedIndex` there and dispatching `input` and
  `change` reaches the page's listeners (not trusted). The page's own `mousedown` listeners still
  run.
- **Code we already ship.** #489's press path in `PageElement`; `Page::isolated_context`
  (#492); `ui::ContextMenu`.

## UI proof
UI-AFFECTING. `script/e2e/495-select-picker.sh` (`compositor sway`): a fixture page with a
`<select>` in groups with a disabled option, which prints its value and each `input` and
`change`; a click on it (the list), a click on an option; a second opening and Escape; then the
keyboard: Alt+Down opens the list, ArrowUp and Enter choose. Shots: `495-01-open`,
`495-02-chosen`, `495-03-escaped`, `495-04-keys`.

## Locked-In Decisions
- D1 — Marley draws the list and sets the value in an isolated world, so the page's handlers see
  an ordinary change.
- D2 — Single selects only, in any frame the hub watches, the main one, a same-site one or a
  cross-site iframe's session.
- D3 — A listener in the page takes the opening press, rather than a hit test before every
  press: no press waits on a round trip, and a keyboard opening is caught too (the probe).
- D4 — The list opens for the user only, so an agent's click never takes the user's focus.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses on a `<select>`, Marley shall show its options under it with the current one marked. | Shot `495-01-open` |
| REQ-002 | WHEN the user chooses an option, the page's select shall take its value and the page shall get `input` and `change`. | Shot `495-02-chosen` |
| REQ-003 | WHEN the user presses Escape in the list, it shall close and the page's value shall stay. | Shot `495-03-escaped` |
| REQ-004 | WHEN the user presses Alt+Down on a focused select, Marley shall show the list, and the arrows and Enter shall choose from it. | Shot `495-04-keys` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the hit test, the options read, the list, the choice; fmt and clippy clean.
- **P3 Test** — the scenario, every shot; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
