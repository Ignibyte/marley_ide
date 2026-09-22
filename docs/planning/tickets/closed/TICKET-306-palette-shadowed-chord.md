# TICKET-306 — Command palette advertises a chord that a context-scoped row has shadowed

- **Forge ticket:** #306 (d5b09d10-581c-47d4-82c1-ad446dfec2d0) (bug, M19 origin → ships M22)
- **Owner:** 466e35ad-09f6-4b81-89e7-b7fd16c1e45d
- **AAR:** 22741ab8-c3bd-4d66-8dee-d896a5d2a530
- **Pipeline doc:** ../../pipeline/active/306-palette-shadowed-chord.spec.md
- **Source ticket:** M22 IDE wrap-up train (#306, #295, #356, #357, #355)
- **Status:** closed

## Summary
The command palette renders each command's STATIC `binding` as keycap chips with no resolution
against the focused surface, so it advertises a chord even when a context-scoped keymap row has
shadowed it — e.g. ⌘⇧L "Split Right" while an editor pane is focused, where ⌘⇧L now resolves to
`select-all-occurrences` (#298). Pressing the advertised chord does something else. The fix shows a
command's chip only when its chord still resolves to THAT command on the active surface's context
stack (via the already-pure `keymap.action_for` + `action_for_command`); otherwise no chip.

## Acceptance
The palette shows a command's keycap chip iff `keymap.action_for(chord, active_stack)` equals that
command's own action (`action_for_command(id)`); otherwise no chip. Full EARS in the pipeline spec.
