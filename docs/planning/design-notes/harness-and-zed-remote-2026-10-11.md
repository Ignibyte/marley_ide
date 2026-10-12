# The harness and Zed's remote server (2026-10-11)

Chad, 2026-10-11:

> lets do research on how the rustal harness can provide first class support to the zed remote
> agent. if its useful for protocols and helping connections / communication we should do it. at
> minimum live along side it

"The Zed remote agent" here is Zed's remote development server: the `remote` crate on the client,
the `remote_server` binary on the far host, and what runs there through it. This note reads both
code bases (Marley at `19dda499a2`, rustal-harness as checked out today) and recommends what to
build. Paths without a prefix are in Marley; `rustal-harness/` marks the other repository.

## The answer in short

The harness should live alongside Zed's remote server and give it agents, not connections. Zed's
remote server already owns a complete link (its own SSH master, a framed protocol, heartbeats,
reconnect, uploads), and the harness deliberately has none of that: it opens no SSH connection,
ships no binary and builds no relay. The useful meeting point is the host's folders. A seat on host
X working in the folder a Zed remote project on X has open gives Marley an agent whose edits show
live in the remote editor and whose reports reach Marley through the harness.

What blocks all of it today is Marley's, not the harness's: Marley ships on the `dev` channel, and
a dev build cannot put a remote server on a host, so no SSH project opens at all.

| Order | What | Repo | Size |
|---|---|---|---|
| First | Marley's own remote server: the install stages a matching `remote_server`, the dev channel names it by commit and uploads it | Marley (one hunk in `remote`) | M |
| Second | Seats in a remote project: a remote window on a harness host starts its agents as seats in its folder | Marley | M |
| Third | One host picker across `ssh_connections`, `marley.fleet.hosts` and `marley.harnesses` | Marley | S to M |
| Not | A harness transport, the harness installing Zed's server, the harness speaking Zed's protocol | | |

No harness ticket is needed for any of the three. Two candidates for later are at the end.

## 1. How Zed's remote server works

### Starting a connection

`remote::connect` goes through a global `ConnectionPool` keyed by the connection options, so every
window on one host shares one connection (`crates/remote/src/remote_client.rs:382-394`,
`1246-1344`). For SSH, `SshRemoteConnection::new` (`crates/remote/src/transport/ssh.rs:797-1012`)
does five things in order:

1. It looks for a live SSH master the user's own config already names: `ssh -G HOST` gives the
   `ControlPath`, `ssh -O check` confirms it, and if it answers Zed uses it and skips
   authentication (`ssh.rs:720-794`, `811-822`).
2. Otherwise it starts its own master, `ssh -N -o ControlPersist=no -o ControlMaster=yes -o
   ControlPath=<temp dir>/ssh.sock HOST`, with password prompts through askpass (`ssh.rs:302-335`,
   `824-884`). Every later command carries `-o ControlMaster=no -o ControlPath=<that socket>`
   (`ssh.rs:1586-1638`).
3. It probes the host: `uname -sm`, `$SHELL`, the home folder and the OS version
   (`crates/remote/src/transport.rs:25-146`, `ssh.rs:955-987`).
4. It makes sure the server binary is there (`ensure_server_binary`, `ssh.rs:1014-1145`).
5. `start_proxy` runs `ssh … env RUST_LOG=… ~/.zed_server/<binary> proxy --identifier ID
   [--reconnect]` and pipes that process's stdio (`ssh.rs:628-701`, `transport.rs:148-259`).

The binary is `~/.zed_server/zed-remote-server-<channel>-<version>`, where the version is the
literal `build` on the dev channel (`ssh.rs:1021-1036`, `crates/paths/src/paths.rs:70-74`). "There"
means `<binary> version` exits 0; its output is never compared (`ssh.rs:1038-1047`, `1073-1075`).
On stable, preview and nightly a missing binary is fetched by `curl` or `wget` on the host, or
downloaded locally and sent by sftp or scp, then unpacked with `gunzip`, `chmod 755` and `mv`
(`ssh.rs:1101-1144`, `1147-1245`, `1247-1287`, `1305-1338`, `1426-1480`). Debug builds instead
compile the server from the local checkout with cargo and upload it (`transport.rs:261-525`,
`ssh.rs:1049-1071`). A release build on the dev channel stops:

