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
  the served families: `terminal` today, while `fleet` and `session` wait for prong 2's C1. A
  client that names one of their tools still reaches it: the fleet is empty, and a session write
  is denied without a grant.
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
  through `open_url_schemas`, and since #505 `browser_check_pick`, which takes a pick's `id` and
  answers whether and how its element was found again and what changed, through
  `check_pick_schemas`) whose grant class,
  `browser.write`, Marley grants when it starts the server. Since #493 every browser tool that
  acts in a page takes `tab`, a page's id from `browser_tabs` (`browser_arguments` adds it to
  each schema), `browser_navigate` takes `new_tab`, and every answer from a page names its tab.
  Since #574 the `tab` argument's and the tools' descriptions say that a call naming none acts in
  the project the agent runs in, and `browser_tabs`' output adds `project` and `default`.
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

## Redaction (`redact.rs`, #516, #562)

- `Redactor::new(patterns)` builds the redactor from the user's regular expressions and returns
  the errors of those that did not compile, which it leaves out. `redact(text)` answers a
  `Redacted { text, count }`: each secret replaced by `[redacted: <kind>]`, and how many.
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
  thread and one request. Before dispatch: a 1 MiB body cap, then `Origin` (none, or loopback),
  then the bearer, then the session (`Mcp-Session-Id`, minted at `initialize`; at most 32, never
  evicted, expired after 30 idle minutes, ended by `DELETE`).
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

## In the app (#491)

`marley_workbench::mcp` starts the server from `zed`'s `main`, writes `mcp-endpoint.json` into
Marley's data directory, removes it on quit, and answers the terminal family on the main thread;
the Claude Code plugin's `bin/marley-mcp-bridge` carries the server to Claude Code over stdio.
`marley_workbench.md` has both.
