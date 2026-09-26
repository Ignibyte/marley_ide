---
pipeline_id: d73ae04a-5388-4dea-a0af-18e9299be146
ticket: docs/planning/tickets/open/TICKET-513-one-marley-per-data-dir.md
status: Phase 4 — Complete PASS
title: "One Marley per data directory: a second launch hands its paths to the first"
type: bug
slice: the Marley app, startup (found in #502's Test)
references: [docs/planning/pipeline/completed/502-installed-release-build.spec.md]
---

## Title
A second Marley started on a data directory where one already runs hands its paths to the
running one, which opens them or comes forward, and exits, instead of starting beside it and
hanging.

## Scope
### In
- `crates/zed/src/main.rs` (Zed crate): the single-instance check runs on the `dev` channel too,
  which is every Marley build's. On Linux it binds `<data dir>/zed-dev.sock`, so it holds per
  data directory: `--user-data-dir` gives a Marley a socket of its own.
- When the check finds a running Marley, the second launch sends each of its paths and URLs to
  that socket before it exits, as `file://` URLs for paths (made absolute against its own working
  directory, so a relative path and a `path:line:column` suffix reach the first Marley intact)
  and as given for URLs; with none it sends `zed://open`. It prints what it did to stdout.
- The Marley function that does it (`crates/marley_workbench/src/single_instance.rs`, new):
  `hand_off(paths_or_urls) -> Result<usize>`, the URLs it sent.

### Out (explicitly deferred)
- A path over 1,024 bytes: Zed's listener reads a datagram into a 1 KiB buffer, so such a path
  would arrive cut. The hand-off refuses it with a message instead of sending it.
- Forwarding the second launch's XDG activation token: gpui's `activate` asks the compositor for
  a token of its own, which Omarchy's Hyprland honours (`focus_on_activate = true`); a
  compositor that ignores it marks the window urgent.
- Writing the endpoint file atomically, and removing it only while it holds this Marley's bearer:
  with one Marley per data directory, no second Marley can remove the first's file.
- Keeping the previous installed binary for a rollback (#502's installer).

## Reference (§20)
Upstream Zed (`crates/zed/src/main.rs`, `crates/zed/src/zed/open_listener.rs`,
`crates/cli/src/main.rs`): on Linux a second Zed on the same channel finds the first through
`zed-<channel>.sock` in the data directory, and Zed's CLI hands its request to the running app
through that socket; the app opens `file://` URLs and treats `zed://open` as "focus the app"
(`OpenRequestKind::FocusApp`, `workspace::activate_any_workspace_window`). Marley keeps all of
it and changes two things: the check also runs on the `dev` channel, and the app binary itself
hands off, since Marley ships no CLI.

### Prior art
- **Code we already ship.** `listen_for_cli_connections` (binds the socket, removes a stale one
  whose owner died: `ConnectionRefused`), the listener thread that turns each datagram into a
  `RawOpenRequest`, `OpenRequest::parse` (`file://`, `zed://open`), `parse_url_arg` (how the app
  turns a command-line argument into a URL: canonicalized paths, known schemes as given),
  `activate_any_workspace_window`; the CLI's Linux `launch`, which sends one datagram to the same
  socket. `paths::set_custom_data_dir` runs before the check, so `--user-data-dir` moves the
  socket.
- **Published material.** `unix(7)` datagram sockets; the xdg-activation protocol; Hyprland's
  `misc:focus_on_activate`.
- **On this box.** Omarchy's Hyprland config sets `focus_on_activate = true` for every window.

## UI proof
UI-AFFECTING (what a second launch does). `script/e2e/513-one-marley-per-data-dir.sh`
(`compositor sway`): the run's Marley opens folder A. The harness starts the same binary on the
same profile with folder B: it exits within 10 seconds, one Marley process runs, and the
running one shows B (`513-01-handed-off`). A second hand-off with no path while the Settings
window has the focus: the running Marley sends an `xdg_activation_v1.activate` request for its
workspace window, seen in its Wayland trace (the run turns `WAYLAND_DEBUG=client` on), shot
`513-02-asked-to-come-forward`; sway's tree goes to the log, since focusing is the compositor's
call. A Marley on a second profile starts beside it (`513-03-two-profiles`). After `kill -9` of
the first, a new launch on the first profile starts though the socket file is still there (the
run log).

## Locked-In Decisions
- D1 — The check holds per data directory, not per machine: `--user-data-dir` still gives a
  second Marley (the e2e runs, a scratch profile).
- D2 — Hand off through Zed's own socket and URLs: no second socket, no new protocol.
- D3 — A hand-off that fails (the running Marley died between the check and the send) is
  printed to stderr and this launch exits; the next launch finds the socket's owner gone, removes
  it, and starts. No retry loop at startup.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts on a data directory where a Marley runs, it shall send its paths to the running Marley and exit within 10 seconds, and the running Marley shall open them. | The run log (one process, the exit), shot `513-01-handed-off` |
| REQ-002 | WHEN Marley starts with no path on a data directory where a Marley runs, the running Marley shall ask the compositor to activate one of its windows. | The run log (an `xdg_activation_v1.activate` request after the hand-off), shot `513-02-asked-to-come-forward` |
| REQ-003 | WHILE a Marley runs on one data directory, a Marley started on another shall start. | The run log (two processes), shot `513-03-two-profiles` |
| REQ-004 | WHEN the socket's Marley has died, Marley shall start on that data directory as before. | The run log (the window after `kill -9` and a relaunch) |
| REQ-005 | WHERE the data directory's socket path is too long for a Unix socket, Marley shall start without the check, and log why. | The first Test run, whose long profile path refused to start; review of `socket_fits` |

## Phase Plan
- **P1 Plan** — this spec; the design in the notes.
- **P2 Code** — the two `main.rs` hunks and `single_instance.rs`; the ledger row first.
- **P3 Test** — the scenario; the gate.
- **P4 Complete** — docs (the guide's "One Marley at a time"), knowledge, close, archive, commit.
