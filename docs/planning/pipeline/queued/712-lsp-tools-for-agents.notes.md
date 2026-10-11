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
