# TICKET-313 — LSP completions: the as-you-type popup with fuzzy ranking + a TextEdit-faithful accept

- **Forge ticket:** #313 ad15e168-b72d-4a46-a64a-90f9b858408d (feature, M20)
- **Owner:** fabd8254-8504-4118-9e8e-8827ad24687a
- **AAR:** 989649bc-21e1-47a0-8624-12a360db0ef1
- **Pipeline doc:** ../../pipeline/active/313-lsp-completions.spec.md
- **Source ticket:** forge #313 (M20 batch #308-317)
- **Status:** in-progress

## Summary
Type in the editor and the language server offers what comes next: a popup under the caret, fuzzy-ranked
by what you have typed (via the finder's own `fuzzy_rank` — one matcher, everywhere), that on Enter
**replaces the typed prefix rather than appending after it**. It is the third consumer of the #311
request→response path, so the wire is free; what is genuinely new is the statefulness — it fires per
keystroke, so the stale-guard (keyed on uri + position + **buffer version**), the suppression rules
(strings/comments, and Esc-until-next-trigger), and the accept's undo bracketing ARE the feature.

## Acceptance
F12-free daily value: type `Vec::` and see items; type `pu` and `push` ranks to the top; press Enter and
`pu` is **replaced**, with one ⌘Z reverting only the accept (never the character typed after it); Esc
closes and stays closed until the next real trigger; no popup inside a string or comment. Full EARS
criteria (REQ-001..008) live in the pipeline spec.

## Plan-phase corrections to this ticket's own text
Two claims inherited from #301's description are FALSE and are dropped (proven by Explore, §18.2):
1. **"the tree knows — crates/syntax `point_at`"** — `point_at(src, byte) -> (row, byte-col)` is a
   coordinate converter. `marley_syntax` exposes no tree/node query at all (its `tree_sitter::Tree`
   never leaves the crate). `in_string_or_comment` is NET-NEW over the app's cached
   `Vec<Vec<(Range, TokenKind)>>` (`Str`/`Comment` spans) — and that cache is async + version-keyed, so
   its staleness on the triggering keystroke is a real design question, not a lookup.
2. **"after the #301 `pair_action` consult"** — `pair_action` has ZERO occurrences in `crates/`; #301 is
   open/unimplemented. The ordering is a PHANTOM dependency; #313 hooks the printable path directly and
   is not blocked. **#301's description carries the same `point_at` error and should be corrected when
   it is worked.**
</content>
