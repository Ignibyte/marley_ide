---
pipeline_id: 1ec27928-39ef-4a63-bf29-7fa68fe44f26
ticket: forge#299 (3e5054ab-439f-43b3-b73f-4e43cba2ac2d) · local docs/planning/tickets/open/TICKET-299-toggle-line-comment.md
aar_id: 5fa3e15f-fc17-45aa-837a-d2282669414c
status: Phase 5 — Complete PASS
title: ⌘/ toggles line comments — language-aware, indent-aligned, over every cursor, in one undo unit
type: feature
milestone: M19
references: [docs/planning/pipeline/completed/298-cmd-d-adds-a-cursor.spec.md, docs/planning/pipeline/completed/297-multi-cursor-gestures-and-shim.spec.md]
---

## Title
**⌘/ — toggle line comments.** Arguably the most-pressed editing chord after save, and Marley does not have it.
It comments (or uncomments) every line touched by every cursor, in ONE undo unit, with the markers **aligned at
one column** rather than staircased down each line's own indent.

## Scope

### In
- **`comment_edits(buffer, rows, token) -> Vec<LineEdit>`** (pure, `crates/editor`) — the whole decision: the
  toggle direction, the minimum-indent column, the blank-line skip, and the uncomment strip. Emits the SAME
  `LineEdit` type Tab/⇧Tab already use.
- **`touched_rows(buffer, set) -> Vec<usize>`** (pure) — the DEDUPED UNION of rows touched by every cursor.
- **A `pub` accessor on the EXISTING comment-token table** in `code_syntax.rs` (see the correction below).
- **Keymap**: a NEW Editor-scoped ⌘/ row + the dispatch arm. Applies through the shipped multi-edit path in
  ONE undo group, and carries the cursors through with `indent::rebase_selections`.

### Out (explicitly deferred)
- **Block comments** (`/* … */`) — a different gesture (⌥⇧A), a different decision.
- **Adding new LANGUAGES to the table.** The table is shared with the highlighter; adding Python/JS/Go is a
  `code_syntax` change that also changes highlighting, and belongs in its own ticket. #299 ships ⌘/ for the
  languages the table already knows.
- Comment-aware auto-indent; comment re-flow.

## THE TICKET'S OWN PREMISE IS WRONG — corrected here
The forge ticket says the token is *"derived from the file's tree-sitter language (crates/syntax already knows
it)"*. **It does not.** Verified in-tree:

- `marley_syntax::highlight_lines(src: &str)` takes **no language argument**. Tree-sitter is **Rust-only**, and
  `app.rs:5630` gates it on `language_of(&path) == Language::Rust`.
- The editor's **one** language seam is `code_syntax::language_of(path)` (app.rs:3069, :3161, :3439, :5630).
- **The comment-token table already exists there** — `lang_spec(lang).line_comment`: Rust→`//`, Toml→`#`,
  Shell→`#`, **Json→`None`**, **Markdown/Plain→no `LangSpec` at all**. It is private (`fn lang_spec`,
  `struct LangSpec`, private field).

Building the ticket as written would ship a **second source of truth** for "what is a line comment in Rust" —
exactly `PR-claude-two-engines-answering-the-same-question-will-diverge-001`, the rule #298 earned by shipping
two match engines that silently disagreed. **Expose the existing table; do not clone it.**

The honest consequence, stated as an acceptance criterion rather than buried: **⌘/ works on `.rs` / `.toml` /
`.sh|.bash|.zsh` and is a NO-OP on JSON, Markdown and Plain.** Never mangle text we cannot comment.

