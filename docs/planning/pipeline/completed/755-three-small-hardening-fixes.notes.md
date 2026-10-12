# Three small hardening fixes — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-755-three-small-hardening-fixes.md
- **Pipeline spec:** 755-three-small-hardening-fixes.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** three items from `docs/t3code_architecture/README.md` ("Facts about Marley the
  survey turned up" and "Smaller things"), approved by Chad on 2026-10-11 with "8.) yes to all".
  The fourth item he approved with them, the rail cleanup (#713), is specced on its own.
- **Classification:** chore. Marley crates, plus one existing Marley hunk in Zed's `terminal`
  crate (part B).
- **Recall (§18.3):**
  - #552 (completed): the agent bar's one-click setup for Codex and OpenCode, and its scenario's
    set-up (`CODEX_HOME`, `XDG_CONFIG_HOME`, stand-in agents, a private session bus).
  - #520 (completed) accepted spoofing: "A shell that sets its own `MARLEY_TERMINAL_ID` can pose
    as another terminal. The id is a convenience for scoping, not authority: the MCP bearer still
    gates every call" (its notes 170-172, 240-242; the architecture decision at
    `architecture-decisions.md:2214-2228`; `identity.rs:6-7`). #704 then keyed the editors area's
    approvals on the id (`agent_control.rs:128-141`), so the premise no longer holds.
  - #575: a restored terminal keeps its id through `MARLEY_RESTORED_TERMINAL_ID`.
  - #583: the relay is the unit's main process; Chromium is its child on a CDP pipe.
  - L-claude-725: a feature that writes outside Marley's data must be pointed at the run's
    folders in a scenario (`CODEX_HOME`, `XDG_CONFIG_HOME`).
  - An existing decision notes a race that "an atomic write in `install_in` would close"
    (`architecture-decisions.md:3658`); that file stays out of scope here.
- **Discovery (Explore, 2026-10-11):**
  - **A.**
    - `agent_notify.rs`: `codex_config` 66-70, `configure_codex` 99-119 (`toml_edit`),
      `set_up` 248-286 (off the main thread, the toast or `show_error`), `write_codex_in`
      292-303 (read, edit, `create_dir_all`, plain `std::fs::write`), `write_plugin_in` 310-318
      (plain write, then 0644).
    - Other writes into other programs' files:
      - Codex's skills copy (`rustal_skill.rs:188-209`) and Claude Code's IDE lock
        (`claude_ide.rs:388-401`) are staged and renamed already.
      - Zed's `tasks.json` (`workflows.rs:461-488`) is a plain write: left out.
      - Marley never writes `~/.claude.json`, Claude's `settings.json` or Codex's `mcp_servers`.
    - `tempfile` is a dev-dependency of `marley_workbench` only (`Cargo.toml:109`); `uuid` is a
      dependency.
  - **B.**
    - The bearer: minted per boot (`transport.rs:189`), written to `mcp-endpoint.json` at 0600,
      checked in `serve_connection` (270-276) through `ClientTable::resolve` (`clients.rs:216-226`).
    - Caller headers: parsed in `read_http_request` (710-724); `caller_terminal` (585-596) checks
      the UUID shape only; an outside client's caller is reset (282-283).
    - The id is set in `crates/terminal/src/terminal.rs:1326-1359` (Marley's hunk), and emptied
      for tasks and remote terminals. The context server's environment blanks it at
      `mcp.rs:291-308`.
    - Senders: the bridge's `caller_headers` (167-184) and `bin/marley-edit`'s `request`
      (98-109). `marley-agent` uses the agent socket, already proven by `SO_PEERCRED`.
    - Users of the caller's terminal:
      - `mcp::caller_terminal` (862-871) and `terminal_of` (874-891);
      - `terminal_drive` (217, 329, 741);
      - `agent_control`'s session key (128-141);
      - `click_pause::Who::of` (71-105);
      - `browser_tools::caller_scope` (139-160);
      - `agent_editor::open` (171).
  - **C.** `service.rs:340-357` (`systemd_run_args`: `KillMode=mixed`, `TimeoutStopSec=10`, no
    `OOMPolicy`); the unit `marley-browser-<12 hex>` (249-252); the only caller
    `browser.rs:3396-3400`. A renderer crash closes its tab (`browser.rs:3574-3580` →
    `page_gone` 1428-1457); Chromium's death fails the hub with "The browser closed its
    connection." (3584-3590). systemd 261 on the box.

### Design
- **C — `crates/marley_browser/src/service.rs`:** `"--property=OOMPolicy=continue"` after
  `TimeoutStopSec`, and a sentence in the function's doc: systemd's default stops the unit when
  the kernel kills any of its processes for memory, which would close every tab of the project.
- **A — new `crates/marley_workbench/src/safe_write.rs`** (Marley crate):
  - `pub(crate) fn replace_in(path: &Path, backup: bool, edit: impl FnOnce(&str) -> anyhow::Result<String>) -> anyhow::Result<Replaced>`,
    `Replaced::{Written, Unchanged}`, synchronous, run on the background executor as `set_up`
    does now.
  - The steps of the spec's scope, with `std::fs::canonicalize` for a path that exists (its
    parent made with `create_dir_all` when it does not), `std::os::unix::fs::MetadataExt` for the
    identity, `OpenOptions::create_new` and `PermissionsExt::from_mode` for the temp, `sync_all`
    on the temp and on the folder (opened read-only), and `std::fs::rename`.
  - A temp left by a failure is removed on every error path (`inspect_err` and `log_err`, as
    `rustal_skill` does).
  - `agent_notify.rs`: `write_codex_in` becomes `safe_write::replace_in(path, true, configure_codex)`;
    `write_plugin_in` becomes `replace_in(path, false, |_| Ok(OPENCODE_PLUGIN.to_string()))` with
    0644 for a new file.
