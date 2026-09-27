---
pipeline_id: 415ababd-46cf-4860-bec8-0b1115ceeb4a
ticket: docs/planning/tickets/open/TICKET-584-outside-clients-from-other-machines.md
status: Phase 4 — Complete PASS
title: "Outside clients from other machines, through Marley's bridge over SSH"
type: feature
slice: prong 3, B8c (slice 3 of #524)
references: [docs/planning/pipeline/completed/524-trusted-outside-browser-access.notes.md, docs/planning/pipeline/completed/583-chromium-devtools-off-tcp.notes.md, docs/planning/pipeline/completed/561-browser-env-opener.notes.md]
---

## Title
#524 lets programs on this machine drive Marley's browser by name: each allowed client has an
endpoint file with a token that is new at every start, and Marley's stdio bridge reads that file.
A client on another machine had no way in. It gets one without a new port: its MCP client
starts Marley's bridge on this machine over SSH, pointed at its own endpoint file. SSH checks who
connects, and the bridge reads the token Marley wrote at its latest start, so nothing on the other
machine changes when Marley restarts, and Marley listens on nothing new. Browser Clients shows the
line to run. The bridge also tells a client that Marley cut off that it is not allowed. Today it
says Marley is not running.

## Scope
### In
- `crates/marley_workbench/src/clients.rs`: the allowed client's panel shows a third copyable
  line under the local command, "From another machine, run it over SSH:", which is `ssh -T -o
  BatchMode=yes <user>@<host> '<the command, with env>'` with this machine's user and host
  names. Cut Off rewrites the registry before it removes the client's endpoint file.
- `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge`: when its endpoint file is
  a client's (`<data>/mcp/clients/<name>.json`) and is gone, and the registry beside it no longer
  names the client, the bridge answers that Marley does not allow this client (cut off, or never
  allowed), in tool errors and in its log, not that Marley is not running. The docstring names
  outside clients and SSH.
- `crates/marley_workbench/Cargo.toml`: `whoami` (already in `Cargo.lock`) for the user and host
  names.
- `script/e2e/584-outside-clients-from-other-machines.sh`: a scenario with an sshd of its own on
  127.0.0.1, and a stand-in MCP client that runs the copied line through it.

### Out (explicitly deferred)
- A fixed loopback port behind `ssh -L`, or `tailscale serve` in front of the MCP server, the
  ticket's first idea. A remote HTTP client keeps its token in its own config. The token is
  minted at each start (AD-claude-524-outside-clients-reach-a-list-of-browser-tools-by-name-001),
  so Marley would refuse that config after its next restart. Serving such clients needs a token
  that lasts across starts, which AD-524 rejected, or MCP's OAuth authorization. That is a
  decision for the phone path (below), not this slice.
- The phone: the Orca survey's item 12 (a loopback endpoint behind `tailscale serve`, QR
  pairing, device tokens, a web app; report 04 §3.2 item 3). The survey's question 8 says nothing
  is built until Chad picks, and he has not. #535 carries the push half.
- CDP from other machines. Outside clients use Marley's tools by name, never raw CDP (AD-524, plan
  D15); the relay of #583 stays loopback with its per-start token.
- Showing the lines again for a client allowed earlier. As in #524, the panel shows them right
  after Allow; the guide gives the paths.
- A Host check on the MCP server. Nothing new reaches it; its Origin check and bearer stay as
  they are.

## Reference (§20)
N/A — Marley-specific. No Warp or Zed behavior covers outside clients of Marley's browser tools.
Warp is an MCP client of remote servers (`docs/warp_architecture/crates/ai.md`, "auth for remote
MCP servers"), never a server for remote clients. Zed's `remote` crate runs SSH toward a remote
project and forwards TCP ports that way; it never serves an MCP client that arrives over SSH. The
protocol references are MCP's stdio transport (the client launches the server as a subprocess)
and OpenSSH's remote command.

### Prior art
1. **Behavior maps.** `docs/orca_architecture/04-remote-control-and-mobile.md` §3.2 item 3 (a
   phone endpoint bound to 127.0.0.1 and published by `tailscale serve --https`, with a 0600
   registry, rotate and revoke), §2.13 (Orca's direct tokens last across restarts, which AD-524
   rejected), and the survey README's item 12 and question 8 (the phone: nothing built until Chad
   picks). The Warp maps have nothing on this seam beyond Warp as an MCP client.
2. **Published material.** MCP 2025-11-25, Transports: over stdio "the client launches the MCP
   server as a subprocess", the server "MUST NOT write anything to its stdout that is not a valid
   MCP message", and "Clients SHOULD support stdio whenever possible". MCP Authorization: optional,
   and "Implementations using an STDIO transport SHOULD NOT follow this specification, and instead
   retrieve credentials from the environment". That is what the bridge does with the client's
   endpoint file. MCP's Streamable HTTP security list (Origin 403, bind 127.0.0.1, authenticate)
   is already met by `marley_mcp`. OpenSSH runs a remote command through the account's login shell;
   `-T` asks for no terminal; `BatchMode=yes` fails instead of prompting. Tailscale Serve
   (`ipn/ipnlocal/serve.go`): a TCP backend gets the tailnet name as `Host`, plus
   `X-Forwarded-Host/Proto/For` and `Tailscale-User-*` identity headers. A local process could
   send the same headers straight to 127.0.0.1, so they prove nothing to the backend. Since
   TS-2026-005, only root may set a `unix:` target.
