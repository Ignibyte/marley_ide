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
  L for slice 1 since its promotion added the bounded read and owned sessions.
- **The split.**
  1. The front door (this spec): named clients with per-start tokens through Marley's MCP browser
     tools, their grants, the tab's mark, the cut-off. Loopback.
  2. The side door: Chromium off TCP. The unit runs a small Marley relay that starts Chromium
     with `--remote-debugging-pipe` and serves Marley over a Unix socket in `$XDG_RUNTIME_DIR`
     (mode 0600, so the kernel admits only the user) and Playwright clients over a loopback CDP
     WebSocket that takes a client's token from this slice's table. It changes the CDP client's
     transport (`marley_browser::cdp::connect` takes a port today), #507's per-project endpoint
     lookups, #523's runner (`MARLEY_CDP_URL`), and the e2e stand-in agent; it amends plan D16.
  3. Other machines: a fixed loopback port behind `ssh -L`, as rustal-harness M9 does, or
     `tailscale serve` with its certificate (report 04 §3.2 item 3), for a Playwright MCP on
     another machine or a phone.
- **Chad's answer, 2026-09-26.** The open question (slice 2 first?): the trusted-client list and
  the closed DevTools port, "either order doesn't matter, we haven't shipped this product yet".
  Slice 1 first, as the default said.
- **Promotion, 2026-09-27** (no task tool; checklist): pick ✓; pre-flight ✓ (no active pipeline,
  a clean tree at 577c3b044c; cargo busy with the release install, which Plan does not need);
  recall ✓; promote ✓ (the pair moved from `queued/`, the backlog row removed, the ticket in
  progress); the seams re-verified ✓ (an Explore agent, all twenty cited seams); the prior-art
  sweep ✓ (the queued sweep stands; the code leg re-read); the spec ✓; the design ✓.
