---
pipeline_id: 483da6ee-f3ed-468a-ab8c-9cb9b603860a
ticket: docs/planning/tickets/open/TICKET-431-displaymap-excerpt-unification.md
status: Phase 5 — Complete PASS
title: DisplayMap excerpt unification — one projection stack (M33)
type: feature
milestone: M33
references: [docs/planning/design-notes/m33-tail-and-wedge-shelf.md]
---

## Title

The B-c chain's recorded prize: fold the multibuffer's slot rim into the ONE
`crates/marley_app/src/display_map.rs` facade so fold∘wrap∘excerpt is a single typed
projection stack. Today two row models coexist — the editor's `DisplayMap` (folds #425 ∘
wrap #426, typed `BufferRow`/`DisplayRow`) and the multibuffer's own materialized
`slots: Vec<Row>` (#427–#430: prefix-summed Header|Note|Line, consumed RAW as `usize` at
the render/movers/caret/jump rim in app.rs). #427 chose the parallel model deliberately
(AD-claude-427) and recorded this unification as the deferred prize in display_map.rs's
own module doc. Why now: #432 (input completeness) is about to add MORE raw-slot
consumers, and every wedge feature after it (wrap inside excerpts, folds inside excerpts)
is a cross-product of two row systems unless the excerpt stage becomes a facade layer
first. Byte-identical by proof — the #425 recipe: the facade property-equals the direct
model, the full suite unchanged, live-drive verified.

## Scope

### In
- An EXCERPT stage in the `DisplayMap` facade: the multibuffer's slot projection
  (Header/Note/Line over materialized groups, prefix-summed like `WrapIndex`) becomes a
  facade layer with a typed slot axis — `locate` at the mb rim answers a typed row through
  the one facade, `DisplayRow` in, model indices out.
- Re-route every mb slot-rim consumer in app.rs through the facade: the
  `multibuffer_body` render closure + sizer + footer compare + caret `slot_of_row`, the
  `handle_multibuffer_key` movers + `scroll_to_item`, `mb_place_caret`,
  `multibuffer_jump_selected`, the two empty-materialize guards, and the frame-sync
  `refresh_cum` seam.
- The mb model may SHRINK: `slots`/`locate`/`move_selection`/`slot_of_row` math may
  re-home into the facade stage (ONE home for slot math — never two copies); the model's
  public behavior for remaining consumers is preserved verbatim (Option-on-empty,
  past-end saturation, non-Line-skipping movers, forward-only selection normalize).
- Equivalence property tests (facade == direct #427–#430 model over generated models
  including Note slots), the raw-conversion grep audit, a trybuild row-type pin if a new
  cross-space mix becomes representable, live smoke on both mb surfaces.

### Out (explicitly deferred)
- Wrap INSIDE excerpts and folds INSIDE excerpts — the features this stack shape enables;
  they are follow-on tickets, never riders here.
- The singleton unification (the editor tab as a one-excerpt multibuffer — Zed's
  masterstroke): a much bigger step; the editor and the mb keep separate stack INSTANCES
  behind the one facade type this ticket.
