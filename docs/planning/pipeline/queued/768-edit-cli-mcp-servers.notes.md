# Edit each CLI's MCP servers — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-768-edit-cli-mcp-servers.md
- **Pipeline spec:** 768-edit-cli-mcp-servers.spec.md

## Phase 1 — Plan (queued, 2026-10-11)
- **Request:** the second slice of the MCP manager, drafted in #720's notes ("The second slice")
  and split out as #768 when Chad asked to spec the survey list. His rule: writes follow #755's
  safe write.
- **Classification:** feature. Marley crate only (`marley_workbench`); no Zed path.
- **Recall (§18.3):**
  - #720 (this tab): the readers, `Names` (keys kept, values skipped), the `.claude.json` place
    (L-claude-720-claude-json-sits-in-the-home-folder-…), F-claude-720 (look projects up after
    the action's update).
  - #755: `safe_write::replace_in` and its backup; L-claude-755 on scenarios.
  - #552: `configure_codex` edits `config.toml` with `toml_edit` and keeps comments.
- **The open question of #720's draft, settled:** would a running Claude Code write its own copy
  over an outside change to `.claude.json`? Read from Claude Code 2.1.296's binary: its
  `saveGlobalConfig` and `saveCurrentProjectConfig` take a lock and apply their change to the
  config read under it ("fresh disk on a clean read"), merging `projects`. An entry Marley renames
  into place between two of its saves is kept. The window left is Claude Code's own
  read-to-write span under its lock; the safe write's re-read narrows Marley's side. Default kept:
  the safe write.

### Design (draft; checked again at promotion)
- **Where:** the form and the writers beside the tab, in a new `mcp_server_form.rs` if
  `mcp_servers.rs` would pass about 1,200 lines.
- **`Row` gains** what an edit needs and the tab never draws: the command and the raw arguments
  (the tab still draws them masked), the URL as stored, and a fingerprint of the entry as read
  (its JSON or TOML text hashed), so a write can refuse when the entry moved.
- **Writers**, each `fn(&str) -> anyhow::Result<String>` for `replace_in`:
  - `codex_entry(text, name, Change, seen)`: `toml_edit`, the table created as a standard table
    with `env` as an inline table, or updated in place (values of kept names untouched).
  - `claude_entry(text, Where, name, Change, seen)`: `serde_json::Value` with `preserve_order`,
    the entry at `mcpServers`, `projects[folder].mcpServers`, or the `.mcp.json`'s `mcpServers`;
    written with two-space indentation and the original final newline.
  - `Change`: `Put { kind, command, args | url, pairs: Vec<(name, Value::Keep | Value::Set(s))> }`
    or `Remove`.
- **The form** (`McpServerForm`, a `ModalView` on the tab's workspace): editors for the name,
  command, arguments, URL; a row of editors per pair; buttons for Where and Kind; Save
  (`menu::Confirm`) runs the write in `background_spawn` and, on success, refreshes the tab and
  closes; on a refusal it keeps the reason in its status line.
- **Remove:** `window.prompt(PromptLevel::Warning, …, ["Remove", "Cancel"])`, then the write.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | Add Server under Codex, fills the form, Save; prints `config.toml` and lists the folder | 768-01-add-form, 768-02-added |
| 002, 003 | Edit on `notes`, changes an argument, Save; prints the `notes` entry of `.claude.json` | 768-03-edit-form, 768-04-edited |
| 004 | Remove on `docs`, the question, Remove; prints `.mcp.json` | 768-05-remove-ask, 768-06-removed |
| 005 | Opens Edit on `search`, rewrites its args in `config.toml` from the script, Save | 768-07-refused |
| 006 | The backups listed after the first write | 768-02-added |
| 007 | — | The gate |

### Risks
- **JSON round trip, checked 2026-10-11:** the box's `~/.claude.json` holds 1,678 integers (none
  past 2^53) and 492 floats, two-space indentation and no final newline. `serde_json` (no
  `arbitrary_precision`) and JavaScript both print a float's shortest round-trip form, so every
  value reads back the same; only an exponent's spelling could differ (`1e21` against `1e+21`),
  and Claude Code rewrites the file in its own form at its next save. Re-serializing the whole
  file is kept; the writer keeps the indentation and whether a final newline was there.
- **Clicks inside a modal** under sway: L-claude-660 (a missed click outside a Zed modal closes
  it silently).
