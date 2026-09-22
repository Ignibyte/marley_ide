---
pipeline_id: 4648b9db-ad7e-45b6-a1aa-8ceddcb62d30
ticket: docs/planning/tickets/open/TICKET-422-pane-divider-drag-routing.md
status: Phase 5 — Complete PASS
title: pane-divider drag resizes the boundary you grabbed — right split, right axis, right band
type: bug
milestone: M20
references:
  - docs/planning/design-notes/simple-rail-shelf.md
  - docs/planning/pipeline/completed/398-add-to-pane-cross-link.spec.md
  - docs/zed_architecture/subsystems/07-workspace-panes-palette.md
---

## Title
Chad: "drag and drop works only on the first item and it's very clunky." That is the
pane-divider resize, not DnD (no rail-row DnD exists — a recorded v2 deferral,
#398 spec :76-78/:138-141). Handles are painted per FLATTENED LEAF boundary
(app.rs:20627-20654, over `rect_list` = depth-first `pane_rects`, workspace.rs:159-207)
but the sink `resize_boundary` (layout.rs:204-208) doc-comments itself top-level-only
(M6 #130) and `resize_split` (layout.rs:150-163) no-ops for `boundary + 1 >= ratios.len()`
— a `Split` always has exactly 2 children (invariant, layout.rs:4-6), so ONLY boundary 0
ever mutates. Every other handle is painted, hoverable, `.occlude()`-ing, and inert.
The clunk is the same block: the strip is drawn `.top(px(0.0)).h(px(content_h))` — 30px
above the pane band (TOP_BAR_H=30.0 app.rs:768; `content_band` layout.rs:126-130; the
#168 files-edge strip :19192 gets `.top(px(content_top))` right); always a vertical 6px
strip even on a Vertical split's horizontal edge; no real capture semantics
(`dragging_divider` app.rs:642, root move/up :20657-20675, per-frame Δx over the full
center width). Fix: route the drag to the boundary actually grabbed — descriptors carry
the owning split's path + axis; resize applies at that path; handles sit on the shared
edge, right axis, right band; the gesture gets defined capture semantics.

## Scope
### In
- A pure divider-descriptor helper beside `pane_rects` (workspace.rs): walk the
  `PaneGroup`, one descriptor per `Split` boundary — {path to the owning split, axis,
  rect on the shared edge} — so routing is unit-testable without gpui.
- Path-aware resize in layout.rs (`resize_boundary` grows a path, or a new
  `resize_at(path, delta, min)` — Design settles the shape; `checked_sub` discipline
  per D5). `resize_split`'s clamp math reused unchanged.
- The app.rs:20627-20676 handle block rewritten over descriptors: band-aligned,
  axis-correct strips (D3); drag routed via the grabbed descriptor (D2); capture
  semantics defined (D4). `dragging_divider`'s shape follows the descriptor.
- Pure layout/geometry tests over nested H/V trees + the top-level regression pin
  (D6); the live-gesture rows carry the honest #130 headless caveat.

### Out (explicitly deferred)
- **Rail-row DnD** — none exists; deferred v2 with gpui's typed primitives recorded as
  the substrate (#398 spec :76-78, D1 :138-141). This bug must not grow it.
- **Adopting gpui's typed `on_drag` pipeline for this gesture** — sweep verdict in
  Prior art leg 3; it stays the recorded v2 substrate for rail DnD, not the divider.
- **PaneGroup algebra changes** beyond the resize sink — the 2-child invariant stands;
  no n-ary splits.
- **Divider visual grammar** — the strip's look (≈6px, `colors.border` fill, accent
  hover) is unchanged; only placement/axis are corrected.
- **Ratio persistence semantics** — unchanged; no new persist-on-release.
- **Non-terminal tabs** — the handle block lives (and stays) inside the
  `active_is_terminal` arm (app.rs:19510-19526); cockpit/code tabs have no pane grid.

## Reference (§20)
**Standard split-pane resize — every IDE/terminal ships it; the divider you grab is
the one that moves.** Observed reference for feel: the React POC's
react-resizable-panels behavior (marley-web
`artifacts/marley-ide/src/components/views/SplitTerminalView.tsx:131-145`) — the
`PanelResizeHandle` sits BETWEEN panels on the shared edge, spans that edge full-length,
is axis-correct per group direction (`w-1` horizontal / `h-1` vertical), accents on
hover, clamps at min sizes, and resize follows the grabbed divider, nested groups
included. The Zed behavior map (docs/zed_architecture/subsystems/07-workspace-panes-palette.md:112-125,
cited in Prior art) confirms the tree-shaped model at behavior level. Marley matches
that BEHAVIOR on its own `PaneGroup` algebra. Clean-room: behavior maps + our own
substrate only; no copyleft source read.

### Prior art
1. **Behavior maps** — Zed deconstruction 07 (:112-125): `PaneAxis` caches
   `bounding_boxes` "for hit-testing during drag-resize"; `resize(pane, axis, amount,
   bounds)` takes the AXIS; a dedicated `pane_group::element` renders the TREE with
   draggable handles (`HANDLE_HITBOX_SIZE`) — handles belong to split nodes, never to
   flattened leaves. That is exactly the model this fix adopts. Warp maps: nothing on
   divider mechanics (01-ui-framework-rendering.md:66 names Marley's own drag-resize
   line); Warp's observed behavior is the same standard. Research, not source.
2. **Published** — split-pane resize is standard (IDEs, tmux): grabbed divider moves,
   both neighbors reflow, min sizes clamp, the pointer may stray mid-drag.
   react-resizable-panels (the POC dep) documents the same contract. Nothing exotic.
3. **Permissive deps (the paying leg)** — gpui 0.2.2 READ at
   `~/.cargo/registry/src/…/gpui-0.2.2`: typed drag primitives in
   `src/elements/div.rs` — `on_drag_move<T>` :282 (capture-phase, fires for the active
   typed drag), `on_drop<T>` :462, `can_drop` :473, `on_drag<T, W>` :499 — and the
   framework side: `on_drag` REQUIRES a drag-preview `Entity<W>` constructor whose
   ghost renders under the cursor (window.rs:2048-2078); activation waits for a
   held-button move (window.rs:3625); ANY mouse-up centrally clears `active_drag`
   (window.rs:3716-3729). **VERDICT: keep `mouse_down` + view state for this gesture.**
   The typed pipeline is DnD-shaped — mandatory ghost view, drop targets — and a
   divider resize wants neither; the root-level move/up listeners already deliver the
   window-wide capture the gesture needs (the #168 files-edge drag proves the pattern
   daily, app.rs:19186-19225). gpui's typed drag stays the recorded rail-DnD v2
   substrate (#398), not this seam. Paying find: `MouseMoveEvent.pressed_button:
   Option<MouseButton>` (interactive.rs:335-340) gives D4 its missed-release
   detection for free. Examples swept: `drag_drop.rs` is value+ghost DnD; **no
   resizable/split example ships** — nothing further to adopt.

## React-first (parity)
N/A — no UI delta: gesture/hit-target bugfix. The divider's visual grammar (≈6px
strip, `colors.border` fill, accent hover) is unchanged — the fix aligns the strip to
the shared edge it was always meant to mark and makes inert handles do their job. The
POC (react-resizable-panels, SplitTerminalView.tsx:131-145) already behaves correctly
and serves as the observed behavior reference above; no React design pass needed.

## Locked-In Decisions
- **D1 — Divider descriptors come from a PURE helper beside `pane_rects`.** One walk
  of the `PaneGroup` yields {path to the owning `Split`, axis, rect} per boundary —
  routing and geometry are unit-testable without gpui. (Exact name/fields = Design;
  the boundary inside a 2-child split is always the one between its children.)
- **D2 — Resize applies AT THE OWNING SPLIT'S PATH, nested included.** Either
  `resize_boundary` grows path-awareness or a new `resize_at(path, delta, min)` walks
  to the split and applies `resize_split` — shape is Design's. The min-ratio clamp is
  preserved unchanged (`PANE_MIN_RATIO = 0.1`, app.rs:1025). The delta normalizes
  against the OWNING split's extent along its axis, not the full center width.
- **D3 — Handle geometry = the shared edge, correct axis, content band.** Horizontal
  strips for a Vertical split's boundary; y offset by `content_top` exactly as the
  #168 files-edge strip does (app.rs:19192); hit width stays ~6px centered on the edge.
- **D4 — Capture semantics defined.** The drag keeps responding while the pointer
  strays off the strip (the root move/up listeners already deliver window-wide — that
  half works today); mouse-up ends it; a missed release (outside the window, focus
  loss) must not stick — candidates verified: `MouseMoveEvent.pressed_button == None`
  while dragging (interactive.rs:335-340) and the existing focus-edge hook
  (app.rs:17189-17194). Design picks the mechanism; the semantics are locked.
- **D5 — `checked_sub`, never eager `- 1` (L TICKET-020).** New boundary/path index
  math in layout.rs uses `checked_sub`; note clippy's `unnecessary_lazy_evaluations`
  actively pushes `.then(|| i - 1)` back to the eager form — do not take that bait.
- **D6 — Top-level 2-pane RESIZE behavior byte-identical.** Same boundary routed, same
  ratio trajectory for a given drag sequence; `resize_split_cases` (layout.rs:550)
  stays green unchanged. The only intended deltas are D3's geometry (band/axis) and
  D4's capture hardening. (`resize_boundary_top_level_only` :584 pins today's
  "nested is a no-op" — Design decides whether it evolves or is superseded by the
  path-aware pins; top-level rows survive verbatim.)

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN any divider in a nested split arrangement is dragged, the system shall resize exactly its owning split's boundary, leaving every other split's ratios unchanged. | Pure unit over nested H/V trees (descriptor → path → resize); kills the flattened-index routing mutant. |
| REQ-002 | WHEN a Vertical split's divider is dragged, the system shall resize along the vertical axis, the delta derived from pointer Δy normalized by the owning split's height. | Pure unit (axis + delta normalization); kills the always-Δx/center-width mutant. |
| REQ-003 | Divider descriptors shall lie on the shared edge between the owning split's two children, within the content band (y offset by `content_top`), hit width ~6px centered on the edge. | Pure geometry unit vs `pane_rects` over the same tree/bounds. |
| REQ-004 | WHILE a divider drag is active, WHEN the pointer leaves the grab strip, the system shall keep applying the drag until mouse-up. | Code-review + inspect probe — real mouse input is env-blocked headless (the #130 stance, recorded honestly); any pure end-condition helper gets a unit. |
| REQ-005 | IF the button release is never observed (release outside the window, focus loss mid-drag), THEN the system shall end the drag on the next observed evidence rather than remain dragging. | Unit over the pure end-condition (per D4 mechanism) + code-assert on the wiring. |
| REQ-006 | WHEN a drag would take either adjacent child of the owning split below `PANE_MIN_RATIO` (0.1), the system shall clamp at the min preserving the pair sum — at any depth. | Pure unit (`resize_split` rows extend over paths); kills the clamp-drop mutant. |
| REQ-007 | WHILE the arrangement is a single top-level 2-pane split, the system shall route and resize byte-identically to shipped #130 behavior. | `resize_split_cases` (layout.rs:550) green unchanged + a legacy-equivalence unit (top-level descriptor x/width vs the shipped handle math). |
| REQ-008 | The full gate suite shall pass on the final diff, with `cargo check --tests` explicitly exercised. | `scripts/gates.sh --diff` exit 0; `cargo check --tests` exit 0 (F-#386: `#[cfg(test)]` fan-out that plain `cargo check` never compiles). |

## Phase Plan
- **P2 Design** — descriptor helper design (name, fields, traversal order, path
  encoding) + the resize sink shape (grow `resize_boundary` vs new `resize_at`;
  `checked_sub` per D5); delta normalization plumbing (owning split extent); the
  app.rs handle-block rewrite plan; D4 mechanism pick (`pressed_button` vs focus-edge
  vs both); disposition of `resize_boundary_top_level_only`; test matrix over nested
  trees (H-root/V-root, depth ≥ 2, 3-leaf L-shapes, min-clamp edges); file manifest;
  confirm the §20 line.
- **P3 Implement** — helper + path-aware resize + handle block rewrite + capture
  semantics; supersede the #130 comment honestly; decl-comment updates.
- **P3.5 Inspect** — independent critics vs the diff: routing off-by-one, axis/delta
  sign, band math, stuck-drag edges, the L TICKET-020 pattern + clippy pushback, §20
  provenance.
- **P4 Validate** — write + RUN the pure layout/geometry tests + negative smokes
  (wrong-path routing, axis swap, dropped clamp); `cargo check --tests`;
  `scripts/gates.sh --diff` green.
- **P5 Complete** — CHANGELOG + arch docs (§21), ledger capture (§19) — including the
  headless-gesture limitation recorded plainly, as the original #130 comment did;
  archive the pair, close the ticket + drop its BACKLOG row.
