---
pipeline_id: b860795e-c6f8-4c20-acec-4cae3d2dd2e8
ticket: docs/planning/tickets/open/TICKET-535-phone-push-notifications.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Agent events pushed to the phone through ntfy on the dev box"
type: feature
slice: prong 2, remote control; the phone path's first slice (report 04 §3.2 item 1)
references: [docs/planning/tickets/open/TICKET-519-claude-code-events-in-the-rail.md, docs/orca_architecture/04-remote-control-and-mobile.md, docs/orca_architecture/01-agents-and-sessions.md, docs/planning/pipeline/completed/514-telemetry-off-by-default.spec.md]
---

## Title
When a Claude Code agent in one of Marley's terminals needs input, finishes or fails, Marley posts
a one-line push to an ntfy server on the dev box, bound to loopback, and the ntfy app on Chad's
phone shows it over the tailnet. The push names the project, the agent and the event, and nothing
the agent wrote leaves the desktop. This is the first slice of the phone path; the phone endpoint,
the native app and a Rust relay are designed in the notes as later slices.

## Scope
### In
- `crates/marley_workbench/src/push.rs` (new, Marley-owned): follows the seats #519's
  `marley_workbench::agent_events` keeps, one per terminal, and for three changes posts to ntfy
  with Zed's HTTP client (`App::http_client`):
  - needs input: the seat starts waiting (a `PermissionRequest`, or a `PreToolUse` of
    `AskUserQuestion`);
  - finished: a `Stop` ends a working turn;
  - failed: a `StopFailure`.
- The push: `POST <url>/<topic>` whose body is one line, `<project>: Claude <event>` (`needs
  input`, `finished`, `failed`), with `Title: Marley`, `Priority: 4` for needs input and failed and
  `3` for finished, and one `Tags` word per kind. `<project>` is the project folder's last
  component with control characters removed, as `notify.sh` builds it today. The line comes from
  one pure function in `marley_agent` that #538's banner title also uses.
- The desktop banner's gate: nothing is pushed for the focused terminal of the active window
  (`crates/marley_workbench/src/notifications.rs:56`). A 5-second cooldown per project makes a
  burst one push; once #538 has its banner cooldown, the two share it.
- Settings, a `push` block in `MarleySettingsContent` (`crates/settings_content/src/marley.rs`, a
  Marley file in a Zed crate) read by `MarleySettings`: `url` (the ntfy server, loopback only),
  `topic`, and `token_file` (optional: a file holding an ntfy access token). Unset means no push.
  A Push section on the Marley settings page, when #515 has added the page.
- The token is read from its file at each post and sent as `Authorization: Bearer`; it is never
  logged and never kept in settings.
- A failed post logs a warning and, once per failure spell, shows a toast naming the server; a
  desktop notification is never held back or repeated by it.
- The dev box's ntfy setup, written into the notes and, at Complete, into the ops handbook: ntfy
  bound to `127.0.0.1`, published on the tailnet by `tailscale serve --https`, `upstream-base-url:
  "https://ntfy.sh"` for the iOS app, `auth-default-access: deny-all`, a read-only user for the
  phone and a write token for Marley.

### Out (explicitly deferred)
- Taking a push back when its event is handled at the desk, and pushing only while the desk is
  away (idle or locked): the next slice (Orca's dismiss and away filter, report 04 §2.6).
- The agent's words (#538's body: the last message, the tool and its input). They stay on the
  desktop until Chad decides they may travel.
