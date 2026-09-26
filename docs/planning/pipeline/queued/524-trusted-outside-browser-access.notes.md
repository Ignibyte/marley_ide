# Trusted outside clients drive Marley's browser — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-524-trusted-outside-browser-access.md
- **Pipeline spec:** 524-trusted-outside-browser-access.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25: "marley should also expose its browser to trusted outside so
  they can drive for the user." The prompt for this spec: design the trusted path (clients named
  and allowed by the user, a per-client token, a CDP proxy or the MCP browser tools behind the
  existing grants, loopback only by default, a visible mark on a tab an outside client drives, a
  way to cut a client off), after checking the flags Chromium starts with and where its port is
  published.
- **Classification / tier:** feature, prong 3 with the MCP server. Too big for one slice, so it is
  split in three (below); this spec is slice 1. Marley crates and the fixture; no Zed path. Size
  M for slice 1.
- **The split.**
  1. The front door (this spec): named clients with per-start tokens through Marley's MCP browser
     tools, their grants, the tab's mark, the cut-off. Loopback.
  2. The side door: Chromium off TCP. The unit runs a small Marley relay that starts Chromium
     with `--remote-debugging-pipe` and serves Marley over a Unix socket in `$XDG_RUNTIME_DIR`
     (mode 0600, so the kernel admits only the user) and Playwright clients over a loopback CDP
     WebSocket that takes a client's token from this slice's table. It changes the CDP client's
     transport (`marley_browser::cdp::connect` takes a port today), #507's and #523's endpoint
     lookups, and the e2e stand-in agent; it amends plan D16.
  3. Other machines: a fixed loopback port behind `ssh -L`, as rustal-harness M9 does, or
     `tailscale serve` with its certificate (report 04 §3.2 item 3), for a Playwright MCP on
     another machine or a phone.
- **Open question for Chad.** Slice 1 lets a named client in without closing the side door, so
  until slice 2 lands every local process can still drive the browser through CDP, as today.
  Build slice 2 first instead? *Default: slice 1 first, since it is what outside clients need and
  it changes nothing that exists; slice 2 next.*
- **Recall (§18.3):**
  - AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001: a per-boot bearer
    in a 0600 file, removed at quit; a bare HTTP entry, an environment variable and a `.mcp.json`
    per project were rejected for the endpoint. The client files follow the same rule (D4).
  - AD-claude-492-agents-drive-the-browser-tab-through-the-mcp-server-001: `browser.write` is
    granted at start because the client's approval of each call and the Agent chip are the checks;
    no tool evaluates script. D1 keeps outside clients inside that design.
  - L-claude-491-a-session-per-agent-needs-room-and-a-close-001: the cap of 32 refuses and never
    evicts, and an idle session lives 30 minutes; D8 caps a client at four.
  - BF-mcp-pre-auth-body-alloc-001 (the failures ledger): the transport caps a request before it
    authenticates; the caller lookup runs after that cap and before any dispatch, as the bearer
    check does now.
  - Brain: a page on the Playwright MCP, which Chad runs on another machine: the likely first
    client of slice 3.