```text
ZED_BUILD_REMOTE_SERVER is not set and no remote server exists at (…)
```

(`ssh.rs:1077-1086`; dev containers stop the same way at `transport/docker.rs:278-287`.) Marley is
a release build on the dev channel (`crates/zed/RELEASE_CHANNEL`), and `script/install-marley:75`
builds only the `marley` binary. So Marley opens no SSH project and no dev container unless a
server built from the same commit already sits at `~/.zed_server/zed-remote-server-dev-build`.

### The server side

`remote_server` has four subcommands: `run`, `proxy`, `version` and `exec`
(`crates/remote_server/src/server.rs:72-95`). `proxy` is a thin relay, one per connection. It finds
or starts a daemon for its identifier and copies its own stdin, stdout and stderr to the daemon's
three Unix sockets in `<data dir>/server_state/<identifier>/` (`server.rs:880-909`, `951-1097`).
The identifier is `dev-workspace-<id>` or `dev-setup-<n>` (`remote_client.rs:348-380`), so each
workspace has its own daemon. A first proxy kills any old daemon for its identifier and starts a
new one (`server.rs:1000-1020`); a `--reconnect` proxy that finds no daemon exits with 90, which the
client reads as "server not running" (`server.rs:990-999`, `crates/remote/src/proxy.rs:10`). The
daemon is a headless GPUI app that serves one client at a time and exits after ten minutes without
one (`server.rs:514-649`).

Since the rename (TICKET-437) the data directory on the host is `~/.local/share/marley`
(`paths.rs:19`, `145-168`), but the binary still lives in `~/.zed_server` under the name every Zed
dev build uses.

### The wire

Each connection is one byte stream: a little-endian `u32` length, then a protobuf `Envelope`
(`crates/remote/src/protocol.rs:8`, `25-50`; `crates/proto/proto/zed.proto:21-33`). Requests and
replies are matched by `id` and `responding_to`, streamed replies end with `EndStream` or `Error`,
and everything (613 message types across `crates/proto/proto/*.proto`) shares that one stream.
The daemon's stderr carries its log records as JSON lines, which the client logs as its own
(`transport.rs:196-232`). Each side sends `RemoteStarted` first; the client waits for it for up to
60 seconds in a release build, then sends a `Ping` (`remote_client.rs:164-165`, `460-502`,
`1777-1821`).

Nothing checks that the two ends speak the same protocol. `rpc::PROTOCOL_VERSION`
(`crates/rpc/src/rpc.rs:19`) is collab's and the remote path never reads it. A server from another
commit is used as long as its `version` runs.

### Liveness and reconnect

The client sends a `Ping` every 5 seconds unless traffic arrived, waits 5 seconds for it, and
reconnects after five misses (`remote_client.rs:161-167`, `785-882`). A reconnect kills the
transport, connects again through the pool, runs `proxy --reconnect` to rejoin the same daemon,
and after `FlushBufferedMessages` resends every message the server has not acknowledged
(`remote_client.rs:588-783`, `1953-1973`; acknowledgements prune the buffer at `1788-1793`). It
gives up after three attempts and the window shows the host as disconnected (`remote_client.rs:167`,
`639-647`). A proxy process that exits starts the same path (`remote_client.rs:884-923`).

### What runs on the host

`HeadlessProject` (`crates/remote_server/src/headless_project.rs:110-359`) runs the worktrees and
their file watching, buffers, language servers, git, tasks, toolchains, prettier, debug adapters
and breakpoints, external agents, MCP context servers, extensions and REPL kernels. Four things go
around the protocol instead, each as its own `ssh` over the master:

- **Terminals:** `ssh -t HOST 'cd DIR && exec env … $SHELL -l'`
  (`crates/project/src/terminals.rs:718-752`, `ssh.rs:452-501`, `2028-2159`).
- **ACP agents:** the client asks the host for the agent's command (`GetAgentServerCommand`,
  answered from the host's own settings, `crates/project/src/agent_server_store.rs:613-708`,
  `847-885`), then runs `ssh -T HOST 'cd ROOT && exec ~/.zed_server/<binary> exec'` and writes a
  length-framed JSON `RemoteCommand` before the ACP stream (`crates/agent_servers/src/acp/transport.rs:35-56`,
  `ssh.rs:503-542`, `crates/remote/src/command.rs:7-47`, `server.rs:156-253`). The agent runs on
  the host in the project's folder and its JSON-RPC rides that ssh session.
