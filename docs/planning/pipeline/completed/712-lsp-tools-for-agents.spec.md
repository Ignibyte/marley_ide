---
pipeline_id: 79043f45-4b80-4517-856e-243849240ae9
ticket: docs/planning/tickets/open/TICKET-712-lsp-tools-for-agents.md
status: Phase 4 — Complete PASS
title: LSP tools for agents
type: feature
slice: Marley's MCP server (docs/planning/intake/zed-control-over-mcp.md), after #704 and #705
references:
  - docs/planning/pipeline/completed/704-editors-read-and-open.spec.md
  - docs/planning/pipeline/completed/680-tool-results-fit.spec.md
  - docs/planning/pipeline/completed/653-marley-as-claude-codes-ide.spec.md
---

## Title
Every agent that reaches Marley's MCP server asks Zed's language servers what an editor knows:
a file's or a project's diagnostics, where a symbol is defined, where it is used, and its hover
text. Chad, 2026-10-11: "6.) yes".

## Scope
### In
- **Four read tools** in the editors family (`Family::Editor`, `Tier::Read`, no grant):
  - `editor_diagnostics {path?, project?, start?}`: with `path`, that file's diagnostics, every
    severity, the primary entry of each group. Without it, the diagnostics of every file Zed's
    summaries count with errors or warnings, in the project `project` names, else the caller's
    project (#520's `Caller`), else every open project; at most 200 files.
  - `editor_definition {path, line, symbol? | column?}`: the definitions of the symbol there.
  - `editor_references {path, line, symbol? | column?, start?}`: the places it is used.
  - `editor_hover {path, line, symbol? | column?}`: the hover text and the range it covers.
- **The position:** `path` absolute and inside an open project's folders; `line` 1-based; then
  `symbol`, a name on that line (its first occurrence), or `column`, 1-based, counted as
  `editor_list` counts it.
- **A file no editor shows** is opened in its project without a tab and registered with its
  language servers for the call; the tool waits up to 15 seconds for a server to run for it.
- **Each location** carries `path`, `line`, `column`, `end_line`, `end_column` and `text` (its
  line, trimmed, at most 200 characters).
- **Redaction, secrets and paging, as `editor_read`'s:**
  - Messages, hover text and location lines pass through the agents' redaction (#516), with
    `redacted` counted.
  - A `path` matching `marley.agent_control.secret_globs` is refused `secret_file`. A location in
    such a file keeps its path and line, and its `text` reads `[secret file]`. A project listing
    leaves such files out and counts them.
  - References and diagnostics come a page at a time: at most 200 entries or 12,000 bytes, with
    `total`, `start` and `next`; hover text is cut at 12,000 bytes with `cut: true`.
- **The editors area:** `admit` at `Level::Read`, so `marley.agent_control.editors: off` refuses
  them and no other mode asks. Each call is listed in Agent Activity, as `editor_read` is.
- **Refusals with codes:** `bad_argument`, `outside_projects`, `no_file`, `secret_file`,
  `no_symbol` (with the line's text), `no_language_server`, `lsp_timeout`.
- **Docs:** the guide's "Zed's editors for agents" rows, `docs/marley_architecture/marley_mcp.md`
  and `marley_workbench.md`.

### Out (explicitly deferred)
- Rename, code actions and formatting through the language server: they change files, and
  #705's edit path and its mode come first.
- Workspace symbols and document outline tools.
- Pulling a whole project's diagnostics afresh (`pull_workspace_diagnostics_once`): the project
  listing answers from what the servers last published.
- Marley's own failed-block diagnostics (#623, source `marley`): left out of every answer.
- Turning #653's IDE rows on: that waits on Chad's by-hand check of the IDE link.
- Outside clients (#524): their tool lists stay as they are.

## Reference (§20)
Upstream Zed, the `project` and `language` crates: `Project::definitions`, `references`, `hover`,
`diagnostic_summaries` and `register_buffer_with_language_servers`, and the buffer's
`diagnostic_groups`. Behavior kept: what Zed's own editor and its agent's `diagnostics`,
`go_to_definition` and `find_references` tools answer, read through public APIs with no Zed hunk.
No Warp analog read.

### Prior art
- **The code we ship:**
  - Zed's agent tools: `crates/agent/src/tools/diagnostics_tool.rs` (92-133 pull fresh
    diagnostics for one file or the project; 171-217 one file's primary entries; 218-254 the
    project's summaries as counts only); `go_to_definition_tool.rs` (68-106) and
    `find_references_tool.rs` (68-97), behind `LspToolFeatureFlag` (`tools.rs:233-243`);
    `symbol_locator.rs` (146-217): a path, a 1-based line and a symbol name found on that line,
    the shape this ticket adopts beside a column. No hover tool exists in any Zed crate.
  - Zed's agent tools never wait for a language server: they work because an editor or the
    action log already registered the buffer (`read_file_tool.rs:448`,
    `action_log.rs:154-157`). `Project::open_buffer` alone starts no server and sends no
    `didOpen` (`lsp_store.rs:5234-5250`), and a local request asks only the servers that opened
    the buffer (`request_filtered_lsp_locally`, `lsp_store.rs:10630-10680`). The only wait in the
    tree is `edit_prediction_cli/src/retrieve_context.rs:351-462`
    (`wait_for_language_servers_to_start`: register, then `LanguageServerAdded` with a timeout).
  - `Project` (`project.rs`): `open_buffer` 3290, `register_buffer_with_language_servers`
    3320-3328 (its `OpenLspBufferHandle` unregisters when dropped, `lsp_store.rs:5340-5360`),
    `definitions` 4416, `references` 4501, `hover` 4654 (no `Result`), `diagnostic_summaries`
    5368, `find_project_path` 5419. Types: `LocationLink` 905, `Location`
    (`language.rs:306-310`), `Hover` and `HoverBlock` 939-957, `diagnostic_groups`
    (`buffer.rs:5462-5485`).
  - Marley: #653's `getDiagnostics` (`claude_ide.rs:718-762` gathering, `file_diagnostics`
    765-804) already reads a project's files and their groups' primary entries; this ticket
    shares `file_diagnostics` rather than writing it twice. #704's `editor_tools.rs` (`read`
    207-274, `not_secret` 191-204, `project_holding` 565-586) and `mcp.rs` (`page_from`
    1203-1252, `PAGE_BYTES` 61, `for_agents` 505-513).
- **Behavior maps:** `docs/zed_architecture/` (the editor and project subsystems); the
  T3 survey 06 §2.4 on result sizes, which #680 adopted.
- **Published material:** the LSP 3.17 specification (`textDocument/definition`, `references`,
  `hover`, `publishDiagnostics` and pulled diagnostics); MCP 2025-06-18, the revision Marley
  serves.

## UI proof
`script/e2e/712-lsp-tools-for-agents.sh`, under `compositor sway`. The repo holds a small Rust
crate (`src/main.rs` with one error and a symbol used twice, `src/lib.rs` that no tab opens) and
a made-up `.env`. #653's stand-in language server, named in `lsp.rust-analyzer.binary.path` with
`session.trust_all_worktrees: true`, publishes one error on `didOpen` of `src/main.rs` and
answers definition, references and hover. A client script on the terminal's PATH calls the tools
through the plugin's bridge and prints each answer. `src/main.rs` sits on the left and a Marley
terminal on the right, so each shot shows the editor and the answer.

Shots:
- `712-01-diagnostics` and `712-01b-project-diagnostics`: `editor_diagnostics` for main.rs and
  for the project: the stand-in's error at its line and column, the editor's underline beside it.
- `712-02-definition`: `editor_definition` of `answer` on line 3: the line it is defined on.
- `712-03-references`: `editor_references`: both uses, `total` 2.
- `712-04-hover`: `editor_hover`: the stand-in's hover text and its range.
- `712-05-no-tab`: `editor_definition` in `src/lib.rs`: answered, and the tab bar shows no
  lib.rs.
- `712-06-refused`: `.env` refused `secret_file`, `/etc/hostname` refused `outside_projects`, a
  symbol not on its line refused `no_symbol` with the line's text.
- `712-07-off`: Editors set to Off: `editor_hover` refused `agent_control_off`.
- `712-08-activity`: Agent Activity lists the calls.

## Locked-In Decisions
- **D1:** four read tools in the editors family, admitted at `Level::Read`. Rejected: a new
  `lsp_*` family with its own area mode (one more switch for reads the Editors mode already
  covers).
- **D2:** a position is `path`, `line` and either `symbol` or `column`. A symbol name is what
  Zed's own agent takes (`SymbolLocator`), since models count columns badly; the column stays for
  a client that has one. Rejected: a column alone; LSP's 0-based UTF-16 positions.
- **D3:** columns in and out are 1-based and counted as `editor_list` counts them (Zed's `Point`,
  bytes within the line), so the editors family speaks one unit. Rejected: characters (a second
  unit beside `editor_list`'s).
- **D4:** a file no editor shows is opened without a tab and registered with its language servers
  for the call, holding the handle until the answer. Rejected: open editors only (the agent would
  need `editor_open`, which takes the user's tab and asks under Ask First).
- **D5:** waits stay inside the server's 30 seconds (L-claude-682): up to 15 seconds for a
  language server to start for the file, then up to 10 seconds for its answer; past either, a
  refusal that says to call again.
- **D6:** one file's diagnostics are pulled afresh first when its server supports pulled
  diagnostics, as Zed's tool does; a project listing answers from what the servers last
  published, with `pending: true` while a server still has updates coming.
- **D7:** every call is listed in Agent Activity: each one hands an agent text from the project's
  files, as `editor_read` does.
- **D8:** Marley's failed-block diagnostics (#623) are left out: they repeat what `terminal_read`
  gives and are not a language server's.
- **D9:** the stand-in language server, not rust-analyzer, proves the tools: its answers are
  fixed and it starts at once, while rust-analyzer's first index can outlast the server's
  30-second wait. The Test phase may also run the client once against rust-analyzer (rustup's,
  found with no download) and record what it saw.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent calls `editor_diagnostics` with a file's path, the system shall answer that file's diagnostics with 1-based positions, severity, message, source and code. | Shot 712-01 |
| REQ-002 | WHEN an agent calls `editor_diagnostics` without a path, the system shall answer the diagnostics of the files with errors or warnings in the caller's project, paged. | Shot 712-01 |
| REQ-003 | WHEN an agent calls `editor_definition` at a symbol, the system shall answer each definition's path, position and line text. | Shot 712-02 |
| REQ-004 | WHEN an agent calls `editor_references` at a symbol, the system shall answer its uses with `total`, `start` and `next`. | Shot 712-03 |
| REQ-005 | WHEN an agent calls `editor_hover` at a symbol, the system shall answer the hover text, redacted, and the range it covers. | Shot 712-04 |
| REQ-006 | WHEN an agent names a file no editor shows, the system shall answer from its language server and open no tab. | Shot 712-05 |
| REQ-007 | IF the path matches a secret glob, lies outside every open project, or the symbol is not on its line, THEN the system shall refuse with `secret_file`, `outside_projects` or `no_symbol` and a next step. | Shot 712-06 |
| REQ-008 | WHILE `marley.agent_control.editors` is `off`, the system shall refuse the four tools with `agent_control_off`. | Shot 712-07 |
| REQ-009 | WHEN any of the four tools answers or refuses, the system shall list the call in Agent Activity. | Shot 712-08 |
| REQ-010 | IF no language server runs for the file within 15 seconds, THEN the system shall refuse with `no_language_server` and say to call again. | The review of the diff |
| REQ-011 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes, every cited seam checked
  again.
- **P2 Code** — `registry.rs` (four rows, four schema arms, the pinned list), `lsp_tools.rs`
  (new, Marley crate), `editor_tools.rs` (routing),
  the guide and the architecture notes; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
