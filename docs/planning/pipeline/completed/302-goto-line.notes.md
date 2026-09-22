# Go to line (#302) — Notes

- **Forge ticket:** #302 8b8263f4-46ef-4251-b187-d037acf1c095
- **AAR:** 215921e4-4469-4e60-8865-983f05ce6225
- **Local ticket doc:** ../../tickets/open/TICKET-302-goto-line.md
- **Pipeline spec:** 302-goto-line.spec.md · pipeline_id d44b0710-76ed-4f74-9f50-fb1b23afe102

<!-- Working scratch. Each phase appends. Excluded from gate:14 doc-todos. -->

## Phase 1 — Plan

Promoted `queued/302-goto-line.spec.md` → `active/` (the SECOND of the goal; #300 shipped `96f593a`).
`active/` was empty. Re-verified every cited seam against live code (`main` @ `96f593a`).

### THE VERIFICATION LEDGER — every seam CONFIRMED; TWO refinements SHRINK the ticket

| Claim | Verdict |
|---|---|
| `scroll_editor_to_row` CENTERS (ScrollStrategy::Center) + is the shared primitive | **VERIFIED** — app.rs:11076-11078 (`scroll_to_item(row, Center)`); drags the horizontal twin (used in #300/#340). No new centering code. |
| `caret_for_line_col(buffer, line, col)` places the caret | **VERIFIED + STRONGER (F1)** — code_view.rs:452: takes a **1-based** line + optional 1-based col, converts to 0-based, and **CLAMPS** (past-EOF `line` → last line's start via `line_start`; past-EOL `col` → line end; no col → col 0). So the placement seam ALSO owns the clamp. |
| the `renaming_symbol` inline-draft overlay (one `Option<Draft>` field + a HIGH match arm + a `text_input_blocked` line) is the smallest precedent | **VERIFIED** — `renaming_symbol: Option<RenameDraft>` (app.rs:214); the match arm at app.rs:12037 (escape/enter/backspace/space/single-char, all `stop_propagation`); `text_input_blocked` (app.rs:7535) gates typing and lists `renaming_symbol` (:7542). |
| ⌃G is FREE everywhere; terminal ⌃G = raw BEL to the PTY | **VERIFIED** — `grep -c 'false, true, false, false, "g"'` keymap.rs → **0**. Editor-scoped ⌃G leaves the terminal's raw ⌃G/BEL path untouched. Roster is now 71/26 (post-#300) → 72/27. |
| `parse_line_col` is the `N`/`N:C` parse prior art | **F2 — parse_line_col parses a COMPILER REF (`path:line:col`), NOT bare `N`/`N:C`** (links.rs:355: `fn parse_line_col(s) -> (&str, Option<usize>, Option<usize>)` — it strips a path prefix). So `parse_goto` is genuinely NEW (bare digit + optional `:col`), inspired by the digit-extraction but not the same fn. Small. |

### Findings (both SHRINK the ticket — the substrate already owns more than the spec assumed)

**F1 [`clamp_goto` DISSOLVES — a "the substrate already does it" win].** The spec proposed BOTH `parse_goto`
AND `clamp_goto(line, total) -> usize`. But `caret_for_line_col` **already** does the 1-based→0-based
conversion AND the past-EOF/EOL clamp (verified in its body + its `caret_for_line_col_places_and_clamps`
test). So `clamp_goto` is redundant — `parse_goto` produces `(1-based line, Option<1-based col>)` and feeds
`caret_for_line_col` DIRECTLY, which clamps. **The ticket's pure surface shrinks to ONE new fn (`parse_goto`).**
This is the prior-art lesson: a locked decision dying because the substrate already does it is a WIN.
(Design confirms: `parse_goto` may still reject `"0"` as inert, OR pass line 0 and let `caret_for_line_col`'s
`saturating_sub(1)` clamp it to line 1 — a UX choice, not a correctness one; both are safe.)

**F2 [parse_line_col is a compiler-ref parser, not the overlay's input grammar].** `parse_line_col`
(links.rs:355) extracts `line`/`col` from a `file:line:col` reference (with a path prefix) — NOT the bare
`N`/`N:C` the overlay reads. `parse_goto` is a NEW, simpler pure fn (a leading run of digits, an optional
`:` + digits). It can borrow the digit-parse approach but is not a reuse of `parse_line_col`. Named so
Design doesn't try to call `parse_line_col` on the raw overlay text.

### Confirmations
- **§20 + `### Prior art` HOLD** — VS Code/Zed = OBSERVED (⌃G, `N:C`, clamp-never-error, live-preview scroll,
  Esc-restores). No new deps. The prior-art sweep's key result is F1 (`caret_for_line_col` owns the clamp).
- **The EARS AC (REQ-001..008) STAND** — F1 folds `clamp_goto`'s clamp REQ into `caret_for_line_col`; the
  truth condition (past-EOF → last line) is unchanged, only WHERE it lives.
- **Locked decisions** (D-CLAMP-NEVER-ERROR, D-ONE-BASED-AT-ONE-SEAM, D-PUSH-ON-COMMIT-ONLY,
  D-LIVE-PREVIEW-SCROLL, D-COPY-THE-RENAME-OVERLAY) all **HOLD** — **D-ONE-BASED-AT-ONE-SEAM is even truer**:
  the one seam is `caret_for_line_col`, already shipped.

### Standing context
Auto-approved, autonomous-through-commit; after #302 → CONTINUE to #303 (don't stop). **chad AT THE MACHINE →
LIVE drives OFF-LIMITS** — the go-to-line is a caret+scroll STATE change (headless-provable via `editor_caret`
+ the scroll target); the overlay RENDER is deferred (the M21/M22 posture). Push UN-OK'd (LOCAL). Batch
lessons: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md) +
[ide-mvp-shelf.md](../../design-notes/ide-mvp-shelf.md).

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

### Architecture

The smallest ticket on the shelf: ONE new pure fn + an overlay that copies the shipped `renaming_symbol`
shape. The jump machinery is entirely shipped (F1: `caret_for_line_col` places AND clamps;
`scroll_editor_to_row` centers).

- **`parse_goto(input: &str) -> Option<(usize, Option<usize>)>`** (NEW, pure, in `code_view.rs` beside
  `caret_for_line_col` — the "line/col addressing" pair). 1-based line + optional 1-based col. `split_once(':')`:
  the line part must be an all-digit run parsing to **≥ 1** (so `""`, `"abc"`, `"0"`, a leading space → `None`);
  if a `':'` is present, the col part must likewise be an all-digit run ≥ 1 (so `"50:"`, `"50:0"`, `":5"` →
  `None` — a partial input mid-typing is a clean `None`, never a panic). `"50"` → `Some((50, None))`; `"50:12"`
  → `Some((50, Some(12)))`; `"999999"` → `Some((999999, None))` (parse_goto validates SYNTAX, `caret_for_line_col`
  validates RANGE — the clamp). Total (§14): every path returns `Some`/`None`, no panic.
- **The overlay** (`app.rs` shim, mutants::skip) — copies `renaming_symbol` (app.rs:12037):
  - A field `goto_line: Option<GotoDraft>` where `struct GotoDraft { input: String, origin_caret: CharOffset }`
    (init `None`). **The origin is the CARET only** — Esc restores it and RE-REVEALS it via
    `follow_editor_caret`, so no raw scroll-offset snapshot is needed (D-ESC-REVEALS-ORIGIN-CARET, below).
  - ⌃G → `dispatch_action("go-to-line")` → `goto_line = Some(GotoDraft { input: "", origin_caret:
    active_caret() })` (no-op off an editor tab).
  - A match arm HIGH in the on_key_down overlay ladder (above the editor router, like renaming_symbol):
    `escape` → restore (set the origin caret + `follow_editor_caret` + `goto_line = None`); `enter` → commit;
    `backspace` → `input.pop()` + re-preview; a single `'0'..='9'` or `':'` char → `input.push` + re-preview;
    everything else swallowed. `cx.stop_propagation()` after.
  - **Commit (enter):** `parse_goto(input)` → `Some((line, col))` → `caret_for_line_col(buffer, line, col)`
    → **PUSH the NavStack** (capture `(path, origin_caret)` BEFORE, push `NavLoc` — a far intra-file jump, the
    `jump_to_sticky_header` class; D-PUSH-ON-COMMIT-ONLY) → `set_active_selections(single caret)` →
    `scroll_editor_to_row(row)` (centers) → `goto_line = None`. `None` (invalid input) → just close, no jump,
    no push (the caret never moved).
  - **Live preview (re-preview on each edit):** if `parse_goto(input)` is `Some`, `scroll_editor_to_row(row)`
    to the would-be target WITHOUT moving the caret (D-LIVE-PREVIEW-SCROLL); the caret stays at origin until
    Enter. `None` input → no preview.
  - `text_input_blocked` (app.rs:7535) gains `|| self.goto_line.is_some()` — the digits never leak into the
    buffer (the #339 "don't drop half the wiring" lesson: the LADDER ARM and the TEXT-BLOCK LINE are the two
    easy-to-forget sites).
  - A `goto_overlay(...)` render fn (mirror `rename_draft_overlay` at app.rs:10474) — a small "Go to line…"
    chip showing the input. RENDER deferred (chad at machine); the STATE is headless-proven.
- **⌃G keymap row** (`keymap.rs`): `chord(false, true, false, false, "g")` → `"go-to-line"`, Editor-scoped
  (verified FREE; the terminal's raw ⌃G/BEL path is untouched). Roster **71→72**, scoped **26→27**.

### §20 — CONFIRMED (N/A-adjacent: OBSERVED behavior)
VS Code / Zed = OBSERVED (⌃G, `N:C`, clamp-never-error, live-preview scroll, Esc restores). The parse +
overlay are Marley-original over shipped seams (`caret_for_line_col`, `scroll_editor_to_row`, the
renaming_symbol overlay). NO new deps.

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/code_view.rs` | **NEW `parse_goto`** (pure, cov/MSI 100) beside `caret_for_line_col`. |
| `crates/marley_app/src/app.rs` | `goto_line: Option<GotoDraft>` field + init + the `GotoDraft` struct; the `"go-to-line"` dispatch arm (opens the overlay); the overlay key arm in the ladder (esc/enter/backspace/digit-or-colon + re-preview); the commit (caret_for_line_col → NavStack push → set caret → scroll) + Esc restore; **the `text_input_blocked` line**; the `goto_overlay` render chip. All shim (mutants::skip). |
| `crates/marley_app/src/keymap.rs` | ⌃G = `chord(false,true,false,false,"g")` → `"go-to-line"`, Editor-scoped; roster 71→72, scoped 26→27 (individual asserts first). |
| `crates/marley_app/src/headless_drive.rs` | the drives (below). |

**The 2 easy-to-forget wiring sites (the #339 lesson):** the overlay-ladder ARM and the `text_input_blocked`
LINE — a missing arm = ⌃G does nothing; a missing text-block line = digits leak into the buffer.

### Regression Test Plan
| REQ | Test | Where | Kind |
|---|---|---|---|
| REQ-001 | `parse_goto` truth table: `"50"`→`(50,None)`, `"50:12"`→`(50,Some(12))`, `""`/`"abc"`/`"50:"`/`":5"`/`"0"`/`"50:0"`→`None`, `"999999"`→`(999999,None)` | code_view | pure table |
| REQ-002 | past-EOF clamps to the last line (a `parse_goto` + `caret_for_line_col` combo row — `caret_for_line_col`'s own clamp is already tested at code_view.rs:958, cited not re-proven) | code_view | pure |
| REQ-003 | ⌃G → type `"50"` → Enter → the primary caret is on line 50 (`line_col(caret).0 == 49`) | headless | drive |
| REQ-004 | Esc restores the prior caret (and reveals it) — the origin caret is back, `goto_line` cleared | headless | drive |
| REQ-005 | Enter on a valid target PUSHES the NavStack (`nav_stack_depth` +1); Esc does NOT | headless | drive |
| REQ-006 | while the overlay is open, typed digits do NOT reach the buffer (text_input_blocked) | headless | drive — the leak row |
| REQ-007 | the live-preview: after typing a valid target the caret is STILL at origin (unmoved until Enter) | headless | drive |
| REQ-008 | ⌃G resolves on an editor tab; the terminal's raw ⌃G path is byte-identical (keymap Editor-scoped) | keymap unit | |

**What a green suite would NOT prove:** the overlay PIXEL (the chip render) + the exact scroll offset — both
deferred (chad at the machine). The caret STATE (origin on Esc, target on Enter) + the NavStack + the leak
are all headless-provable; the scroll TARGET is a deterministic `scroll_editor_to_row(row)` (the row is
asserted), the pixel deferred.

### Risks
1. **The origin-capture / Esc restore (D-ESC-REVEALS-ORIGIN-CARET)** — v1 captures the CARET only and Esc
   re-reveals it via `follow_editor_caret` (which centers it), rather than restoring the exact pre-⌃G scroll
   offset. A minor, defensible simplification (Esc returns you to your caret, visible); the exact-pixel
   restore (via `active_scroll_px`) is a named follow-up if wanted. Avoids the scroll-sync timing.
2. **The leak gate** — the `text_input_blocked` line is load-bearing (a miss = digits in the buffer); pinned
   by REQ-006.
3. **The phantom-row / clamp** — owned entirely by `caret_for_line_col` (already tested); `parse_goto` never
   touches ranges. LOW.

### Live-pixel note
The go-to-line is a caret + scroll-target STATE change — headless-provable via `editor_caret` + the NavStack
depth + `goto_line` state. The overlay chip render + the exact scroll pixel are deferred (chad at the
machine); the state asserts + the `scroll_editor_to_row(row)` mechanism carry it, deferred-not-skipped.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement

### Built (to the manifest)
- **`crates/marley_app/src/code_view.rs`** — pure `parse_goto(input) -> Option<(usize, Option<usize>)>`
  beside `caret_for_line_col`. `split_once(':')`; each part via `s.parse::<usize>().ok().filter(|&n| n >= 1)`
  (usize::from_str rejects empty/non-digit/signed/whitespace; the `≥1` filter rejects `0`); a `?` on the col
  makes a bare/trailing `:` → None. SYNTAX only — `caret_for_line_col` owns the range clamp.
- **`crates/marley_app/src/app.rs`** — `struct GotoDraft { input, origin_caret }` + the `goto_line:
  Option<GotoDraft>` field + `None` init; the `"go-to-line"` dispatch arm (seeds the origin caret); the
  overlay key arm HIGH in the ladder (before `renaming_symbol`) — escape→`goto_restore`, enter→`goto_commit`,
  backspace→pop+`goto_preview`, a DIGIT-or-`:` char→push+`goto_preview` (via `key_char` so shift-`;`→`:`
  works; a letter/space is swallowed); the three shim methods (`goto_commit` = parse→caret_for_line_col→
  NavStack-push-the-ORIGIN→set caret→scroll; `goto_restore` = origin caret + `follow_editor_caret`;
  `goto_preview` = scroll to the target without moving the caret); the `text_input_blocked` line; the
  `goto_overlay` render chip (**fixed near top-center, NOT caret-anchored** — the live preview scrolls the
  caret out of view, so a caret-anchored chip would vanish) + its wiring in the render stack.
- **`crates/marley_app/src/keymap.rs`** — ⌃G = `chord(false,true,false,false,"g")` → `"go-to-line"`,
  Editor-scoped; roster 71→72, scoped 26→27, both asserted individually before the count bumps.

### parse_goto — WATCHED it work (throwaway probe, run then DELETED; `grep zzz_goto_probe` → 0)
`"50"`→`(50,None)`; `"50:12"`→`(50,Some(12))`; `""`/`"abc"`/`"50:"`/`":5"`/`"0"`/`"50:0"`/`" 50"`/`"50:0x"`→
`None`; `"999999"`→`(999999,None)` (syntax valid; `caret_for_line_col` will clamp the range); `"1:1"`→
`(1,Some(1))`. All correct — the Phase 4 truth table now has known-good expected values.

### Deviations from design
- **The overlay chip is FIXED-position (top-center), not caret-anchored** (a small delta from "mirror
  rename_draft_overlay"): the live preview deliberately scrolls the origin caret off-screen, and a
  caret-anchored chip (which `rename_draft_overlay` returns `None` for when the caret is out of view) would
  DISAPPEAR mid-type. A fixed chip stays visible. Named + reasoned.
- **No separate `"space"` arm** (rename has one to ALLOW spaces): a space is invalid in a line number, so it
  falls through to `_ => {}` and is swallowed — simpler than rename's.

### Verified
`cargo check --workspace` CLEAN; `cargo clippy -p marley --all-targets` CLEAN; `cargo nextest -p marley --lib
-E 'test(/chord|roster|keymap/)'` → 20 passed (count 72, scoped 27, `all_chords_lists_every_binding`);
`cargo fmt --all`; §20 CLEAN. Test hooks deferred to Phase 4 (the #337 F2 rule).

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

Two parallel general-purpose critics — lens 1 `parse_goto` + the commit/restore/preview logic, lens 2 the
wiring completeness + THE LEAK GATE. Both verified concretely (a throwaway `parse_goto` truth-table test run
then DELETED; the leak gate traced to the insert path). **Critic 2 (wiring) came back FULLY CLEAN** — the
leak gate is DOUBLE-protected (`text_input_blocked` gates `ime::replace_text`'s only two callers at
:11863/:11896 BEFORE the buffer, AND the overlay arm's unconditional `stop_propagation; return` covers even
`_ => {}`), the verb strings match, ⌃G is off-terminal-inert twice over (Editor-scope resolution + the
dispatch no-op), the fixed chip renders whenever `goto_line.is_some()`, mutants 0/8, roster 72/27. Critic 1
found NO critical/high/medium and **3 LOW notes**; 1 fixed, 2 dispositioned.

| # | Sev | Finding | Verdict | Disposition |
|---|-----|---------|---------|-------------|
| L1 | **LOW** | **`goto_restore` omitted `clear_marked`** — asymmetric with `goto_commit`. A caret placement is an out-of-band edit; `clear_marked`'s contract says "every non-IME edit/interaction path calls this" (a stale IME composition span would misdirect the next replace). No user-visible repro (needs mid-IME-composition when a control chord fires), but a real latent gap. | **REAL — confirmed** (the asymmetry is in the diff). | **FIXED** — added `s.clear_marked()` before the placement in `goto_restore`, symmetric with `goto_commit`. |
| L2 | LOW | `"50:12:3"` (two colons) → `None`, where VS Code takes line 50 col 12. `split_once(':')` gives col `"12:3"` → parse fails. | **REAL but ACCEPTED** — the D-CLAMP-NEVER-ERROR stance: odd/partial input is inert `None`, never a mis-jump. A user types `"50:12"`; the extra `:3` is unusual. | **No change** — documented as the deliberate "inert on odd input" design (a lenient-parse follow-up if anyone asks). |
| L3 | LOW | Hand-rolled `set_active_selections(SelectionSet::single(Selection::caret(off)))` vs the shipped `EditorSurface::set_single_caret(off)`. | **REAL (cleanup)** | **FIXED** — both `goto_commit` and `goto_restore` now use `set_single_caret` (which is exactly that single-caret write); it does NOT bundle `clear_marked`, so the L1 fix keeps the explicit `clear_marked` before it. |

**Both fixes verified:** `cargo clippy -p marley --all-targets` CLEAN; `cargo nextest -p marley --lib -E
'test(/chord|roster|keymap/)'` → 20 passed; `cargo fmt`. **Phase 4 owes:** the `parse_goto` truth table
(the mutation surface, 8 mutants), plus the headless drives (⌃G→type→Enter, Esc-restore, NavStack-on-commit,
the leak, the live-preview-caret-unmoved).

**Lesson:** a caret-placement path (jump/restore/go-to) must call `clear_marked` before the placement,
symmetric with every sibling — a stale IME composition span misdirects the next replace. Grep new
caret-placement sites for a MISSING `clear_marked` against the ones that have it.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 4 — Validate

**Tests added (6):**

| Test | File | REQ | What it pins |
|---|---|---|---|
| `parse_goto_line_and_col_syntax` | code_view.rs `mod tests` | REQ-001 | The truth table — the 8-mutant surface. Valid `"50"`/`"50:12"`/`"1"`/`"1:1"`/`"999999"`; None `""`/`"abc"`/`"5a"`/`"50:"`/`":5"`/`":"`/`"0"`/`"1:0"`/`" 50"`/`"5 0"`/`"50:12:3"`; overflow `"1".repeat(30)`→None (no panic). Separators: `split_once` (`"50:12:3"`→None, a rsplit would parse), the `>= 1` filter (`"0"`/`"1:0"`→None), the line-`?` (`":5"`), the col-`?` (`"50:"`). |
| `goto_line_jump_places_caret_headless` | headless_drive.rs | REQ-003 | ⌃G→type`"3"`→Enter → caret at line-3 start (offset 4), overlay closed. Whole path: verb→dispatch→overlay arm→goto_commit. |
| `goto_line_escape_restores_caret_headless` | headless_drive.rs | REQ-004 | caret@6 (line 4), ⌃G→`"1"`→Esc → caret back @6, no jump, overlay closed. |
| `goto_line_navstack_pushes_on_commit_not_escape_headless` | headless_drive.rs | REQ-005 | Esc leaves nav depth; Enter bumps it +1 (the origin caret). |
| `goto_line_digits_do_not_leak_into_buffer_headless` | headless_drive.rs | REQ-006 | **the leak gate** — ⌃G open, type `"5"` → `editor_text` byte-identical (stop_propagation + text_input_blocked). |
| `goto_line_preview_does_not_move_caret_headless` | headless_drive.rs | REQ-007 | ⌃G→`"3"` (valid) → caret STILL @origin (preview scrolls, never places). |

Re-added one `#[cfg(test)]` accessor — `goto_line_open_for_test() -> bool` (the #337 F2 rule: only because 5 drives read it). Two test-file wrappers `goto_open`/`nav_depth`.

**REQ coverage:** REQ-001 (table), REQ-002 (caret_for_line_col's own clamp test `caret_for_line_col_places_and_clamps` — cited, not re-tested, per design), REQ-003/004/005/006/007 (drives above), REQ-008 (⌃G chord + roster asserts, Phase 3). The overlay chip PIXEL + exact center-scroll offset are **deferred-not-skipped** (LIVE drive off-limits — chad at the machine); the caret/NavStack/leak/preview STATE is fully headless-proven.

**Runs (actual):**
- `cargo nextest -p marley --lib -E 'test(/goto_line_/) + test(parse_goto...)'` → **6 passed** (the drives + the table). The `simulate_keystrokes("3"/"enter"/"escape")` real key path routes to the overlay arm — proven by the leak drive (buffer byte-identical) and the jump drive (caret placed on Enter).
- `cargo mutants --list -f code_view.rs | grep -c parse_goto` = **8** (the tested pure surface); `-f app.rs | grep -ic goto` = **0** (shims mutants::skip'd).
- **Full `scripts/gates.sh --diff` → GATE GREEN [diff]**, 15/15:
  - gate:4 coverage **100% lines** (parse_goto covered — no dead branch; the fn is total).
  - gate:5 mutation **8 caught / 0 missed → MSI 100.0%**.
  - Receipt `5943d069…` == live `gate_state_hash` (commit-valid).
- No pre-existing failures encountered; no #348 hang this run.

## Phase 5 — Complete
- Docs updated; AAR capture; archive.
