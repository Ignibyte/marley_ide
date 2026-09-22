---
pipeline_id: a0b0c5ba-4359-4009-b750-935a3b81b9a7
ticket: forge#336 (c07703c9-e388-471e-b2f1-e005a8178310) · local docs/planning/tickets/open/TICKET-336-hscroll.md
aar_id: 2c3e5411-190b-4131-acff-2b625dbe46fb
status: Phase 5 — Complete PASS
title: Horizontal scroll — an over-wide line's tail is unreachable today (the defect ticket)
type: bug
milestone: M22
references: [the #266 uniform_list render, the #270 caret-follow + per-file scroll memory (#273), the #198 scrollbar_thumb math, AD-claude-two-boundary-maps-for-phantom-text-001, AD-claude-editor-offset-column-model-001, forge#337 (the font-metrics seam this ticket's cell_w will later read)]
---

<!-- PRE-AUTHORED (the Fable method): written in batch at docs/planning/pipeline/queued/m22-hscroll.spec.md
     (committed 2f55c8d) from a live-code read at `main` @ a9eb589, then PROMOTED here by /pipeline:plan,
     which re-verified every cited seam. The Phase 1 verification ledger is in the notes. -->

## Method note (first run of promote-don't-author)
Phase 1 did NOT author this spec — it promoted it and **re-verified all 10 cited seams against live code**.
That verification is the method's load-bearing step: a pre-authored spec is only as good as its citations, and
citing a seam that has since moved would silently poison Design. The ledger (VERIFIED / DRIFTED per claim) is
the Phase 1 notes entry. Everything below this line was written before the ticket started; the phases that
follow CONFIRM these decisions rather than derive them.

## Title
A line wider than the code area CLIPS at the list edge and its tail **cannot be seen or reached by any
means** — there is no horizontal scroll axis at all. `CodeViewState.scroll` is a row index;
`clamp_scroll_px(px, total_rows, cell_h)` (code_view.rs:277) is vertical-only; the row render is
`.whitespace_nowrap()` (app.rs:4469, deliberate — a wrap would paint over the next row's slot). VS Code
clips too (`wordWrap: off` is its default), but it gives a horizontal scrollbar; Marley gives nothing.
This is a **defect**, not a missing convenience — and it is deliberately NOT soft wrap: h-scroll keeps
row↔line 1:1 and needs **no display map**, which is why it ships in B-a while wrap waits on B-c.

## Scope
### In
- **One shift seam, not N rider fixes (D-SHIFT).** ✅ **Phase 1 (F6) found this is on STRONGER footing than
  assumed: the container already exists.** `code_row` (app.rs:4601, built :4465-4473) already wraps ONLY the
  code text, is already `.relative()`, and already sits as a sibling AFTER the git lane (:4578) and gutter
  (:4591) — so REQ-005 (never shift the gutter/lanes) holds by the existing structure, and D-SHIFT only ADDS
  a translate + `overflow_hidden` to it. The riders genuinely ride: the caret bar (:4519, `.left(px(ccol *
  cell.w))`) and squiggle (:4550) are `.absolute()` children in CONTENT coords, and the selection band / #272
  find bands / #331 hint spans are **not elements at all** — they are `HighlightStyle` backgrounds ON the
  text (:4442-4472), so they ride trivially.
  ⚠️ **CORRECTED (F2): it is TWO translate sites, not one.** The #330 sticky header does NOT ride —
  `sticky_header_overlay()` (app.rs:4741) attaches to `body`, not the list (:4702), and builds an independent
  tree with its own gutter + `StyledText` (:4807-4838). It needs its own shift. D-SHIFT still holds (2 sites,
  not N), but "the ONE container" was overstated.
  The OUTBOUND conversions are the real rider surface — and **(F5) there are FOUR click sites, not one**:
  mouse_down (:4606), drag (:4656), the IME `character_index_for_point` (:11010), and the #311 hover-dwell
  (:9513, already col-domain — trace its producer). All bottom out through `offset_for_click`
  (code_view.rs:225). This is the #331 lesson applied in advance: keep ONE coordinate domain and shift at ONE
  boundary — but Design must first settle WHICH domain `x0` is in (see D-X0 below).
- **The pure seam `h_scroll.rs`** (cov/MSI 100): `clamp_scroll_x(x, content_cols, viewport_px, cell_w)`,
  `follow_caret_x(scroll_x, caret_col, viewport_px, cell_w) -> f32` (keep the caret inside
  `[scroll_x + margin, scroll_x + viewport − margin]`, else the minimal adjust; margin ≈ 4 cols), and
  `content_cols(visible rows' display widths, caret row's width) -> usize`.
- **Content width = the max over VISIBLE rows + the caret's row, recomputed per frame** — never an O(file)
  scan. This matches VS Code's OBSERVED behavior (its h-scrollbar length visibly changes as you scroll
  vertically); the caret row is always included so the follow can never be clamped away from its own caret.
- **Wheel/trackpad dx** → `scroll_x`. ⚠️ **CORRECTED at Phase 1 (F1): the editor has NO wheel handler to
  extend** — it was DELETED in #266 (app.rs:4274: *"the old editor wheel handler is gone"*); the editor
  scrolls via gpui's own `uniform_list(...).track_scroll(editor_scroll)` (app.rs:4696). The 3 live
  `on_scroll_wheel` handlers are the rail (:12209), the Files tree (:12531) and the terminal (:13153) —
  none is the editor's. **Design fork:** can an `on_scroll_wheel` on the list/parent observe dx WITHOUT
  fighting `uniform_list`'s internal dy consumption, or is the event consumed before it bubbles? Answer
  empirically. **Fallback if dx is unobservable:** ship wheel-less (follow + thumb-drag carry REQ-001), and
  say so — do NOT fight the list.
- **A bottom h-thumb** when content overflows the viewport (reuse the #198 `scrollbar_thumb` proportional
  math; hidden when content fits). ⚠️ **Phase 1 (F4):** it IS axis-agnostic (pure ratio math over three
  `usize`s — nothing vertical in the body), but it takes **`usize`** while D-CLAMP is in **px f32** → a
  px→cols conversion at the call boundary, and its **sole caller is the TERMINAL pane** (app.rs:13676), so
  there is NO editor-side drag precedent to copy. Draggable v1 only if that cost is small once seen;
  wheel+follow are the load-bearing affordances.
- **Caret-follow on every caret move** (typing, arrows, End, click, F12 land, find next) — the horizontal
  twin of #270, applied in the same place caret-row follow already runs.
- **Per-file memory**: `scroll_x` stores alongside the #273 per-file scroll row and restores with it.
- **The sticky header (#330) and every code row shift together** — one rule, no pinned-at-0 exception.
### Out (explicitly)
- Soft wrap (B-c — needs the display map). The gutter/git/diagnostic lanes never shift (fixed rail). No
  `editor.*` setting — h-scroll is not a mode, it is the absence of a defect. Terminal panes (alacritty owns
  its own grid; no change).

## Reference (§20)
**VS Code = OBSERVED behavior** (clip-not-wrap default; h-scrollbar sized from visible lines, recomputed as
you scroll; caret-follow margins). No copyleft source read. The mechanism is Marley's own layout math over
the shipped `LineLayout` column domain — the same one #331 hardened
(`AD-claude-two-boundary-maps-for-phantom-text-001` — display widths here are CONTENT geometry; the shift
must never fork the column domain, hence one container, not per-rider offsets).

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions (design confirms against live code; deltas → the notes)
- **D-SHIFT** — one translated container per row, code region only, `overflow_hidden`; riders inherit.
- **D-WIDTH** — visible-rows + caret-row max, per frame; documented VS Code-parity quirk.
- **D-CLAMP** — `scroll_x ∈ [0, max(0, content_px − viewport_px + slack)]`, slack = 2 cols; content narrower
  than the viewport pins 0.
- **D-ZERO-IDENTICAL** — `scroll_x == 0` renders byte-identically to today (the OFF-identity discipline; a
  property/headless test pins it).
- **D-MEMORY** — scroll_x joins the per-file store. ⚠️ **CORRECTED (F3): the target is
  `OpenFile.scroll_px: f32`** (editor_surface.rs:53 — a raw px offset, ≤ 0, **session-only, never
  persisted**), NOT `CodeViewState.scroll` (app.rs:388 records that `cv.scroll` now serves ONLY the #246
  read-only split pane). Choke point `sync_editor_scroll()` (app.rs:10525); restore at :10564 is
  `set_offset(point(px(0.0), px(incoming_px)))` — **that hard-coded `px(0.0)` x is the exact slot scroll_x
  restores into.** Session-only matches the existing behavior; persisting across restarts is out.
- **D-X0 (NEW — the Phase 1 F5 fork; Design MUST settle it before implementing the click math).** All four
  click sites subtract `x0`, written by a canvas probe at **app.rs:4496-4500 from `bounds.origin.x` — and
  that canvas is an `.absolute()` child of `code_row`, the element D-SHIFT translates.** If gpui's translate
  moves the canvas's bounds then `x0` already carries `−scroll_x`, `rel` is ALREADY content-domain, and
  REQ-003's `rel_x + scroll_x` **double-counts**; if the translate is paint-only, `+ scroll_x` is right.
  **There are ZERO `.translate`/`TransformationMatrix` uses in the repo — no precedent settles this.** Design
  resolves it EMPIRICALLY (a throwaway probe reading `x0` at a non-zero scroll), and the expected fix is to
  hoist the x0 canvas OUT of `code_row` onto the row div so `x0` stays screen-domain while the text/caret/
  squiggles shift. Whichever way it lands, the answer gets a NAMED invariant + a test, not a comment.

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | render a line wider than the viewport such that increasing `scroll_x` makes its TAIL visible, down to the final glyph at max scroll | pure (clamp+width) + headless |
| REQ-002 | keep the caret visible on every caret move: a caret past the right/left margin adjusts `scroll_x` by the minimum that restores the margin | pure `follow_caret_x` table |
| REQ-003 | map a click at a scrolled position to the char under the pointer, so click/drag/shift-click land identically at any scroll — at **all FOUR** sites (mouse_down, drag, IME `character_index_for_point`, hover-dwell). ⚠️ the `+ scroll_x` form is NOT assumed: D-X0 decides whether `x0` is screen- or content-domain first | pure + headless (+ the D-X0 probe) |
| REQ-004 | clamp `scroll_x` to `[0, content − viewport + slack]`, where content = the max display width over visible rows ∪ {caret row}; narrower content pins 0 | pure truth table |
| REQ-005 | never shift the gutter or the git/diagnostic lanes; only the code region translates | headless/review |
| REQ-006 | shift the caret bar, selection bands, squiggles, find bands and hint spans by exactly the shared `scroll_x` — they are `code_row` children / highlights ON the text, so they ride it (F6-verified) — **AND** shift the #330 sticky header, which is a SECOND site (F2: it hangs off `body`, not the list) | review + headless spot-check |
| REQ-007 | route wheel/trackpad dx to `scroll_x` while dy routing is unchanged — **OR**, if Design finds dx unobservable under `uniform_list`'s dy consumption (F1), ship wheel-less and RECORD that, with follow + thumb carrying REQ-001. Never fight the list. | headless (+ the F1 feasibility answer in the notes) |
| REQ-008 | show the bottom thumb iff content overflows; its length/offset follow the #198 proportional math | pure (reuse) |
| REQ-009 | at `scroll_x == 0`, produce a render byte-identical to the pre-ticket path | property/headless |
| REQ-010 | restore `scroll_x` with the per-file scroll memory on file switch | headless |

## Phase Plan
**P2 — the two Phase-1 forks come FIRST, because both can move the manifest.** (a) **D-X0 (F5)**: probe
whether a gpui translate moves `bounds.origin.x`; decide hoist-the-canvas vs `+ scroll_x`; name the invariant.
(b) **F1 wheel feasibility**: can dx be observed around `uniform_list`? Decide route or fallback. Then the
manifest + the Regression Test Plan (≥1 row per REQ). P3 the pure `h_scroll.rs` seam first, then the two
translate sites (F2), the four click sites (F5), follow, memory (F3 → `OpenFile.scroll_px`'s sibling), and
the thumb (F4's px→cols conversion). P3.5 critics on the click domain (the double-count), the clamp ×
caret-row interaction, the sticky-header second site, and zero-identity. P4 the tables + headless drives +
gate — **LIVE pixel drive is OFF-LIMITS (chad is at the machine; synthetic input hits his frontmost window)**
→ units + headless + mechanism, documented as deferred-not-skipped. P5 docs (editor.md's h-scroll paragraph;
the roadmap's "no soft wrap / tail unreachable" note flips to "h-scroll ships; wrap is B-c").
Standing traps: the batch list in [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
