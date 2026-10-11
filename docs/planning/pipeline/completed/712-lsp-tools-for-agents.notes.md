# LSP tools for agents — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-712-lsp-tools-for-agents.md
- **Pipeline spec:** 712-lsp-tools-for-agents.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-11)
- **Request:** the 2026-10-09 intake audit left the LSP tools of `mcp-first-class-control-plane.md`
  unbuilt. Chad, 2026-10-11, approving it from the list: "6.) yes".
- **Classification:** feature, Marley's MCP server. Marley crates only; no Zed path.
- **Recall (§18.3):**
  - #704 (completed): the editors family, `admit`, the secret globs, `editor_read`'s page and
    redaction, the `prompt_*` rename. Its lesson L-claude-704-edits-through-bash-skip-the-phase-hook:
    edits go through Write and Edit after `/pipeline:code`.
  - #653 (completed): `getDiagnostics` from `diagnostic_summaries` and `open_buffer`, and the
    stand-in language server (L-claude-653-a-stand-in-language-server-by-its-binary-path-001:
    `lsp.rust-analyzer.binary.path` plus `session.trust_all_worktrees: true`). Its IDE rows ship
    untested, so the link is off on every Claude Code today.
  - #680 (completed): 12,000-byte pages, `{result, code, reason, next_steps}` refusals.
  - L-claude-682: a tool answers inside the server's 30 seconds or the server answers
    `timed_out` while the app still works.
  - L-claude-649: a refusal's reason never repeats the tool's name.
  - Prevention rules on LSP positions (`docs/planning/knowledge/prevention-rules.md`: a position
    an external service computed holds only for the state it was computed from; an LSP uri
    compared to a stored path must be canonicalized first). Both apply: answer positions from the
    snapshot the request used, and match a location's path to a project by canonical path.
- **Discovery (Explore, 2026-10-11):**
  - `crates/marley_workbench/src/editor_tools.rs`: `answer` 38-61 (routing by tool), `after_admit`
    64-71, `open_editors` 83-112, `secret_globs` 176-188, `not_secret` 191-204, `read` 207-274,
    `editor_named` 277-300, `open` 470+, `project_holding` 565-586.
  - `crates/marley_workbench/src/agent_control.rs`: `Area` 34-42, `Level` 85-92, `admit`
    161-227 (Off refuses reads too, 169-178; `Level::Read` never asks, 184-187).
  - `crates/marley_workbench/src/agent_activity.rs`: `log` 165-188 (no refusal), `gate` 147-161
    (write tier only).
  - `crates/marley_workbench/src/mcp.rs`: `editor_` prefix to `editor_tools::answer` 541-544;
    `MAX_READ_LINES` 53, `PAGE_BYTES` 61, `for_agents` 505-513, `agent_redactor` 486-492,
    `page_from` 1203-1252.
  - `crates/marley_mcp/src/registry.rs`: editor rows 410-454, `editor_schemas` 2102-2176 whose
    `_` arm (2148) returns `editor_open`'s schema, so each new verb needs its own arm; the
    pinned list 2565-2638 and its count, 54, at 2629.
  - `crates/marley_workbench/src/claude_ide.rs`: `diagnostics` 718-762, `file_diagnostics`
    765-804, `MAX_DIAGNOSTIC_FILES` 49.
  - `crates/marley_workbench/src/failures.rs:60, 199-269`: failed blocks published as
    diagnostics with source `marley`.
  - Zed: see the spec's prior art. The language servers on the box: rust-analyzer (rustup),
    clangd, taplo, intelephense.

