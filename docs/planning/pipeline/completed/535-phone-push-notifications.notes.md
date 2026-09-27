# Agent events pushed to the phone through ntfy on the dev box — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-535-phone-push-notifications.md
- **Pipeline spec:** 535-phone-push-notifications.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25, on the phone: a full native app eventually, and "we could
  potentially have the app drive it or a small native rust relayer". This ticket is the first step
  only: agent events (needs input, finished, failed) pushed to the phone through a self-hosted ntfy
  on the dev box. The phone endpoint and a Rust relay are recorded below as later slices.
- **Classification / tier:** feature, prong 2 (remote control). Rust in `marley_workbench` and
  `marley_agent` (Marley crates) and one Marley file in a Zed crate
  (`crates/settings_content/src/marley.rs`). Nothing new draws in Marley's window but a toast.
  Deliberate in the backlog: it waits for Chad to start the phone path, and it needs #519.
- **Recall (§18.3):**
  - AD-claude-478 and AD-claude-482: OSC 9 and 777 become `Event::MarleyNotification`; the
    plugin's `Notification` and `Stop` hooks answer with a `terminalSequence`, gated on
    `TERM_PROGRAM=zed`. Claude Code's hooks reference now documents `terminalSequence` (read
    2026-09-25), which AD-claude-482 said it did not.
  - L-claude-478-omarchys-notifications-go-to-quickshell-001: a banner Marley posts goes to
    Omarchy's Quickshell. This scenario raises banners, so it runs Marley on a private bus.
  - L-claude-482-claude-codes-print-mode-drops-a-hooks-terminal-sequence-001: `claude -p` drops a
    hook's sequence; the fake `claude` prints the frames itself.
  - L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001: the shape of blocking
    work off the main thread in a Marley crate.
  - #514's scenario proves a network behavior with `server_url` at a dead local port; this one
    uses a live fake server on loopback and a TEST-NET address for the refused case.
  - L-claude-502-a-dev-channel-panic-reaches-stderr-only-001 and #502's D3: Marley stays on the
    `dev` channel, which is why Zed's credential store is a plain file here.
  - Brain: not consulted at drafting (the overnight drafters work read-only). The promotion runs
    `brain_ask`.
- **Discovery:**
  - #519's spec (drafted the same night, `519-claude-code-events-in-the-rail.spec.md`): the
    `marley-event` frame, `marley_agent::claude_events` for the fold, `agent_events` for the seats,
    no bell and no banner for a frame, and `notify.sh`'s fixed sentences kept.
  - `crates/marley_workbench/src/notifications.rs`: `init` (lines 28-45) subscribes every
    `TerminalView` to its terminal's `Event::MarleyNotification`; `notify` (49-72) returns early
    when the window is active and the view holds focus (line 56), else posts through
    `show_system_notification` with a tag per terminal; `show_sender` (76-90) shows the terminal
    on a click.
  - `crates/marley_workbench/claude_plugin/marley/hooks/hooks.json`: `Notification`
    (`permission_prompt`, `idle_prompt`) and `Stop` run `notify.sh`, whose fixed sentences
    (lines 6-11) and `TERM_PROGRAM` gate (line 5) #519 keeps; its own frames come from a new
    `hooks/event.py`.
  - `crates/terminal/src/terminal.rs:715-725`: `insert_zed_terminal_env` sets `TERM_PROGRAM=zed`
    for local and remote terminals; line 741 declares `Event::MarleyNotification`, emitted at
    1806.
  - `crates/marley_dcs/src/notification.rs`: `MAX_NOTIFICATION` is 4 KiB (line 11); `parse`
    (92-118) reads OSC 9 and `777;notify`.
  - `crates/http_client/src/http_client.rs:123-133`: `HttpClient::send(http::Request<AsyncBody>)`;
    `crates/gpui/src/app.rs:1740` hands out the app's client.
  - `crates/settings_content/src/marley.rs`: `MarleySettingsContent { layout }`, a Marley file with
    its touchpoint row (`docs/marley/zed-touchpoints.md`, the row for this path);
    `crates/marley_workbench/src/marley_workbench.rs:160-179`: `MarleySettings::from_settings`.
    #515's spec adds the Marley page at `crates/settings_ui/src/marley_page.rs`, where later
    settings add their sections.
  - `crates/zed_credentials_provider/src/zed_credentials_provider.rs:45-66`: on
    `ReleaseChannel::Dev` the development provider, "a local file", unless
    `ZED_DEVELOPMENT_USE_KEYCHAIN` is set; `crates/gpui_linux/src/linux/platform.rs:53`:
    `KEYRING_LABEL` is `zed-github-account`.
  - `crates/gpui_linux/src/linux/system_notifications.rs`: banners go through `notify-rust` on the
    session bus (lines 38-87), one thread per banner waiting for its action; `dismiss` is a no-op
    on Linux (89-93).
  - `crates/marley_mcp/src/transport.rs:118` binds `127.0.0.1:0`, and
    `crates/marley_mcp/src/permission.rs:14` is `GrantTable`: the seams slice 4 below builds on.
  - Orca (MIT, `/srv/stacks/orca-refs/orca`): `src/main/runtime/push/push-dispatcher.ts` (title
    80, body 180 characters; `needs-input` or `finished`); `src/shared/mobile-e2ee-v2-contract.ts`
    (281 lines), `src/shared/mobile-e2ee-v2-framing.ts` (141), `src/shared/mobile-e2ee-v2-fixtures.ts`
    (47), `mobile/src/transport/mobile-e2ee-v2-key-schedule.ts` (48); `LICENSE` is MIT, "Copyright
    (c) 2026 Lovecast Inc.".
  - The dev box: `tailscale` and `tmux` installed, no ntfy package. A tailnet peer can reach a
    port a machine listens on unless a firewall rule stops it, so ntfy binds loopback and only
    `tailscale serve` publishes it.
