# TICKET-251 — Editor focus + input intercept — type into the buffer, the caret moves (Enter⇒\n)

- **Forge ticket:** #251 (b97191e2-eff7-4db8-878a-90e7c1709fdc) (feature, M15 sprint #28)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 9269d495-736b-41dc-8db0-466860c4cb4c
- **Pipeline doc:** ../../pipeline/completed/editor-input-intercept.spec.md
- **Source:** the M15 "Editable Editor" train; the marquee interaction (the first WRITE path to the editor buffer).
- **Status:** closed

## Summary
Make the editor tab capture keystrokes: when an editor tab is active, route a plain key to the active file's
`Buffer` + caret (reuse `input::apply_key` via a thin pure `apply_editor_key` — Enter⇒insert `\n`, else delegate)
and RETURN, instead of leaking to a hidden terminal. The #250 caret starts moving + text inserts → the #250
faithful render redraws it live. Plain keys are captured by the editor (unhandled ones swallowed, not leaked);
⌘/ctrl chords fall through to the keymap; overlays (finder/palette/renaming) capture keys first (unchanged).

## Acceptance
Typing a char into an active editor tab inserts at the caret + advances it (render shows it); Backspace deletes
left (no-op at 0); Left/Right move one char clamped; Enter inserts `\n` (NOT submit — the prompt's Enter⇒Submit
unchanged); keys don't leak to a terminal when the editor is active + ⌘/ctrl chords still reach the keymap + the
terminal input is unchanged when a terminal tab is active. Pure units cov/MSI 100 on `apply_editor_key`; driven
proof (type → text appears + caret advances). Full EARS in the pipeline spec.
