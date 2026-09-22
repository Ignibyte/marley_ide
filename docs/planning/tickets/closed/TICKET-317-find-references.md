# TICKET-317 — Find references (⇧F12): every usage, grouped by file, one Enter from the caret

- **Forge ticket:** #317 51426ddb-f716-4de2-9865-42ce0b17c1d2 (feature, M20)
- **Owner:** claude (session 99b5bc91)
- **AAR:** fe3478a7-7ca2-4ee9-a922-3f9baa3cda28
- **Pipeline doc:** ../../pipeline/active/317-find-references.spec.md
- **Source ticket:** M20 language intelligence (#308-317); the read-only sibling of #312 go-to-definition + the last leg of navigate-by-meaning (#312 def + #304 file-symbols + this)
- **Status:** closed

## Summary
⇧F12 on a symbol answers "who uses this?" — every reference in a picker, grouped by file, each row showing its line's text with the match span emphasized; Enter jumps (centered, NavStack-pushed so ⌃- returns). The #312 `DefPicker` is definition-shaped (flat, path:line labels, no line text, no grouping) so references need a NEW grouped row model; the genuine reuse is the request path + the #312 position-keyed stale-guard + `jump_to_definition`'s open+center+push-NavStack + `cap_with_tail` + `FinderState` + `fuzzy_score`.

## Acceptance
Group/sort/dedupe references (current-file first) with a computed count + honest "+K more" cap; a picker of file-header + `line: text` rows (match span emphasized), type-to-filter; Enter jumps + pushes the NavStack; a superseded ⇧F12 drops; zero-results/no-capability flashes quietly. Full EARS in the pipeline spec.
