# 432-multibuffer-input-completeness — pipeline notes

## Phase 1 — Plan (2026-08-15)

**Request.** Close the #428 recorded v1 seams on the editable multibuffer (TICKET-432,
M33 shelf tail item 2): (a) precise click-column caret placement, (b) ⌘V paste through the
ONE insert mechanism (multi-line grows the window like Enter), (c) IME composition on the
mb's own `handle_input` canvas. Spec drafted QUEUED per the shelf's promote-one-at-a-time
Fable→Opus workflow; authored fresh against the post-#430 tree.

**Classification.** Work pipeline, feature, M — three independent input arms on shipped
infra (no new machinery, no model changes beyond a geometry cell); one shippable slice.

**Recall (§18.3).** AD-claude-428-excerpt-editing-rides-existing-machinery-001 (the
frame: every arm reuses the #428 chassis). F/PR-claude-428-a (the input seam is per-surface
and ALREADY registered for the mb — #428 inspect C1; validate must still land a LIVE
typed/composed char, direct method calls mask the seam). F/PR-claude-428-b (read-side IME
queries must answer from the mb caret anchor + `mb_marked`, never the buffer's live
`SelectionSet` — the park/restore choke does not fire on mb frames). F-claude-428-c (paste
journals only through `mb_after_edit`'s real-edit + depth-keyed guards — no new journal
logic). AD-claude-two-boundary-maps-for-phantom-text-001 + PR-claude-phantom-text-forks
(click/caret-bar are CARET-map consumers; mb rows have no phantoms today — classification
recorded against future mb inlays). BF-claude-click-col-quantized-before-nearest-scan
(float domain until the final compare; `offset_of_col_f`, never round-then-scan).
PR-claude-composite-type-over-one-edit-001 (paste = ONE `edit_at_selections`, one undo
unit; pin with an undo-count assert). PR-claude-new-chord-shadowed-by-hardcoded-key-001
(the ⌘V arm's ladder placement needs the hardcoded-intercept sweep; drive the real
keystroke). BF-claude-overlay-keys-leak-keychar-to-ime-fallback (consuming arms
stop_propagation or the IME fallback re-delivers). L-claude-426-live-drives-need-a-click-
and-reseeded-geometry-001 (headless geometry seeds reset on ANY render — seed last; live
drives click-into-surface first). PR-claude-pin-word-class-and-merge-semantics (the
multibyte fixture: char-end ≠ byte-end, e.g. `aébc`).

