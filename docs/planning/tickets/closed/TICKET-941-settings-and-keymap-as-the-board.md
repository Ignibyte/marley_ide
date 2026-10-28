# TICKET-941 — Settings and the keymap as the board draws them

- **Ticket:** LOCAL #941 (feature, the Marley look, Settings and the keymap)
- **Owner:** 065ccd79-6425-4f2c-bdc5-39a3e181e7f8
- **Pipeline doc:** [941-settings-and-keymap-as-the-board.spec.md](../../pipeline/completed/941-settings-and-keymap-as-the-board.spec.md)
- **Source ticket:** the design review of 2026-10-23 (docs/marley/design-review-2026-10-23/D-panels-editor.md, boards Settings, Keymap)
- **Boards:** Settings, Keymap
- **Size:** M
- **Status:** closed

## Summary

The Settings window's section list shows chevrons where the Settings board puts a small lettered
tile per section. The Layout choice reads "Zed | Marley" where the board puts Marley first. The
prompt's three switches sit inside Layout, where the board gives them their own "At the prompt"
group, and the board's "Agent text full width" row is missing from the Marley page. The page's
row titles are in Title Case where the board writes sentence case. In the keymap editor, the
columns carry Zed's names, Marley's own bindings read "Default" like Zed's, keystroke search is an
icon, and the record dialog speaks of a conflict where the board says what else the keys do and
offers Save Anyway. This ticket brings both screens to their boards. Zed's other settings pages
keep Zed's look (#844).

## Acceptance

The section list has a tile per section; Layout reads Marley | Zed; "At the prompt" is its own
group; "Agent text full width" is on the Marley page and writes `agent.limit_content_width`; row
titles are in sentence case. The keymap's columns read Action, Arguments, Keys, Where and From,
Marley's bindings read Marley and the user's read Yours, "Find by keys" is a text button, and the
record dialog reads "New keys for <action>", "Also <action>, in <where>" and Save Anyway; shot in
Marley Night and Marley Day.
