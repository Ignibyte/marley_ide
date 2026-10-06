# T3 Code survey 04: environments, remote access and the phone

Read from a local clone of `github.com/pingdotgg/t3code` at `17c0878941` (2026-10-06). Paths are relative to that
repo root unless they start with `crates/` (Marley).

## 1. Summary

T3 Code splits every install into an environment and its clients. The environment is a Node
server (`t3`, or the one the Electron app bundles) that owns the provider processes, terminals,
git, files and an event-sourced SQLite store; web, desktop and mobile clients drive it over one
authenticated WebSocket RPC. Agents are children of the server, so they keep working with no
client attached, and `t3 service install` keeps that server up as a systemd user service with
lingering. Reaching it from elsewhere is a choice of route (LAN, `tailscale serve`, desktop-managed
SSH, or T3 Connect, a hosted relay that provisions Cloudflare tunnels and sends phone pushes), and
every route ends at the same per-environment auth: a five-minute one-time pairing token, 30-day
scoped sessions, short-lived WebSocket tickets, and a table that names the scope of each of the 177
RPC methods. The phone side is a React Native app of about 110,000 lines with threads, approvals,
questions, an offline queue, native terminals, a streamed view of the server's browser, Live
Activities and a usage widget. Much of this answers a problem Marley does not have, since Marley is
a desktop app on a machine whose phone is already on the tailnet, and Claude Code's own Remote
Control already puts Claude Code sessions, their prompts and their pushes on the Claude app. Worth
taking: (1) pushes for every agent Marley tracks, Codex included, with T3's per-kind toggles and
grouping; (2) Claude Code's Remote Control switched on and shown from Marley, which covers Claude on
the phone for the price of a flag; (3) `OOMPolicy=continue` on the systemd units Marley starts;
(4) T3's pairing and scope model, which confirms and sharpens Orca report 04's plan for a phone
endpoint behind `tailscale serve`; (5) a small phone page for what Claude's app cannot show:
Codex approvals, held browser clicks, terminal-write requests and a Browser tab stream.

## 2. Features

### 2.1 The environment and its clients

**What the user sees.** One app on three surfaces. The desktop app opens on its own machine's
environment; the same threads, terminals and browser tabs show on a laptop browser or a phone
connected to that machine, and an agent keeps working while no client is open. Settings lists
"environments", and a thread belongs to exactly one of them.

**How it works.** `docs/internals/overview.md` sets the rule: "Provider processes, terminals, Git,
and project files belong to the server", and a remote client "must never substitute its own
filesystem, provider credentials, or machine state for the environment's". The wire contract is
`packages/contracts/src/rpc.ts` (1,912 lines), an Effect RPC group carried on one WebSocket per
environment. Clients share a connection runtime in `packages/client-runtime` (37,400 lines; the
connection part is 4,700): one supervisor owns retries per environment, and React views only read
state (`docs/internals/connection-runtime.md`). Subscriptions are per view: a client that shows one
thread subscribes to that thread's stream, mounted views share one live stream, and the cache keeps
state and its replay cursor for five idle minutes so going back needs no new snapshot
(`packages/client-runtime/src/state/threads.ts`). The desktop renderer is a client like the others:
it loads the bundled UI from a `t3code://` scheme and talks to the environment's own URL, which is
why the desktop can drop its local server and act as a remote-only client
(`docs/internals/remote.md`, "Desktop without a local environment").

The environment keeps an ID that survives restarts and address changes, written atomically with a
recovery file so two initialisers pick the same winner (`apps/server/src/environment/
ServerEnvironment.ts`). Routes are "reachability hints"; identity is checked on every connect.

