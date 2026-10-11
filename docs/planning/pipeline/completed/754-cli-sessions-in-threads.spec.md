---
pipeline_id: b24d5935-511e-4225-bb4b-8efb9f969d5f
ticket: docs/planning/tickets/open/TICKET-754-cli-sessions-in-threads.md
status: Phase 4 — Complete PASS
title: CLI sessions in Threads
type: feature
slice: the finish line (docs/planning/design-notes/finish-line-2026-10-11.md), item 4, old CLI sessions in Threads; finishes #744's Reopen Thread…
references:
  - docs/planning/pipeline/completed/737-the-threads-page.spec.md
  - docs/planning/pipeline/completed/540-session-resume-after-restart.spec.md
  - docs/planning/pipeline/queued/750-codex-resumes-and-a-cut-off-turn-continues.spec.md
  - docs/planning/tickets/open/TICKET-744-one-new-agent.md
  - docs/planning/tickets/open/TICKET-745-threads-in-the-rail.md
  - docs/t3code_architecture/01-agents-and-providers.md
  - docs/t3code_architecture/02-threads-and-composer.md
  - docs/orca_architecture/01-agents-and-sessions.md
---

## Title
The Claude Code and Codex sessions Chad ran in terminals are listed with Zed's conversations: on the
Threads page (#737), in the rail's Threads list (#745) and in Reopen Thread… (#744). Marley reads
them from each CLI's own session files, read-only, through a pure parser per CLI, and keeps only
sessions started in the CLI's own TUI. The page's search finds a session by its title, its folder or
the words of its prompts. Choosing one runs `claude --resume <id>` or `codex resume <id>` in a center
terminal in the session's folder, or brings forward the Marley terminal already running it. Chad,
2026-10-11, of the survey's item 4 ("Old CLI sessions in Threads … finishes the Reopen Thread you
just asked for"): "4.) yes"; of New Agent from Thread: "my thought is that you would re open it".

## Scope
### In
- **Where the files are:** Claude Code's `<config>/projects/*/*.jsonl`, `<config>` being
  `CLAUDE_CONFIG_DIR` or `~/.claude`; Codex's `<home>/sessions/YYYY/MM/DD/rollout-*.jsonl` and
  `<home>/session_index.jsonl`, `<home>` being `CODEX_HOME` or `~/.codex`. Never written.
