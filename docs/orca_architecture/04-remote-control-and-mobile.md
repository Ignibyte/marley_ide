# Orca survey 04: remote control

Area: the mobile companion, the pairing relay, the headless runtime, SSH worktrees, ephemeral
VMs, and the security model across them. Sources: Orca at `/srv/stacks/orca-refs/orca` (checked
out 2026-09-25), `stablyai/orca-multipass-recipes` read through `gh`, and for comparison Marley
(`/srv/stacks/marley_ide`) and rustal-harness (`/srv/stacks/rustal-harness`). Orca paths are relative to its repo root; Marley
paths start with `crates/`.

## 1. Summary

Orca's remote control is five systems: a React Native phone app (App Store, TestFlight, Android APK)
that pairs from a QR code and drives the desktop over JSON-RPC on a WebSocket; a hosted relay on
GCP that splices phone and desktop sockets and cannot read the encrypted payload; a push gateway
holding Orca's APNs key; a headless runtime (`orca serve`, Electron under Xvfb, or `orcad` on Node);
and SSH worktrees, where a Node daemon on the remote owns PTYs, git and files and outlives disconnects.
Worth taking: approvals and follow-ups from the phone (Marley can answer Claude Code's `PermissionRequest`
hook with a real allow/deny, where Orca types "1" or Esc into the TUI); pushes on "needs input" and
"finished"; a chat view built from the agent's transcript file; remote terminals that survive a drop;
remote port detection with one-click forwarding. On a private network such as a tailnet, the path
Orca itself recommends, the relay, region placement and Orca's push gateway have no job in Marley.

## 2. Features

### 2.1 The phone app

**What you see and do:** a list of paired hosts (several desktops at once, merged into one worktree
list); per worktree the agent and its status (working, waiting on input, done); session tabs for
terminals, agent chats, browser pages and markdown; the file tree and previews; source control (stage,
unstage, commit, push, pull, discard, AI commit messages); diff review with line comments; PR checks,
review threads and merge; GitHub, Linear and GitLab task lists; agent session history; Claude and
Codex account switching with usage; Quick Commands; image attachments; voice dictation transcribed on
the desktop; workspace creation from an issue or a branch; and a view of the desktop's browser tab in
"Web" or phone-size "Mobile" viewport (`docs/site/content/docs/mobile.mdx`). The docs call the app
"read-mostly". The RPC allowlist makes it read-write (see 2.3).

**Underneath:** Expo Router screens in `mobile/app/` (`h/[hostId]/session/[worktreeId].tsx`, `files/`,
`source-control/`, `review/`, `pr/`, `tasks.tsx`, `accounts.tsx`); the transport in
`mobile/src/transport/`; the terminal as xterm.js inside a WebView
(`mobile/src/terminal/TerminalWebView.tsx`); device tokens in the iOS keychain through
expo-secure-store (`mobile/src/transport/host-device-token-store.ts`); push through expo-notifications
(`mobile/src/notifications/`). Two native modules: `mobile/modules/orca-notification-dismissal` (iOS,
retracts delivered banners) and `mobile/modules/orca-mobile-web-shell`, a WebView host for UI bundles
the desktop serves over RPC (`mobileWeb.bundle.*`), enabled only in builds made with
`EXPO_PUBLIC_MOBILE_SHELL=ota` and off in store builds
(`mobile/src/mobile-web-shell/use-mobile-web-shell-enabled.ts`). `mobile/rpc-foundation/goldens/` holds
787 recorded RPC exchanges that pin the app's behavior per scenario, and `mobile/scripts/mock-server*.ts`
(1,831 lines) is a stand-in desktop answering about 55 methods. Bundle id `com.stably.orca.mobile`,
app version 0.0.51 (`mobile/app.json`). iOS is a first-class target, so an iPhone works.

**Good and bad:** the desktop stays the source of truth, and the app reports delivery ambiguity
instead of guessing ("Response unconfirmed, check chat before retrying" in
`mobile/src/session/mobile-native-chat-permission-send.ts`). It is also 186K lines of non-test
TypeScript in `mobile/src` and `mobile/app` (session 38.7K, tasks 26.4K, transport 17.6K, terminal
11.5K), with 863 commits since the first mobile commit on 2026-05-04. The repo's own findings files
list what a phone link breaks: `mobile/issue-5049-unresponsive-session-findings.md` (the reconnect loop
parked for good after 12 attempts with no AppState listener, half-open sockets taking about 28 s to
reap, screens driving a closed client after a reconnect swapped it) and
`mobile/terminal-output-streaming-findings.md` (daemon-backed PTYs never fed the phone's output
stream, so sends succeeded and no output came back).

**Size:** a product of its own.

**Marley today:** lacks it.

### 2.2 Pairing

**What you see and do:** on the desktop, Settings → Mobile shows a QR code and a copyable
`orca://pair?code=…` link; the phone scans or pastes it. With an Orca account signed in on the desktop,
the offer also carries a relay invite; without one the phone needs a LAN or Tailscale address. A saved
host's address can be edited later without re-pairing (home LAN versus Tailscale). A headless host
prints the QR with `orca serve --mobile-pairing` (`docs/site/content/docs/remote-servers.mdx`). Paired
devices are listed and revocable, and revoking terminates their open sockets.

**Underneath:** the code is base64url JSON `{v: 2, endpoint, deviceToken, publicKeyB64, scope, relay?}`.
`publicKeyB64` is the desktop's static X25519 key; `scope` is `mobile` or `runtime`; `relay` holds
`directorUrl` and `cellUrl` (both must be canonical HTTPS origins), `assignmentEpoch`, `relayHostId`, a
43-character `inviteToken` and an expiry at most 10 minutes out (`src/shared/mobile-relay-pairing-offer.ts`,
`src/shared/pairing.ts`). The desktop's `DeviceRegistry` (`src/main/runtime/device-registry.ts`) is a JSON
file in userData written 0600 (ACLs on Windows): per device a UUID, a name, a token of 24 random bytes
in hex, the scope, `pairedAt`, `lastSeenAt`, an optional relay binding and push registration. An
unscanned token is handed out again until a phone uses it or the user regenerates the code. The
keypair is a second JSON file (`src/main/runtime/e2ee-keypair.ts`). Both loaders refuse to overwrite a
file they could not read, since regenerating would un-pair every phone. Creating a mobile offer
rebinds the WebSocket listener from 127.0.0.1 to 0.0.0.0 on the same port, default 6768
(`src/main/runtime/runtime-rpc/runtime-rpc-network-exposure.ts`), and later launches bind wide from the
start once a network-reach device exists (`docs/reference/orcad-operations.md`, "Bind policy"). The
address picker puts a Tailscale 100.64/10 address first and skips docker, virbr, vmnet and similar
bridges (`src/shared/pairing-address-auto-selection.ts`).

