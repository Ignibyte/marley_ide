# `marley_mcp`

Marley's MCP server: a hand-written Streamable HTTP server on loopback, a pure core that routes
each JSON-RPC message, and a `std::net` transport around it. Ported from the gpui era (#370 to
#379; `orchestration-shell.md` keeps that history); since #491 Marley starts it at startup. MIT
OR Apache-2.0, with the Marley crates' lint table.

## The core (`dispatch.rs`, `registry.rs`, `tools.rs`)

- `handle_message(ctx, subscriptions, message)` turns one message into what goes back:
  `initialize` (the protocol revision `2025-06-18`, `tools.listChanged`, subscribable
  resources), `tools/list`, `tools/call`, and the fleet resource's `resources/*`. An unknown
  method or tool is a protocol error; a denied write or a failed call is a tool result with
  `isError`.
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
  `truncated` and `redacted` out. The app answers it, as it answers `terminal_type`.
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
