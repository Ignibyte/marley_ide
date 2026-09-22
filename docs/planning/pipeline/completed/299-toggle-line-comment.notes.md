# ⌘/ toggle line comment — Notes

- **Forge ticket:** #299 `3e5054ab-439f-43b3-b73f-4e43cba2ac2d`
- **AAR:** `5fa3e15f-fc17-45aa-837a-d2282669414c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-299-toggle-line-comment.md
- **Pipeline spec:** 299-toggle-line-comment.spec.md
- **Depends on:** #296 (`135b439`) · #297 (`edb62d4`) · #298 (`bc27c70`) — all SHIPPED + PUSHED

## Phase 1 — Plan

### What this ticket actually is
A line-prefix edit over the deduped union of rows touched by every cursor, in one undo unit. **Most of the
machinery already exists** — `indent.rs` ships `LineEdit`, `line_span`, `rebase_selections` for exactly this
shape (Tab/⇧Tab are the same kind of edit). The genuine deltas are:

1. the **decision** (toggle direction · min-indent column · blank skip · the uncomment strip),
2. the **union+dedupe** of `line_span` across N cursors (`line_span` handles only one selection),
3. a **`pub` accessor on the existing comment-token table**.

### THE PREMISE CORRECTION (the most valuable thing this phase produced)
The forge ticket says the token comes from "the file's tree-sitter language (crates/syntax already knows it)".
**It does not.** `marley_syntax::highlight_lines(src)` takes no language — tree-sitter is Rust-only and the app
gates it on `language_of(&path) == Language::Rust`. The editor's one language seam is
`code_syntax::language_of(path)`, and **the comment-token table already lives there**
(`lang_spec(lang).line_comment`), private.

Built as written, #299 would have shipped a SECOND source of truth for "what is a line comment in Rust" —
`PR-claude-two-engines-answering-the-same-question-will-diverge-001`, the rule earned on #298 by shipping two
match engines that silently disagreed about what an occurrence *is*. Caught at PLAN time by grepping for the
table before trusting the ticket. **Expose the existing one.**

Honest consequence, promoted to an acceptance criterion (REQ-006) rather than buried: **⌘/ works on `.rs` /
`.toml` / `.sh` and is a NO-OP on JSON / Markdown / Plain**, because those have no token in the table. Adding
languages is a `code_syntax` change that also changes highlighting → its own ticket.

### THE ROUND-TRIP HAZARD (found at plan time, like #298's case divergence)
`///` doc comments are everywhere here (39 in `indent.rs` alone), and `/// foo` **starts with** `//`. Under the
reference's prefix test it is "already commented", so uncommenting strips two chars → **`/ foo`** — mangled, and
not reversible (⌘/ again gives `// / foo`).

Worked it through by hand before writing the AC, and the common case is **safe**:
- Select a fn WITH its doc comments → the block is MIXED → **comment all** → `// /// foo` → ⌘/ → strip one
  layer → `/// foo` **restored byte-identical**. Doc comments survive the ordinary flow.
- It only bites when the selection is EXCLUSIVELY doc-comment lines.

