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

### Promotion (2026-10-11)
- Moved from `queued/` to `active/`; the BACKLOG row removed; the ticket in progress. Brain:
  consultation d2746839fee94deaa0805f08823b873c, nothing on this seam.
- **The files, read again on the box (structure only):** Claude Code's records as the draft says;
  the titles are `custom-title.customTitle` and `ai-title.aiTitle`; user records carry `isMeta`,
  `toolUseResult`, `promptSource` and `turnOrigin`; of the newest 40 files' first MiB, 4484 records
  say `entrypoint: cli`, 170 `sdk-cli`, 69 `sdk-ts`. Codex's `session_meta` holds `id`, `cwd`,
  `originator` (`codex-tui`, `Codex Desktop`, `codex_work_desktop`, `zed`), `source`,
  `thread_source` (`user`, absent, `subagent`, `guardian_review`), `parent_thread_id`; prompts are
  `event_msg` `item_completed` items `{type: UserMessage, content: [{type: text, text,
  text_elements}]}`; `session_index.jsonl` lines are `{id, thread_name, updated_at}`.
- **Seams checked again** at `71775d80c8` (an Explore pass over all eleven):
  - `threads_page.rs` now holds all three lists: the page (`ThreadsPage`, a center tab,
    `open` 112-125, `sections` 191-231, `render_chips` 233-263 with a chip per `AgentId`,
    `render_row` 290-330, `matches` 334-342, `open_thread` 362-420), the rail's Threads view
    (`rail.rs` `render_thread_list` 5147-5218 draws `threads_page::conversations(query)` and
    clicks through `open_conversation` 5222 → `open_thread`), and Reopen Thread…
    (`ReopenThreadPicker` 518, `ReopenThreadDelegate` 562-570 over `conversations("")`, its
    confirm 646-663 → `open_thread`). One `Row { thread_id, agent, title, folder, when }` (148-155)
    from `row_of(&ThreadMetadata)` (93-109). So the CLI rows join `Row` and `conversations` once
    and all three lists show them.
  - `agents.rs`: `cli_icon` 123; `agent_line` 319-345 and `launch_input` 364 give a line and a
    Codex `Joining`; `start_in_terminal` 398 opens a center terminal in a folder and types it;
    `start_cli_in` 1112.
  - `resume.rs`: `Sessions` is private and keyed by terminal id and view id; **nothing maps a
    session id to its terminal.** A seat carries it (`session_id` for Claude Code, `thread` for
    Codex) and a seat's id is its view's (`agent_events::seat_id`), so the running terminal is
    found through `AgentEvents`. `marley_agent::resume_line` (247-291) as the draft says.
  - `browser::reveal_terminal(view, window, cx)` (7791) brings a terminal tab forward in the
    window's workspaces, center or panel.
  - `thread_metadata_store.rs`: `entries()` (601) includes archived threads; `entry_by_session`
    (595).
  - Settings: `agents_section() -> [SettingsPageItem; 20]`; `MarleySettings` holds three bools,
    clippy's limit, so the switch is an enum as `ResumeAgents` is.
  - `script/e2e.sh` exports `CODEX_HOME=$E2E_WORK/codex-home` (708) and not `CLAUDE_CONFIG_DIR`; a
    scenario's `export` in `setup` reaches Marley (509's scenario does).
- **Found in passing (pre-existing, fixed here since the code is rewritten):**
  - `sections` chains `entries()`, which already includes archived threads, with
    `archived_entries()`, so every archived thread showed twice under ARCHIVED and in its count.
  - `conversations` says it lists unarchived conversations but never checks `archived`, so Reopen
    Thread… and the rail's Threads view listed archived ones.