## Reference (§20)
**Zed / VS Code / JetBrains ⌘/** — *toggle line comment*: if every touched non-blank line is already commented,
uncomment; otherwise comment them all, inserting the token at a single aligned column and preserving each
line's relative indent. Blank lines are skipped. Marley matches the observed BEHAVIOR on its own `SelectionSet`
+ the shipped `indent.rs` line-edit machinery. Clean-room §20 — observed behavior only; no Zed or VS Code
source read or translated.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — the token comes from the EXISTING `code_syntax` table.** Expose it (`line_comment_for(lang)`) reading
  the same `lang_spec` the highlighter reads. One table, two features. Rejected: a new table in `crates/editor`
  (a second source of truth), and threading tree-sitter (which does not know the language).
- **D2 — the pure seam is language-AGNOSTIC.** It takes the token as `&str`. The editor crate has no business
  knowing about file extensions; the app resolves `Language` → token and passes it down. This is also what
  keeps the table single.
- **D3 — markers go at the MINIMUM indent of the touched NON-BLANK lines.** Not each line's own indent, which
  staircases the markers down the block and looks broken. Each line's *relative* indent is preserved (the token
  is inserted at the min column, so a deeper line keeps its extra spaces after the token).
- **D4 — BLANK lines are skipped** (never padded with a bare token) **and must not drag the min-indent to 0.**
  A blank line inside a 4-space block would otherwise pull every marker to column 0.
- **D5 — "already commented" = the token is the FIRST NON-WHITESPACE on the line.** So `code(); // trailing` is
  NOT a commented line — a trailing comment must not flip the toggle's direction.
- **D6 — uncomment strips the token + AT MOST ONE following space.** `// x` → `x`, and `//x` → `x`.

## THE ROUND-TRIP HAZARD — hunted at plan time for `//`, and INSPECT proved I had only found HALF of it
Plan spotted that `/// foo` starts with `//`, so the reference's naive prefix test (`trim_start().starts_with(
token)`) calls it "already commented" and uncomments it to `/ foo` — mangled, and not reversible. Plan recorded
that as D7 + REQ-008 and moved on, matching the reference.

**Inspect showed the class is far bigger, and that I had examined only the token I happened to think about.**
`#` — the token for TWO of the three supported languages (Shell, TOML) — was never looked at:

| line | naive ⌘/ gives | second ⌘/ | recoverable? |
|---|---|---|---|
| `//! PURE — …` — a Rust MODULE DOC; **every file in this crate opens with one** | `! PURE — …` | `// ! PURE — …` | **no** |
| `#!/bin/sh` — a SHEBANG; the script silently stops being executable, failing at EXEC time | `!/bin/sh` | `# !/bin/sh` | **no** |
| `/// doc` | `/ doc` | `// / doc` | **no** |
| `//////` · `####` — a banner rule | one token shorter EACH press, down to `""` — and a blank row is an ABSORBING state (blanks are skipped, so it never comes back) | | **no** |
| `## Section` | `# Section` | `Section` | **no** |

**D7's written rationale was FALSE.** It claimed *"from an already-commented start the toggle NORMALIZES
(`//x` → `x` → `// x`), which is correct behavior, not a bug"* — asserted as a general property. It is not one:
it holds only when the text *after* the token does not itself start with the token. Written as a test, that
rationale would have encoded a falsehood and then defended it (`PR-claude-a-test-can-encode-a-bug-and-defend-it-001`
— the exact failure #297 earned).

### D7 (SUPERSEDED) — the toggle must never SHRED a marker it cannot restore
**The token must not be followed by MORE OF ITSELF, nor by `!`.** Those are RICHER markers — doc comments,
shebangs, banner rules — not plain comments. ⌘/ therefore *comments* them (`//! x` → `// //! x` → back to
`//! x`) instead of stripping them. Every row in the table above becomes a clean two-cycle, and a plain
`// x` / `//x` / `# x` still uncomments exactly as before.

This is a **deliberate divergence from the reference** (§0 — do not ship known, irreversible harm). VS Code and
Zed both shred these; that is not a behavior anyone wants, and here it would mangle the first line of every file
in this repository. The round-trip guarantee is now unconditional: **comment → uncomment is byte-identical, and
so is uncomment → comment.**

## Multi-cursor — REUSE the #298 template, do not re-derive it
The op is over the **UNION** of rows touched by EVERY cursor (a selection spanning rows 3–7 touches 5 rows;
three carets on rows 1/9/20 touch 3). **Deduped** — two cursors on the same row comment it ONCE.

> **ASK THE SET, TAKE OPERANDS BY KIND — NEVER BY POSITION.** `primary()` is merely member 0 (the TOPMOST) and
> `last()` merely the bottom-most, and since #297's ⌘-click **either can be a bare caret**. Any "is there a
> selection?" / "which rows?" question must be asked of the WHOLE SET. That exact substitution produced FOUR
> HIGH bugs on #298.

`indent::line_span(buffer, anchor, caret)` already computes ONE selection's row span — **including** the
subtlety that a selection ending at column 0 of the next row does not include that row. Reuse it per member and
union the results; that is the genuine delta.

## Grounding (verified in-tree, not guessed)
- `crates/editor/src/indent.rs` already has: `LineEdit = (CharOffset, usize, String)` — `(at, remove_chars,
  text)`, ascending, applied BACK-TO-FRONT; `line_span`; `indent_edits`/`dedent_edits`; `rebase_through`;
  `rebase_selections(set, &[LineEdit])` (#297 — Tab/⇧Tab CARRY the cursors through a line-prefix edit).
  **A comment toggle IS a line-prefix edit.** Comment = `(at, 0, "// ")`; uncomment = `(at, n, "")`.
- **⌘/ is FREE** — no keymap collision (unlike #298's ⌘⇧L vs `split-right`). A new Editor-scoped row.
  The roster guard (`keymap.rs:875`) asserts `chords.len() == 50` / 9 scoped → becomes **51 / 10**.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the touched block is MIXED (some lines commented, some not), the system shall COMMENT ALL of them. | Pure unit + **live drive**. |
| REQ-002 | WHEN every touched non-blank line is ALREADY commented, the system shall UNCOMMENT them — stripping the token and at most ONE following space (`// x` → `x`; `//x` → `x`). | Pure unit (both spacings). |
| REQ-003 | WHEN the touched lines have RAGGED indents, the system shall insert every marker at the MINIMUM indent — the same column on every line, with each line's relative indent preserved. | Pure unit + **live drive** (read the pixels: no staircase). |
| REQ-004 | BLANK lines shall be SKIPPED (never given a bare token), and a blank line shall NOT drag the minimum indent to 0. | Pure unit — a 4-space block with a blank line in it still comments at column 4. |
| REQ-005 | The op shall cover the DEDUPED UNION of rows touched by EVERY cursor, apply in ONE undo unit, and leave every cursor alive and on its text. | Pure unit (2 cursors on one row → ONE toggle) + headless (⌘Z reverts the whole toggle in one step) + **live drive**. |
| REQ-006 | WHEN the file's language has no line-comment token (JSON / Markdown / Plain), ⌘/ shall be a NO-OP — the buffer is byte-identical and the dirty flag does not move. | Pure unit + headless. **Never mangle text we cannot comment.** |
| REQ-007 | Comment-then-uncomment shall return the buffer BYTE-IDENTICAL — unconditionally, now that D7 stops the toggle shredding markers it cannot restore. | Pure round-trip unit + a randomized fuzz over ragged/blank/tab/multi-token blocks + **live drive**. |
| REQ-008 | A row carrying a RICHER marker — `//!` (module doc), `///` (doc), `#!` (shebang), `//////`/`####` (banner), `## …` — shall be COMMENTED, not shredded, and shall round-trip BYTE-IDENTICALLY. The naive prefix test destroys all five irreversibly. | Pure unit over all five, both tokens. |
| REQ-009 | The new pure seams shall be at 100% line coverage and MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — the two pure seams' signatures; how the token reaches them (D1/D2); the exact `LineEdit`
  shapes for comment vs uncomment; the union/dedupe of `line_span` per member; the keymap row + the roster-guard
  updates (50→51, 9→10 scoped); the test plan. **Confirm D7 and REQ-008 with the real diff in hand.**
- **P3 Implement** — the pure seams first, then the shim.
- **P3.5 Inspect** — adversarial critics on the diff. **Spawn them and WAIT.** Four critics have found real HIGH
  bugs on each of the last three tickets (#298: FIVE HIGH, including the flagship gesture permanently dying one
  occurrence short — with the gate fully green). Lenses: the min-indent + blank-line interaction (can a blank or
  a whitespace-only line drag the column to 0, or get a bare token?); the toggle-direction predicate (does a
  trailing comment, a `///`, or a whitespace-only line flip it?); the multi-cursor union (overlapping selections,
  two cursors on one row, a selection ending at column 0); the round trip (is comment/uncomment REALLY an
  inverse?); and the no-token languages. **Snapshot the tree before spawning; each critic gets its OWN scratch
  test file, never `src/`** (`PR-claude-parallel-critics-must-not-share-a-working-tree-001` — on #298 a critic
  reported two findings as false PASSES because it compiled against another critic's transient patch).
- **P4 Validate** — units + the TRACED `cargo mutants --list` kill set (never guessed operators; expect latent
  mutants in the long-untouched `indent.rs` / `code_syntax.rs`, since a green MSI only covers files IN THE DIFF)
  + the gate; then the **LIVE DRIVE**: a ragged-indent block → ⌘/ → markers at one column → ⌘/ → byte-identical
  → ⌘Z; then ⌘⌥↓ ×2 → ⌘/ → every cursor's row toggles at once. Capture and READ the pixels.
- **P5 Complete** — CHANGELOG + editor.md, AAR, close #299.
