# TICKET-303 — The missing destructive ops: delete word / to line edge / delete line

- **Forge ticket:** #303 38691b71-83d9-4a0d-acb2-e40ea9a4bc34 (feature, M19)
- **Owner:** claude (this session — the /work 300…259 goal)
- **AAR:** ca13a3bd-f9d0-4224-9008-0a44bfb77d63
- **Pipeline doc:** ../../pipeline/active/303-delete-ops.spec.md (promoted 2026-07-18; pipeline_id ee3a0377-591c-4e9a-9866-e377f7b493c8)
- **Source ticket:** the IDE-MVP shelf ([ide-mvp-shelf.md](../../design-notes/ide-mvp-shelf.md)) — the THIRD of `/work 300,302,303,304,305,314,315,316,317,259`
- **Status:** closed

## Summary
The #257 word-motions (⌥←/⌥→) shipped without their destructive halves — deleting a word is still one
Backspace at a time. This adds ⌥⌫/⌥⌦ (delete word left/right), ⌘⌫ (to line start), ⌃K (to line end), and
⌘⇧K (delete whole line). The deleted range is BY CONSTRUCTION the range the matching motion travels — a
pure `delete_range_for(op, buffer, sel)` table reusing `movement.rs`'s `move_word_left/right` + line
boundaries, applied through the #299 grouped-edit idiom with `rebase_selections`' clamp (which — unlike
#300 — is the CORRECT post-delete caret carry).

## Acceptance
⌥⌫ deletes the word left using the same boundary ⌥← travels; a non-empty selection wins for every op; ⌃K at
EOL eats the `\n`; ⌘⇧K on the last line eats the preceding `\n` (no orphan blank line); N cursors delete in
one undo unit; the terminal's readline ⌃K and the editor's plain backspace stay byte-identical. Full EARS in
the spec.
**Central premise under re-verification (Phase 1):** the spec claims a "translation collapse" — `key_from_keystroke`
returning `Key::Backspace` before the modifier check, so ⌥⌫/⌘⌫ are mis-routed today. If false, the routing
scope reshapes (see the Phase 1 ledger in the notes).
