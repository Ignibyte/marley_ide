# TICKET-513 — One Marley per data directory: a second launch hands its paths to the first

- **Ticket:** LOCAL #513 (bug, the Marley app: startup)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/513-one-marley-per-data-dir.spec.md
- **Source ticket:** found in #502's Test, 2026-09-25: the installed Marley and the debug build
  share `~/.local/share/marley`, the dev channel skips Zed's single-instance check, and a second
  Marley on the same data directory hangs
- **Status:** closed

## Summary
Every Marley build is on Zed's `dev` channel, and Zed skips its single-instance check on that
channel so its developers can run a dev build beside another. Marley's installed and debug builds
share one data directory, and two Marleys on it hang. Starting Marley from the menu while it
runs, or running `marley <folder>` in a terminal, starts that second app. On Linux the check is a
datagram socket in the data directory, `zed-dev.sock`, that the running app reads URLs from:
`file://` opens a path and `zed://open` brings a window forward. Marley runs the check on the dev
channel too, so it holds per data directory, and a second launch sends its paths (or
`zed://open` when it has none) to the Marley that runs, then exits. Omarchy's Hyprland focuses a
window that asks for activation (`focus_on_activate`), so the running Marley comes forward.

## Acceptance
A second `marley` on the same data directory exits within seconds, and the running Marley opens
the folder it was given, or comes forward when it was given none; Marleys on different data
directories still run side by side.