- **Discovery:**
  - Chromium's start (`crates/marley_browser/src/service.rs`): `chromium_args` (81) passes
    `--headless`, `--remote-debugging-port=0`, `--user-data-dir=<profile>`, `--no-first-run`,
    `--no-default-browser-check`, `--password-store=basic` and `--no-startup-window`; no
    `--remote-debugging-address`, so Chromium binds 127.0.0.1. The port is published in
    `<profile>/DevToolsActivePort`, the port on the first line and the browser's WebSocket path on
    the second (`endpoint_in`, 212). On the dev box on 2026-09-25 the running unit listened on
    `127.0.0.1` only (`ss -ltnp`), its profile folder mode 0700 and the file 0644 inside it.
  - Who can reach that port: any local process. A TCP port on loopback checks no user; a client
    needs only the port, which a scan of loopback finds. Plan D15 says "the exposure is to
    processes of the same user, who could already read the profile directory", which holds for
    the file and not for the port. On the dev box, processes of more than a dozen other system
    users run (database, web and mail services among them). D15's text is corrected at Complete.
  - `crates/marley_mcp`: `transport.rs` binds `127.0.0.1:0` (118), mints the bearer from
    `/dev/urandom` (`read_entropy`, `mint_secret`), checks `Origin` and the bearer before any
    dispatch (178), then the session gate (`session_gate`), then `handle_message` under the lock,
    and `ask_app` (280) hands deferred calls to the app with the lock released; `ServerData` (34)
    holds the snapshot, the grants, the surface index, a version and the sessions. `auth.rs`:
    `is_loopback`, `origin_allowed`, `ct_eq`, `bearer_ok`. `permission.rs`: `GrantTable` (14),
    `decide` (54), where the read tier always passes. `expose.rs`: `ExposeConfig` (18) with
    `allow` (21, read classes, unused) and `allow_write`. `session.rs`: `SESSION_CAP` (16), the
    registry (43), `assign` (63), `terminate` (109). `dispatch.rs`: `tools_call` (133), the
    decision (153), the deferred arm (165). `registry.rs`: `Family` (12), `browser_write` (229:
    the grant class `browser.write`), `tools_list` (260). The browser family's read tools are
    `tabs`, `look`, `snapshot`, `console`, `network`, `picks`, `pick`, `recordings`, `recording`
    and `annotations`; its write tools are `annotate`, `navigate`, `back`, `click`, `type`,
    `press` and `scroll`. `marley_mcp.rs`: `AppCall` (130) with `new` (141) and `answer` (155).
  - `crates/marley_workbench/src/mcp.rs`: `start` (60) builds the shared state with
    `GrantTable::from_classes(["browser.write"])` (78), writes `mcp-endpoint.json`, registers the
    bridge with Zed's agents, removes the file at quit; `answer` (219) routes by tool name.
  - The bridge (`crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge`):
    `endpoint_path` reads `$MARLEY_MCP_ENDPOINT`, else the default file; it polls every two
    seconds and answers an empty tool list while no Marley answers; it sends the bearer only to
    `127.0.0.1`, `localhost` or `::1`.
  - The Browser tab: `agent_started` (`browser.rs:1610`), `agent_ended` (1626, which also puts an
    `Agent` entry in the page's minute), `agent_chip` (1670), the chip drawn at 4131; the tools'
    `answer` (`browser_tools.rs:53`).
  - Zed's own MCP server: `context_server::listener::McpServer::new`
    (`crates/context_server/src/listener.rs:55`) binds `mcp.sock` in a temporary folder with
    `UnixListener` and registers tools with `add_tool` (87); no crate in the tree starts it now.
    Slice 2's relay can follow its shape for Marley's side of the CDP link.
- **Decisions:** D1 to D9 in the spec.

### Design
- **Pure core (`marley_mcp`).** `pub enum Caller { Marley, Client(ClientGrant) }` with
  `ClientGrant { name: String, write: bool }`. `pub struct ClientTable` holding
  `(ClientGrant, token, last_call_ms, sessions)` per client, its `Debug` redacting tokens as the
  session registry's does. `resolve(presented) -> Option<Caller>`: the per-boot bearer first, then
  every client token with `ct_eq`, no early exit. `decide_for(caller, spec, grants)`: `Marley` goes
  to today's `decide`; a client passes only when `spec.family == Family::Browser` and the tool is
  read tier, or write tier with `write`; otherwise `Deny` naming the grant it lacks.
  `tools_list_for(caller)` filters the list by the same rule. The session registry records each
  session's caller and counts a client's sessions; `terminate_all(name)` ends a client's.
- **Transport.** After the size cap and `Origin`, `resolve` the bearer (403 when `None`); pass the
  caller to the session gate (the four-session cap), into `RequestCtx`, and into `AppCall`, which
  gains `caller()`. `ServerData` gains `clients: ClientTable`; the app updates it under the lock.
  A standing stream whose session was terminated closes at its next wake.
- **The app (`clients.rs`).** At `mcp::start`, read `clients.json` (a file it cannot parse is
  left as it is and logged, never overwritten, as Orca's loaders refuse), mint a token per client
  with `mint_secret`, fill the table, and write each `clients/<name>.json` with
  `write_discovery_file_in` (the URL, and the `Authorization` header as in
  `transport::discovery_json`); remove them at quit. Allow: validate the name (letters, digits,
  `-` and `_`; not taken), add the entry, mint, write, update the table. Cut off: take it out of
  the table, terminate its sessions, delete its file, save the registry. The modal is a Zed
  `ModalView`: the list (name, "reads" or "reads and acts", the last call's age, Cut Off), then the
  form (a single-line `Editor` for the name, a checkbox, Allow); after Allow it shows the file's
  path with Copy and the line `MARLEY_MCP_ENDPOINT=<file> <data>/mcp/marley-mcp-bridge`.
- **The mark.** `AgentAction` gains `client: Option<SharedString>`; the chip reads the client's
  name when there is one. `PageState` gains `driven_by: Option<(SharedString, Instant)>`, set by a
  client's write; the toolbar shows "Driven by <name>" and a Cut Off button while that client is
  in the table and the instant is within a minute; Cut Off runs the same cut-off. The page's
  minute records the client's name in its `Agent` entry.
- **Refusing late calls.** `browser_tools::answer` checks the call's caller against the live
  table before it acts; a client cut off while its call waited gets "cut off by the user".
- **File manifest.** Marley crates: `crates/marley_mcp/src/clients.rs` (new), `transport.rs`,
  `session.rs`, `dispatch.rs`, `marley_mcp.rs`; `crates/marley_workbench/src/clients.rs` (new),
  `mcp.rs`, `browser_tools.rs`, `browser.rs`, `marley_workbench.rs` (the action). Scripts:
  `script/e2e/browser-fixture.sh`, `script/e2e/524-trusted-outside-browser-access.sh`. No new
  dependency (`ct_eq` and `mint_secret` are the crate's own). No Zed path.
- **Ledger rows.** None in `docs/marley/zed-touchpoints.md`. At Complete: an AD for outside
  clients; the correction of D15's reasoning in the plan.

### E2E plan
The scenario's tokens live only in the run's scratch profile and are deleted with it. To show a
refused old token, the scenario copies `driver`'s endpoint file to `$E2E_WORK` before the
cut-off, and `reader`'s before the relaunch, and never prints either.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `marley: browser clients`; type `reader`, Allow; type `driver`, check "May act in pages", Allow | `524-01-allowed`; the log: `stat -c %a` of both files, `jq 'keys'` of each |
| REQ-002 | `mcp_agent --endpoint <reader file> tools`; the same for `driver` | the log: ten read tools for `reader`, seventeen for `driver`, no `terminal_` tool |
| REQ-003 | `mcp_agent --endpoint <reader file> click-on button Save`; `mcp_agent look` | the log: the refusal naming `browser.write`; the page unchanged in the look |
| REQ-004 | the browser on the page; `mcp_agent --endpoint <driver file> type-into textbox Name "from the driver"` | `524-02-driven` |
| REQ-005 | click Cut Off; `curl` with the copied `driver` token | `524-03-cut-off`; the log: 403, and no `clients/driver.json` |
| REQ-007 | `mcp_agent tools` through Marley's own endpoint; `ss -ltnp` for Marley's pid | the log: every tool; `127.0.0.1` |
| REQ-006 | `quit_marley`; `launch_marley`; `curl` with the copied `reader` token; `mcp_agent --endpoint <reader file> tabs` | the log: 403 for the old token, the tabs through the new file |
| REQ-008 | four `initialize` requests with `reader`'s token held open, then a fifth; then Marley's bridge | the log: 503 for the fifth, a session for the bridge |

### Risks
- Slice 1 alone leaves the DevTools port as it is: open to every local process. The ticket says
  so, the open question asks Chad, and slice 2 is filed at Complete.
- A same-user process can still read Marley's own endpoint file and every client file (they are
  the user's files); the client grants limit a client that plays by the rules and holds only its
  own file, such as a sandboxed agent. The kernel's file modes are the only wall between users.
- A client's endpoint file names its token; a client that copies it elsewhere keeps it for that
  run only (D4).
- `resolve` compares the bearer against every token on each request; with a handful of clients
  that is nothing, and it keeps the compare constant-time across entries.
- The modal is new UI in a Marley crate; its text field and checkbox follow the pick caption's
  editor and Zed's `Checkbox`, and the scenario clicks them under sway.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
