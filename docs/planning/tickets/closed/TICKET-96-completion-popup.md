# TICKET-96 — the tab-completion popup (follow-up to #89)

- **Forge ticket:** #96 `e75ddab6-8c2b-45ef-852b-5594ebb4b31f` (feature; sprint #21 — the last open ticket)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `465f7aa8-0f17-4034-b06b-49c6e1dfbd53`
- **Pipeline doc:** ../../pipeline/active/completion-popup.spec.md
- **Status:** closed

## Summary
Many matches + prefix already typed → a pane-bound popup (pure CompletionState + popup_window) under the
prompt: Tab/↓/↑ cycle, Enter/click accepts (+space unless a dir/), Esc dismisses, other keys type through.
