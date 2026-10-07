# TICKET-686 — Keymap changes the user accepts

- **Ticket:** LOCAL #686 (feature, prong 2 C: Marley's MCP server)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** ../../pipeline/completed/686-keymap-changes-accepted.spec.md
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 1, item 4
  (split from TICKET-682 on 2026-10-07)
- **Status:** closed

## Summary
`keymap_change` lets an agent propose a key binding (keystrokes, an action or none to unbind, a
context) for the user's `keymap.json`. Marley shows the change and applies it only when the user
accepts, through Zed's own keymap updater (`KeymapFile::update_keybinding`, the keymap editor's
path), so the file's comments and other bindings stay; an action Marley does not know is refused
before anything is shown. It shares TICKET-682's card and its answers (applied, declined, no
answer).

## Acceptance
An agent proposes `ctrl-alt-m` for `marley::OpenGuide` in `Workspace`; Marley shows the change;
Apply adds the binding with the file's comments kept and the key works at once; Decline leaves the
file as it was; an unknown action is refused with no card.
