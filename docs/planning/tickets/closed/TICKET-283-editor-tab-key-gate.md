# TICKET-283 — Editor tab: swallow modified Enter/Tab/Backspace (hidden-prompt leak)

- **Forge ticket:** #283 (401ba6b7-d6e6-4886-94e9-7db9a1e4adc6) (bug, M17)
- **Owner:** autonomous /goal run (session a2a59fa8)
- **AAR:** f836f1d7-a885-49bc-b0cd-975b9087051b
- **Pipeline doc:** ../../pipeline/active/283-editor-tab-key-gate.spec.md
- **Source ticket:** M17 follow-up shelf (#282–287, sprint #30); from #276 routing critic F4
- **Status:** closed

## Summary
On an editor tab, a modified Enter/Tab/Backspace (⌘Enter, ⌃Enter, ⌘Backspace, ⌃Tab) skips the editor
key-router (gated `!platform && !control`) and falls to the terminal handlers, which act on the HIDDEN
prompt via `focused_terminal` (#71): ⌘Enter submits it, ⌘Backspace edits it, ⌃Enter/⌃Tab write raw
bytes. Add a pure "swallow on a non-terminal tab" predicate + a shim gate (mirroring #278) so these keys
no-op instead of corrupting an invisible surface.

## Acceptance
Editor-tab ⌘Enter/⌃Enter/⌘Backspace/⌃Tab leave the hidden prompt unchanged; terminal-tab
Enter/Tab/Backspace stay byte-identical; other keys keep their #267 fallthrough. Pure predicate at
cov/MSI 100 + headless `hidden_prompt_text` asserts. Full EARS in the pipeline spec.
