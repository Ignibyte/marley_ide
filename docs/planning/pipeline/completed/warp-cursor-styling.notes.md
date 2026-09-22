# Warp cursor & selection styling — Notes

- **Forge ticket:** #220 (6172b7fc-2430-425b-af71-3b0b7cea493d)
- **AAR:** 79dfdb78-0b12-4a7f-af7c-653ba2c48791
- **Local ticket doc:** docs/planning/tickets/open/TICKET-220-warp-cursor-styling.md
- **Pipeline spec:** warp-cursor-styling.spec.md

## Phase 1 — Plan
- **Request:** match Warp's cursor + selection. Auto-approved (/work 195-222). NOTE the ticket desc is
  STALE (the "2px bar" caret was already made a block by #218).
- **Classification / tier:** work pipeline, small/bounded. Systems: app.rs cursor render (shim, focus
  branch) + a possible pure `selection_tint` helper (only if D-B calibrates).
- **Forge recall (§18.3):** #218 shipped the block cursor (`split_caret_char` + reverse-video); #191
  established `is_focused` (`pane_id == focused`) — the exact flag D-A needs, already at the render site.
  #194's color helpers (`contrast_ratio`) + the exact-value/f32-not-mutated rules apply if D-B calibrates.
  AAR opened.
- **Discovery (measured, the edit surface for Design):**
  - The prompt block cursor (app.rs ~4655, from #218): inside the inner gapless flex,
    `div().bg(colors.accent).text_color(colors.background).child(at.unwrap_or_else(|| " ".to_string()))`
    — a SOLID accent block reverse-videoing the caret char, ALWAYS solid.
  - It renders inside the pane loop `for (pane_id, r) in &rect_list` (~4186); `let is_focused = pane_id ==
    focused;` (~4191) is IN SCOPE — no new plumbing for the focus branch.
  - `prompt_visible = !is_command_running` (~4353) gates the prompt row (#193) — the cursor only shows on
    a pane at its prompt (not while a foreground command runs). So the "unfocused pane cursor" case is a
    pane sitting at its prompt while another pane is focused (a split) — a real, capturable state.
  - `selection_bg = hsla(accent.h, accent.s, accent.l, 0.3)` (~4357), applied to selected rows (4412
    header, 4537 line_row). `find_bg = success @ 0.35` (unrelated).
- **Genuine deltas:** D-A make the block cursor focus-aware (solid focused / hollow-accent-outline
  unfocused; the char in `foreground` when hollow). D-B evaluate the selection tint (accent@0.3) vs Warp →
  calibrate (pure `selection_tint` + test) or keep.
- **Deferred (documented):** the block shape (#218); cursor BLINK (timer-driven, separate); the find tint;
  the M13 editor caret (#208).
- **Decisions:** D1 hollow unfocused (1px accent outline, char in foreground); D2 reuse `is_focused` (#191);
  D3 D-A shim-only capture-validated (like #217), D-B a pure helper only if calibrating; D4 tokens only; D5
  auto-approved, document with captures.
- **Open questions for Design:** (1) the gpui hollow idiom — `div().border_1().border_color(colors.accent)`
  with NO `.bg` (transparent) + the char child in `text_color(colors.foreground)`; confirm a 1px border on
  a one-cell div doesn't shift the caret line (the #88 gapless flush) — a border adds to the box; may need
  a matching approach so the focused (bg, no border) and unfocused (border, no bg) cells are the SAME width
  (box-sizing). Design confirms whether gpui's border is inside/outside the content box + any width fix.
  (2) the selection tint: capture a real drag-select (drive a `drag:` over output rows) + judge vs Warp;
  decide calibrate-or-keep + the exact value if calibrating.

## Phase 2 — Design

### Discovery verified (the 2 open questions, answered)
- **(1) gpui border box model → REJECT the hollow outline; use a DIM FILL.** gpui has NO box-sizing
  (`grep box_sizing` = none); `Style.border_widths` (style.rs:211) maps to taffy's border, and taffy sizes
  an auto element as content + padding + BORDER. So `.border_1()` on the auto-sized ONE-cell cursor grows
  it by 2px → either the caret jitters on focus change (border only on unfocused) or the caret is
  permanently 2px wider (border on both), regressing #218's exact-one-cell block + the #88 flush. The
  geometry-clean unfocused-cursor treatment is a DIM translucent FILL (a recognized convention — iTerm2 et
  al. dim rather than hollow): it keeps the cell byte-identical in size (bg-only, no border). Decision:
  - FOCUSED (`is_focused`) → UNCHANGED from #218: `div().bg(colors.accent).text_color(colors.background)`
    (solid accent, reverse-video the caret char).
  - UNFOCUSED → `div().bg(colors.accent.opacity(0.4)).text_color(colors.foreground)` (a dim accent fill;
    the caret char in the NORMAL foreground, since it's not on a bright fill). Same one-cell geometry.
  Distinctness: focused a=1.0 (bright solid) vs unfocused a=0.4 (dim) — clearly different; and 0.4 > the
  selection's 0.3, plus a one-cell cursor ≠ a full-row selection, so no confusion.
- **(2) Selection tint (accent@0.3) → KEEP (documented).** Captured a real drag-selection
  (`scratchpad/220-selection2.png`): accent@0.3 renders as a subtle dark-teal highlight over the selected
  rows with the text fully legible — a clean, standard accent-based terminal selection. Warp's is a touch
  more neutral, BUT I cannot drive Warp to capture its exact selection color, so calibrating to "more
  neutral" would be INVENTING a value, not matching a measured reference — less defensible under §20 than
  the existing measured-original accent@0.3. Bounded discipline → KEEP, no churn, no pure helper. (REQ-003
  = evaluated + kept with rationale; REQ-004 = N/A, not calibrated.)

### Architecture / approach
- **SHIM-only** (app.rs cursor render inside `#[cfg_attr(test, mutants::skip)] fn render`): a focus branch
  on the one cursor-block child (~4659-4664), reusing `is_focused` (~4191, already in scope). No pure value
  to extract (the focused/unfocused difference is gpui Div styling, not a computable value) → capture-
  validated like #217 (shim-only, no unit test). §14 clean (no logic, no IO).
- Purely visual: the caret POSITION (`state.caret`/`split_caret_char`), the focused-cursor look (byte-
  identical to #218), the `before`/`after` text, the #88 gapless flush, the selection + find tints, and
  alt-screen rendering are ALL unchanged.

### File manifest
- `crates/marley_app/src/app.rs` — the prompt block-cursor child (~4659-4664): branch the cursor `div()`
  on `is_focused` — focused = `bg(accent).text_color(background)` (unchanged); unfocused =
  `bg(accent.opacity(0.4)).text_color(foreground)`. A `#220 (Warp parity)` comment. No other file.

### Regression Test Plan
| REQ | test |
|---|---|
| REQ-001 | driven capture — the FOCUSED pane's prompt block cursor is a solid bright accent block (unchanged from #218). |
| REQ-002 | driven capture — a SPLIT (≥2 terminal panes at their prompts): the focused pane's cursor is solid/bright, the UNFOCUSED pane(s)' cursor is a DIM accent fill (visibly different), same cell size (no shift). (`220-selection.png` already shows the BEFORE: all 3 panes solid.) |
| REQ-003 | review + capture (`220-selection2.png`) — the drag-selection tint reads Warp-like; KEPT (accent@0.3), rationale documented. |
| REQ-005 | review + capture — the caret position, the focused-cursor look, and the find-match tint are unchanged. |
- **Uncoverable by unit test:** the shim render (`mutants::skip`) — validated by the driven SPLIT capture
  (like #217). No pure logic added (D-B kept), so no new unit test; the gate's cov/MSI is unaffected (no
  new coverable/mutable code).

### Risks / decisions
- **R1 — border grows the box** → rejected hollow-outline for the geometry-clean dim-fill (above). The
  focused cell is byte-identical to #218 (no regression); the unfocused cell is the same size (bg-only).
- **R2 — dim-fill legibility.** Unfocused char in `foreground` (L0.90) over a 0.4-accent fill composited on
  the pane bg (~L0.3) → readable; at EOL the caret char is a space (just the dim block shows).
- **R3 — 0.4 vs the 0.3 selection.** Close in alpha but different context (one-cell cursor vs full-row
  selection) → no confusion; if the capture reads too faint or too selection-like, bump toward 0.5 (a
  validate-tunable, noted).
- **R4 — single-pane case.** With no split, the only pane is always focused → the cursor is always solid
  (no visible change) — correct; the delta only manifests in splits (which is where the bug was).

## Phase 3 — Implement
- **app.rs** — the prompt block-cursor child (~4659): replaced the single `div().bg(accent).text_color(
  background).child(caret)` with an `is_focused` branch — focused = `div().bg(colors.accent).text_color(
  colors.background)` (byte-identical to #218); unfocused = `div().bg(colors.accent.opacity(0.4)).text_color(
  colors.foreground)` (a dim accent fill); then `.child(at.unwrap_or_else(|| " ".to_string()))` on either.
  A `#220 (Warp parity)` comment noting the dim-fill-not-border rationale (gpui has no box-sizing).
- `is_focused` (`pane_id == focused`, ~4191) confirmed in scope at the cursor render (compile-verified).
- **Deviations from design:** none. (Dim-fill unfocused as designed; `before`/`after`/selection_bg untouched.)
- `cargo fmt` + `cargo check -p marley` clean (only the pre-existing transitive `block v0.1.6` note).

## Inspect (Phase 3.5)
Proportionate to a trivial-shim change (one focus branch, focused arm byte-identical to #218) — a focused
critic (correctness/geometry/clean-room) + my own thorough self-review. **No findings.**

**Critic returned — fully corroborates the self-review: "No defects. Verified geometry, correctness,
legibility, clean-room."** Extra detail: `bg`/`text_color` are paint-only in gpui (don't touch taffy layout)
→ both arms are the same auto-sized one-cell box; the focused arm is byte-identical to #218; `is_focused` is
read BY VALUE in a block-expression `.child({…})` (NOT a closure — no capture problem); `at` consumed once;
the unfocused char contrast ≈ 6.7:1 (AA pass), a large lightness gap from the solid. Two LOW BY-DESIGN
observations (no change): (1) the unfocused cursor accent@0.4 is only 0.1 alpha from the selection tint
accent@0.3 — but different footprints (one-cell cursor vs full-row band), cosmetic; (2) the dim block's
edge-vs-bg contrast ~2.3:1 is under the 3:1 non-text guideline — the INTENDED de-emphasis of an unfocused
cursor (Warp dims too), the 6.7:1 foreground glyph carrying perceptibility. Both accepted as by-design.

**Self-review (all lenses, evidence-backed):**
- **Geometry (load-bearing) — VERIFIED.** Both arms are `div().bg(...).text_color(...)` with NO
  `.border`/`.px`/`.py`/`.w`/`.m` (confirmed in the diff) → the focused + unfocused cursor cells are the
  SAME size (one mono cell), and the inner flex still has no `.gap` → the #88 flush holds. The FOCUSED arm
  `div().bg(colors.accent).text_color(colors.background)` is BYTE-IDENTICAL to the pre-diff (#218) cursor
  (visible in the diff: the removed line and the focused-arm line are the same tokens) → zero regression to
  #218's exact-one-cell block. The design's border-avoidance premise is verified: gpui has no `box_sizing`
  (grep = none) and `Style.border_widths` (style.rs:211) maps to taffy border which inflates an auto-sized
  element — so a `.border_1()` WOULD have grown this one-cell div by 2px; the dim-fill correctly avoids it.
- **Correctness — VERIFIED.** `is_focused` (`pane_id == focused`, ~4191) is in scope: the cursor child is
  built INLINE in the render expression inside the `for (pane_id, r) in &rect_list` loop (NOT inside an
  event-handler `move` closure), so it captures directly (compile confirms). `at.unwrap_or_else(|| " ")`
  is applied to `cell` identically for both arms (one `.child()` after the if/else) — no dropped/duplicated
  char; `at` (Option<String>) is consumed once (its only use). At EOL (`at==None` → `" "`) both arms render
  a one-cell block (focused bright / unfocused dim), no glyph — correct.
- **Legibility — VERIFIED.** Unfocused char in `foreground` (L0.90) over `accent.opacity(0.4)` composited on
  the pane bg (L0.05) → the fill lands ~L0.24, foreground on it is readable. The dim (a=0.4) is clearly
  distinct from the focused solid (a=1.0); vs the selection tint (accent@0.3) it's a different context (a
  one-cell cursor vs a full-row highlight) + slightly stronger — no confusion.
- **Clean-room / regression — VERIFIED.** Tokens only (accent/background/foreground via `.opacity`) — no new
  `hsla`/hex in the diff. `git diff` touches ONLY the cursor child; `before`/`after`/`selection_bg`/`find_bg`/
  alt-screen and the caret POSITION (`state.caret`/`split_caret_char`) are unchanged.

**Verdict:** no findings requiring a fix. No forge `failure-record` (no bug). Lenses: geometry/box-sizing,
correctness/scope, legibility, clean-room, regression.

## Phase 3.5 — Inspect
- (superseded by "## Inspect (Phase 3.5)" above)

## Phase 4 — Validate
- **No unit test** (shim-only change, no new pure logic; D-B selection kept → no helper) — like #217. The
  render is `mutants::skip`; validated by driven capture. `cargo nextest run -p marley` = **301 passed, 2
  skipped** (the render change broke none of the existing tests).
- **Driven SPLIT capture** (RE-BUNDLED during inspect; stale quit + relaunched; switched to the terminal-3
  tab which holds a 3-pane split): `scratchpad/220-cursor-split2.png` (+ `-crop`):
  - **REQ-001** — the FOCUSED pane (left, with the #191 accent border) shows a SOLID bright cyan block cursor
    at `❯ Marley ▊`.
  - **REQ-002** — both UNFOCUSED panes (middle, right) show a DIM/darker-teal block cursor — visibly fainter
    than the focused one, SAME cell size (no shift). Vs the BEFORE `220-selection.png` (all 3 solid-bright).
  - **REQ-005** — the focused cursor look is byte-identical to #218; the caret position + selection tint
    unchanged.
  - (`cmd:d` opens new TABS in Marley, not pane splits — the split command is ⌘⇧L #197; used the existing
    terminal-3 3-pane split for the capture.)
- **REQ-003/004 (selection):** D-B evaluated + KEPT (accent@0.3, `220-selection2.png`) — REQ-004 N/A (not
  calibrated).
- **Gate:** `git add -A` + `scripts/gates.sh --diff` — GREEN (below). No new coverable/mutable code (the
  cursor render is inside the `mutants::skip` render) → cov/MSI unaffected.
- **Pre-existing exclusions:** none (the transitive `block v0.1.6` note is upstream).

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md — #220 entry under [Unreleased]/Changed (above #219). app_shell.md — the
  #218 block-cursor note extended with the #220 focus-aware behavior + the dim-fill-not-border rationale +
  the selection-kept note.
- **Knowledge (forge):** `aar-submit` 79dfdb78 (completed, effectiveness 5). No `failure-record` (no bug —
  the 2 critic LOWs were by-design). **Prevention rule recorded** →
  `PR-claude-gpui-no-box-sizing-border-inflates-001` (cc790948): gpui has no box-sizing → a border inflates
  an auto-sized box; for a fixed-cell element use a fill or a matched-border on all states, never a bare
  border on one state (else the cell widens/jitters). Will bite the M13 editor caret (#208).
- **Lessons:** (1) the gpui box-sizing gotcha (→ the prevention rule) — the design's border-vs-fill call was
  load-bearing. (2) a ticket description can be STALE relative to sibling work — #220 said "2px bar" but
  #218 had already made it a block; discovery caught it, so #220 scoped to the genuine remaining delta
  (focus-state) instead of redoing #218. (3) `cmd:d` opens new TABS in Marley, not pane splits (split = ⌘⇧L
  #197) — used the existing terminal-3 3-pane split for the focused/unfocused capture. (4) bounded
  discipline on D-B: kept accent@0.3 (can't measure Warp's selection clean-room → keeping the
  measured-original beats inventing a "more neutral" value).
- **Close/archive:** TICKET-220 open→closed; forge ticket-close #220 done; pipeline pair → completed/.