**Good and bad:** pinning the desktop's public key in the QR means neither the network nor the relay
can impersonate the desktop, and per-device tokens make revocation real. Direct tokens never expire,
though the troubleshooting section says codes "expire after a few minutes"; only relay invites do. The
mobile README says the QR carries a "TLS fingerprint". It carries the X25519 key, and nothing in
`src/main` ever sets the WebSocket server's TLS options, so the direct link is plain `ws://`.

**Size:** about 1,500 lines across the files named.

**Marley today:** the MCP server has one per-boot bearer in an endpoint file for loopback clients
(`crates/marley_mcp/src/discovery.rs`). No device pairing.

### 2.3 Transport, encryption, and what a phone may call

**Underneath:** `src/main/runtime/rpc/ws-transport.ts` is a `ws` server with a 1 MiB message cap, 128
connections, a 10 s pre-auth timeout and a 15 s ping sweep. Every socket starts with an
application-layer handshake (`src/main/runtime/rpc/e2ee-channel.ts`), in one of two versions.

- v1, used on every direct (LAN or Tailscale) connection. The phone makes a fresh X25519 key per socket
  and sends `e2ee_hello {publicKeyB64}` in plaintext; the desktop answers `e2ee_ready` in plaintext; each
  later frame is `base64(24-byte random nonce ‖ nacl.box.after(...))` under one shared key for both
  directions. The phone's first encrypted frame is `e2ee_auth {deviceToken}`, answered by
  `e2ee_authenticated` (`mobile/src/transport/rpc-client-socket-session.ts:111-126`,
  `e2ee-channel.ts:189-243`).
- v2, required only on relay sockets (`src/main/runtime/rpc/mobile-socket-wiring.ts:149`) and added with
  the relay on 2026-07-14 (#8536). Hello and ready carry 32-byte nonces and a context (`transport`,
  `relayHostId`). HKDF-SHA256 over X25519(phone ephemeral, desktop static), with salt `H(label ‖ nonces)`
  and info `label ‖ H(transcript)`, yields a phone-to-desktop key, a desktop-to-phone key and a session
  id. Each frame is `secretbox` under a deterministic nonce (session-id prefix, direction, payload kind,
  64-bit counter) and the receiver accepts only the next counter, so replayed, reordered or reflected
  frames fail. The auth message must echo the transcript hash (`src/shared/mobile-e2ee-v2-contract.ts`,
  `src/shared/mobile-e2ee-v2-framing.ts`, `mobile/src/transport/mobile-e2ee-v2-key-schedule.ts`; 470
  lines in all, with test vectors in `src/shared/mobile-e2ee-v2-fixtures.ts`).

After the handshake a request is JSON `{id, method, params}` and a reply `{id, ok, result | error}`; a
streaming method replies many times under one id. Terminals, screencasts and TCP tunnels use binary
frames with a 16-byte header: a kind byte, a version, an opcode, a 32-bit stream id and a 64-bit
sequence (`src/shared/terminal-stream-protocol.ts`, `browser-screencast-protocol.ts`,
`browser-network-tunnel-protocol.ts`). `status.get` returns `RUNTIME_PROTOCOL_VERSION` (3, minimum 2)
and about 80 named capabilities (`src/shared/protocol-version.ts`), and the phone shows a block screen
when versions are incompatible. `docs/reference/remote-wire-compatibility.md` sets the rules for
evolving the wire: a new optional field is safe; a new stream opcode must be negotiated, because
decoders drop unknown opcodes without an error; an unknown enum arm must degrade, never fail the reply.

Authorization is the device token, looked up in the registry (`device-registry.ts:238`). A
`mobile`-scope token may call 294 of the roughly 620 runtime methods
(`src/main/runtime/runtime-rpc/runtime-rpc-mobile-method-allowlist.ts`), a list that includes
`terminal.create`, `terminal.send`, `git.push`, `git.discard`, `worktree.rm`, `settings.update`,
`ssh.connect`, `files.createFile` and `github.mergePR`. A `runtime`-scope token (another desktop, the web
client, the CLI) may call everything.

**Good and bad:** v2 is careful work and the layout to copy if Marley ever wants app-layer
encryption. The path most people use still runs v1, with no counters, no direction binding and an
unauthenticated plaintext `e2ee_ready`. An on-path attacker can neither read nor forge frames but can
replay a captured one inside the same socket, and a replayed `terminal.send` is a repeated command.
Desktop-to-remote-runtime connections use v1 as well (`src/shared/remote-runtime-request-websocket.ts:47-56`).
The phone's token amounts to a shell on the desktop, and it never expires.

**Size:** RPC core 8K lines, method handlers 29K, the phone's transport 17.6K.

**Marley today:** Streamable HTTP MCP bound to `127.0.0.1:0` only (`crates/marley_mcp/src/transport.rs:118`),
an Origin check, the bearer, read tools free and write tools behind an explicit per-class grant
(`crates/marley_mcp/src/permission.rs`, `[mcp.expose] allow_write`). Nothing another device can reach.

### 2.4 Terminals on the phone

**What you see and do:** terminal tabs with live output and colors, an accessory row for keys that are
awkward on a phone keyboard (Tab, Shift+Tab), a "Live" mode that sends each keystroke to the PTY and a
buffered command bar (`mobile/mobile-terminal-direct-input-default.md` makes Live the default for newly
seen terminals), selection and copy, pinch zoom, pasted images.

**Underneath:** two subscribe paths. The legacy one answers `terminal.subscribe` first with
`{type: "scrollback", cols, rows, serialized}` (an ANSI rendering of screen and scrollback) and then with
`{type: "data", chunk}` replies; `mobile/scripts/mock-server-terminal-stream.ts` is the smallest correct
server for it. The binary one, `terminal.multiplex`, carries many terminals on one stream with opcodes
for output, snapshot start, chunk and end, input, resize, ack, `ClaimViewport`, `SetOutputPaused` and
`WriteUnavailable`. Snapshots come from `@xterm/headless` emulators kept per session
(`src/main/daemon/headless-emulator.ts` in the terminal daemon). Flow control runs on acknowledged
windows: 512 KiB per stream growing to 2 MiB, 2 MiB in total growing to 8 MiB, 256 KiB pending
(`src/shared/terminal-multiplex-flow-control.ts`). A client that falls behind loses its backlog and
receives a fresh screen snapshot instead, reason `ack-pending-overflow`
(`src/main/runtime/rpc/methods/terminal/terminal-multiplex-flow-control.ts`). If the encrypted outbound
queue still overflows, the socket closes with 1013 (`e2ee-channel.ts`).