- Any SumTree / sync-chain machinery — the memo-rebuild shape (#425/#426) stays.
- Any change to `build`/`build_with_meta`/`rebuild_lines` semantics, #428 anchors/targets/
  journal/editing behavior, #429 batches, or the #430 refresh epoch.
- #432's input seams (click-column caret, ⌘V paste, IME) — the next ticket.
- Any user-visible behavior change whatsoever, on any surface.

## Reference (§20)

**Zed (the editor reference), architecture-level — behavior only.** In Zed the excerpt
layer sits at the BASE of the stack: `MultiBuffer` IS the text model (singleton or
N-file), the `DisplayMap` transform stack sits on top and "neither knows nor cares
whether there's one excerpt or a thousand", and excerpt headers are BLOCK rows a layer
inserts between lines — non-document rows minted by the projection, exactly the behavior
Marley's Header/Note slots already implement at smaller shape. The behavior adopted here:
ONE authority owns every slot↔row conversion, header/band rows are projection-inserted
slots inside that authority, and layers compose behind it instead of colliding beside it.
Cited research (deconstruction docs, never Zed source):
`docs/zed_architecture/subsystems/03-editor-multibuffer.md` §3 (the stack + the uniform
layer contract + `DisplayRow` as the top coordinate), §6 (excerpts at the base; headers
as blocks; the singleton unification — recorded as deferred Out here). Marley adopts the
CONTRACT, not the container: no SumTree, no six layers — a third stage beside folds/wrap
in the shipped facade, Vec-backed prefix sums. Clean-room statement: no Zed source was or
will be read for this ticket; the subsystem doc is the sole Zed reference. Byte-identical
by design, so there is no behavioral capture to match — the §20 value is the layering
reference plus the wall.

### Prior art

1. **Behavior maps** — `docs/zed_architecture/subsystems/03-editor-multibuffer.md` §3
   (transform/summary/snapshot contract; each layer introduces its coordinate), §6 (the
   excerpt tree under the stack; "excerpt headers are BlockMap blocks"; `singleton: true`
   = a plain editor — the endgame this ticket does NOT take). Research only.
2. **Published** — a view-model line mapping with injected non-document rows is standard
   editor architecture (VS Code's view model lines, CodeMirror 6 block widgets); the
   prefix-sum/binary-search small shape is public CS. Verdict: nothing to import — the
   pattern is already implemented twice in-tree; this ticket removes the "twice".
3. **Our permissive deps** — **gpui 0.2.2**: the `uniform_list` rim is `usize`
   (`item_count`, range, `scroll_to_item`) — verified at the #425 registry read,
   unchanged; gpui offers no row-projection/slot-mapping helper for this seam.
   **ropey** (via `marley_editor::Buffer`): line indexing ground truth only; no display
   projection. No sum-tree crate exists in the permissive dep set and none is wanted.
   **In-tree art is the headline**: `WrapIndex.cum` (display_map.rs), `FoldProjection`
   (half-open runs, AD-claude-half-open-intervals), and the mb's own `slots` mint
   (multibuffer.rs `refresh_cum`) — #431 UNIFIES shipped Marley art; it imports nothing.

## React-first (parity)

N/A — no UI delta: byte-identical refactor proven by property tests + unchanged captures.
The POC's `MultibufferView.tsx` stays the standing #427–#430 parity reference; nothing to
design first because nothing the user sees changes (REQ-001/002/006 pin it).

## Locked-In Decisions

- **D1 — Byte-identical, proven not claimed (the #425 recipe).** Facade outputs are
  property-equal to the direct #427–#430 model over discriminating generated models
  (Note slots, multi-file, disjoint windows, empty, post-refresh shapes); the full
  workspace suite passes with zero behavioral edits (a test moving WITH re-homed code
  keeps its assertions verbatim; any assertion-value change is a stop-the-line inspect
  flag); the editor's fold∘wrap instance stays bit-for-bit (its suite + captures
  unchanged); a live drive on both mb surfaces closes the proof.
- **D2 — The mb model's PUBLIC behavior/API stays; consumers move to the facade; the
  model may shrink.** Every app.rs consumer converts through the facade after this
  ticket. Slot math (`slots` mint, locate, movers, slot_of_row, the forward-only
  normalize) gets ONE home — if it moves into the facade stage, the model drops it
  rather than keeping a delegating copy (PR-1691: one derivation, never N textual
  copies). Build/rebuild/anchors/journal/kind/footer semantics are untouched.
- **D3 — No behavior riders.** No wrap-in-excerpts, no folds-in-excerpts, no new UI, no
  input-seam work, no persistence change. The deliverable is the stack shape + the typed
  rim + the proof; anything user-visible found tempting mid-flight is a new ticket.

## Acceptance Criteria (EARS)

| # | EARS requirement (shall) | Verify |
|---|---|---|
| REQ-001 | WHEN the facade's excerpt stage projects any multibuffer model, its outputs — total slot count, locate (Header/Note/Line, `None` only when empty, past-end saturating to the last Line), slot-of-(file,row), and mover/normalize targets (non-Line rows skipped, ends clamped, forward-only normalize) — shall equal the direct #427–#430 model's for the same model. | Equivalence property/table tests over generated + discriminating fixtures (PR-1370: ≥3 out-of-order windows, Note-bearing and note-less files, empty model, adjacent bands, post-`refresh_cum` reshapes); every slot probed ≥2 past range. |
| REQ-002 | The full pre-existing workspace suite shall pass with zero behavioral edits to existing tests; tests re-homed with moved code keep their assertions verbatim. | `cargo nextest run --workspace` green + diff audit at inspect (every existing-test hunk classified move-only). |
| REQ-003 | WHEN the refactor completes, the mb model's raw slot conversions shall have no production consumer in app.rs outside the facade: `.locate(`, `slot_of_row(`, `.total_rows()`, `move_selection(` on the mb model resolve only inside the facade/its absorbed home. | Grep audit — original pattern, ANY receiver (PR-137) — recorded at inspect and re-verified at validate. |
| REQ-004 | The mb rim's slot axis shall be typed `DisplayRow` (model indices `fi`/`li`/`mi` stay `usize` — they are indices, not row spaces); a cross-space row mix at a facade excerpt signature shall fail to compile. | trybuild in `marley_text_offsets`' standing `tests/ui` suite IF a new mix becomes representable; else the existing `mixed_rows_fail` pin covers the axis and the design records why (honestly, at inspect). |
| REQ-005 | The editor tab's fold∘wrap projection shall stay bit-for-bit: the standing display_map unit suite and headless render/geometry drives pass unchanged. | Existing suites in the REQ-002 run; zero display_map.rs editor-path assertion edits. |
| REQ-006 | WHILE the app runs with a materialized search multibuffer AND a problems multibuffer, render/selection/jump/caret shall behave exactly as before, and the new pure surface shall hold cov 100 / MSI 100. | Live smoke drive against the #427/#430 captures + `scripts/gates.sh --diff` green (gate:4/gate:5). |

## Phase Plan

- **P2 Design** — the excerpt stage's shape (third facade arm vs a stage enum; where
  `slots` lives; the typed row vocabulary at the rim; two instances, one facade; the
  per-frame construction cost priced — the index is built at refresh, never per crossing,
  BF-fold-projection-parse-on-pump-tick); the full rim-site manifest; the model-shrink
  call (and the one-slice-vs-split decision if churn demands it); test plan + risks.
- **P3 Implement** — the stage + re-routed sites, `cargo check --workspace --tests` after
  every signature cluster (F-#386); typed fields flipped so the compiler finishes the
  site sweep (L-claude-425).
- **P3.5 Inspect** — independent critics vs the diff; the REQ-003 straggler grep; the
  REQ-002 move-only audit; provenance (no Zed source); the D2 one-home check.
- **P4 Validate** — write + RUN the equivalence property suite and the trybuild call;
  live smoke on both surfaces; `scripts/gates.sh --diff` green.
- **P5 Complete** — archive, ledger capture (§19), CHANGELOG + arch-docs (§21:
  editor.md's display-map and multibuffer sections converge into one stack story), close
  the ticket, sweep BACKLOG.
