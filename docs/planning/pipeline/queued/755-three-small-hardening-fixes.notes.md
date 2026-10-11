# Three small hardening fixes — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-755-three-small-hardening-fixes.md
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
