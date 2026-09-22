# TICKET-267 — B2 input: EntityInputHandler (real OS/IME text input)

- **Forge ticket:** #267 7f1f13d4-b1fb-4334-bb1a-bba77a0a8eca (feature, M16)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** 669ad392-f54b-41d7-86f8-7d670061de95
- **Pipeline doc:** ../../pipeline/completed/267-b2-input.spec.md
- **Source ticket:** sprint #29 (forge)
- **Status:** closed

## Summary
The editor gets real OS text input: `EntityInputHandler` on RootView
routes the platform's NSTextInputClient calls (insertText → replace_text,
setMarkedText → replace_and_mark) into pure `marley_editor::ime` ops over
a new UTF-16↔CharOffset seam (ropey try_ conversions). Registration is
paint-scoped to the #266 editor branch, so terminal tabs keep the raw
path untouched. The hand-rolled char-insert arm is deleted (handler-only
insertion — the dispatch order proven in gpui 0.2.2 makes two live paths
a double-insert); the remaining editor key arms stop propagation. Dead
keys (⌥E→é) and IME composition work for the first time.

## Acceptance
UTF-16 seam exact + clamped; replace/mark/unmark ops pure + undoable;
typing/Enter/Backspace behavior unchanged through the new path; ⌥E→é
proven live; terminal prompt byte-identical. Full EARS in the spec.