VS Code and Zed both do exactly this, so §20 says match it. The trap is REQ-007: *"toggle then toggle again is
byte-identical"* is **simply false** as stated, and a naive test would encode that falsehood and then defend it
(`PR-claude-a-test-can-encode-a-bug-and-defend-it-001` — #297's flagship failure). So the guarantee is scoped
to an **uncommented starting state** (D7), and the `///` outcome gets its OWN pinning test (REQ-008) so it is a
recorded decision rather than an accident.

### Grounding (verified in-tree, not guessed)
- `LineEdit = (CharOffset, usize, String)` = `(at, remove_chars, text)`; ascending; applied BACK-TO-FRONT.
  Comment = `(at, 0, "// ")`; uncomment = `(at, n, "")`. Exactly the shape needed.
- `line_span(buffer, anchor, caret) -> (first, last)` already handles the subtlety that a selection ending at
  **column 0** of the next row does not include that row. Reuse per member; union + dedupe.
- `rebase_selections(set, &[LineEdit])` already carries N cursors through a line-prefix edit (#297).
- **⌘/ is FREE** — no collision (unlike #298's ⌘⇧L vs `split-right`, which cost a scoped shadow). Roster guard
  50 → 51 rows, 9 → 10 scoped.

### Risks
- **The min-indent × blank-line interaction** is the most likely place for a real bug: a blank (or
  whitespace-only) line must neither receive a bare token nor drag the column to 0. Inspect gets a lens on it.
- **The toggle-direction predicate** decides everything. A trailing `code(); // x`, a `///`, or a
  whitespace-only line must not flip it. D5 pins the rule (token = first non-whitespace).
- **`indent.rs` / `code_syntax.rs` are long untouched** → a green MSI says nothing about them, because the gate
  only mutates files IN THE DIFF (`PR-claude-an-equivalent-mutant-means-delete-the-redundancy-001`, earned on
  #298 where `word_range_at` had carried an unkillable mutant since #265). **Expect latent mutants; budget for
  fixing them at source.**

## Phase 2 — Design

### Architecture
A comment toggle is a **line-prefix edit** — structurally identical to Tab/⇧Tab, which `indent.rs` already
ships. So this is an assembly job over proven parts, plus one genuinely new decision.

**Tab's dispatch arm is the precedent, and it names this ticket in its own comment** (app.rs:6588):
> *"It acts on the PRIMARY cursor's line span: a block indent is a LINE-range op, and multi-cursor block indent
> is a **#299/#300-class feature**, not this ticket."*

Its shape is what ⌘/ copies exactly: read the set → build `Vec<LineEdit>` → **ONE** `begin/end_undo_group`
around **raw `buffer.edit()`** calls applied **BACK-TO-FRONT** → `rebase_selections` → `set_selection`.
**Never `edit_at_selections` here** — it self-brackets, and `begin_group` OVERWRITES an open group and
silently discards its records, so the whole toggle would be lost (the #296 D6 contract).

**§20 confirmed.** Reference = Zed / VS Code / JetBrains ⌘/. Matched behaviors: all-commented → uncomment, else
comment-all; markers at ONE column with relative indent preserved; blanks skipped; uncomment strips the token +
at most one space. Observed behavior only — no Zed or VS Code source read or translated.

### THE ROUND TRIP — verified by hand BEFORE any code (REQ-007)
```
start | '    foo();' ¶ '' ¶ '        bar();' ¶ '    baz();'
⌘/ #1 | '    // foo();' ¶ '' ¶ '    //     bar();' ¶ '    // baz();'
⌘/ #2 | '    foo();' ¶ '' ¶ '        bar();' ¶ '    baz();'
markers all at the SAME column? {4} → no staircase · blank untouched · ROUND TRIP byte-identical ✓
```
**This is why "at most ONE space" (D6) is load-bearing, not cosmetic.** The deep line becomes
`    //     bar();` — the token at column 4, ONE space, then `bar`'s own 4 spaces of relative indent.
Uncomment removes exactly `token + 1 space` = 3 chars → `        bar();`. **Strip all following whitespace
instead and the deep line silently loses its relative indent, and the round trip is no longer an inverse.**
It gets a dedicated test.

### The four open questions — RESOLVED
- **(a) Whitespace-only lines are BLANK** (`line.trim().is_empty()`, which catches `""` and `"    "`; note
  `Buffer::line_text` already strips the trailing `\n`). They get no marker and do **not** contribute to the
  min-indent — otherwise a stray 3-space line inside a 4-space block would drag every marker to column 3.
  **This predicate deliberately DIFFERS from `indent_edits`'s** (which skips only `.is_empty()`, so Tab *does*
  indent a whitespace-only line). Not an inconsistency: they answer different questions. Four extra invisible
  spaces on a blank-looking line are harmless; a dangling `// ` on one is not. Stated so a critic reads it as a
  decision.
- **(b) The min-indent is a leading-whitespace CHAR COUNT**, and the token is inserted at that char offset.
  Exactly right for any *consistent* indent style (all tabs → the min is a tab count; all spaces → a space
  count), and it degrades gracefully on a mixed-indent block — which is already broken text. Same char-offset
  model `indent_edits`/`line_start` already work in. No display-column arithmetic; no `tab_width`.
- **(c) Uncomment strips AT THE TOKEN'S OWN POSITION** (the first non-whitespace), not at a fixed column —
  which is what makes it an exact inverse of comment-at-min-indent, because commenting always leaves the token
  as the line's first non-whitespace char. Verified above.
- **(d) A NEW `comment.rs`; `touched_rows` goes in `indent.rs`.** `indent.rs` is *whitespace* ops; comment
  tokens are a different concern, and a separate module keeps the new coverage/mutant surface isolated.
  `touched_rows` is the N-cursor generalization of `line_span`, so it belongs beside it — and it is the seam a
  future Tab fix needs.

### Does `touched_rows` fix Tab's primary-only bug for free? NO — filing it.
`indent_edits(buffer, first, last, tab_width)` takes a **contiguous range**. Multi-cursor rows are a
**discontiguous set**, so fixing Tab means widening both `indent_edits` and `dedent_edits` to `&[usize]` — a
refactor of two shipped fns with their own tests and mutant sets, **plus a user-visible behavior change** (Tab
with three cursors would indent three blocks instead of one). That deserves its own ticket, its own inspect and
its own drive. **Not free → filed as a follow-up.** `comment_edits` is designed to take `rows: &[usize]`
precisely so the shape is already proven when that ticket lands.

### File manifest
| file | change |
|---|---|
| `crates/editor/src/comment.rs` | **NEW.** `comment_edits(buffer, rows: &[usize], token: &str) -> Vec<LineEdit>` — the whole decision. |
| `crates/editor/src/indent.rs` | ADD `touched_rows(buffer, set) -> Vec<usize>` — the deduped, ascending union of `line_span` over EVERY member (the N-cursor generalization of `line_span`, beside it). |
| `crates/editor/src/lib.rs` | `mod comment;` + re-export `comment_edits`, `touched_rows`. |
| `crates/marley_app/src/code_syntax.rs` | ADD `pub fn line_comment_for(lang: Language) -> Option<&'static str>` = `lang_spec(lang).and_then(\|s\| s.line_comment)` — **exposes the EXISTING table; `LangSpec`/`lang_spec` stay private.** ONE table, two features. |
| `crates/marley_app/src/keymap.rs` | NEW Editor-scoped ⌘/ row → `"toggle-comment"`. Roster guard **50→51**, scoped **9→10** (two assertions: `:875` and the scoped count at `:897`). |
| `crates/marley_app/src/app.rs` | The `"toggle-comment"` dispatch arm — **copies Tab's shape exactly**. |

### The seams
```rust
// indent.rs — the N-cursor generalization of line_span.
pub fn touched_rows(buffer: &Buffer, set: &SelectionSet) -> Vec<usize>
//   line_span() per MEMBER (never primary()! — the #298 template), union, dedupe, ascending.
//   Two cursors on one row → that row ONCE.

// comment.rs — the whole decision.
pub fn comment_edits(buffer: &Buffer, rows: &[usize], token: &str) -> Vec<LineEdit>
//   blank(row)      = line_text(row).trim().is_empty()                    (D4 — and NOT min-indent fodder)
//   commented(row)  = line_text(row).trim_start().starts_with(token)      (D5 — first non-whitespace)
//   token.is_empty() OR no live rows      → vec![]   (total; no panic)
//   ALL live rows commented → UNCOMMENT: at = line_start + leading_ws;
//                             remove = token.chars().count() + (1 if the next char is ' ')   (D6)
//   else                    → COMMENT:   col = MIN leading_ws over LIVE rows;                (D3)
//                             at = line_start + col; insert "{token} "
//   Edits ASCENDING in `at` (rows are ascending) — the LineEdit contract; the caller applies BACK-TO-FRONT.
```
`remove` counts CHARS (`token.chars().count()`, not `.len()`) — `LineEdit`'s second field is a char count.

**The empty-token guard is live, not dead code**: `comment_edits` is `pub`, so a test can pass `""` directly.
(Contrast #298, where an unreachable guard was the one uncovered line and had to be deleted.) Without it,
`"".starts_with("")` is true for every line → "all commented" → it would strip a leading space off every line.

### The dispatch arm (mirrors Tab, app.rs:6596-6622)
Resolve the token FIRST (an immutable borrow of `s` that ends before the `&mut` buffer borrow — E0502
otherwise), then:
`None` → **NO-OP, nothing touched** (REQ-006). `Some(token)` → `touched_rows` → `comment_edits` → one
`begin_undo_group(before)` → raw `buffer.edit()` **back-to-front** (`edits.iter().rev()`) → `rebase_selections`
→ `end_undo_group(after)` → `set_selection(after)`. An empty `edits` list opens then **drops** the group with
no undo step and no redo-clobber (undo::end_group, #297 inspect F1), so a no-op ⌘/ leaves the dirty flag alone.
No `follow_editor_caret` — the cursors do not change rows.

### RISK: the `/` key name is a genuine unknown
`binding_from_keystroke` passes gpui's raw `keystroke.key` straight through, and **there is no punctuation
binding in the keymap yet**, so nothing in-tree tells us whether gpui calls this key `"/"` or `"slash"`.
**I am not guessing it**: the headless `simulate_keystrokes("cmd-/")` test PROVES it end-to-end and fails
loudly at implement if the name is wrong. (Reading gpui's source to find out is unnecessary; the test is the
oracle.)

### Regression Test Plan

| # | Test | Where | Proves |
|---|---|---|---|
| T1 | Mixed block (one line already `// x`, one not) → COMMENT ALL; the commented line becomes `// // x` | `comment.rs` unit | REQ-001 |
| T2 | All-commented → UNCOMMENT, for BOTH `// x` and `//x` (no space) | `comment.rs` unit | REQ-002, D6 |
| T3 | Ragged block (4/8/4 indents) → every marker at column 4; **assert the exact strings**, incl. `    //     bar();` | `comment.rs` unit | REQ-003, D3 |
| T4 | **The relative-indent guard**: uncommenting `    //     bar();` yields `        bar();`, NOT `    bar();` — "at most ONE space" is what makes it an inverse | `comment.rs` unit | REQ-002/007 |
| T5 | A BLANK line and a WHITESPACE-ONLY (`"   "`) line inside a 4-space block: neither gets a marker, and neither drags the column below 4 | `comment.rs` unit | REQ-004, (a) |
| T6 | A trailing comment (`code(); // x`) does NOT count as commented → the block still COMMENTS | `comment.rs` unit | D5 |
| T7 | Empty `rows`, an all-blank block, and an EMPTY token → `vec![]` (no panic, no edits) | `comment.rs` unit | totality |
| T8 | `touched_rows`: two cursors on ONE row → that row ONCE; three carets on rows 1/9/20 → exactly [1,9,20]; a selection spanning rows 3–7 → [3..=7]; a selection ending at **column 0** of row 8 does NOT include row 8 (`line_span`'s rule, exercised through the union) | `indent.rs` unit | REQ-005 |
| T9 | **The ROUND TRIP**: comment→uncomment over a ragged + blank + whitespace-only fixture is BYTE-IDENTICAL | `comment.rs` unit | REQ-007 |
| T10 | **The `///` DECISION** (pinned, not accidental): a block of only `/// doc` lines is treated as commented and uncomments to `/ doc`; a block MIXING `/// doc` with code comments-all and round-trips the doc lines back intact | `comment.rs` unit | REQ-008 |
| T11 | `line_comment_for`: Rust→`//`, Toml→`#`, Shell→`#`, **Json→None, Markdown→None, Plain→None** | `code_syntax.rs` unit | REQ-006 |
| T12 | Keymap resolution: ⌘/ → `"toggle-comment"` on Editor; **None** on Terminal / cockpit / no-context (it is a new row, not a shadow). Roster 51 rows / 10 scoped. | `keymap.rs` unit | shim |
| T13 | **Headless**: open a `.rs` fixture, select a ragged block, ⌘/ → commented at one column; ⌘/ → byte-identical; **ONE ⌘Z reverts the whole toggle**; cursors survive. **Reads values → `reap_sessions` → THEN asserts** (a panic before reap HANGS on the PTY drop chain). | `headless_drive.rs` | REQ-005/007 + the `/` key name |
| T14 | **Headless**: a `.json` fixture → ⌘/ → buffer byte-identical AND the dirty flag unmoved | `headless_drive.rs` | REQ-006 |
| T15 | The TRACED `cargo mutants --list -f` kill set for `comment.rs` + `indent.rs` + `code_syntax.rs` — **never guessed operators**. Expect **latent** mutants in the long-untouched `indent.rs`/`code_syntax.rs`: a green MSI only covers files IN THE DIFF (`word_range_at` carried an unkillable mutant from #265 to #298 for exactly this reason). Fix any at SOURCE (§0) — an equivalent mutant means DELETE the redundancy. | gate | REQ-009 |
| — | **THE LIVE DRIVE** (validate): ragged block → ⌘/ → one column, no staircase → ⌘/ → byte-identical → ⌘Z; then ⌘⌥↓ ×2 → ⌘/ → every cursor's row toggles at once. Capture and READ the pixels. **Never put `focus` between a gesture and its assertion** — it CLICKS, and a click collapses the cursor set by design. | drive | all |

Uncoverable: none. Every seam is pure and reachable; the shim is covered headlessly + on pixels.

### Risks
- **The min-indent × blank interaction** is the likeliest real bug — a whitespace-only line must neither get a
  marker nor drag the column. T5 pins it; inspect gets a lens on it.
- **The toggle-direction predicate** decides everything. A trailing comment, a `///`, or a whitespace-only line
  must not flip it. D5 + T6 + T10.
- **`indent.rs` / `code_syntax.rs` are long untouched** → expect latent mutants the gate has never seen.

## Phase 3 — Implement

Built to the manifest, sequenced so the tree compiled at each step. **Compiled clean first try; 1100 tests
pass; clippy `-D warnings` clean.**

### `crates/editor/src/indent.rs` — `touched_rows` (the N-cursor union)
`line_span` per MEMBER (`set.selections().iter()` — **never `primary()`**, the #298 template), flat-mapped into
`first..=last`, then `sort_unstable` + `dedup`. Two cursors on one row yield that row ONCE — a duplicate would
emit two edits at one offset and double the prefix. Each member keeps `line_span`'s "a selection ending at
column 0 of the next row does not touch that row" convention for free. Documented as the seam forge **#307**
needs to make Tab multi-cursor aware.

### `crates/editor/src/comment.rs` (NEW) — the whole decision
`comment_edits(buffer, rows, token) -> Vec<LineEdit>`. Guards first (empty token, no live rows → `vec![]`),
then the direction (`all_commented` over the LIVE rows), then either the uncomment strip (at the token's own
position) or the comment insert (at the MIN indent of the live rows).

Two private helpers carry the load-bearing decisions, each documented where it lives:
- `is_blank` = `line.trim().is_empty()` — catches `""` AND `"    "`. **Deliberately a different predicate from
  `indent_edits`'s** `.is_empty()` (so Tab *does* indent a whitespace-only line). Not an inconsistency: extra
  invisible spaces are harmless; a dangling `// ` is visible litter, and a stray 3-space line would drag every
  marker to column 3.
- `indent_chars` = the leading-whitespace CHAR count. A char count, not a display column — correct for any
  consistent indent style, graceful on a mixed one. No `tab_width` anywhere.

`remove` counts CHARS (`token.chars().count()`), not bytes — `LineEdit`'s second field is a char count.

The **"at most ONE space"** rule is documented in the fn as what makes uncomment an EXACT INVERSE of
comment-at-min-indent, with the worked example: `    //     bar();` → strip 3 chars → `        bar();`. Strip
all following whitespace instead and the deep row loses its relative indent.

The **empty-token guard is live code, not dead**: `comment_edits` is `pub`, so a test passes `""` directly.
(Contrast #298, where an unreachable guard was the single uncovered line and had to be deleted.) Without it
`"".starts_with("")` is true for every row → "all commented" → it would strip a leading space off every line.

### `crates/marley_app/src/code_syntax.rs` — expose the EXISTING table
`pub fn line_comment_for(lang) -> Option<&'static str>` = `lang_spec(lang).and_then(|s| s.line_comment)`.
**`LangSpec` and `lang_spec` stay private — only the token escapes.** ONE table, two features: the ⌘/ toggle
and the syntax colors can never disagree about what a line comment IS.

### `crates/marley_app` — the shim
- **keymap.rs**: a NEW Editor-scoped ⌘/ row → `"toggle-comment"`. Unlike ⌘D and ⌘⇧L it **shadows nothing** —
  the chord was free — so it costs nothing anywhere else. Both roster guards updated: `chords.len()` 50 → **51**
  and the scoped count 9 → **10**, plus a `contains` assertion for the new chord. **They passed, which is the
  proof the counts are right rather than merely bumped.**
- **app.rs**: the `"toggle-comment"` arm, copying Tab's shape exactly — token resolved FIRST (the immutable
  borrow of `s` must end before `active_buffer_mut()`, E0502 otherwise) → `None` = touch nothing → else ONE
  `begin_undo_group` around RAW `buffer.edit()` calls applied **BACK-TO-FRONT** → `rebase_selections` →
  `end_undo_group` → `set_selection`. **No `edit_at_selections` here**: it self-brackets, and `begin_group`
  overwrites an open group and discards its records, so the whole toggle would be lost (#296 D6). An empty edit
  list opens then DROPS the group — no undo step, no redo-clobber, no version bump — so a no-op ⌘/ leaves the
  file clean. No `follow_editor_caret`: the toggle never moves a cursor to a different row.

### Deviations from design
None.

### Still unproven (by design)
**The `/` key name.** `binding_from_keystroke` passes gpui's raw `keystroke.key` through and there is no other
punctuation binding in the keymap, so nothing in-tree says whether gpui calls it `"/"` or `"slash"`. I used
`"/"`. Phase 4's headless `simulate_keystrokes("cmd-/")` is the oracle and will fail loudly if that is wrong —
which is the right way to settle it, rather than reading gpui/Zed source (§20).

## Inspect (Phase 3.5)

**ONE HIGH that silently destroys the user's code, plus a MEDIUM class that irreversibly mangles the first line
of every file in this repository.** Four critics, each isolated to its own scratch file, each required to
REPRODUCE rather than reason. 1100 tests, clippy clean, compiled first try — none of it evidence.

**Three critics found the HIGH independently. The fourth read the same code and declared it CLEAN**, having
misread the guard as testing `old.inserted` when it tests `new.inserted`. That is the case for running four.

### F1 [HIGH] — ⌘/ at N cursors, type one character, press ⌘Z → **the buffer is corrupted, unrecoverably**
`crates/editor/src/undo.rs:143` (`coalesces_into`), reached for the first time via the new toggle arm.

```
"foo\nbar\n" · two carets MID-LINE · ⌘/ → "// foo\n// bar\n" · type X → "// foXo\n// baXr\n"
                                                        ⌘Z → "oXo\naXr\n"     ← the f and b are GONE
```
The undo stack is then **empty** — a second ⌘Z does nothing. The text cannot be recovered.

`end_group` offers every closing group to `coalesces_into`, whose four conditions the toggle satisfies exactly:
bracketed ✓, record count == cursor count (N rows, N cursors) ✓, `prev.sel_after == group.sel_before` (⌘/ moves
nothing) ✓, every record a pure insert with a 1-char *new* insert ✓. So the typed char is appended onto the
toggle's record — but the toggle's records are anchored at the **min-indent column**, not at the cursors, so the
char is filed at an offset it never occupied and `undo` deletes `at..at + inserted.len()`: the wrong characters.

The false premise is stated **verbatim** in the fn's own doc:
> *"Because the cursors did not move, each new insert lands precisely at the end of its own record's text, so
> appending char-wise is contiguous by construction — no offset arithmetic is needed or wanted."*

True for a typed run. False for the first bracketed group in the codebase whose records are not cursor-anchored.
`record()`, its single-cursor twin, **checks** contiguity (`last.at + last.inserted.chars().count() == rec.at`);
`coalesces_into` only **argued** it. Enter is affected identically (one char, `"\n"`).

**VERDICT: REAL.** Reproduced by three critics and by me, independently.

**FIX — state the intent instead of inferring it.** `UndoGroup` gains `cursor_anchored: bool`, set `true` ONLY
by the N-cursor typed-insert path (`edit_ranges_restoring`) and `false` by every hand-bracketed group
(`begin_undo_group` — Tab/⇧Tab, ⌘/, replace-all). `coalesces_into` now requires it. Chosen over the surgical
alternative (restoring a contiguity check with cross-coordinate-space arithmetic — which two critics verified by
hand and which does work) because **inferring intent from geometry is precisely what failed**, and the tag kills
the whole class: every future bracketed line op (#307's multi-cursor Tab, #300's line-move, #303's delete-word)
would otherwise walk into it. #297's typed-run coalescing — the entire reason the guard exists — is preserved
and pinned by a test.

### F2 [MEDIUM] — the token-prefix mangling class: I hunted it for `//` and never looked at `#`
`crates/editor/src/comment.rs` (`is_commented`)

Plan found that `/// foo` → `/ foo` and recorded it as D7/REQ-008, matching the reference. **It examined only
the token it happened to think about.** `#` — the token for TWO of the three supported languages — destroys
more, and worse:

| line | naive ⌘/ | recoverable? |
|---|---|---|
| `//! PURE — …` — **every file in this crate opens with one** | `! PURE — …` | no |
| `#!/bin/sh` — the script silently stops executing, failing at EXEC time | `!/bin/sh` | no |
| `/// doc` | `/ doc` | no |
| `//////` · `####` | one token shorter per press → `""`, an ABSORBING state | no |
| `## Section` | `# Section` → `Section` | no |

And **D7's rationale was FALSE** — "the toggle NORMALIZES" was asserted as a general property; it holds only
when the text after the token doesn't itself start with the token. A test written to that rationale would have
encoded the falsehood and defended it.

**VERDICT: REAL.** Raised independently by two critics; both graded it against the `///` decision I had already
blessed, which is what made the asymmetry visible.

**FIX:** the token must not be followed by MORE OF ITSELF, nor by `!` — those are richer markers, so ⌘/ COMMENTS
them (`//! x` → `// //! x` → back) rather than shredding them. A deliberate divergence from the reference (§0 —
do not ship known irreversible harm). All five cases now round-trip byte-identically; plain `// x`/`//x`/`# x`
uncomment exactly as before; the mixed doc+code block still restores intact. D7 and REQ-008 rewritten.

### F3 [LOW] — `comment_edits` documented an ASCENDING postcondition it did not enforce
An unsorted `rows` emits edits out of order (the caller applies them BACK-TO-FRONT → corruption); a duplicated
row prefixes a line twice. Unreachable today (the sole caller passes the sorted, deduped `touched_rows`) — but
it is a `pub` cross-crate fn and #307 adds a second caller. **VERDICT: REAL (latent).** FIX: sort + dedup
inside, making the documented contract true.

### F4 [LOW] — a mixed tab/space block staircases; ACCEPTED, and it exposed a hole in my own fuzz
`min(indent_chars)` is a CHAR count, so a tab-indented row and a 4-space row put their markers in different
DISPLAY columns. Already documented as accepted ("graceful on a mixed-indent block, which is already broken
text"); it round-trips and never panics. **The sharp part of the finding:** my own no-staircase fuzz assert
**cannot see it** — `l.find(token)` is a byte index, not a display column. Recorded rather than silently trusted.

### F5 [LOW] — the arm's `clear_marked()` is dead
The dispatcher already calls it for every keymap-resolved chord, and `toggle-comment` has no palette path.
**VERDICT: REAL but ACCEPTED** — it is one line and mirrors the #297/#298/`select-all` arms exactly; removing it
from this arm alone would be worse. A sweep of all four belongs in its own change.

### RULED OUT by execution (not assumed)
- **The `/` key name — RESOLVED, `"/"` is CORRECT.** A real headless `simulate_keystrokes("cmd-/")` fired the
  toggle and toggled back; and gpui names the key `"/"` on all three platforms. The negative half was proven
  too: `"slash"` resolves to `None`, so the test was not vacuous. **Not a dead chord.**
- **CRLF — clean, with a proof of WHY**: a `\r` on a live row is always AFTER a non-whitespace char, and a row
  of pure whitespace is `is_blank` by definition (because `trim()` and `is_whitespace()` are the same predicate),
  so the token can never land between the `\r` and the `\n`. Round trip byte-identical on a CRLF fixture.
- **Roster counts HONEST** — enumerated from source: 51 rows, 10 scoped, **0** same-context duplicates.
- **REQ-006 no-op is real** — headless on `.json` AND `.md`: text, `version()` and the dirty flag all identical
  before/after. An empty edit list opens then DROPS the group without clearing redo.
- **`line_comment_for`** correct for all 6 `Language` variants + 20 extensions; `.sh`/`.bash`/`.zsh` all → `#`.
- **`touched_rows`** — dedup, union, the column-0 carve, overlapping/nested selections, rows past EOF: clean.
- **Min-indent × blank, the `at` arithmetic, Unicode/NBSP/CJK, the uncomment rebase, the SelectionSet invariant,
  `active_file()`'s language, no-editor-focused** — all executed, all clean.
- **My own round-trip fuzz** (3,324 randomized blocks: ragged/tab/blank/whitespace-only rows, discontiguous row
  SUBSETS, 4 tokens) found no break, and pins that an UNSELECTED row is never modified.

### Forge capture
- **Failures:** `BF-claude-undo-group-absorbed-a-typed-char-and-destroyed-text-001` (CRITICAL),
  `BF-claude-hunted-the-hazard-for-one-token-and-never-checked-the-other-001` (HIGH).
- **Prevention rules:** `PR-claude-a-guard-must-check-its-precondition-not-argue-it-001` (CRITICAL),
  `PR-claude-when-you-find-a-hazard-enumerate-every-value-that-reaches-it-001` (HIGH).

### State at Phase 3.5 exit
1105 tests pass · clippy `-D warnings` clean · rustdoc clean · no scratch files · tree verified against the
pre-inspect snapshot (only `comment.rs` differs — that is the F2 fix; **no critic clobbered the diff**).

Phase 4 owes: the traced `cargo mutants --list` kill set (never guessed operators — and `undo.rs`/`buffer.rs`
are NEW to the diff now, so expect latent mutants there); coverage/MSI 100 on `comment.rs` + `touched_rows` +
`is_commented` + the `cursor_anchored` guard; the round-trip fuzz promoted to a permanent test; and the LIVE
DRIVE. **The F1 regression test is the highest-value test in this ticket** — ⌘/ at 2 cursors → type → ⌘Z must
revert ONLY the char.

## Phase 4 — Validate

**GATE GREEN [diff] — 15/15.** 1122 tests · coverage 100% lines · MSI 100% · clippy · rustdoc · miri · visual.
Proven on live pixels.

### The traced mutant set (never guessed)
`cargo mutants --list -f <file>` on all four editor files, then killed exactly what it printed.

| file | mutants | caught | unviable | **missed** |
|---|---|---|---|---|
| `comment.rs` | 20 | 20 | 0 | **0** |
| `indent.rs` | 53 | 52 | 1 | **0** |
| `undo.rs` | 34 | 32 | 2 | **0** |
| `buffer.rs` | 65 | 52 | 13 | **0** |

`undo.rs` and `buffer.rs` were long untouched and newly in the diff — exactly where #298 found a mutant latent
since #265 — so I expected latent survivors. **There were none.** Two of `comment.rs`'s mutants sit precisely on
the guards inspect added (`delete match arm Some('!')` and `replace match guard token.starts_with(c)`), and both
die to the richer-marker round-trip test. In `undo.rs`, `178:13`/`179:13` (`&&` → `||`) make the new
`cursor_anchored` guard permissive and **restore the data-corrupting bug** — the F1 regression kills both.

**Note the mutation blind spot, stated rather than glossed:** cargo-mutants does not mutate function-call
ARGUMENTS, so the `false`/`true` literals passed to `begin_group` are invisible to it. Both directions are
therefore pinned by BEHAVIOR tests instead: F1 fails if a hand-bracketed group is wrongly marked
cursor-anchored, and F1b fails if the typed-insert path is not.

### Tests added
- **F1 — the highest-value test in the ticket** (`buffer.rs`): two carets MID-LINE → a line-anchored group →
  type one char → **ONE ⌘Z reverts only the char**, and a SECOND reverts the toggle. Against the pre-inspect
  code this produced `"oXo\naXr\n"` — the user's letters deleted, undo stack drained.
- **F1b**: #297's typed-run coalescing still merges three chars at two cursors into ONE undo. The tag must seal
  the hand-bracketed group *without* over-sealing the typed one; a naive fix breaks this half.
- **REQ-008 (rewritten)**: all six richer markers round-trip byte-identically — `//!` (module doc — the first
  line of every file in this crate), `///`, `//////`, `#!/bin/sh`, `## Section`, `####` — and the guard does
  NOT over-fire (`// x`, `//x`, `# x` still uncomment; a trailing comment still comments; a mixed doc+code
  block restores the doc row intact).
- **REQ-001..007**: mixed → comment-all; all-commented → uncomment (both spacings); ragged 4/8/4 → every marker
  in ONE column (exact strings); **the relative-indent guard** (uncommenting `    //     bar();` gives
  `        bar();`, not `    bar();` — "at most ONE space" is what makes it an inverse); blank AND
  whitespace-only rows get no marker and don't drag the column; zero-indent; degenerate inputs → `vec![]`.
- **F3**: an unsorted/duplicated `rows` still yields ascending, deduped edits.
- **`touched_rows`**: two cursors on one row → that row ONCE; a multi-row selection covers all its rows (kills
  the `!sel.is_caret()` negation, which would collapse a range to its head's row); `line_span`'s column-0 carve
  survives the union and a real cursor there re-includes it; last row, empty buffer.
- **The round-trip FUZZ, promoted from the inspect probe** — 4,000 randomized blocks (ragged/tab/blank/
  whitespace-only rows, DISCONTIGUOUS row subsets, four tokens). Asserts round-trip byte-identity, no-staircase,
  and that **an unselected row is never modified**. Its known hole is stated in the test, not papered over: the
  no-staircase assert uses `find(token)`, a BYTE index, so it cannot see the mixed tab/space display-column
  staircase (accepted and documented on `indent_chars`).
- **Keymap**: ⌘/ → `toggle-comment` on Editor, **None** on Terminal/cockpit/no-context (it shadows nothing) —
  plus the negative half: `"slash"` resolves to None, so the row would be DEAD had gpui named it that.
- **Headless** — the `/` key ORACLE: a real `simulate_keystrokes("cmd-/")` on a booted window fired the toggle
  and toggled back. Every unit test builds the chord by hand and would pass either way; only this proves the row
  is live. Plus REQ-006: on a `.json` file, text, buffer `version()` and the dirty flag are ALL unchanged.

### THE LIVE DRIVE — proven on pixels
Drove the already-open `find.rs` (the ⌘P finder does not commit its row — same as #298; did not burn time on
it). Clicked ONCE into the text to focus, then **no `focus` verb** — it CLICKS, and a click collapses the cursor
set by design.

Rows 33–35 of `find.rs` are a natural ragged block: `if !on_word…` (4 spaces), `return None;` (**8**), `}` (4).

| capture | what the pixels showed |
|---|---|
| `03-commented.png` | ⌘⌥↓ ×2 → **three cursors** → ⌘/ → `····// if …` · `····//     return None;` · `····// }` — all three markers at the SAME x (the column where the 4-space indent ends), though row 34 is indented EIGHT. **No staircase**, and row 34's `return` is pushed right by its own surviving relative indent. |
| `04-uncommented.png` | ⌘/ again → **byte-identical**; `return None;` back at 8 spaces, deeper than the `if`. |
| `05-typed.png` | Typing at the three cursors lands MID-LINE (`!on_worZd`, `returnZ`, `}Z`) — the exact shape that used to corrupt. |
| `06-one-undo.png` | **THE F1 FIX** — ONE ⌘Z removes only the `Z`s. The toggle survives and the code is whole (`!on_word`, `return`, `}`). Before the fix this keypress deleted four characters from each line's token column. |

`find.rs` on disk is untouched (⌘S was never pressed; `git diff` shows no change to it).

### Pre-existing / accepted
- The mixed tab/space display-column staircase (documented on `indent_chars`; round-trips, never panics).
- The arm's `clear_marked()` is dead (the dispatcher already does it) — kept for consistency with the three
  sibling arms; a sweep of all four belongs in its own change.