**Discovery (exact seams, post-#430 tree).**
- Click: `multibuffer_body` `app.rs:6406`; cell_w already measured `app.rs:6427-6434`
  (`em_advance` at Command size); row = px_2 + `gutter_label(row+1, 5)` (`app.rs:6603-6605`,
  fixed 5-wide, `code_view.rs:540`) + gap_2 + text cell `app.rs:6607-6630`; caret bar =
  raw char-col × cell_w `app.rs:6619` (no LineLayout — switches to `col_of_offset`); the
  click handler DISCARDS the event (`_e`) and calls `mb_place_caret(id, slot)`
  `app.rs:6631-6639`; `mb_place_caret` anchors at line END `app.rs:13537-13573` (its own
  doc: "precise column mapping rides the geom hook later"). Editor recipe to reuse:
  recorded `editor_geom` (`EditorFrameGeom` `app.rs:521-526`, canvas recorder
  `app.rs:7458-7494`, test seed `seed_editor_geom_for_test` `app.rs:19023`) → float col →
  `offset_for_click` `code_view.rs:248` → `offset_of_col_f`.
- Insert chassis: `mb_edit_prepare` `app.rs:13236`; `mb_edit_transient` `app.rs:13320`
  (saves/restores the live set; third closure param is `&mut self.mb_marked` — shipped
  waiting for IME); `mb_after_edit` `app.rs:13253` (real-edit-only journal, touched, caret
  re-anchor); `mb_replace_text` `app.rs:13350` (commit arm, live since #428);
  `mb_newline` `app.rs:13520` (the window-growth precedent: one `edit_at_selections("\n")`
  + `sync_multibuffer_live` `app.rs:14037` rebuild over grow-at-edges anchors
  `multibuffer.rs:119-122`).
- Paste ladder: editor ⌘C/⌘X/⌘V block `app.rs:20675-20730` (paste = `clear_marked` + ONE
  `edit_at_selections`, `EditOrigin::Human` — the shape to mirror), gated on
  `active_editor().is_some()`; on an mb tab it SKIPS and ⌘V falls through to the terminal
  arm `app.rs:20745-20767` via `focused_terminal_mut` (`app.rs:6308` →
  `workspace.rs:764` — workspace-pane-resolved, NOT tab-gated) — **today an mb-tab ⌘V
  types the clipboard into the invisible terminal prompt/PTY** (and ⌘C copies the hidden
  terminal's selection, `app.rs:20734`). #428 recorded the family as "dead"; ladder trace
  shows misdirected, worse — named in the spec as a fail-closed REQ. The mb bare-key arm
  `app.rs:20588-20601` is modifier-gated and never sees ⌘-chords.
- IME: `EntityInputHandler` impl `app.rs:19643-19871` — `replace_text_in_range` has the mb
  arm (`app.rs:19712-19717`); `replace_and_mark_text_in_range` (19781), `marked_text_range`
  (19675), `selected_text_range` (19660), `text_for_range` (19645), `unmark_text` (19688),
  `bounds_for_range` (19806, editor_geom-backed), `character_index_for_point` (19837) are
  all editor-only. The mb canvas registration exists (`app.rs:6684-6697`, #428 inspect C1).
  `mb_marked` `app.rs:571`, cleared at arrows/Esc/undo/redo/place-caret
  (`app.rs:13185,13203,13699,13807` + the #428 inspect LOW fix). `marley_editor::ime`:
  `edit_target` (explicit→marked→selection→caret, clamped), `replace_and_mark` `ime.rs:167`
  (no-op guard, sets marked + selection — head feeds the caret re-anchor),
  `marked_utf16`/`selected_utf16`/`text_for_range`/`utf16_ix_to_char_ix`. Subtlety pinned:
  after every mb edit the live set is RESTORED, so `selected_utf16(buffer)` would report
  the FILE TAB's selection — the mb read arms must derive from the caret anchor
  (`char_to_utf16(off)` point range; composition-internal ranges collapse to the head, v1
  recorded).
- `text_input_blocked` `app.rs:11631` — overlay roster only; no change (the #428 finding
  holds).

**Decisions.** D1 one column map (editor's `LineLayout`, float; bar switches to
`col_of_offset`); D2 mb geometry cell via the `editor_geom` canvas idiom + a test seed;
D3 paste = clear composition + ONE `edit_at_selections` on the transient set → `mb_after_edit`
(multi-line un-special-cased; growth = Enter's anchor arithmetic); D4 IME via the standing
`ime` fns + `mb_marked`, read arms anchor-derived; D5 fail-closed ladder (no plain-⌘C/X/V
reaches the hidden terminal from an mb tab; stop_propagation on consuming arms); D6 no
render/journal/guard machinery changes. Out: rich clipboard, column-select paste,
cross-file multi-paste, ⌘C/⌘X function, mb selection model, marked-underline styling,
tab-expanded row render (recorded #427 caveat — the map is tab-aware so the click↔bar
round-trip stays self-consistent).

**Prior-art verdicts (§20).** (1) Zed behavior maps: `03-editor-multibuffer.md` covers
the input path generically (two entry points; `replace_text_in_range` +
`replace_and_mark_text_in_range` as the protocol; paste = the same per-selection insert;
"adopt the gpui trait verbatim, back with `Buffer::edit`") — no multibuffer-specific input
chapter exists; the excerpt surface inherits the editor contract, which is this ticket.
(2) Published: gpui's `InputHandler` self-documents as a 1:1 NSTextInputClient exposure
(gpui-0.2.2 `src/platform.rs`; UTF-16 ranges, `bounds_for_range` =
`firstRectForCharacterRange` for the candidate window) — `marley_editor::ime` already
encodes the target-resolution and document-absolute-range semantics; nothing new needed.
(3) Permissive deps: gpui (Apache-2.0, crates.io) IS the seam — `EntityInputHandler`
(`src/input.rs:10`) + per-frame `ElementInputHandler` registration; reading it is adoption,
outside the clean-room wall. In-tree: everything exists (chassis, `mb_marked`, ime fns,
click recipe, geom idiom, window growth) — the ticket is arms + one geometry cell.

**React-first judgment (honest).** UI-AFFECTING zone B. Click-column: the POC already
ships it (`MultibufferView.tsx:222-231`, CELL_W 7.83 / GUTTER_W 48, clamped round;
caret bar at `col × CELL_W`) and MARLEY-PARITY's row records "POC mono-col click vs Rust
line-end caret" as the standing gap — Rust closes it against the POC reference (POC
`Math.round` is a stand-in; Rust uses the real float-midpoint inversion). Paste: a REAL
POC delta — no paste/clipboard handling exists in the POC keydown surface
(`MultibufferView.tsx:62-167`) — built React-first through the existing `writeThrough`
(multi-line splits rows like its Enter arm, lines 94-118). IME: no POC twin is buildable
(window-keydown splicing hosts no browser composition) — recorded Rust-only, parity-N/A
for that leg only.

Ticket stays in `tickets/open/`; spec QUEUED (promote to active/ for Phase 2 per /work).

## Phase 2 — Design

Designed at HEAD bd79306 — **after #431 landed**, so the Phase-1 seam lines shifted ~+4
and three seams CHANGED SHAPE (re-verified live): `mb_place_caret(mb, slot: DisplayRow)`
(typed since #431), the render's slot handling is typed (`DisplayRow::from(slot)` mint at
the rim), and mb slot lookups go through `m.display_map()` doors (`excerpt_slot_of_row` /
`excerpt_locate`) — the #431-adjacent re-check the spec's Out note demanded. Re-read
whole: the `EntityInputHandler` impl (app.rs:19647-19875), the mb chassis
(:13240-13400), the ⌘C/X/V ladder (:20679-20771), `EditorFrameGeom` (:873-887) + seed
(:19027), `offset_for_click`/`col_of_offset` (code_view.rs:248/:72).

### Architecture (per arm; §20 confirmed — the gpui-trait/Zed-doc input contract, no
Zed source; React-first zone B stands with the honest three-way split)

- **D-A — the mb geometry cell.** New `MbFrameGeom { x0, y0, first: DisplayRow,
  last: DisplayRow, cell_w, cell_h }` in a `Cell` beside `editor_geom` (a SEPARATE cell —
  both surfaces can be alive in different panes; the editor's stays untouched). CONTRACT:
  `x0` = the text cell's left edge in window px (after px_2 + 5-wide gutter + gap_2);
  `y0` = the FIRST VISIBLE slot's top; `first`/`last` = the uniform_list render range
  (end-exclusive). Recording: `first`/`last` + `cell_w`/`cell_h` inside the
  `multibuffer_body` closure (it owns `range`); `x0`/`y0` by the `editor_geom` zero-size
  canvas idiom on the list/text cell (implement picks the exact probe point; the CONTRACT
  above is what tests pin). Reads outside paint use the last frame (the editor_geom
  discipline); zeroed cell (pre-first-paint) ⇒ every consumer answers `None`/no-op.
  Test seed `seed_mb_geom_for_test(first, last, cell_w, cell_h, x0, y0)` — the
  `seed_editor_geom_for_test` sibling; L-claude-426: seed LAST before each probe.
- **D-B — click column.** The Line-row click closure stops discarding the event: from
  the recorded `MbFrameGeom`, `col_f = (event.position.x − geom.x0) / cell_w` (FLOAT —
  BF-claude-click-col-quantized), passed as `mb_place_caret(mb, slot, Some(col_f))`. The
  signature grows `col: Option<f32>`; `None` = the shipped line-end arm (kept for any
  non-click caller). Inside: `in_line = col.map(|c| offset_for_click(&line, c,
  self.code_tab_width)).unwrap_or(line_end)` — `offset_for_click` owns tab-awareness,
  wide-glyph midpoint, multibyte, and the [0, len] clamps (gutter-left → 0; past-end →
  len). ONE column map (D1): the same `LineLayout` inversion the editor click uses.
- **D-C — caret-bar agreement.** The bar's `left(px(ccol × cell_w))` (raw char col)
  switches to `line_layout(&line.text, tab_width).col_of_offset(ccol) × cell_w` — the
  same layout's forward map, computed only on the one caret row per frame. Click → caret
  → bar round-trips in one pixel domain (REQ-001's second half).
- **D-D — ⌘V + the fail-closed ladder.** A new arm in the key handler DIRECTLY AFTER
  the editor ⌘C/X/V block (:20679-20734, gated `active_editor().is_some()` — editor
  tabs unchanged) and BEFORE the terminal ⌘C/⌘V arms (:20738/:20749 — today's
  misdirection): `platform && !shift && key ∈ {c,x,v} && active_mb_id().is_some()` ⇒
  ⌘V runs `mb_paste`; ⌘C/⌘X are CONSUMED no-ops (no selection model — nothing copies,
  and nothing reaches the hidden terminal). Consume discipline mirrors the sibling
  editor block exactly (same return path — the BF-overlay-keys-leak rule rides the
  handler's standing shape). `mb_paste`: clipboard text (`None`/empty ⇒ consumed
  no-op) → `mb_edit_prepare` (`None` — no caret ⇒ consumed no-op) → `self.mb_marked =
  None` FIRST (paste ends composition, the #267 rule, BEFORE the edit — R2) → ONE
  `mb_edit_transient(tid, off, |buffer, set, _| buffer.edit_at_selections(set, &text,
  EditOrigin::Human))` (PR-claude-composite-type-over-one-edit; multi-line is the SAME
  edit — growth is the standing #428 anchor arithmetic + next-frame
  `sync_multibuffer_live`, `mb_newline`'s exact precedent) → `mb_after_edit` (journal/
  touched/caret-after — F-claude-428-c guards ride unchanged).
- **D-E — the IME arm table** (every arm behind `text_input_blocked()` first, then the
  `active_editor()` else-arm — the shipped `replace_text_in_range` routing shape):
  | method | mb arm |
  | `text_for_range` | target buffer via `mb_edit_prepare` → `ime::text_for_range` (document-absolute over the TARGET) |
  | `selected_text_range` | caret POINT range: `buf.char_to_utf16(off)`, reversed false — from the ANCHOR, never `selected_utf16(buffer)` (PR-claude-428-b; composition-internal selection collapses to the head, v1 recorded) |
  | `marked_text_range` | `ime::marked_utf16(target_buf, self.mb_marked)` |
  | `unmark_text` | `self.mb_marked = None` + notify |
  | `replace_text_in_range` | SHIPPED (`mb_replace_text` — the commit arm) |
  | `replace_and_mark_text_in_range` | prepare → `mb_edit_transient(.., |b, _s, marked| ime::replace_and_mark(b, marked, range, new_text, new_sel))` → `mb_after_edit` (the transient's third param exists for exactly this; head re-anchors the caret; coalesced updates keep ONE journal entry via depth-keyed dedupe — R4, validate pins one-⌘Z-per-composition) |
  | `bounds_for_range` | caret → target `line_col` → `m.display_map().excerpt_slot_of_row(fi, row)` (the #431 door) → in `[first,last)` ⇒ rect `(x0 + col_of_offset(col)×cell_w, y0 + (slot−first)×cell_h, cell)`; `None` off-viewport / zeroed geom |
  | `character_index_for_point` | point → `slot = first + dy/cell_h` (clamped) → `excerpt_locate` → `Row::Line(fi, li)` ON THE CARET'S OWN FILE (`fi` mismatch ⇒ `None`, v1 recorded — R5) → float col → `offset_for_click` → `char_to_utf16` |
- **D-F — no model changes.** multibuffer.rs untouched; every arm is app.rs wiring over
  the #428 chassis + the #431 doors.

### File manifest

- **React half (built + captured FIRST at implement):**
  `marley-web/artifacts/marley-ide/src/views/MultibufferView.tsx` — a ⌘V paste arm in
  the keydown surface through the existing `writeThrough`: single-line splices at the
  caret col; multi-line splits the row + renumbers exactly like its Enter arm
  (lines 94-118). Screenshot + READ; typecheck green. (Click-column already ships in the
  POC — the standing reference; IME has no POC twin — recorded Rust-only.)
- **Rust half:** `crates/marley_app/src/app.rs` ONLY — `MbFrameGeom` + cell + recorder
  in `multibuffer_body` + `seed_mb_geom_for_test`; the caret-bar `col_of_offset` switch;
  the click closure passes the float col; `mb_place_caret` grows `col: Option<f32>`;
  `mb_paste`; the ladder mb arm; the six new `EntityInputHandler` mb arms +
  `bounds_for_range`/`character_index_for_point` mb arms. `headless_drive.rs` — the new
  drives (written at validate). `code_view.rs`/`multibuffer.rs` — untouched.

### Regression test plan

| REQ | Test | How |
|---|---|---|
| REQ-001 | click-column mapping units + drive | pure asserts on `offset_for_click` fixtures incl. `aébc` (char≠byte), a wide glyph (midpoint flip), a tab line; headless: seed mb geom → `mb_place_caret(slot, Some(col))` for plain / multibyte / past-end→len / negative→0; caret-bar col via `col_of_offset` asserted on a tab-bearing line; LIVE click at a measured x → bar at that column |
| REQ-002 | single-line paste drive | headless: arm caret → clipboard "XYZ" → real `cmd-v` keystroke → target text spliced at caret; journal len 1; ONE ⌘Z reverts; ⌘⇧Z restores |
| REQ-003 | multi-line paste drive | same with "a\nb" → rendered rows grew (`display_map().excerpt_total()` before/after) + window edges re-resolved; single ⌘Z reverts whole paste |
| REQ-004 | IME drives | headless: real `EntityInputHandler` calls — compose (`replace_and_mark`) → update → commit (`replace_text_in_range`) ⇒ exactly one `é`, marked/selected/text_for_range answered from anchor+`mb_marked`; compose → Esc drops preedit…; `bounds_for_range` exact rect on seeded geom + `None` off-viewport; `character_index_for_point` inverse + cross-file `None`; LIVE: `alt:e` then `e` composes on the mb surface (drive.swift's shipped verb; PR-claude-428-a demands the live leg) |
| REQ-005 | fail-closed ladder drives | headless: mb tab + live terminal → clip + `cmd-v` ⇒ terminal prompt buffer UNCHANGED, no PTY write, mb target got the clip; no-caret `cmd-v` ⇒ zero edits anywhere, consumed; `cmd-c`/`cmd-x` on mb tab ⇒ clipboard unchanged, terminal copy arm never runs |
| REQ-006 | gate | `scripts/gates.sh --diff` GREEN (cov/MSI 100 on any new pure surface) |
| parity | POC paste ↔ live paste | POC capture (⌘V two-line clip in MultibufferView) ↔ live capture at the same state; pixel-sample per MARLEY-PARITY; click gap re-captured CLOSED |

Uncoverable: the macOS IME candidate WINDOW itself (platform chrome — needs a real
input-method session; the live dead-key compose covers the observable text path).

### Risks

- **R1 — ladder ordering**: the mb arm must land between the editor block and the
  terminal arms; implement re-runs the hardcoded-intercept sweep
  (PR-claude-new-chord-shadowed) for any OTHER `"c"|"x"|"v"` interception above it
  (modal overlays consuming first is existing, correct behavior).
- **R2 — order of clear-vs-edit**: `mb_marked = None` BEFORE the paste edit, or the
  splice lands inside a live marked span.
- **R3 — geometry staleness**: mb_geom is render-recorded; IME queries between frames
  read the last frame (the editor_geom contract); zeroed ⇒ None. The #431 model pairing
  invariant is untouched (geometry is render-side).
- **R4 — composition journaling**: per-update version bumps coalesce into one undo
  group; `journal_note_edit`'s depth-keyed dedupe keeps ONE entry — validate pins
  one-⌘Z-per-composition (else it's a real finding, not a test to soften).
- **R5 — cross-file point queries** answer `None` (v1; recorded here + in the arm doc).
- **R6 — caret-drop paths keep clearing composition** (arrows/Esc/undo/redo/place-caret
  already clear `mb_marked` — new click-with-column path keeps that behavior).

## Phase 3 — Implement

**React-first (the paste leg, built + verified FIRST):** the ⌘V arm landed in
`marley-web/.../components/views/MultibufferView.tsx` (path note: the file lives under
`src/components/views/`, not the Phase-1 shorthand `src/views/`) — clipboard read in the
keydown handler, single-line splice at the caret col, multi-line via
`newLines = [head+first, …middles, last+tail]` with rows below renumbered `+grow`
(the Enter arm's exact growth), caret after the inserted text, a caret-moved-mid-read
guard on the async clipboard. Typecheck green. DRIVEN + captured at 5173 (Playwright):
clicked col 14 of `let value = compute(input); // 88`, pasted `PASTED_ONE\nPASTED_TWO`
→ row 88 `…compPASTED_ONE`, NEW row 89 `PASTED_TWOute(input); // 88` with the caret
bar after `PASTED_TWO`, context rows renumbered 90/91, the diagnostic band stayed
anchored, other groups untouched. Captures (READ):
`scratchpad/432-poc-mb-before.png`, `scratchpad/432-poc-paste-after.png`.

**Rust half (app.rs only, per manifest):**
- `MbFrameGeom` + `mb_geom: Rc<Cell<…>>` (+ init) — the editor-geom twin; recorded once
  per batch by a zero-width, row-tall canvas probe on the FIRST LINE row's text cell
  (headers/bands have no text cell — the probe waits for a Line row; `y0` normalizes to
  slot `first`'s top via the uniform pitch; the probe's own height IS `cell_h`, so no
  separate line-height derivation exists to drift).
- `caret_pos` now carries the DISPLAY column (`line_layout(...).col_of_offset` once per
  frame for the one caret row); the bar paints at it — click↔bar share ONE map (D1).
- The click closure reads the recorded geometry at click time → unrounded
  `col = (x − x0)/cell_w` → `mb_place_caret(id, slot, Some(col))`; zeroed (pre-paint)
  geometry passes `None`.
- `mb_place_caret(…, col: Option<f32>)` — `Some` inverts through `offset_for_click`
  (tab-aware, wide-glyph midpoint, [0, len] clamps); `None` keeps the line-END v1 arm
  (all pre-existing drive callers pass `None` — assertions preserved).
- `mb_paste` + the ladder arm: platform+!shift `c|x|v` with an active mb tab consumes
  BETWEEN the editor block and the terminal arms — ⌘V through
  clear-composition → ONE `edit_at_selections` → `mb_after_edit`; ⌘C/⌘X consumed
  no-ops. The hidden-terminal misdirection is closed.
- The six `EntityInputHandler` mb arms per the D-E table (text_for_range /
  selected_text_range [anchor point-range] / marked_text_range [`mb_marked`] /
  unmark_text / replace_and_mark [through the transient chassis' third param] +
  `bounds_for_range` and `character_index_for_point` over the recorded mb geometry
  with the #431 doors; cross-file point → `None`).

**Deviation (recorded):** `seed_mb_geom_for_test` was written here but REMOVED before
close — clippy `-D warnings` correctly flagged it dead (its first consumers are the
Phase-4 drives). It re-lands at validate WITH the drives, keeping every commit-point
state honest (no dead code, no false allow).

Checks: `cargo check --workspace --tests` green; `cargo clippy --workspace
--all-targets -D warnings` green; `cargo fmt --all` applied.

## Inspect (Phase 3.5)

Four critics (routing/ladder, IME-state, provenance/security, simplification) — two
died mid-run to a usage limit and were RESUMED against the fixed tree after the first
fix round (their partial-then-final coverage is complete; both re-verified every
earlier fix). Lead verified each finding independently. The ledger, by severity:

1. **[MED · provenance] ⌘⇧V still reached the hidden terminal from an mb tab** — the
   arm's `!shift` gate let "paste without formatting" muscle memory fall through to
   the terminal ⌘V arm (no shift check there). REAL → **FIXED**: the mb arm consumes
   ALL modifier variants of platform c/x/v. Shadow audit
   (PR-claude-new-chord-shadowed) re-run by two critics independently: ⌘⇧C is the
   git-pane arm's ABOVE; the keymap binds zero c/x/v chords; the newly-consumed
   variants previously either misdirected (⌘⇧V/⌘⌥V → PTY; ⌘C variants → hidden-pane
   clipboard clobber) or no-op'd. Nothing legitimate lost.
2. **[MED · routing] PLAIN keys on an mb tab double-delivered into the hidden
   workspace terminal** (pre-existing since #428, not introduced here — but this
   ticket owns the fail-closed seam): a printable fed the mb caret through the
   platform text path AND fell the ladder to the R29 tail, typing the same char into
   the invisible prompt — or streaming it to a live PTY mid-command; shift-arrows
   scrolled the hidden scrollback. REAL → **FIXED**: an mb plain-key claim, the
   exact #251 editor twin (`active_mb_id + !platform + !control` → bare `return`, no
   stop_propagation so the platform path still delivers printables/IME).
3. **[MED · IME] the chord-verb choke ended only the EDITOR's composition** — a tab
   switch away from a composing mb left `mb_marked` standing (the mb caret survives
   switches), and the next commit spliced at stale offsets. REAL → **FIXED**: the
   choke's else-arm clears `mb_marked` when an mb tab is active.
4. **[MED-adjacent · lead's own find] `mb_place_caret` NEVER cleared the
   composition** — REQ-005 lists click among the caret-drop gestures and the #428
   notes claimed the clear existed; `git show HEAD` proves it never did (a stale
   OFFSET span would splice the next IME update cross-file in the worst case).
   **FIXED**: cleared on a successful place (inside the anchor arm — a failed place
   moves neither caret nor span; both resumed critics re-verified placement).
5. **[LOW · routing+IME] ←/→/⌫/⌦/⏎ kept the span** while moving/editing at the
   caret (up/down/Esc already cleared). REAL → **FIXED**: clears in each router arm
   (#267 discipline, editor-symmetric).
6. **[LOW · IME] the epoch re-mint dropped the caret but not the span** — the
   "caret and composition die together" invariant broke on reload. REAL → **FIXED**
   beside the caret drop.
7. **[LOW · routing+provenance] the EDITOR-tab ⌘⇧V residual** (the editor block
   keeps `!shift`; the fall-through types into the hidden terminal) — pre-existing,
   out of this ticket's surface (D3 no riders). RECORDED as a follow-up candidate
   for the next ladder ticket; noted in the F- append.
8. **[NOTE · IME] "one ⌘Z per composition" re-scoped, not softened**: UndoHistory
   coalesces only contiguous single-char inserts, so a multi-update composition
   (に→にほ→commit) is N groups = N mb ⌘Z steps — byte-for-byte the EDITOR's accepted
   ladder (pinned by its own `undo_ladder_through_composition` test). The dead-key
   flow (⌥E→e) coalesces to ONE entry. Validate pins the dead-key flow at one entry
   and asserts the multi-update ladder matches the editor's, exactly.
9. **[LOW ×7 · simplification] all applied**: slot-from-y spelling aligned to the
   editor twin (`geom.first + …`); the probe doc names its LOAD-BEARING row-tall
   height (a "simplify to zero-size" trap); the `.max(0.0)` one-spelling fix; dead
   Copy rebindings removed; zeroed-geom guards hoisted above `mb_edit_prepare` in
   both geometry arms; `mb_paste` dropped to private; `mb_caret_buf` read helper
   collapses the three simple read arms (write arms keep `prepare` — owned ids
   across `&mut self`).
10. **Hardening beyond the ask**: a batch with NO Line row now ZEROES `mb_geom`
    (headers/footer-only ranges previously left a stale prior range for IME rects —
    the routing critic verified the intra-frame ordering is safe and the residual is
    one cosmetic frame).
11. **Rejected with reasons**: merging the mb ladder arm into the editor block
    (one-block-per-surface grain; docs entangle); extracting a shared editor/mb
    geometry helper (the arms genuinely fork — h-scroll/segments/folds vs flat;
    false coupling for net-zero lines); `EditorFrameGeom` reuse for the mb cell (a
    dead `code_w=0` would be a live trap). RECORDED-only: the React chord-guard
    asymmetry (POC `!ctrl&&!alt`; consumption is Rust-side fail-closed) and the POC
    splice-spread pathological-clip limit (POC-only, inert).
12. **Clean (multi-critic, evidence-backed)**: clean-room (zero Zed vocabulary; the
    `ime_transaction` concept in the deconstruction doc deliberately NOT imported —
    undo rides the #428 journal); no clipboard→PTY path (zero new
    `write_bytes`/`session.` calls); no new panic surface (all casts/underflows
    guarded — `slot < geom.first` precedes the typed Sub; `offset_of_col_f` clamps
    negatives); secrets clean both repos; gates untouched, one new `mutants::skip`
    in the standing shim style; PR-claude-428-b honored in every read arm
    (compiler-certified borrow split in `mb_edit_transient`; head read BEFORE the
    selection restore); no-op edits journal nothing; the click/bar round-trip shares
    ONE origin (both live in the same relative text_cell) and ONE map
    (`LineLayout` forward + inverse); `text_input_blocked` consistency matches the
    editor arms method-by-method; the five-tab-scenario ladder trace ends with NO
    ⌘-c/x/v path from an mb tab reaching `focused_terminal_mut`.

Post-fix verification: `cargo check --workspace --tests`, `clippy -D warnings`,
`fmt` all green; scoped mb/problems/display_map suites **57/57** (the real-keystroke
write-through drives included — the plain-key claim preserves platform-path
delivery).

## Phase 4 — Validate

**New drives (7, headless_drive.rs, all green first run)** —
`seed_mb_geom_for_test` re-added WITH its consumers (the Phase-3 deviation closed):
- `mb_click_column_places_caret_headless` (REQ-001) — 3.4→3 nearest-boundary,
  99→line-end clamp, −2→0 clamp, `None`→line-end v1 arm.
- `mb_click_column_is_char_domain_on_multibyte_headless` (REQ-001) — `aébc xyz`:
  click 1.6 → CHAR col 2 (not a byte index); the bar's forward map agrees.
- `mb_paste_single_line_journals_once_headless` (REQ-002) — real `cmd-v`: splice at
  col 5, journal len 1, one ⌘Z reverts, ⌘⇧Z restores.
- `mb_paste_multiline_grows_window_headless` (REQ-003) — "ONE\nTWO" is ONE edit:
  `excerpt_total` +1 (the window grew), one journal entry, one ⌘Z reverts whole.
- `mb_ime_composition_and_geometry_headless` (REQ-004) — REAL `EntityInputHandler`
  calls: compose→update→commit = exactly one é (marked (16,17) pinned; the read arms
  answer from anchor+`mb_marked`); `bounds_for_range` exact rect on seeded geom
  (x0+col×cell_w, y0+(slot−first)×cell_h) + off-viewport None;
  `character_index_for_point` inverse + cross-file None.
- `mb_composition_dies_with_caret_gestures_headless` (REQ-004/5 + the inspect
  finds) — ← clears the SPAN while keeping the caret; a click clears; the next
  commit lands at the NEW caret (no stale-span splice — `Zepsilon` pinned).
- `mb_clipboard_and_plain_keys_fail_closed_headless` (REQ-005 incl. the inspect
  ⌘⇧V + plain-key finds) — ⌘V AND ⌘⇧V paste into the mb (2× "LEAK" in the target);
  the hidden terminal prompt is BYTE-UNCHANGED across ⌘V/⌘⇧V/plain-chars/no-caret-⌘V;
  no-caret ⌘V is a consumed no-op; ⌘C keeps the clipboard sentinel (no hidden copy).

**Suites (real runs in transcript):** new drives 7/7; full
`cargo nextest run --workspace` **2251/2251 PASS** + doctests clean.

**LIVE drive (bundled app, real pixels — captures in the session scratchpad, all
READ):**
- `432-live-click.png` — clicked mid-word on row 510: the caret bar landed AT THE
  CLICKED COLUMN (`fn loc(m|: &Multibuffer…`), not line end — REQ-001 live.
- `432-live-paste.png` — ⌘V of `LIVE1\nLIVE2` (pbcopy-seeded): row 510 became
  `fn loc(mLIVE1`, a NEW row `LIVE2: &MultibufferModel…` grew the window with the
  caret after the pasted tail, rows below renumbered (511→512, 513→514), the dirty
  dot on the band — REQ-002/003 live, the POC's exact growth grammar.
- `432-live-ime.png` — ⌥E then e composed exactly one `é` at the caret
  (`LIVE2é:`) through the real macOS dead-key path — REQ-004 live
  (PR-claude-428-a's live-leg demand met).
- `432-live-undone.png` — ⌘Z ×5 restored the original surface (rows/renumbering
  reverted); no ⌘S issued, disk untouched.
- Harness notes: the fresh bundle re-trips the removable-volume TCC dialog (click
  Allow via UserNotificationCenter + relaunch); first launch can wedge windowless —
  kill + relaunch (the #431 recipe, reconfirmed).

**Parity pair (React-first, REQ + MARLEY-PARITY):** `432-poc-paste-after.png`
(implement, READ) ↔ `432-live-paste.png`. STRUCTURAL: identical paste grammar —
split at the click column, tail row minted, rows below renumbered, caret after the
tail, band dirty dot. PIXEL (magick-sampled): body (11,12,15)↔(14,15,17), group
band (26,27,31)↔(25,26,28), match wash (17,30,34)↔(18,28,31) — all three are the
#430-pinned Δ≤4 token families, verbatim. The MARLEY-PARITY "POC mono-col click vs
Rust line-end caret" gap is CLOSED from the Rust side (the click capture). Verdict:
**PASS**. IME stays recorded Rust-only (no POC composition host — per the spec).

**Gate:** `scripts/gates.sh --diff` → first run RED on gate:1 alone (rustfmt drift in
the appended drives — the #431 recipe repeat); `cargo fmt --all` (verified `--check`
clean, drives re-run 7/7), full re-run: **GATE GREEN [diff] — 15/15**, receipt
written (2026-08-15 11:25). Coverage ≥100 / MSI ≥100 on the diff held both runs.

## Phase 5 — Complete

- **§21 (a) CHANGELOG**: entry under Unreleased → Added (the three seams + the
  inspect's fail-closed harvest + the proof chain).
- **§21 (b) Architecture**: `docs/marley_architecture/editor.md` — the **#432**
  paragraph (geometry cell, one column map, the paste chassis ride, the IME arm
  table's shape, the three-MEDIUM fail-closed harvest); the deferred list re-cut
  (three seams gone; the editor-tab ⌘⇧V residual + the no-selection-model ⌘C/⌘X
  recorded).
- **§21 (c) Parity sync**: `marley-web/docs/MARLEY-PARITY.md` MultibufferView row —
  **#432 ✅ LANDED 2026-08-15**: the mono-col click gap CLOSED (Rust side), paste
  React-first + pixel-paired, IME/fail-closed-ladder/undo recorded Rust-only;
  remaining 428-era gaps re-listed. The POC paste arm shipped at implement is the
  lasting React twin — no back-port needed (the Rust port followed it 1:1).
- **Knowledge (§19)**: inspect appended `F-claude-432-a` (composition outlived its
  caret — the false #428 claim), `F-claude-432-b` (plain-key double-delivery),
  `PR-claude-fail-closed-claims-enumerate-every-input-lane-001`; complete appends
  `AD-claude-432-mb-input-rides-recorded-geometry-and-a-surface-claim-001` and
  `L-claude-432-resume-a-dead-critic-with-a-fix-delta-note-001`.
- Ticket closed → `tickets/closed/`; BACKLOG swept at promotion (no stale row);
  pipeline pair archived to `completed/`.
