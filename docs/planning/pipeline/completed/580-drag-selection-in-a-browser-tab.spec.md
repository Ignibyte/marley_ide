---
pipeline_id: 233bb5e5-a657-4dc0-8c52-abe6012dfeae
ticket: docs/planning/tickets/closed/TICKET-580-drag-selection-in-a-browser-tab.md
status: Phase 4 — Complete PASS
title: "A drag in a Browser tab keeps its selection: #518's report was its test page's layout"
type: bug
slice: prong 3, after #489 (found in #518's Test)
references: [docs/planning/tickets/closed/TICKET-580-drag-selection-in-a-browser-tab.md, docs/planning/pipeline/completed/518-fuller-pick-bundle.spec.md, docs/planning/pipeline/completed/489-browser-input.spec.md]
---

## Title
#518 reported that a drag in a Browser tab leaves no selection once the button is up. The page's
own event log shows otherwise. The selection collapsed on a mouse move, before the release, when
the pointer passed the end of the text on #518's test page. That page places its paragraph
absolutely over a body with no height. A drag that ends inside the text keeps its selection, and
so does a drag past the end of a paragraph in normal flow. Marley's input path is right. This
ticket corrects the record and puts #518's scenario back on the drag its plan named.

## Scope
### In
- The diagnosis, recorded in the notes.
- `script/e2e/518-fuller-pick-bundle.sh`: select the paragraph with a drag that ends inside the
  text, as #518's plan had it, in place of the double click.
- Corrections to what #518 recorded: the known limit in
  `docs/marley_architecture/marley_workbench.md`;
  F-claude-518-a-drag-in-a-browser-tab-leaves-no-selection-001 and
  L-claude-518-select-with-a-double-click-in-a-scenario-001, replaced by one lesson; REQ-008's
  verification in #518's archived spec, and a correction entry in its archived notes.

### Out (explicitly deferred)
- Any change to Marley's mouse path (`PageElement::listen`, `BrowserHub::mouse_press`,
  `mouse_move`, `mouse_release`): none is needed.
- What Chromium does with a point beside an absolutely placed paragraph: that is the page's
  layout, the same in any Chromium.

## Reference (§20)
N/A — Marley-specific. The behavior at stake is Chromium's own text selection under a mouse drag,
which Marley passes through as CDP `Input.dispatchMouseEvent` (#489). The page is the reference:
its mouse and `selectionchange` events, logged to its console, show what Chromium did with each
event Marley sent. Neither Warp nor upstream Zed has a browser.

### Prior art
- **Behavior maps:** none apply; no reference app embeds a browser.
- **Published material:** CDP `Input.dispatchMouseEvent` (`type`, `button`, `buttons`,
  `clickCount`); the Selection API's `selectionchange` event.
- **The code we ship:** `crates/marley_workbench/src/browser.rs`: `PageElement::listen` (a press
  in the page focuses the tab and reaches the page; while a button is held, moves and the release
  reach it wherever the pointer is), `BrowserHub::mouse_press`, `mouse_move` (each move names the
  held button) and `mouse_release`; `crates/marley_browser/src/input.rs` `mouse_event`;
  `crates/marley_browser/src/page.rs` `selected_text` (`getSelection()` in an isolated world); the
  fixture's `mcp_agent console` (#492), which gave the whole event sequence. No change is needed,
  so there is nothing to adopt.

## UI proof
`script/e2e/518-fuller-pick-bundle.sh` (`compositor sway`): its selection step becomes a drag
across the sentence that ends inside the text. The list item's pick after it carries the
selection, and the shot `518-04-selection-pick` shows the dragged text still selected.

## Locked-In Decisions
- D1 — No change to Marley's input path. The page's log shows each press, move and release
  arriving with the right buttons, and the selection surviving the release wherever the drag
  ended on text.
- D2 — #518's scenario selects with a drag that ends inside the text, from x 42 to 300 on the
  sentence's row. A drag past the end would test the page's layout, not Marley.
- D3 — The wrong ledger entries are replaced, not kept beside corrections. They are hours old,
  and a reader who greps for a drag bug must find what is true.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user drags across a Browser tab's text and lets go inside it, the page shall keep the selection the drag made, as `browser_look` and a pick read it. | #518's scenario: the list item's pick after the drag carries a `selected_text` that begins "Select this sentence", and the shot `518-04-selection-pick` shows the text still selected |
| REQ-002 | No known limit, failure or lesson shall claim that a Browser tab loses a drag's selection. | Review: a grep of `docs/` for the old claims finds only this pipeline's correction entries |

## Phase Plan
- **P1 Plan** — the diagnosis (two probes), this spec, and the notes' design.
- **P2 Code** — nothing to build: no application code changes.
- **P3 Test** — the scenario's drag; its run and the shot read; `script/gates.sh --diff` green,
  since the scenario is gated.
- **P4 Complete** — the corrections (the known limit, the ledger, #518's archived pair); close
  the ticket as not a Marley bug; archive; commit and push.
