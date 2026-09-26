# One Marley per data directory — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-513-one-marley-per-data-dir.md
- **Pipeline spec:** 513-one-marley-per-data-dir.spec.md

## Phase 1 — Plan
- **Request:** found in #502's Test (2026-09-25): the installed and the debug Marley share
  `~/.local/share/marley`, and a second Marley on it hangs; part of Chad's goal ("the rest go
  ahead and begin implementing it now").
- **Classification / tier:** bug; one Zed crate (`zed`, two small hunks in `main.rs`) and a new
  module in a Marley crate.
- **Recall (§18.3):**
  - L-claude-438-the-headless-output-borrows-one-of-chads-workspaces-001: "the dev channel skips
    the single-instance check, so a second `marley <path>` starts a second app (it hung)".
  - L-claude-460-a-fresh-install-drive-beside-chads-own-window-001: with its own
    `--user-data-dir` a second app does not hang; the check must stay per data directory.
  - #437's critic: the socket derives from `data_dir()`, and Dev skips the check.
  - Brain (consultation 6c853fd51c3a4fec910a3dd52c6cbd22): nothing on this seam.
- **The scenario on the unfixed build (L-claude-512):** the second launch with `repo-b` started
  a second app (its own window, tiled beside the first, behind its trust prompt) and still ran
  after 15 seconds; it also took over the endpoint file, so the stand-in's `terminal_list`
  reached the second app, whose list held `repo-b` alone. The run failed at "the second launch
  exits", as the bug predicts. Two changes came of it: the scenario's terminals get a scratch
  HOME, and "repo-b opened" requires one listing with both folders, which only the first
  Marley's can hold.
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
  in `marley_workbench.rs`; `release_channel` in `marley_workbench`'s manifest (not a dependency
  yet); the ledger row for `main.rs` widened first. Written at Plan: the scenario
  `script/e2e/513-one-marley-per-data-dir.sh`, the runner's `E2E_MARLEY` and `E2E_CLASS` (the
  binary and the window class, for a scenario that starts a Marley of its own), and the stand-in
  agent's `terminals` command.

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

## Phase 2 — Code
- **Built:** `crates/marley_workbench/src/single_instance.rs` (declared `#[cfg(unix)]`):
  `hand_off(paths_or_urls) -> Result<usize>` sends one datagram per URL to
  `<data dir>/zed-<channel>.sock` (a path that exists canonicalized as `file://`; `file://`,
  `zed://`, `zed-cli://` and `ssh://` URLs as given; anything else made absolute against the
  working directory, so `src/main.rs:10:5` keeps its suffix; `zed://open` when there is none; a
  URL over 1,024 bytes refused before anything is sent), and answers the line to print: "Marley
  is already running on <dir>; it was handed N of this launch's paths" or "… asked to come
  forward"; its error says why the paths did not reach it. `crates/zed/src/main.rs`: the check's
  condition keeps `ZED_STATELESS` only, `ReleaseChannel` leaves the import, and on Linux and
  FreeBSD the failure branch prints `hand_off`'s line on stdout, or its error on stderr, before
  Zed's own "zed is already running" and `return`. `release_channel` joins `marley_workbench`'s manifest. The
  ledger row for `main.rs` names both hunks.