- **Debugger ports:** `ssh -N -L` (`crates/project/src/debugger/session.rs:3007`, `ssh.rs:544-566`).
- **Extension uploads:** sftp, then scp (`crates/extension_host/src/extension_host.rs:2291`,
  `ssh.rs:568-626`).

Settings reach the host as the raw user settings file and nothing else
(`crates/project/src/project_settings.rs:979-999`). A setting Marley sets in memory with
`update_default_settings` never reaches it.

### The transport seam

`RemoteConnection` (`remote_client.rs:1634-1688`) is small: `start_proxy` receives two `Envelope`
channels and returns a task with an exit code; `build_command` (a PTY command), `build_stdio_command`
(the `exec` path), `build_forward_ports_command`, `upload_directory`, `kill`, and the platform,
shell and path style. `start_proxy` need not spawn a process; the mock transport pumps channels
(`transport/mock.rs:258-296`).

A new transport is still not cheap. `RemoteConnectionOptions` is a closed enum
(`remote_client.rs:1346-1402`), stored in the workspace database
(`crates/workspace/src/persistence/model.rs:36-46`) and matched by variant in 24 files outside the
`remote` crate (recent projects, workspace, title bar, sidebar, project, agent UI, REPL and more).
And every transport must hand back a local command that gives a PTY on the host, a stdio command,
port forwards and file upload. Docker and WSL get those from `docker exec` and `wsl.exe`; a
harness transport would need the harness to provide them.

## 2. How the harness reaches hosts

The harness has no SSH client. Nothing in its crates runs `ssh` or links an SSH library
(`rustal-harness/crates/harness-cli/Cargo.toml:13-19`). Whoever is on the near end runs `ssh HOST
rh --state ROOT <verb>` and `rh` speaks over that session's stdio: ACP
(`rustal-harness/docs/ACP.md:15-26`), MCP (`rustal-harness/docs/MCP.md:34-37`) and the seat verbs
(`rustal-harness/docs/AGENT_SEATS.md:181-187`). Marley does exactly this: it follows a host's
harness with `ssh -T -o BatchMode=yes <keepalive> -- HOST rh --state ROOT mcp`
(`crates/marley_workbench/src/harness_hosts.rs:75-129`), attaches with `ssh -t`
(`harness_hosts.rs:135-157`), and registers a seat thread as an `agent_servers` entry `ssh … rh
--state ROOT acp --seat NAME` (`crates/marley_workbench/src/harness_seat.rs:436-481`).

On the host, `rh serve` runs the runtime with a private tmux server or the native backend. Its
control socket `ROOT/runtime.sock` is mode 0600, refuses peers of another user and takes one
newline-delimited JSON request per connection (`rustal-harness/crates/harness-runtime/src/terminal.rs:1804-1872`,
`rustal-harness/crates/harness-runtime/src/protocol.rs:8-29`). The socket stays local by decision
(D68, `rustal-harness/docs/DECISIONS.md:1845-1852`; `rustal-harness/crates/harness-runtime/src/compatibility.rs:14-20`).
The remote native view is `rh remote serve` as the forced command of a dedicated sshd with
forwarding disabled and one session per connection, reached by a client that turns multiplexing off
(`ControlMaster=no`, `ControlPath=none`; `rustal-harness/docs/REMOTE.md:77-129`).

The decisions point the same way. D164: Marley reaches each other box over its own SSH connection
and the harness does not join hosts into one fleet (`rustal-harness/docs/DECISIONS.md:364-375`).
D165: agents on different machines meet only through the Brain (`DECISIONS.md:347-356`). D181: the
harness builds no relay (`DECISIONS.md:154-157`). D178: on another box Zed runs `ssh HOST rh acp`,
so no port opens (`DECISIONS.md:186-208`).

The harness installs nothing on a host: "Installation remains an explicit administrator operation"
(`rustal-harness/docs/REMOTE.md:21-22`). Its agent binaries are pinned to the path recorded on the
machine that built `rh` (`rustal-harness/crates/harness-runtime/src/claude/mod.rs:84-91`,
`codex/mod.rs:91-98`), so another host needs that exact file or `rh seat add --binary`
(`rustal-harness/crates/harness-cli/src/seats.rs:52-54`); Marley's seat commands pass no
`--binary` (`harness_seat.rs:615-640`). Nothing compares a local `rh` version with a remote one.