**Good.** The split is clean and enforced: the RPC group is the only door, and the docs keep
transport health, data freshness and credential renewal as separate states, so a failed shell
subscription never reads as "reconnecting" (`connection-runtime.md`, "Transport health and data
freshness are separate").

**Bad.** The agents' lives are tied to the server process. A server restart, update or crash
interrupts every turn and terminal; "Continue threads after restarts" is off by default and only
resumes providers with saved resume state (`docs/user/updating.md`).

**Size.** Server about 252,000 lines of non-test TypeScript, web 240,000, packages 171,000.

**Marley today: lacks.** Marley is one GPUI process. Its agents run in its terminals and end with
it (it asks first, #550, and resumes Claude Code sessions at the next launch, #540). The nearest
thing to an environment that outlives the UI is the harness: with `embedded_harness` on, Marley runs
`rh serve` and its sessions, kept by tmux, outlive Marley ("The harness's sessions", #632).

### 2.2 Running headless: `t3`, `t3 serve`, the user service and updates

**What the user sees.** `t3` starts the server and opens the local web app; `t3 serve` starts it
with no browser; `t3 service install` installs a background service; `t3 service status` reports
problems by name (`linger-disabled`, `restart-pending`); `t3 update [version]` moves to a release and
asks before restarting the service, since a restart interrupts running turns
(`docs/user/background-service.md`). A client whose server is older offers **Update server**, which
updates and relaunches the host remotely (`docs/user/updating.md`).

**How it works.** The unit is rendered by `renderBootServiceUnit` in `apps/server/src/cloud/
bootService.ts:97-126`: `Restart=always`, `RestartSec=5`, `KillMode=mixed`, logs appended to a file,
`WantedBy=default.target`, and `OOMPolicy=continue` with the reason in a comment at lines 114-118:
agent tool calls run as children of the server in the same cgroup, and "With the systemd default of
OOMPolicy=stop, the kernel killing one greedy child stops the whole unit: the server, every live
agent, and the user's connection." Install checks `loginctl` and enables lingering when it can, or
prints the `sudo loginctl enable-linger` line to run (`bootService.ts:693-740`).

The service runs a launcher, not the server: `apps/server/src/serviceLauncher.ts` (637 lines, Node
built-ins only, so it works across server versions) owns which runtime version boots. An update is
a trial: the launcher records the pending update durably, snapshots SQLite's main file, WAL and
shared memory, starts the target, and waits up to 120 s (`PREPARED_TIMEOUT_MS`, line 34) for the
child to finish migrations, bind HTTP and park its long-running work. Only then does it commit the
version; a failed or slow trial restores the snapshot and the old version
(`docs/internals/server-updates.md`). Runtimes are unpacked release archives under
`~/.t3/runtime/versions/<version>`, so no npm cache or Node install is involved.

**Good.** `OOMPolicy=continue` and the trial-commit-rollback launcher are both answers to failures
someone hit. The status command names the fault and the fix.

**Size.** Launcher 637 lines, unit and preflight about 1,000, `t3 update` 599.

**Marley today: has part.** Marley is a desktop app with no service mode; `just install` replaces
files by rename, and a running Marley keeps its old binary ("Install and run"). Marley starts
Chromium as a transient user unit with `KillMode=mixed` and `TimeoutStopSec=10` but no
`OOMPolicy` (`crates/marley_browser/src/service.rs:340-356`), so systemd's default applies: a kernel
OOM kill of one renderer stops the project's whole Chromium unit and every tab in it.

### 2.3 Version negotiation and capabilities

**What the user sees.** A client and a server on different releases still connect when they speak
the same orchestration protocol. When they do not, the connection is refused with a notice naming
the machine to update, never half-connected (`docs/user/updating.md`, "When versions don't match").

**How it works.** Two layers. `ORCHESTRATION_PROTOCOL_VERSION` (2, `packages/contracts/src/
environment.ts:13`) must match exactly; `packages/client-runtime/src/connection/compatibility.ts:
9-27` turns a mismatch into a blocked connection and, when the host can self-update, offers the
update. Inside a protocol version, the environment descriptor carries about 40 optional capability
flags (`ExecutionEnvironmentCapabilities`, `environment.ts:98-216`), each documented with its
version-skew rule ("Absent on pre-settlement servers, so clients treat missing as unsupported and
never send the commands"). The overview's PR-linking table shows the pattern in use: three client
behaviours keyed on which flags the server advertises.

**Good.** Capabilities read as a contract, with the absent case written down per flag. It is the
same discipline Orca's `docs/reference/remote-wire-compatibility.md` asks for.

**Marley today: has part.** Marley's MCP server speaks MCP's own versioned protocol; the fleet
contract has versions and an `incompatible` state ("Real stores (#611)"). There is no client of
Marley's own on another device to negotiate with.

### 2.4 Pairing, sessions and scopes

**What the user sees.** On a desktop host, Settings → Connections → Network access, then a pairing
link and QR code; on a headless host, `t3 pair` prints a QR code, the URL, the token and its expiry
for the running server without restarting it. The other device scans or pastes the link once, and
afterwards reconnects on its own. Settings lists pairing links and client sessions, and each can be
revoked (`docs/user/remote-access.md`, "Pair over a LAN or private network", "Manage or revoke
access").

**How it works.**

- Credentials. A pairing link is a one-time token valid five minutes
  (`DEFAULT_ONE_TIME_TOKEN_TTL_MINUTES`, `apps/server/src/auth/PairingGrantStore.ts:243`). Exchanging
  it yields a session: a browser cookie, a bearer token or a DPoP-bound token, 30 days by default
  (`DEFAULT_SESSION_TTL`, `apps/server/src/auth/SessionStore.ts:423`). Bearer and DPoP clients fetch a
  WebSocket ticket that lives five minutes (`SessionStore.ts:424`) over authenticated HTTP, "so
  long-lived tokens stay out of socket URLs" (`docs/internals/environment-auth.md`). The read model
  of links holds metadata only; the raw secret appears once, in the creation response.
- Scopes. Eight: `orchestration:read`, `orchestration:operate`, `terminal:operate`, `review:write`,
  `access:read`, `access:write`, `relay:read`, `relay:write` (`packages/contracts/src/auth.ts:
  81-98`). A paired client gets the standard set, everything but access and relay management
  (`auth.ts:103-109`; `EnvironmentAuth.ts:890`). Creating a link needs `access:write` and every scope
  it delegates, so pairing can narrow a grant and never widen it.
- Enforcement. `RPC_REQUIRED_SCOPES` in `apps/server/src/auth/RpcAuthorization.ts:28-211` maps every
  WebSocket method to one scope (70 read, 93 operate, 9 terminal, 2 review, 3 access and relay), and
  the `satisfies Readonly<Record<WsRpcMethod, AuthEnvironmentScope>>` at line 211 makes a method
  without a scope a type error. A middleware checks the scope before any handler runs. The browser
  stream checks it per socket: a viewer without `orchestration:operate` watches and cannot click
  (`apps/server/src/preview/ServerBrowserStream.ts:72-77`).
- No trust in loopback. The auth posture depends on where the server binds, not on who connects
  (`apps/server/src/auth/EnvironmentAuthPolicy.ts:22-36`): a desktop-managed local server accepts
  only the desktop's own bootstrap handoff, and a standalone server always wants a pairing token,
  even from 127.0.0.1. This matters behind `tailscale serve`, where every request arrives from
  loopback.
- The hosted pairing URL puts the backend in the query and the secret in the fragment, which never
  reaches the hosting origin; the client strips it from history after the exchange
  (`docs/internals/remote.md`, "Hosted web is a client"; `apps/web/src/hostedPairing.ts`).

**Good.** The scope table checked by the compiler is the piece to copy. So is refusing implicit
loopback trust, and keeping the long-lived token out of every URL.

**Bad.** The default pairing hands a phone `terminal:operate` and `orchestration:operate`, which is a
shell on the host and the right to change provider settings. Direct LAN routes are plain HTTP: the
`DirectEndpoints` comment says so (`apps/server/src/environment/DirectEndpoints.ts:48-58`), and
Desktop's Network access binds 0.0.0.0 (`apps/desktop/src/backend/DesktopServerExposure.ts:32`).
There is no application-layer encryption anywhere, so confidentiality comes from Tailscale,
the tunnel's TLS or nothing.

**Size.** Auth 4,400 lines; `t3 pair` 534.

**Marley today: lacks.** Marley's MCP server takes a per-boot bearer from a 0600 endpoint file on
loopback, read tools free and write tools behind a class grant ("Grants and what keeps an agent in
check"). A program on another machine reaches it only through the SSH bridge line Browser Clients
shows (#584), which relies on SSH keys rather than pairing.

### 2.5 Tailscale

**What the user sees.** `t3 serve --tailscale-serve`, or `t3 pair --tailscale` for a server that
already runs, publishes the server at `https://<machine>.<tailnet>.ts.net/` and prints a pairing link
on that name. The mapping persists until `tailscale serve --https=443 off`
(`docs/user/remote-access.md`, "Tailscale HTTPS").

**How it works.** `packages/tailscale/src/tailscale.ts` (382 lines) wraps the CLI.
`ensureTailscaleServe` (lines 341-350) runs `tailscale serve --bg --https=<port>
http://127.0.0.1:<port>`. Before touching a mapping, `t3 pair` probes `/.well-known/t3/environment`
at the tailnet URL and sorts the answer three ways (`apps/server/src/cli/pair.ts:200-226`): a T3
descriptor (pair with it), nothing answering or a 502 to 504 from the proxy (a stale mapping it may
replace), or some other service (refuse, and suggest `--tailscale-serve-port`). Tailscale's stderr
is never logged raw, since it can carry auth keys and node names; it is reduced to four labels
(`tailscale.ts`, `TailscaleStderrDiagnostic`). The server also reports the tailnet and private LAN
IPv4 addresses it is bound to, which clients save as extra routes (2.6;
`DirectEndpoints.ts:54`).

**Good.** This is Orca report 04's item 3 as shipped by someone else: bind loopback, let
`tailscale serve` front it with a real certificate, pair on the ts.net name. The three-way probe
before rewriting a serve mapping is the detail Marley would otherwise miss.

**Marley today: has part.** Marley's ntfy push assumes ntfy on the dev box published to the phone by
`tailscale serve` (#535), but Marley itself runs no `tailscale` command and serves nothing on the
tailnet.

### 2.6 Several routes to one machine

**What the user sees.** A saved machine can hold several routes (T3 Connect, LAN, Tailscale, a
public URL, SSH). The app connects over the first that answers, skips a silent home LAN when away,
moves back to a faster route when one returns, and learns the machine's current LAN and tailnet
addresses after connecting once over any route (`docs/user/remote-access.md`, "Reach one machine
several ways").

**How it works.** `packages/client-runtime/src/connection/routes.ts` (374 lines) orders routes by
kind (`relay`, `loopback`, `lan`, `tailnet`, `public`, `ssh`); `driver.ts` walks them; each direct
route is first checked with the public descriptor, so a LAN address that another machine answers on
another network never receives the credential (`docs/internals/remote.md`). `supervisor.ts`
(1,039 lines) owns retries: a jittered delay whose ceiling doubles from 2 s to five minutes
(`retryDelayMs`, line 137), reset after 30 s connected; while on a fallback route it re-checks the
better ones every 60 s and on network changes, and holds a route that answered but failed for five
minutes so a flaky LAN cannot bounce the connection (lines 35-49). A long mobile background
suspension replaces the session at once rather than probing a socket the OS may have killed
(`connection-runtime.md`, "One transport retry owner").

**Good.** The retry rules are written down with their reasons, and they are the rules any phone
client of Marley's would need (Orca's issue-5049 is the same lesson learned the hard way).

**Marley today: lacks.** Nothing of Marley's connects from another device. Marley's own reconnect
rules live in the Rusty connection (1, 2, 4 up to 60 s) and remote terminals (1 s doubling to two
minutes, #641), both on the desktop side.

### 2.7 SSH environments, managed by the desktop

**What the user sees.** Settings → Connections → Add environment → SSH, with a host or an SSH alias.
T3 starts or reuses a server there and forwards its port; projects, credentials and agents stay on
the remote host. The host needs `curl` or `wget`, `tar` and `sha256sum` and nothing else; the first
launch downloads the server (`docs/user/remote-access.md`, "Desktop-managed SSH"). Windows users can
pick a WSL distro the same way.

**How it works.** `packages/ssh/src/tunnel.ts` (1,768 lines). A shell script sent over SSH installs
the exact release archive (about 70 MB, checksum verified) under `~/.t3/runtime/versions/<version>`
behind a lock, picks a free remote port from 3773 upward, starts the server, waits for it, and mints
a pairing token with `t3 auth pairing create --json` (lines 440-520, 731). The forward is system
OpenSSH with `-N -L <local>:127.0.0.1:<remote>`, `ExitOnForwardFailure=yes`, no connection sharing,
and `ServerAliveInterval=15` with `ServerAliveCountMax=3` (lines 1137-1158). Password and passphrase
prompts go through an askpass helper the desktop answers (`packages/ssh/src/auth.ts`). Cleanup stops
the remote server only if this launcher started it.

**Good.** No Node on the host and no per-host build, unlike Orca's relay daemon. A server found
already running is left alone on disconnect.

**Marley today: has.** Zed's remote projects ("Open Remote Folder" under Add Project) upload Zed's
`remote_server` to the host over SSH and run files, language servers, git and tasks there. Marley
adds remote terminals in a tmux session that survive a dropped link (#543, #641), blocks over ssh
(#526), passphrase prompts asked in Marley (#596), and host metrics over SSH in the Fleet panel
(#610). What T3 adds is agents that keep running on the host through T3's own server; Marley's
equivalent is the agent CLI in a remote tmux terminal, or a harness on that host.

### 2.8 T3 Connect, the hosted relay

**What the user sees.** Sign in to a T3 Connect account on the host (`t3 connect`, with a device
code over SSH) and on the phone; the phone lists the account's machines and connects with no router
or Tailscale setup. Pushes, Live Activities and webhook URLs for scheduled tasks need it
(`docs/user/remote-access.md`, "T3 Connect"; `docs/user/mobile-notifications.md`).

**How it works.** `infra/relay` (14,900 lines) is a Cloudflare Worker stack deployed with Alchemy:
Clerk for identity, PlanetScale Postgres, queues, Durable Objects, APNs and FCM delivery
(`infra/relay/README.md`, `docs/operations/android-notifications.md`). For each linked environment it
provisions a Cloudflare tunnel whose connector runs on the host and exposes only the server's
loopback origin. After bootstrap the client talks to the environment through the tunnel hostname;
the relay is not in the data path. Its role in auth is to ask the environment, with signed and
replay-guarded proofs, to mint a one-time bootstrap credential bound to the client's DPoP key; the
client redeems it with the environment directly, so the relay never holds an environment session
(`docs/internals/t3-connect.md`, "The relay is a trusted broker"; `apps/server/src/cloud/
CloudLink.ts`). Idle tunnels are reclaimed after five minutes down and recreated on wake under the
same hostname. Webhooks for scheduled tasks are the one thing the relay forwards, and only if the
environment opts in does it hold them up to 24 hours in a Durable Object while the host is away.

For pushes, the environment publishes each thread's state to the relay, signed with its key, when
"agent activity publishing" is on (`apps/server/src/relay/AgentAwarenessRelay.ts`). The payload is
the project title, the thread title, a phase (`starting`, `running`, `waiting_for_approval`,
`waiting_for_input`, `completed`, `failed`, `stale`), a fixed headline such as "Approval needed",
the model name and a deep link (`packages/contracts/src/relay.ts:22-31, 122-133`;
`packages/shared/src/agentAwareness.ts:58-85, 122-139`). No prompt or agent text leaves.

**Good.** The relay holds no environment session, and the push payload is fixed strings plus two
titles. The docs state the remaining trust assumption: a compromised relay signing key is not made
harmless by DPoP.

**Bad.** It is a hosted service with an account, five vendors and a deploy pipeline, and pushes,
Live Activities and webhooks exist only through it: "a direct or Tailscale connection alone does not
enable push notifications" (`mobile-notifications.md`).

**Size.** Relay 14,900 lines; the server's cloud side 6,300.

**Marley today: lacks, by design.** On a tailnet the relay has no job, as Orca report 04 found for
Orca's. Marley's pushes go to a self-hosted ntfy (#535).

### 2.9 The hosted web app and the remote-only desktop

**What the user sees.** `app.t3.codes` is the same web client served from T3's host; it connects
straight to environments over HTTPS and keeps its connection list in the browser. A desktop app can
switch off its local environment and only drive other machines (`docs/user/remote-access.md`,
"Hosted web app", "Using the Desktop App as a Remote Only").

**How it works.** Covered in 2.1 and 2.4: the hosted page proxies nothing, so it cannot reach a
plain-HTTP LAN server from an HTTPS page.

**Marley today: lacks.** Marley has no web client. The question for Marley is whether a page served
by Marley over `tailscale serve` is worth having (section 3, item 5).

### 2.10 Spreading new threads across machines

**What the user sees.** With two or more machines switched on, Settings → Connections → Load
balancing picks a machine for each new thread by its free CPU and memory. Each machine is Normal,
Prefer, Less often or Manual only; the choice is made once per draft and stays
(`docs/user/remote-access.md`, "Balance new threads across machines").

**How it works.** `packages/client-runtime/src/load-balancing.ts` is 40 lines. A machine counts
only with a resource sample under 15 s old, CPU under 95 % and more than 5 % memory free; its score
is `weight × cpuCount × (1 − cpuUtilization) × availableMemory / totalMemory` (lines 25-33). The
samples come from a small Rust process using `sysinfo` (`native/resource-monitor/src/main.rs`,
`docs/internals/resource-telemetry.md`).

**Good.** Small, and it only chooses where a thread starts; a running thread never moves.

**Marley today: has part.** The Fleet panel already reads each host's CPU, memory, disk and network
over SSH every 5 s ("Hosts over SSH (#610)"). Nothing places work with it: New Agent in Worktree
always runs on this machine ("Worktree agents").

### 2.11 The phone app

**What the user sees.** iOS and Android apps (App Store, Google Play, TestFlight and a beta group for
nightly builds). Threads with the same composer, model picker, approvals, questions with file
answers, queued and steered messages, `/` commands and skills; diffs and pull-request review; files;
terminals; the server's browser tabs in a floating view with picture-in-picture; device simulators;
usage and limits; settings filtered by environment; "Update" for a connected environment. Drafts and
queued messages survive restarts and queue while offline, uploads resume on reconnect
(`docs/user/composer.md`, "Queue messages offline on mobile"). Other apps share into T3 through the
system share sheet. On iOS 26 the composer's microphone transcribes on the device
(`docs/internals/voice-input.md`). The app downloads updates in the background and applies them when
the user next leaves it (`docs/user/updating.md`, "Mobile updates").

**How it works.** Expo 58 on React Native 0.88 (`apps/mobile/package.json`). About 109,000 lines in
`apps/mobile/src` (threads 30,000, settings 8,800, review 6,000, terminal 2,500, browser 1,900,
agent awareness 1,750, voice 1,300) and eight native modules in `apps/mobile/modules`. The terminal
is native: Ghostty's `libghostty` surface on iOS and `libghostty-vt` with a Canvas view on Android,
fed by the same terminal RPC stream as the web client (`apps/mobile/modules/t3-terminal/README.md`).
The phone shares the connection runtime and state code with the web client
(`packages/client-runtime`), which is why a feature lands on both. Terminal size follows the last
client to resize: the phone's `resize` call sets the server PTY's columns and rows outright
(`apps/server/src/terminal/Manager.ts:2953-2967`; `apps/mobile/src/features/terminal/
ThreadTerminalRouteScreen.tsx:903`).

**Good.** Phone and desktop run the same orchestration commands, so an approval from the phone is
the same object as one from the desktop. Agents in T3 are SDK sessions with structured approvals, so
the phone never types keystrokes into a TUI (Orca's phone does).

**Bad.** A second product: native modules, a patched `react-native-screens`
(`docs/internals/mobile-navigation.md`), app-store releases, and an Apple developer account. A phone
that opens a shell terminal reflows it for everyone.

**Size.** About 112,000 lines with the native modules.

**Marley today: has part.** Marley has no phone app. For Claude Code, though, Claude Code's own
Remote Control gives the phone what T3's app gives a T3 thread. `claude --remote-control` (or `/rc`
in a running session, or `remoteControlAtStartup: true` in `~/.claude/settings.json`) keeps an
interactive session in the terminal and makes it available in the Claude app and at claude.ai/code,
with prompts, permission prompts and `AskUserQuestion` forwarded, files and photos from the phone,
and pushes "when actions required"; the session makes outbound HTTPS requests only, through
Anthropic's API, and needs a Pro, Max, Team or Enterprise login
(code.claude.com/docs/en/remote-control, read 2026-10-06). That works for Claude Code in a Marley
terminal today. Codex, Marley's own inbox entries and the Browser tab have no phone route.

### 2.12 Push, Live Activities and the widget

**What the user sees.** With T3 Connect, a phone gets alerts when an agent finishes, fails, needs
approval or asks for input, each kind a toggle, and a tap opens the thread. iOS Live Activities and
Android ongoing notifications follow running work without opening the app (Android 16 shows a
status-bar chip that reads Working, Approve or Answer), and finished results stay up to 15 minutes.
Alerts stay quiet only for the thread on screen on that phone; viewing the thread on another device
does not silence the phone (`docs/user/mobile-notifications.md`). A home-screen widget shows the
remaining Claude and Codex quota (`docs/user/usage.md`, "Subscription usage widget").

**How it works.** The relay turns published states (2.8) into APNs and FCM messages and Live
Activity updates (`infra/relay/src/agentActivity/`). Two threads that reach approval together make
one "2 agents need attention" alert, two that finish together one "2 agents finished"; publishing the
same state again makes no alert; completions older than two minutes never alert; quiet running work
expires after two hours and approval or input states after 24 (`docs/operations/
android-notifications.md`, "Focused delivery check"). The preferences are six booleans
(`RelayAgentAwarenessPreferences`, `relay.ts:33-41`).

**Good.** The grouping and the "never alert on an old completion" rule fix the two ways a push
channel gets muted by its user: bursts and stale noise.

**Marley today: has part.** Marley pushes one line to a local ntfy when Claude Code in a terminal
the user is not looking at needs input, finishes or fails, with a five-second cooldown per project
(#535; `crates/marley_workbench/src/push.rs`). The agent kind is fixed to Claude (`push.rs:89`,
`event_line(&project, AgentKind::Claude, event)`), so a Codex on its App Server (#650), whose waits
already reach the inbox (#651), pushes nothing. The settings are a URL, a topic and a token file;
there are no per-kind switches, no grouping across projects, and no click target.

### 2.13 Terminals and the browser from another device

**What the user sees.** A terminal opened on the desktop can be attached from the phone or a laptop
browser, with up to 5,000 lines of server-side scrollback (`docs/user/terminal.md`). Browser tabs
belong to the environment: every device sees the same tabs, agents keep using them with no device
connected, and a person must take control before typing into an agent's tab, then release it. A
read-only connection watches. While in control, the page's clipboard, file picker and downloads go
to the viewer's device (`docs/user/remote-access.md`, "Browser on a remote environment").

**How it works.** PTYs belong to the server (`apps/server/src/terminal/Manager.ts`), so a terminal
outlives its client but not the server; history is restored after a restart without the process
(`docs/internals/terminal-runtime.md`). The browser is a headless Chrome the server downloads once
(about 120 MB). Remote viewers get a JPEG screencast over a WebSocket at `maxWidth` 1280 and
`maxHeight` 800 by default, quality 70 (`ServerBrowserStream.ts:32, 86-91`); Chromium's ack for each
frame waits for the viewer's own ack, and a viewer that falls behind is disconnected rather than
buffered. `SessionControl` (`apps/server/src/preview/SessionControl.ts`, 135 lines) serialises
actions per tab and drains running work before control changes hands; an agent whose control was
taken gets "Browser control changed. Refresh the snapshot before trying again."

**Good.** Take and release with a generation, plus per-frame acks, is a small and complete model for
sharing one page between an agent and a remote human.

**Marley today: has part.** Agents and the user share the Browser tab on the desktop, with Ctrl-I
Take Over and Hand Back for agent writes into terminals (#525), the Agent chip, and the click
consequence card (#571). Outside programs can drive the browser through Browser Clients, also over
SSH (#524, #584). Nothing streams a terminal or a Browser tab to a phone.

## 3. Bring to Marley

Orca report 04 ranked a phone path for Marley: push first, then structured approvals, then a
loopback endpoint behind `tailscale serve` with QR pairing and scoped tokens, then a small web app.
Push shipped (#535), the inbox shipped (#508, Codex answered from it in #651), and remote terminals
that survive a drop shipped (#543). T3 confirms the endpoint design and adds two facts that change
the order. Claude Code's own Remote Control covers Claude on the phone, so a Marley phone client is
worth building only for what Claude's app cannot show. And T3 ships without application-layer
encryption, relying on Tailscale or TLS, which supports dropping Orca's optional E2EE port from the
plan.

1. **Push for every agent Marley tracks, with T3's kinds and grouping.**
   *Why.* Chad runs Codex as well as Claude Code, and today only Claude Code pushes. T3's
   preferences and delivery rules keep a push channel worth leaving on.
   *Seam.* `crates/marley_workbench/src/push.rs`: take the agent kind from the seat instead of
   `AgentKind::Claude`; feed it from Codex's App Server thread state (#650) and Zed agent threads
   waiting on confirmation; add `marley.push.notify_on` with `approval`, `input`, `completion` and
   `failure`, after `RelayAgentAwarenessPreferences` (`packages/contracts/src/relay.ts:33-41`);
   group events from several projects inside the cooldown into one line ("2 agents need you:
   marley_ide, rusty"); drop a completion older than two minutes. ntfy's `Click` header can carry a
   URL once item 5 exists.
   *Size.* S.
   *Hard.* Keeping text out: send the kind and the project, as T3 does, and no agent text.

2. **Claude Code's Remote Control, switched on and shown from Marley.**
   *Why.* It puts Claude Code's prompts, permission prompts, questions and pushes on the phone for
   sessions running in Marley's terminals, with no Marley server and no pairing. It is Claude Code's
   feature; Marley's part is to start it and to say when it is on.
   *Seam.* `crates/marley_workbench/src/agents.rs` and the Permission modes table: a setting
   `marley.claude_code_remote_control` that adds `--remote-control` to the line Marley types for
   Claude Code; a chip on the agent row while the session's arguments or events show it; and item 1
   skipping Claude events while Claude's own "Push when actions required" is on, so the phone does
   not ring twice.
   *Size.* S.
   *Hard.* The session travels through Anthropic's API, needs a subscription login, and is off with
   `ANTHROPIC_BASE_URL` pointed elsewhere. Marley cannot read Claude Code's `/config` push toggles
   reliably, so the dedupe may need its own switch.

3. **`OOMPolicy=continue` on the units Marley starts.**
   *Why.* T3 found that systemd's default stops a whole unit when the kernel OOM-kills one child
   (`apps/server/src/cloud/bootService.ts:114-118`). Marley's per-project Chromium is such a unit: a
   renderer killed under memory pressure would close every tab of the project and end any recording.
   *Seam.* `systemd_run_args` in `crates/marley_browser/src/service.rs:340`, one more
   `--property=OOMPolicy=continue`; the same for any unit the embedded harness gets.
   *Size.* S.
   *Hard.* Check that the relay, as the unit's main process, notices a dead Chromium and that the
   tab says so.

4. **A device endpoint on T3's auth model.**
   *Why.* Every phone feature beyond push and Claude's own app rests on it, and T3 shows the shape
   in production.
   *Seam.* A new crate beside `marley_mcp` (or a second transport of it), bound to 127.0.0.1, with
   Marley running `tailscale serve --bg --https=<port> http://127.0.0.1:<port>` after T3's three-way
   probe (`apps/server/src/cli/pair.ts:200-226`). A `marley: pair a device` modal shows a QR code
   with a one-time token valid five minutes, carried in the URL fragment. The exchange issues a
   30-day session listed in the settings page with Revoke, which closes its sockets; WebSocket
   upgrades use five-minute tickets. Every method names its scope in an exhaustive `match`, the Rust
   form of `RPC_REQUIRED_SCOPES` (`RpcAuthorization.ts:28-211`). Scopes for Marley: `read`,
   `inbox:answer`, `agent:prompt`, `terminal:input`, `browser:operate`, `access:write`; a phone link
   defaults to the first two. No request is trusted for coming from loopback, since `tailscale
   serve` makes every request do so (`EnvironmentAuthPolicy.ts:22-36`).
   *Size.* M.
   *Hard.* GPUI has no web view to serve, so item 5 is a separate client. Tailscale HTTPS
   certificates must be on for the tailnet.

5. **A phone page for what Claude's app cannot show.**
   *Why.* Codex approvals (#651), held browser clicks (#571), agent writes into running programs
   (#525) and the harness's questions (#534) wait on Chad at the desk today.
   *Seam.* A small web page served by item 4: the Needs-you inbox with its Allow and Deny
   decisions, the rail's agent rows across agents, a terminal's screen read through the same code as
   `terminal_screen`, and a Browser tab as a JPEG stream from the CDP screencast Marley already
   receives, with T3's per-frame ack and take-and-release control (`ServerBrowserStream.ts:72-140`,
   `SessionControl.ts`). Reconnection follows T3's supervisor rules: jittered backoff to five
   minutes, reset after 30 s up, and a fresh session after a long background
   (`packages/client-runtime/src/connection/supervisor.ts:35-49, 137`).
   *Size.* L.
   *Hard.* iOS Safari suspends sockets in the background, so the page must load from a snapshot
   every time it shows. A terminal shown on the phone stays at the desktop's size; T3's
   last-resize-wins would reflow Claude Code on the desktop.

6. **New Agent in Worktree on another host, placed by the Fleet panel's numbers.**
   *Why.* The Fleet panel already knows each host's load (#610); T3 shows that placement can be
   40 lines (`packages/client-runtime/src/load-balancing.ts`).
   *Seam.* The worktree prompt gains a host picker with "least loaded" first; the worktree and the
   agent's terminal open through Zed's remote project and Marley's tmux remote terminal (#543).
   *Size.* L.
   *Hard.* Worktree agents' review, merge and port offsets (#511, #590) assume a local repository.

## 4. Skip

- T3 Connect: the relay, Clerk, Cloudflare tunnels, PlanetScale and the push queues. On a tailnet
  the phone already reaches the dev box, and running a hosted service is a job of its own.
- The native phone apps, Live Activities, Android ongoing notifications and the usage widget. They
  need an Apple developer account, store releases and a push service with Apple's key, and Claude's
  app already covers Claude Code sessions.
- The multi-route catalog with learned LAN routes and route fallback. A phone with Tailscale always
  on has one route; keep only the supervisor's retry rules for item 5.
- Desktop-managed SSH environments that install a server on the host. Zed's remote projects already
  upload `remote_server`, and Marley's remote terminals keep agents alive in tmux.
- The launcher's trial, commit and rollback for server updates. Marley has no remote clients that
  depend on its version; the design suits rustal-harness's `rh` upgrades more than Marley.
- Pairing that grants a shell by default. Marley's phone links should start at read and inbox
  answers.
- Plain-HTTP LAN routes and Desktop's 0.0.0.0 bind. Marley keeps the loopback rule.
- Application-layer encryption on the phone link. T3 ships without it; Tailscale's WireGuard and the
  ts.net certificate cover the path Marley would use.
- The hosted web app, the remote-only desktop and WSL environments: no use on one Linux desktop.
- Letting a phone resize a shared PTY.

## 5. Open questions

1. Is Claude Code's Remote Control acceptable as the phone path for Claude Code, given that the
   session travels through Anthropic's API? *Default: Marley changes nothing; Chad can turn it on
   himself with `remoteControlAtStartup`.*
2. Which events should reach the phone for Codex and for Zed agent threads: approvals and
   questions only, or finishes and failures too? *Default: Claude Code's three events, as today.*
3. With Claude's app covering Claude Code, is a Marley phone page (items 4 and 5) still wanted for
   Codex approvals, held clicks and the Browser tab? *Default: nothing built.*
4. What may a paired phone do: read and answer the inbox only, or also prompt agents, type into
   terminals and drive a Browser tab? *Default: read and answer the inbox.*
5. Should worktree agents ever start on another host? *Default: this machine only.*
