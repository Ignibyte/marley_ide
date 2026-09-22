# Warp prompt input-row styling — Notes

- **Forge ticket:** #218 (6167542d-7df6-454c-9364-243637980ab9)
- **AAR:** 97bc81fe-e922-470f-8719-099afe411db2
- **Local ticket doc:** docs/planning/tickets/open/TICKET-218-warp-prompt-styling.md
- **Pipeline spec:** warp-prompt-styling.spec.md

## Phase 1 — Plan
- **Request:** match Warp's prompt look (marker/strip/cwd chips/caret). Auto-approved (/work 195-222).
- **Classification / tier:** work pipeline, small/bounded. Systems: app.rs prompt input-row render
  (shim, `#[cfg_attr(test, mutants::skip)]` `fn render`) + a new pure caret-split helper beside
  `prompt.rs` (cov/MSI 100).
- **Forge recall (§18.3):** knowledge-search (prompt/caret/cursor render) surfaced only the generic
  mutation traps already known — f32/hsla literals aren't mutated → guard with EXACT-value asserts
  ([[PR-claude-no-default-struct-return-needs-full-value-assert]]); a struct-with-no-Default return
  yields no viable mutant → assert the full value. No prompt-specific prior failure. AAR opened.
- **Discovery (the edit surface for Design):**
  - `prompt.rs` (113 lines, pure): `SegmentKind{Cwd,Git}`, `PromptSegment{text,kind}`,
    `prompt_segments(&PromptInfo) -> Vec<PromptSegment>`, `pwd_label(&str) -> &str`. All cov/MSI 100.
    (There's `split_at_caret` in `input.rs`, byte-halves only — NOT grapheme-aware for a block cursor.)
  - `app.rs` prompt render (~4591-4632): builds `input_row` = `div().flex().flex_row().items_center()
    .gap_2().bg(colors.surface).rounded(colors.corner_radius).child(❯ in accent)`; then per-segment
    `Cwd→colors.foreground / Git→colors.accent`; then the #88 gapless inner flex `before | 2px accent
    bar | after`. `TERMINAL_FONT_SIZE` = the cell height source; `em_advance(px(TERMINAL_FONT_SIZE))`
    the cell width (used elsewhere for the mono metric).
- **Genuine deltas (bounded — not a rewrite):**
  - **D-A** (dominant) — drop `.bg(colors.surface)` + `.rounded(colors.corner_radius)` on the input row.
  - **D-B** — `❯` `colors.accent` → `colors.muted`.
  - **D-C** — Cwd `colors.foreground` → `colors.muted`; Git stays `colors.accent`.
  - **D-D** — caret: the `div().w(px(2.0)).h(px(TERMINAL_FONT_SIZE)).bg(colors.accent)` bar → a block
    cursor: reverse-video the grapheme at the caret (a cell-sized `div` `bg`=cursor color with the
    grapheme as a `fg`=pane-bg child); standalone one-cell block at EOL. Driven by a pure helper.
  - New pure helper `split_caret_grapheme(text, caret) -> (before, Option<grapheme>, after)` beside
    `prompt.rs` (Design fixes the exact signature/type). Keeps caret logic tested.
- **Decisions:** D1 block reverse-videos the caret grapheme (standalone block at EOL); D2 a pure
  caret-split helper (cov/MSI 100, exact-value); D3 tokens only (muted/accent/background — no new
  hsla); D4 auto-approved, document with captures.
- **Open questions for Design:** the exact cursor color token (accent vs a dedicated cursor token vs
  foreground) + the block cell width (`em_advance` vs a fixed px) + whether the grapheme unit is a
  `char` or a full grapheme cluster (unicode-segmentation is already a dep? — Design checks).

## Phase 2 — Design

### Discovery verified (the 3 open questions, answered)
- **(1) Helper unit + signature.** `unicode-segmentation` is NOT a direct dep of `marley_app`
  (transitive only) → use **char boundaries**, no new dep. This matches the whole editor: the caret is
  `CharOffset` (from the pure `marley_text_offsets` crate), and the sibling `split_at_caret`
  (input.rs:122) is char-based (`text.chars().take(n)`). The new helper mirrors it exactly:
  `pub fn split_caret_char(text: &str, caret: CharOffset) -> (String, Option<String>, String)`
  — `(before, at, after)` where `at` = the char UNDER the caret (the block-cursor grapheme), `None`
  at EOL. Placed **in `input.rs` beside `split_at_caret`** (NOT prompt.rs as the Phase-1 hint said):
  they are the same domain (buffer/caret editing, both consume `CharOffset`), and prompt.rs is a
  different concern (cwd/git SEGMENTS). Deviation logged (§14 cohesion — put it with its sibling).
  Body: `let n = caret.as_usize(); before = chars().take(n); at = chars().nth(n).map(to_string);
  after = chars().skip(n+1)`. Out-of-range `n` saturates (take/nth/skip all tolerate it) → no panic,
  matching `split_at_caret`. A char (not a grapheme cluster) is correct for Marley's char-indexed
  caret; ZWJ-cluster handling is a separate future concern if the caret model ever moves to graphemes
  (out of scope; documented).
  - `split_at_caret` STAYS — it's the distinct *insertion-halves* primitive (char at caret goes to
    `after`), still `pub use`-exported (lib.rs:63) + tested. `split_caret_char` is the *cursor-isolate*
    primitive (char at caret isolated for the block). Two genuinely different ops, no dedup forced.
- **(2) Cursor color.** No dedicated `cursor` token in `ThemeColors` (fields: background/foreground/
  accent/on_accent/surface/border/danger/success/muted). The old bar used `colors.accent` → the block
  uses **`bg = colors.accent`** (Marley's cursor color) with **`text_color = colors.background`**
  (the pane fill) for the reverse-video glyph. A dedicated cursor token is a possible future addition
  (out of scope).
- **(3) Block cell width.** The block **wraps its content** — mid-line it wraps the caret grapheme
  (one cell in the mono font), at EOL it wraps a single space `" "` (also one cell). No explicit
  `px()` width needed → the block auto-sizes to exactly one cell AND aligns with the `before`/`after`
  text divs on the same baseline (all three are text divs in the gapless inner flex). Cleaner than a
  fixed `fallback_cell(..).w` and inherently correct for wide chars (a 2-cell CJK/emoji caret char
  gives a 2-cell block — correct cursor coverage).

### Architecture / approach
- **PURE seam** (`input.rs`): the new `split_caret_char` — gpui-free, `CharOffset`-in / owned-Strings-out,
  cov/MSI 100 with exact-value tests (validate). §14 clean (no IO, no panic on any input incl.
  out-of-range/empty/multi-byte).
- **SHIM** (`app.rs` prompt input-row render, ~4596-4631, inside the `#[cfg_attr(test, mutants::skip)]`
  `fn render`, #193 `prompt_visible`-gated): the 4 deltas. No new types/logic in the shim — it consumes
  the pure helper + the existing `colors` tokens.
- The 4 deltas, precisely:
  - **D-A** — drop `.bg(colors.surface)` + `.rounded(colors.corner_radius)` from the `input_row` builder
    chain (the prompt sits on the pane background).
  - **D-B** — the `❯` marker child `div().text_color(colors.accent).child("\u{276f}")` → `.text_color(colors.muted)`.
  - **D-C** — the segment match arm `SegmentKind::Cwd => colors.foreground` → `colors.muted`
    (`SegmentKind::Git => colors.accent` unchanged).
  - **D-D** — line 4597 `let (before, after) = split_at_caret(...)` → `let (before, at, after) =
    split_caret_char(&state.buffer.text(), state.caret);`. The inner gapless flex's middle child
    (the `div().w(px(2.0)).h(px(TERMINAL_FONT_SIZE)).bg(colors.accent)` bar) →
    `div().bg(colors.accent).text_color(colors.background).child(at.unwrap_or_else(|| " ".into()))`.
    The `before` / `after` text divs + the gapless inner-flex structure (#88) are UNCHANGED → the block
    sits flush against the typed text.
  - Import swap: app.rs:71 `use crate::input::{apply_key, split_at_caret, Key, KeyOutcome}` →
    `{apply_key, split_caret_char, Key, KeyOutcome}` (app.rs no longer calls `split_at_caret`); lib.rs:63
    `pub use` adds `split_caret_char` beside `split_at_caret`.
- **Purely visual** — `state.caret` and all input/editing (`apply_key`, arrows, edit) are untouched; only
  the RENDER of the caret changes (block-on-char vs bar-before-char). The block-on-char semantic (cursor
  covers the char to the right of the insertion point) is standard terminal + Warp behavior.

### File manifest
- `crates/marley_app/src/input.rs` — ADD `pub fn split_caret_char(text, caret) -> (String, Option<String>, String)`
  beside `split_at_caret` (pure, char-based; doc comment). Its exact-value tests land at validate.
- `crates/marley_app/src/lib.rs` — add `split_caret_char` to the `pub use input::{…}` (line 63).
- `crates/marley_app/src/app.rs` — (a) import swap (line 71); (b) the prompt input-row render
  (~4596-4631): D-A drop bg/rounded, D-B ❯→muted, D-C Cwd→muted, D-D block-cursor sub-render from
  `split_caret_char`.

### Regression Test Plan
| REQ | test |
|---|---|
| REQ-005 (helper) | `input.rs` `#[cfg(test)]` `split_caret_char_isolates_the_caret_char` — exact-value matrix: `""`→`("",None,"")`; `"abc"`@0→`("",Some("a"),"bc")`; `"abc"`@1 (mid)→`("a",Some("b"),"c")`; `"abc"`@3 (EOL)→`("abc",None,"")`; `"h\u{e9}llo"`@1 (multi-byte)→`("h",Some("\u{e9}"),"llo")`; `"ab"`@9 (out-of-range)→`("ab",None,"")`. Exact tuples kill the take/nth/skip count mutants (`n`/`n+1`) → cov/MSI 100. |
| REQ-003 | driven capture — type text, move the caret INTO the line (⬅), capture → a block reverse-videos a mid-line char (the char shows dark-on-accent) |
| REQ-004 | driven capture — type text, caret at end, capture → a standalone one-cell accent block after the last char |
| REQ-001 | driven capture — the prompt row has NO gray strip / rounded box (sits on the near-black pane bg) |
| REQ-002 | driven capture — the `❯` marker + the cwd segment are DIM (muted); a git branch (if present) stays accent |
- **Uncoverable by unit test:** the shim render itself (`#[cfg_attr(test, mutants::skip)] fn render`) —
  validated by the driven captures (like #217/#216/#192), not a headless unit test. The pure helper
  carries the mutation load.

### Risks / decisions
- **R1 — caret char moves into the block.** Previously the char at the caret sat in `after` (shown
  plain) with a bar before it; now it's isolated into the block (reverse-video). This is the intended
  Warp block-cursor semantic (block covers the char to the right of the insertion point). No input
  change — `state.caret` identical; purely a render swap. Verify in the REQ-003 capture.
- **R2 — char vs grapheme cluster.** Char boundaries (matches `CharOffset` + `split_at_caret` + the
  whole char-indexed editor; no new dep). A ZWJ emoji cluster would block one code-point-char, not the
  full cluster — acceptable + consistent; grapheme-caret is out of scope. No panic (char boundaries
  always valid; out-of-range saturates).
- **R3 — #88 gapless caret line.** Preserved — the inner flex still has no `.gap`; the block replaces
  the bar in the same middle slot. Verify the block sits flush against the typed text in the capture.
- **R4 — legibility after dropping the strip.** `before`/`after` inherit the pane's default foreground
  (bright) on `background` (near-black) → high contrast (unchanged from today). `muted` (❯/cwd) was the
  caption color validated on `surface` (#194 `contrast_ratio`); on the darker `background` its contrast
  is HIGHER, and Warp's ❯/cwd are dim by design → correct. The block glyph is `background`-on-`accent`
  (bright cyan) → legible reverse-video. If a capture reads too faint, that's a validate finding.

## Phase 3 — Implement
- **input.rs** — added `pub fn split_caret_char(text, caret: CharOffset) -> (String, Option<String>, String)`
  beside `split_at_caret` (char-based: `take(n)` / `nth(n).map(to_string)` / `skip(n+1)`), with the
  doc comment. `split_at_caret` kept (distinct insertion-halves primitive).
- **lib.rs** — added `split_caret_char` to `pub use input::{…}` (line 63).
- **app.rs** — (a) import swap line 71 (`split_at_caret` → `split_caret_char`; app.rs no longer calls
  `split_at_caret`); (b) the prompt input-row render (~4596): line 4597 now `let (before, at, after) =
  split_caret_char(...)`; **D-A** removed `.bg(colors.surface)` + `.rounded(colors.corner_radius)`;
  **D-B** the ❯ child `colors.accent` → `colors.muted`; **D-C** `SegmentKind::Cwd` arm
  `colors.foreground` → `colors.muted`; **D-D** the inner gapless flex's middle child (the 2px bar) →
  `div().bg(colors.accent).text_color(colors.background).child(at.unwrap_or_else(|| " ".to_string()))`.
  Comments added for each Warp-parity delta; the #88 gapless inner-flex structure preserved.
- **Deviations from design:** none. (As designed: the helper landed in `input.rs` beside its sibling,
  not `prompt.rs` — this was decided at design, not a Phase-3 deviation.)
- `cargo fmt` + `cargo check -p marley` clean (only the pre-existing transitive `block v0.1.6`
  future-incompat note, unrelated). `px` import stays (used elsewhere in render); `split_at_caret`
  no longer imported in app.rs (still exported + used by its tests).

## Inspect (Phase 3.5)
Two parallel critics over the diff (correctness; clean-room/simplification/consistency/dead-code) +
my own independent verification. **No confirmed findings — nothing to fix.**

**Critic 1 — Correctness (returned: no defects; verified A/B/C):**
- **Helper panic-free + exact partition** — traced all 6 cases (empty, @0, @mid, @EOL, out-of-range@9,
  multibyte "héllo"@1). `CharOffset::as_usize` is a trivial getter (no panic); every branch goes through
  `.chars()` (char-boundary — no byte slice); `take`/`nth`/`skip` saturate at iterator end (no panic on
  n>len). Invariant holds: `before+at+after == text` when `at=Some`; `before==text & after==""` when
  `at=None` (three fresh `chars()` iterators — no cross-consumption). REAL — verified, no defect.
- **Mutation matrix sufficient for MSI 100** — the only mutable spots are the `n + 1` operator, the
  whole-body-default, and the map closure. `skip(n+1)→skip(n)` (`+`→`*`) and `→skip(n-1)` (`+`→`-`) both
  die on `"abc"@1` (distinct `after="c"` ≠ `"bc"`/`"abc"`; the `-` case also underflow-panics @0);
  body→`("",None,"")` dies on @0/@mid; closure→`String::new()` dies on @0. `take(n)`/`nth(n)` have no
  operator on bare `n` → no mutant generated. No surviving mutant; no extra case needed.
- **Render is a pure change** — the diff touches only the import (app.rs:71) + the render block;
  `apply_key`/`buffer.edit` untouched; `before+at+after` = old `before+after` (same glyphs, caret char
  now reverse-videoed); inner flex has no `.gap` (#88 flush intact); EOL `" "` block is a render child
  only (`after==""`, never written to the buffer). No broken consumer; `TERMINAL_FONT_SIZE` still used
  (4076/4127/…). Two LOW non-defects flagged (O(3n) 3-pass — mirrors `split_at_caret`'s 2-pass house
  style; `split_at_caret` now production-unused but `pub` → no clippy flag) — **no action**, agreed.

**Critic 2 returned (after the ledger draft) — its verified areas match my independent pass; two new notes
folded in below (F2 = a real cleanliness heads-up, F4 = a better reason to keep the code as-is):**
- **F1 [MED] — `split_caret_char` has no unit test yet → MSI risk.** NOT a diff defect — the helper tests
  land at Phase 4 Validate by the design's test plan (the shim render is `mutants::skip`; the pure helper
  carries the mutation load). Action: write the exact-value matrix at validate (planned). Accepted.
- **F2 [MED] — `colors.corner_radius` is now render-orphaned.** VERIFIED: this diff removed the last
  `.rounded(colors.corner_radius)`; crate-wide `corner_radius` now lives only at ui_components/lib.rs
  (field:62, set light:87/dark:110 = px(6.), test:184) — no surface renders it (the sole remaining
  rounding is `.rounded_full()` at app.rs:5540, no token). Does NOT fail the gate (a `pub` field → clippy
  never flags pub-field dead_code; still set + value-asserted by the lib.rs:184 test → covered; the px
  literal isn't mutated). Removing a `pub` theme token is scope-creep for a styling ticket, and it's a
  legitimate RESERVED token — the palette/dialog cards currently use square corners and a future ticket
  may apply it. **Decision: leave in place; recorded as a follow-up (below).**
- **F3 [LOW] — `split_at_caret` render-vestigial** (same as Critic 1's note): not dead (pub + tested); no
  action.
- **F4 [LOW / defends the code] — the `at.as_deref().unwrap_or(" ")` simplification does NOT compile.**
  gpui impls `IntoElement` for `&'static str` / `String` / `SharedString` — NOT a borrowed non-`'static`
  `&str`. `at` is a local `Option<String>`, so `at.as_deref().unwrap_or(" ")` is a non-`'static &str` →
  `.child()` fails the bound. The current `at.unwrap_or_else(|| " ".to_string())` returns `String`
  (impls IntoElement) and is lazy (allocates the space only at EOL). **Keep as-is** (a firmer reason than
  the ledger's original "no real savings").
- **F5 [LOW / informational] — the block reverse-videos one `char`, not a grapheme cluster.** Consistent
  with the char-based `CharOffset` caret model (the whole editor indexes chars); not a regression vs the
  old bar. The doc comment leads with "CHAR" (accurate). No action.

**Critic 2 lenses — also independently verified by me (concrete checks, corroborating Critic 2):**
- **Clean-room §20** — `git diff | grep -iE 'hsla|hex|rgb('` on added lines = CLEAN; only token references
  (`colors.muted`/`accent`/`background`). No Warp asset/literal. REAL — clean.
- **Consistency** — `grep '276f'` app.rs: exactly ONE *rendered* `❯` (line 4611, the prompt); the others
  (1053/4325/4591/4605/4624) are comments. Only the prompt's `.bg(colors.surface)`+`.rounded` was removed
  (the surface-strip list jumps 4611→4672); all 20 other `.bg(colors.surface)` sites (docks, list entries,
  sticky header 4672, palette/dialog cards) are untouched by the diff → the command-palette/launcher keeps
  its card look (correct — only the bare terminal prompt changed, matching Warp). REAL — clean.
- **Dead-code** — `cargo clippy -p marley --all-targets` = clean (only the pre-existing transitive
  `block v0.1.6` future-incompat note). `split_caret_char` used (app.rs:4598) + exported (lib.rs:63);
  `split_at_caret` still exported + referenced 9× (input.rs def+tests); `px` still used; no unused import.
  REAL — clean.
- **Simplification** — `.child()` accepts `&str` (app.rs:3755 passes a literal), so
  `at.as_deref().unwrap_or(" ")` compiles — but `SharedString::from(&str)` still allocates → no real
  savings; the owned-`String` form is consistent with `before`/`after`. Non-issue — no change.
- **Legibility** — dark theme: `background` L=0.05 (near-black), `surface` L=0.155, `muted` L=0.60. Moving
  `muted` (❯/cwd) from surface onto the darker background RAISES contrast (Δ0.445 → Δ0.55); the block
  glyph is `background` L0.05 on `accent` L0.55 (Δ0.50, legible reverse-video). Confirmed OK.

**Verdict:** no findings requiring a code fix in the diff. F1 (helper unit test) is the planned Phase 4
work. F2 (orphaned `corner_radius`) is a tracked follow-up (below). Lenses covered: correctness,
panic-safety, mutation-resistance (MSI-100 plan), render-purity, clean-room, consistency, dead-code,
simplification, legibility.

**Follow-up FILED → forge #224** (id aefd6a51-9d9e-42c0-9fd4-77b05db7dc7b, chore, M12.2/cleanup):
`colors.corner_radius` is a reserved theme token with no render consumer after #218 (the prompt strip was
its last). Either apply it to the palette / dialog / find-bar cards (they currently use square corners — a
Warp-parity opportunity) or reclaim the token. Won't fail the gate (pub field, set + test-asserted).
Cross-refs the #223 chrome-token thread.

## Phase 4 — Validate
- **Unit test (REQ-005/003/004):** added `split_caret_char_isolates_the_caret_char` in input.rs (beside
  the R35 `split_at_caret` test) — exact-value matrix: empty→`("",None,"")`; `"abc"`@0→`("",Some("a"),"bc")`;
  @1 (mid)→`("a",Some("b"),"c")`; @3 (EOL)→`("abc",None,"")`; `"h\u{e9}llo"`@1→`("h",Some("\u{e9}"),"llo")`;
  `"ab"`@9 (out-of-range)→`("ab",None,"")`. `cargo nextest run -p marley` = **300 passed, 2 skipped** (was
  299 + this one); the isolated run confirms `input::tests::split_caret_char_isolates_the_caret_char` PASS.
- **Driven captures (live app — a stale #217 instance was found running the old binary; QUIT it + relaunched
  the fresh 19:34 bundle to be certain):**
  - `scratchpad/218-atrest.png` (+ `-crop`): a fresh empty prompt — **REQ-001** no gray strip/box (bare on
    the near-black pane bg); **REQ-002** the `❯` marker + the `Marley` cwd render DIM (muted gray, not the old
    bright accent/foreground); **REQ-004** a solid cyan (accent) BLOCK cursor sits after the cwd (empty buffer
    → standalone one-cell block). Matches the Warp ref `195-warp-ref.png` (bare prompt, dim chevron, block).
  - `scratchpad/218-eol.png` (`-crop`): after `type:echo hi` (no enter) — `echo hi` in bright foreground, then
    the block cursor at EOL after `i` (**REQ-004** with a real command).
  - `scratchpad/218-midline.png` (`-crop`): after two `left` arrows — the block moved onto the `h` of `hi` and
    **reverse-videos it** (a cyan block with a DARK `h` glyph inside; the `i` after stays bright) — **REQ-003**
    the block-on-char reverse-video. `echo [h]i`.
- **Harness:** added `left`/`right` (keycodes 123/124) to `scripts/selftest/drive.swift`'s `keycodes` map +
  the header doc — a plain caret-move verb the harness lacked (it had `up/down` but not `left/right`). A small
  reusable addition; enabled the REQ-003 mid-line capture. (Confirms the app's existing `Key::Left`→
  `move_char_left` path in `apply_key` drives `state.caret`, which the block render reads.)
- **Gate:** `git add -A` + `scripts/gates.sh --diff` — first run RED on gate:1 rustfmt (the newly-added test
  wasn't fmt'd); `cargo fmt` (input.rs +5/-1, test whitespace only) → re-stage → **GATE GREEN [diff]** — all
  15 pass incl **coverage ≥100%** + **mutation MSI ≥100%** (the exact-value matrix kills every `split_caret_char`
  `take/nth/skip(n+1)` mutant) + visual/AX. Receipt written.
- **Pre-existing exclusions:** none. (The transitive `block v0.1.6` future-incompat note is upstream, unrelated,
  and does not gate.)

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG.md — #218 entry under [Unreleased]/Changed (above #217). app_shell.md — the
  prompt input-row entry updated (M12.2 #218: no strip/rounding, muted ❯+Cwd, block cursor via
  `split_caret_char`; corner_radius now reserved; the new left/right drive verbs) + the input-module
  summary line (split_caret_char named beside split_at_caret).
- **Knowledge (forge):** `aar-submit` 97bc81fe (completed, effectiveness 5). No `failure-record` (inspect
  found no real bug — F1 was the planned validate test, F2/#224 is a follow-up not a defect). No new
  prevention rule/AD (the design reused the established pure-seam + exact-value-matrix patterns).
- **Lessons:** (1) a stale app instance from a prior ticket can shadow the fresh binary in a driven capture
  — QUIT all `Marley.app` procs + relaunch before capturing (added to the validate routine). (2) the
  drive harness lacked plain left/right caret verbs; adding them (keycodes 123/124) was the clean way to
  get a real mid-line block-cursor capture rather than inferring it. (3) removing a render consumer can
  orphan a theme token silently (corner_radius) — the inspect critic caught it; filed #224.
- **Follow-up:** forge #224 (corner_radius apply-or-reclaim).
- **Close/archive:** TICKET-218 open→closed; forge ticket-close #218 done; pipeline pair → completed/.