- **B:**
  - `crates/marley_terminal/src/identity.rs`: `TOKEN_VARIABLE`; `terminal_token(id) -> String`
    (the map's token, minted on first ask); `token_matches(id, token) -> bool` (constant time);
    the module doc's "it is no authority" sentence replaced.
  - `crates/terminal/src/terminal.rs` (Zed crate, the existing `// Marley:` hunk at 1341-1352):
    `env.insert(TOKEN_VARIABLE, id.map(terminal_token).unwrap_or_default())` beside the id. The
    row in `docs/marley/zed-touchpoints.md` for #520's hunk is extended before the edit.
  - `crates/marley_workbench/src/mcp.rs`: the context server's environment blanks
    `TOKEN_VARIABLE`; the server is started with `Arc::new(identity::token_matches)` as its
    terminal check.
  - `crates/marley_mcp/src/transport.rs`: `HttpRequest.terminal_token`; the header arm in
    `read_http_request`; `spawn` and `Server` take `terminal_check: Arc<dyn Fn(&str, &str) -> bool + Send + Sync>`;
    in `serve_connection`, for `Principal::Marley`, a terminal whose token fails is set to
    `None`, with one `log::info!` naming the id.
  - `crates/marley_workbench/claude_plugin/marley/bin/marley-mcp-bridge` (`caller_headers`) and
    `crates/marley_workbench/bin/marley-edit` (`request`): `Marley-Terminal-Token` from
    `MARLEY_TERMINAL_TOKEN`, sent only with `Marley-Terminal`. The plugin's version goes up so
    Claude Code takes the new bridge (`claude_plugin.rs`'s update path).
  - Docs: the guide, `docs/marley_architecture/marley_mcp.md` and `terminal_blocks.md` (where
    #520's identity is written up), an architecture decision (`AD-claude-755-…`) superseding
    #520's on spoofing.
- **File manifest:**
  - Marley crates: `marley_browser` (`service.rs`); `marley_workbench` (`safe_write.rs` new,
    `agent_notify.rs`, `marley_workbench.rs` module line, `mcp.rs`, the bridge, `bin/marley-edit`,
    the plugin's manifest); `marley_terminal` (`identity.rs`); `marley_mcp` (`transport.rs`).
  - Zed path: `crates/terminal/src/terminal.rs`, its touchpoint row extended.
  - Docs as above; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001-003 | `config.toml` a symlink to `dotfiles/codex.toml` (0600, a comment and a `[tui]` key); clicks the Codex chip; runs a check script in the terminal | 755-01-codex-written |
| 003 | Clicks Connect OpenCode to Marley; lists the plugins folder | 755-02-plugin-written |
| 004 | — | The review of the diff |
| 005, 006 | Splits the terminal; the left runs the client: `terminal_list`, its `self` row, and `env` shows the token set | 755-03-honest |
| 007 | The right sets `MARLEY_TERMINAL_ID` to the left's id and runs the client: no `self`, `terminal_read` refused | 755-04-spoofed |
| 008 | Opens a Browser tab on the fixture site; prints `systemctl --user show -p OOMPolicy` of the unit | 755-05-oom-policy |
| 009 | Opens `hog.html` (allocates 32 MiB every 100 ms once a button is pressed) in a second tab; sets `MemoryMax` to `memory.current` plus 64 MiB with `systemctl --user set-property --runtime`; presses the button through `mcp_agent`; waits for the OOM line in the unit's journal | 755-06-after-oom |
| 010 | — | The gate |

### Risks
- **Two renderers in one process:** if Chromium puts both fixture tabs in one renderer, the kill
  closes both. The two pages come from two `serve_site` origins, opened as separate targets, so
  they get separate processes; the shot shows whether they did.
- **The kernel picks the victim:** renderers carry `oom_score_adj` 300, the browser process and
  the relay 0, so the hungry renderer goes first. If the memory controller is not delegated to the
  user's manager, `set-property` fails; the scenario then records that and REQ-009 rests on
  REQ-008's shot plus the review.
- **An old bridge in Claude Code's plugin cache** sends no token until the plugin updates; its
  calls lose only their terminal (D5).
- **The token map grows** by one entry per terminal for the run: a few dozen strings.

### The split (proposed)
Part B is larger than A and C together: six places, one of them a Zed crate hunk, two programs in
other languages, a plugin version bump, and a decision that replaces #520's. It can leave as its
own ticket with no change to A or C: its scope (B), D4 and D5, REQ-005 to REQ-007, shots 755-03
and 755-04, and the B rows of the design above move as they are. A and C then make a small chore
of their own. Until a number is minted for B, the three stay together in the order C, A, B (D7).

### Promotion (2026-10-11)
- Moved from `queued/` to `active/`; the BACKLOG row removed; the ticket in progress. Brain:
  consultation b274d5c1a0f34a259853cb8c3a8a0744, nothing on this seam. The three parts stay one
  ticket (D7's order, C, A, B).
- **Seams checked again** at `48685b89f8` (an Explore pass over all eleven):
  - C: `service.rs` `systemd_run_args` (334-357, `KillMode=mixed`, `TimeoutStopSec=10`), called
    only from `service::start` (374), which `browser.rs:3396-3400` calls; `unit_name` (246-252).
  - A: `agent_notify.rs` `configure_codex(text: &str) -> anyhow::Result<String>` (99),
    `set_up` (248, the write in `background_spawn(futures::future::lazy(..))`, `show_toast` or
    `show_error`), `write_codex_in` (292-303) and `write_plugin_in` (310-318), both a plain
    `std::fs::write`; `OPENCODE_PLUGIN` (28). `uuid` is a dependency (`Cargo.toml:90`). No shared
    safe-write helper; `clients.rs`, `claude_ide.rs`, `rustal_skill.rs` and `shared_plugin.rs`
    stage and rename on their own.
  - B: `identity.rs` `TERMINAL_ID_VARIABLE` (16), `new_terminal_id` (29), `is_terminal_id` (35),
    the doc's "it is no authority" (6-7), the `RwLock` statics with setters (43-145);
    `marley_terminal` has `uuid` with `v4`, no `subtle` anywhere. Zed's `terminal.rs` hunk
    (1344-1379) sets the id where `task.is_none() && !is_remote_terminal`, else empties it.
    `mcp.rs` `offer_to_zeds_agents` (274, env 286-310 blanks the id and the project) and
    `start` (85, `transport::spawn(shared, effects, caller)` at 124). `transport.rs` `spawn`
    (182), `Server` (214-221), `serve_connection` (244-288, an outside client's caller reset),
    `HttpRequest` (571-580), the header arms (716-721), `caller_terminal` (586-596, a shape
    check); `auth::ct_eq` (53). The bridge's `caller_headers` (167-184) and `marley-edit`'s
    `request` (98-110, only `Marley-Terminal`). The plugin is at 1.6.0 (`plugin.json:4`,
    `marketplace.json:11`); `claude_plugin::needs_update` updates a lower installed version.
    The touchpoint row for `terminal.rs` is line 85.
  - The scenarios: #552's (`CODEX_HOME`, `XDG_CONFIG_HOME`, stand-ins, the chip at 460,954) and
    the browser fixture (`offline_chromium`, `serve_site`, `mcp_agent`, `browser_unit_of`,
    `palette "marley: open browser"`).
- **What differs from the draft, and where the design takes it:** nothing of substance.
  `configure_codex` already returns a `Result`, which `replace_in`'s edit takes as is;
  `identity::token_matches` compares in constant time itself (no `subtle`; `marley_terminal`
  cannot reach `marley_mcp`'s `ct_eq`); the transport's terminal check is handed in as
  `Arc<dyn Fn(&str, &str) -> bool + Send + Sync>` at `spawn`.

## Phase 2 — Code (2026-10-11)
- **Built, in D7's order:**
  - C: `systemd_run_args` adds `--property=OOMPolicy=continue`, its doc saying why.
  - A: `safe_write.rs` (new): `replace_in(path, backup, edit) -> Replaced`, the spec's seven
    steps; `Seen` holds the bytes and the identity, `real_path` follows a link (a dangling one
    names the file to create), `write_temp` sets the mode again after the umask, `write_backup`
    writes beside the path as named. `write_codex_in` and `write_plugin_in` call it.
  - B: `identity::TOKEN_VARIABLE`, `terminal_token`, `token_matches`; Zed's builder sets the
    token beside the id (`as_deref().map(terminal_token).unwrap_or_default()`, so empty wherever
    the id is); `offer_to_zeds_agents` blanks it; `transport::TerminalCheck` handed to `spawn`,
    `HttpRequest::terminal_token` from `Marley-Terminal-Token`, and the check in
    `serve_connection` after the client block. The bridge and `marley-edit` send the token with
    the id, only a 32-hex value. The plugin goes 1.6.0 → 1.7.0 (`plugin.json`,
    `marketplace.json`, the guide's version line).
- **Deviations from the design:** the temp is removed after the write's closure on any error
  (one place) rather than with `inspect_err` at each step. An existing `marley.js` keeps its own
  mode; only a new one gets 0644 (the spec's step 4; the old code forced 0644 every time).
- **Clippy found and fixed:** `significant_drop_tightening` in `token_matches` (the expected token
  is copied out of the lock first); `doc_markdown` and an over-long first doc paragraph in
  `safe_write.rs`.
- **The review:**
  - REQ-004 rests on it: the re-read compares bytes and identity, the `bail!` comes before the
    rename, the temp is removed on every error, and `set_up` shows the error through
    `show_error` as before. The backup is written before the re-read (the spec's order), so a
    refused write still leaves the backup holding the bytes Marley read: harmless, and noted.
  - REQ-005's empty cases: a task, a remote terminal and a local terminal of a remote project
    get no id, so the token is empty too; Zed's agents' bridge gets it blank.
  - An old bridge in Claude Code's plugin cache sends no token: its calls run as from no
    terminal, logged at info once per call. The bridge's liveness check is a TCP connect, not a
    request, so it logs nothing.
  - No entity re-entrancy (no GPUI code changed beyond the server's start); nothing from Warp;
    the Zed hunk is additive, its row in `zed-touchpoints.md` (line 85) extended first.
- **The gate:** `script/gates.sh --diff` green, 17 of 17. After it, three edits that change no
  behavior: the identity module's doc reflowed to 100 columns, a comment on the bridge's
  `TOKEN`, and the browser fixture's `serve_site` poll ending with `|| true` (L-claude-755-…);
  the gate ran again, green, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/755-three-small-hardening-fixes.sh`, under `compositor sway`,
  exit 0, every check passing. Shots read:
  - `755-01-codex-written`: the toast "Codex notifications are on"; the listing shows
    `config.toml -> …/dotfiles/codex.toml` still a link, `config.toml.marley-backup` at 600, the
    target at 600, 3 notification keys, 0 temp files, and the backup's first lines holding the
    old comment and model (REQ-001 to REQ-003).
  - `755-02-plugin-written`: the plugins folder holds `marley.js` alone, `-rw-r--r--`, no backup,
    no temp (REQ-003).
  - `755-03-honest`: two terminals; the right prints `1` (a 32-hex `MARLEY_TERMINAL_TOKEN`), and
    its client's own row is `self`, `self rows: 1` (REQ-005, REQ-006).
  - `755-04-spoofed`: the same terminal with the left's `MARLEY_TERMINAL_ID` and its own token:
    the same two rows, none `self`, `self rows: 0` (REQ-007).
  - `755-05a-calm`: the Browser tab on the calm page; the log prints `OOMPolicy=continue` and
    `ActiveState=active` (REQ-008).
  - `755-05b-hungry`: a second tab, "hungry page", beside the calm one, the unit capped at its
    use plus 400 MiB with swap off.
  - `755-06-after-oom`: the hungry tab gone (Marley closes a crashed page's tab), the calm tab
    still drawing; the log prints `ActiveState=active`, `NRestarts=0` and the journal's "The
    kernel OOM killer killed some processes in this unit." (REQ-009).
- **Fixes to the scenario on the way, none in the product:**
  - `serve_site` called outside `$(…)` ended the run silently under `set -e`: moved into the
    substitution, then the fixture itself made safe (L-claude-755-serve-site-…).
  - The first spoof check borrowed a listed id, which is the view's entity id, so it passed with
    nothing tested; it now borrows the left terminal's real `$MARLEY_TERMINAL_ID`
    (L-claude-755-terminal-lists-id-…).
  - The first OOM attempt swapped instead of killing (7.7 GiB eaten): `MemorySwapMax=0` and a
    2 GiB ceiling in the page (L-claude-755-a-renderer-killed-…).
  - `755-05b-hungry` added after reading `755-06`, which alone could not show a second tab had
    been open.
- **Departures from the plan's rows:** the OOMPolicy shows in the run's log, not a terminal
  (`755-05a`); the hog eats on a timer, not a button pressed through `mcp_agent`; and
  `terminal_read` refused with no terminal was not exercised (REQ-007 is the `self` rows). The
  spec's UI proof names the shots as run.
- **What no scenario reaches:** REQ-004's race (a file changed between the read and the rename)
  rests on the review.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Fixed, #755); the guide (the agent bar's Codex and OpenCode
  setup and the backup, the bridge's token, the Chromium unit's OOM policy) and the in-app guide
  (#552's card, Shell integration, A Chromium per project, the variables table); the
  architecture notes `marley_mcp.md`, `terminal_blocks.md`, `marley_browser.md` and
  `marley_workbench.md`. `zed-touchpoints.md` line 85 describes the shipped hunk.
- **Knowledge appended:**
  - AD-claude-755-a-terminal-proves-its-id-with-a-token-for-the-run-001 (supersedes the "no
    authority" clause of AD-claude-520-…, whose status line says so);
  - AD-claude-755-other-programs-files-are-replaced-through-one-checked-helper-001;
  - L-claude-755-terminal-lists-id-is-the-views-not-the-environments-001;
  - L-claude-755-a-renderer-killed-for-memory-needs-a-cap-and-no-swap-001;
  - L-claude-755-serve-site-no-longer-ends-a-run-outside-a-substitution-001.
  - No `F-` block: neither phase found a bug in the product.
- **Brain:** consultation b274d5c1a0f34a259853cb8c3a8a0744 closed with a decision
  (`decisions/marley-proves-each-terminals-id-with-a-token-…`).
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed.