- **Recall (§18.3):**
  - AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001: a per-boot bearer
    in a 0600 file, removed at quit. The client files follow the same rule (D4).
  - AD-claude-492-agents-drive-the-browser-tab-through-the-mcp-server-001: `browser.write` is
    granted at start because the client's approval of each call and the Agent chip are the checks;
    no tool evaluates script. D1 keeps outside clients inside that design.
  - AD-claude-520-each-terminal-names-itself-and-the-bridge-names-the-caller-001: `Caller` is
    taken, and "the id scopes defaults and is no authority: the bearer gates every call". The new
    type is `Principal`, carried beside it (D11 drops a client's `Marley-*` headers).
  - PR-claude-cap-client-size-before-alloc-pre-auth-001 and BF-mcp-pre-auth-body-alloc-001: a
    pre-auth path caps what a client controls before allocating. The body is capped; the request
    line and the header lines are not (D10).
  - BF-375-session-cap-no-reclamation and L-claude-491-a-session-per-agent-needs-room-and-a-close-001:
    a cap that refuses needs a reclamation path. Clients open no GET stream, so a client's fifth
    session ends its own oldest (D8).
  - F-claude-561-an-opener-on-the-default-endpoint-would-open-tabs-in-another-marley-001: a helper
    finds the Marley that wrote it; the client files live under the data directory that wrote them.
  - Brain: consultation 5d5fc98defb44023b84e0ce7ed4e95cb (a second wording,
    fa024d89c96343d9bb7cf913682e245f, closed as a duplicate): nothing on this seam beyond the
    Playwright MCP page, the likely first client of slice 3.
- **Discovery, re-verified at 577c3b044c** (what moved since the draft of 2026-09-25):
  - `marley_mcp` still binds `127.0.0.1:0` (`transport.rs:118`), checks `Origin` and the bearer
    before any dispatch (177), and caps the body (`MAX_BODY_BYTES`, 423). The request line and
    each header are read with `read_line` and no cap (430 to 462), and the server sets no read
    timeout, so a local process can grow one line until Marley aborts, or hold connection threads
    open without end: a pre-existing bug, fixed here (D10), an F- block at Complete.
  - #520 added `Caller { terminal, project, cwd }` (`marley_mcp.rs:135`), built from the
    `Marley-Terminal`, `Marley-Project` and `Marley-Cwd` headers (`transport.rs:456`) for any
    bearer, carried by `AppCall::new(tool, arguments, caller, answer)` (158) and read with
    `caller()` (174); `ask_app` (281) takes it. `RequestCtx` (89) has no caller.
  - `read_entropy` (`transport.rs:93`) is private; `mint_secret` is `secret.rs:44`.
    `write_discovery_file_in` (`discovery.rs:28`) and `remove_discovery_file_in` (49) hard-code
    the file's name; the main endpoint is `<data>/mcp-endpoint.json`, and `<data>/mcp/` is made
    with the default mode by `write_program_in` (`mcp.rs:280`).
  - `SessionEntry { id, last_seen_ms }` (`session.rs:35`) has no owner, so any bearer can use or
    DELETE any live session id; `validate` (77) compares with no early exit.
  - `tools/list` (`dispatch.rs:28`) takes no context; `resources/list`, `read` and `subscribe`
    (29 to 31) and the GET stream never reach `decide`; `tools/call` (133) finds unlisted tools by
    name (`fleet_snapshot`, which since #547 carries each Claude Code session's prompt, tool,
    message and folder). The GET stream (`serve_sse_stream`, `transport.rs:324`) pushes a
    `resources/updated` on every fleet change, and ends only when a write fails.
  - The browser family (`registry.rs`) has 20 tools: read `tabs`, `look`, `snapshot`, `console`,
    `network`, `picks`, `pick`, `recordings`, `recording`, `draft_test` (#506), `annotations`;
    write `annotate`, `check_pick` (#505), `navigate`, `open_url` (#561), `back`, `click`,
    `type`, `press`, `scroll`. `browser_write` is at 278 and `tools_list` at 309.
  - The app (`mcp.rs`): `start` (68) grants `browser.write` (89) and calls `transport::spawn`
    (95); after that only the `publisher` task holds `Shared`, and the `McpServer` global (56)
    keeps `failure` and `snapshots`. `answer` (389) sends `browser_*` to `browser_tools::answer`.
  - `browser_tools.rs`: `answer` (89), `caller_scope` (105) and `caller_project` (133): a caller
    in no project gets the tab the user focused last and the active window's project;
    `page_of` (336); the chip is set by `acting` (1007), `check_pick` (771) and `annotate` (1430).
  - `browser.rs`: `AgentAction { text, ended }` (159), `agent_started` (2371), `agent_ended`
    (2387, which records `RecordedEntry::Agent { did }` and sets #504's `agent_unseen`),
    `agent_chip` (2436), `render_toolbar` (5290) with the chip at 5366 after the Scripts,
    Pick, Annotate and record buttons.
  - The bridge: `endpoint_path` (54), a two-second poll, loopback only for the bearer, no GET
    stream, DELETE at close, and a 403 raised as `Unreachable`, answered as "Marley is not
    running" (163, 186). `marley-open-url` (#561) is a second client of the same endpoint file.
  - The fixture: `mcp_agent` sets `MARLEY_MCP_ENDPOINT` inline, so it cannot take another file;
    `write_mcp_agent`'s client sends the bridge's stderr to `DEVNULL`, so it sees no status.
  - No Marley crate has a form modal (the two `ModalView`s wrap pickers) or a `Checkbox`.
  - `crates/marley_browser/src/service.rs`: `chromium_args` (232) passes no
    `--remote-debugging-address`; each project has its own `DevToolsActivePort` since #507.
- **Decisions:** D1 to D13 in the spec. D2 became an explicit list; D8, D10 to D13 are new at the
  promotion.

### Design
- **The pure core (`marley_mcp::clients`).** `pub enum Principal { Marley, Client(ClientGrant) }`,
  `pub struct ClientGrant { pub name: String, pub write: bool }`. `ClientTable` holds a
  `ClientEntry { grant, token, last_call_ms }` per client; its `Debug` names no token.
  `allow(name, write)` checks the name (1 to 32 of letters, digits, `-` and `_`; not `agent` or
  `marley` in any case, which the chip would show as Marley's own; not taken) and mints the token
  with `mint_secret(read_entropy())` (`read_entropy` becomes `pub(crate)`), returning it;
  `cut_off(name)`; `touch(name, now)`; `list()` for the modal. `resolve(presented, marley_bearer)`
  compares the presented bearer with `ct_eq` against Marley's bearer and every token and keeps the
  match, with no early exit. `CLIENT_READ_TOOLS` (10) and `CLIENT_WRITE_TOOLS` (8), and
  `permits(principal, tool) -> Result<(), String>`: Marley passes; a client passes a tool on its
  grant's list; a read client's write tool is refused as "needs the grant to act in pages; this
  client may only read"; any other tool as "not open to outside clients".
- **Where the table lives.** `pub type Clients = Arc<RwLock<ClientTable>>`, apart from `Shared`,
  so resolving a bearer never takes the data lock: #375's rule that an unauthenticated request
  never touches the session registry holds. `spawn` takes it; the app keeps a clone.
- **The bounded read (`transport.rs`).** Before `read_http_request`, a 10-second read timeout on
  the stream, cleared once the request is read. Each line is read through `Read::take` at 8 KiB
  plus one; a line that fills it, or a 101st header line, is `TooLarge`, answered 431 with no
  further read. A timeout ends the connection without a reply.
- **The principal in the transport.** After the size cap and `Origin`, `resolve` (403 when none);
  a client's request gets `Caller::default()` (D11) and a `touch`; a client's GET gets 405. The
  session gate takes the owner (`Owner::Marley` or the client's name). `RequestCtx` gains
  `principal`; `handle_message` hands it to `tools/list` (filtered for a client),
  `tools/call` (`permits` before `lookup`) and `resources/*` (a JSON-RPC error for a client).
  `ask_app` passes it to `AppCall::new`, which gains `principal()`. The app's calls:
  `transport::allow_client(&clients, name, write) -> Result<String, ClientError>` and
  `transport::cut_off_client(&shared, &clients, name)`, which removes the client and ends its
  sessions under the data lock.
- **Owned sessions (`session.rs`).** `SessionEntry` gains `owner`. `assign(id, owner, now)`: for a
  client with four sessions, its least recently seen session is ended first; the global cap of 32
  applies as before. `validate` and `terminate` match the owner, so another principal's id is
  unknown (404). `terminate_owned_by(name)`.
- **Files (`discovery.rs`, `marley_workbench::clients`).** `write_discovery_file_in(dir, name,
  json)` and `remove_discovery_file_in(dir, name)`; Marley's own call passes `mcp-endpoint.json`.
  The registry, `<data>/mcp/clients.json`: `{ "clients": [{ "name", "write", "allowed_at" }] }`,
  written 0600 through a temporary file renamed into place, read with serde; a file that does not
  parse is logged and left, and Allow and Cut Off then refuse with the reason. The client files:
  `<data>/mcp/clients/` made 0700, `<name>.json` in the shape of `discovery_json`. At start every
  file in the folder is removed (a crash leaves stale ones), then one is written per client; at
  quit all are removed.
- **The modal (`clients.rs`).** `BrowserClients`, a `ModalView` opened by `marley: browser
  clients`: a row per client (name, "reads" or "reads and acts", the last call's age or "no call
  yet", Cut Off), then the form (a one-line `Editor` for the name, `ui::Checkbox` "May act in
  pages", Allow), then, after an Allow, the endpoint file's path with Copy and the line
  `MARLEY_MCP_ENDPOINT=<file> <data>/mcp/marley-mcp-bridge` with Copy. An error (a bad or taken
  name, an unreadable registry) shows under the form. Escape closes it.
- **The app's server (`mcp.rs`).** The `McpServer` global keeps `Shared` and `Clients` once the
  server runs. `start` reads the registry, allows each client, and writes the files; the quit
  removes them. `answer` hands the principal to `browser_tools::answer` and refuses a client's
  call to any other tool (the transport already did; this is the second wall).
- **The mark and the late call (`browser_tools.rs`, `browser.rs`).** `run` takes the client's
  name; before it acts, a client no longer in the table is refused "cut off by the user". The
  name reaches `acting`, `check_pick` and `annotate`, and `agent_started(target, text, client)`;
  `AgentAction` gains `client`, and `agent_chip` draws the name in place of "Agent". A client's
  write tool sets `PageState::driven_by` (the name and the instant); the toolbar draws "Driven by
  <name>" and a Cut Off button while that client is allowed and the instant is within a minute,
  with a timer that redraws at the minute's end; Cut Off runs the same cut-off as the modal.
  `agent_ended` records `RecordedEntry::Agent { did, by }` (`by` new, skipped when empty, so old
  recordings read).
- **The bridge.** A 403 raises `Refused`; the bridge then answers `tools/list` empty and a tool
  call with D13's words, and keeps polling the file, so a client allowed again or restarted
  Marley's new file is picked up.
- **File manifest.** Marley crates: `crates/marley_mcp/src/clients.rs` (new), `transport.rs`,
  `session.rs`, `dispatch.rs`, `discovery.rs`, `marley_mcp.rs`; `crates/marley_browser/src/recorder.rs`;
  `crates/marley_workbench/src/clients.rs` (new), `mcp.rs`, `browser_tools.rs`, `browser.rs`,
  `marley_workbench.rs` (the action and the module), `claude_plugin/marley/bin/marley-mcp-bridge`.
  Scripts: `script/e2e/browser-fixture.sh`, `script/e2e/524-trusted-outside-browser-access.sh`,
  `script/e2e/golden`. The old unit tests of `registry.rs`, `dispatch.rs` and `session.rs` keep
  compiling (`--all-targets`). No new dependency. No Zed path, so no ledger row.
- **At Complete.** CHANGELOG (Added; the bounded read under Fixed); `marley_mcp.md`,
  `marley_workbench.md`, `marley_browser.md`; the plan's D15 (the DevTools ports reach every local
  process, not the user's alone) and D17 (named clients); an AD for outside clients, an F- for
  the unbounded read; slices 2 and 3 filed.

### E2E plan
`compositor sway`, the offline Chromium, a loopback page `form.html` with a Name field, a Save
button and a line that shows what was saved. The tokens stay in the run's scratch profile and are
never printed: `mcp_http` prints statuses and messages, and the old tokens are kept by copying the
endpoint files into `$E2E_WORK` before they change.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `marley: browser clients`; type `reader`, Allow; type `driver`, click "May act in pages", Allow | `524-01-allowed`; the log: `stat -c %a` of the folder and both files, `jq -r 'keys'` of each |
| REQ-002 | `mcp_agent --endpoint <reader file> tools`; the same for `driver` | the log: 10 tools for `reader`, 18 for `driver`, none of `terminal_`, `fleet_`, `session_`, `draft_test`, `open_url` |
| REQ-003 | the browser on `form.html`; `mcp_agent --endpoint <reader file> click-on button Save`; `look` | the log: the refusal naming the grant; the saved line empty |
| REQ-004 | `mcp_agent --endpoint <driver file> type-into textbox Name "from the driver"`, then `click-on button Save` | `524-02-driven`: the chip reads "driver", the toolbar "Driven by driver" and Cut Off, the page "Saved: from the driver" |
| REQ-010 | `mcp_http <reader file>`: `tools/call terminal_read`, `tools/call fleet_snapshot`, `resources/read`, GET | the log: two tool errors, a JSON-RPC error, 405 |
| REQ-005 | copy `driver`'s file; click Cut Off | `524-03-cut-off`: the mark gone; the log: `mcp_http` with the copy: 403; no `clients/driver.json`; the registry names only `reader` |
| REQ-011 | `mcp_agent --endpoint <the copy> tabs` | the log: the bridge's refusal, not "not running" |
| REQ-007 | `mcp_agent tools` through Marley's own file; `ss -ltnp` for Marley's pid | the log: every listed tool; `127.0.0.1` only |
| REQ-008 | `mcp_http` initializes five sessions with `reader`'s token, then uses the first | the log: 404 for the first, the fifth working, then `mcp_agent tabs` through Marley's own |
| REQ-009 | python: a header line of 16 KiB; a socket that sends half a request and waits 12 seconds | the log: 431; the socket closed; `mcp_agent tools` after |
| REQ-006 | copy `reader`'s file; `quit_marley`; `launch_marley`; `mcp_http` with the copy; `mcp_agent --endpoint <reader file> tabs` | the log: 403 for the old token; the tabs through the rewritten file |

Not reached by a scenario: D7's refusal of a call that reaches the app after its client's
cut-off. The call would have to wait between the transport and the app across a click, a window
of milliseconds; the check (`run` looks the client up before it acts) is read in the diff's review.

### Risks
- Slice 1 alone leaves the DevTools ports as they are: open to every local process. The ticket
  says so, Chad chose the order, and slice 2 is filed at Complete.
- A same-user process can still read Marley's own endpoint file and every client file (they are
  the user's files); the grants limit a client that plays by the rules and holds only its own
  file, such as a sandboxed agent. The kernel's file modes are the only wall between users.
- A client's endpoint file names its token; a client that copies it elsewhere keeps it for that
  run only (D4).
- `resolve` compares the bearer against every token on each request, and `touch` takes the
  table's write lock for a moment; with a handful of clients that is nothing.
- The modal is Marley's first form modal and first `Checkbox`; the scenario clicks both under
  sway, so their coordinates come from the first run's shots.
- Changing `assign`, `validate`, `AppCall::new` and the discovery helpers' signatures touches the
  crate's old unit tests, which must keep compiling though they no longer run (§7).

## Phase 2 — Code
- **Checklist** (no task tool): `marley_mcp` (clients, sessions, dispatch, the registry's list,
  discovery, the transport, the crate root) ✓; `marley_browser` (the recorder's `by`) ✓;
  `marley_workbench` (clients and the modal, mcp, browser_tools, browser, the crate root, the
  keymap) ✓; the bridge ✓; the fixture ✓; check, fmt, clippy, dylint ✓; the review ✓. The
  Rust was written while the release install's golden set ran, and cargo, the bridge and the
  fixture waited for it: its scenarios run both from the checkout.
- **Built.**
  - `marley_mcp::clients` (new): `Principal`, `ClientGrant`, `ClientTable` (`allow`, `cut_off`,
    `resolve`, `touch`, `is_allowed`, `clients`; its `Debug` names no token), `ClientInfo`,
    `ClientError`, `check_client_name`, the two lists and `permits`.
  - `session.rs`: `SessionEntry.owner`; `assign(id, owner, now)` ends a client's least recently
    seen session at `CLIENT_SESSION_CAP` (4); `validate` and `terminate` match the owner;
    `terminate_owned_by`; `session_decision` and `session_gate` take the owner.
  - `dispatch.rs`: a client's `resources/*` is a JSON-RPC error; `tools/list` goes through
    `registry::tools_list_for(principal)`; `tools/call` runs `permits` after `lookup`.
  - `transport.rs`: `Clients`; `ServerHandle::clients`; `allow_client` and `cut_off_client`, which
    take one lock at a time; a `Server` the connection threads share; `Deadlined`, the reader
    that runs against the 10-second deadline; `read_bounded_line`, `MAX_LINE_BYTES`,
    `MAX_HEADER_LINES`, `RequestError`; the principal after `Origin`; a client's headers
    dropped and its GET answered 405; `serve_post`, split out of `serve_connection`.
  - `discovery.rs`: `write_endpoint_file_in` and `remove_endpoint_file_in`, which Marley's own
    two helpers now call with `mcp-endpoint.json`.
  - `marley_browser::recorder`: `Entry::Agent { did, by }`.
  - `marley_workbench::clients` (new): the registry's `*_in` IO, the client files, the
    `ClientsState` global, `start`, `is_allowed`, `allow`, `cut_off`, and
    `BrowserClientsModal`. `mcp.rs`: `clients::start` once the server runs, and `permits` before
    any app call as the second wall. `browser_tools.rs`: `by` from the principal down to
    `acting`, `check_pick` and `annotate`, and `still_allowed` at the start of `run` and after
    `settled`. `browser.rs`: `AgentAction.client`, `PageState.driven_by`, `DRIVEN_MARK`,
    `driven_by`, `forget_client`, the chip's name and `render_driven_by`. The actions
    `BrowserClients` and `AllowBrowserClient`; `BrowserClients > Editor` binds Enter and Escape.
  - The bridge: `Refused` (a subclass of `Unreachable`, so every old catch holds) on a 403, and
    `REFUSED` in the tool call's error. The fixture: `mcp_agent --endpoint <file>`,
    `mcp_http` and `write_mcp_http`.
- **Deviations.**
  - The server makes its own client table and hands it out through `ServerHandle::clients`,
    where the plan had `spawn` take one: `spawn`'s signature, and so the transport tests' own
    `start`, stay as they were.
  - The late-call check and the toolbar's mark read the main thread's copy of the registry, not
    the server's table: `mcp.rs` keeps the rule that the main thread never waits on the server's
    locks. Cut Off changes that copy first and takes the locks in a background task.
  - At start the registry is read and set on the main thread before any token is minted, so a
    client that connects the moment its file appears finds itself allowed.
  - The modal reads the last calls off the main thread when it opens and every five seconds.
  - `listed_for` went: `tools_list_for` filters through `permits`.
- **Review.**
  - A 431 closed the connection with the rest of the header unread, which on Linux resets it,
    and a reset can drop the reply before the peer reads it. Fixed: `write_status_then_drain`
    shuts the write side and reads up to 64 KiB for a second first. (A 400 for an oversized
    `Content-Length` was never at risk: nothing of the body was sent unread in a way the old
    tests exercise, and it is left as it was.)
  - An `initialize` racing its client's cut-off can leave a session behind; no request can use
    it, since the token is gone, and it idles out in 30 minutes. Accepted.
  - Importing `PoisonError` would have made rustc's `unused_qualifications` flag the file's
    existing `std::sync::PoisonError::into_inner` paths; the new code spells the path out too.
  - `python3 -m py_compile` of the bridge left a `__pycache__` beside it; removed, and the
    bridge is parsed with `ast` from now on.
  - Security read of every path that takes a bearer or a byte before it: the bounded lines and
    the deadline come before any guard; `resolve` runs after the size cap and `Origin`, compares
    every token in constant time and holds only the table's read lock; a client's session ids
    are its own; a client reaches no resource, no stream, and no tool off its list, in the core
    and again in the app; tokens live only in `ClientTable`, whose `Debug` hides them, and in
    the 0600 files.
  - Provenance: Marley's own code; Orca's registry and allowlist read as behavior (report 04),
    no Orca source. No Zed path.
- **Checks.** `cargo check -p marley_mcp -p marley_browser -p marley_workbench --all-targets` ✓
  (after four errors, then the modal's missing `Debug`, fixed by keeping it in the crate);
  `cargo fmt` ✓; `./script/clippy` on the three crates ✓ (after similar names, three first doc
  paragraphs, `serve_connection`'s length and the contexts' mutability); `cargo dylint` ✓, no
  hit; `shellcheck` on the fixture ✓.

## Phase 3 — Test
- **Checklist** (no task tool): REQ-001 to REQ-011 ✓; the golden set ✓; the gate ✓.
- **The scenario.** `script/e2e/524-trusted-outside-browser-access.sh` (`compositor sway`, the
  offline Chromium, a loopback `form.html` with a Name field, Save and a line that shows what
  was saved). It follows the plan's table with three changes: `driver` is allowed first, while no
  client is listed, so the checkbox sits where the empty list puts it (x 902, y 219); `snapshot
  full` reads the page's text, which the default snapshot of interactive elements leaves out;
  and the old token after the restart goes to the new server with the new file's URL (a copy of
  the new file with the old header), since the restarted server listens on a port of its own.
- **Runs.** The first run (`e2e1.log`) failed at "the registry names both, driver to act": the
  guessed checkbox click fell on `reader`'s row, and `driver` was allowed to read only. The
  second (`e2e2.log`) failed at "the page is as it was": the default snapshot has no text. The
  third (`e2e3.log`) passed every check to the restart, where the old file's URL refused the
  connection. The fourth (`e2e4.log`) passed all 33 checks:
  - REQ-001: the folder 700, both files and the registry 600; each file holds `headers`,
    `type` and `url`, its header `Authorization`; the registry `[["driver",true],
    ["reader",false]]` and no token.
  - REQ-002: `reader` lists 10 tools (the ten reading ones), `driver` 18, neither a
    `terminal_`, `fleet_`, `session_`, `draft_test` or `open_url` tool.
  - REQ-003: `browser_click refused: … the client reader may only read them`; the page's full
    snapshot still reads "Nothing saved."
  - REQ-010: `terminal_read` and `fleet_snapshot` refused as "not open to outside clients";
    `resources/read` error -32601; GET 405.
  - REQ-005: the old `driver` token answered 403 after Cut Off; its file gone; the registry
    `["reader"]`.
  - REQ-011: through the bridge with the old file, `browser_tabs refused: Marley refused this
    endpoint file's token: the user cut this client off, or the file is older than the Marley
    that runs…`, and no "not running".
  - REQ-007: Marley's own bridge lists `terminal_read`, `browser_draft_test` and
    `browser_open_url`; the server listens on `127.0.0.1:<port>` alone.
  - REQ-008: five `reader` sessions opened (200 each); the first used again 404, the fifth
    200; Marley's own `tabs` then answers.
  - REQ-009: a 16 KiB header line 431; a connection that sent half a request closed after
    10 s; `reader`'s tools answer after both.
  - REQ-006: after `quit_marley` and `launch_marley`, `reader`'s file is there again,
    `driver` has none, the old token answers 403, and the new file lists the ten tools.
- **Shots** (scratchpad `524/shots`, from the fourth run; only Marley in each):
  - `524-00-empty`: Browser Clients open over the terminal: its title, the line on tokens and
    grants, "No client is allowed.", the name field, "May act in pages" and Allow.
  - `524-01-allowed` (REQ-001): `driver`, "reads and acts · no call yet", and `reader`,
    "reads · no call yet", each with Cut Off; below the form, "reader's endpoint file" with its
    path and Copy, and "Point Marley's bridge at it:" with the command and Copy.
  - `524-02-driven` (REQ-004): the Browser tab on the form, "from the driver" in the field and
    "Saved: from the driver"; the toolbar reads "Driven by driver", Cut Off, and the chip names
    `driver` in place of "Agent", "clicked button “Save”" (a crop of the toolbar confirms it).
  - `524-02b-before-cut-off`: five seconds on, the chip gone and "Driven by driver" with Cut Off
    kept at the toolbar's end (x 1330, y 87).
  - `524-03-cut-off` (REQ-005): after the click, no mark on the toolbar; the page unchanged.
- **Focus.** Headless sway: `hyprland: 0 Marley windows before the run, 0 after; the run added
  no rule and did not reload it`; sway stopped with the run's Marley, pointer and keyboard.
- **The golden set** (33 with 524, `regress/20260927-074817`, the debug build): all 33 passed,
  491, 492, 520 and 574 among them, whose agents reach the server with Marley's own bearer and
  the `Marley-*` headers the transport now drops for clients only.
- **The gate.** `script/gates.sh --diff` (log: scratchpad `524/gate.log`): rustfmt, clippy (every
  target), cargo-audit, cargo-deny, cargo-shear, gitleaks, shellcheck, no-suppressions,
  source-bans, docs, zed-ledger, manifests, spelling, semgrep and dylint all PASS, and the
  receipt; `GATE GREEN [diff]`.
- **Not reached by a scenario:** D7's refusal of a call that reaches the app after its client's
  cut-off, as the plan said; `still_allowed` was read in the review.
- **Pre-existing:** nothing.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented.** `CHANGELOG.md`: Added (programs outside Marley drive its browser, by name) and
  Fixed (the tool server bounds what it reads before it knows who asks).
  `docs/marley_architecture/marley_mcp.md`: the transport's bounds, and an Outside clients
  section. `marley_workbench.md`: an Outside clients section. `marley_browser.md`: the `Agent`
  entry's `by`. `docs/marley/three-prong-plan.md`: D15 corrected (the DevTools ports reach every
  local process, not the user's alone), D17 extended (named outside clients), rows B8a (#524,
  shipped), B8b (#583) and B8c (#584). No path outside the Marley-owned set changed.
- **Knowledge appended:** F-claude-524-the-mcp-servers-read-before-auth-had-no-bound-001,
  F-claude-524-a-431-closed-with-the-request-unread-001,
  PR-claude-bound-every-read-before-auth-in-size-and-time-001,
  L-claude-524-a-restarted-mcp-server-listens-on-a-new-port-001,
  L-claude-524-the-default-snapshot-holds-no-text-001,
  AD-claude-524-outside-clients-reach-a-list-of-browser-tools-by-name-001. The brain:
  consultation 5d5fc98defb44023b84e0ce7ed4e95cb closed with
  `decisions/outside-programs-reach-marleys-browser-by-name-through-the-mcp-server-marley-524`,
  follow-up by 2026-10-27.
- **Filed:** TICKET-583 (slice 2, Chromium's DevTools off TCP) and TICKET-584 (slice 3, other
  machines), at the top of the Queue.
- **Closed:** `tickets/closed/TICKET-524-trusted-outside-browser-access.md`; its backlog row went
  at the promotion.
