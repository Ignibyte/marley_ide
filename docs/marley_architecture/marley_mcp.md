# `marley_mcp`

Marley's MCP server: a hand-written Streamable HTTP server on loopback, a pure core that routes
each JSON-RPC message, and a `std::net` transport around it. Ported from the gpui era (#370 to
#379; `orchestration-shell.md` keeps that history); since #491 Marley starts it at startup. MIT
OR Apache-2.0, with the Marley crates' lint table.

## The core (`dispatch.rs`, `registry.rs`, `tools.rs`)

- `handle_message(ctx, subscriptions, message)` turns one message into what goes back:
  `initialize` (the protocol revision `2025-06-18`, `tools.listChanged`, subscribable
  resources, and since #680 `instructions`, the `INSTRUCTIONS` text under 2 KB on when to reach
  for which tool), `tools/list`, `tools/call`, and the fleet resource's `resources/*`. An unknown
  method or tool is a protocol error; a denied write or a failed call is a tool result with
  `isError`.
- **Docs, settings and actions (#681).** `Family::Docs` (`docs_search`, `docs_read`),
  `Family::Settings` (`settings_schema`, `settings_read`) and `Family::Actions` (`actions_list`),
  served and read-only, with no grant class; the app answers them like the terminal family
  (`dispatch.rs`'s deferred arm). Their schemas are `docs_schemas`, `settings_schemas` and
  `actions_list_schemas`; outside clients reach none of them (they are on no client list).
  `INSTRUCTIONS` names the five. Since #682 `settings_change` joins `Family::Settings` as a write
  tool, grant class `settings.write` (granted at start, as `terminal.write` is), with
  `settings_change_schemas`; its answer is the app's, which asks the user first.
- **Refusals with a code (#680).** `Refusal { code, reason, next_steps }` is how a call is refused:
  `tools::tool_refusal` answers `{result: "refused", code, reason, next_steps}`. `From<String>`
  and `From<&str>` give `Refusal::REFUSED`, so the app's handlers that refuse in words alone stay
  as they were. `AppCall::answer` takes a `Result<ToolAnswer, E: Into<Refusal>>` and
  `AppOutcome::Answered` carries a `Refusal`, which `deferred_response` answers with the tool's
  name in front of the reason. `dispatch.rs`'s own refusals carry `not_granted` (a client's
  grant), `tool_off` (a conditional tool turned off), `not_permitted` (a write class not
  granted), `timed_out`, `unavailable` and `bad_argument` (the surface tool's arguments), each
  with a next step.
- The registry is the one table of tools: a family and a verb per row, with its tier, its grant
  class and its description. `tool_name` joins family and verb with `_` (`terminal_blocks`),
  since Claude Code and the Anthropic API take tool names without dots (#491). `tools/list` lists
  the served families, `terminal`, `browser` (#492) and `ports` (#521), while `fleet` and
  `session` wait for prong 2's C1. A client that names one of their tools still reaches it: the
  fleet is empty, and a session write is denied without a grant.
- **Driving a running program (#525).** `terminal_screen` (read) and `terminal_type` (write,
  grant class `terminal.write`), with their schemas in `terminal_screen_schemas` and
  `terminal_type_schemas`; the terminal family's calls defer to the app as before. Outside
  clients reach neither: they are on no client list.
- **Running at the prompt (#556).** `terminal_run` (write, grant class `terminal.write`), with
  its schemas in `terminal_run_schemas`: a `command` of one line up to 4,096 bytes and
  `wait_seconds` up to 20 in; the block's index, exit code, duration, output, `running`,
  `truncated` and `redacted` out, and since #680 the output's newest page with `first_line`,
  `last_line`, `total_lines`, `previous` and `line_cut`, as `terminal_read` gives them (it takes
  `before`). The app answers it, as it answers `terminal_type`.
- **Conditional tools (#567).** `CONDITIONAL_TOOLS` names the tools listed and called only while
  the user turns them on: `browser_find` and `terminal_find`, each the System One use of its
  name. `RequestCtx.enabled` holds the ones on, from `ServerData.enabled`, which the app sets
  through `transport::set_enabled` off its main thread. `tools_list_for(principal, enabled)`
  leaves an off tool out, and `tools_call` refuses one by name after `permits`, naming the
  setting. The server sends no `notifications/tools/list_changed` when the set changes: a client
  sees it at its next `tools/list`.
- **`find` (#567)** is the find tools' match by words, pure: `words` (lowercased, split at
  everything but letters and digits, stop words left out), `matches` (an item holding every word
  as a whole word), `local` (`Sure`, `Several` or `Nothing`) and `windows(count, size)`. The
  tools' schemas are `find_schemas` and `terminal_find_schemas`, over `query_schema` and
  `found_properties`: `query` (200 characters), and back `source`, `sure`, the `ref` or `line`,
  `candidates`, `present`, `verify`, `next` and `note`, and `cut` for a block. Neither tool is on
  the outside clients' lists.
- A call whose answer is the app's comes back as `Outgoing::Deferred(PendingCall)`; every
  terminal tool's does. `deferred_response(pending, outcome)` builds its response from the app's
  `ToolAnswer` (a structured result, and text of its own when it has some, such as a block's
  output), or a tool error that names the tool: the app's reason, no answer within
  `APP_CALL_TIMEOUT_SECONDS` (30), or an app that takes no calls.

- The `browser` family (#492) is served too: eleven read tools (`browser_tabs`, since #493,
  `browser_look`, `browser_snapshot`, `browser_console`, `browser_network`, since #496
  `browser_picks` and `browser_pick`, since #498 `browser_annotations`, since #499
  `browser_recordings` and `browser_recording`, and since #506 `browser_draft_test`, which takes
  a recording's `id` and answers a Playwright test and where it goes, through
  `draft_test_schemas`) and nine write
  tools (`browser_navigate`, `browser_back`, `browser_click`, `browser_type`, `browser_press`,
  `browser_scroll`, since #498 `browser_annotate`, since #561 `browser_open_url`, which takes
  a `url` and a program's `directory` and answers `opened` with the project, or a `reason`,
  through `open_url_schemas` (since #586 the `url` may be a local HTML page, a `file:` URL or a
  path), and since #505 `browser_check_pick`, which takes a pick's `id` and
  answers whether and how its element was found again and what changed, through
  `check_pick_schemas`) whose grant class,
  `browser.write`, Marley grants when it starts the server. Since #493 every browser tool that
  acts in a page takes `tab`, a page's id from `browser_tabs` (`browser_arguments` adds it to
  each schema), `browser_navigate` takes `new_tab`, and every answer from a page names its tab.
  Since #574 the `tab` argument's and the tools' descriptions say that a call naming none acts in
  the project the agent runs in, and `browser_tabs`' output adds `project` and `default`. Since
  #507 `browser_tabs`' description says it lists every project's browser, each project with a
  Chromium and a profile of its own, and `browser_navigate`'s that a new tab opens in the
  caller's project's browser, with that project's cookies and logins.
  Since #501 Zed's own agents reach the server too, through the workbench's context server
  `marley`, which runs the same bridge over stdio.
  `browser_pick` takes a pick's `id`; its output schema spells out the bundle (`pick_schemas`,
  built on `bundle_schema`), each listener with its `original` place through the script's source
  map since #497, and every line and column counted from 1. Since #518 the bundle adds the
  element's HTML, styles, sibling texts, the selection and the React component
  (`element_context_properties`), and since #505 the pick adds its latest `check`
  (`check_schema`).
  A `ToolAnswer` can carry an image (`ToolImage`), which reaches the client as an MCP image block
  after the text.
- The `ports` family (#521) is served with one read tool, `ports_list`: no arguments, and an
  output of one `ports` array, each entry's `project`, `folder`, `address`, `port`, `url`,
  `pid`, `name` and `cwd` (`ports_list_schemas`). `dispatch` defers it to the app, as it does the
  terminal and browser families. It is not in `CLIENT_READ_TOOLS`, so outside clients (#524)
  neither see nor call it.
- The `editor` family (#649) is not served: `tools/list` leaves it out, and its grant class,
  `editor.write`, is on no outside client's list, so `clients::permits` refuses it to them.
  Marley's own `marley-edit` calls it with Marley's bearer and the `Marley-Terminal` header.
  `editor_open` (Write) takes an absolute `path` and answers `edit` (an id) and `file`;
  `editor_wait` (Read) takes `edit` and `wait_seconds` (1 to 20) and answers `closed`
  (`editor_schemas`). `dispatch` defers both to the app.

## Redaction (`redact.rs`, #516, #562)

- `Redactor::new(patterns)` builds the redactor from the user's regular expressions and returns
  the errors of those that did not compile, which it leaves out. `redact(text)` answers a
  `Redacted { text, count }`: each secret replaced by `[redacted: <kind>]`, and how many.
- `marker(kind)` is that stand-in, public since #565: the System One layer hides its own key
  with it, since no built-in rule names `MARLEY_SYSTEM_ONE_KEY`.
- The built-in rules (`BUILT_IN`, compiled once) run in order:
  - private key blocks (PEM and PGP; one with no END line yet is hidden to the end of the text);
  - cookie headers (the label kept and the whole value hidden, up to a quote that closes the
    argument the header sits in);
  - values assigned to secret-named variables, the name kept (hyphenated names such as
    `x-api-key` count);
  - `Bearer` values (the word kept);
  - authorization credentials (the label and the scheme kept, a Digest parameter list hidden
    whole);
  - a URL's password, or a lone token as its userinfo (the scheme, the user and the `@` kept);
  - then AWS key ids, GitHub, Slack, Stripe, Google and `sk-` keys, and JWTs.
- A value an earlier rule already turned into a marker is left alone, so a token assigned to
  `GITHUB_TOKEN=` counts once. The order is load-bearing:
  - the cookie rule comes before `secret`, so a cookie named `token=` is hidden once, as a cookie;
  - `authorization` is not a `secret` name, since that rule's value would stop at the space after
    `Basic`;
  - the `authorization` rule follows `bearer token` and leaves its marker.
- The user's patterns run last, as `[redacted: pattern]`.
- #562 compared the rules with Orca's redactor (`src/main/observability/redactor.ts`, MIT, read
  at `1c2cf120e3`) rule by rule:

  | Orca's rule | Marley's answer |
  |---|---|
  | Labelled pairs (`api[-_]?key`, `token`, `secret`, `password`, `bearer`, `authorization`), the label hidden too | The `secret` rule keeps the name; since #562 it takes the hyphen forms and `BEARER` and `PRIVKEY`. `authorization` has its own rule, which also hides a two-word credential (`Basic …`) that Orca's `\S+` value cuts at the space |
  | Anthropic, OpenAI, GitHub, AWS key id and Slack shapes | The same shapes, some wider (`api key` takes both `sk-` forms, `github_pat_`, `ASIA`); the kinds stay coarse, `api key` rather than one per vendor |
  | `aws_secret_access_key = <40 characters>` | The `secret` rule (the name holds `SECRET` and `ACCESS_KEY`) |
  | JWT | The same, with the payload starting `eyJ` too |
  | Any PEM block | Private-key blocks only: certificates and public keys are not secrets, and an agent reading a chain needs them |
  | URL userinfo, the whole of it | A password after the user, or a lone token of 20 characters or more; a bare user name (`https://admin@host/`) stays |
  | Every `.env` line | Left out (#516): `env` output would lose `PATH` and every harmless value |
  | The attribute blocklist (`cookie`, `set-cookie`, `proxy-authorization`, …) | The `cookie` rule and `proxy-authorization` in the `authorization` rule, for text; the structured keys (`env`, `install_id`) have no text form |
  | Idempotent passes, `[redacted:<tag>]` | The marker skip in `apply`, `[redacted: <kind>]` |
- The module is pure: the workbench calls it on what its tools answer (`marley_workbench.md`,
  the MCP server), and the terminal's own buffer never changes.

## The transport (`transport.rs`, `session.rs`, `auth.rs`, `discovery.rs`, `secret.rs`)

- `spawn(shared, effects, caller)` binds `127.0.0.1:0`, mints a 32-hex bearer from
  `/dev/urandom`, and runs an accept thread for the life of the process; each connection gets a
  thread and one request. Before dispatch (#524 bounds the first three): the request line and
  each header line at 8 KiB and at most 100 header lines (431, the rest drained for a second so
  the peer reads the reply), 10 seconds to send the whole request (`Deadlined`), a 1 MiB body
  cap, then `Origin` (none, or loopback), then the bearer or a client's token, then the session
  (`Mcp-Session-Id`, minted at `initialize`; at most 32, never evicted, expired after 30 idle
  minutes, ended by `DELETE`).
- `handle_message` runs under the shared state's lock, and the answers go out after it is
  released. A deferred call goes to the app through `caller` (an `AppCaller`, which must return
  at once) as an `AppCall`, and the connection's thread waits on its answer channel.
- `discovery_json(url, bearer)` is the endpoint as an MCP client's server entry (`type`, `url`,
  and an `Authorization` header); `write_discovery_file_in(dir, json)` writes it with mode 0600,
  set before the bearer's bytes land, and `remove_discovery_file_in(dir)` removes it. The bearer
  is never logged, and `ServerHandle`'s `Debug` hides it.

## The caller (#520)

- `read_http_request` keeps three more headers the Claude Code plugin's bridge sends:
  `Marley-Terminal` (kept only with a UUID's shape), `Marley-Project` and `Marley-Cwd`
  (percent-decoded, absolute, at most 4,096 bytes). They make a `Caller`, which `ask_app` hands
  the app on each `AppCall` (`AppCall::caller`). A malformed value names nothing and never refuses
  a call; the caller is a default for what a call that names no terminal acts on, never an
  authority, since the bearer gates every call.
- Since #571 a session keeps the name its client gave at `initialize` (`client_name_of`,
  `clientInfo.name`, printable ASCII, at most `MAX_CLIENT_NAME` characters): the session gate
  (`gate`) names it on the new session (`SessionRegistry::name_client`), and `serve_post` reads it
  under the same lock for each request (`client_of`) into `Caller::client`, a courtesy that sorts
  callers, such as Zed's agent, which names itself `Zed`.

## The agent socket (`agent_socket.rs`, #652, Unix only)

- `spawn(path, handler)` removes a socket a Marley left at `path`, binds a `UnixListener`, sets
  the file to 0600 and accepts on a thread of its own, each connection on another. A connection's
  peer credentials (`rustix`'s `socket_peercred`) must name the user Marley runs as; then one line
  of at most 64 KiB is read within 2 s and handed to `handler` as an `AgentRequest` with the
  peer's pid, on the connection's thread. The app answers through `AgentRequest::answer`; after
  3.5 s with no answer the peer reads `marley_not_running`.
- What the line means is the app's (`marley_workbench::agent_reports`); this module knows no
  report. `AgentSocket` removes its file when dropped.

## Claude Code's IDE server (`ide.rs`, `ide_transport.rs`, #653)

- `ide` is pure. `lock_json(pid, folders, token)` is the lock file Claude Code reads (`pid`,
  `workspaceFolders`, `ideName` `Marley`, `transport` `ws`, `authToken`, `runningInWindows`);
  `is_stale_marley_lock(json, alive)` says whether a lock file names Marley and a process gone.
  `Upgrade` collects an upgrade request's headers and `admit(upgrade, token, port)` takes it or
  refuses it: not a WebSocket upgrade (400), another host or a page's `Origin` (403), the token
  in `X-Claude-Code-Ide-Authorization` missing or wrong, compared with `auth::ct_eq` (401). The
  client's version comes from its `User-Agent` (`version_of_user_agent`) or `initialize`'s
  `clientInfo`.
- `step(message, parts)` answers `initialize` (the client's protocol version echoed, `Marley` as
  the server), `tools/list`, `prompts/list` (empty), `ping`, and turns `ide_connected` into the
  client's pid and a `tools/call` into a `Tool` for the app. `Parts { selection, mention }` is what
  a client may be sent: without `selection`, `getCurrentSelection`, `getLatestSelection` and
  `getOpenEditors` are neither listed nor called. Any other tool is a tool error naming it.
  `selection_changed`, `at_mentioned` and the five answers (`selection_answer`,
  `open_editors_answer`, `folders_answer`, `diagnostics_answer`, each a text item of JSON) build
  the messages, places in LSP lines and UTF-16 characters.
- `ide_transport::spawn_ide(judge, handler)` binds `127.0.0.1:0`, mints the token with
  `mint_secret`, and serves each client on a thread of its own, at most `MAX_CONNECTIONS` (8;
  the ninth gets 503). The upgrade is read with the HTTP transport's `Deadlined` reader,
  `read_bounded_line` and its limits, so nothing before the token grows without bound; the 101
  carries `derive_accept_key` and `mcp` when asked, and `tungstenite`'s `from_partially_read`
  takes the bytes read past the headers, frames and messages capped at 1 MiB. A client's loop
  reads in 50 ms slices and between them writes what the app queued (`IdeServer::notify`), the
  replies of the calls the app answered (`IdeCall::answer`, 30 s at most), and a ping every 30 s;
  a client that misses two pongs is dropped. The `Judge` decides each client's `Parts` from its
  version on the client's thread. Dropping `IdeServer` closes every client and wakes the accept
  loop so it ends.

## Outside clients (`clients.rs`, #524)

- `Principal` is who holds a request's bearer: `Marley` for the per-boot bearer, or
  `Client(ClientGrant { name, write })` for a program the user allowed. It sits beside
  `Caller`, which says where a call comes from and grants nothing. `ClientTable` keeps each
  client's grant, token and last call; its `Debug` names no token. `resolve(presented,
  marley_bearer)` compares with `ct_eq` against the bearer and every token, with no early exit.
- `permits(principal, tool)`: Marley calls every tool; a client only `CLIENT_READ_TOOLS`
  (`browser_tabs`, `look`, `snapshot`, `console`, `network`, `picks`, `pick`, `annotations`,
  `recordings`, `recording`) and, with `write`, `CLIENT_WRITE_TOOLS` (`navigate`, `back`,
  `click`, `type`, `press`, `scroll`, `annotate`, `check_pick`). The lists are explicit, so a tool
  added later reaches no client until it is named there. `check_client_name` takes 1 to 32
  letters, digits, `-` and `_`, and refuses `agent` and `marley`, which the Browser tab's chip
  would show as Marley's own.
- The transport keeps the table apart from the data lock (`Clients`, an `RwLock`), so resolving
  a bearer never touches the session registry; `ServerHandle::clients` hands it to the app, and
  `allow_client` (mints a token) and `cut_off_client` (takes the client out, then ends its
  sessions, one lock at a time) change it. For a client, the `Marley-*` headers are dropped, a
  GET gets 405, `resources/*` a JSON-RPC error, `tools/list` names its grant's tools
  (`tools_list_for`), and `tools/call` runs `permits` after `lookup`, so an unlisted tool is
  refused too.
- Sessions record their owner: another principal's session id is unknown (404); a client holds
  at most `CLIENT_SESSION_CAP` (4), and its fifth `initialize` ends its own least recently used
  one, since a client opens no standing stream whose hang-up would reap it; `terminate_owned_by`
  ends a cut-off client's.
- `write_endpoint_file_in(dir, name, json)` and `remove_endpoint_file_in(dir, name)` write and
  remove an endpoint file at 0600; the discovery helpers call them with `mcp-endpoint.json`.

## In the app (#491)

`marley_workbench::mcp` starts the server from `zed`'s `main`, writes `mcp-endpoint.json` into
Marley's data directory, removes it on quit, and answers the terminal family on the main thread;
the Claude Code plugin's `bin/marley-mcp-bridge` carries the server to Claude Code over stdio.
`marley_workbench.md` has both.
