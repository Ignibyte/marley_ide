# A second launch hands its launcher's activation token to the running Marley — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-545-activation-token-hand-off.md
- **Pipeline spec:** 545-activation-token-hand-off.spec.md

## Phase 1 — Plan
- **Request:** TICKET-545, found in #513's Test (2026-09-26): sway did not act on the running
  Marley's request to come forward.
- **Classification:** feature, the app's startup; Marley crate `marley_workbench` and four small
  Zed hunks (`gpui`, `gpui_linux`, `zed` twice).
- **Recall:** #513's notes (the listener, the hand-off, `hand_over` in #507's scenario). Brain:
  consultation b2df8fc525814c748791a707b23b8b65, nothing on this seam.
- **Discovery.** `main` builds the application (line 348) before the single-instance check (368),
  and the Wayland client removes `XDG_ACTIVATION_TOKEN` from the environment when it starts
  (`client.rs` 195 to 204), so the token must be read first. wlroots issues a valid token only to
  the client whose surface has the keyboard focus, with one of its input serials; sway's default
  `focus_on_window_activation` is `urgent`.
- **Design.** `single_instance::keep_activation_token()` stores the variable in a `OnceLock`;
  `send` puts `zed://marley-activation-token/<token>` first. The listener thread checks
  `single_instance::activation_token_in(&url)` and calls `gpui::set_next_activation_token`. gpui:
  a `Mutex<Option<String>>` static with the two functions. `WaylandWindow::activate` takes the
  slot first and calls `activation.activate(token, surface)`.
- **Manifest.** `crates/marley_workbench/src/single_instance.rs`; `crates/zed/src/main.rs`,
  `crates/zed/src/zed/open_listener.rs`, `crates/gpui/src/platform.rs`,
  `crates/gpui_linux/src/linux/wayland/window.rs` (rows first); the scenario.

## Phase 2 — Code
- **Built.** `single_instance::keep_activation_token` (a `OnceLock`), `activation_token_in`, and the
  token datagram first in `send`; `main` calls the first on Linux and FreeBSD before anything
  else; the listener thread's check; gpui's `set_next_activation_token` and
  `take_next_activation_token` (a `std::sync::Mutex<Option<String>>` static); the early branch in
  `WaylandWindow::activate`. Four ledger rows written first (main.rs's grown, three new).
- **Review.** The token is only read at startup, so a launch that runs keeps it for its own first
  window as upstream does. The slot is taken once; a stale token only reaches the compositor,
  which refuses it. The datagram never reaches `OpenRequest::parse`.
- **Gate.** GATE GREEN [diff].

## Phase 3 — Test
- **Scenario.** `script/e2e/545-activation-token-hand-off.sh` (sway, `focus_on_window_activation
  smart` set with swaymsg): a GTK4 stand-in launcher (`Gdk.AppLaunchContext.get_startup_notify_id`,
  which asks with GTK's last press serial) takes the focus to the right of Marley's window.
- **Found on the way.** The first run's token did nothing: the launcher had had no press, so GTK
  asked with no press serial and wlroots handed back a random token, which it does for any refused
  request. A click on the launcher before it asks gives the serial a real launcher's click gives.
- **Shots (run 2, read).**
  - `545-01-no-token`: a hand-off with no token; Marley's terminal cursor hollow, the focus still on
    `dev.e2e.Launcher` in sway's tree. REQ-002.
  - `545-02-token`: a hand-off with the launcher's token (32 characters); the cursor filled, sway's
    focused window `dev.zed.Zed-Dev`. REQ-001.
  - The run's Marley log holds no `marley-activation-token` line. REQ-003.
- **Deviation.** The ticket asked for a Wayland trace of the `activate` request; the two hand-offs,
  one without and one with the token, show the same thing through sway's own verdict.

## Phase 4 — Complete
- Ledger: L-claude-545-a-refused-activation-token-looks-like-a-good-one-001.
- Brain: decision recorded on consultation b2df8fc525814c748791a707b23b8b65.
