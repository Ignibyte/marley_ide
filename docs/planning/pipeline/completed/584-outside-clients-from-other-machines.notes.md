# Outside clients from other machines, through Marley's bridge over SSH — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-584-outside-clients-from-other-machines.md
- **Pipeline spec:** 584-outside-clients-from-other-machines.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** TICKET-584, slice 3 of #524: #524's clients from another machine, the ticket's idea
  being a fixed loopback port reached through `ssh -L` or `tailscale serve`, with the client's
  token and Cut Off as on the machine itself. Autonomous. The release install of #583 ran while
  this was planned, so no cargo ran.
- **Classification / tier:** feature, prong 3, B8c; size S to M (a Python change in the bridge, a
  line in the modal, a reorder in Cut Off, a scenario with an sshd of its own).
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo
  busy with the install, so the toolchain went unchecked this time, unchanged since #583); recall
  ✓; mint ✓ (pipeline id 415ababd-46cf-4860-bec8-0b1115ceeb4a; the backlog row removed; the ticket
  in-progress); prior art ✓; spec ✓; design ✓.
- **Recall (§18.3):**
  - AD-claude-524-outside-clients-reach-a-list-of-browser-tools-by-name-001: a client's token is
    minted at each start and written only to its own 0600 endpoint file, which the client points
    Marley's bridge at; tokens that last across restarts (Orca's direct tokens) were rejected.
    That decides this ticket: a remote HTTP client with its token in its config would be refused
    after the next start.
  - L-claude-524-a-restarted-mcp-server-listens-on-a-new-port-001: the port changes at each
    start too; a scenario that shows a stale token refused sends it to the new port.
  - F-claude-561-an-opener-on-the-default-endpoint-would-open-tabs-in-another-marley-001 and its
    rule: a program finds the Marley that wrote it. The SSH line names the client's own file, so
    the right Marley answers, a scenario's profile included.
  - F-claude-524-the-mcp-servers-read-before-auth-had-no-bound-001: the server's pre-auth read is
    bounded; nothing here changes what reaches it.
  - The brain (consultation 2518fe36d9244fa0aebf7ca23566654d): nothing on outside MCP clients from
    other machines. A stray consultation opened by `rusty-cli brain ask --help` was closed with
    no decision.
