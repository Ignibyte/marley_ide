# Trusted outside clients, paired with a code — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-811-trusted-outside-clients.md
- **Pipeline spec:** 811-trusted-outside-clients.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "we need a way to have a trusted device that can perform write
  actions as well. So registering a mcp server to marley needs to be trusted not just open and
  then given permissions (read, write, execute etc)". 2026-10-18, asked which of a pairing prompt
  and a list in settings: "which do you think is best?"; the answer recommended pairing; Chad:
  "lets make it happen".
- **Classification / tier:** feature; Marley's MCP server and its clients.
- **Recall (§18.3):**
  - AD-claude-524: outside clients by name, browser tools only, tokens per start in a 0600
    endpoint file each, the list in `<data>/mcp/clients.json`, never in Zed's settings.
  - AD-claude-584: another machine runs the bridge here over SSH; SSH proves who connects.
  - AD-claude-755: a terminal proves its id with `MARLEY_TERMINAL_TOKEN`; same-user processes
    reading another's environment are out of scope.
  - The map (Explore, this session): the shared bearer in `mcp-endpoint.json` resolves to
    `Principal::Marley` with every tool, so any program that reads the file (the plugin's bridge in
    any terminal, `marley-open-url`, a scenario's stand-in) writes and runs. That is the hole the
    ticket names.
  - Rusty's brain (consultation `de291379e919490181546cf82dcf37e8`): nothing on this seam.
- **Discovery:** `crates/marley_mcp/src/{clients.rs,transport.rs,dispatch.rs,registry.rs,
  marley_mcp.rs}`, `crates/marley_workbench/src/{mcp.rs,clients.rs,marley_workbench.rs}`, the
  bridge (`claude_plugin/marley/bin/marley-mcp-bridge`), `bin/marley-open-url`,
  `script/e2e/browser-fixture.sh`.

### Design
- **`marley_mcp::clients`**: `Power { Read, Write, Run }` (ordered); `ClientGrant` gains
  `power: Option<Power>` (none: a Browser Clients entry, browser lists as before);
  `Principal::Unpaired`; `power_needed(spec)` from the tier and grant class (`RUN_CLASSES`:
  `terminal.write`, `harness.write`, `agents.write`; `RUN_TOOLS`: `thread_answer`);
  `permits(principal, tool) -> Result<(), Refusal>`: `not_paired` for an unpaired program's
  non-read tool, `not_granted` for a client's; `PAIRING_TOOLS` callable by anyone.
  `resolve(presented, bearers, proven_terminal)`: the own bearer → Marley; the shared bearer with a
  proven terminal → Marley, without → Unpaired; a client token → its client. Every token compared
  in constant time.
- **`transport`**: the terminal proof moves before the principal; `spawn` mints the own bearer
  (`ServerHandle::own_bearer`); an unpaired program, like a client, gets no GET stream;
  `allow_client(…, power)`, `set_client_power`.
- **`dispatch`/`registry`**: `Family::Client` (`client_pair`, `client_pair_status`), Read tier,
  not served (unlisted), answered by the app; resources refused to an unpaired program as to a
  client; the refusal from `permits` carries its own code.
- **Workbench `mcp.rs`**: writes `mcp/own-endpoint.json` with the own bearer and points Zed's
  context server at it; removes it at quit; `answer` routes `client_*` to `clients::answer`.
- **Workbench `clients.rs`**: `RegisteredClient.power` (serde default none); `Pairings` in
  `ClientsState`; `client_pair` (name checks, one pending per name, three at most, a six-digit code
  and a 16-hex id from `rand`, the app-wide notification for `PAIR_WAIT` = 120 s);
  `client_pair_status` (waiting, trusted with the endpoint, the bridge and the SSH line, refused,
  expired; a final state is given once); `pair_client` and `set_power`; Outside Clients: the
  headline, a paired row's Read/Write/Run buttons, the browser rows as before.
- **`marley_workbench.rs`**: the action `OutsideClients`, registered beside `BrowserClients`.
- **Bridge**: `marley-mcp-bridge pair NAME`; its refusal texts name Outside Clients.
- **`marley-open-url`**: the `Marley-Terminal` headers.
- **e2e**: `browser-fixture.sh`'s `mcp_agent` defaults to the own endpoint (the stand-in plays
  Marley's own agent); `491-marley-mcp.sh` likewise.
- No Zed path changes; no spawn site.

### Visual check plan
| REQ | The scenario | Shot / check |
|---|---|---|
| REQ-001 | `mcp_agent --endpoint <shared>`: `tool terminal_list` answers; `refusal terminal_type` gives `not_paired`. | run checks |
| REQ-002 | `marley-mcp-bridge pair laptop` in the background; its output holds the code. | `811-01-prompt` |
| REQ-003 | A click on Read and Write; the command's output says trusted with the endpoint line; `marley: outside clients`. | `811-02-clients` |
| REQ-004, REQ-005 | As `laptop`: `refusal terminal_type …` gives `not_granted`; Run in the list; `terminal-type python …` types, and the bar names `laptop`. | run checks, `811-03-run` |
| REQ-006 | Cut Off in the list; `laptop`'s `terminal_type` is refused. | run check |
| REQ-007 | The stand-in on the own endpoint types into the REPL. | run check |
- Not reached: a second machine (#584's scenario covers the SSH line); Zed's agents through the
  context server (the own endpoint stands in; the review checks the context server's file).

### Risks
- Programs outside Marley's terminals that wrote through the shared bearer now get `not_paired`:
  Claude Code in tmux or another terminal app, harness seats, a hand-run bridge. The refusal says
  how to pair; the changelog and guide say so.
- `codex_app_server`: a Codex App Server Marley launches outside a terminal would be unpaired; it is
  off by default, and noted.

## Phase 2 — Code
- **Built:** as the design. `marley_mcp`: `Power` (serde, lowercase), `power_needed`,
  `RUN_CLASSES`, `RUN_TOOLS`, `PAIRING_TOOLS`, `Principal::Unpaired`, `permits` returning a
  `Refusal` with its own code, `ClientTable::resolve(presented, shared, own, proven_terminal)` and
  `set_power`; the transport proves the terminal before it resolves the principal, mints the own
  bearer (`ServerHandle::own_bearer`) and refuses a GET to anyone but Marley; `allow_client(…,
  power)`, `set_client_power`; `Family::Client` with `client_pair` and `client_pair_status`
  (Read tier, unlisted). The workbench: `own-endpoint.json` for Zed's context server, removed at
  quit; `route` hands `client_*` to `clients::answer`; `clients.rs` pairs (`begin_pairing`,
  `pairing_status`, the app-wide notification with four buttons), keeps `power` in
  `clients.json`, and Outside Clients shows a paired row's power with Read, Write and Run; the
  `OutsideClients` action. The bridge's `pair NAME`; `marley-open-url`'s terminal headers; the
  e2e stand-in on the own endpoint.
- **Deviations:** the pairing tools are routed before `answer`, not inside it: `answer` and `start`
  were each one line under clippy's 100. A tool `tools/list` does not list (fleet, session,
  prompt) is Marley's own: an unpaired program and a paired client are refused it whatever its
  tier, so `fleet_snapshot` stays as closed to them as AD-524 kept it from clients.
- **Review of the diff:**
  - The terminal proof ran after the principal; it now runs first, since it decides whether the
    shared bearer is Marley's.
  - `marley-open-url` sent no terminal headers, so a program's `$BROWSER` in a Marley terminal
    would have become unpaired and fallen back to `xdg-open`.
  - Scenario 491's harness-side calls only read, so it stays as it is; the fixture's stand-in,
    which writes in many scenarios, moves to the own endpoint.
  - Clippy: a long first doc paragraph; `change_power` takes no `self`.
- **Gate:** `script/gates.sh --diff` green on the tested tree (every gate PASS, the receipt
  written), run after the visual check.

## Phase 3 — Test
- **Scenario:** `script/e2e/811-trusted-outside-clients.sh` (`compositor sway`). Run 1 found the
  notification's buttons, run 2 the list's; run 2 also showed the list's text printing backticks
  around the command, fixed (plain words) and rebuilt. Run 3 passed every check and every shot
  was read:
  - checks: an unpaired `terminal_list` answers; its `terminal_type` is refused `not_paired` with
    the pair command in its words (REQ-001); the pair command prints a code (`820 412`) and, after
    Read and Write, `Paired: laptop may read and write.` with `clients/laptop.json` in its line
    (REQ-002, REQ-003); `laptop`'s `terminal_type` is refused `not_granted`, "needs the power to
    run; the client laptop may read and write" (REQ-004); after Run, `laptop` types 13 bytes
    (REQ-005); after Cut Off, its call answers "Marley does not allow this client" (REQ-006); the
    stand-in on the own endpoint still types (REQ-007).
  - `811-01-prompt` (REQ-002): bottom right, "A program asks to pair with Marley as "laptop"",
    "Check that it shows the code 820 412", "Asked from /srv/stacks/marley_ide", and Read, Read and
    Write, Read Write and Run, Refuse.
  - `811-02-clients` (REQ-003): Outside Clients, its two lines of text, `laptop` "may read and
    write · last call just now" with Write pressed, Read, Run and Cut Off.
  - `811-03-run` (REQ-004, REQ-005): `>>> print(6 * 7)`, `42`, and the bar "laptop typed into
    python3: "print(6 * 7)" <enter>" with Take Over and its close button.
  - `811-04-cut-off` (REQ-006): Outside Clients with "No client is allowed."
- Not reached: Zed's agents through the context server (the own endpoint is the same file and
  bearer the stand-in used); a second machine (#584's scenario).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: "A program pairs before it acts"); `docs/marley/guide.md`
  ("How agents reach it" with what each may do, "Pairing a program (#811)", the bridge's `pair`);
  `docs/marley/walkthrough.md` stop 7.19, 7.15's command and the header;
  `docs/marley_architecture/marley_mcp.md` (principals, powers, `permits`, the own bearer) and
  `marley_workbench.md` (the own endpoint, `route`, pairing, Outside Clients). No Zed path.
- **Knowledge:** AD-claude-811-programs-outside-marley-pair-before-they-act-001,
  F-claude-811-marley-open-url-proved-no-terminal-001,
  F-claude-811-a-label-printed-markdown-backticks-001.
- **Brain:** `brain decide` on consultation `de291379e919490181546cf82dcf37e8`:
  `decisions/programs-outside-marley-pair-with-a-code-before-they-act`, follow-up by 2026-11-01.
- **Regression to watch:** the golden scenarios that write through the fixture's stand-in now use
  the own endpoint; any scenario that writes from outside a terminal with the shared file would
  now be refused `not_paired`. #806 holds the golden set's repair.