Three more facts decide the options:

- A seat's folder comes only from `rh seat add --cwd`, which must be an absolute folder that exists
  on that host (`rustal-harness/crates/harness-cli/src/seats.rs:43-45`, `152-160`). ACP
  `session/new` ignores the client's `cwd` (`rustal-harness/crates/harness-runtime/src/acp/seat.rs:160-163`)
  and `rh acp --seat` never starts a seat (`rustal-harness/docs/ACP.md:159`).
- `rh acp` takes text prompts only and never asks the editor for a file or a terminal
  (`rustal-harness/crates/harness-runtime/src/acp.rs:168-178`). The seat reads its files itself,
  on its own host.
- The shared Claude Code plugin reports to the harness when `RH_BIN` and `RH_STATE` are set,
  otherwise to Marley when `MARLEY_BIN` and `MARLEY_TERMINAL_ID` are
  (`rustal-harness/crates/harness-runtime/src/claude/plugin/register.js:10-19`, `135-147`). Marley
  leaves `MARLEY_BIN` empty in a remote project's terminals
  (`rustal-harness/docs/planning/MARLEY_REQUESTS.md:194-197`). A Claude Code started by hand in a
  Zed remote terminal therefore reports to nobody, while a seat on that host reports to the
  harness, which Marley follows.

## 3. The options

| Option | What it buys | Cost | Code lives in | Verdict |
|---|---|---|---|---|
| a. The harness puts the matching `zed-remote-server` on its hosts | SSH projects on harness hosts only | An SSH client and an uploader in a harness that has neither by design; Marley still has to build and hand over the binary | harness and Marley | No; a′ instead |
| a′. Marley stages its own server and Zed uploads it | SSH projects and dev containers on every host | One hunk in `remote`, an install flag, a second build target | Marley | First |
| b. A harness transport for Zed's client | One link per host shared with seats | A new enum variant in 24 files; the harness would need PTY exec, port forwards, uploads and a relay, all declined | both | No |
| c. Seats in a remote project's folder | Agents on the host, edits seen live, reports through the harness | Host matching, the New Agent default, the remote-window thread fix | Marley | Second |
| d. Live alongside with one host list | One place to pick a host | Matching three settings lists | Marley | Third |
| e1. `rh acp` through the remote server's `exec` | ACP rides Zed's own link | A user-settings entry per seat on the host; needs a′ | settings only | Not now |
| e2. The harness speaks Zed's protocol | Nothing the seat needs | 613 message types that change with every Marley commit, no version check | harness | No |
| e3. One SSH master per host | Fewer handshakes | An `ssh_config` stanza; a caveat | ops | Optional |

### a. The harness installs Zed's server: no

It fixes only harness hosts, and the harness has no near end to copy from: it never opens an SSH
connection and installs nothing (section 2). The copy would run from Marley's box, which is where
Zed's own uploader already runs, over the connection Zed has opened (`ssh.rs:1247-1287`). The
binary has to come from the same commit as Marley, so whoever builds Marley owns the match.

### a′. Marley's own remote server: build first

Three parts:

1. **Name the server by commit on the dev channel.** `zed-remote-server-dev-<sha>` instead of
   `-dev-build`, from the `ZED_COMMIT_SHA` both binaries already bake in
   (`crates/zed/build.rs:49-81`, `crates/remote_server/build.rs:20-32`). A stale server never
   matches, Marley stops sharing a file name with any other Zed dev build, and the host cleans up
   by itself: `cleanup_old_binaries` removes every dev binary whose suffix is no version and which
   is not running (`server.rs:1428-1463`).
2. **Stage the server at install.** `script/install-marley` builds `remote_server` the way
   `script/bundle-linux:98-137` does: a separate cargo invocation, for
   `x86_64-unknown-linux-musl`, so feature unification with the editor cannot change its libraries
   and older glibc on Ubuntu LTS hosts does not matter. It writes the gzipped binary to
   `~/.local/share/marley/remote_servers/dev/linux-x86_64/<sha>.gz`, the layout Zed's updater
   already uses for downloaded servers (`crates/auto_update/src/auto_update.rs:626-629`). The musl
   build needs the rustup target and `musl-gcc` (`bundle-linux:133-136`).