Terminal size follows a "floor". When the phone subscribes or types, the desktop PTY is resized to the
phone's columns and rows, the desktop pane shows a mobile-driver overlay with a take-back button, and
the phone size stays after the phone leaves unless the user chose a 1, 5 or 30 minute restore in
settings (`src/main/runtime/orca-runtime-mobile-took-floor.ts`, `orca-runtime-apply-mobile-display-mode.ts`,
`src/renderer/src/components/terminal-pane/MobileDriverOverlay.tsx`,
`src/renderer/src/components/settings/mobile-auto-restore-options.ts`). While the phone drives, input
from other paired clients is refused (`isTerminalInputLockedForClient` in
`src/main/runtime/rpc/methods/terminal/terminal-input-delivery.ts`). A `terminal.send` that asks for
`requireAgentStatus: "sendable"` is refused while the agent sits at a permission prompt, so a follow-up
cannot answer the dialog by accident (`terminal_guard_permission` in
`src/main/runtime/rpc/terminal-agent-send-guard.ts`, called from `terminal-send-method.ts:143`).

**Good and bad:** bounded buffers with snapshot recovery keep memory flat under a slow phone, and the
permission guard is a detail to copy. Phone-fit resizes the real PTY, so Claude Code on the desktop
reflows to phone width and stays that way by default.

**Size:** about 5K lines of terminal RPC on the desktop; 11.5K lines of terminal code on the phone.

**Marley today:** agents read terminals through `terminal_list`, `terminal_read` and `terminal_blocks`.
Nothing streams a terminal to another device or takes input from one.

### 2.5 Chat view and approvals from the phone

**What you see and do:** long-press a tab to switch a Claude or Codex session between the raw terminal
and "Chat UI": a transcript, a composer with `@` file mentions and agent-specific `/` commands, a model
picker, image attachments, and permission cards with Allow and Deny.

**Underneath:** for an agent running in a terminal, the desktop reads the agent's own transcript file
(Claude's JSONL under `~/.claude/projects`, Codex's session files), decodes it into messages, and sends
the phone the last 40 by default with long bodies clipped for mobile
(`src/main/native-chat/transcript-line-decoders-claude.ts`, `src/main/runtime/rpc/methods/native-chat.ts`).
The composer's text goes into the PTY. A permission card comes from the Claude `PermissionRequest`
hook's envelope, published as `agentStatus.interactivePrompt` with `{approval: {tool, summary}}`, or
from a heuristic parse of the prompt text in the last assistant message. Tapping sends the literal
keystroke ("1", "2" or Esc) without Enter (`mobile/src/session/mobile-native-chat-permission.ts`,
`mobile-native-chat-permission-send.ts`). The first file says so: "there is no structured permission
event on mobile." Orca registers `PermissionRequest` only to set the pane's status and never answers
it (`src/main/claude/hook-settings.ts:92`, `src/main/agent-hooks/server/server-claude-status-rules.ts`).
Sessions that Orca runs itself through the Claude Agent SDK (`agentSession.*`) get structured approvals;
terminal sessions do not.

**Good and bad:** reading the transcript gives the phone something readable without drawing a TUI on a
six-inch screen. The keystroke approach depends on the TUI's current menu numbering and on heuristics.

**Size:** transcript decoding and the chat RPCs are a few thousand lines; the phone's chat screens live
in the 38.7K-line session folder.

**Marley today:** the rail shows each agent's state, and the plugin's hooks (`Notification` with
`permission_prompt` and `idle_prompt`, and `Stop`) emit OSC 777 notifications
(`crates/marley_workbench/claude_plugin/marley/hooks/hooks.json`, `notify.sh`). No transcript view; a
prompt can only be answered in the terminal. Queued: one approvals inbox in the rail.

### 2.6 Push notifications

**What you see:** a banner when an agent finishes or needs input, following the desktop's notification
categories; tapping opens the session; the banner disappears when the desktop handles the event;
filters for sound and for "only when the desktop is away".

**Underneath:** the phone registers its APNs or FCM token with the desktop (`notifications.registerPush`),
which stores it on the device's registry row. `src/main/runtime/push/push-dispatcher.ts` maps agent
states to `needs-input` or `finished`, clips the title to 80 and the body to 180 characters, and posts to
`push.onorca.dev` (`cloud/apps/push`, 3K lines). The desktop proves itself to the gateway with its X25519
key (a challenge, then a 24-hour session); no account is needed. The gateway holds Orca's Apple `.p8`
key for topic `com.stably.orca.mobile`, sends Android through FCM, and allows 64 devices and 300 events
per 15 minutes per host (`cloud/packages/push-contract/src/push-limits.ts`, `cloud/docs/push-gateway.md`).
Title and body reach the gateway, Apple and Google in plaintext
(`cloud/packages/push-contract/src/send-messages.ts`). A `dismiss` push retracts an earlier banner.

**Good and bad:** dismissal and the away filter keep the phone quiet while you sit at the desk. A
headless `orca serve` sends no pushes at all, because finished-agent detection runs in the desktop
renderer (`docs/reference/headless-linux-server.md`, "Pairing troubleshooting"). The store app can only be
pushed through Orca's gateway, since Orca holds the APNs key.

**Size:** 1.5K lines on the desktop, 3K in the gateway, 1.8K on the phone.

**Marley today:** desktop notifications from OSC 9/777. Nothing reaches the phone.

### 2.7 Reconnect and liveness on the phone

Reconnect waits 0.5, 1, 2, 4, 8, 15, 30 and 60 s over 12 attempts, then retries every 90 s
(`mobile/src/transport/rpc-client-reconnect-schedule.ts`). Coming to the foreground or a network change
resets the budget and probes at once (`connection-revival-triggers.ts`, `notifyForeground` in the RPC
client). A probe runs after 20 s of silence with an 8 s deadline (`rpc-session-liveness-watchdog.ts`), and
the server pings every 15 s. Streams re-subscribe after a reconnect and start from a fresh snapshot. On
the relay, the phone keeps probing the direct address and moves back after 3 successes within 30 s, with
a 60 s minimum stay and a 60 s cooldown after a failure (`mobile-endpoint-supervisor.ts`). Mutations that
may or may not have landed are not replayed blindly: `worktree.create` carries a `clientMutationId`,
`terminal.create` has an idempotency capability, and the UI says "unconfirmed" when it cannot know.

