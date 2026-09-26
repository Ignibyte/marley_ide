# One Marley per data directory — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-513-one-marley-per-data-dir.md
- **Pipeline spec:** 513-one-marley-per-data-dir.spec.md

## Phase 1 — Plan
- **Request:** found in #502's Test (2026-09-25): the installed and the debug Marley share
  `~/.local/share/marley`, and a second Marley on it hangs; part of Chad's goal ("the rest go
  ahead and begin implementing it now").
- **Classification / tier:** bug; one Zed crate (`zed`, two small hunks in `main.rs`) and a new
  module in a Marley crate.
- **Recall (§18.3):** #502's notes (the two-Marleys hang, the guide's "One Marley at a time"
  workaround); L-claude-460 (Marley's "exits silently" reports); the e2e runner refuses a
  Hyprland run while a Marley window is open for the same reason. Brain: asked at promotion.
- **Discovery:** `crates/zed/src/main.rs` 262 (`paths::set_custom_data_dir`, before the check),
  359 to 383 (the check, skipped on `ReleaseChannel::Dev`; `println!("zed is already running")`),
  51 (`ReleaseChannel` imported for that line only), 1813 (`parse_url_arg`);
  `crates/zed/src/zed/open_listener.rs` 409 (`listen_for_cli_connections`: the socket
  `zed-<channel>.sock`, the stale-socket removal, a 1 KiB datagram buffer), 126 and 179
  (`zed://open` is `FocusApp`); `crates/workspace/src/workspace.rs` 10758
  (`activate_any_workspace_window`); `crates/gpui_linux/src/linux/wayland/window.rs` 1789
  (`activate` asks for an xdg-activation token); Omarchy's `looknfeel.lua` sets
  `focus_on_activate = true`.

### Design
- **The check on the dev channel.** The condition keeps `ZED_STATELESS` and drops the dev
  channel, with a `// Marley:` comment; `ReleaseChannel` leaves the import if nothing else uses it.
- **The hand-off.** In the failure branch, before Zed's `println!` and `return`, on Linux and
  FreeBSD: `marley_workbench::single_instance::hand_off(&args.paths_or_urls)`, its error printed
  to stderr. `hand_off` connects an unbound `UnixDatagram` to
  `paths::data_dir().join(format!("zed-{}.sock", *RELEASE_CHANNEL_NAME))` and sends one datagram
  per URL: for each argument, a canonicalized path as `file://<path>`; an argument that starts
  with a scheme Zed takes (`file://`, `zed://`, `zed-cli://`, `ssh://`) as given; anything else as
  `file://` plus the argument made absolute against the working directory (a `path:line:column`
  that does not exist as a file keeps its suffix, which the first Marley parses). With no
  argument it sends `zed://open`. A URL over 1,024 bytes is refused before anything is sent.
  It prints "Marley is already running on <data dir>; it was handed N paths" (or "was asked to
  come forward").
- **File manifest:** `crates/zed/src/main.rs` (Zed crate: the condition, the hand-off call, the
  import); `crates/marley_workbench/src/single_instance.rs` (new, Marley crate) and its `mod` line
  in `marley_workbench.rs`; `release_channel` in `marley_workbench`'s manifest if it is not a
  dependency already; the ledger row for `main.rs` widened first.

### E2E plan
| REQ | Fixture and steps | Proof |
|---|---|---|
| REQ-001 | `repo-a` opened by the runner; from the harness, `$marley --user-data-dir $E2E_PROFILE $E2E_WORK/repo-b` with the runner's sway environment; wait for it to exit | Its exit within 10 s and its stdout line; `pgrep` counts one Marley on the profile; `513-01-handed-off` shows `repo-b` |
| REQ-002 | A second window or the Settings window takes the focus (`marley: open settings`); the hand-off with no path | `swaymsg -t get_tree`: the workspace window focused or urgent; `513-02-came-forward` |
| REQ-003 | `$marley --user-data-dir $E2E_WORK/profile-2 $E2E_WORK/repo-c` | Two Marley processes; `513-03-two-profiles` |
| REQ-004 | `kill -9` the first; the socket file stays; `launch_marley` | A window within 90 s; the log line of the start |

### Risks
- A running Marley that is hung still owns the socket, so a relaunch hands off to it and exits:
  the user must kill the hung one (the guide's troubleshooting says how). Before, the relaunch
  started a second Marley that hung too.
- A quit in progress still owns the socket: a launch in that second hands off to a Marley that is
  closing, and nothing opens. D3 accepts it; a second launch a moment later starts.
- Zed's own upstream merge: the condition's line is a known conflict spot; the ledger row names it.