### Design
- **New `crates/marley_workbench/src/lsp_tools.rs`** (Marley crate):
  - `answer(call, cx)` for the four tools; `editor_tools::answer` routes their names here, after
    `agent_activity::log` and `admit(Area::Editors, Level::Read)`.
  - `resolve(arguments, cx) -> Task<Result<Target, Refusal>>`: the path checked (absolute,
    `project_holding`, `not_secret`, the file there), the buffer opened with
    `Project::open_buffer`, a position from `symbol` (searched on the line, after Zed's
    `SymbolLocator`, written anew) or `column`, clipped to the line. `Target` holds the buffer,
    the anchor and the `OpenLspBufferHandle`.
  - `with_server(target, cx)`: `register_buffer_with_language_servers`; when
    `language_servers_for_local_buffer` is empty, wait for `LanguageServerAdded` or the buffer's
    registration (`Project` event `LanguageServerBufferRegistered`) up to 15 seconds, else
    `no_language_server`.
  - Each request (`definitions`, `references`, `hover`) raced against a 10-second timer
    (`cx.background_executor().timer`), else `lsp_timeout`.
  - `location_json(location, cx)`: path (canonical), 1-based line and column from the target
    buffer's snapshot, the line's text trimmed to 200 characters through `for_agents`, or
    `[secret file]`.
  - Diagnostics: one file through `claude_ide::file_diagnostics` (made `pub(crate)`), after
    `pull_diagnostics_for_buffer` when the server supports it; the project listing from
    `diagnostic_summaries(false, cx)` filtered to the chosen projects' worktrees, each file opened
    and read; entries with source `marley` dropped; secret files counted in
    `hidden_secret_files`.
  - Paging: a cut by entries (200) and bytes (12,000), `start`/`next`, written beside
    `mcp::page_from` rather than through it, since it pages lines of one text.
- **`crates/marley_workbench/src/editor_tools.rs`:** the four names routed to `lsp_tools`.
- **`crates/marley_workbench/src/claude_ide.rs`:** `file_diagnostics` `pub(crate)`.
- **`crates/marley_mcp/src/registry.rs`:** four `Family::Editor` rows (`diagnostics`,
  `definition`, `references`, `hover`, `Tier::Read`, no grant) with descriptions that say what
  a position is and that answers are paged; four explicit schema arms; the pinned list and count
  (58) updated so the tree builds.
- **Docs:** the guide's "Zed's editors for agents"; `docs/marley_architecture/marley_mcp.md` (the
  tool list) and `marley_workbench.md` (the module).
- **File manifest:** `lsp_tools.rs` (new), `editor_tools.rs`, `claude_ide.rs`,
  `marley_workbench.rs` (the module line) in `marley_workbench`; `registry.rs` in `marley_mcp`;
  the guide and two architecture notes; the scenario. No Zed path, so no touchpoint row.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001, 002 | Opens main.rs left, a terminal right; runs `lsp-ask diag src/main.rs` and `lsp-ask diag` | 712-01-diagnostics |
| 003 | `lsp-ask def src/main.rs 3 answer` | 712-02-definition |
| 004 | `lsp-ask refs src/main.rs 2 answer` | 712-03-references |
| 005 | `lsp-ask hover src/main.rs 2 answer` | 712-04-hover |
| 006 | `lsp-ask def src/lib.rs 1 helper` with lib.rs unopened | 712-05-no-tab |
| 007 | `lsp-ask hover .env 1 API_TOKEN`, `lsp-ask hover /etc/hostname 1 x`, `lsp-ask def src/main.rs 3 nowhere` | 712-06-refused |
| 008 | Sets `marley.agent_control.editors` to `off` through the settings file, then `lsp-ask hover …` | 712-07-off |
| 009 | `marley: open agent activity` | 712-08-activity |
| 010 | — | The review of the diff (the wait and its refusal) |
| 011 | — | The gate |

The stand-in answers `initialize` with `definitionProvider`, `referencesProvider` and
`hoverProvider`, publishes the error on `didOpen` of main.rs, and answers each request from fixed
positions in the fixture. Each answer is also saved to the run's folder for the scenario's
`expect` lines.

### Risks
- **A server still starting:** a cold server may take longer than 15 seconds; the refusal says
  to call again, and the second call finds it running.
- **Registration for one call:** dropping the handle unregisters the buffer and sends `didClose`
  when no editor holds it. A burst of calls on one file opens and closes it each time; acceptable
  for reads, and cheaper than holding buffers no one shows.
- **Byte columns:** a line with multi-byte characters gives a column an agent counting
  characters would not; `symbol` avoids it, and the descriptions say so.