The three failure modes in issue-5049 (a parked retry loop, half-open sockets, a screen holding a
swapped-out client) apply to any phone client on iOS as much as Android, and make a test checklist.

**Size:** inside the transport figures in 2.3.

**Marley today:** nothing to compare.

### 2.8 The relay (`cloud/`)

**What it is:** a director that assigns each desktop to a cell and signs the assignment, cells that carry
the connections, and PostgreSQL (Cloud SQL) for state, on GCP in `us-central1` and `asia-east2`, run by
Terraform and 25 GitHub workflows that are switched off in the public repo (`cloud/README.md`,
`cloud/infra/terraform/`, `cloud/packages/relay-contract/src/relay-regions.ts`).

**Desktop side:** the desktop must be signed in to an Orca account that carries the `relay.use`
entitlement (`src/main/runtime/relay/relay-auth-context.ts`). It gets a 5-minute ES256 relay JWT
(`purpose: host-control`, a `relayHostId` claim) from `login.onorca.dev` and opens a control socket to its
cell: `HostHello` with the `relayHostId`, the host public key and the assignment epoch; an encrypted
`HostChallenge`; a proof over a 16-field transcript (`src/main/runtime/relay/relay-host-proof.ts`,
`cloud/packages/relay-contract/src/control-messages.ts`, `cloud/apps/relay/src/relay-token-verifier.ts`).
`relayHostId` is the first 16 base64url characters of the SHA-256 of the host public key
(`src/main/runtime/relay/relay-http-client.ts:89`).

**A phone connection:** the phone opens a socket to the cell URL from the QR and presents the invite (10
minutes, 5 attempts). The cell tells the desktop on the control socket (`ConnectionOpen {connId,
connTicket}`), the desktop opens a data socket with that ticket, and the cell splices the two
(`cloud/apps/relay/src/splice-forwarder.ts`). The splice forwards frames unchanged, pauses the sending TCP
socket when the receiver has 256 KiB buffered and resumes it below 64 KiB, and closes both after 10 s
wedged or 8.25 MiB queued; frames may be 8 MiB, a host may hold 8 connections, and idle links close after
10 minutes (`relay-contract/src/protocol-limits.ts`, `admission-budgets.ts`). Inside, E2EE v2 runs end to
end and the phone presents its device token to the desktop. The desktop then installs, at the relay, the
hash of a 30-day resume token for that phone (`DeviceCredentialInstall` in `credential-messages.ts`), so
later connections skip the invite.

**What the relay sees and can do:** account ids from the JWT, `relayHostId`, the host public key, IP
addresses, frame sizes and timing. It cannot read frames, impersonate the desktop to the phone (the key
is pinned from the QR and the transcript binds `relayHostId`), or impersonate the phone to the desktop
(the device token travels inside the encryption). It can refuse or degrade service. A "moved" message
counts only when it comes from the configured director origin (`isTrustedNewerMove` in
`director-messages.ts`).

**Region placement:** the desktop probes each region after a warm-up request, takes three samples,
compares the minimums, caches the choice for 24 hours, and otherwise lands in `us-central1`
(`docs/reference/relay-regional-placement.md`). `docs/relay-region-correction/` is the plan for moving an
existing assignment without dropping live connections.

**Self-hosting:** possible in principle, undocumented in practice. The server runs as one `combined`
process on SQLite (`ORCA_RELAY_ROLE` in `cloud/apps/relay/src/config.ts`; `database.ts` uses `node:sqlite`),
but it verifies desktop JWTs against the JWKS of an issuer that lives in the private `stablyai/orca-cloud`
repo, requires several GCP service-account settings, and the pairing schema accepts only HTTPS director
and cell origins. The desktop can be pointed elsewhere with `ORCA_RELAY_URL` and the `ORCA_CLOUD_*`
variables (`src/main/orca-profiles/profile-cloud-auth-config.ts`); the relay-token endpoint behind them
would have to be written from scratch.

**Operations:** `cloud/docs/relay-reconnect-2026-09-findings.md` logs a 2026-09-04 incident across cells
numbered up to c28: crash cascades (75 container deaths in 2.6 hours), Postgres row-lock stalls, and 18
monitor dry-runs with one pass while trying to roll the fleet.

**Good and bad:** the credentials are layered well. The relay holds only hashes of its own routing
credentials, the phone's real credential is checked by the desktop inside the encryption, and a
compromised relay gets metadata and nothing else. The costs are an Orca account with a relay
entitlement on every desktop that wants it, a multi-region service to operate, and the fact that the
careful v2 encryption exists only on this path.

**Size:** relay server 21.7K lines, push gateway 3K, contracts 2K, desktop relay client 4.7K, plus a large
share of the phone's transport.

**Marley today:** none. On a tailnet none is needed.

### 2.9 The headless runtime

**What you see and do:** Settings → Remote Orca Servers → "Advertise this app as a server" → New Link,
pick the Tailscale address, copy the access link; on the client, Add Server and paste it. The server keeps
repos, worktrees, terminals, agents and provider accounts. Every client (another desktop, the web client
served from the same port, the phone, the CLI) sees the same sessions, with a sidebar filter to hide
workspaces another client created. Each client gets its own revocable grant under "Shared Server Access"
(`docs/site/content/docs/remote-servers.mdx`, `ways-to-run.mdx`).

**Underneath:**

- `orca serve --pairing-address <ip> [--port 6768] [--json] [--mobile-pairing]` is the packaged Electron
  app without a window. On Linux it starts Xvfb on `:99` and needs the GTK and NSS library set; `--json`
  prints one `orca_server_ready` line for supervisors; exit code 3 means another instance holds the
  profile (`docs/reference/headless-linux-server.md`, 1,010 lines, half of them upgrade and rollback
  scripts).
- `orcad` is the same runtime on plain Node (`src/main/orcad/`, 3.9K lines). It binds 127.0.0.1 by
  default, accepts only literal IPs, locks its data root, exits 78 on configuration faults, and refuses a
  mobile pairing offer while pinned to loopback. Clients are meant to reach it through an SSH local
  port-forward, and the SSH code can install, health-check, activate and roll back an `orcad` on a remote
  host (`src/main/ssh/orcad-remote-deploy.ts`, `orcad-remote-launch.ts`, `docs/reference/orcad-operations.md`).