- **Discovery** (Explore; the facts this design rests on):
  - `marley_mcp::transport::spawn` binds `127.0.0.1:0` (transport.rs:173) and mints Marley's
    bearer and an empty client table at each call (175-177); `serve_connection` checks Origin,
    then the bearer, then the session (251-252); there is no Host check (`read_http_request`
    reads Origin, Authorization, Mcp-Session-Id, the `Marley-*` headers and Content-Length,
    646-660). Origin passes when absent or loopback (auth.rs:40-42).
  - A client's endpoint file is `<data>/mcp/clients/<name>.json` in a 0700 folder
    (clients.rs:106-128), `{"type":"http","url":…,"headers":{"Authorization":"Bearer …"}}`
    (transport.rs:743-746). The registry `<data>/mcp/clients.json` holds names, grants and times,
    no token (clients.rs:33-59). Quit clears the client files; the next start writes them again for
    each listed client with a new token (clients.rs:177-237).
  - The modal shows, right after Allow only, "{name}'s endpoint file" and "Point Marley's bridge at
    it:" with `MARLEY_MCP_ENDPOINT=<file> <data>/mcp/marley-mcp-bridge`, each copyable
    (`render_allowed`, clients.rs:536-583, quoted with `ShellKind::Posix.try_quote`).
  - `clients::cut_off` (clients.rs:315-343): the main thread's registry and the tab's marks first;
    then off the main thread `cut_off_client`, the file's removal, and only then the registry's
    rewrite.
  - The bridge (`claude_plugin/marley/bin/marley-mcp-bridge`, Python, written to
    `<data>/mcp/marley-mcp-bridge` at each start by `mcp.rs:248`) reads `$MARLEY_MCP_ENDPOINT`,
    re-reads the file before each call (`forward`: stale when the file differs) and reconnects
    with the client's own `initialize`; while no Marley answers it answers `initialize` itself,
    lists no tools and says "Marley is not running…" on a call; it polls every 2 s and sends
    `notifications/tools/list_changed` when Marley comes or goes; on a 403 it says the token was
    refused; the bearer goes only to a loopback `http` URL; stdout carries JSON-RPC alone.
  - **A bug on this seam.** Cut Off deletes the client's file, so a bridge pointed at the real file
    finds none and says "Marley is not running… Start Marley and call again", while Marley runs.
    #524's REQ-011 ("say that the token was refused, not that Marley is not running") was proved
    with a copy of the old file (`driver-old.json`, 524's scenario:146-160), a file no client
    holds. After the next start the cut-off client's file is still gone, and the message is the
    same. Fixed here (D3); an F- block at Complete.
  - Zed's `remote` crate forwards TCP ports over SSH from Zed's machine outward
    (`SshPortForwardOption`, settings_content.rs:1436-1443; ssh.rs:381-403, 1803-1821), numeric
    ports only; Marley needs none of it.
  - The e2e profile is `$XDG_RUNTIME_DIR/marley-e2e/profile.XXXXXX` (40 bytes); scenarios read the
    clipboard under their sway with `WAYLAND_DISPLAY=$SWAY_DISPLAY wl-paste --no-newline` (579,
    503); `mcp_agent --endpoint <file>` runs the real bridge (browser-fixture.sh:142-152).
  - An unprivileged sshd of a scenario's own runs on this box (a probe: OpenSSH 10.5p1,
    `UsePAM no`, its own host key and authorized_keys, key auth on 127.0.0.1; `ssh -T … true`
    wrote nothing to stdout, so the account's shell start-up prints nothing to a remote command).
- **Decisions:** D1 to D4 in the spec. The ticket's fixed port is dropped: the bridge already
  reads each start's token on the machine where Marley runs, so SSH's remote command is the
  whole transport. The ticket's summary gets a line saying so.

### Design
- **Approach.**
  - *The SSH line* (`clients.rs`, `render_allowed`). Beside `command` it builds
    `remote = format!("env MARLEY_MCP_ENDPOINT={} {}", quote(file), quote(bridge))` and
    `ssh = format!("ssh -T -o BatchMode=yes {user}@{host} {}", quote(&remote))`, with `user` from
    `whoami::username()` and `host` from `whoami::fallible::hostname()`. A host that cannot be read
    shows as `<this machine>`, which the user replaces. A third label, "From another machine, run
    it over SSH:", and a third `copyable` row (`browser-client-copy-ssh`) sit under the local
    command. The names are read when the panel renders the allowed client, a few syscalls on the
    main thread, as the path strings are.
  - *Cut Off's order* (`clients.rs`, `cut_off`). The background closure writes the registry, then
    removes the file. A failed registry write keeps the file, whose token is already refused, so
    the bridge says the token was refused; the error still reaches the modal as today.
  - *The bridge* (Marley-owned Python, embedded by `include_str!`). A `NotAllowed(Unreachable)`
    error and a `NOT_ALLOWED` text: "Marley does not allow this client: the user cut it off in
    Browser Clients, or never allowed it. Ask the user to allow it again there." A function
    `not_allowed()` says whether the named endpoint file is a client's (its folder is `clients`),
    is absent, and the registry beside the folder (`../clients.json`) reads and does not list its
    stem. `connect` raises `NotAllowed` when `read_endpoint()` finds nothing and `not_allowed()`
    holds, else `Unreachable` as now. `local_answer` takes the reason's text instead of a
    `refused` flag: `REFUSED` for `Refused`, `NOT_ALLOWED` for `NotAllowed`, `NOT_RUNNING` for the
    rest. Its error for other methods reads "… needs Marley, which does not allow this client"
    for `NotAllowed`. `watch` logs "Marley no longer allows this client" instead of "Marley quit"
    when the file went and `not_allowed()` holds. The docstring says the bridge also serves
    outside clients, on this machine or over SSH from another. The bridge never reads Marley's own
    endpoint file for this; the registry holds no token.
  - Nothing in `marley_mcp` changes; no Zed path is touched, so no ledger row.
- **File manifest.**
  - `crates/marley_workbench/src/clients.rs` (Marley crate): `render_allowed`'s SSH line;
    `cut_off`'s order.
  - `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge` (Marley crate, Python):
    `NotAllowed`, `NOT_ALLOWED`, `not_allowed()`, `connect`, `local_answer`, `handle`, `watch`, the
    docstring.
  - `crates/marley_workbench/Cargo.toml` (Marley crate): `whoami` (the workspace's entry if it has
    one, else `1.6.1`, the version in `Cargo.lock`).
  - `script/e2e/584-outside-clients-from-other-machines.sh` (new) and `script/e2e/golden` (+584)
    at Test.
  - Docs at Complete: `CHANGELOG.md`, `docs/marley/guide.md` (Browser Clients: from another
    machine), `docs/marley_architecture/marley_workbench.md` (the panel's third line, Cut Off's
    order, the bridge's answer), `docs/marley/three-prong-plan.md` (row B8c as shipped).

### E2E plan
`compositor sway`. Setup: `offline_chromium`; a site with `index.html` ("Over SSH") and
`next.html`; `git init` a repository and `open_path` it. The sshd of the scenario's own: `ssh-keygen`
a host key and a client key into a 0700 `$E2E_WORK/ssh`, `authorized_keys` from the client key,
an `sshd_config` (`ListenAddress 127.0.0.1`, a free port, `HostKey`, `AuthorizedKeysFile`,
`PidFile`, `UsePAM no`, `StrictModes no`, password and keyboard login off, TCP and stream-local
forwarding off, no X11 or agent forwarding), started as `/usr/bin/sshd -D -e -f …` and stopped in
`teardown`. The stand-in remote client `remote-client.py` reads the copied line with `shlex`,
checks its shape, and runs `ssh -F /dev/null -p <port> -i <key> -o IdentitiesOnly=yes -o
IdentityAgent=none -o UserKnownHostsFile=<file> -o StrictHostKeyChecking=accept-new -o
BatchMode=yes -T 127.0.0.1 <the copied remote command>`: the copied line's own command, only the
destination and the options changed. It speaks MCP over the session's pipes (initialize,
initialized, then the step's calls), prints each result on one line, prints any stdout line that
is not JSON-RPC as `not JSON-RPC: …`, and with `hold` keeps the session open between steps (a
FIFO for its next calls).

| REQ | Fixtures and steps | Proof |
|---|---|---|
| REQ-001 | Palette "marley: browser clients"; type `laptop`, click "May act in pages", Return. Click the SSH line's Copy button; read the clipboard. | Shot `584-01-allowed`: the endpoint file, the local command and the SSH line. The log: the clipboard parsed into `ssh -T -o BatchMode=yes`, `$(id -un)@$(hostname)`, and `env MARLEY_MCP_ENDPOINT=<laptop's file> <profile>/mcp/marley-mcp-bridge`; `grep -c` of laptop's token in the clipboard is 0. |
| REQ-002 | Open the browser on `index.html` (Ctrl+L). `remote-client.py tools`, then `tabs`, then `navigate next.html`. | The log: "18 tools", the tab's title, the navigation's result. Shot `584-02-driven`: the tab on `next.html`, its chip naming `laptop`, "Driven by laptop" with Cut Off. |
| REQ-003 | Every `remote-client.py` run. | The log holds no `not JSON-RPC:` line (an `expect` on each run's output). |
| REQ-004 | `remote-client.py hold` opens a session and lists tools; `quit_marley`; the held session calls `tabs` (Marley down); `launch_marley`, settle; the held session calls `tabs` again and quits. | The log: the call while down says "Marley is not running"; the tools-changed notices; the call after the start names the page; the stand-in never restarted its SSH session. |
| REQ-005 | Cut Off in the tab's toolbar (sway click). `remote-client.py tabs` over SSH; `mcp_agent --endpoint <laptop's real file path> tabs` locally. Then `quit_marley`, `launch_marley`, and `remote-client.py tabs` once more. | Shot `584-03-cut-off`. The log: each answer says "does not allow this client" and none says "not running"; laptop's file is gone and the registry lacks it. |
| REQ-006 | While a held session is open (REQ-004's second half): `ss -ltnpH` for Marley's pid. | The log: Marley listens only on `127.0.0.1:<its MCP port>`; the sshd is the scenario's and is listed apart. |
| REQ-007 | — | Review of `clients::cut_off` in Code; REQ-005's run is its observable half. |

Beyond reach: another physical machine and a real network path. The scenario's sshd on
127.0.0.1 runs the same remote command through the account's shell, as a login from elsewhere
does. A real `tailscale` or LAN path adds only the network, which SSH carries and Marley never
sees. A password prompt is also out of reach, since `BatchMode` is there to fail instead. Then
524 runs again (its bridge and modal), and the golden set with 584 added.

### Risks
- The host name shown may not be the one the other machine uses (an SSH alias, a tailnet name):
  the user edits it; the guide says so. Reversible: a later ticket can let the user name it.
- The bridge now reads `clients.json`'s shape (`clients[].name`). Both are Marley's own files,
  written by the same build; the scenario exercises the read.
- `python3` must be on the machine's `PATH` for a non-interactive SSH login, as it must be for
  the bridge today; `/usr/bin/python3` is.
- An SSH that would prompt fails at once under `BatchMode`; the MCP client reports a failed
  server. The guide says to connect once by hand and use a key or an agent.
- The brain decision at Complete: remote MCP clients run Marley's bridge over SSH, not a fixed
  port, because tokens are minted per start.

## Phase 2 — Code
- **Checklist** (no task tool): the bridge ✓; `clients.rs` ✓; `Cargo.toml` ✓; check, fmt and
  clippy ✓; the review ✓. No Zed path, so no ledger row.
- **Built.**
  - `marley-mcp-bridge`: `NOT_ALLOWED`; `not_allowed()`, true when the named endpoint file sits in
    a `clients` folder, is gone, and `../clients.json` reads and does not list the file's stem;
    `NotAllowed(Unreachable)`. `Unreachable`, `Refused` and `NotAllowed` carry, as class
    attributes, the text a tool call gets (`why`) and how an unknown method's error ends
    (`state`); `local_answer(message, reason)` takes the error. `connect` raises `NotAllowed` when
    the file is missing and `not_allowed()` holds. `watch` logs "Marley no longer allows this
    client" in place of "Marley quit" then. The docstring names outside clients, here or over SSH.
  - `clients.rs`: `render_allowed` builds `over_ssh`, `ssh -T -o BatchMode=yes <user>@<host>
    '<env + the local command>'`, and shows it as a third copyable row ("From another machine,
    run it over SSH:", `browser-client-copy-ssh`). `this_machine()` reads the names through
    `whoami::fallible`, a placeholder word standing in for a name that cannot be read. `cut_off`
    writes the registry before it removes the file; its doc says so.
  - `Cargo.toml`: `whoami = { version = "1.6.1", default-features = false }`.
- **Deviations from the plan.**
  - `default-features = false` on `whoami`: its default `web` feature put `web-sys` (wasm only)
    into the lock's `whoami` entry. rust-analyzer in the editor rewrote `Cargo.lock` the moment the
    manifest changed (`cargo metadata`, no build), which showed it. Now the lock only gains
    `whoami` in marley_workbench's list.
  - `local_answer` takes the error rather than a text. The error carries both the call's text and
    the unknown method's phrase, so a refused token's `-32601` now ends "which refused this
    endpoint file's token" instead of "which is not running", the same wrong-state class as the
    bug this ticket fixes.
- **Checks.** `cargo check -p marley_workbench` clean; `cargo fmt -p marley_workbench` changed
  nothing; `just clippy marley_workbench` (all targets, `-D warnings`) clean. The bridge parses
  (`ast`). A smoke of the edited bridge in the scratchpad, with no Marley running: a client file
  that is missing, beside a registry that lists only `reader`, got "Marley does not allow this
  client…" on `tools/call` and "resources/read needs Marley, which does not allow this client".
  `reader`'s missing file still got "Marley is not running…".
- **Review of the diff.**
  - REQ-001: the SSH line quotes each word for the remote shell, then the whole command once more
    for the local one, and holds only paths and names, no token. REQ-005: a Cut Off client, a
    typo'd name and a client cut off before a restart all read as not allowed. A listed client
    whose file is gone (Marley quit or starting) reads as not running. Marley's own endpoint file
    never passes the `clients` folder test. REQ-007: the registry is written first; a failed
    registry write returns before the file goes, leaving a file whose token is already refused, so
    the bridge says the token was refused, and the error reaches the modal as before.
  - Re-entrancy: nothing new touches an entity; `this_machine()` is two syscalls in render, beside
    the path strings.
  - Provenance: no Warp material, no Zed function body carried over; `whoami` is a dependency
    and `ShellKind::try_quote` is called, not copied.
  - Found nothing else to fix.

## Phase 3 — Test
- **Checklist** (no task tool): REQ-001 ✓; REQ-002 ✓; REQ-003 ✓; REQ-004 ✓; REQ-005 ✓; REQ-006 ✓;
  REQ-007 by review ✓ (Phase 2); 524 again ✓; the golden set with 584 (below); the gate (below).
- **The scenario:** `script/e2e/584-outside-clients-from-other-machines.sh`, `compositor sway`.
  Its setup starts an sshd of its own on 127.0.0.1 (its own host key, one authorized key, `UsePAM
  no`), and `over-ssh`, which runs the command copied from the modal through that sshd with the
  scenario's own options (`-F /dev/null`, `IdentityAgent=none`, its own known-hosts file,
  `BatchMode=yes`). `remote` runs the fixture's stand-in agent with `over-ssh` as its bridge;
  `hold-client.py` holds one session open across a restart and reports any stdout line that is
  not JSON-RPC.
- **Runs.** `just build` (debug) first. Run 1 stopped at the Copy click: the first guess missed,
  and the 584-01 shot put the SSH line's Copy at (1056, 428). Run 2 passed REQ-001 to REQ-004 and
  REQ-006 and failed REQ-005's first check. The click at #524's toolbar point (1330, 87) landed on
  the toolbar's last-action text, which pushes "Driven by laptop · Cut Off" left by an amount that
  changes with the URL (L-claude-584-a-tabs-toolbar-moves-with-the-last-actions-text-001). The
  scenario now cuts laptop off with its row in Browser Clients, (1048, 198). Run 3: 20 of 20
  checks pass. The focus report: "hyprland: 0 Marley windows before the run, 0 after; the run
  added no rule and did not reload it".
- **What the log shows** (run 3): the copied line parsed into `ssh -T -o BatchMode=yes`, this
  machine's `user@host` (`id -un`, `uname -n`), and `env MARLEY_MCP_ENDPOINT=<laptop's file>
  <profile>/mcp/marley-mcp-bridge`, with no token in it. Over SSH: 18 tools, none that evaluates
  script; the tab "Over SSH"; `browser_navigate` to next.html, and the tab then titled "The next
  page". A raw session over SSH: 3 lines, 0 not JSON-RPC, answers to [1, 2, 3]. Marley's process
  listened on `127.0.0.1:<its MCP port>` alone. The held session listed 18 tools and read "The
  next page". After the quit it got a tools-changed notice and "Marley is not running…". After
  the start it got a second notice and read "The next page" again, on the same SSH session. After
  Cut Off, over SSH and through a local bridge on laptop's own file: "browser_tabs refused: Marley
  does not allow this client: the user cut it off in Browser Clients, or never allowed it…",
  never "not running". laptop's file was gone and the registry was `[]`. After a restart, the
  same answer.
- **The shots** (run 3, read):
  - `584-01-allowed`: Browser Clients with laptop, "reads and acts · no call yet" and Cut Off;
    under the form, "laptop's endpoint file", "Point Marley's bridge at it:", and "From another
    machine, run it over SSH:" with `ssh -T -o BatchMode=yes <user>@<host> 'env MARLEY…`, each
    with a Copy button (REQ-001).
  - `584-02-driven`: the Browser tab on next.html ("Where the client over SSH went"); the toolbar
    reads "Driven by laptop", Cut Off, a `laptop` chip and "went to …/next.html"; the rail's row is
    "The next page" (REQ-002).
  - `584-03-cut-off`: Browser Clients after Cut Off: "No client is allowed." and the empty form;
    behind it the toolbar has no mark (REQ-005).
  - `584-04-mark-gone`: the tab on index.html, the toolbar with no "Driven by" and no chip
    (REQ-005).
  - Each shows only Marley's window in the scenario's sway. They name this machine's user and
    host, so they stay in the scratchpad.
- **524 again:** 33 of 33 checks pass. Its stale-copy check still reads "Marley refused this
  endpoint file's token…", which is the answer for a file that is there with a dead token. Its
  `524-01-allowed` shows reader's panel with the SSH line under the local one; `524-03-cut-off`
  shows the tab without its mark.
- **The golden set** with 584 added (`script/e2e/golden`, 35 entries), on the debug build:
  "regress: all 35 passed"; 584 took 109 s.
- **The gate.** The first `script/gates.sh --diff` was red on gate:10 alone (15 passed). Its
  history scan found a `generic-api-key` match in #583's pushed commit 05f8f014ca:
  `script/e2e/583-chromium-devtools-off-tcp.sh:59`, the ws-probe's `Sec-WebSocket-Key:
  dGhlIHNhbXBsZSBub25jZQ==`, RFC 6455's sample nonce (§1.3), public and no secret. #583's gate
  passed because the working-tree scan never covered `script/e2e`, and the history scan reached
  the line only once it was committed, after the push to the public origin
  (F-claude-584-gitleaks-saw-a-scenario-only-after-it-was-pushed-001). A pushed commit cannot
  change, so there were two fixes:
  - `.gitleaks.toml`'s allowlist names that one value, beside Warp's public Firebase key, the one
    public value it allowed before, with the reason in its description;
  - gate:10's working-tree scan now covers `script/e2e.sh`, `script/e2e`, `script/regress` and
    `script/install-marley`.
  A redacted scan of those found no other match; the commit and `script/e2e` then scan clean. The
  second run: 16 passed, 0 failed, "GATE GREEN [diff]", which wrote the receipt. Both logs are in
  the scratchpad (`584/gate.log`, `584/gate2.log`).
- **Pre-existing:** none in scope.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented.** `CHANGELOG.md`: Added (browser clients on other machines, over SSH), Fixed (a
  cut-off client is told so; the secrets gate reads the e2e scenarios before they are
  committed). `docs/marley_architecture/marley_workbench.md`: the panel's SSH line and
  `this_machine`, Cut Off's order, the bridge's `NotAllowed` and its error classes' texts.
  `docs/marley/guide.md`: a paragraph after "How agents reach it"'s table on running a client
  from another machine. `docs/marley/three-prong-plan.md`: D17, and row B8c shipped (size S). No
  Zed path was touched, so `docs/marley/zed-touchpoints.md` needs no row.
- **Knowledge appended:** F-claude-584-a-cut-off-clients-bridge-said-marley-was-not-running-001,
  F-claude-584-gitleaks-saw-a-scenario-only-after-it-was-pushed-001,
  PR-claude-prove-what-a-program-says-on-the-file-it-reads-001,
  PR-claude-every-marley-folder-a-commit-adds-to-is-in-gate-10s-tree-scan-001,
  L-claude-584-a-scenario-runs-an-sshd-of-its-own-001,
  L-claude-584-a-tabs-toolbar-moves-with-the-last-actions-text-001,
  AD-claude-584-other-machines-run-marleys-bridge-over-ssh-001. The brain: consultation
  2518fe36d9244fa0aebf7ca23566654d closed with
  `decisions/marleys-clients-on-other-machines-run-its-bridge-over-ssh-marley-584`, follow-up by
  2026-10-27.
- **Closed:** `tickets/closed/TICKET-584-outside-clients-from-other-machines.md`; its backlog row
  went at the promotion.