- **Deviations:** the design had the Marley function print; the Marley crates' lint table denies
  `print_stdout` and `print_stderr` (clippy's first run), so `hand_off` answers the line and
  `main.rs`, which prints already, prints it. Diff pairs (`--diff a b`) are not handed off: the
  listener's datagram carries one URL and no pair; a second launch with `--diff` exits as before.
- **Review:** no gpui state, so no re-entrancy; `url_for` is written from the forms
  `OpenRequest::parse` accepts, not taken from `parse_url_arg`'s body, and differs from it
  (relative paths made absolute, no zed.dev channel links). A check that failed for a reason
  other than a running Marley (a data directory it cannot write) now also prints the hand-off's
  error, which names the socket; before, only "zed is already running" printed. Both Zed hunks
  carry a `// Marley:` comment.
- **Checks:** `cargo check -p marley_workbench -p zed` clean; clippy on both with every target,
  warnings as errors: the first run denied the three prints, the second is clean; rustfmt clean.

## Phase 3 — Test
- **Scenario:** `script/e2e/513-one-marley-per-data-dir.sh` (`compositor sway`), written at Plan
  and red on the unfixed build at "the second launch exits" (a second app, tiled beside the first,
  still running after 15 s, and holding the endpoint file). The runner gained `E2E_MARLEY`,
  `E2E_CLASS` and `E2E_LOG` for scenarios, and the stand-in agent a `terminals` command.
- **Found in Test, fixed:** the first run on the fixed build never opened a window. The profile
  sat under this session's long `SHOT_DIR`, so `<profile>/zed-dev.sock` was 134 bytes, over a
  Unix socket's 108; Zed's bind failed, the check read it as a Marley running, the hand-off failed
  ("path must be shorter than SUN_LEN") and Marley exited. Any deep `--user-data-dir` would have
  done the same. Fixed at the source: `single_instance::socket_fits()` gates the check inside
  Linux's arm (a `// Marley:` hunk), and a data directory whose socket does not fit starts without
  the check and logs a warning (REQ-005, added). The runner now makes profiles under
  `$XDG_RUNTIME_DIR/marley-e2e/` (a 53-byte socket path), so every scenario's Marley runs the
  check whatever `SHOT_DIR` is; the runner removes them at the end, as before.
- **REQ-002, what the trace showed:** with the Settings window focused, the no-path hand-off made
  the running Marley ask: a diagnostic run with `WAYLAND_DEBUG=client` shows
  `get_activation_token` with the click's serial, the token back, and `activate(token,
  wl_surface#28)`, the workspace window. Sway did not act on it (its tree: the Settings window
  still focused, nothing urgent): gpui asks for the token from the window it activates, not the
  focused one, and sway checks tokens against the seat's focus. Omarchy's Hyprland focuses on any
  activation (`focus_on_activate = true`; the runner turns it off during Hyprland runs for that
  reason), which this run cannot show without moving Chad's focus. The criterion is the request,
  so the scenario runs the first Marley with `WAYLAND_DEBUG=client` and counts `activate`
  requests before and after the hand-off; the tree goes to the log. The Wayland-correct follow-up
  (the second launch forwarding its launcher's `XDG_ACTIVATION_TOKEN`) is ticketed at Complete.
- **Checks in the final run (all pass):** the second launch exits ("… it was handed 1 of this
  launch's paths"); one Marley on the profile; `terminal_list` from the first Marley holds
  terminals at `repo-a` and `repo-b`; the no-path launch exits ("… asked to come forward"); an
  activation request after it; the second profile's Marley runs; after `kill -9`, with
  `zed-dev.sock` left behind, a relaunch starts.
- **Shots, read:**
  - `513-01-handed-off`: one window; the rail lists `repo-b` above `repo-a`, each with its
    terminal; the title bar reads `repo-b`.
  - `513-02-asked-to-come-forward`: the Settings window on the Marley page beside the workspace
    window, the Settings window still focused (sway's call, above).
  - `513-03-two-profiles`: the first Marley's two windows and, on the right, the second profile's
    Marley on `repo-c` behind its trust prompt, its terminal on the scratch `$` prompt (the
    settings copy now carries the runner's scratch HOME).
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule
  and did not reload it".
- **Gate:** `just gate-diff`. The first run was red on clippy alone: `socket_fits` needed
  `#[must_use]` and a short first doc paragraph (the Marley crates' pedantic lints); the
  function came in Test, after Code's clippy run. Fixed; the second run ended `GATE GREEN
  [diff]`, 16 passed.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Fixed: One Marley at a time); `docs/marley_architecture/marley_workbench.md`
  (One Marley per data directory); `docs/marley/zed-touchpoints.md` (the `main.rs` row names the
  condition, `socket_fits` and the hand-off); `script/e2e.sh`'s header (`E2E_MARLEY`, `E2E_CLASS`,
  `E2E_LOG`; profiles under the runtime directory); the browser fixture's header (`terminals`).
  The guide's "One Marley at a time" and troubleshooting entry are rewritten in the guide draft,
  which lands with the docs batch.
- **Knowledge:** F-claude-513-a-socket-path-too-long-read-as-a-running-marley-001,
  PR-claude-check-a-unix-socket-path-against-sun-path-001,
  L-claude-513-gpui-asks-for-activation-from-the-window-it-activates-001,
  L-claude-513-a-second-launch-reaches-wayland-before-the-check-001,
  AD-claude-513-one-marley-per-data-directory-001.
- **Brain:** consultation 6c853fd51c3a4fec910a3dd52c6cbd22 closed with
  `decisions/one-marley-per-data-directory-a-second-launch-hands-off-through-zeds-own-socket`,
  follow-up by 2026-10-10.
- **Found here and ticketed:** TICKET-545 (the launcher's activation token handed to the running
  Marley), with a Queue row.
- **Ticket:** closed; the pair archived to `completed/`.