- Clients attach with a `runtime`-scope pairing offer over the same WebSocket and E2EE v1.
- The terminal daemon (`src/main/daemon/`, 29.6K lines) is a separate Node process that owns every local
  PTY. It speaks NDJSON over a Unix socket (`hello {version, token, clientId, role}`, then `createOrAttach`,
  `write`, `resize`, `pausePty`, `resumePty`, `kill`, `signal`, `listSessions`, `detach`), keeps a headless
  xterm per session for snapshots and cold restore, and names its socket by protocol version
  (`daemon-v<N>.sock`) so a newer runtime adopts an older daemon instead of killing it. On Linux it starts
  under `systemd-run --user --scope`, so restarting the service leaves terminals running (with
  `loginctl enable-linger` for a service account).

**Good and bad:** two ideas worth keeping. One is the vocabulary for a remote process, `live`,
`unverifiable`, `exited`, with the rule that losing contact is never evidence of exit
(`docs/reference/ssh-execution-boundary.md`). The other is the daemon's socket-ownership protocol built on
"never collapse can't-tell into dead" (`src/main/daemon/AGENTS.md`). On the other side, `orca serve` puts
a GUI toolkit and a virtual X server on a server, a headless host sends no pushes, `orcad` keeps its
credentials unsealed on disk, and there is no continuous health endpoint and no daemon log rotation (the
"What is not covered" list in `orcad-operations.md`).

**Marley today:** Zed's `remote_server` is a headless project server (worktrees, LSP, git, tasks) that does
not own terminals. rustal-harness is the planned session runtime: the `rh` runtime and CLI with a private
tmux server, an SQLite journal, managed input where a connection starts as an observer and must claim
control with a generation number, snapshot-plus-event subscriptions, `rh mcp` on stdio, and remote entry
through an OpenSSH `ForceCommand` with no network listener (`/srv/stacks/rustal-harness/docs/MANAGED_INPUT.md`,
`docs/M8_ACCEPTANCE.md`, M9 in `docs/ROADMAP.md`).

### 2.10 SSH worktrees

**What you see and do:** Settings → SSH, add a target by form or from `~/.ssh/config`, Test, then pick it
under "Run on" when creating a worktree. `git worktree add` runs on the remote, agents launch there, the
editor streams saves, and diffs and commits work from the laptop. A status chip shows connected,
reconnecting or disconnected. A sleeping laptop leaves agents running and they reattach with scrollback.
Files and folders can be downloaded, and VS Code Remote-SSH can open the path
(`docs/site/content/docs/ssh.mdx`).

**Underneath:** the client uses the `ssh2` JavaScript library and hands over to system OpenSSH for
ProxyJump, ProxyCommand, GSSAPI and FIDO2 keys; OpenSSH connection reuse is on by default. The first
connect uploads a relay bundle to a content-hash directory under `~/.orca-remote/` and starts it detached
(`nohup`, SIGHUP ignored). The remote needs Node and npm, found by probing nvm, fnm, mise, asdf, volta, n
and system paths (`src/main/ssh/ssh-remote-node-resolution.ts`), and on Linux `node-pty` compiles on the
host, so terminals need make, a C++ compiler and python3 (files and git work without them). The daemon
binds a Unix socket and writes a credential file. Each connect and reconnect runs `node relay.js
--connect` over an exec channel, which checks the credential and the build version and then bridges stdin
and stdout to the socket (`src/relay/relay.ts`, `relay-daemon.ts`, `relay-connect-channel.ts`). Messages
are JSON-RPC 2.0 in a 13-byte frame copied from VS Code's PersistentProtocol (type, id, ack, length), with
a 5 s keepalive and a 20 s timeout (`src/main/ssh/relay-protocol.ts`). The remote side runs PTYs, agent
CLIs, git (`src/relay/git-handler.ts`), file reads, watches and ripgrep search, repo setup hooks, and
non-interactive agent runs for commit messages (`src/relay/agent-exec-handler.ts`). GitHub and GitLab
calls stay on the laptop. Status from agent hooks on the remote arrives as relay notifications
(`src/main/ssh/ssh-relay-session.ts:1567-1625`).

**Persistence and reconnect:** PTYs are children of the remote daemon, so quitting Orca detaches.
Leases are persisted, and the next connect reattaches with a replay of the last 102,400 characters. "Keep
terminals alive until reset" is the default; otherwise a grace period of 60 s to 7 days applies. The
client's reconnect ladder is 1, 2, 5, 5, 10, 10, 10, 30 and 30 s (`src/main/ssh/ssh-connection-utils.ts:35`).
Two weaknesses are on record. An app update installs a relay under a new hash directory, the old daemon
refuses the new build, and every terminal it owns becomes unreachable though still running (#13852).
And the checkpointed output recovery has never run on an SSH reconnect, because the reconnecting client
keeps its id, so panes repaint from the byte tail (`docs/reference/ssh-reconnect-source-recovery.md`).

**Host keys:** until STA-4319 the `ssh2` path accepted every host key; the verifier "records a SHA-256
fingerprint and then `return true`" (`docs/reference/ssh-host-key-verification.md`). Phase 1 now checks
`known_hosts` and Orca's own store, accepts first contact silently with a notice, and refuses changed or
revoked keys. Passwords are cached in memory and replayed on reconnect.

**The control plane stays on the laptop:** on the SSH host, `orca` is a shim that sends argv, cwd, env and
stdin through the relay, and the laptop runs its full bundled CLI with them against its own runtime, with
no command allowlist (`ssh-relay-session.ts:1453-1490`, `src/main/ssh/ssh-remote-orca-cli.ts`,
`ssh-remote-cli-host-passthrough.ts`). Agents on the remote use it for orchestration. It also means any
process running as that user on the SSH host can run any `orca` command against the laptop's runtime,
`terminal create` and `terminal send` on the laptop's own worktrees included (read from the code, not
tried).

**Good and bad:** the work outlives the laptop, and the design notes are honest about what is known
after a disconnect. The price is 33K lines on the client and 29.5K in the remote daemon, a Node toolchain
on every host, relays locked to the client's build, months without host-key checks, and a remote host
inside the laptop's trust boundary.

