---
pipeline_id: 25679944-2eae-41b5-b6ae-705b71833a0a
ticket: docs/planning/tickets/open/TICKET-524-trusted-outside-browser-access.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Trusted outside clients drive Marley's browser"
type: feature
slice: prong 3 with prong 2's MCP server (plan D15, D17); slice 1 of 3
references: [docs/orca_architecture/04-remote-control-and-mobile.md, docs/planning/pipeline/completed/491-marley-mcp-in-the-app.spec.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md]
---

## Title
Outside programs the user trusts reach Marley's browser through its MCP server: each client is
allowed by name, with a grant to read pages or also to act in them, and holds a token of its own,
new at each start, in an endpoint file of its own. A client sees and calls only the browser tools
its grant allows; the tab it drives shows its name; the user cuts it off at once. The server stays
on loopback. Chromium's own DevTools port, open to every local process, is the second slice's.

## Scope
### In
- `crates/marley_mcp/src/clients.rs` (new, pure): `Caller` (`Marley` for the per-boot bearer, or
  `Client { name, write }`); `ClientTable` (the live clients with their tokens:
  `resolve(bearer) -> Option<Caller>`, comparing with `ct_eq` against every entry; `revoke(name)`;
  when each last called); `decide_for(caller, spec, grants)`: Marley as today, a client only in
  the browser family, read tools with either grant, write tools with the write grant;
  `tools_list_for(caller)`.
- `crates/marley_mcp/src/transport.rs`: `serve_connection` resolves the caller from the bearer
  (else 403, as today) and carries it into `handle_message` and `ask_app`; each session records
  its caller; a client holds at most four sessions; `ServerData` holds the `ClientTable`, and a
  revoke terminates the client's sessions.
- `crates/marley_mcp/src/marley_mcp.rs`, `dispatch.rs`, `session.rs`: `RequestCtx` and `AppCall`
  carry the caller; `tools/list` and `tools/call` go through `decide_for`.