- **Decisions:** D1 to D7 in the spec.

### Design
- **Approach.** #519's `marley_workbench::agent_events` keeps a seat per terminal, folded from the
  `marley-event` frames (OSC 777 under that reserved title, a base64 JSON summary with `event`,
  `session_id`, `prompt_id`, `tool`, `preview`, `message`, `error` and more). `push.rs` follows
  those seats and acts on three changes: a seat that starts waiting, a `Stop` that ends a working
  turn, a `StopFailure`. For each it asks the same gate `notify` asks (factored into one function
  both call). If
  `MarleySettings::push` is set and the gate passes, it builds the request: the line from
  `marley_agent::event_line(project, kind, event)` (new, pure; #538's banner title is the same
  string), `Title: Marley`, the priority and the tag, and the bearer from `token_file` when named.
  It checks the URL's host is loopback and the token file's mode is 0600 before anything else.
  The post runs as a background task; its result comes back to the main thread, where a failure
  logs a warning and, the first time in a spell, shows a workspace toast ("Marley could not reach
  ntfy at <url>"). A later success ends the spell.
- **Cooldown.** Until #538 exists, the push keeps its own map of the last post per project and
  skips a post within 5 seconds of the last (Orca's `NOTIFICATION_COOLDOWN_MS`). When #538 lands,
  both read one cooldown.
- **Project name.** The rail's project name for the terminal's group is the natural source, but
  it is disambiguated per window ("marley_ide" versus "a/marley_ide"). The line takes the terminal's
  workspace root's last component, cleaned of control characters, as `notify.sh` builds it, so the
  phone reads the same name the desktop banner reads.
- **File manifest.**
  - `crates/marley_workbench/src/push.rs` (new, Marley crate); `crates/marley_workbench/src/marley_workbench.rs`
    (`mod push`, `push::init`, the `push` field on `MarleySettings`);
    `crates/marley_workbench/src/notifications.rs` (the gate as a shared function);
    `crates/marley_workbench/Cargo.toml` (`http_client`, a workspace crate).
  - `crates/marley_agent/src/marley_agent.rs` (Marley crate): `event_line`, the agent's short name
    ("Claude") and the event words.
  - `crates/settings_content/src/marley.rs` (Zed crate path, Marley's file): `push:
    Option<MarleyPushSettingsContent { url, topic, token_file }>`. Its touchpoint row is updated.
  - After #515: the Push section in `crates/settings_ui/src/marley_page.rs` (Zed crate path,
    #515's file) and its row.
  - `script/e2e/535-phone-push-notifications.sh` and its fakes (Test).
- **Ledger rows.** The touchpoint row for `crates/settings_content/src/marley.rs` gains `push`.
  At Complete: a lesson on ntfy's iOS delivery path, and the ops handbook page for the ntfy stack.

### E2E plan
Shared fixtures: a scratch repository opened as the project; a private session bus
(`dbus-daemon --session --fork --print-address`, exported as `DBUS_SESSION_BUS_ADDRESS` in `setup`
before the launch) with a fake `org.freedesktop.Notifications` in Python GI that logs each
`Notify` summary and body; the fake ntfy (`python3 -m http.server`-style handler on a free loopback
port, logging each request); a fake `claude` first on the PATH that prints #519's frames for the
event named in a trigger file; the profile copy's settings with `marley.push` pointing at the fake
server. `teardown` stops the bus and both fakes by their pids.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-007 | Launch without `marley.push`; start the fake agent in terminal A, focus a second terminal B; trigger `Stop`. Then write the setting (Zed reloads it) | The fake server's log is empty |
| REQ-001 | Trigger a permission request in A while B holds focus | `535-01-needs-input`; one `POST /<topic>`, line `repo: Claude needs input` |
| REQ-002 | Trigger `Stop` in A (after the cooldown) | `535-02-finished`; one request, `repo: Claude finished` |
| REQ-003 | Trigger `StopFailure` (`rate_limit`) in A | `535-03-failed`; one request, `repo: Claude failed` |
| REQ-004 | The three requests above | The log's headers and bodies in full: `Title`, `Priority`, `Tags`, the line, nothing else |
| REQ-005 | Focus A and trigger `Stop` | `535-04-focused`; no request |
| REQ-006 | In B, a plain shell: `printf '\e]9;built\a\e]777;notify;make;done\a'` | No request; the private bus's log shows the desktop banners |
| REQ-008 | `token_file` at 0600 with a dummy token (`tk_e2e`), one event; then the file at 0644, one event | The first request's `Authorization` header; no second request; the warning in `535.marley.log` |
| REQ-009 | `url` set to `http://192.0.2.1:9`, one event | No request; Marley's log names the refused host |
| REQ-010 | Stop the fake server, trigger an event | `535-05-no-server` (the toast); the bus's log still has the banner |

Not reachable by a scenario: the phone. Test sets up ntfy on the box (a container or the release
binary, bound to `127.0.0.1`), `tailscale serve --https` in front, the ntfy app on Chad's phone
subscribed over the tailnet, and one real Claude Code permission prompt; the notes record what the
lock screen shows and ntfy's log of the poll request to ntfy.sh.

### Risks
- **ntfy.sh is in the iOS path.** A self-hosted server's iOS pushes ride ntfy.sh's poll relay;
  if ntfy.sh is down or rate-limits, iOS notifications come late (the app fetches on its own
  schedule). Android is unaffected: its app holds its own connection to a self-hosted server.
- **The phone must be on the tailnet.** The Tailscale app on the phone, ideally always on, and
  HTTPS certificates enabled for the tailnet before `tailscale serve --https` works (report 04
  §3.2 item 3). The Test phase checks the phone is on it first.
- **The lock screen shows the line.** Project names appear on the phone's lock screen; iOS's
  "Show Previews: When Unlocked" hides them. The line carries nothing else by D2.
- **#519's event shapes.** The three kinds and the project are all this ticket reads; if #519
  names them differently, only the mapping in `push.rs` changes.
- **Zed's `proxy` setting.** Zed's HTTP client follows it; if Chad ever sets one, check that a
  loopback post still reaches ntfy.
- **The token file.** A token in a file is only as safe as the home directory; the mode check
  stops the common mistake. A token that leaks can publish to the topic (ntfy's tokens carry the
  user's rights, docs.ntfy.sh/config), so Marley's user gets write on the one topic only.

### The phone path after this slice (design, not built here)
1. **Dismiss and away.** ntfy's `X-Sequence-ID` lets a later request replace or clear a
   notification (docs.ntfy.sh/publish); a push is given the terminal's id and turn as its sequence
   id, and the event handled at the desk clears it (Orca: `dismiss` in `push-dispatcher.ts`).
   "Away" needs an idle or lock signal that gpui does not expose (the Wayland idle-notify protocol,
   or the session's lock state); a small watcher in the workbench, pushing only while away when a
   setting asks.
2. **Approvals from the phone.** After #508: the push's `Click` URL opens the approval, answered
   through the `PermissionRequest` hook's decision (report 04 §3.2 item 2), with Orca's prompt
   identity and revision so a late tap cannot answer a newer prompt.
3. **A phone endpoint.** A new Marley crate (working name `marley_remote_control`) serving HTTPS and
   a WebSocket on `127.0.0.1`, published by `tailscale serve`, which gives the phone a real
   certificate. Pairing by a QR in a Marley modal carrying the URL, a per-device token and, if
   app-layer encryption is wanted, the desktop's public key. A 0600 device registry with Orca's
   `DeviceRegistry` rules (`src/main/runtime/device-registry.ts`: a pending token handed out again
   until used, rotate, revoke that closes live sockets, refuse to overwrite a file it could not
   read). Scopes from `marley_mcp`'s `GrantTable` with new write classes (`approval.answer`,
   `agent.prompt`); raw terminal input never by default (report 04 §3.2 item 3).
4. **The native app.** Chad's decision: a full native app, eventually. On the tailnet it talks to
   slice 3's endpoint directly. Off the tailnet, a small Rust relay splices the phone's socket to
   the desktop's and reads nothing: Orca's splice shape (`cloud/apps/relay/src/splice-forwarder.ts`,
   report 04 §2.8: forward frames unchanged, pause the sender at 256 KiB buffered, resume under
   64 KiB, close both after 10 seconds wedged). Encryption end to end in Orca's E2EE v2 layout
   (`src/shared/mobile-e2ee-v2-contract.ts` and `-framing.ts`,
   `mobile/src/transport/mobile-e2ee-v2-key-schedule.ts`): X25519 between the phone's per-socket
   key and the desktop's static key pinned by the QR; HKDF-SHA256 with salt over the label and both
   nonces and info over the label and the transcript hash, giving one key per direction and a
   session id; each frame sealed under a nonce built from the session id, the direction, the
   payload kind and a 64-bit counter; the receiver accepts only the next counter, so replayed,
   reordered or reflected frames fail; the auth message echoes the transcript hash. What the relay
   sees: both addresses, frame sizes and timing. It cannot read a frame, pose as the desktop (its
   key is pinned) or pose as the phone (the device token travels inside the encryption). Skip
   Orca's v1: its frames can be replayed inside a socket (report 04 §4).
   Crates: `hkdf` 0.12 and `sha2` 0.10 are already in `Cargo.lock`, and `ring` 0.17 has X25519
   and ChaCha20-Poly1305. Orca seals with XSalsa20-Poly1305 (`secretbox`); a port that should match
   Orca's fixtures byte for byte (`src/shared/mobile-e2ee-v2-fixtures.ts`) adds `x25519-dalek` and
   `crypto_secretbox`. Code taken from Orca keeps its MIT notice and names the Orca file in its
   header (docs/orca_architecture/README.md).
5. **Push once the app exists.** Stay on ntfy (the app subscribes as ntfy's does, or through
   UnifiedPush on Android), or send through APNs directly, which needs an Apple developer account
   and a key of Chad's. What leaves the box stays D2's line unless Chad widens it.

## Chad's answer, 2026-09-26
- "lets do the planning phases for this": promote after #547, through `/pipeline:plan`.

### Promotion (2026-09-26): what changed from the queued design
- **Recall at promotion:** L-claude-478-omarchys-notifications-go-to-quickshell-001 (the private
  bus stays: this scenario raises banners); L-claude-547-marleys-path-is-the-login-shells-001 and
  PR-claude-name-the-fakes-the-app-runs-001 (the stand-in `claude` runs inside a terminal, so the
  scenario's `.bashrc` puts it first; Marley itself runs nothing here but its HTTP client);
  F-claude-547-a-timer-armed-at-the-first-event-fired-before-the-last-was-a-minute-old-001 (the
  cooldown is measured from the last post, per project). Brain consultation
  a7148ae197fb4de586e54d0eb6666ac0: nothing on this seam.
- **Seams re-verified at `ca19a9dd15`** (#519 and #547 have landed):
  - `crates/marley_workbench/src/agent_events.rs`: `AgentEvents` (one `FleetSnapshot`, a seat per
    terminal view, keyed by the view's id), `on_frame` (drops a frame unless Claude Code is the
    foreground, decodes, folds, applies), `end`, `forget`, `next_quiet_change`. It emits nothing
    per frame, so the design changes: `on_frame` returns the seat's state before and after, and
    the caller acts on it.
  - `crates/marley_workbench/src/notifications.rs:38-51`: the subscription that routes a
    `marley-event` title to `on_frame` has the view and the window, which the focus gate needs;
    it calls `push::on_change(view, before, after, window, cx)` after `on_frame`. The gate
    (`:66`, the active window's focused terminal) becomes one function, `looking_at`, that
    `notify` and `push` both call.
  - The kinds from `marley_fleet::State`: needs input is a change into `Waiting`; finished is a
    change into `Idle` from `Working` or `Waiting`; failed is a change into `Error`.
  - `crates/settings_content/src/marley.rs` now holds `layout`, `redact_secrets_for_agents`,
    `redaction_patterns` and `no_update_after_minutes`; `push` joins them. #515's page exists
    (`crates/settings_ui/src/marley_page.rs`, sections Layout, Agents, Privacy), and `settings_ui`
    renders `String` fields (`settings_ui.rs:554`), so the Push section is in this slice.
  - The plugin (1.2.0) runs `hooks/event.py` for every event and `hooks/notify.sh` for the
    `Notification` hook and `Stop`, so a real permission prompt raises both a frame (the rail, and
    now the push) and notify.sh's banner (the desktop). REQ-010's "still show the desktop
    notification" is that banner, on the private bus.
  - `crates/http_client/src/http_client.rs` and `App::http_client` unchanged.
- **The scenario's stand-in:** the #519/#547 shape (a Python `claude` first on the terminal's PATH
  that runs the real hooks at each Enter) replaces the trigger-file fake; it also runs
  `notify.sh` where Claude Code does.
- **Needs Chad at Test:** the real phone. ntfy is not installed on the box and `tailscale serve
  --https` publishes a port on the tailnet; both are changes to his machine and network, so Test
  asks before setting them up, and records what the lock screen shows.

## Phase 2 — Code
- **Built.**
  - `marley_agent`: `AgentKind::short_name` ("Claude"); `TurnEvent` (`NeedsInput`, `Finished`,
    `Failed`) with `of_change(before, after)` over `marley_fleet::State` (a change into
    `Waiting`; into `Idle` from `Working` or `Waiting`; into `Error`; a wait or a failure that
    repeats is none) and `words`; `event_line(project, kind, event)`, the one line, with the
    project's control characters dropped. `claude_events::CWD_LABEL` names the seat's `cwd`.
  - `agent_events::on_frame` returns the seat's state before the frame and the seat after it.
  - `notifications`: `looking_at(view, window, cx)`, the gate `notify` and `push` share; the
    `marley-event` branch hands `on_frame`'s change to `push::on_change`.
  - `push.rs` (new): `on_change` (the kind; `marley.push` set; not looking at it; `target_url`,
    which refuses a host that is not loopback and a topic ntfy would not take; a 5-second
    cooldown per project, from the last post; the post off the main thread); `post` (the token
    read from its file only there, refused when group or others can read it; `Title`,
    `Priority` 4 or 3, `Tags` `question`, `white_check_mark` or `x`, the line as the body);
    `report` (a refusal is logged; a failure is logged and shows one toast until a post
    succeeds). The project is the last folder of the seat's `cwd`, as notify.sh names it.
  - Settings: `MarleySettingsContent::push` (`MarleyPushSettingsContent { url, topic,
    token_file }`, Zed's `settings_content`, its row widened first), `MarleySettings::push`
    (`PushSettings`, set only when a URL and a topic are), and the Marley page's Push section
    (three text fields; `settings_ui`, its row widened first). `http_client` joins the
    workbench's dependencies; it re-exports `Url` and `Host`.
- **Deviations.**
  - A refused push (the token file's mode, a missing token) is logged only; a toast is kept for
    a server that does not answer (REQ-010), so a setting's mistake does not toast at every
    event.
  - The project's name comes from the seat's `cwd` label (where Claude Code started), not the
    workspace root, which needs no entity lookup and matches notify.sh's `CLAUDE_PROJECT_DIR`.
- **Review of the diff.**
  - The token file's read and mode check run inside the background post (Zed's
    `blocking_io_on_foreground` lint, as #547 learned); the URL check is pure and runs first.
  - Re-entrancy: `on_change` runs in the terminal view's subscription and reads only the
    settings, the focus and globals.
  - Nothing of the agent's goes in the push: the line is built from the folder name, the agent's
    short name and the event's words (D2).
- **Checks.** `cargo check --all-targets`, `cargo clippy --all-targets --all-features -D
  warnings` and `cargo fmt` over `marley_agent`, `marley_workbench`, `settings_content` and
  `settings_ui`: clean.

## Phase 3 — Test
- **The scenario:** `script/e2e/535-phone-push-notifications.sh` (`compositor sway`). Setup starts
  a monitor on the user's own bus (for a banner that must not reach it), a private session bus
  (`dbus-daemon --session --fork`) with a notification server of the scenario's own (dbus-python)
  that logs each banner, a fake ntfy on a free loopback port that logs each request as a JSON
  line, and a stand-in `claude` stepped through a FIFO, since keys cannot reach a terminal that
  is not focused: at each step it runs the plugin's real `event.py` for every event and
  `notify.sh` where Claude Code runs it. Marley's window moves off the active workspace
  (`sway_msg workspace 2`) for each event that should push and comes back for its shot.
- **A red of the scenario's own:** the first two runs printed nothing and exited 1. The wait for
  the fake ntfy's port ran `grep | cut` in an assignment at setup's top level, where `set -e` and
  `pipefail` end the run on the first grep that finds nothing yet (browser-fixture.sh's
  `serve_site` does the same inside a command substitution, where `set -e` does not reach). Fixed
  with `|| true`.
- **The run, every check passing** (the requests quoted from the fake's log):
  - REQ-006: `sleep 3; printf` of an OSC 9 and an OSC 777 in a plain shell, the window away: no
    request, and both banners on the private bus (`|built`, `|make|done`).
  - REQ-007: with `marley.push` removed, a turn's end pushes nothing.
  - REQ-001: `POST /marley-e2e`, `title: Marley`, `priority: 4`, `tags: question`, body
    `repo: Claude needs input` (24 bytes).
  - REQ-002: `priority: 3`, `tags: white_check_mark`, `repo: Claude finished`.
  - REQ-003: `priority: 4`, `tags: x`, `repo: Claude failed`.
  - REQ-005: with the window active and the terminal focused, a turn's end pushes nothing.
  - REQ-004: every request's headers are `title`, `priority`, `tags`, `content-type`, `accept`,
    `user-agent` (Zed's, with the build), `host`, `content-length`, and `authorization` where set;
    each body is the line alone.
  - REQ-008: a 0600 token file sends `authorization: Bearer tk_e2e`; at 0644 nothing is posted
    and Marley's log says the file "can be read by others".
  - REQ-009: `http://192.0.2.1:9` posts nothing, and Marley's log says it "is not a server on
    this machine".
  - REQ-010: with the fake killed, the "repo finished" banners went from 5 to 6 (the desktop
    notification still shown), and no banner reached the user's own bus.
- **The shots, read:**
  - `535-01-needs-input`: the row `waiting · Add a README`, then `Permission for Write: README…`.
  - `535-02-finished`: `idle · Add a README`, then `Added README.md.`
  - `535-03-failed`: `failed · Run the tests`, then `rate_limit`.
  - `535-04-focused`: `idle · Check the lints`, then `No lints.` (the event whose push was held).
  - `535-05-no-server`: the toast "Marley could not push to your phone: the server at
    http://127.0.0.1:46499 did not take it (error sending request for url
    (http://127.0.0.1:46499/marley-e2e))."; the row `idle · Write the changelog`.
  - Each row carries the bell dot: notify.sh's banners mark the terminal as #478 does.
- **Not reached by a scenario:** the phone. ntfy is not installed on the box, and `tailscale serve
  --https` would publish a port on the tailnet; both wait for Chad's yes (asked 2026-09-26, with
  which phone and whether Tailscale runs on it), and the notes will record the lock screen then.
- **Golden set:** 535 joins it; `just regress` on the debug build: all 13 pass (535 in 112 s).
- **Gate:** `script/gates.sh --diff` red once on gate:14 (the `push` module's doc linked the
  crate-private `on_change`), fixed at the source; then `GATE GREEN [diff]`, 16 passed.
- **Verdict:** Phase 3 PASS for every criterion; the real phone stays a hand check for when Chad
  says yes to ntfy on the box.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added); `docs/marley_architecture/marley_workbench.md` (the push)
  and `marley_agent.md` (`TurnEvent`, `event_line`, `short_name`); the two Zed rows describe what
  shipped. The ops handbook's ntfy page waits for the real setup, which waits for Chad's yes.
- **Knowledge:** L-claude-535-a-setup-wait-loop-under-set-e-exits-without-a-word-001,
  AD-claude-535-agent-events-reach-the-phone-as-one-line-through-ntfy-on-the-box-001.
- **Brain:** consultation a7148ae197fb4de586e54d0eb6666ac0 closed with a decision.
- **Ticket:** closed; the phone's hand check and the handbook page are its follow-up, noted in
  the ticket.