**Marley today:** Zed's remote development runs `remote_server` on the host and speaks protobuf over
SSH, with a 5 s heartbeat and 3 reconnect attempts (`crates/remote/src/remote_client.rs:161-166`). Its
terminals are separate `ssh` processes built by `build_command` (`crates/remote/src/transport/ssh.rs:330`,
`crates/project/src/terminals.rs:548`), so they end with the connection. They do set `TERM_PROGRAM=zed` on
the remote (`crates/terminal/src/terminal.rs:721`, "used by both local terminals and remote terminals"),
so Marley's plugin, if installed on the remote host, can raise OSC 777 notifications that travel back
through ssh (untested). `marley_remote` parses `[user@]host[:port]`, builds `ssh -p N -- user@host`, keeps
saved hosts in `[[remote.hosts]]`, adds "connect:" palette actions, and badges the pane ⇄ or ✗ by whether
ssh has exited (`crates/marley_remote/src/marley_remote.rs`). That plain ssh does not forward
`TERM_PROGRAM`, and the plugin's `notify.sh` exits unless `TERM_PROGRAM=zed`, so Claude Code in a
`marley_remote` pane raises no Marley notifications. Host keys stay with the user's OpenSSH, which is the
safer arrangement.

### 2.11 Ports

**SSH:** the right sidebar's Ports tab lists "Detected" ports. The remote daemon reads `/proc/net/tcp` and
`/proc/net/tcp6`, maps socket inodes to PIDs through `/proc/*/fd`, drops port 22, sshd and its own process,
and returns at most 50 (`src/relay/port-scan-handler.ts`). The client polls every 12 s, backs off to 30 s
while nothing changes, stops while the window is hidden, and keeps ports that were open at the first scan
out of the suggestions (`src/main/ssh/ssh-port-scanner.ts`). One click forwards to `127.0.0.1` locally
through `ssh2` or `ssh -L` (`src/main/ssh/ssh-port-forward.ts`); privileged remote ports map above 10000 (80
to 10080); forwards survive restarts and reconnects.

**Local:** the workspace port scanner attributes listeners to worktrees by the listening process's working
directory and command line, and a watcher reads PTY output at line boundaries for the URLs dev servers
print, keeping one per worktree and port and dropping it when nothing listens on that port
(`src/main/ports/advertised-url-watcher.ts`, `local-workspace-port-attribution.ts`).

**Through a paired runtime:** a TCP tunnel inside the RPC WebSocket with window updates
(`src/shared/browser-network-tunnel-protocol.ts`), so a client-side browser can load a dev server on the host.

**Good and bad:** the first-scan baseline keeps sshd and system daemons out of the suggestions, and the
backoff keeps a quiet host cheap. The remote scan covers Linux and Windows only; a macOS host returns an
empty list (`port-scan-handler.ts`, the `ports: []` branch).

**Size:** 2.4K lines local, under 1K remote.

**Marley today:** Zed takes static `port_forwards` on an SSH connection (`ssh -N -L`,
`build_forward_ports_command` in `crates/remote/src/transport/ssh.rs`) and detects nothing. Queued:
localhost URLs printed in a terminal open in a Browser tab, and port offsets for worktree agents.

### 2.12 Ephemeral VMs

**What it is:** a recipe in the project's `orca.yaml` (`environmentRecipes`) or from a plugin (`vmRecipes`)
with `create`, `suspend`, `resume` and `destroy` shell commands. `create` must print JSON
`{schemaVersion, connection: {type: "ssh", target} | {type: "orca-server", pairingCode}, projectRoot}`.
Orca sets `ORCA_VM_MODE`, `ORCA_VM_INSTANCE_ID`, `ORCA_REPO_PATH`, `ORCA_REPO_URL`, `ORCA_REPO_BRANCH` and
a few more for the commands (`src/shared/ephemeral-vm-recipe-process.ts:179-199`,
`src/shared/ephemeral-vm-recipes.ts`), then connects over SSH or pairs with the `orca serve` the recipe
started. A skill (`orca-per-workspace-env`) has an agent write the recipe; the UI sits under Settings →
Experimental → Cloud VM. Runtime-owned SSH targets skip host-key persistence, since each launch brings a
new key. Providers are the user's (Vercel Sandbox, Fly, Modal, Docker, plain SSH per `ways-to-run.mdx`).

**The official Multipass pack** (`stablyai/orca-multipass-recipes`, a single commit on 2026-07-13) is four
`multipass` commands keyed on `$ORCA_VM_NAME`. The runner never sets that variable and `create` prints no
connection JSON, so the recipe as published cannot finish a create.

**Good and bad:** the contract is small and names no provider. It is four shell commands and one JSON
line, and Orca's SSH or pairing code does the rest. The official example does not work, and recipes are
shell commands from the repository, which Orca limits by reading them only from the project's primary
checkout (`ways-to-run.mdx`).

**Size:** 5.3K lines.

**Marley today:** nothing. Marley's remote hosts are long-lived boxes.

### 2.13 The security model, path by path

| Path | Transport | Encryption | Who proves what | Credential life and storage | A stolen credential gives |
|---|---|---|---|---|---|
| Phone to desktop, LAN or Tailscale | `ws://` on 0.0.0.0:6768 | App-layer v1: nacl box, random nonces, one key both ways, no counters | Desktop: pinned X25519 key. Phone: device token | Never expires; JSON 0600 on the desktop, keychain on the phone | 294 methods including `terminal.send` and `git.push`, so a shell on the desktop |
| Phone through the relay | `wss://` to a cell | v2: a key per direction, counters, transcript bound to `relayHostId` | Desktop to relay: Orca JWT and key proof. Phone to relay: invite, then 30-day resume token. Phone to desktop: device token | The relay stores hashes only | The same; the relay credential alone opens nothing |
| Push | HTTPS to `push.onorca.dev`, then APNs or FCM | None end to end; title and body in plaintext | Desktop: X25519 challenge | 24-hour session | Up to 300 banners per 15 minutes to that host's phones |
| Desktop to a remote runtime | `ws://` on a private network | v1 | Runtime-scope token | Revocable grant | Every method |
| `orcad` | Loopback, reached by SSH `-L` | SSH | SSH | Credentials unsealed in the data root | Local access to the data root |
| SSH worktree | `ssh2` or OpenSSH | SSH | Host keys since STA-4319; user by key, agent or cached password | Password and passphrase held in memory | The remote host can drive the laptop's full `orca` CLI |

Orca's docs tell users to keep all of this on a private network ("Do not forward the Orca port directly
to the public internet. Prefer Tailscale, WireGuard, a trusted LAN, SSH forwarding, or an authenticated
tunnel", `remote-servers.mdx`), which is how Marley would run it anyway.

## 3. Bring to Marley