3. **Upload it from the dev branch.** In `ensure_server_binary`, where the dev channel now bails
   (`ssh.rs:1077-1086`), use the staged file through the existing `upload_local_server_binary`
   and `extract_server_binary`, and bail with a message naming the install flag when there is
   none. The same hunk goes in `docker.rs:278-287`. About 30 lines with `// Marley:` comments and
   a row in `docs/marley/zed-touchpoints.md`.

Cost: one more compile of the server's dependency graph for a second target on each install that
asks for it, on a box where only one cargo runs at a time. A macOS host is not covered: Zed
cross-builds for another OS with `cargo-zigbuild` (`transport.rs:381-421`), and `apple-darwin` needs
the macOS SDK, which this box lacks. A macOS host needs a build on a Mac from the same commit.

An install from a tree with uncommitted changes gives the editor and the server the same commit
name over possibly different protocols; `install-marley` already prints that the tree is dirty
(`script/install-marley:135-140`), and the staged file should carry the same warning in its log
line.

### b. A harness transport: no

What it would buy, one link per host shared by Zed and the seats, OpenSSH already gives without
code (e3). What it would cost is a new `RemoteConnectionOptions` variant across 24 files and a
harness that runs commands on a host for a remote client, forwards ports and receives files, which
is what D68, D164 and D181 decline and `docs/REMOTE.md` locks out for the native view. Nothing
found in either code base needs a channel only the harness could provide.

### c. Seats in a remote project: build second

A seat can already work in any existing absolute folder on its host (`seats.rs:43-45`, `152-160`),
and New Agent's **On `<harness>`…** already starts one in a typed folder (`harness_seat.rs:527-611`).
What is missing is Marley knowing that a remote window and a harness are on the same host:

- **Match the host.** A remote window's `RemoteConnectionOptions::Ssh` gives `username`, `host` and
  `port`; a `marley.harnesses` entry gives an `SshTarget` with `user`, `host` and `port`
  (`harness_hosts.rs:20-30`, `crates/marley_remote/src/marley_remote.rs:32-42`). Equal triples
  are the same host.
- **New Agent's default.** Today `places_for` offers a remote window's roots as local folders
  (`crates/marley_workbench/src/agents.rs:747-784`), a path that exists only on the host. With a
  match, the first place becomes **On `<harness>` in `<root>`**; without one, New Agent says the
  host has no harness.
- **The thread view from a remote window.** By reading the code (not run), a seat thread opened
  from a remote window cannot start. Thread tabs build on the hosting workspace's project
  (`crates/marley_workbench/src/thread_tab.rs:12-14`, `621-645`), a remote project lists only the
  agents the host names (`agent_server_store.rs:710-786`), and Marley's seat entry lives in the
  client's in-memory defaults (`harness_seat.rs:461-475`), which the host never receives
  (`project_settings.rs:979-999`). The seat reads no file through Zed (`thread_tab.rs:617-619`), so
  the simplest fix hosts such a thread on a local project, for instance in the Default group, and
  links it from the remote project's rail row. The Test phase should confirm the failure first.

What it buys: the seat and the remote project share the folder on the same host. Zed's remote
worktree watches that folder (`headless_project.rs:127`), so the seat's edits appear in open
buffers and the git panel as it makes them; the seat reports to the harness, which Marley already
follows on that host; and the agent survives Marley closing, which a CLI typed into a remote
terminal does not. Nothing new is needed in the harness, but the host must run `rh serve`
(`rustal-harness/docs/AGENT_SEATS.md:220-222`) and hold the agent at its pinned path.

### d. Live alongside: build third

The two already coexist on a host without touching each other: Zed keeps its files in
`~/.zed_server` and `~/.local/share/marley`, the harness in its root, and neither reads the other's.
The friction is in Marley's settings, which name hosts three times:

| Setting | Used by |
|---|---|
| `ssh_connections` (Zed's) | Remote projects, and Marley's remote terminals #543 (`crates/recent_projects/src/remote_connections.rs:31-40`, `crates/marley_workbench/src/remote.rs:147-148`) |
| `marley.fleet.hosts` | The host collector #610 (`crates/settings_content/src/marley.rs:260-280`) |
| `marley.harnesses` | Harness hosts #740 (`crates/settings_content/src/marley.rs:100`, `316-327`) |

The smallest shared piece leaves all three settings where they are and matches their entries by
destination, then shows one host picker that offers, per host, what that host can do: open a remote
project, open a remote terminal, start a new agent on its harness. No setting moves.

### e. Smaller ideas

- **e1. `rh acp` through Zed's own link.** In a remote project an `agent_servers` entry
  `{"command": "rh", "args": ["--state", ROOT, "acp", "--seat", NAME]}` is resolved on the host and
  run through `zed-remote-server exec` in the project's folder (`agent_server_store.rs:613-708`,
  `acp/transport.rs:35-56`), with no ssh of its own. It works without code once a′ lands, but only
  from the user's settings file, which is the one thing the host receives. Writing seat entries
  there is worse than c's local thread. Not now.
- **e2. The harness reading Zed's protocol.** No. The protocol is 613 message types that move with
  every Marley commit and have no version check, and the seat reads its own files.
- **e3. One SSH master per host.** Zed reuses a master that is alive when it connects and never
  kills one it did not start (`ssh.rs:720-822`, `425-433`). An `ssh_config` stanza for the harness
  hosts with `ControlMaster auto`, a `ControlPath` and `ControlPersist` makes Marley's harness
  follower, started with Marley, open a master that the remote project, the seat commands and the
  attach terminals then share. Two cautions. Zed reuses only a master already alive; otherwise it
  makes a private one (`ssh.rs:840`). And a `ControlPath` must not go in a connection's `args`:
  ssh keeps the first value of an option, so Zed's own master would sit on the shared path
  (`ssh.rs:326-330`), and Zed kills its master on every reconnect (`remote_client.rs:671-677`),
  cutting everything riding it. The native view's dedicated sshd stays out of this. An ops note
  for the handbook, not a ticket.

## 4. Recommendation

| # | Repo | Ticket | Needs |
|---|---|---|---|
| M1 | Marley | Marley's own remote server (a′): stage `remote_server` for Linux x86_64 at install, name the dev server by commit, upload the staged one; same for dev containers. Shipped as #445 slice 1, 2026-10-11 | none |
| M2 | Marley | Seats in a remote project (c): match a remote window's host to `marley.harnesses`, default New Agent there to the harness in the project's root, make a seat thread from a remote window work or say why | M1 to see the edits in a remote project |
| M3 | Marley | One host picker (d) over the three host lists | M2 |

Not to build: a harness transport (b), the harness installing Zed's server (a), the harness
speaking Zed's protocol (e2), and seat entries in the host's settings (e1). Each either repeats
what Zed's link already does or reopens D164 and D181, and nothing in the code needs them.

Harness tickets: none needed. Two candidates its session may weigh on its own, neither blocking:
an `rh compatibility` answer Marley can compare across hosts, since nothing compares versions
today; and `rh acp --seat` for Codex seats, refused today (`rustal-harness/docs/ACP.md:158`).
Contract changes would go through `rustal-harness/docs/planning/MARLEY_REQUESTS.md` as usual.

## Traps found on the way

- **A Marley debug build compiles the server with cargo when it connects.** Without
  `ZED_BUILD_REMOTE_SERVER=never` or a `ZED_COPY_REMOTE_SERVER` file, a debug build runs `cargo
  build -p remote_server` from the checkout at connect time (`transport.rs:261-296`, `460-479`).
  On this box that is a second cargo. Set one of the two before opening a remote project from a
  debug build.
- **Turning on `remote/build-remote-server-binary` in the release build is no fix.** It uploads
  the server on every connect and falls back to the same cargo build when the file is missing
  (`ssh.rs:1049-1071`, `transport.rs:271-296`).
- **No protocol check between the ends** (section 1). Until M1 lands, a server left on a host by an
  older Marley is used without a word.
- **Seats on another host need the agent at the build machine's path** or `--binary` (section 2).

## Open questions for Chad

1. Should `install-marley` build the remote server on every install, or only when asked
   (`--remote-server`)? Default: only when asked, until remote projects are daily use; the error on
   connect names the flag.
2. Remote projects on a macOS host need a server built on a Mac from the same commit. Build one
   there, or leave that host without remote projects? Default: without, for now.
3. A seat thread started from a remote window: open it in the Default group and link it from the
   remote project's row? Default: yes.