- `crates/marley_workbench/src/clients.rs` (new): the registry `<data>/mcp/clients.json` (mode
  0600: each client's name, grant and when it was allowed, no token); at each start a token per
  client from OS entropy, written with the URL into `<data>/mcp/clients/<name>.json` (mode 0600,
  the shape and write path of `mcp-endpoint.json`), removed at quit and at a cut-off; the Browser
  Clients modal (`marley: browser clients`): the clients with their grant, their last call and
  Cut Off, and a form (a name, "May act in pages", Allow), after which it shows the client's
  endpoint file with Copy and the line that points Marley's bridge at it.
- `crates/marley_workbench/src/browser_tools.rs` and `browser.rs`: a client's action names the
  client in the tab's chip, where Marley's own agents show "Agent"; while the client holds a
  session and acted on the page in the last minute, the toolbar keeps "Driven by <name>" with
  Cut Off; the recorder's agent entry names the client.
- `crates/marley_workbench/src/mcp.rs`: the server starts with the client table; `answer`
  refuses a call whose client was cut off while the call waited.
- `script/e2e/browser-fixture.sh`: `mcp_agent --endpoint <file>`.
  `script/e2e/524-trusted-outside-browser-access.sh`.

### Out (explicitly deferred)
- **Slice 2, the side door.** Chromium's DevTools port stays open on 127.0.0.1, with no
  credential, to every local process until then. Slice 2 starts Chromium with
  `--remote-debugging-pipe` under a small Marley relay in the same unit, which serves Marley over
  a Unix socket in `$XDG_RUNTIME_DIR` (mode 0600) and serves Playwright clients (#523's runner,
  the Playwright MCP, the e2e fixture's stand-in agent) a CDP WebSocket on loopback that takes a
  client's token. It amends plan D16 ("any other CDP client find[s] it" in `DevToolsActivePort`)
  and needs its own spec.
- **Slice 3, other machines.** A fixed loopback port behind `ssh -L` (rustal-harness M9's rule:
  "Remote clients arrive through M8's authenticated SSH path") or `tailscale serve` (report 04
  §3.2 item 3). Loopback only until then.
- Terminal, fleet and session tools for clients; per-project or per-tab grants (a client reaches
  every project's tabs, as Marley's own agents do, #507 D8).
- A section on #515's Marley settings page: the registry is Marley's file, not Zed settings (D3).
- Pairing by QR, push, app-layer encryption (report 04 §2.2 and §2.3): not needed on loopback.

## Reference (§20)
Orca's paired devices (report 04 §2.2): a token per device, a registry written 0600, devices
listed and revocable, and a revoke that closes the device's open sockets; a scope that decides
which methods a token may call (`src/main/runtime/device-registry.ts`,
`src/main/runtime/runtime-rpc/runtime-rpc-mobile-method-allowlist.ts`). Marley keeps the named
clients, the per-client tokens, the scope (as grants over tool classes) and the revoke that ends
sessions. It leaves out what report 04 §2.13 faults: direct tokens that never expire, and a scope
that amounts to a shell on the desktop. The house pattern on this box is rustal-harness M9's
("An MCP server in the harness on loopback with separate read and write grants. Remote clients
arrive through M8's authenticated SSH path", `/srv/stacks/rustal-harness/docs/ROADMAP.md`, M9;
its R9-05: "The server shall refuse unauthenticated callers and write verbs without a write
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
  already follow (bind loopback, validate `Origin`, authenticate; `auth.rs`, line 1). Chromium's
  DevTools endpoint takes no credential (plan D15: "takes no token (Chromium offers none)").
  `--remote-debugging-pipe`, for slice 2: Playwright launches its own Chromium with it
  (`playwright-core` 1.63.0, `lib/coreBundle.js`), and Puppeteer's `pipe` launch option is
  documented as "Connect to a browser over a pipe instead of a WebSocket".
- **The code we already ship.** `marley_mcp` has every part but the table: `bearer_ok` and
  `ct_eq` (`auth.rs`), `origin_allowed` and `is_loopback`, the bind to `127.0.0.1:0`
  (`transport.rs:118`) and the bearer check before any dispatch (`transport.rs:178`),
  `mint_secret` over `/dev/urandom` (`secret.rs`), the 0600 file written through its open handle
  (`discovery.rs`, `write_discovery_file_in`), the session registry and its cap of 32
  (`session.rs:16`), `GrantTable` and `decide` (`permission.rs:14`, `54`), where read tools need no
  grant, and `ExposeConfig` (`expose.rs:18`), whose read classes (`allow`) are "carried opaquely"
  and never consumed: the shape a client's read grant needs. The bridge follows
  `MARLEY_MCP_ENDPOINT` and sends a bearer only to a loopback URL (`marley-mcp-bridge`,
  `endpoint_path`). The tab's Agent chip (`agent_started`, `browser.rs:1610`; `agent_ended`, 1626;
  `agent_chip`, 1670; drawn at 4131 as `Chip::new("Agent")`). Zed's `context_server` carries an
  MCP server that binds a Unix socket in a temporary folder
  (`crates/context_server/src/listener.rs:55`) and that nothing in the tree starts today; a socket
  file admits only who can open its path, the property slice 2 wants for Marley's own CDP link.
  Does a crate we build own the seam? Yes: `marley_mcp` owns authentication, sessions and grants,
  so the ticket extends it and adds no second server.

## UI proof
UI-AFFECTING (a modal, the tab's chip and mark). `script/e2e/524-trusted-outside-browser-access.sh`
(`compositor sway`: it clicks Cut Off). Setup: the offline Chromium; a loopback page with a Name
field and a Save button; the scratch repository. Steps and shots: `marley: browser clients`, then
`reader` allowed to read only and `driver` allowed to act, the modal listing both with their
endpoint files (`524-01-allowed`); the browser on the page; the stand-in agent, through
`driver`'s endpoint file, types into the Name field, and the tab shows "driver" in its chip and
"Driven by driver" with Cut Off (`524-02-driven`); a click on Cut Off, the mark gone
(`524-03-cut-off`). The run log carries: both files' modes and keys (never their tokens);
`reader`'s tool list and its refused click; `driver`'s request with its old token refused with
403 after the cut-off; Marley's own endpoint still listing every tool; `ss -ltnp` for the
server's port; after a relaunch, `reader`'s rewritten file working and its old token refused; a
fifth `reader` session refused with 503 while Marley's own bridge still starts one.

## Locked-In Decisions
- D1: The front door first: outside clients reach the browser through Marley's MCP browser tools,
  never raw CDP, in this slice. The tools act where the user watches (the tab comes forward, the
  chip says what happened), run no script (AD-claude-492), navigate only to `http` and `https`,
  and hide secrets in what they read (plan D15). Raw CDP would hand a client the cookies, script
  evaluation and every page, with nothing on the screen.
- D2: A client has a name and one grant: read (`browser_tabs`, `look`, `snapshot`, `console`,
  `network`, `picks`, `pick`, `annotations`, `recordings`, `recording`) or read and write (plus
  `navigate`, `back`, `click`, `type`, `press`, `scroll`, `annotate`). Clients reach the browser
  family only: never the terminal tools, whose output can hold anything, nor the fleet and session
  tools.
- D3: The registry is Marley's own file, `<data>/mcp/clients.json` (0600), never Zed settings: a
  project's `.zed/settings.json` merges into the user's settings, so a repository could grant
  itself a client. It holds names, grants and dates, and no token.
- D4: A client's token is minted at each start, as the bearer is, and written only into that
  client's endpoint file, `<data>/mcp/clients/<name>.json` (0600), removed at quit and at a
  cut-off. No token outlives the Marley that minted it, and none needs rotating by hand. A client
  reads its file through Marley's bridge (`MARLEY_MCP_ENDPOINT=<file>`), which rereads it when
  Marley restarts; a sandboxed client gets that one file mounted in.
- D5: Loopback only: the server keeps binding `127.0.0.1:0`. How another machine reaches it is
  slice 3's.
- D6: The mark: a client's action names the client in the chip in place of "Agent"; while the
  client holds a session and acted on the page within the last minute, the toolbar keeps "Driven
  by <name>" with a Cut Off button.
- D7: A cut-off takes the client out of the live table (its next request gets 403), terminates its
  sessions (its standing stream closes), answers any of its calls still waiting in the app with a
  refusal, and deletes its endpoint file and its registry entry. To come back it must be allowed
  again.
- D8: A client holds at most four sessions, so one client cannot use up the server's 32
  (L-claude-491).
- D9: Marley's own bearer and bridge keep every tool, and the grants Marley gives itself at start
  (`browser.write`, `mcp.rs:78`).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user allows a client by name in Browser Clients, the system shall add it to the registry and write its endpoint file, with the server's URL and a token of the client's own, at mode 0600. | Shot `524-01-allowed`; the log: modes and keys |
| REQ-002 | WHEN a client initializes with its token, the system shall list only the browser tools its grant allows, and no terminal, fleet or session tool. | The log: `reader`'s and `driver`'s tool lists |
| REQ-003 | WHEN a read-only client calls a browser write tool, the system shall refuse it with a tool error naming the missing grant and leave the page unchanged. | The log: `reader`'s click refused; the page's field unchanged |
| REQ-004 | WHILE a client with the write grant acts on a page, the system shall name the client in the tab's chip, and show "Driven by <name>" with Cut Off in the toolbar while the client's session lives and it acted within the last minute. | Shot `524-02-driven` |
| REQ-005 | WHEN the user cuts a client off, the system shall refuse the client's next request with 403, end its sessions, refuse its waiting calls, delete its endpoint file and remove the tab's mark. | Shot `524-03-cut-off`; the log |
| REQ-006 | WHEN Marley starts, the system shall mint a new token for each allowed client and rewrite its endpoint file, and refuse a token from an earlier start. | The log after `launch_marley` |
| REQ-007 | WHILE clients are allowed, the system shall keep every tool for Marley's own bearer and listen on 127.0.0.1 only. | The log: `mcp_agent tools`; `ss -ltnp` |
| REQ-008 | WHEN a client that holds four sessions initializes again, the system shall refuse it with 503 and still give Marley's own bridge a session. | The log |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes. At promotion, ask Chad the
  notes' open question (the order of slices 1 and 2), and check what #516 changed in the MCP
  server and the fixture.
- **P2 Code:** the pure table and decisions, the transport, the app's registry and modal, the mark;
  fmt and clippy clean; a review of the diff against each REQ, with a security read of every path
  that takes a bearer.
- **P3 Test:** write and run the scenario and read every shot; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley_architecture/marley_mcp.md`; the plan's D15 and D17
  (who may reach the browser, and the open DevTools port's true reach); the ledger capture; close
  the ticket, archive, commit; file slices 2 and 3 as tickets.
