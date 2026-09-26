# A Wayland seat without a keymap no longer kills Marley — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-512-no-keymap-no-panic.md
- **Pipeline spec:** 512-no-keymap-no-panic.spec.md

## Phase 1 — Plan
- **Request:** found in #502's Test on 2026-09-25 (see its notes, Phase 3).
- **Classification / tier:** bug, a Zed crate (`gpui_linux`); a small additive hunk.
- **Discovery:** `crates/gpui_linux/src/linux/wayland/client.rs` lines 1850 to 1965 (the
  `wl_keyboard` dispatch); the backtrace from #502's first run.

### Design
- To be written at promotion.
