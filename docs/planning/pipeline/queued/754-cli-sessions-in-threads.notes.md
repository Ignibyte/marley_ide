# CLI sessions in Threads — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-754-cli-sessions-in-threads.md
- **Pipeline spec:** 754-cli-sessions-in-threads.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** Chad, 2026-10-11, on the survey list (`docs/planning/design-notes/finish-line-2026-10-11.md`),
  item 4, "Old CLI sessions in Threads … finishes the Reopen Thread you just asked for": "4.) yes".
  Of New Agent from Thread, the same day: "my thought is that you would re open it" (#744's Reopen
  Thread…). The survey's text: "a pure parser for both CLIs' session files, a Sessions list per
  project, and `marley: search agent sessions` with T3's ranking and snippets; Enter opens `claude
  --resume` or `codex resume` in the session's folder" (`docs/t3code_architecture/README.md`, item
  3). The Threads page (#737) and the rail's Threads list (#745) take the place of a per-project
  Sessions list, as the tabs batch moved history there.
- **Classification:** feature, M (two parsers, the scan and cache, three lists, Reopen). It grows if
  #745's list does not share the page's rows.
- **Order:** after #744 (Reopen Thread…), #745 (the rail's Threads list) and #750 (`resume_line`
  per agent, Codex's sessions in `resume.rs`'s table).
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **Recall (§18.3):**
  - AD-claude-737: the history is Marley's own page over `ThreadMetadataStore`; search over titles
    and folders; a click brings an open conversation forward or opens it in a tab.
  - AD-claude-540: the live session id comes only from `SessionStart`, "never from
    `~/.claude/projects`". This ticket keeps that rule (D2): the files list, they do not map.
  - #540, #750: Reopen reuses `resume_line` and the startup handshake, the folder one quoted word
    after `cd --`, the id checked as 8-4-4-4-12 hexadecimal.
  - F-claude-588: a scenario names what it opens; the run's profile copies the user's Zed threads,
    so the page lists them too (#737's notes say the same).
  - F-claude-547: stand-ins go first on the terminal's PATH; `CLAUDE_CONFIG_DIR` is not set by the
    runner, so the scenario's `setup` exports it, which reaches Marley and its terminals.
  - PR-claude-blocking-io-off-the-gpui-main-thread-selftest-proves-it-001: every read off the main
    thread.
  - Orca report 01 §3 item 7 and T3 report 02 §3 item 2 name the same work; this is its unbuilt
    half (#540 built the live resume).
  - Brain: no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Discovery (2026-10-11; re-verify at promotion):**
  - `threads_page.rs`: `OpenThreads` 27–35, `init` 37, `open` 47–60, `Row` 83–90 (`thread_id,
    agent, title, folder, when`), the search editor 102–106, `sections` 126–180 (store entries
    without drafts, then archived, by `updated_at`), `render_chips` 182–212, `render_row` 239–279,
    `matches` 283–291, `shown_folder` 294, `open_thread` 310–362 (a tab, a panel, else
    `thread_tab::open_saved`). Callers: `status_buttons.rs:103-108`, `assistant.rs:967`.
  - `thread_metadata_store.rs`: `ThreadMetadata` 309–326, `is_draft` 331, `display_title` 335,
    `folder_paths` 344, `entries` 601, `archived_entries` 612.
  - `thread_import.rs`: 129, 689 (`session_list` at 729), 794–820, 855–888 (archived at 884);
    `acp_thread/src/connection.rs:392` `AgentSessionInfo { session_id, work_dirs, title,
    updated_at, created_at, meta }`; `agent_servers/src/acp.rs` 472, 507–545, 995–1010; ids
    `claude-acp`, `codex-acp` (`custom.rs:19-20`).
  - `agents.rs`: `cli_icon` 119, `thread_icon` 130, `thread_agent_name` 148, `start_cli` 298,
    `agent_line` 338, `launch_mode` 368, `start_in_terminal` 417–527, `NewAgent` 677, `show_picker`
    726, `NewAgentPicker` 883, `Start` 1009, `run_in` 1041–1066.
  - `rail.rs`: `group_context_menu` 4964 (New Agent… 5158, New Agent Thread 5167), `group_threads`
    7656, `ThreadSnapshot` built 7734, `filter_match` 8923; `marley_rail::ThreadRow` 744. No
    Threads switch yet (#745), no Reopen code yet (#744).
  - `marley_agent`: pure, gpui-free (`marley_agent.rs:4`), modules 37–45, `serde_json` among its
    dependencies; no reader of either CLI's files anywhere in `crates/marley_*`.
  - Redaction: `marley_mcp/src/redact.rs` `Redactor` 150, `redact` 176; `mcp.rs` `agent_redactor`
    486, `for_agents` 505.
  - The files on this box (structure only, read 2026-10-11): Claude Code records carry `type`
    (`user`, `assistant`, `system`, `attachment`, `ai-title`, `custom-title`, `last-prompt`, …),
    `sessionId`, `cwd`, `entrypoint`, `isSidechain`, `timestamp`; Codex rollouts open with
    `session_meta` (`id`, `cwd`, `timestamp`, `source`, `originator`, `thread_source`,
    `cli_version`), then `event_msg` (`task_started`, `item_completed` with `UserMessage` and
    `AgentMessage` items, `task_complete` with `last_agent_message`); `session_index.jsonl` lines
    are `{id, thread_name, updated_at}`.

### Design
- **`marley_agent::claude_sessions` and `marley_agent::codex_sessions` (Marley crate, new modules):**
  each `parse(head: &[u8], tail: &[u8], file_name: &str) -> Option<CliSession>` over whole lines
  only (a cut line at either window's edge is dropped); `CliSession { agent: AgentKind, id, folder,
  title: Option<String>, first_prompt: Option<String>, prompts: Vec<String> }` (prompts capped at 64
  KiB in all). Claude Code: the id from `sessionId` (else the file name), the folder from the first
  `cwd`, `entrypoint == "cli"` and not `isSidechain`, the newest `custom-title`, then `ai-title`, the
  prompts from user records that are not meta, commands, command output or tool results. Codex: the
  `session_meta` rules of the spec, the prompts from `UserMessage` items (and `user_message`
  events). `codex_sessions::names(index: &[u8]) -> HashMap<id, name>`. `search(session, query) ->
  Option<Snippet>`: a case-insensitive match in the title, the folder, then the first prompt that
  holds it, with up to 240 characters around it.
- **`marley_workbench::cli_sessions` (Marley crate, new):** `scan_in(claude_dir, codex_dir, cache)`
  off the main thread: the newest 500 files per CLI, the head and tail reads, the cache by path,
  size and time; a `CliSessions` global (the list, newest first, and when it was read) refreshed
  when the page or the rail's list shows, at most every 30 s, and emptied when the switch is off.
  `reopen(workspace, session, window, cx)`: a running terminal (`resume::terminal_of(session id)`)
  forward, else `start_in_terminal` with `resume_line` in the folder, or a toast for a missing
  folder.
- **`threads_page.rs`:** `Row` holds either a thread or a CLI session; `sections` merges them by
  time and drops CLI rows whose id a store entry holds; chips for the two CLIs; `matches` widened,
  the snippet line under the row; the click reopens.
- **The rail (#745) and Reopen Thread… (#744):** read the same global and call `reopen`.
- **`resume.rs`:** `terminal_of(session id)`, the terminal running a session now.
- **Settings (Zed paths, rows exist):** `crates/settings_content/src/marley.rs`
  (`list_cli_sessions`), `assets/settings/default.json`, `crates/settings_ui/src/marley_page.rs`
  (Agents section).
- **File manifest:** `crates/marley_agent/src/{marley_agent.rs, claude_sessions.rs,
  codex_sessions.rs}`; `crates/marley_workbench/src/{marley_workbench.rs, cli_sessions.rs,
  threads_page.rs, rail.rs, agents.rs, resume.rs}` (Marley crates); the three settings files;
  `docs/marley/guide.md`; `script/e2e/754-cli-sessions-in-threads.sh`.

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| 001, 002 | `marley: open threads` | 754-01-page: the three fixture sessions; no `sdk-cli` or subagent row |
| 003 | Types `tokenizer` in the search | 754-02-search: the first session and its snippet |
| 004 | Clears the search, picks the Codex chip | 754-03-chip |
| 005 | Picks All, clicks the first Claude Code row | 754-04-reopen: a terminal tab in the repository; the log's `--resume <id> @ <repo>` |
| 006 | Clicks the Codex row | 754-05-codex: the log's `resume <id> @ <repo>` |
| 007 | Clicks the first Claude Code row again | 754-06-forward: that terminal in front; no new log line |
| 008 | Switches the rail to Threads (#745) | 754-07-rail |
| 009 | Reopen Thread… from the repository's + menu (#744), picks the `sub` session | 754-08-reopen-thread: the log's `--resume <id2> @ <repo>/sub` |
| 010 | `profile_setting marley.list_cli_sessions false`, reopens the page | 754-09-off |
| 011, 012, 013 | review | the dedupe in `sections`; the folder check in `reopen`; the scan's thread, bounds and read-only opens |
Not reachable by a scenario: a real Claude Code or Codex resuming (an account and a model call);
the stand-ins print and log what the resume line gives them.

### Risks
- **Formats move.** Codex 0.162 already stopped writing `user_message` events; both formats are
  undocumented. The parsers fail soft: a session whose prompts cannot be found keeps its id, folder
  and time, titled by its id's start.
- **Big files.** The head and tail windows keep each read near 1.25 MiB; the cache keeps a scan of
  unchanged files to a `stat` each.
- **Duplicates with Zed's import.** Only an id match dedupes; a session imported under another id
  would show twice.
- **#744 and #745 are not built yet.** Their lists' shapes decide how much of the rail and the
  picker this ticket touches; re-plan at promotion.
- **The scenario's profile** lists the user's real Zed conversations; `CLAUDE_CONFIG_DIR` and
  `CODEX_HOME` point at the fixtures, so no real CLI file is read.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