### 3.1 Orca's pieces against Marley's

| Orca | Marley today | Gap |
|---|---|---|
| Remote daemon owns PTYs; terminals survive a drop | Zed remote terminals and `marley_remote` panes are ssh processes that die with the link | Persistence: tmux now, rustal-harness later |
| Remote git, files, search | Zed `remote_server` | None |
| Remote agent status through the relay | OSC 777 through ssh; Zed remote-project terminals set `TERM_PROGRAM=zed` | `marley_remote` panes; plugin installed on each host |
| Port scan and one-click forward | Static `port_forwards` in Zed | Detection |
| Terminal daemon: snapshots, survives restarts | Terminals live in the Marley process | rustal-harness |
| `orca serve` shared by laptop, web and phone | Nothing | `rh` on each box, aggregated by Marley's fleet view (rustal-harness M9) |
| Phone RPC server with device tokens | `marley_mcp`: loopback, one bearer, write grants | A phone endpoint |
| Push gateway | Nothing | ntfy or Web Push |
| Relay | Nothing | Not needed on the tailnet |

### 3.2 A Marley remote-control slice

The slice assumes the phone and the desktop share a private network (a tailnet, which the phone joins
with the Tailscale app), and that Marley reaches remote hosts over SSH as it does today. No relay.

The pieces, in the order they pay off:

1. **Push to the iPhone.** The plugin hooks already fire on permission, idle and stop. Add a notifier
   beside the OSC 9/777 handler that posts the event to a self-hosted ntfy on the dev box. ntfy's iOS app
   gets instant delivery for self-hosted servers through an `upstream-base-url` poll request relayed by
   ntfy.sh that carries a message id rather than the text (verify against ntfy's docs; the phone must be
   on the tailnet to fetch the message). The notification's click URL opens the session once the phone
   client exists. Later, Web Push from Marley itself: VAPID keys, iOS 16.4+ home-screen web apps, and a
   payload encrypted to the subscription under RFC 8291, so Apple's push service relays ciphertext. From
   Orca: send a dismiss when the event is handled at the desk, filter on "desk away", give each event an
   id.

2. **Structured approvals.** Claude Code's `PermissionRequest` hook can settle the dialog. Its output
   `{"hookSpecificOutput": {"hookEventName": "PermissionRequest", "decision": {"behavior": "allow"}}}`
   (or `"deny"` with a `message`) decides on the user's behalf; command, `http` and `mcp_tool` hooks
   default to a 600 s timeout; the hook input carries `session_id`, `transcript_path`, `cwd`, `tool_name`
   and `tool_input` (checked in the hooks reference at code.claude.com/docs/en/hooks on 2026-09-25). The
   plugin already bundles the `marley` MCP server
   (`crates/marley_workbench/claude_plugin/marley/.mcp.json`), so a hook of type
   `mcp_tool` with `server: "plugin:marley:marley"` can call a new tool, say `approval_request`, that
   waits until the rail inbox or the phone answers and returns nothing on timeout so the TUI dialog takes
   over. That replaces Orca's keystroke injection with a decision Claude Code understands, and makes the
   queued rail inbox and the phone answer the same object. From Orca: give each prompt an identity and a
   revision so a late tap cannot answer a newer prompt, show "unconfirmed" when the answer's fate is
   unknown, and refuse a follow-up while a prompt is open.

