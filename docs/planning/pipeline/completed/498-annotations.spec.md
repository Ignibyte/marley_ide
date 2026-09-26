---
pipeline_id: b8d629ab-a478-416f-87af-0c6d270dd851
ticket: docs/planning/tickets/open/TICKET-498-annotations.md
status: Phase 4 — Complete PASS
title: "B4: Draw annotations on the page, for Chad and for the agent"
type: feature
slice: prong 3 B4 (wave 2)
references: [docs/marley/browser-handoff.md, docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md]
---

## Title
Boxes and notes Marley draws over a page, stored in page coordinates so they stay on what they
mark as the page scrolls; Chad draws them by hand, an agent through Marley's MCP server.

## Scope
### In
- **The model.** An annotation is a box in page coordinates (the document's CSS pixels: a
  viewport point plus the page's scroll), a note, who made it (the user or an agent) and when;
  each page keeps its own, in the hub, for the session.
- **Drawing.** The Browser tab draws each annotation of its page over the frame, placed from the
  frame's own metadata (its scroll offsets and page scale), so a box moves with the content it
  marks and never lags the frame it is drawn on; a box scrolled out of view is not drawn; an
  agent's box looks different from the user's. Nothing is injected into the page to draw.
- **Chad's annotate mode.** A toolbar button and `marley::Annotate` turn it on: a drag in the
  page draws a box instead of reaching the page, and a note field opens by it; Enter keeps the
  annotation, Escape drops it. A click on an annotation's note selects it; Delete removes it.
- **For agents.** `browser_annotate` (a write tool: `ref` from `browser_snapshot`, or a box `x`,
  `y`, `width`, `height` in viewport pixels, and a `note`) draws a box around the element or the
  area and answers with the annotation's id; `browser_annotations` (a read tool) lists the page's
  annotations with their boxes in page coordinates, notes and makers; `browser_annotate` with
  `clear` removes the agent's own.
- A page that navigates to another document drops its annotations.

### Out (explicitly deferred)
- Annotations kept across launches or shared (`browser-handoff.md` open decision 4 is Chad's);
  arrows and freehand strokes; annotations that follow an element inside its own scrolling
  container or a fixed-position one.

## Reference (§20)
Chrome DevTools' element highlight and screenshot annotation tools, and Cursor's browser, where
the agent marks up what it sees; `browser-handoff.md` pillar B fixes the shape: "drawn by gpui,
stored in page coordinates, placed per frame from screencast metadata. Nothing is injected into
the page for drawing." Marley matches it with gpui elements over the page element and each
frame's metadata. Warp: N/A.

### Prior art
- **Published material.** CDP's `Page.screencastFrame` metadata (`scrollOffsetX`,
  `scrollOffsetY`, `pageScaleFactor`, `deviceWidth`, `offsetTop`) and `Page.getLayoutMetrics`
  (the visual viewport's page offsets), which place a document point in a frame.
- **Observed.** The 2026-09-24 probe: each frame's metadata carries the viewport in DIP, the
  scroll offsets in CSS pixels and the pinch scale (`three-prong-plan.md`, prong 3's probe,
  answer 5).
- **Code we already ship.** #488's `FrameMetadata` and `PageElement`, #489's `PageMapping`
  (window to page), #495's `to_window` (page to window), #492's refs and `box_center`, gpui's
  absolute elements and `canvas` for drawing over the frame.

## UI proof
UI-AFFECTING. `script/e2e/498-annotations.sh` (`compositor sway`, offline): a long fixture page
with headings down it. Steps: annotate mode, a drag around a heading and a note
(`498-01-drawn`); two wheel detents down (`498-02-scrolled`: the box still on its heading); the
stand-in agent's `browser_annotate` on a ref (`498-03-agent`: a box of the agent's around that
element with its note); `browser_annotations` (the run log: both, with their page coordinates
and makers); a click on the user's note and Delete (`498-03b-deleted`, the run log: the agent's
alone); a navigation (`498-04-gone`: no annotation).

## Locked-In Decisions
- D1 — Page coordinates are the document's CSS pixels, from each frame's own scroll offsets.
- D2 — gpui draws; the page gets no drawing script (the handoff's rule).
- D3 — Session only, per page, dropped on navigation, until Chad decides open decision 4.
- D4 — An agent's annotations are marked as the agent's, and the agent clears only its own.
- D5 — `browser_annotate` on a ref scrolls the element into view first, as the other write tools
  do, so the user sees what the agent marks.
- D6 — An annotation's time is seconds since the Unix epoch.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user drags a box in annotate mode and writes a note, the tab shall draw the box and its note over the page. | Shot `498-01-drawn` |
| REQ-002 | WHILE the page scrolls, each annotation shall stay on the content it marks. | Shot `498-02-scrolled` |
| REQ-003 | WHEN an agent calls `browser_annotate` with a ref and a note, the tab shall draw a box of the agent's around that element with the note. | Shot `498-03-agent` |
| REQ-004 | WHEN an agent calls `browser_annotations`, the answer shall list each annotation with its box in page coordinates, its note and its maker. | The run log |
| REQ-005 | WHEN the page navigates to another document, its annotations shall go. | Shot `498-04-gone` |
| REQ-006 | WHEN the user clicks an annotation's note and presses Delete, that annotation shall go. | Shot `498-03b-deleted`; the run log |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design (the note's editor, selection and
  removal).
- **P2 Code** — the model in the hub, the drawing, annotate mode, the two tools; fmt and clippy
  clean.
- **P3 Test** — the scenario, every shot; #489's and #492's scenarios again; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
