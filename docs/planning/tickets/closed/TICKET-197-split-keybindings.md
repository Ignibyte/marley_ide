# TICKET-197 — Split keybindings (⌘-chords for Split Right / Split Down)

- **Forge ticket:** #197 (59f70ac0-a0bc-438f-8a84-9a25344e1090) (feature, M12.2)
- **Owner:** c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** da6cfe79-3922-4077-9b77-844dc2c3e128
- **Pipeline doc:** ../../pipeline/active/split-keybindings.spec.md
- **Source ticket:** M12.2 "Terminal fidelity & cockpit UX" (sprint #25)
- **Status:** closed

## Summary
Splitting a terminal pane (Split Right = horizontal neighbor, Split Down = vertical) is
reachable only via the right-click context menu today. Add keyboard chords + command-palette
entries that dispatch to the existing `split_focused_pane(axis)`. Note: ⌘D's `"split-pane"`
keymap action is a misnomer — it runs `new_terminal_pane()` (#135), not a tile-split; the
design may rename it `"new-terminal"` (behavior unchanged). Pure seam: the keymap chord→action
resolution + a duplicate-chord collision guard (cov/MSI 100).

## Acceptance
Each split chord resolves + splits the focused pane in its direction; no chord collision;
the palette lists Split Right / Split Down. Full EARS (REQ-001..004) in the pipeline spec.