3. **The code we ship.** It already owns the seam. `marley-mcp-bridge` re-reads its endpoint file
   before each call, reconnects with the client's own `initialize` after a restart, answers on its
   own while Marley is down, sends its bearer only to a loopback URL, and writes only JSON-RPC to
   stdout. Run over SSH, it is the remote client's adapter with no change. So the ticket's fixed
   port is dropped: the substrate already does the job.
   `marley_mcp::transport` binds `127.0.0.1:0`, checks Origin and the bearer, and has no Host
   check. Zed's `remote` crate (`SshPortForwardOption`, `build_forward_ports_command`) forwards
   TCP ports only, from Zed's machine outward; it is not needed. `Cargo.lock` holds `whoami` 1.6.1
   and `gethostname` 1.1.0, and no tailscale crate. `util::shell::ShellKind::Posix.try_quote`
   (shlex) quotes the line.

## UI proof
UI-affecting: `script/e2e/584-outside-clients-from-other-machines.sh` under `compositor sway`
(clicks: the modal's checkbox, the SSH line's Copy button, the client's Cut Off in the modal). Its
`setup` starts an sshd of its own on 127.0.0.1 with its own host key and authorized key, never
touching `~/.ssh`. A stand-in MCP client runs the line the modal copied, with the scenario's own
SSH options and `127.0.0.1` in place of the host. Shots: `584-01-allowed` (the panel's three
lines), `584-02-driven` (the tab names the SSH client), `584-03-cut-off` (the modal after Cut
Off), `584-04-mark-gone` (the tab without its mark).

## Locked-In Decisions
- D1 — A client on another machine runs Marley's stdio bridge on this machine over SSH, pointed at
  its own endpoint file. Nothing new listens: no fixed port, no setting, no second listener. SSH
  authenticates the user, the house path to this machine; the bridge takes the credential from
  the machine it runs on, as MCP's authorization spec asks of stdio. The client's grant, its tab
  mark and Cut Off are #524's, unchanged.
- D2 — The allowed client's panel shows `ssh -T -o BatchMode=yes <user>@<host> '<command>'` under
  the local command, with a Copy button. The command inside is `env MARLEY_MCP_ENDPOINT=<file>
  <bridge>`, each word quoted by `ShellKind::Posix`, and the whole quoted once more for the shell
  that runs `ssh`. `env` works in every login shell; csh has no `NAME=value` prefix. `-T` gives
  the session no terminal. `BatchMode` makes an SSH that would ask for a password fail at once,
  instead of prompting inside the MCP client's own terminal. The user and host are this
  machine's (`whoami`). A user whose other machine knows this one by another name edits the
  host. The line holds no token.
- D3 — A bridge whose client file is gone, while the registry beside it no longer names the
  client, answers that Marley does not allow the client and that the user can allow it again in
  Browser Clients. Only a client listed in the registry whose file is gone is told that Marley is
  not running. Cut Off rewrites the registry before it removes the file, so a bridge never sees a
  running Marley's cut-off client as not running. The bridge reads the registry (names and grants,
  no token), never Marley's own endpoint file.
- D4 — HTTP clients on other machines and the phone wait (see Out); the reason is AD-524's
  per-start tokens, not transport.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method (a named e2e shot, the gate's
exit code, a negative smoke, or review).

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user allows a client in Browser Clients, the system shall show under the local command the line `ssh -T -o BatchMode=yes <user>@<host> '<command>'`, with this machine's user and host names and the client's endpoint file and bridge in the command, and a Copy button that puts exactly that line on the clipboard, with no token in it. | Shot `584-01-allowed`; the log: the clipboard parsed (the destination, the file, the bridge), and the token absent |
| REQ-002 | WHEN a client runs the copied line from another machine (the scenario's SSH session in place of the host), the system shall list the tools of its grant (eighteen for a client allowed to act), answer its calls, and name it in the Browser tab it acts in. | The log: the tool count and a call's result over SSH; shot `584-02-driven` |
| REQ-003 | WHILE a client talks to the bridge over SSH, the system shall write nothing but JSON-RPC messages to the session's standard output. | The log: the stand-in parses every line it read as JSON-RPC |
| REQ-004 | WHEN Marley quits and starts again while a client's SSH session stays open, the system shall answer the session's calls while Marley is down with "Marley is not running", and after the start with Marley's answers, with no change on the client's side. | The log: the held session's calls before, during and after the restart, and the tools-changed notices |
| REQ-005 | WHEN the user cuts a client off, the system shall answer the client's next call over SSH, and a local bridge pointed at the client's own endpoint file, that Marley does not allow the client, and never that Marley is not running; and the same after Marley starts again. | The log after Cut Off and after the restart; shots `584-03-cut-off`, `584-04-mark-gone` |
| REQ-006 | WHILE clients on other machines are connected, the system shall listen on no new socket: Marley's process listens only on its MCP server's port on 127.0.0.1. | The log: `ss -ltnp` for Marley's process |
| REQ-007 | WHEN Cut Off runs, the system shall rewrite the client registry before it removes the client's endpoint file. | Review of `clients::cut_off` |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the bridge (the not-allowed answer and log, the docstring), `clients.rs` (the SSH
  line, Cut Off's order), `Cargo.toml` (`whoami`); fmt and clippy clean; a review of the diff.
- **P3 Test** — write and run `script/e2e/584-outside-clients-from-other-machines.sh` and read
  every shot; 524 again (the bridge and the modal); the golden set with 584 added;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG and architecture docs (§21): the guide's Browser Clients section
  (from another machine), `marley_workbench.md`, the plan's row; ledger capture (§19); close the
  ticket, archive, commit and push.
