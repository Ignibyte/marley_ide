# TICKET-302 — Go to line (⌃G): a line-number overlay that jumps + centers

- **Forge ticket:** #302 8b8263f4-46ef-4251-b187-d037acf1c095 (feature, M19)
- **Owner:** claude (this session — the /work 300…259 goal)
- **AAR:** 215921e4-4469-4e60-8865-983f05ce6225
- **Pipeline doc:** ../../pipeline/active/302-goto-line.spec.md (promoted 2026-07-18; pipeline_id d44b0710-76ed-4f74-9f50-fb1b23afe102)
- **Source ticket:** the IDE-MVP shelf ([ide-mvp-shelf.md](../../design-notes/ide-mvp-shelf.md)) — the SECOND of `/work 300,302,303,304,305,314,315,316,317,259`
- **Status:** closed

## Summary
⌃G opens a small "Go to line…" overlay; typing a number jumps + centers the target, Enter commits, Esc
cancels and restores the prior caret. In a 2000-line file today you scroll. The jump machinery already
exists end-to-end — `scroll_editor_to_row` centers, `caret_for_line_col` places AND clamps — so the genuine
new work is the parse (`parse_goto` for `N` / `N:C`) + the overlay (copying the `renaming_symbol` shape).

## Acceptance
⌃G, type `50`, Enter → the caret is on line 50 and the view is centered on it; `50:12` places the column;
past-EOF clamps to the last line (never an error); Esc restores the prior caret. Full EARS in the spec.
**Plan finding:** `caret_for_line_col` already owns the 1-based→0-based + past-EOF/EOL clamp, so the spec's
separate `clamp_goto` dissolves — `parse_goto` is the ONLY new pure fn.