- Answering an approval from the phone (report 04 §3.2 item 2, after #508).
- A phone endpoint, pairing and device tokens, the native app, and a Rust relay: later slices,
  designed in the notes.
- Agents other than Claude Code, and OSC 9 or 777 notifications from any other program.
- Web Push (VAPID keys, RFC 8291 payload encryption) sent by Marley itself.
- A non-loopback ntfy server.

## Reference (§20)
Warp: N/A. Its Remote Control needs Warp's cloud and a login, and the Warp once-over rules it out
(docs/planning/design-notes/warp-once-over-2026-09-25.md). Upstream Zed pushes nothing to a phone.
The reference is Orca's push path, read from its MIT source (report 04 §2.6):
`src/main/runtime/push/push-dispatcher.ts` maps agent states to `needs-input` or `finished`, clips
the title to 80 characters and the body to 180, and posts to Orca's gateway, which hands title and
body in plaintext to Apple and Google. Marley keeps the event kinds and a short line, drops the
body, and puts ntfy on the box in the gateway's place, so only a message id and a topic hash reach
Apple.

### Prior art
- **Behavior maps.** Report 04 §2.6 (Orca's push, dismissal and the away filter), §3.2 item 1
  (ntfy first, Web Push later), §3.3 (what of Orca to reuse), §2.13 (the security model per path);
  report 01 §2.5 (Orca fans out to the phone before the desktop's focus gate, with a 5-second burst
  cooldown per worktree: `src/main/notifications/notification-delivery-service.ts`,
  `src/shared/notification-burst-cooldown.ts`).
- **Published material.** ntfy's docs, read 2026-09-25. Publishing is `POST` or `PUT` to
  `<server>/<topic>` with `Title`, `Priority` (1 to 5), `Tags` and `Click` headers and
  `Authorization: Bearer tk_…`, a message up to 4,096 bytes (docs.ntfy.sh/publish). For iOS, a
  self-hosted server with `upstream-base-url` sends ntfy.sh "only the message ID (in the
  `X-Poll-ID` header), and the SHA256 checksum of the topic URL", under the fixed text "New
  message", and the phone then fetches the message from the self-hosted server
  (docs.ntfy.sh/config). The Android app "won't use Firebase for any self-hosted servers"
  (docs.ntfy.sh/subscribe/phone). Access control is `auth-file`, `auth-default-access` and access
  tokens (docs.ntfy.sh/config). Claude Code's hooks reference (code.claude.com/docs/en/hooks, read
  2026-09-25) lists `StopFailure` with its error kinds (`rate_limit`, `overloaded`,
  `authentication_failed` and more).
- **Code we already ship.** `crates/marley_workbench/src/notifications.rs` (the banner gate at line
  56, the per-terminal tag); `crates/terminal/src/terminal.rs:741` (`Event::MarleyNotification`);
  `crates/marley_dcs/src/notification.rs` (the OSC 9 and 777 scanner, 4 KiB cap);
  `crates/http_client/src/http_client.rs:128` (`HttpClient::send`), reached through
  `App::http_client` (`crates/gpui/src/app.rs:1740`); `crates/settings_content/src/marley.rs` and
  `MarleySettings` (`crates/marley_workbench/src/marley_workbench.rs:162`) for the settings block.
  For the token: `crates/zed_credentials_provider` keeps credentials in a plain local file on the
  `dev` channel Marley runs on, and gpui's own keyring path files them under Zed's label
  (`KEYRING_LABEL`, `crates/gpui_linux/src/linux/platform.rs:53`), which #445 owns, so neither is
  the place yet. No crate in `Cargo.lock` speaks ntfy, and none is needed: it is one HTTP request.

## UI proof
UI-AFFECTING, though not in Marley's window: the push shows on the phone, which no scenario
reaches, and Marley draws nothing new except a toast when the server is down.
`script/e2e/535-phone-push-notifications.sh` (`compositor sway`, so Marley's window is the active
one and the focused-terminal case is real) runs the real Marley with a fake ntfy, a Python HTTP
server on a loopback port that logs each request's method, path, headers and body, and a fake
`claude` (L-claude-480, started as #481's scenario starts one) that prints #519's frames for a
permission request, a finished turn and a failed turn when the scenario touches a trigger file.
Marley runs on a private D-Bus session bus whose fake notification server logs each banner, so no
banner reaches Chad's desktop (the helper #538's scenario also uses). The proof of each push is the
fake server's log; each shot shows the agent terminal at its event,
so every log line ties to what Marley showed: `535-01-needs-input`, `535-02-finished`,
`535-03-failed`, `535-04-focused` (the event while its terminal holds focus, which pushes
nothing), `535-05-no-server` (the toast after the fake server stops). The real phone is checked by
hand at Test, ntfy on the box and the app on Chad's phone with one event from a real Claude Code,
and recorded in the notes.

