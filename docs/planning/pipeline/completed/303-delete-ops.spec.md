---
pipeline_id: ee3a0377-591c-4e9a-9866-e377f7b493c8
ticket: forge#303 (38691b71-83d9-4a0d-acb2-e40ea9a4bc34) · local docs/planning/tickets/open/TICKET-303-delete-ops.md
aar_id: ca13a3bd-f9d0-4224-9008-0a44bfb77d63
status: Phase 5 — Complete PASS
title: The missing destructive ops — delete word (⌥⌫/⌥⌦), to line start/end (⌘⌫/⌃K), delete line (⌘⇧K)
type: feature
milestone: M19
references: [THE TRANSLATION COLLAPSE: key_from_keystroke returns Key::Backspace BEFORE the modifier check (app.rs:2465 — ⌥⌫/⌘⌫ lose their modifiers; ⌥⌫ 1-char-deletes TODAY; ⌘⌫ is swallowed at input.rs:88; forward-delete is a silent no-op in apply_editor_key_multi input.rs:264), movement.rs move_word_left/right (:67/:81 — motion IS the deletion range), the #299 grouped-raw-edit idiom (app.rs:6676), rebase_selections' clamp is CORRECT here (a caret inside its own deleted span → the span start — the exact opposite of #300), all five chords verified FREE, terminal ⌃K = readline KillToEnd via op_for_ctrl_key (input.rs:72 — Editor scope leaves it untouched)]
---

## Title
The #257 motions shipped without their destructive halves — deleting a word is still one Backspace at a
time. ⌥⌫/⌥⌦ delete the word left/right, ⌘⌫ to line start, ⌃K to line end, ⌘⇧K the whole line. The deleted
range is BY CONSTRUCTION the range the matching motion would travel — motion and deletion can never
disagree because they are the same function.

**The central finding this spec carries (recon 2026-07-17): these chords are NOT cleanly free at the app
layer even though the keymap has no rows for them.** `key_from_keystroke` early-returns `Key::Backspace`
BEFORE the modifier check (app.rs:2465), so **⌥⌫ silently does a 1-char delete today** (a wrong behavior
users may have habituated to — this ticket changes it to the right one), ⌘⌫ is swallowed as a no-op
(input.rs:88), and forward-delete (`"delete"`) reaches `apply_editor_key_multi` and does nothing
(input.rs:264). The work is a small restructuring of the key-translation seam, not just five keymap rows.

## Scope
### In
- **The pure op→range table (crates/editor, cov/MSI 100):** `delete_range_for(op, buffer, sel) ->
  Option<Range<CharOffset>>` where `op ∈ {WordLeft, WordRight, ToLineStart, ToLineEnd, WholeLine}`:
  - **The universal rule first:** a non-empty selection → the selection itself, for EVERY op.
  - WordLeft/Right → `caret ↔ move_word_left/right(buffer, caret)` (movement.rs:67/:81 — the SAME fns
    ⌥←/⌥→ ride; zero new boundary logic). At the buffer edge the motion returns the caret → `None` (no-op,
    not an underflow).
  - ToLineStart/End → `line_start(row) ↔ caret` / `caret ↔ line_end`; **⌃K at EOL eats the `\n`** (the
    emacs/observed behavior — pressing ⌃K twice deletes the text then joins the line).
  - WholeLine → the line INCLUDING its trailing `\n`; **at the LAST line (no trailing `\n`) eat the
    PRECEDING `\n` instead** so no orphan blank line remains — text round-trips (the pinned edge).
- **Multi-cursor via the #299 idiom + rebase:** per-cursor ranges (overlaps pre-merged by the SelectionSet
  invariant), `begin_undo_group(before)` → raw `edit(range, "")` BACK-TO-FRONT → `end_undo_group(after)`.
  **`rebase_selections` IS the right tool here** — its clamp (a position inside a removed span → the
  span's start, indent.rs:131) is exactly where a deletion should leave the caret. The #300 spec bans the
  same fn for moves; the pair of decisions documents the asymmetry so neither ticket cargo-cults the other.
- **The routing fix (the shim's real work):** five Editor-scoped keymap rows — ⌥⌫ `(F,F,T,F,"backspace")`,
  ⌥⌦ `(F,F,T,F,"delete")`, ⌘⌫ `(T,F,F,F,"backspace")`, ⌃K `(F,T,F,F,"k")`, ⌘⇧K `(T,F,F,T,"k")` — all
  verified FREE in the keymap (roster 67→72, scoped 22→27, individual asserts first). Plus the seam fix:
  the backspace/enter early-return at app.rs:2465 moves BELOW the modifier check (or the router learns the
  chords) so modified-backspace is visible to keymap resolution at all — design picks the exact shape;
  the REQ is behavioral. **⌃K is Editor-scoped precisely because the terminal's readline ⌃K
  (KillToEnd, input.rs:72) must stay byte-identical** — a regression drive pins it.
  Plain-delete-forward (no modifier) also starts WORKING as the standard 1-char forward delete — it is
  currently a silent no-op and fixing the seam makes it fall out; named so it's tested, not accidental.
- One undo unit per press; the group never leaks into the next keystroke (#338's redo lesson).
### Out (explicitly)
- Kill-ring/yank semantics (deleted text goes nowhere but undo — macOS-editor convention, not emacs);
  smart-whitespace variants (delete-to-indentation); ⌘⌦ as a ToLineEnd alias (⌃K only v1, named);
  camelCase sub-word boundaries (movement.rs owns word rules; changing them is its own ticket).

## Reference (§20)
VS Code / Zed / macOS text system = OBSERVED (the five chords; selection-wins; ⌃K's EOL join; ⌘⇧K's
last-line closure). The range table + routing fix are Marley-original over shipped seams.

### Prior art
1. **Behavior maps / observed** — the op semantics above, incl. the two nasty edges (⌃K at EOL; ⌘⇧K at the
   last line).
2. **Published material** — none needed.
3. **OUR OWN CODE — the sweep found the real work:** `movement.rs` already owns every boundary this ticket
   needs (`move_word_left/right`, and `line_col`/`line_start` for the line ops) — the deletion RANGES are
   free; `rebase_selections`' clamp is the CORRECT carry for deletions (verified against its doc — the
   inverse of #300's finding); the #299 grouped-apply idiom is the transaction shape. What no crate owns —
   and what the sweep EXPOSED — is the translation collapse at app.rs:2465 that makes these chords
   invisible today; that seam fix is the ticket's genuine delta.

## Locked-In Decisions (design confirms; deltas → the notes)
- **D-MOTION-IS-THE-RANGE** — deletion ranges come from the motion fns; no second boundary engine.
- **D-SELECTION-WINS** — any active selection is what every op deletes.
- **D-REBASE-IS-CORRECT-HERE** — the clamp = deletion's carry; documented against #300's opposite call.
- **D-CTRL-K-EATS-EOL-NEWLINE** — the observed join; pinned by table.
- **D-LAST-LINE-EATS-PRECEDING-NEWLINE** — ⌘⇧K leaves no orphan blank; round-trip pinned.
- **D-FIX-THE-TRANSLATION-SEAM** — modifiers reach the keymap for backspace/delete; the terminal's
  readline ⌃K and the plain-backspace editor path stay byte-identical (regression rows).

## Acceptance Criteria (EARS)
| # | The system shall | Verify |
|---|---|---|
| REQ-001 | delete the word left/right of the caret using the SAME boundaries ⌥←/⌥→ travel | pure table (motion-fn reuse) |
| REQ-002 | delete the active selection instead, for every op | pure |
| REQ-003 | no-op (None) at the buffer edge for word ops — no underflow, cursors intact | pure |
| REQ-004 | delete caret→line-start on ⌘⌫ and caret→line-end on ⌃K, ⌃K at EOL eating the newline | pure table |
| REQ-005 | delete the whole line incl. its `\n` on ⌘⇧K; at the LAST line eat the preceding `\n` (round-trip) | pure — the pinned edge |
| REQ-006 | apply N-cursor deletes in one undo unit, carets carried by the rebase clamp, merged where they collide | pure + headless |
| REQ-007 | route ⌥⌫/⌥⌦/⌘⌫/⌃K/⌘⇧K on the editor while the terminal's readline ⌃K and the editor's PLAIN backspace stay byte-identical | headless regression rows |
| REQ-008 | make plain forward-delete a 1-char forward delete (today a silent no-op) | headless |
| REQ-009 | revert one press with ONE ⌘Z; the group does not leak into the next keystroke | headless |

## Phase Plan
P2 design the translation-seam fix (move the early-return vs teach the router — trace BOTH the keymap path
and the IME/platform fallback so ⌥⌫ can't double-fire) + the op enum's home; P3 the pure range table first
(its truth tables incl. both nasty edges), then the seam fix + rows + dispatch; P3.5 critics on the
double-fire risk (keymap arm AND the old collapsed-Backspace path both alive), the terminal ⌃K regression,
the selection-wins interplay with pair-backspace (#338's arm must not swallow a ⌥⌫); P4 tables + drives +
gate; P5 docs. Standing traps: [m22-editing-bar.md](../../design-notes/m22-editing-bar.md).
