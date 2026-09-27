---
pipeline_id: 25679944-2eae-41b5-b6ae-705b71833a0a
ticket: docs/planning/tickets/open/TICKET-524-trusted-outside-browser-access.md
status: Phase 4 — Complete PASS
title: "Trusted outside clients drive Marley's browser"
type: feature
slice: prong 3 with prong 2's MCP server (plan D15, D17); slice 1 of 3
references: [docs/orca_architecture/04-remote-control-and-mobile.md, docs/planning/pipeline/completed/491-marley-mcp-in-the-app.spec.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md, docs/planning/pipeline/completed/520-terminal-identity.spec.md, docs/planning/pipeline/completed/574-browser-tools-in-the-callers-project.spec.md]
---

## Title
Outside programs the user trusts reach Marley's browser through its MCP server: each client is
allowed by name, with a grant to read pages or also to act in them, and holds a token of its own,
new at each start, in an endpoint file of its own. A client sees and calls only the browser tools
on its grant's list; the tab it drives shows its name; the user cuts it off at once. The server
stays on loopback, and what it reads before it knows who is asking is bounded in size and time.
Chromium's own DevTools port, open to every local process, is the second slice's.

## Scope
### In
- `crates/marley_mcp/src/clients.rs` (new, pure): `Principal`, who holds the bearer of a request
  (`Marley` for the per-boot bearer, `Client(ClientGrant { name, write })`), beside #520's
  `Caller`, which says where a call comes from and grants nothing; `ClientTable` (each client's
  grant, its token and its last call; `resolve(bearer)` compares with `ct_eq` against Marley's
  bearer and every client's token with no early exit; `allow`, which mints the token; `cut_off`);
  the two lists of tools a client may call, `CLIENT_READ_TOOLS` and `CLIENT_WRITE_TOOLS`, and
  `permits(principal, tool)`.
- `crates/marley_mcp/src/transport.rs`: the request line and each header line are read through a
  cap (8 KiB, at most 100 header lines, 431 past either), and a connection has 10 seconds to send
  its request; the principal is resolved after the size cap and `Origin` (403 when none); a
  client's `Marley-*` headers are dropped; a client's GET gets 405; `ask_app` carries the
  principal; the server makes the client table and hands it out through `ServerHandle::clients`.
- `crates/marley_mcp/src/session.rs`: each session records its owner; a session id presented by
  another principal is unknown (404); a client holds at most four sessions, and a fifth
  `initialize` ends that client's own oldest; a cut-off ends every session the client owns.
- `crates/marley_mcp/src/dispatch.rs`, `marley_mcp.rs`: `RequestCtx` and `AppCall` carry the
  principal; for a client, `tools/list` names only its grant's tools, `tools/call` refuses any other
  tool by name whether listed or not, and `resources/*` is refused.
- `crates/marley_mcp/src/discovery.rs`: the endpoint file's name is a parameter, for the clients'
  files beside Marley's own.
- `crates/marley_browser/src/recorder.rs`: the minute's `Agent` entry names the client that acted.
- `crates/marley_workbench/src/clients.rs` (new): the registry `<data>/mcp/clients.json` (0600:
  each client's name, grant and when it was allowed, no token); at each start a token per client,
  written with the URL into `<data>/mcp/clients/<name>.json` (0600, in a 0700 folder), all removed
  at quit and one at its cut-off; the Browser Clients modal (`marley: browser clients`): the
  clients with their grant, their last call and Cut Off, and a form (a name, "May act in pages",
  Allow), after which it shows the client's endpoint file with Copy and the line that points
  Marley's bridge at it.
- `crates/marley_workbench/src/mcp.rs`: the server starts with the registry's clients, whose
  state (the server's handles and the main thread's copy of the registry) `clients.rs` keeps.
- `crates/marley_workbench/src/browser_tools.rs` and `browser.rs`: a client's action names the
  client in the tab's chip, where Marley's own agents show "Agent"; while the client is allowed
  and acted on the page in the last minute, the toolbar keeps "Driven by <name>" with Cut Off; a
  client's call that reaches the app after its cut-off is refused before it acts.
- `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge`: a 403 is told as a refusal
  of the endpoint file's token, not as "Marley is not running".
- `script/e2e/browser-fixture.sh`: `mcp_agent --endpoint <file>`, and `mcp_http` for requests
  whose status matters. `script/e2e/524-trusted-outside-browser-access.sh`.

### Out (explicitly deferred)
- **Slice 2, the side door.** Chromium's DevTools ports (one per project since #507) stay open on
  127.0.0.1, with no credential, to every local process until then. Slice 2 starts Chromium with
  `--remote-debugging-pipe` under a small Marley relay in the same unit, which serves Marley over
  a Unix socket in `$XDG_RUNTIME_DIR` (mode 0600) and serves Playwright clients (#523's runner,
  the Playwright MCP, the e2e fixture's stand-in agent) a CDP WebSocket on loopback that takes a
  client's token. It amends plan D16 ("any other CDP client find[s] it" in `DevToolsActivePort`)
  and needs its own spec.
- **Slice 3, other machines.** A fixed loopback port behind `ssh -L` (rustal-harness M9's rule:
  "Remote clients arrive through M8's authenticated SSH path") or `tailscale serve` (report 04
  §3.2 item 3). Loopback only until then.
- Terminal, fleet and session tools for clients; `browser_draft_test` (it reads the project's
  Playwright config and names the project's root) and `browser_open_url` (the terminal opener's
  tool); per-project or per-tab grants (a client reaches every project's tabs, as Marley's own
  agents do, #507 D8).
- Annotations per client: `Maker::Agent` has no name, so a client's clear removes every agent's
  boxes, as one agent's does today.
- A section on #515's Marley settings page: the registry is Marley's file, not Zed settings (D3).
- Pairing by QR, push, app-layer encryption (report 04 §2.2 and §2.3): not needed on loopback.

## Reference (§20)
Orca's paired devices (report 04 §2.2): a token per device, a registry written 0600, devices
listed and revocable, and a revoke that closes the device's open sockets; a scope that decides
which methods a token may call (`src/main/runtime/device-registry.ts`,
`src/main/runtime/runtime-rpc/runtime-rpc-mobile-method-allowlist.ts`). Marley keeps the named
clients, the per-client tokens, the scope (as an allowlist of tools under two grants) and the
revoke that ends sessions. It leaves out what report 04 §2.13 faults: direct tokens that never
expire, and a scope that amounts to a shell on the desktop. The house pattern on this box is
rustal-harness M9's ("An MCP server in the harness on loopback with separate read and write
grants. Remote clients arrive through M8's authenticated SSH path", `/srv/stacks/rustal-harness/docs/ROADMAP.md`,
M9; its R9-05: "The server shall refuse unauthenticated callers and write verbs without a write
grant, and shall listen only on loopback").
Upstream Zed: no per-client access anywhere; its context servers are MCP clients, and the MCP
server it carries (`context_server::listener::McpServer`) serves one Unix socket with no notion
of who connects. Warp: N/A.

### Prior art
- **Behavior maps and reports.** Report 04 §2.2 (Orca's `DeviceRegistry`: per device a UUID, a
  name, 24 random bytes of token, the scope, when it was paired and last seen; the loaders refuse
  to overwrite a file they could not read; direct tokens never expire), §2.3 (a `mobile` token may
  call 294 of about 620 methods, `terminal.send` and `git.push` among them), §2.13 (the path by
  path table; Orca's docs send users to a private network or SSH forwarding), §3.2 item 3
  (Marley's phone endpoint: loopback behind `tailscale serve`, scoped with `marley_mcp`'s
  `GrantTable`). Report 03 §2.11 (Orca's agents drive the browser through its CLI, which reads an
  auth token from a runtime metadata file; no MCP).
- **Published material.** The MCP transport's security warning, which `marley_mcp`'s guards
  already follow (bind loopback, validate `Origin`, authenticate; `auth.rs`, line 1). The
  Streamable HTTP transport lets a server answer a GET with 405 when it offers no stream there.
  Chromium's DevTools endpoint takes no credential (plan D15: "takes no token (Chromium offers
  none)"). `--remote-debugging-pipe`, for slice 2: Playwright launches its own Chromium with it
  (`playwright-core` 1.63.0, `lib/coreBundle.js`), and Puppeteer's `pipe` launch option is
  documented as "Connect to a browser over a pipe instead of a WebSocket".
- **The code we already ship** (re-read at 577c3b044c). `marley_mcp`: `ct_eq` and `bearer_ok`
  (`auth.rs:53`, `72`); the bind to `127.0.0.1:0` (`transport.rs:118`), the per-boot bearer from
  `mint_secret(read_entropy())` (120; `read_entropy` is private to the transport), the body cap
  (`MAX_BODY_BYTES`, 423), the `Origin` and bearer checks before any dispatch (177), the session
  gate (194), `handle_message` under the lock (239) and `ask_app` (281), which takes #520's
  `Caller` built from the `Marley-Terminal`, `Marley-Project` and `Marley-Cwd` headers (456);
  `SessionEntry` records no owner (`session.rs:35`), under a cap of 32 (16) with a 30-minute idle
  life (26), and `SessionRegistry::validate` (77) already compares ids with no early exit;
  `GrantTable` and `decide` (`permission.rs:14`, `54`); `tools/list` (`dispatch.rs:28`) takes no
  context, `resources/*` (29 to 31) and the GET stream never reach `decide`, and `tools/call`
  (133) finds unlisted tools by name, such as `fleet_snapshot`, which since #547 carries each
  Claude Code session's prompt and folder; the browser family (`registry.rs`) has 20 tools, 11
  read and 9 write. The request line and headers are read with `read_line` and no cap (430 to
  462), and the server sets no read timeout. The bridge (`marley-mcp-bridge`) follows
  `MARLEY_MCP_ENDPOINT`, sends a bearer only to a loopback URL, opens no GET stream, and reports
  a 403 as "Marley is not running". The tab's chip is set by `acting`, `check_pick` and
  `annotate` (`browser_tools.rs:1007`, 771, 1430) through `agent_started` and `agent_ended`
  (`browser.rs:2371`, 2387) and drawn as `Chip::new("Agent")` in `render_toolbar` (5366). No
  Marley crate has a form modal or a `Checkbox` yet; Zed's `ui::Checkbox` and the Scripts tray's
  one-line `Editor` (#523) are the pieces. Zed's `context_server` carries an MCP server that
  binds a Unix socket in a temporary folder (`crates/context_server/src/listener.rs:55`), which
  nothing in the tree starts; a socket file admits only who can open its path, the property
  slice 2 wants for Marley's own CDP link. Does a crate we build own the seam? Yes: `marley_mcp`
  owns authentication, sessions and grants, so the ticket extends it and adds no second server.

## UI proof
UI-AFFECTING (a modal, the tab's chip and mark). `script/e2e/524-trusted-outside-browser-access.sh`
(`compositor sway`: it clicks in the modal and on Cut Off). Setup: the offline Chromium; a
loopback page with a Name field and a Save button; the scratch repository. Steps and shots:
`marley: browser clients`, then `reader` allowed to read only and `driver` allowed to act, the
modal listing both, with the last one's endpoint file (`524-01-allowed`); the browser on the page;
the stand-in agent, through `driver`'s endpoint file, types into the Name field, and the tab shows
"driver" in its chip and "Driven by driver" with Cut Off (`524-02-driven`); a click on Cut Off,
the mark gone (`524-03-cut-off`). The run log carries: both files' modes and keys (never their
tokens); `reader`'s and `driver`'s tool lists; `reader`'s refused click and its refused
`terminal_read`, `fleet_snapshot`, `resources/read` and GET; `driver`'s request with its old token
refused with 403 after the cut-off, and the bridge's words for it; Marley's own endpoint still
listing every tool; `ss -ltnp` for the server's port; after a relaunch, `reader`'s rewritten file
working and its old token refused; a fifth `reader` session ending its first; an over-long header
line answered 431 and a silent connection closed after 10 seconds, with Marley still answering.

## Locked-In Decisions
- D1: The front door first: outside clients reach the browser through Marley's MCP browser tools,
  never raw CDP, in this slice. The tools act where the user watches (the tab comes forward, the
  chip says what happened), run no script (AD-claude-492), navigate only to `http` and `https`,
  and hide secrets in what they read (plan D15). Raw CDP would hand a client the cookies, script
  evaluation and every page, with nothing on the screen.
- D2: A client has a name and one grant, and calls only the tools its grant lists. Read:
  `browser_tabs`, `look`, `snapshot`, `console`, `network`, `picks`, `pick`, `annotations`,
  `recordings`, `recording`. Read and act: those, and `navigate`, `back`, `click`, `type`,
  `press`, `scroll`, `annotate`, `check_pick`. Never: `browser_draft_test`, `browser_open_url`,
  the terminal, fleet and session tools, `resources/*` and the GET stream. The lists are explicit,
  so a tool added later reaches no client until it is listed (Orca's mobile allowlist).
- D3: The registry is Marley's own file, `<data>/mcp/clients.json` (0600), never Zed settings: a
  project's `.zed/settings.json` merges into the user's settings, so a repository could grant
  itself a client. It holds names, grants and dates, and no token. A registry Marley cannot read
  is left as it is and logged, and Allow refuses to write over it (Orca's loaders refuse).
- D4: A client's token is minted at each start, as the bearer is, and written only into that
  client's endpoint file, `<data>/mcp/clients/<name>.json` (0600, the folder 0700), removed at
  quit and at a cut-off. No token outlives the Marley that minted it, and none needs rotating by
  hand. A client reads its file through Marley's bridge (`MARLEY_MCP_ENDPOINT=<file>`), which
  rereads it when Marley restarts; a sandboxed client gets that one file mounted in.
- D5: Loopback only: the server keeps binding `127.0.0.1:0`. How another machine reaches it is
  slice 3's.
- D6: The mark: a client's action names the client in the chip in place of "Agent"; while the
  client is allowed and acted on the page within the last minute, the toolbar keeps "Driven by
  <name>" with a Cut Off button.
- D7: A cut-off takes the client out of the live table (its next request gets 403), ends its
  sessions, refuses any call of its that reaches the app afterwards, and deletes its endpoint file
  and its registry entry. To come back it must be allowed again.
- D8: A client holds at most four sessions, so one client cannot use up the server's 32
  (L-claude-491). Clients open no GET stream, whose hang-up is how Marley's own sessions are
  reaped (BF-375), so a fifth `initialize` ends the client's own oldest session rather than being
  refused: a client that crashed four times is not locked out for half an hour. No client ever
  ends a session it does not own.
- D9: Marley's own bearer and bridge keep every tool, and the grants Marley gives itself at start
  (`browser.write`, `mcp.rs:89`).
- D10: What the server reads before it knows who is asking is bounded: the request line and each
  header line at 8 KiB and at most 100 header lines (431 past either), the body at 1 MiB as
  before, and 10 seconds to send the request. Today a local process can send one endless line and
  grow a string until Marley aborts, or hold connection threads open forever.
- D11: A client's `Marley-Terminal`, `Marley-Project` and `Marley-Cwd` headers are dropped: they
  name a Marley terminal, which a client is not. A client's call with no `tab` acts on the tab the
  user focused last, and a page it opens goes to the project of the active window, #574's answer
  for a caller in no project.
- D12: A session belongs to the principal that opened it: a session id presented with another
  principal's bearer is unknown (404).
- D13: The bridge tells a refused token as a refusal: "Marley refused this endpoint file's token:
  the client was cut off, or the file is from an earlier start", where a 403 read "Marley is not
  running" before.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user allows a client by name in Browser Clients, the system shall add it to the registry and write its endpoint file, with the server's URL and a token of the client's own, at mode 0600 in a 0700 folder. | Shot `524-01-allowed`; the log: modes and keys |
| REQ-002 | WHEN a client initializes with its token, the system shall list only the tools its grant names: ten for reading, eighteen for acting, and no terminal, fleet, session, draft or opener tool. | The log: `reader`'s and `driver`'s tool lists |
| REQ-003 | WHEN a read-only client calls a browser write tool, the system shall refuse it with a tool error naming the missing grant and leave the page unchanged. | The log: `reader`'s click refused; the field unchanged in a look |
| REQ-004 | WHILE a client allowed to act acts on a page, the system shall name the client in the tab's chip, and show "Driven by <name>" with Cut Off in the toolbar while the client is allowed and acted within the last minute. | Shot `524-02-driven` |
| REQ-005 | WHEN the user cuts a client off, the system shall refuse the client's next request with 403, delete its endpoint file and registry entry, and remove the tab's mark. | Shot `524-03-cut-off`; the log |
| REQ-006 | WHEN Marley starts, the system shall mint a new token for each allowed client and rewrite its endpoint file, and refuse a token from an earlier start. | The log after `launch_marley` |
| REQ-007 | WHILE clients are allowed, the system shall keep every tool for Marley's own bearer and listen on 127.0.0.1 only. | The log: `mcp_agent tools`; `ss -ltnp` |
| REQ-008 | WHEN a client that holds four sessions initializes again, the system shall end that client's oldest session and give it a new one, and leave Marley's own sessions alone. | The log: the first session's next request 404, the bridge's session working |
| REQ-009 | WHEN a request's line or a header line passes 8 KiB, or a connection sends no whole request within 10 seconds, the system shall answer 431 or close the connection, and keep answering others. | The log: 431, the closed socket, a tool list after |
| REQ-010 | WHEN a client calls a tool outside its grant's list by name, reads a resource or opens a GET stream, the system shall refuse it: a tool error, a JSON-RPC error, and 405. | The log: `terminal_read`, `fleet_snapshot`, `resources/read`, GET |
| REQ-011 | WHEN Marley refuses the token in the bridge's endpoint file, the system shall say in the tool call's error that the token was refused, not that Marley is not running. | The log: `driver`'s call through the bridge after the cut-off |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. Chad's answer on the order of
  slices 1 and 2 is in the notes (either order; slice 1 first).
- **P2 Code:** the pure table and the allowlists, the bounded read, the transport's principal,
  the owned sessions, the app's registry, files and modal, the mark and the bridge's refusal; fmt,
  clippy and dylint clean; a review of the diff against each REQ, with a security read of every
  path that takes a bearer or a byte before it.
- **P3 Test:** write and run the scenario and read every shot; the golden set with 524 added;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_mcp.md`, `marley_workbench.md`,
  `marley_browser.md`; the plan's D15 and D17 (who may reach the browser, and the open DevTools
  ports' true reach); the ledger capture (the unbounded pre-auth read is an F- block); close the
  ticket, archive, commit and push; file slices 2 and 3 as tickets.
