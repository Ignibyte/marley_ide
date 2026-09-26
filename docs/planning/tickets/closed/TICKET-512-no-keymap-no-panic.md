# TICKET-512 — A Wayland seat without a keymap no longer kills Marley

- **Ticket:** LOCAL #512 (bug, platform: gpui's Wayland client)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/512-no-keymap-no-panic.spec.md
- **Source ticket:** found in #502's Test, 2026-09-25: the installed Marley started the way Omarchy's menu starts an entry, on a seat whose keyboard had just gone, panicked at `crates/gpui_linux/src/linux/wayland/client.rs:1921`
- **Status:** closed

## Summary
gpui's Wayland client builds its keyboard state from the compositor's `wl_keyboard.keymap`
event and then unwraps that state on every `modifiers` and `key` event. A compositor may send a
`modifiers` event with no usable keymap before it: the seat has no keyboard (the keymap comes
as `no_keymap`), or the keymap fails to compile (an `expect` panics in the keymap handler
itself). Marley then dies on its first keyboard event, and on the `dev` channel the panic goes
to stderr only, which a program started from the menu never shows: the "exits silently"
reports in L-claude-460 fit this, though they are not proven to be it. Upstream Zed has the
same code today. The handler skips keyboard events until a keymap arrives, and logs a keymap it
cannot use instead of panicking.

## Acceptance
Marley started on a seat with no keyboard keeps running, and takes keys once a keyboard arrives.