- **Locations outside the projects** (a library's source under `~/.rustup`): answered with their
  line, since a language server points there for a reason; a secret glob still hides the text.
- **Diagnostics never published:** a server that publishes only for open files answers nothing
  for the rest of the project; `pending` and the descriptions say the listing is what the
  servers have reported.

### Promotion (2026-10-11)
- Moved from `queued/` to `active/`; the BACKLOG row removed; the ticket in progress. Brain:
  consultation c1f96806519b415f9fde76ffce8de1d8, nothing on this seam.
- **Seams checked again** at `0bf2e8a89d` (an Explore pass over all eleven):
  - `editor_tools.rs`: `answer(mut call: AppCall, cx: &App)` (38) routes exact names and answers
    through `call.answer(..)`, often from a spawned task; `agent_activity::log(&mut call, cx)` at
    44 before `after_admit` (64); `not_secret(path, cx) -> Result<(), Refusal>` (191, the file name
    against the globs); `read` (207) pages through `mcp::page_from` and redacts through
    `mcp::for_agents`; `project_holding(path, cx)` (565) gives the window and the workspace whose
    visible worktree holds a path. `Refusal::new(code, reason).next(step)`
    (`marley_mcp.rs:277-303`); `ToolAnswer { structured, text, image }` (255).
  - `agent_control::admit(&call, Area::Editors, Level::Read, what, cx)` (161): Off refuses
    `agent_control_off`, reads never ask.
  - `mcp.rs`: the `editor_` prefix to `editor_tools::answer` (541-544); `PAGE_BYTES` 12,000 (61);
    `agent_redactor` (486), `for_agents` (505). `browser_tools::caller_scope` (139) finds the
    caller's workspace by its terminal, then its project, then its cwd.
  - `registry.rs`: editor rows 412-456; `editor_schemas` (2108) whose `_` arm (2154) is
    `editor_open`'s, so each new verb needs its own arm; the pinned names 2577-2633 and
    `assert_eq!(registry().len(), 54)` at 2635. Outside clients get no editor tool
    (`clients.rs:12,26`).
  - Zed: `Project::open_buffer` (3290), `register_buffer_with_language_servers` (3320, an
    `OpenLspBufferHandle` that must be held), `definitions` (4416), `references` (4501), `hover`
    (4654, `&self`, no `Result`), `diagnostic_summaries` (5368), `find_project_path` (5419);
    `Event::LanguageServerBufferRegistered { server_id, buffer_id, .. }` (346);
    `LspStore::language_servers_for_local_buffer` (9871) and `pull_diagnostics_for_buffer` (9028).
    The three requests go only to a server whose capabilities name the provider.
  - `failures.rs`: Marley's failed-block diagnostics carry `source: "marley"` from server
    `BLOCK_DIAGNOSTICS` (60, 255).
  - #653's stand-in language server (`653-…sh:260-302`) answers `initialize` with
    `textDocumentSync` only and publishes one error on `didOpen` of `src/main.rs`; #704's
    scenario calls tools through the plugin's bridge with a Python client (`write_client`).
- **What differs from the draft, and where the design takes it:**
  - **`editor_read` opens no buffer;** it reads open editors only. The new tools open the file's
    buffer through `Project::open_buffer` in the workspace `project_holding` gives, which opens no
    tab, and register it with its servers for the call.
  - **`claude_ide::file_diagnostics` is not shared:** it answers the IDE's 0-based UTF-16
    positions, and these tools speak `editor_list`'s 1-based line and byte column (D3). The
    tools read `diagnostic_groups` themselves, the primary entry of each group, as Zed's own
    diagnostics tool and #653 both do; `claude_ide.rs` is not touched.
  - **A running server** is asked of the `LspStore` (`language_servers_for_local_buffer`), and the
    wait subscribes to the project's `LanguageServerBufferRegistered` for the buffer's id.
  - **The project listing** takes the caller's workspace through `browser_tools::caller_scope`,
    else `project` by name, else every open project.
  - **The stand-in** gains `definitionProvider`, `referencesProvider` and `hoverProvider` and
    fixed answers; the scenario writes its own copy, as #653's did.

## Phase 2 — Code
- **Built.**
  - `registry.rs` (`marley_mcp`): four `Family::Editor` rows at `Tier::Read` (`diagnostics`,
    `definition`, `references`, `hover`) after `save`; `editor_lsp_schemas`,
    `position_properties`, `page_start` and `editor_diagnostics_schemas` behind their own
    `editor_schemas` arm; the pinned names and the count, 58.
  - `lsp_tools.rs` (new, `marley_workbench`): `answer` (logged, admitted at `Level::Read`),
    `open` (the checks, `open_buffer`, registration held for the call, the 15 s server wait),
    `at_position` and `point_in` (a symbol's first byte offset, or a byte column), `definition`,
    `references`, `hover` (each raced against 10 s), `file_diagnostics` (a pull, then up to 3 s
    for a fresh file's first ones), `project_diagnostics` (`diagnostic_summaries` over the named,
    the caller's or every local project, secret files hidden and counted), `entries_of` (primary
    entries, Marley's own left out), `location_json`, `range_json`, `page_of`, `cut_to`.
  - `editor_tools.rs`: the four names routed; `secret_globs`, `not_secret` and
    `project_holding` made `pub(crate)`. `mcp.rs`: `all_workspaces`, `PAGE_BYTES` `pub(crate)`.
    `browser_tools.rs`: `caller_workspace`, which `caller_scope` now uses.
- **Deviations from the design:** `claude_ide::file_diagnostics` is not shared (its units are
  the IDE protocol's, the promotion's finding); the project listing carries no `pending` flag,
  since Zed exposes a server's publishing as no per-file state; the server wait polls the
  `LspStore` every 200 ms rather than subscribing to `LanguageServerBufferRegistered`, which reads
  the same state with less to tear down.
- **Review against the criteria.** The calls run in a spawned task, outside any window's update,
  so `all_workspaces` and `project_holding` see every window (PR-claude-596). The registration
  handle lives until the answer is built, then unregisters a file no editor holds. Every wait
  fits the MCP server's 30 s. A path is checked against the globs before it is opened, and a
  location in a secret file keeps its path and line only. Clippy found `too_many_lines` in the
  schemas (split in three), a long first doc paragraph in `editor_tools.rs`, and two
  `needless_pass_by_ref_mut`; rustdoc on the two crates was run before the gate.
- **Gate:** `script/gates.sh --diff` green on its first run, every gate and the receipt.

## Phase 3 — Test
- **Scenario:** `script/e2e/712-lsp-tools-for-agents.sh` under `compositor sway`: a Rust crate
  fixture, a stand-in language server advertising definition, references and hover (#653's,
  extended), and `lsp-ask` on the terminal's PATH calling the tools through the plugin's bridge
  from a Marley terminal. Nine checks, all pass.
- **Shots:**
  - `712-01-diagnostics` (REQ-001): `main.rs` on the left with `missing` underlined; the answer
    `…/src/main.rs:7:5: error: cannot find value 'missing' in this scope`, `total=1`.
  - `712-01b-project-diagnostics` (REQ-002): the caller's project, the same entry.
  - `712-02-definition` (REQ-003): `…/src/main.rs:1:4: fn answer() -> i32 {`.
  - `712-03-references` (REQ-004): `5:17` and `6:20` with their lines, `total=2`.
  - `712-04-hover` (REQ-005): the hover's Markdown and "The answer, always 42.", `cut=False`.
  - `712-05-no-tab` (REQ-006): `…/src/lib.rs:1:8: pub fn helper() -> u8 {`; no `lib.rs` tab.
  - `712-06-refused` (REQ-007): `secret_file` for `.env`, `outside_projects` for `/etc/hostname`,
    `no_symbol` with "let total = answer();".
  - `712-07-off` (REQ-008): `agent_control_off`.
  - `712-08-activity` (REQ-009): every call listed, done or refused with its code.
- **Not driven:** REQ-010 (a server that never starts; review), real rust-analyzer (D9 left it
  optional; not run).
- **Calibration:** the pane split left a spare terminal in front on the left; the scenario brings
  `main.rs` forward there.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`; `docs/marley/guide.md` (Zed's editors for agents); the in-app
  guide's tool table; `docs/marley_architecture/marley_mcp.md` (the editor family) and
  `marley_workbench.md` (`lsp_tools.rs`). No Zed path, so no ledger row.
- **Knowledge:** AD-claude-712-agents-ask-zeds-language-servers-through-four-editor-reads-001. No
  F: review and test found no bug.
- **Brain:** consultation c1f96806519b415f9fde76ffce8de1d8 closed with a decision (follow-up
  2026-11-11).
