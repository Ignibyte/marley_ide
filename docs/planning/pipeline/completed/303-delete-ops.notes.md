# The missing destructive ops (delete word / to line edge / delete line) — Notes

- **Forge ticket:** #303 38691b71-83d9-4a0d-acb2-e40ea9a4bc34
- **AAR:** ca13a3bd-f9d0-4224-9008-0a44bfb77d63
- **Local ticket doc:** docs/planning/tickets/open/TICKET-303-delete-ops.md
- **Pipeline spec:** 303-delete-ops.spec.md

<!-- Working scratch. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** the destructive halves of the #257 word-motions — ⌥⌫/⌥⌦ delete word, ⌘⌫/⌃K to line edge, ⌘⇧K delete line. Promoted from the pre-authored queued spec (the Fable method); THIRD of the goal `/work 300,302,303,304,305,314,315,316,317,259`.
- **Classification / tier:** work pipeline, one shippable slice (a pure op→range table + a key-translation seam fix + 5 keymap rows). M19 editor power-tools.
- **Promotion done:** `git mv` queued→active; pipeline_id `ee3a0377-591c-4e9a-9866-e377f7b493c8`; aar `ca13a3bd-f9d0-4224-9008-0a44bfb77d63`; TICKET-303 created.
- **Seam re-verification (the pre-authored spec was written on Fable; #300 + #302 both landed since, drifting app.rs twice — EVERY cited line was treated as stale).** Verified inline against `main` @ `e1c6436` (an Explore agent was also dispatched for cross-check; the inline findings below are authoritative).

### Phase 1 findings (seam re-verification)

**F1 — THE PREMISE IS TRUE, BUT ITS FEARED FIX IS UNNECESSARY (the #302-class catch — a confident sentence shrinks the ticket).**
The spec's central claim holds: `key_from_keystroke` (app.rs:**2473**, not :2465) returns `Key::Backspace`/`Key::Enter` at lines 2475-2479 **before** the modifier gate at 2481 (`if control || platform { return Key::Other }`). So ⌥⌫ (alt only) enters the editor plain-key block (app.rs:**12681**, gated `!platform && !control`), maps to `Key::Backspace`, and **1-char-deletes today** — confirmed.
BUT the spec's proposed remedy — "move the backspace early-return below the modifier check (or the router learns the chords)" (D-FIX-THE-TRANSLATION-SEAM) — is **not needed**. The keymap chord path already runs FIRST and already preserves every modifier:
- app.rs:**12660-12662**: `let binding = binding_from_keystroke(&event.keystroke); … if let Some(action) = view.keymap.action_for(&binding, stack) { dispatch_action(action); … return; }` — this is ABOVE the `key_from_keystroke` block (12681).
- `binding_from_keystroke` (app.rs:**2512**) captures `{cmd, ctrl, alt, shift, key}` faithfully — so a row `(F,F,T,F,"backspace")` for ⌥⌫ matches HERE and returns before `key_from_keystroke` is ever called.
- **Consequence:** adding the 5 Editor-scoped keymap rows + their `dispatch_action` arms routes ALL FIVE modified chords correctly with **NO change to `key_from_keystroke`**. D-FIX-THE-TRANSLATION-SEAM largely dissolves to "add the rows." (⌘⌫/⌃K/⌘⇧K never even reach the 12681 block — it's `!platform && !control` — so they already rely on keymap resolution; ⌥⌫/⌥⌦ reach it today ONLY because no row intercepts them yet.)

**F2 — THE ONE GENUINE TRANSLATION-LAYER EDIT: plain forward-delete.** REQ-008 (make plain `delete` a 1-char forward delete) is the only change to the input layer. `apply_editor_key_multi` (input.rs:**261**) handles ONLY `Key::Backspace` (line 264); `Key::DeleteForward` falls through to the implicit no-op. Plain `delete` has no modifier → no keymap row → reaches `key_from_keystroke` → `Key::DeleteForward` (app.rs:2487) → `apply_editor_key_multi` → nothing. Fixing REQ-008 = add a `Key::DeleteForward` arm to `apply_editor_key_multi`. The 5 chords do NOT touch input.rs; only plain forward-delete does. (Design: decide whether forward-delete is a `delete_range_for(WordRight-of-one-char)`… no — it's a 1-char forward delete, a distinct tiny arm, NOT the word op.)

**F3 — the reuse seams CONFIRMED (real locations; all drifted):**
| seam | spec cited | REAL | verdict |
|---|---|---|---|
| `move_word_left` | movement.rs:67 | movement.rs:**67** | exact |
| `move_word_right` | movement.rs:81 | movement.rs:**81** | exact |
| `rebase_through` clamp | indent.rs:131 | indent.rs:**123** (`if p < at+remove { return at+delta }`, lines 131-133) | DRIFTED; clamp CONFIRMED — a position inside a removed span → the span start = the correct post-delete caret home. **D-REBASE-IS-CORRECT-HERE holds** (the inverse of #300, where the same clamp was WRONG for a move). |
| grouped-edit idiom | app.rs:6676 (#299) | `apply_line_reorder` app.rs:**3206** (#300) — `before → begin_undo_group → raw edit() BACK-TO-FRONT → end_undo_group → set_selection` | the #300 template is the exact shape to mirror (a per-cursor delete list applied back-to-front + carried set) |
| terminal ⌃K | op_for_ctrl_key input.rs:72 | `op_for_ctrl_key` input.rs:**68**, `"k" => KillToEnd` input.rs:**72** | CONFIRMED independent — the terminal ⌃K is on the readline path (`apply_readline`), and an Editor-scoped chord does not even resolve on a terminal tab (its `key_context` has no Editor scope). REQ-007's terminal-⌃K regression is byte-identical by construction. |

**F4 — all 5 chords FREE; roster is 72/27 now (spec's 67→72, 22→27 is pre-batch stale).** No keymap row binds `"backspace"` or `"delete"` at all (grep clean). For `"k"`: only `(T,F,F,F,"k")` = **⌘K exists** (already bound) — ⌃K `(F,T,F,F,"k")` and ⌘⇧K `(T,F,F,T,"k")` differ from it (ctrl-vs-cmd; shift) and are FREE. **Design must confirm the ⌘K↔⌘⇧K shift-disambiguation** (they collide only if `action_for` ignores shift — it does not, per binding_from_keystroke). Target roster: **72→77 total, 27→32 scoped** (5 Editor-scoped rows), individual `.contains` asserts before the count bump (#337 discipline).

**F5 — no double-fire (the spec's P3.5 worry is defused by F1).** Because the keymap resolves + `return`s (app.rs:12662-12676) before `key_from_keystroke`, the 5 chords cannot double-fire. Plain backspace (no row) still flows to `key_from_keystroke` → `apply_editor_key_multi` (unchanged) → byte-identical (REQ-007). The #338 pair-backspace arm lives inside `apply_editor_key_multi`'s Backspace path (the `auto_close` branch, input.rs:266) — reached only by plain backspace, so a keymap-routed ⌥⌫ bypasses it entirely; no interaction.

**Scope reshaping (net of F1+F2):** the ticket is now (a) the pure `delete_range_for` table in crates/editor (unchanged from spec), (b) **5 keymap rows + 5 dispatch_action arms** (NOT a key_from_keystroke restructure — F1), (c) **one `Key::DeleteForward` arm in `apply_editor_key_multi`** for plain forward-delete (F2), (d) the grouped-apply mirroring `apply_line_reorder`. D-FIX-THE-TRANSLATION-SEAM is REFRAMED: "the keymap already sees modifiers; route via rows, and add the one forward-delete arm" — recorded so Phase 2 doesn't cargo-cult a key_from_keystroke rewrite.

**Cross-check (an Explore agent independently traced all 7 claims — CORROBORATED; two refinements folded in):**
- **⌘⌫ mechanism CORRECTED (net conclusion unchanged):** ⌘⌫ has `platform=true`, so it is gated OUT of the `key_from_keystroke` editor block (12681 `!platform`) and instead hits `swallow_hidden_prompt_key` (app.rs:**12937** → input.rs:88), which returns true for `"backspace"` → `stop_propagation` → **swallowed no-op today** (matching the spec's original "⌘⌫ swallowed, input.rs:88" — my first read of "⌘⌫ → Key::Backspace" was wrong; it never reaches key_from_keystroke). A keymap row `(T,F,F,F,"backspace")` resolves at 12662 before either path → fixed.
- **THE SHARP PHASE-2 CONSTRAINT (agent, Claim 7):** the new word-delete MUST be a **DISTINCT dispatch op** (a new verb → its own apply path mirroring `apply_line_reorder`), **NOT folded into `apply_editor_key_multi`'s `Key::Backspace` arm** — that arm's `auto_close=true` branch is the #338 pair-backspace (`backspace_pairing_at_selections`, buffer.rs:445), and folding a word-delete there would make ⌥⌫ inherit auto-pair widening. Keep the delete ops entirely on the keymap→dispatch_action→grouped-edit path; leave `apply_editor_key_multi` for plain backspace (+ the one new plain-forward-delete arm) only.
- **Corroborations:** the readline `KillWordBack` (input.rs:**142-148**) ALREADY does `move_word_left(buffer, caret)` → `kill_span` — the motion-as-deletion-endpoint reuse (D-MOTION-IS-THE-RANGE) is already proven in the terminal path. ⌘K is bound TWICE — `(T,F,F,F,"k")` global clear-screen (keymap.rs:189) + Editor hover (keymap.rs:304) — both differ from ⌃K/⌘⇧K (ctrl-vs-cmd; shift), so the two new "k" chords are free (design confirms `action_for` honors shift/ctrl, which `binding_from_keystroke` carries). Tuple order CONFIRMED `(cmd, ctrl, alt, shift, key)` (keymap.rs:37).

- **§20 + prior art:** CONFIRMED unchanged — the five chords + their edges (⌃K-eats-EOL, ⌘⇧K-last-line) are OBSERVED macOS/readline/VS-Code behavior; every deletion boundary is OUR `movement.rs`; the sweep's "what no crate owns = the routing" refines to "the routing is mostly already there (F1); the genuine delta is the range table + the forward-delete arm."
- **AC / decisions:** all 9 REQ hold. D-FIX-THE-TRANSLATION-SEAM reframed (F1) — not reopened, its REQ-007/008 behavior stands; the mechanism is smaller than feared.

## Phase 2 — Design

### 1. Architecture / approach
The whole ticket is a pure range table + a grouped-apply shim that mirrors #300's `apply_line_reorder`
byte-for-byte (Phase 1 F3), routed through 5 keymap rows (F1 — no `key_from_keystroke` change), plus one
small forward-delete arm (F2). No new deps; §14 total/no-panic; §20 observed macOS/readline/VS-Code.

**The pure seam — NEW `crates/editor/src/delete.rs`** (a sibling to `line_move.rs`/`movement.rs`; re-exported
from `lib.rs`). Kept OUT of `movement.rs` deliberately: that file is pure MOTION (returns a caret); this is
pure EDIT (returns a range/edits). Two public items:

- `pub enum DeleteOp { WordLeft, WordRight, ToLineStart, ToLineEnd, WholeLine }`
- `pub fn delete_range_for(op: DeleteOp, buffer: &Buffer, sel: &Selection) -> Option<Range<CharOffset>>` — the
  per-cursor range. **D-SELECTION-WINS lives HERE (pure):** `if sel.anchor() != sel.head()` → `Some(min..max)`
  for EVERY op (a non-empty selection is what every op deletes). Else, on the bare caret `c = sel.head()`:
  | op | range | edge → None |
  |---|---|---|
  | WordLeft | `move_word_left(buffer,c) .. c` | `move_word_left == c` (offset 0) |
  | WordRight | `c .. move_word_right(buffer,c)` | `move_word_right == c` (EOF) |
  | ToLineStart | `move_line_home(buffer,c) .. c` | `move_line_home == c` (already col 0) |
  | ToLineEnd | `c .. move_line_end(buffer,c)`; **at EOL** (`move_line_end==c`) → `c .. c+1` (eat the `\n`, D-CTRL-K-EATS-EOL-NEWLINE) | `c==len_chars` (EOF, no `\n`) |
  | WholeLine | trailing `\n` (`row+1 < len_lines`) → `line_start(row) .. line_start(row+1)`; **last line** (`row>0`) → `line_start(row)-1 .. len_chars` (eat the PRECEDING `\n`, D-LAST-LINE-EATS-PRECEDING-NEWLINE); **only line** → `line_start(0) .. len_chars` (empties the buffer) | never None (a line always exists) |
  The motion fns (`move_word_left/right` movement.rs:67/81, `move_line_home` :96, `move_line_end` :102) OWN
  every boundary — D-MOTION-IS-THE-RANGE, zero new boundary logic; `move_line_end` already returns
  "just before the trailing `\n`, or len_chars on the final line", so the EOL/EOF discrimination is a single
  `== c` test.
- `pub fn delete_edits(buffer: &Buffer, set: &SelectionSet, op: DeleteOp) -> Option<(Vec<LineEdit>, SelectionSet)>`
  — the multi-cursor collector, shaped EXACTLY like `move_lines` so the app shim is a one-word swap. Maps each
  selection through `delete_range_for`, drops `None`s, **merges overlapping ranges** (two carets on one line
  both doing WholeLine → one deletion — REQ-006 collision-merge), sorts ASCENDING, builds `(at, remove, "")`
  `LineEdit`s, and returns `(edits, rebase_selections(set, &edits))`. **`rebase_selections` IS the carry**
  (D-REBASE-IS-CORRECT-HERE, indent.rs — a caret at the deleted span's end rebases to its start; verified:
  WordLeft `[t,c)` → caret `c` rebases to `t`; WordRight `[c,t)` → caret `c` stays at `c`; both = the deletion
  start, the correct home). Returns `None` iff EVERY cursor yielded `None` (all at edges → a clean no-op, no
  undo group opened — the `.first()?`/total-fn coverage lesson from #300: use `edits.is_empty().then(...)`
  via `?` on a non-dead path).

**The app shim — NEW `apply_delete(&mut self, op: DeleteOp)`** (app.rs, `#[cfg_attr(test, mutants::skip)]`):
a verbatim clone of `apply_line_reorder` (app.rs:3206) with `move_lines(...)` → `delete_edits(..., op)`. Same
`clear_marked()` → `before = active_selections().clone()` → `begin_undo_group(before)` → raw `edit(range,"")`
BACK-TO-FRONT → `end_undo_group(carried)` → `set_selection(carried)` → `follow_editor_caret()`. **THE SHARP
CONSTRAINT (Phase 1):** this is a DISTINCT path — it never touches `apply_editor_key_multi`'s `Key::Backspace`
arm, so ⌥⌫ cannot inherit the #338 auto-pair widening.

**Routing (F1 — rows only):** 5 dispatch_action verbs → `apply_delete(op)`:
`"delete-word-left"→WordLeft`, `"delete-word-right"→WordRight`, `"delete-to-line-start"→ToLineStart`,
`"delete-to-line-end"→ToLineEnd`, `"delete-line"→WholeLine`. 5 Editor-scoped keymap rows resolve at
app.rs:12662 BEFORE `key_from_keystroke` — no seam change.

**Plain forward-delete (F2, REQ-008)** — the ONE input.rs edit: an `else if key == Key::DeleteForward` arm in
`apply_editor_key_multi` (input.rs:261) calling a NEW `Buffer::delete_forward_at_selections(set, origin)`
(mirrors `backspace_at_selections` buffer.rs:425 but forward: a non-empty selection → delete it; else delete
the 1 char right of the caret, saturating at len_chars — a no-op at EOF). Honors a selection (D-SELECTION-WINS)
symmetric with backspace. This is NOT the auto-pair path (no `_pairing_` variant) — plain forward-delete never
eats a matched closer.

### 2. File manifest
| File | Change |
|---|---|
| `crates/editor/src/delete.rs` | NEW — `DeleteOp`, `delete_range_for`, `delete_edits` + the pure truth-table tests |
| `crates/editor/src/lib.rs` | `mod delete; pub use delete::{DeleteOp, delete_range_for, delete_edits};` |
| `crates/editor/src/buffer.rs` | NEW `delete_forward_at_selections(&mut self, set, origin)` (mirrors `backspace_at_selections`, forward) |
| `crates/marley_app/src/app.rs` | `apply_delete(op)` shim (mirrors `apply_line_reorder`); 5 `dispatch_action` arms (verb→op) |
| `crates/marley_app/src/input.rs` | the `Key::DeleteForward` arm in `apply_editor_key_multi` → `delete_forward_at_selections` |
| `crates/marley_app/src/keymap.rs` | 5 Editor-scoped rows (⌥⌫/⌥⌦/⌘⌫/⌃K/⌘⇧K); roster 72→77, scoped 27→32; individual `.contains` asserts first (#337) |
| `crates/marley_app/src/headless_drive.rs` | the drives (per-chord range, N-cursor undo/carry/merge, REQ-007/008/009 regressions) |

### 3. Regression Test Plan (≥1 row per REQ)
| # | Test | Kind | Pins |
|---|---|---|---|
| REQ-001 | `delete_range_for_word_uses_motion_boundaries` | pure | WordLeft/Right ranges == `move_word_left/right`; **multibyte gap `"foo é😀 bar"`** (the char-vs-byte separator, #300/#336 class) |
| REQ-002 | `delete_range_for_selection_wins_for_every_op` | pure | a non-empty sel → `min..max` for all 5 ops |
| REQ-003 | `delete_range_for_word_edge_is_none` | pure | WordLeft@0 → None; WordRight@len → None (no underflow) |
| REQ-004 | `delete_range_for_line_edges` | pure | ToLineStart `[home,c)`; ToLineEnd `[c,end)`; **⌃K at EOL → `[c,c+1)` (the `\n`)**; ToLineEnd at EOF → None |
| REQ-005 | `delete_range_for_whole_line_roundtrips` | pure | WholeLine mid `[start,next)`; **last line `[start-1,len)`**; only line `[0,len)`; delete each line of `"a\nb\nc"` → `"a\nb"`/`"a\nc"`/`"b\nc"`, NO orphan blank |
| REQ-006 | `delete_edits_multi_cursor_merges_and_carries` (pure) + `delete_word_n_cursors_one_undo_headless` | pure + headless | overlapping ranges merged; carets carried by rebase; one undo unit |
| REQ-007 | `terminal_ctrl_k_still_kills_to_end_headless` + `editor_plain_backspace_unchanged_headless` | headless | terminal ⌃K = KillToEnd byte-identical; plain ⌫ = 1-char (the regression rows) |
| REQ-008 | `plain_forward_delete_removes_char_right_headless` | headless | `delete` now deletes 1 char right (was a no-op); honors a selection |
| REQ-009 | `delete_word_one_undo_no_group_leak_headless` | headless | one ⌘Z reverts the whole delete; a typed char after is its OWN unit |

Coverage/MSI 100 on `delete.rs` (the pure surface); `apply_delete`/dispatch arms are app.rs shims (mutants::skip);
`delete_forward_at_selections` is a buffer method (mutation-included — a real test). No uncoverable path — every
op is buffer+selection STATE, fully headless (LIVE pixel N/A — no render change; the caret/scroll follow is the
existing `follow_editor_caret`).

### 4. Risks / decisions
- **The DISTINCT-dispatch-op** (load-bearing, Phase 1): word-delete goes through `apply_delete`, NEVER
  `apply_editor_key_multi`'s Backspace arm — else it inherits #338 auto-pair widening. Inspect will re-check.
- **The two pinned edges:** ⌃K-at-EOL-eats-`\n` and ⌘⇧K-last-line-eats-preceding-`\n` — both round-trip
  tests (delete leaves no orphan blank / joins the line), the ticket's real subtlety.
- **⌘K ↔ ⌘⇧K disambiguation:** ⌘K `(T,F,F,F,"k")` is bound twice; ⌘⇧K `(T,F,F,T,"k")` differs by `shift`,
  which `binding_from_keystroke` carries and `action_for` honors — a `keymap` unit asserts ⌘⇧K resolves to
  delete-line and ⌘K still resolves to its existing verbs (no collision).
- **Forward-delete honors a selection** (decided): symmetric with backspace (D-SELECTION-WINS). A bare-caret
  forward-delete at EOF is a no-op (saturate), not a panic.
- **The all-edges no-op:** `delete_edits` returns `None` when every cursor is at an edge → no undo group
  opened, no set rewrite (mirrors `move_lines`' `None`; the #300 `.first()?` totality lesson — no dead guard).

## Phase 3 — Implement

Built to the manifest; `cargo check --workspace` clean, `cargo fmt` applied, `git add -N delete.rs` done
(so its mutants aren't skipped at the --diff gate — the M21 lesson).

**What was built:**
- **`crates/editor/src/delete.rs`** (NEW, pure) — `DeleteOp` (5 variants, `Debug/Clone/Copy/PartialEq/Eq`
  mirroring `VDir`), `delete_range_for(op, buffer, sel) -> Option<Range<CharOffset>>` (D-SELECTION-WINS first,
  then the 5 op arms incl. the ⌃K-at-EOL `\n`-eat and the WholeLine 3-way last-line/only-line edges),
  `delete_edits(buffer, set, op) -> Option<(Vec<LineEdit>, SelectionSet)>` (filter_map → sort → merge
  overlapping/adjacent/duplicate → ascending `(at, remove, "")` edits → `rebase_selections` carry; `None` when
  every cursor is at an edge). No `mod tests` yet — Phase 4 owns the truth tables (the §7 split).
- **`crates/editor/src/lib.rs`** — `mod delete;` + `pub use delete::{delete_edits, delete_range_for, DeleteOp};`
  (mirrors the line_move re-export).
- **`crates/editor/src/buffer.rs`** — NEW `delete_forward_at_selections(&mut self, set, origin) -> SelectionSet`
  mirroring `backspace_consuming` but FORWARD (bare caret → `[head, (head+1).min(len))`; selection → itself),
  reusing the tested `edit_ranges_restoring` machinery (undo group + set_selection internal). Never pairs.
- **`crates/marley_app/src/app.rs`** — `apply_delete(op)` shim (`#[cfg_attr(test, mutants::skip)]`), a verbatim
  clone of `apply_line_reorder` with `delete_edits` as the source; 5 `dispatch_action` arms (verb → op).
- **`crates/marley_app/src/input.rs`** — the `else if key == Key::DeleteForward` arm in `apply_editor_key_multi`
  → `delete_forward_at_selections` (the viable-mutant `-> ()` comment preserved; DeleteForward now EDITS).
- **`crates/marley_app/src/keymap.rs`** — 5 Editor-scoped rows (⌥⌫→delete-word-left, ⌥⌦→delete-word-right,
  ⌘⌫→delete-to-line-start, ⌃K→delete-to-line-end, ⌘⇧K→delete-line); roster **72→77 / scoped 27→32** with a
  `#303 ×5` breakdown; individual `all_chords`/`scoped_entries` `.contains` asserts before the count bumps
  (#337). NEW resolution test `delete_ops_resolve_and_ctrl_k_is_editor_only` — the shift-disambiguation
  (⌘⇧K→delete-line while ⌘K stays lsp-hover/clear-screen) AND ⌃K-on-terminal → `None` (REQ-007 at the keymap
  level — the readline ⌃K path is untouched).

**Deviations from design:** none. The design's F1 held exactly — no `key_from_keystroke` change; the 5 rows
route via `keymap.action_for` (app.rs:12662) before the seam.

**Compile/test as-you-go:** `cargo check --workspace` clean; `cargo nextest -p marley --lib -E
'test(/chord|roster|keymap|delete_ops|all_chords|scoped/)'` → 21 passed; `cargo nextest -p marley_editor`
→ 221 passed (the new buffer method + delete.rs break nothing). NO test expansion beyond compile + the
roster/keymap asserts — Phase 4 owns delete.rs's tables + the drives.

## Phase 3.5 — Inspect

Two parallel general-purpose critics (correctness of delete.rs; wiring/regression) + my own independent
verification (a throwaway integration test `tests/zz_303_throwaway.rs` over the edges, since removed; clippy;
fmt; verb-string match; mutants --list). **Result: ONE real LOW defect (C1-LOW), fixed at source; all else CLEAN.**
Critic 1 (correctness) independently ran 8/8 edge checks PASSED — corroborating my throwaway exactly — AND
caught the one asymmetry my self-check missed (I discarded the WholeLine-on-empty return with `let _ =`
instead of asserting it — the value of a second lens even after a passing self-check).

**Lenses covered:** delete.rs range-table correctness + the 2 pinned edges + totality; multi-cursor
merge/carry; the DISTINCT-dispatch-op constraint; the routing/double-fire (F1); the forward-delete arm
(REQ-008); the terminal ⌃K + plain-backspace regression (REQ-007); roster honesty; clean-room; mutation surface.

| # | Finding | Severity | Verdict | Resolution |
|---|---|---|---|---|
| C2-1..6 | Wiring critic swept all 6 axes | — | **CLEAN — no defects** | Distinct-dispatch confirmed (`apply_delete` calls `delete_edits` + undo-group prims, NEVER `backspace_pairing`); verb strings byte-exact across keymap↔dispatch; the `return` at app.rs:12712 (above `key_from_keystroke`) defuses the ⌥⌫ double-fire; forward-delete correct at EOF (zero-width no-op, no panic); terminal ⌃K resolution → None (op_for_ctrl_key untouched, 0 diff hunks); roster 77/32 honest; no zed/warp, no unwrap. |
| S-1 | **My throwaway "failure" #1 — multibyte WordRight** | N/A (test-calibration, NOT a code defect) | **REJECTED — code is correct** | `WordRight` over `"aé😀 b"` from 0 returns `0..2` (`"aé"`), because `😀` is NOT a word char (`is_word_char` = alphanumeric ∨ `_`). I'd expected `0..3`. That it returns the CHAR offset 2 (byte would be 3) actually PROVES char-correctness (the #300/#336 separator). Fixed my expectation; re-ran → pass. |
| S-2 | **My throwaway "failure" #2 — adjacent-line merge** | N/A (by design) | **REJECTED — code is correct** | Two carets on ADJACENT lines (WholeLine `[0,2)`+`[2,4)`) MERGE to one span `[0,4)` (my `s <= last.1` fold), text-identical to two separate deletes. I'd asserted `len()==2`; the design says merge adjacent/overlapping/duplicate. Fixed the test to use NON-adjacent lines for the "2 edits" case; re-ran → pass. |
| E-1 | delete.rs edge round-trips (my throwaway + Critic 1's 8/8) | — | **ALL PASSED** | WordLeft/Right + edge-None; multibyte char offsets; ToLineStart; ToLineEnd + ⌃K-at-EOL-eats-`\n` (`ab\ncd` caret 2 → `[2,3)`) + EOF→None; WholeLine 3-way (`a\nb\nc`: line1→`b\nc`, line2→`a\nc`, **last line→`a\nb` no orphan blank**, only line→`""`) + trailing-newline phantom (`a\nb\n` caret 4 → `Some(3..4)`, sane); delete_edits merge + disjoint (2 edits → `b\nd\ne`) + all-edges→None; D-SELECTION-WINS incl. backward drag; empty-buffer × 5 ops → no panic (§14 total). |
| **C1-LOW** | **WholeLine on a truly EMPTY buffer returned `Some(0..0)`, not `None`** | **LOW** | **REAL — confirmed + FIXED** | The only-line `else` branch returned `Some(start..len)` unconditionally → `Some(0..0)` on `""`, asymmetric with the 4 other ops (all `None` at their edge). Since `apply_delete` opens an undo group DIRECTLY (no `edit_ranges_restoring` no-op guard), ⌘⇧K in an empty editor would record a dead ⌘Z step (reachable: a new/emptied file). **Fixed:** `(start < len).then(\|\| …)` → `None` on empty. `cargo check` clean; Phase 4 pins empty→None + non-empty→Some. Forge `BF-claude-wholeline-delete-empty-buffer-returns-zero-width-not-none-001`. My throwaway missed it (discarded the return with `let _ =`); the independent correctness critic caught it. |

**Mutation surface (verified):** delete.rs = 36 mutants (the pure target, `git add -N`'d so not silently
skipped); buffer.rs `delete_forward` = 3; app.rs `apply_delete`/dispatch = 0 (mutants::skip). Phase 4 kills the 39.

**Hygiene:** clippy clean (only the pre-existing `block v0.1.6` future-incompat, unrelated); `cargo fmt --check`
clean; a stray throwaway from a critic was flagged and removed (`git status` clean of all throwaways).

**Lesson (a class worth recording):** before asserting a word-boundary or a merge result, pin the
`is_word_char` class (alphanumeric ∨ `_` — emoji/punctuation STOP a word) and the adjacent-range merge
semantics — a "failing" edge test is as often a miscalibrated expectation as a bug (both my throwaway trips
were mine, and BOTH confirmed the code once corrected). This is the char-class twin of the #336/#300
char-vs-byte discipline. → PR below.

## Phase 4 — Validate

**Tests added (16):**
- **crates/editor/src/delete.rs `mod tests` (9)** — the pure truth tables (the 37-mutant surface): word ops + motion boundaries + edge-None; the char-vs-byte separator (`"aébc"` WordRight→0..4 not byte-5); ToLineStart; **ToLineEnd + ⌃K-at-EOL-eats-`\n` + EOF→None** (the `e != c` / `c < len` discrimination); **WholeLine 3-way + round-trips + the C1-LOW empty→None**; delete_edits merge/disjoint/carry/None; D-SELECTION-WINS incl. backward drag; empty-buffer totality.
- **crates/editor/src/buffer.rs `mod tests` (1)** — `delete_forward_at_selections`: 1-char right, EOF no-op, selection-wins, multi-cursor.
- **crates/marley_app/src/headless_drive.rs (6)** — REQ-001/004/005 (each verb deletes the right range end-to-end); REQ-006 (N-cursor one-undo + carry; two-carets-same-line merge); REQ-008 (plain forward-delete via the KEY path); REQ-007 (plain backspace unchanged); REQ-009 (one ⌘Z + no group leak).

**REQ coverage:** REQ-001..009 all pinned (REQ-007's terminal-⌃K half is the keymap unit `delete_ops_resolve_and_ctrl_k_is_editor_only` — ⌃K-on-TERM→None + op_for_ctrl_key untouched). No UI/render change → no LIVE pixel to defer (all delete ops are buffer+selection STATE, fully headless-proven).

**Gate reds fixed at SOURCE (first `--diff` run, §0):**
- **gate:3 (tests)** — the pre-existing `apply_editor_key_multi_backspaces_at_every_cursor_and_ignores_the_rest` asserted `Key::DeleteForward` was inert; the #303 forward-delete arm made it EDIT. Updated the test: removed DeleteForward from the "inert" loop + added its positive behavior (`"abc"` caret 1 → `"ac"`). This one failing baseline test CASCADED into gate:4 (coverage — llvm-cov aborted) and gate:5 (mutation — baseline red); both went green once fixed.
- **gate:14 (docs)** — `DeleteOp`/`delete_range_for` doc-comments linked to `[`crate::movement`]`, a PRIVATE module → rustdoc `-D warnings` rejects a public item linking to a private one. Fixed: plain-text `` `movement` motions `` (the specific fns are re-exported; the module is not).

**Runs (actual):**
- `cargo nextest -p marley_editor -E 'test(/delete::/)'` → 9 passed; buffer forward-delete → 1 passed; the 6 drives → 6 passed.
- **Full `scripts/gates.sh --diff` (2nd run) → GATE GREEN [diff]**, 15/15: gate:4 coverage **100% lines**; gate:5 mutation **37 caught / 0 missed → MSI 100.0%** (delete.rs's ~34 + buffer's delete_forward 3; app.rs shims skipped). Receipt `84d775e9…` == live `gate_state_hash` (commit-valid).
- No #348 hang either run; no pre-existing failures beyond the one in-scope test the ticket's behavior change necessitated updating.

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.
