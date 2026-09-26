# B4: Draw annotations on the page, for Chad and for the agent — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-498-annotations.md
- **Pipeline spec:** 498-annotations.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-25)
- **Request:** wave 2 of prong 3, pillar B (`browser-handoff.md`).
- **Classification:** feature; `marley_workbench` (the model in the hub, the drawing, annotate
  mode, the tools), `marley_mcp` (two rows). No Zed path expected.
- **Recall (§18.3):** the probe's answer 5 (frames carry the scroll offsets and the scale; input
  is in viewport CSS pixels); #495's page-to-window mapping; #492's refs; open decision 4
  (persistence) is Chad's, so D3 keeps them for the session.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-25)
- **Pre-flight:** #497 committed (12f07ffbe4); no other active pipeline; cargo idle; the README
  marker present.
- **Recall:**
  - The brain (consultation `b55fa8d231c942a0bc93698a5eea5989`): nothing on this seam.
  - #496 and #497 as built: the tray under the toolbar moves the page down by whole rows; the
    tab's `key_down` gets Escape since #496; the `browser_*` write tools bring the tab to the
    front (`show_for_agent`) and place a ref through `place` (scrolled into view, a cross-site
    iframe's owner as its offset).
  - The probe's answer 5: each frame's metadata carries the viewport in DIP, the scroll offsets
    in CSS pixels and the pinch scale.
- **Seams re-verified:** `FrameMetadata { offset_top, page_scale_factor, device_width,
  device_height, scroll_offset_x, scroll_offset_y }` comes with each frame and sits beside it in
  `PageState`; `PageElement` paints the frame at its natural size from the page area's top left,
  and `PageMapping` maps window to viewport (`map`) and back (`to_window`); `Page::viewport`
  reads `Page.getLayoutMetrics` (scroll and scale); `DOM.getBoxModel` answers quads in the
  frame's viewport (`content_quad`); `Page.frameNavigated` for the main frame is where a new
  document shows; `IconName::Pencil` exists.

### Design
- **The model (the hub).** `Annotation { id, page_box: PageBox, note, maker: Maker (User, Agent),
  made_at }` per page (`PageState::annotations`), numbered by the hub across the session;
  `annotations(target)`, `add_annotation`, `remove_annotation`, `clear_agent_annotations`; the
  main frame's `frameNavigated` clears them (a fragment or history move does not).
- **Drawing.** The view's render reads the frame and its metadata together and places each
  annotation from them: a document point `(x, y)` sits at `((x − scrollX) × zoom) / scale_x`,
  `((y − scrollY) × zoom + offsetTop) / scale_y` from the page area's top left, `scale` being the
  metadata's DIP width over the width the frame is drawn at. Each is an absolute `div` in the
  page area, which clips: the box (a 2 px border and a light fill: the user's in the theme's
  warning hue, the agent's in its accent hue) and its note on a chip at the box's top left (an
  agent's with the Sparkle icon), a selected one's border thicker. A box wholly out of the
  viewport is not drawn. Nothing reaches the page to draw.
- **Annotate mode.** A toolbar button (`Pencil`, lit while on) and `marley::Annotate`. While on,
  `PageElement` registers a drag instead of the page's press, move and release (the wheel still
  scrolls the page): the drag's corners in document points (viewport point over zoom, plus the
  scroll at that frame), drawn as a dashed draft; on release a note field (a single-line
  `Editor` in `MarleyAnnotationNote`) opens under the box: Enter keeps it
  (`marley::KeepAnnotation`), Escape drops it (`marley::DropAnnotation`). Escape with no draft
  ends the mode. A drag shorter than 4 pixels either way is a click and draws nothing.
- **Select and Delete.** A click on a note's chip selects that annotation and gives the page the
  focus; Delete or Backspace in `key_down` removes the selected one; a press anywhere else in the
  page drops the selection.
- **For agents.** `browser_annotate` (write): `ref`, or `x`, `y`, `width`, `height` in viewport
  pixels, and `note`; a ref's element scrolled into view first (D5) and its border box read
  (`Page::border_box`), through its cross-site iframe's owner as `place` does; the box plus the
  scroll from `Page::viewport` is the document box; the answer names the annotation's id and
  the tab. `clear: true` removes the agent's own. `browser_annotations` (read): each annotation's
  id, box in page coordinates, note, maker and `made_at` (D6).
- **Manifest:** `crates/marley_browser/src/page.rs` (`border_box`);
  `crates/marley_workbench/src/browser.rs`, `browser_tools.rs`, `marley_workbench.rs` (three
  actions), `keymap.json`; `crates/marley_mcp/src/registry.rs`; `script/e2e/498-annotations.sh`,
  `script/e2e/browser-fixture.sh` (the agent's `annotate`, `annotations`). No Zed path.

### E2E plan
| REQ | Step (`compositor sway`, offline) | Shot or log |
|---|---|---|
| REQ-001 | a long page with headings; annotate mode, a drag around the second heading, a note, Enter | `498-01-drawn` |
| REQ-002 | five wheel detents down | `498-02-scrolled`: the box still around its heading |
| REQ-003 | the stand-in agent's `browser_annotate` on a heading's ref, with a note | `498-03-agent` |
| REQ-004 | its `browser_annotations` | the run log: both, their page boxes and makers |
| REQ-006 | a click on the user's note, Delete; `browser_annotations` again | `498-03b-deleted`, the run log |
| REQ-005 | a navigation to another page | `498-04-gone` |
| (regression) | #496's scenario (the page's input path) and #492's (the write tools) | their shots |

### Risks
- An element in its own scrolling container, or a fixed one, moves apart from its annotation
  (out of scope, as the spec says).
- The pick tray above the page moves the page area down; annotations are placed from the page
  area's own top left, so they move with it.
- A page that reflows when its viewport changes shifts content away from its annotations.

### Checklist (no TaskCreate in this harness)
pick ✓, pre-flight ✓, recall ✓, promote ✓, prior art ✓, spec ✓, design ✓.

## Phase 1 — closeout
Phase 1 PASS (autonomous).

## Phase 2 — Code (2026-09-25)
- **Built:**
  - `Page::border_box` (the border quad's bounds, in the node's frame's viewport), sharing the
    box-model read with `content_quad`.
  - The hub: `Maker`, `Annotation { id, page_box, note, maker, made_at }`, a page's
    `annotations`, `next_annotation`; `annotations`, `add_annotation`, `remove_annotation`,
    `clear_agent_annotations`; `left_document` on the main frame's `frameNavigated`.
  - The view: `AnnotateMode` (`Off`, `On`, `Dragging`, `Noting` with its note field) and
    `selected_annotation`; `annotate` (the action and the toolbar's Pencil button),
    `start_drag`, `drag_to`, `end_drag` (a box under 4 CSS pixels a side draws nothing),
    `keep_annotation`, `drop_annotation`, `select_annotation`; `key_down`: Escape ends annotate
    mode, Delete or Backspace removes the selected annotation.
  - The drawing: `Placement` from the frame drawn now and its metadata; `render_annotations`,
    `render_annotation` (the box, the note's chip above it, which a click selects, an agent's
    with the Sparkle icon), `render_note_field`; the page area clips.
  - `PageElement`: in annotate mode `listen_for_a_box` takes the press, move and release as a
    drag in document points; `listen_for_the_wheel` keeps the wheel reaching the page either
    way; a press in the page drops the selection.
  - The tools: `browser_annotate` (a ref's border box after `ref_origin`, which `place` now
    shares, or an area; `clear`), `browser_annotations`; their registry rows and schemas.
  - The actions `Annotate`, `KeepAnnotation`, `DropAnnotation`; `MarleyAnnotationNote > Editor`
    binds Enter and Escape. The stand-in agent's `annotate`, `annotate-clear`, `annotations`.
- **Deviations from the design:** none.
- **Review against the criteria:**
  - REQ-001, REQ-002: a box is placed from the same frame it is drawn over, so it moves with
    that frame's scroll and never lags it; the page area clips what scrolls out.
  - REQ-003: the agent's ref is scrolled into view before its box is read, and the box is
    stored with the scroll read after that.
  - REQ-005: the drop runs on the main frame's `frameNavigated` only; a fragment move keeps the
    boxes.
  - REQ-006: the chip occludes the page under it, so its click selects without reaching the
    page, and Delete comes to `key_down` because the chip gives the page the focus.
  - Nothing is injected into the page; boxes have no hitbox, so the page under them still takes
    the pointer outside annotate mode.
- **Checks:** clippy on the three crates clean; `cargo fmt` clean.
- **Checklist (no TaskCreate in this harness):** page.rs ✓, hub ✓, view ✓, drawing ✓,
  PageElement ✓, tools ✓, registry ✓, actions and keys ✓, fixture ✓, review ✓.

## Phase 2 — closeout
Phase 2 PASS (autonomous).

## Phase 3 — Test (2026-09-25)
- **Scenario:** `script/e2e/498-annotations.sh` (`compositor sway`, offline Chromium): a loopback
  `long.html` 3,000 pixels tall with four headings placed absolutely (the second at 240, the
  third at 640), and `other.html`. Steps: `marley: annotate` from the palette; a drag from
  (30, 230) to (450, 290) in the page, "Check this heading", Enter; two wheel detents down; the
  stand-in agent's `annotate heading "Third heading"`; its `annotations`; two detents up, a click
  on the user's note chip, Delete, `annotations` again; a navigation to `other.html`,
  `annotations` once more.
- **Shots (in the scratchpad, `e2e-498/`), each read:**
  - `498-01-drawn` (REQ-001): the Pencil lit; an amber box around "Second heading" and its chip
    "Check this heading" above it.
  - `498-02-scrolled` (REQ-002): the page 200 pixels down, the heading near the top, and its box
    and chip still around it.
  - `498-03-agent` (REQ-003): a blue box around "Third heading" with the chip "✦ The agent looked
    here", the Agent chip "annotated heading “Third heading”", the user's box still in place.
  - `498-03b-deleted` (REQ-006): back at the top, the user's box gone, the agent's still around
    its heading.
  - `498-04-gone` (REQ-005): "Another page", and no box.
- **The run log (REQ-004):** `annotation 1 by the user: 'Check this heading' at 30,230 420x60`,
  `annotation 2 by the agent: 'The agent looked here' at 40,640 400x40` (the heading's border
  box, in page coordinates); after Delete the agent's alone; after the navigation "no
  annotations".
- **What the first runs found, fixed at the source:**
  - The snapshot gave refs to interactive nodes only, `full` or not, so an agent could not name
    a heading to annotate. With `full`, a node the tree names (not text, not the document) now
    gets a ref too; the stand-in agent's `annotate` takes a full snapshot.
  - Five wheel detents scrolled the annotated heading out of view, which showed nothing of
    REQ-002; the scenario scrolls two (the spec's UI proof says so now).
  - Clippy after `cargo fmt`: two closures reformatted into blocks wanted their semicolons.
- **Regressions:** `492-browser-tools.sh`: its three shots as before (its default snapshot's refs
  unchanged). `496-element-picker.sh`: the first rerun made no pick, because the scenario clicked
  where the pick button sat, which is the annotate button's place since #498; with `PICK_X` at
  the pick button's new place (1314) every shot and log line is as before.
- **Focus report:** every run in the headless sway; "hyprland: 0 Marley windows before the run,
  0 after; the run added no rule and did not reload it".
- **Gate:** `just gate-diff` — 16 passed, 0 failed, `GATE GREEN [diff]`; the receipt matches the
  tree.
- **Checklist (no TaskCreate in this harness):** REQ-001 ✓, REQ-002 ✓, REQ-003 ✓, REQ-004 ✓,
  REQ-005 ✓, REQ-006 ✓, 492 ✓, 496 ✓, gate ✓.

## Phase 3 — closeout
Phase 3 PASS (autonomous).

## Phase 4 — Complete (2026-09-25)
- **Documented:** `CHANGELOG.md` (#498 under Added); `docs/marley_architecture/marley_browser.md`
  (Boxes; full snapshots' refs), `marley_workbench.md` (Annotations; the two tools),
  `marley_mcp.md` (eight read tools, seven write); the plan's B4 row shipped and open decision
  6. No path outside the Marley-owned set changed.
- **Knowledge appended:** `F-claude-498-a-full-snapshot-gave-no-heading-a-ref-001`,
  `L-claude-498-draw-over-a-streamed-page-from-the-frame-drawn-001`,
  `L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001`,
  `AD-claude-498-annotations-are-the-hubs-in-document-coordinates-001`.
- **Brain:** consultation `b55fa8d231c942a0bc93698a5eea5989` closed with
  `decisions/marleys-annotations-are-the-hubs-in-document-coordinates-drawn-by-gpui`.
- **Ticket:** closed; the BACKLOG row went at promotion.