- **What differs from the draft, and where the design takes it:**
  - **One module per crate:** the two parsers share `CliSession` and `search`, so they live in
    `marley_agent::cli_sessions` (pure), and the scan, cache, global and Reopen in
    `marley_workbench::cli_sessions`.
  - **`Row` gains a target:** `Target::{Thread(ThreadId), Cli(Arc<CliSession>)}`, the moment it
    sorts by, the icon, and a snippet; `open_row` dispatches the click for all three lists. The
    chips become `Chip::{Agent(AgentId), Cli(AgentKind)}`, the CLIs' named "Claude Code CLI" and
    "Codex CLI" beside Zed's agents of the same names.
  - **The running terminal** is found from the `AgentEvents` seats (not `resume.rs`), then
    brought forward with `browser::reveal_terminal`.
  - **Reopen** builds its line with a new `agents::resume_input` beside `agent_line` (the
    permission mode, and a Codex App Server through `Joining` when #650's switch is on), checks
    the folder off the main thread, and opens through `start_in_terminal`.
  - **The scan's trigger:** `cli_sessions::refresh` when the page opens, the rail shows its
    Threads view, and Reopen Thread… opens; the page and the rail observe the global.
  - **The switch** is `ListCliSessions::{Listed, Off}` in `MarleySettings`.

## Phase 2 — Code
- **Built.**
  - `marley_agent::cli_sessions` (new, pure): `CliSession { agent, id, folder, title, prompts }`
    with `shown_title` and `search` (`Found::{Listed, Prompt(snippet)}`), `Windows { head, tail }`
    (whole lines only), `parse_claude`, `parse_codex` (untitled), `codex_names`; prompts through
    `claude_events::prompt_origin`, capped at 64 KiB; a 240-character snippet.
  - `marley_workbench::cli_sessions` (new): `refresh` (off the main thread, 30 s apart, the switch
    honoured), `scan_in(claude, codex, cache)`, the newest 500 files per CLI, 1 MiB head and 256 KiB
    tail, the cache by path, size and time, Codex's names applied each scan; the `CliSessions`
    global and `listed`; `reopen` (the running terminal through `agent_events::view_running` and
    `browser::reveal_terminal`, else the folder checked off the main thread and
    `agents::resume_input` into `start_in_terminal`, or a toast).
  - `threads_page.rs`: `Row` with `Target`, `updated_at`, `snippet`, and `icon`, `detail`, `key`;
    `open_row`; `cli_rows` (the store's session ids left out), `matches_cli` with `CliSearch`
    (the snippet through `mcp::for_agents`); `Chip::{Agent, Cli}`; `sections` reads `entries()`
    once (the archived fix) and merges CLI rows; `conversations` leaves archived threads out and
    adds CLI rows by title and folder; the picker opens through `open_row`; the page observes
    `CliSessions` and asks for a scan when it opens, as Reopen Thread… does.
  - `rail.rs`: the Threads view draws through `Row`'s helpers, opens through `open_row`, asks for
    a scan when it shows, and redraws when a scan lands.
  - `agents::resume_input`, `agent_events::view_running`, `ListCliSessions` in `MarleySettings`,
    and the setting on the three Zed paths, rows widened first.
- **Deviations from the design:** one module per crate (the parsers share `CliSession` and
  `search`); `parse_codex` takes no names, since the index changes without the rollout and the
  scan names each Codex session afresh.
- **Review against the criteria.**
  - The snippet went out unredacted in the first cut; it now passes #516's redactor while agents'
    redaction is on (the spec's In).
  - Reopen runs inside the window's update (a click), so the running terminal is looked for in the
    window's own root (PR-claude-596) and the folder check is off the main thread
    (PR-claude-blocking-io-off-the-gpui-main-thread-selftest-proves-it-001).
  - The scan's reads are `File::open`, read-only; a record that does not parse is skipped.
  - Clippy found `or_fun_call`, `implicit_hasher` (gone with the names parameter), a long first
    doc paragraph, `similar_names`, `clone_on_ref_ptr`, `case_sensitive_file_extension_comparisons`,
    `single_match_else`, `redundant_clone` and `needless_pass_by_ref_mut`.
- **Gate, first run: red on two.** gate:14, the public `CliSession::prompts` doc linked the
  private `MAX_PROMPT_BYTES` (`rustdoc::private_intra_doc_links`): it says "64 KiB" now. gate:21,
  Zed's `async_block_without_await` on the scan's `background_spawn(async move { scan_in(..) })`:
  `futures::future::lazy`, as `codex_server`'s blocking checks do. Neither shows in `./script/clippy`
  on the crates, so only the gate finds them.
- **Gate, second run: red on gate:14 again,** the public `cli_sessions` module's doc linking five
  private items; it names the bounds in words now, and `RUSTDOCFLAGS="-D warnings" cargo doc
  --no-deps` on the three crates passed before the third run. A repeat of
  PR-claude-no-public-to-private-intra-doc-link-001: `./script/clippy` does not run rustdoc, so
  the scoped doc build belongs before the gate, not after it.
- **Gate, third run:** `script/gates.sh --diff` green, every gate and the receipt.

## Phase 3 — Test
- **Scenario:** `script/e2e/754-cli-sessions-in-threads.sh` under `compositor sway`; fixtures in
  `CLAUDE_CONFIG_DIR` (exported in `setup`) and the runner's `CODEX_HOME`; stand-in `claude` and
  `codex` log to `launches.log`. Four checks, all pass on the final run: `claude --resume` of the
  first session in the repository, `codex resume` in the repository, no second `claude` when the
  running session is chosen again, and `claude --resume` of the `sub` session in `sub`.
- **Shots:**
  - `754-01-page` (REQ-001, REQ-002): "Fix the flaky tokenizer test" (Claude Code CLI),
    "Usage meter" (Codex CLI), "Release notes for sub" (Claude Code CLI), newest first, with
    folders and times; the chips All, Marley, Rusty, Claude Agent, Claude Code CLI, Codex CLI; no
    SDK or subagent row; Show archived (7), each once (REQ-014).
  - `754-02-search` (REQ-003): `parser`, a word only the prompt holds: the first session with
    "fix the flaky parser test in tokenizer.rs" in italics under it.
  - `754-03-chip` (REQ-004): Codex CLI: only "Usage meter".
  - `754-04-reopen` (REQ-005): a new terminal tab running `cd -- '<repo>' && claude --resume
    1111…`; the rail's row `idle`.
  - `754-05-codex` (REQ-006): a new terminal running `codex resume 4444…` in the repository.
  - `754-06-forward` (REQ-007): the first session's terminal in front, no new tab.
  - `754-07-rail` (REQ-008): the rail's THREADS view with the three sessions.
  - `754-08-reopen-thread` (REQ-009): from the repository's `+`, "release notes" finds "Release
    notes for sub"; Enter opened a `sub` terminal (`09`'s tab bar).
  - `754-09-off` (REQ-010): `list_cli_sessions` false: no CLI row and no CLI chip.
- **Not driven:** REQ-011, REQ-012, REQ-013 (review), and a real resume (an account and a model).
- **Calibration:** the palette's "marley: reopen thread" ran Open Threads, so the picker is opened
  from the `+` menu as #772's scenario does; the first search word matched the title, so the
  search uses a word only the prompt holds; shellcheck's SC2153 read `CODEX_HOME` as a misspelling
  of the fixture's `CODEX_ONE`, renamed.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added; Fixed: archived threads once); `docs/marley/guide.md`
  (the Threads page's CLI sessions, the rail's view, Reopen Thread…); the in-app guide (the rail's
  Threads step, the `marley: reopen thread` row); walkthrough stop 6.1b;
  `docs/marley_architecture/marley_workbench.md` (the page's `Row` and the new `cli_sessions.rs`
  section) and `marley_agent.md` (`cli_sessions.rs`). The three Zed rows name `list_cli_sessions`.
- **Knowledge:** F-claude-754-the-threads-page-listed-archived-threads-twice-001;
  L-claude-754-the-clis-session-files-say-who-started-them-001;
  AD-claude-754-past-cli-sessions-are-read-from-the-clis-files-and-reopened-with-their-resume-001.
  The two gate reds repeat PR-claude-no-public-to-private-intra-doc-link-001 and the
  `async_block_without_await` lessons; no new rule.
- **Brain:** consultation d2746839fee94deaa0805f08823b873c closed with a decision (follow-up
  2026-11-11).
