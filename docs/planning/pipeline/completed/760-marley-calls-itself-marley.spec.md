---
pipeline_id: c246d333-f639-4e32-93df-0ca4527c9e37
ticket: docs/planning/tickets/closed/TICKET-760-marley-calls-itself-marley.md
status: Phase 4 — Complete PASS
title: Marley calls itself Marley - its app id, its name and its socket
type: chore
slice: packaging, the release identity (TICKET-445), slice 2 of 5
references:
  - docs/planning/pipeline/completed/445-marley-release-identity.spec.md
  - docs/planning/pipeline/completed/437-marley-app-identity.spec.md
  - docs/planning/pipeline/completed/513-one-marley-per-data-dir.spec.md
---

## Title
Marley's windows still tell the desktop they are `dev.zed.Zed-Dev`, its About window and window
titles say "Zed Dev", and its single-instance socket is `zed-dev.sock`. This slice gives Marley its
own app id, `com.ignibyte.Marley` (Ignibyte's domain, reversed, as Zed's is `dev.zed.Zed`), its own
name, "Marley", and its own socket, `marley-dev.sock`, while it stays on Zed's `dev` channel, so the
database (`db/0-dev`), the `"dev"` settings block and the credentials file do not move. It is the
second of #445's five slices (#445's spec, Out item 1).

## Scope
### In
- **The app id.** `ReleaseChannel::app_id()` on `dev` is `com.ignibyte.Marley`: every window's
  Wayland app id and X11 `WM_CLASS` (the workspace, About, Settings, Audio Test, Copilot sign-in,
  call and agent popup windows; `zed.rs:366`, `:1730`, `settings_ui.rs:918`,
  `audio_test_window.rs:276`, `sign_in.rs:65`, `collab_ui.rs:45`, `agent_notification.rs:55`).
- **The name.** `ReleaseChannel::display_name()` on `dev` is `Marley`: the About window's line,
  `${appName}` in a window title template, the update notification, System Specs, the LSP and ACP
  client titles. Nothing persists or keys on it.
- **The socket.** `<data dir>/marley-dev.sock` from one function, `single_instance::socket_in`,
  which Zed's listener (`open_listener.rs:431`) now calls instead of building its own name.
- **The desktop entry's** `StartupWMClass=com.ignibyte.Marley` (`script/install-marley:187`).
- **The e2e runner** finds the run's windows by the new class, and also counts a window of the old
  class, so a Marley installed before this change is still seen (the Hyprland guard, #743's run of
  the installed build).
- Docs: the guide's class line and the in-app guide's, the architecture note's socket, the
  `paths.rs` guard test's comment.

### Out (explicitly deferred)
- The desktop entry's file name (`com.ignibyte.Marley.desktop`), the `marley://` scheme and the
  `zed://` links: slice 4, #762.
- The keyring label: slice 3, #761. The updater: slice 5, #763.
- Zed's `cli` crate, which builds the socket name itself (`cli/src/main.rs:948-957`): Marley ships
  no `cli`. Windows' `app_identifier()`, the macOS bundle and the Flatpak and snap manifests:
  Marley ships on Linux only.
- The About window's own title, "About Zed", and the window title template's default.

## Reference (§20)
Upstream Zed, the `release_channel` crate: each channel names its app id (`dev.zed.Zed`,
`dev.zed.Zed-Dev`, …) and its display name (`Zed`, `Zed Dev`, …), and gpui hands the app id to the
compositor as the Wayland app id and the X11 class (`gpui_linux` `wayland/window.rs:596`,
`x11/window.rs:1597-1610`). Marley keeps the mechanism and gives the `dev` arm its own values, as a
packager of Zed would. The socket keeps Zed's per-channel form, `<name>-<channel>.sock`, under the
data directory (`open_listener.rs:427-457`).

### Prior art
- **The code we ship:** `release_channel::ReleaseChannel::{app_id, display_name}`
  (`lib.rs:206-235`); their seven app-id callers, all `WindowOptions.app_id`; the display name's
  callers, all text (`zed.rs:1559`, `workspace.rs:6935-6939`, `auto_update_ui.rs:312`,
  `system_specs.rs:63,95`, `telemetry.rs:694-696`, `lsp.rs:1152-1156`, `acp.rs:830-834`), none a
  path, a key or a label; `single_instance.rs:59-66` and `open_listener.rs:431`, which build the
  same name twice; `paths.rs:640-663`, whose tests assert the directories and the `dev` channel,
  not the id.
- **Behavior maps:** none; the ids are packaging.
- **Published material:** the freedesktop desktop entry spec (`StartupWMClass`), xdg-shell's
  `set_app_id`, reverse-DNS application ids.

## UI proof
`script/e2e/760-marley-calls-itself-marley.sh`, under `compositor sway`, on a scratch project.
- `760-01-window`: Marley's window, with sway's tree giving its app id `com.ignibyte.Marley`.
- `760-02-about`: the About window (`zed: about`), its line starting "Marley".
- `760-03-socket`: the run's profile holds `marley-dev.sock` and no `zed-dev.sock`; a second launch
  on the scratch folder prints "Marley is already running" and the folder opens in the running
  window.
- `760-04-restarted`: Marley quit and started again: the same project in the rail, and the setting
  the scenario wrote still in force.

## Locked-In Decisions
- **D1 — `com.ignibyte.Marley`.** Ignibyte's domain is `ignibyte.com` (the ops handbook); reversed,
  as Zed's `dev.zed.Zed` reverses `zed.dev`. Rejected: `marley` (no namespace),
  `dev.zed.Marley` (Zed's domain).
- **D2 — Stay on `dev`, change the `dev` arm.** The channel keys the database, the settings block
  and the credentials file (AD-claude-437); changing the arm moves none of them. Rejected: a
  `Marley` channel (migrations of all four, and every `match` on the channel).
- **D3 — `marley-<channel>.sock`, built once.** Zed's form with Marley's name; `open_listener.rs`
  calls `single_instance::socket_in`, so the listener and the hand-off cannot disagree. Rejected:
  `marley.sock` (drops the channel a debug build may override), a second `format!`.
- **D4 — No shim for the old socket.** A Marley installed before this change listens on
  `zed-dev.sock` until it restarts, and a new launch beside it would not hand off. The install
  already says to quit a running Marley first, and Chad is told at each reinstall. Rejected: a
  check for the old socket at start (a Zed hunk in `main.rs` for one restart).
- **D5 — The runner counts both classes.** The new class is the run's; the old one keeps the
  Hyprland guard seeing a Marley that was installed before.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Marley's windows shall carry the app id and X11 class `com.ignibyte.Marley`. | Shot 760-01 and sway's tree; the review of the seven callers |
| REQ-002 | The About window shall name the app "Marley". | Shot 760-02 |
| REQ-003 | Marley shall listen on `<data dir>/marley-dev.sock`, and a second launch on the same data directory shall hand its paths to the running one. | Shot 760-03 and its checks |
| REQ-004 | WHEN Marley restarts, it shall keep its projects, groups and settings (the database, the `"dev"` block and the credentials file stay where they are). | Shot 760-04 |
| REQ-005 | The installed desktop entry shall name `StartupWMClass=com.ignibyte.Marley`. | The review of `script/install-marley`; the next install |
| REQ-006 | The e2e runner shall find the run's windows by the new class and count a window of either class as a Marley that is open. | Every scenario run on the new build; the review of `script/e2e.sh` |
| REQ-007 | The change shall pass `script/gates.sh --diff`. | The gate |
