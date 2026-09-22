# TICKET-256 — Editor copy / cut / paste over the selection (⌘C / ⌘X / ⌘V)

- **Forge ticket:** #256 (1743bdba-bcf0-4872-9f9d-6118ef7267bd) (feature, M15)
- **Owner:** (autonomous /goal /work 249 to 258)
- **AAR:** cf78b64b-15eb-4705-9f81-25e3ba4bca05
- **Pipeline doc:** ../../pipeline/completed/clipboard.spec.md
- **Source ticket:** M15 — The Editable Editor (sprint #28, fa328492); the #242 decomposition (#248–258)
- **Status:** closed

## Summary
⌘C copies the editor selection to the system clipboard, ⌘X cuts (copy + delete the selection via the Buffer
edit), and ⌘V pastes (replacing an active selection, else inserting at the caret) — reusing #255's selection,
#249's `Buffer::edit`/`text_in_range`, #253's undo, and the existing gpui clipboard plumbing the terminal already
uses. The editor-clipboard branch lives in `on_key_down`'s platform-chord region (before the terminal cmd-C/cmd-V),
guarded on an active editor tab. The only new pure logic is `paste_edit` (the range to replace + the post-paste
caret); cov/MSI 100. Deps #255/#249/#253.

## Acceptance
⌘C copies the selected text (no selection → no-op), ⌘X copies + deletes it, ⌘V inserts the clipboard replacing a
selection or at the caret with the caret after; a terminal tab keeps the existing terminal copy/paste; the
paste-range math is pure. Driven-proven live. Full EARS criteria in the pipeline spec. Deferred: copy-line,
multi-cursor, ctrl-editing (#257).
