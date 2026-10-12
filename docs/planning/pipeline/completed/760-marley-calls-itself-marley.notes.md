# Marley calls itself Marley - its app id, its name and its socket — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-760-marley-calls-itself-marley.md
- **Pipeline spec:** 760-marley-calls-itself-marley.spec.md

## Phase 1 — Plan (2026-10-11)
- **Request:** #445's split (Chad, 2026-10-11, "7.) yes" to Marley's own identity); its spec's Out
  item 1 drafted this slice. Taken from the Queue under the goal "continue until all tickets are
  finished", with the defaults written in the spec.
- **Classification:** chore, packaging; two small Zed hunks (`release_channel`, `open_listener`),
  one Marley crate, two Marley scripts. Size S.
- **Recall (§18.3):**
  - AD-claude-437-marley-identity-is-app-name-and-the-dev-channel-001 and
    PR-claude-a-zed-fork-identity-is-more-than-app-name-001: the identity beyond `APP_NAME` is
    keyed by the channel; stay on `dev` and give each piece its own value.
  - #513 (one Marley per data directory): the socket's path must fit `sun_path`
    (PR-claude-check-a-unix-socket-path-against-sun-path-001); `socket_fits` measures it.
  - The e2e runner and #743's scenario name the class.
  - Brain: consultation 854b755eefb74017868cf8b900be7e32, nothing on this seam.
- **Discovery** (an Explore pass at `b111895a65`):
  - `app_id()`'s seven callers set `WindowOptions.app_id`; gpui hands it to xdg-shell and to
    `WM_CLASS`. Notifications use `set_app_identity("marley", "Marley")` and ignore the id. No
    portal, D-Bus name, keyring label or path uses it.
  - `display_name()`'s callers are text: the About line (`zed.rs:1559`), `${appName}`
    (`workspace.rs:6935-6939`), the update notification, System Specs (clipboard, bug-report URL),
    telemetry (off by default), LSP `client_info.name`, ACP's title. Paths and keys use
    `dev_name()` or `RELEASE_CHANNEL_NAME`.
  - The socket: `open_listener.rs:431` (listener), `single_instance.rs:59-66` (Marley's
    hand-off and `marley <verb>`), `cli/src/main.rs:948-957` (Zed's CLI, not shipped);
    `socket_fits` measures the full path.
  - No test asserts either value; `paths.rs:640-663`'s comment names the app id among the
    channel's reasons.
  - Hard-coded: `script/install-marley:187`, `script/e2e.sh:59` (with `:125-128`, `:139`,
    `:160-163`, `:782`, `:804`), the in-app guide `:340`, `docs/marley/guide.md:83`, the
    architecture note `:70`, `single_instance.rs:4`. `~/.config/hypr` names no Zed class.
  - `ignibyte.com` is Ignibyte's domain (the ops handbook).

### Design
- **`crates/release_channel/src/lib.rs` (Zed crate):** the `Dev` arms of `display_name()` and
  `app_id()`, with a `// Marley:` line each.
- **`crates/zed/src/zed/open_listener.rs` (Zed crate):** the socket path from
  `marley_workbench::single_instance::socket_path()`.
- **`crates/marley_workbench/src/single_instance.rs` (Marley crate):** `socket_in` builds
  `marley-<channel>.sock`; `socket_path` public; the module doc.
- **`crates/paths/src/paths.rs` (Zed crate, Marley's hunk):** the guard tests' comment.
- **`script/install-marley`, `script/e2e.sh` (Marley-owned):** `StartupWMClass`; the runner's
  `class` and a `legacy_class` matched alongside it.
- **Docs:** the guide, the in-app guide, the architecture note, CHANGELOG.
- **Ledger rows:** `crates/release_channel/src/lib.rs` (new), `crates/zed/src/zed/open_listener.rs`
  (check for an existing row first), `crates/paths/src/paths.rs` (extend).

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| 001 | Starts Marley on a scratch project under sway | 760-01; `swaymsg -t get_tree` gives the app id |
| 002 | `zed: about` from the palette | 760-02 |
| 003 | Lists the profile; a second launch with a new folder | 760-03; the socket listing; the launch's stdout |
| 004 | Writes a setting, quits, starts again | 760-04; the rail shows the project, the setting holds |
| 005 | — | The review; the next `just install` writes it |
| 006 | — | Every run of this scenario finds the window by the new class |

### Risks
- **A Marley installed before this change** listens on `zed-dev.sock`; a new launch beside it does
  not hand off and two Marleys share the data directory (D4). Reinstall, then quit and restart.
- **A window rule on the old class** stops matching. None on this box (`~/.config/hypr`).
- **Language servers** now see `client_info.name = "Marley"` instead of "Zed Dev"; none in the
  tree branches on it.

## Phase 2 — Code (2026-10-11)
- **Built:** the ledger row for `crates/release_channel/src/lib.rs` and the `paths.rs` and
  `open_listener.rs` rows extended, first. `release_channel`: the `Dev` arms of `display_name`
  (`Marley`) and `app_id` (`com.ignibyte.Marley`). `open_listener.rs`: the socket path from
  `marley_workbench::single_instance::socket_path` (its `RELEASE_CHANNEL_NAME` import gone).
  `single_instance.rs`: `socket_path` public and `#[must_use]`, `socket_in` builds
  `marley-<channel>.sock`, the module doc. `paths.rs`: the guard tests' comment.
  `script/install-marley`: `StartupWMClass=com.ignibyte.Marley`. `script/e2e.sh`: `class` is the
  new id, `legacy_class` the old one, matched beside it in the Hyprland and sway lookups and given
  the same window rule. The guide, the in-app guide and the architecture note.
- **Deviations from the design:** none.
- **The review:** both senders (`hand_off`, `hand_verb`) and the listener read `socket_path`, so
  they cannot name different sockets; `socket_fits` measures the new, three bytes longer name;
  Zed's `cli` still builds `zed-<channel>.sock`, and Marley ships no `cli` (Out).
- **Clippy:** `must_use_candidate` on the now public `socket_path`. Clean under the gate's scope.
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/760-marley-calls-itself-marley.sh`, under `compositor sway`. Exit
  0, six checks passing. Shots read:
  - `760-01-window`: Marley on `first`; sway's tree lists `com.ignibyte.Marley` and no
    `dev.zed.Zed-Dev` (REQ-001).
  - `760-02-about`: the About prompt, "Marley 1.25.0 (debug)", the commit and the version
    (REQ-002).
  - `760-03-socket`: the profile holds `marley-dev.sock` (a socket) and no `zed-dev.sock`; the
    second launch printed "Marley is already running" and `second` is a group in the same window
    (REQ-003).
  - `760-04-restarted`: quit and started again, the rail holds Default, `second` with its
    terminal, and `first`; `db/0-dev` is where it was (REQ-004).
  - The runner found every window by the new class (REQ-006).
- **Fix on the way:** the second folder's trust prompt took the keys, so the quit went to it; the
  scenario trusts it before the shot.
- **What no scenario shows:** the desktop entry (REQ-005), written by the next `just install`;
  X11's `WM_CLASS` (Marley runs on Wayland here), the same value through gpui's X11 window.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #760); the guide's class line; the in-app guide's note;
  `marley_workbench.md`'s socket; the three ledger rows describe what shipped.
- **Knowledge appended:**
  AD-claude-760-marleys-app-id-name-and-socket-are-its-own-on-the-dev-channel-001.
- **Brain:** consultation 854b755eefb74017868cf8b900be7e32 closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed. At the next
  install, Chad quits the running Marley before starting the new one (the socket's name moved).