3. **A phone endpoint.** A new crate (working name `marley_remote_control`) serving HTTPS and a WebSocket.
   Marley's rule that it binds loopback only can stay: bind 127.0.0.1 and let
   `tailscale serve --https=<port> http://127.0.0.1:<port>` publish it on the tailnet with a real
   certificate for the ts.net name (HTTPS certificates must be enabled for the tailnet first). That also
   gives the PWA the secure context that service workers and Web Push require. Pairing follows Orca's
   shape: a QR in a Marley modal with the URL and a per-device token (plus the server's public key if
   app-layer encryption is wanted, in Orca's v2 layout), and a 0600 registry with pending, rotate and
   revoke, where revoke closes open sockets. Scope phone tokens with `marley_mcp`'s `GrantTable` and new
   write classes (`approval.answer`, `agent.prompt`, maybe `terminal.input`), in place of Orca's
   shell-equivalent allowlist.

4. **A phone client as a PWA** served by that endpoint: the fleet list from `fleet_snapshot` (`marley_fleet`
   sessions ordered by attention: error, question, stale); a session page with the transcript as chat (the
   hook payload names `transcript_path`, so Marley knows which JSONL belongs to which terminal); approval
   cards; a composer that sends a prompt to the agent's terminal; and a terminal peek in xterm.js fed a
   snapshot plus live output at the desktop's size, with no phone-fit resize. Per-turn diffs, when that
   ticket lands, are the next screen.

5. **Remote terminals that outlive the connection.** Change `marley_remote`'s argv to
   `ssh -t -p N -- host tmux -L marley new-session -A -s <pane-id>` so a dropped link leaves the shell and
   its agent running and the next connect reattaches; later swap tmux for `rh attach`. tmux gets in the
   way of the plugin's notifications twice: it sets `TERM_PROGRAM=tmux` in its panes, which fails
   `notify.sh`'s gate, and it does not pass an unwrapped OSC 777 to the outer terminal. So the gate
   needs a Marley variable of its own (set with `new-session -e`), and inside `$TMUX` the hook must wrap
   the sequence in tmux's DCS passthrough with `allow-passthrough on`. Check both against the tmux
   version on the remote hosts.

6. **Remote ports.** Poll `ss -Htlnp` over the SSH connection Zed already holds, ignore the ports open at
   the first scan, offer a forward through Zed's `build_forward_ports_command`, map privileged ports above
   10000, and open a Browser tab at the local port. This joins the queued localhost-URL ticket, which
   should adopt Orca's rule that a printed URL counts only while a process listens on its port.

### 3.3 What of Orca's to reuse under MIT, and the cost

| Piece | Use | Cost |
|---|---|---|
| E2EE v2 contract, framing and key schedule (470 lines of TypeScript) with fixtures | Port to Rust (`x25519-dalek`, `hkdf`, `sha2`, `crypto_secretbox`) if the phone link should be encrypted beyond Tailscale and TLS; check against `src/shared/mobile-e2ee-v2-fixtures.ts` | M |
| Terminal stream header, opcodes, ack windows, snapshot on overflow | The design for the phone's terminal peek | Part of item 4 |
| `DeviceRegistry` semantics | Pending-token reuse, rotate, refuse to overwrite an unreadable file | Part of item 3 |
| `/proc/net/tcp` scanner and advertised-URL watcher | Port the logic | Part of item 6 and the queued ticket |
| Claude and Codex transcript decoders | Reference for a Rust decoder | Part of item 4 |
| Approval-prompt heuristics (`mobile-native-chat-permission.ts`) | Fallback for agents without a structured hook | S |
| issue-5049 and the streaming findings | Test checklist for the phone client | None |
| The store iOS app, unchanged, against a Marley shim | Marley would implement v1 E2EE and about 20 of the mock server's methods (`status.get` reporting protocol 2 or higher, `repo.list`, `worktree.ps`, `session.tabs.list` and `subscribe`, `terminal.list`, `subscribe`, `send` and `unsubscribe`, `settings.get`, `ui.get` and `set`), with `mobile/rpc-foundation/goldens/` as the record of what the app sends | L, and it breaks whenever Orca ships; no push unless Marley registers with `push.onorca.dev`, which accepts any host key but is Stably's service |
| The relay, unchanged | Not needed on the tailnet; self-hosting means writing the private auth service's relay-token endpoint | Skip |
| The app, forked | 186K lines, an Apple developer account and APNs key, and a codebase moving at about 180 commits a month | Skip |

A first useful slice, items 1 and 2 (pushes, and approvals answered from the rail or from the phone once
the notification opens the page), is about a week. Items 1 to 4 together are about four weeks.

### 3.4 Ranked list

| # | Feature | Why | Lands in | Size | Hard parts and queue notes |
|---|---|---|---|---|---|
| 1 | Structured approvals through the `PermissionRequest` hook | Answer Claude Code from the rail or the phone without typing into the TUI | Plugin `hooks.json` (`mcp_tool` hook), a waiting tool in `marley_mcp`, the rail | M | Queued (approvals inbox). The bridge gives up after 40 s (`REQUEST_TIMEOUT` in `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge`) and Marley waits 30 s for the app, so approvals need a long-poll path. Check whether the TUI dialog stays answerable while the hook waits. Claude Code on a remote host cannot reach the desktop's loopback MCP without an `ssh -R` forward. The ticket should take Orca's prompt revision, "unconfirmed" state, and follow-up guard. |
| 2 | Push to the iPhone on needs-input, finished, error | Walk away from the desk | A notifier beside the OSC 9/777 handler: ntfy first, Web Push later | S (ntfy), M (Web Push) | iOS Web Push needs a home-screen PWA over HTTPS. Decide what text leaves the box. Take Orca's dismiss-on-handled, away filter and event ids. |
| 3 | Phone endpoint: QR pairing, scoped device tokens, `tailscale serve` in front | Every phone feature rests on it | New crate; `GrantTable` write classes | M | Keep the loopback bind. Revoking must close live sockets. A phone should not get a shell by default. |
| 4 | Phone client (PWA): fleet list, chat view, approvals, send, terminal peek | The remote control itself | Served by item 3; reads `marley_fleet` | L | Terminal snapshots need ANSI serialization of the alacritty grid; transcript decoding; reconnect handling per issue-5049. |
| 5 | Remote terminals that survive disconnects | Agents on remote hosts keep running; reattach after sleep | `marley_remote` argv with tmux; rustal-harness later | S | The remote command is a shell string; pane identity for reattach; tmux on each box, which also breaks the notification path (row 6). The queued worktree-agents ticket can later gain a "Run on" host the way Orca's worktree dialog has. |
| 6 | Marley notifications from remote agents | The rail and the phone hear from remote Claude Code | A Marley variable in `marley_remote`'s remote command, a gate on it in `notify.sh`, the plugin installed on each host | S | Inside tmux, `TERM_PROGRAM` reads `tmux` and OSC 777 needs DCS passthrough. Confirm that Zed remote-project terminals already deliver OSC 777. |
| 7 | Remote port detection and one-click forward into a Browser tab | Dev servers on a remote host, opened in Marley | Zed remote connection plus `marley_browser` | M | Queued (localhost URLs in a Browser tab; port offsets). Needs a poller per host and a forwards list on the project row. |
| 8 | Transcript chat view on the desktop | Read an agent's turns without scrolling a TUI | A view over the JSONL beside the Agent Panel | M | Claude Code's transcript format changes without notice. |
| 9 | `live`, `unverifiable`, `exited` for remote seats | The rail must not call a disconnected agent dead | `marley_fleet` states, rail icons | S | Only host evidence may produce `exited`. |
| 10 | Browser tab screencast on the phone | Check a page away from the desk | Forward `marley_browser`'s CDP frames to the PWA | M | Touch-to-mouse mapping; bandwidth over the tailnet. |

## 4. Skip

- The relay, director, cells and region placement: the tailnet already connects the phone, and
  self-hosting needs Orca's private auth service.
- Orca's push gateway and its FCM path: the APNs key is Orca's, and ntfy or Web Push need no Apple
  developer account.
- Forking the phone app: 186K lines at about 180 commits a month.
- E2EE v1: frames can be replayed inside a socket. If Marley adds app-layer encryption, use the v2 layout.
- Phone-fit resizing of the desktop PTY: it reflows Claude Code on the desktop to phone width.
- A Node relay daemon on every SSH host: Zed's `remote_server` plus tmux or `rh` covers it without a Node
  toolchain on the remote hosts.
- `orca serve` as Electron under Xvfb: Marley's headless parts are `remote_server` and `rh`.
- Letting an SSH host run the laptop's full CLI: it puts every remote box inside the laptop's trust
  boundary.
- Ephemeral VM recipes: Marley's remote hosts are long-lived, and the official pack does not run.
- The desktop-served web shell inside the phone app: a PWA served by Marley updates with Marley.
- Desktop-side dictation: the iPhone keyboard dictates.

## 5. Open questions

1. Is the phone on the tailnet, or should it be? The slice above assumes it is.
2. PWA or native app? The PWA needs no App Store and gets pushes from iOS 16.4 on, but only after Add to
   Home Screen.
3. What should a phone token allow: answering approvals and prompting agents only, or raw terminal input
   as well?
4. Should approvals from agents on remote hosts reach the desktop's rail? That needs a reverse forward
   set up by `marley_remote` and the plugin installed on each box.
5. What may leave the box in a push: project and agent names only, or the tool and command awaiting
   approval?
6. Is one afternoon worth spending pointing the stock Orca iOS app at a Marley shim, to learn what the
   phone should show before building the PWA?