## Locked-In Decisions
- D1 — ntfy on the dev box, bound to loopback and published on the tailnet by `tailscale serve`:
  no push gateway of Marley's, no Apple developer account, and Marley's own post never leaves the
  machine. The iOS app's instant delivery goes through ntfy.sh's poll relay, which carries an id
  and a hash, not the text.
- D2 — What leaves the box. To ntfy.sh and then Apple's push service, for the iOS app only: a
  message id, the SHA-256 of the topic URL and the fixed text "New message". To the phone, over the
  tailnet: the title `Marley`, the line `<project>: Claude <event>`, a priority and a tag. Nothing
  else: no last assistant message, no tool or tool input, no path, command, prompt or session id.
- D3 — Only Claude Code's agent events from #519 push. An OSC 9 or 777 that any other program
  prints (a build script's "done") is for the desk, and so are the plugin's fixed sentences, which
  #519 keeps beside its frames.
- D4 — The banner's gate decides: no push for the focused terminal of the active window, since
  Chad is looking at it. "Only while away" needs a way to tell away, and waits for the next slice.
- D5 — The token lives in a file the settings name, read at each post, so a new token needs no
  restart. A file that group or others can read is refused with a warning, and nothing is posted.
- D6 — `url` must name a loopback host (`127.0.0.1`, `[::1]` or `localhost`); Marley refuses any
  other, so no push text crosses a network on its way to ntfy.
- D7 — The post runs off the main thread, as a background task of the workbench
  (L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001 for its shape), and its
  failure never touches the desktop notification.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Claude Code in a terminal asks for permission or asks a question, and that terminal is not the focused terminal of the active window, the system shall post one push to `<url>/<topic>` whose line is `<project>: Claude needs input`. | The fake server's log; shot `535-01-needs-input` |
| REQ-002 | WHEN Claude Code's turn ends (`Stop`) in such a terminal, the system shall post one push whose line is `<project>: Claude finished`. | The log; shot `535-02-finished` |
| REQ-003 | WHEN Claude Code's turn fails (`StopFailure`) in such a terminal, the system shall post one push whose line is `<project>: Claude failed`. | The log; shot `535-03-failed` |
| REQ-004 | WHEN the system posts a push, the request shall carry the title `Marley`, the one line, a priority and a tag, and nothing from the agent's output. | The log: every request's headers and body in full |
| REQ-005 | WHILE a terminal is the focused terminal of the active window, the system shall post nothing for its events. | The log, no request; shot `535-04-focused` |
| REQ-006 | WHEN a program that is not an agent prints OSC 9 or OSC 777, the system shall post nothing. | The log after a `printf` of both in a plain shell |
| REQ-007 | WHERE `marley.push` is not set, the system shall post nothing. | The run's first part, before the setting is written: the log is empty |
| REQ-008 | WHERE `token_file` names a file of mode 0600, the system shall send its content as `Authorization: Bearer`; WHERE the file is readable by group or others, the system shall post nothing and log a warning. | The log's headers; the part with the file at 0644 and Marley's log |
| REQ-009 | WHERE `url` names a host that is not loopback, the system shall post nothing and log why. | The part with `url` set to `http://192.0.2.1:9` (a TEST-NET address): Marley's log, and no connection |
| REQ-010 | WHEN the ntfy server does not answer, the system shall show one toast naming the server and still show the desktop notification. | Shot `535-05-no-server`; the private bus's log (the banner still sent) |

## Phase Plan
- **P1 Plan** — this spec, and the design and the test plan in the notes; `brain_ask` at
  promotion.
- **P2 Code** — the pure line builder in `marley_agent`; `push.rs` and its wiring in
  `marley_workbench`; the `push` settings block and its touchpoint row; fmt and clippy clean; a
  review of the diff.
- **P3 Test** — write and run the scenario and read every shot; the phone by hand; `script/gates.sh
  --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md`, the ops handbook's
  ntfy page, ledger capture, close the ticket, archive, commit.