- **Which sessions:** Claude Code sessions whose records say `entrypoint: "cli"` and that are not
  sidechains; Codex sessions whose first record (`session_meta`) names the TUI as `originator`
  (`codex-tui`, `codex_cli_rs` in older releases) and whose `thread_source` is `user` or absent.
  The `source` is not the test: a TUI joined to an App Server (#650) records `vscode` there. Left
  out: `claude -p` and SDK sessions (`sdk-cli`, `sdk-ts`, among them the harness's and Zed's own
  agent's), Codex sessions of Zed's agent (`originator: "zed"`) and of Codex's desktop app,
  subagents and reviews, and sessions with no prompt of the user's.
- **What a row holds:** the CLI's icon (`agents::cli_icon`), a title, the session's folder (home as
  `~`) and its last change (the file's modification time). The title: Claude Code's newest
  `custom-title`, else its newest `ai-title`, else the first prompt; Codex's `thread_name` in
  `session_index.jsonl`, else the first prompt. A prompt is the user's own words: Claude Code's user
  records that are not meta, a command, a command's output or a tool result; Codex's `UserMessage`
  items (and the older `user_message` events).
- **Bounds:** per CLI the newest 500 files by modification time; from each file at most its first
  1 MiB and its last 256 KiB; a cache by path, size and modification time, so an unchanged file is
  not read again; a scan off the main thread when the Threads page or the rail's Threads list is
  shown, at most once every 30 seconds. A record that does not parse is skipped.
- **The Threads page:** CLI rows among All conversations, newest first with Zed's; a chip each for
  Claude Code and Codex; the search matches a CLI row's title, folder and prompts, and a prompt
  match shows up to 240 characters around it, through #516's redactor while redaction is on.
- **The rail's Threads list (#745):** the same rows, filtered by the rail's filter on titles and
  folders.
- **Reopen Thread… (#744):** lists the CLI sessions with Zed's conversations; choosing one reopens it.
- **Reopen:** a session running in a Marley terminal now (its id in `resume.rs`'s table, Claude
  Code's from its hooks or report, Codex's from #750) brings that terminal forward. Otherwise a new
  center terminal in the shown group, in the session's folder, through `start_in_terminal` with
  `resume_line` (#750): `claude [#532's arguments] --resume <id>`, or `codex resume [#532's
  arguments] <id>` with Marley's App Server when it is on (#650). A folder that no longer exists
  starts nothing and says so in a toast.
- **Archived threads once:** the page lists each archived conversation once (it listed them
  twice), and Reopen Thread… and the rail's Threads view list unarchived ones, as `conversations`
  says.
- **One row per conversation:** a CLI session whose id is a Zed conversation's `session_id` (Zed's
  Import Threads brings Claude Code sessions in through its ACP adapter) is listed once, as Zed's.
- **The switch:** `marley.list_cli_sessions: Option<bool>`, default `true`, in
  `MarleySettingsContent`, `default.json` and the Marley page's Agents section. Off, no file is read
  and no CLI row shows.
- `docs/marley/guide.md`: the rows, the search, Reopen, the switch.
- `script/e2e/754-cli-sessions-in-threads.sh`.

### Out (explicitly deferred)
- Searching the agents' replies, and the rest of a long transcript past the windows read (T3 reads
  1 MB of each too).
- A session's detail (model, branch, message count, the latest turns; Orca's AI Vault), deleting a
  session, copying its resume command.
- Other CLIs (OpenCode, Gemini CLI): two formats are enough to keep up with.
- Handing a session to the other CLI, "Continue in Codex" (#717).
- Sessions on remote hosts, and the harness's sessions (its own Sessions tab, #632).
- Marley's palette search across sessions (`marley: search agent sessions`): the page's search
  covers it.

## Reference (§20)
Upstream Zed, `agent_ui::thread_import::ThreadImportModal`: an agent's own past sessions listed
through ACP's `session/list` and added to the thread store, skipping ids the store already holds
(`collect_importable_threads`), and `ThreadsArchiveView`'s search over titles and folders. Behavior
kept: past sessions shown beside the store's conversations, one row per session id, found by title
and folder. Marley's difference: the sessions come from the CLIs' own files, are listed as they
change rather than imported once, and reopen in a terminal through the CLI's own resume.

### Prior art
- **The code we ship (the highest-yield leg):**
  - `crates/agent_ui/src/thread_import.rs`: `ThreadImportModal` (129); `fetch_sessions_for_agent`
    (689) asks the connection for `session_list`; `collect_all_sessions` (794) pages through it;
    `collect_importable_threads` (855) makes each an archived `ThreadMetadata` (884) and skips ids
    the store has. The installed `claude-agent-acp` (0.89.1) answers `session/list` from Claude Code's
    own transcripts, so Import Threads already brings CLI sessions in, but by hand, archived, and
    reopened through the ACP adapter in a thread view, not the TUI in a terminal. `codex-acp` is not
    installed here, so its listing is unchecked. The dedupe rule is taken from it.
  - `crates/agent_servers/src/acp.rs`: `AcpSessionList` (472), `list_sessions` (507), made only
    when the agent's capabilities include `session_capabilities.list`.
  - `crates/agent_ui/src/thread_metadata_store.rs`: `ThreadMetadata` (309, `session_id` 311),
    `display_title` (335), `entries` (601), `archived_entries` (612).
  - `crates/agent_ui/src/threads_archive_view.rs`: `fuzzy_match_positions` (105, a contiguous,
    case-insensitive match), `update_items` (271).
  - `crates/marley_workbench/src/threads_page.rs` (#737): `open` (47), `Row` (83–90), `sections`
    (126–180), `render_chips` (182), `render_row` (239), `matches` (283–291), `shown_folder` (294),
    `open_thread` (310–362). The CLI rows join `Row` and `sections`.
  - `crates/marley_workbench/src/agents.rs`: `cli_icon` (119), `launch_mode` (368),
    `start_in_terminal` (417), the picker (`show_picker` 726, `Start` 1009, `run_in` 1041);
    `rail.rs`: the + menu (`group_context_menu` 4964, New Agent… 5158), `group_threads` (7656),
    `filter_match` (8923).
  - `crates/marley_workbench/src/resume.rs` (#540, #750): the terminal ↔ session table that tells a
    running session's terminal; `marley_agent::resume_line`, per agent after #750.
  - `CLAUDE_CONFIG_DIR` and `CODEX_HOME` are resolved the same way in `claude_plugin.rs:116-117` and
    `agent_notify.rs:67-68`.
  - #516's redaction: `marley_mcp::redact::Redactor` (150, `redact` 176) through
    `mcp::agent_redactor` (486) and `for_agents` (505).
  - `crates/marley_agent`: pure and gpui-free (`marley_agent.rs:4`), with `serde_json` already; the
    two parsers live there.
- **Behavior maps:** T3 report 01 §2.13 (`AgentSessionScanner.ts`: Claude's `projects`, Codex's
  `sessions`, the `cwd` from each transcript, 5,000 transcripts and 1 MB per transcript at most)
  and §3 item 4; report 02 §2.7 (`ThreadSearch.ts`: one best hit per thread, the user's words
  first, then newest, a 240-character snippet) and §3 item 2 (prompts and final replies only, fail
  soft, snippets redacted); Orca report 01 §2.6 (titles from `custom-title`, `ai-title` or the first
  prompt, Codex's from `session_index.jsonl`; the newest files by modification time; a parse cache
  by size and time) and §3 item 7.
- **Published material:** Claude Code's `--resume <id>` (sessions found by their folder);
  `codex resume [SESSION_ID]` (0.162.0: filters by folder unless `--all`); ACP's `session/list`.
  The two file formats are undocumented. On this box on 2026-10-11: 257 Claude Code session files
  (the largest 446 MB, the median 288 KB; 114 `cli`, 139 `sdk-cli`, 4 `sdk-ts`; `ai-title` records
  repeat through each file) and 176 Codex rollouts (the largest 145 MB; 50 from the TUI as a user
  thread, 2 of them through an App Server with `source` `vscode`; 97 subagents; the rest the
  desktop app's, Zed's or reviews). Codex 0.162 writes the user's prompts as `item_completed`
  `UserMessage` items and no `user_message` events.

## UI proof
`script/e2e/754-cli-sessions-in-threads.sh`, `compositor sway`, a scratch repository with `sub`.
`setup` exports `CLAUDE_CONFIG_DIR=$E2E_WORK/claude` (Marley and its terminals); the runner already
sets `CODEX_HOME` to the run's folder. Fixtures: two `cli` Claude Code sessions (one in the
repository, its prompt "fix the flaky parser test in tokenizer.rs" and an `ai-title`; one in `sub`
with a `custom-title`), an `sdk-cli` one, a Codex `cli` rollout in the repository (a `UserMessage`
"add a usage meter", named in `session_index.jsonl`) and a Codex subagent rollout. Stand-in `claude`
and `codex` first on the terminal's PATH log their arguments and folder; the `claude` one prints
#519's `SessionStart` for the id it resumes. The runner's profile copy brings the user's own Zed
conversations onto the page too, as in #737; the checks rest on the fixture's rows.

Shots:
- `754-01-page`: the Threads page with the three fixture sessions, each with its CLI's icon, title,
  folder and time; no `sdk-cli` or subagent row.
- `754-02-search`: `parser`, a word only its prompt holds, in the search: the first Claude Code
  session, with its prompt's snippet.
- `754-03-chip`: the Codex chip: only the Codex session.
- `754-04-reopen`: the first Claude Code row clicked: a terminal tab in the repository; the log's
  `--resume <id>` in the repository.
- `754-05-codex`: the Codex row clicked: the log's `resume <id>` in the repository.
- `754-06-forward`: the first Claude Code row clicked again: the same terminal tab in front, no new
  terminal, no new log line.
- `754-07-rail`: the rail's Threads list with the fixture's rows.
- `754-08-reopen-thread`: Reopen Thread… from the repository's `+` lists them; "release notes"
  finds the `sub` session; Enter: the log's `--resume <id>` in `sub`.
- `754-09-off`: `marley.list_cli_sessions` false: the page with no CLI row.

## Locked-In Decisions
- **D1 — Each CLI's own files, a pure parser per CLI in `marley_agent`;** the IO beside them in
  `marley_workbench`, through functions that take the folder (§14). Rejected: Zed's Import Threads
  over the ACP adapters (one-off and by hand, archived rows, reopened through the adapter rather
  than the TUI, Codex's listing unchecked); Codex's `state_5.sqlite` thread table (an internal store
  whose schema version is in its file name); the App Server's `thread/list` (a server per scan, and
  #650's switch is off by default).
- **D2 — AD-claude-540 still holds.** It keeps the files from deciding which session a live terminal
  runs; that still comes only from the hooks, a report or the App Server. Here the files only list
  what can be reopened.
- **D3 — Only sessions started in the CLI's TUI.** Rejected: listing `claude -p` and SDK sessions
  (the harness's and Zed's own agent's, which the store already lists) and subagents (not
  conversations Chad had).
- **D4 — Bounded reads and a cache,** since a session file can pass 400 MB: the newest 500 files
  per CLI, 1 MiB from the head and 256 KiB from the tail, reread only when size or time changes,
  off the main thread (PR-claude-blocking-io-off-the-gpui-main-thread-selftest-proves-it-001).
- **D5 — One list.** CLI rows sit among Zed's conversations by time, with a chip per CLI, so Reopen
  Thread… and the page show one history. Rejected: a section of their own.
- **D6 — Reopen never makes a second copy:** a running session's terminal comes forward (#697's
  one-place rule); otherwise the CLI's own resume in the session's folder, with #532's arguments,
  through `start_in_terminal`, as #540 and #750 do.
- **D7 — Prompt search on the page only;** the rail's filter keeps its title and folder rule, so the
  rail stays light. Snippets go through #516's redactor while it is on.
- **D8 — A switch of its own, on by default:** it reads local files and reaches nothing outside the
  box.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the Threads page shows, it shall list the interactive Claude Code and Codex sessions in the CLIs' own files among Zed's conversations, newest first, each with its CLI's icon, title, folder and time. | Shot 754-01 |
| REQ-002 | The system shall not list `claude -p` or SDK sessions, sidechains, or Codex subagent, review or desktop sessions. | Shot 754-01 |
| REQ-003 | WHEN text is typed in the page's search, the page shall list the CLI sessions whose title, folder or one of the user's prompts holds it, with a snippet around a prompt match. | Shot 754-02 |
| REQ-004 | WHEN a CLI's chip is chosen, the page shall list only that CLI's sessions. | Shot 754-03 |
| REQ-005 | WHEN a Claude Code session that runs in no Marley terminal is chosen, the system shall open a center terminal in the shown group, in the session's folder, running `claude --resume <id>` with the project's permission arguments. | Shot 754-04; the log |
| REQ-006 | WHEN a Codex session that runs in no Marley terminal is chosen, the system shall open a center terminal in the session's folder running `codex resume <id>`. | Shot 754-05; the log |
| REQ-007 | WHEN a session that runs in a Marley terminal is chosen, the system shall bring that terminal forward and start nothing. | Shot 754-06; the log |
| REQ-008 | The rail's Threads list shall list the CLI sessions and reopen one as the page does. | Shot 754-07 |
| REQ-009 | Reopen Thread… shall list the CLI sessions and reopen the chosen one. | Shot 754-08; the log |
| REQ-010 | WHERE `marley.list_cli_sessions` is false, the system shall read no CLI session file and list none. | Shot 754-09; review |
| REQ-011 | WHEN a CLI session's id is a Zed conversation's session id, the system shall list it once, as that conversation. | Review of the diff |
| REQ-012 | WHEN a chosen session's folder no longer exists, the system shall start nothing and say so. | Review of the diff |
| REQ-013 | The system shall read the CLIs' files off the main thread, within the bounds of D4, and never write to them. | Review of the diff |
| REQ-014 | The Threads page shall list each archived conversation once, and Reopen Thread… and the rail's Threads view shall list no archived conversation. | Review of the diff |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes, re-verified at promotion against what #744,
  #745 and #750 shipped.
- **P2 Code** — `marley_agent::{claude_sessions, codex_sessions}` (the parsers); a `cli_sessions.rs`
  in `marley_workbench` (the scan, the cache, the global, Reopen); `threads_page.rs` (rows, chips,
  search, snippets); the rail's Threads list and Reopen Thread… (as #744 and #745 built them); the
  three settings files; the guide; the scenario; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
