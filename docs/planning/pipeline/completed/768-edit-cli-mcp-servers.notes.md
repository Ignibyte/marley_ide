# Edit each CLI's MCP servers — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-768-edit-cli-mcp-servers.md
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

### Promotion (2026-10-11)
- Moved from `queued/` to `active/` at `f91507f30a`; the BACKLOG row removed; the ticket in
  progress. Brain: consultation 6c60b48b44eb4e6e8e1cc8a3fd29f299, nothing on this seam.
- **Seams checked again** (the code is #720's and #755's, from today; read directly):
  `safe_write::replace_in` (`safe_write.rs:76`, `edit: impl FnOnce(&str) -> anyhow::Result<String>`);
  `mcp_servers.rs`: `open` 58, `Row` 118, `refresh` 178, `render_row` 443 (a free function, no
  listener yet), `claude_json` 632, `Names` 686, `claude_rows_in` 718, `codex_rows_in` 816;
  `agent_notify::configure_codex` 99; `harness_seat::NewSeatModal` (690-1000, the form pattern).
  `shlex` 2.0.1 is a workspace dependency with `split` and `try_join`, not yet one of
  `marley_workbench`'s.

### Design (as built)
- **New `mcp_server_form.rs`** (Marley crate), so `mcp_servers.rs` stays the tab:
  - `Place`: `ClaudeUser`, `ClaudeLocal(folder)`, `ClaudeProject(folder)`, `Codex`; its file and
    its words.
  - `Change`: `Put(Server)` or `Remove`; `Server { kind: Command { command, args } | Url { url },
    pairs: Vec<(name, Pair::Keep | Pair::Set(value))> }`.
  - Writers for `replace_in`: `claude_change(text, place, name, change, seen)` over
    `serde_json::Value` (written back with two-space indentation and the original final newline)
    and `codex_change(text, name, change, seen)` over `toml_edit`. `seen` is the hash of the entry
    as the tab read it (`None` for an add); a different hash, or an added name already there,
    refuses before anything is written. An edit starts from the stored entry, so keys the form
    does not know stay.
  - `McpServerForm`, a `ModalView`: name (fixed when editing), Where (add only), Command or URL,
    the pairs (an existing one is a label, a value editor reading "set; type to replace", and
    Clear; a new one two editors), Add Variable or Add Header, Save. The write runs in
    `background_spawn`; success refreshes the tab, shows a toast and closes; a refusal stays.
- **`mcp_servers.rs`:** `Row` gains `place` and `edit` (command, raw arguments, URL, the entry's
  hash; never a value); the readers fill them; the tab keeps the open folders for Where; Add
  Server under Claude Code and Codex, Edit and Remove on their rows; Remove asks with
  `window.prompt`, then writes.
- **`Cargo.toml`** of `marley_workbench`: `shlex.workspace = true`.

### Design (the draft it came from)
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

## Phase 2 — Code (2026-10-11)
- **Built:** `mcp_server_form.rs` (new: `Place`, `Kind`, `Stored`, `entry_hash`, the JSON and TOML
  writers, `McpServerForm`, `remove`); `mcp_servers.rs` (rows carry `stored`, the readers fill it,
  `codex_row` split out of `codex_rows_in`, the tab keeps `workspace` and `folders`, Add Server,
  Edit and Remove, `open_form`, `claude_json` and `refresh` made `pub(crate)`);
  `marley_workbench.rs` (the module); `Cargo.toml` (`shlex`). The guide's MCP Servers section and
  the in-app card.
- **Deviations from the design:**
  - The form shows stored arguments and a URL masked and keeps them when left as shown (from the
    Test phase, F-claude-768-…); the design had them back as stored.
  - Stored pairs for Codex are `env` and `http_headers` only: the names `env_vars`,
    `env_http_headers` and `bearer_token_env_var` list stay on the row but out of the form.
  - The remove prompt's detail names the file and its backup in one line.
- **The review:**
  - The hash on both sides is of the same parse: compact JSON of the entry's `Value` (read from
    its `RawValue`, written from the whole file's `Value`), or the TOML item's text.
  - `Keep` reads a stored value only inside the writer; no row, form field or log holds one.
  - The form is built inside the workspace's update (`toggle_modal`) and reads no entity of the
    tab; the tab's refresh after a save goes through `update_in` on the tab.
  - A local scope for a folder Claude Code never ran in is refused rather than half-written.
- **Clippy found and fixed:** `missing_const_for_fn` (twice), `single_match_else`, `unused_self`
  (twice, the section and row renderers made associated), `redundant_closure_for_method_calls`,
  `needless_pass_by_value`, `too_many_lines` (`codex_row` split out). `cargo fmt`, then clippy
  again, clean.
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/768-edit-cli-mcp-servers.sh`, under `compositor sway`, exit 0,
  every check passing. Calibrated over three runs (the tab's buttons, the form's fields, the
  prompt's Remove). Shots read:
  - `768-00-tab`: Add Server beside both headings; Edit and Remove on Claude Code's four rows and
    Codex's two; none under Zed.
  - `768-01a-form-open`: "Add a Server to Codex": Name, Command and URL, Command, Arguments, Env
    variables, Add Variable, Save; no Where for Codex.
  - `768-01-add-form`: `lint`, `lint-mcp`, `--stdio`, `LINT_TOKEN` with its value in dots, Clear.
  - `768-02-added`: `lint user on`, `lint-mcp --stdio`, `env: LINT_TOKEN`, and the toast "lint
    saved. Codex reads its servers at its next start." The log: `[mcp_servers.lint]` after
    `old-wiki`, the comment and `search` kept, `config.toml.marley-backup` beside it.
  - `768-03-edit-form`: "Edit notes (Claude Code, user)": `notes-mcp`,
    `--stdio '--api-key=[redacted: secret]'`, `API_TOKEN` and `NOTES_DIR` "set; type to replace"
    with Clear each.
  - `768-04-edited`: `env: API_TOKEN, NOTES_DIR, NOTES_MODE`, the toast; the log: the args
    `["--stdio", "--api-key=not-a-real-key"]`, `API_TOKEN` and `NOTES_MODE` as expected,
    `numStartups` and the float in `tipsHistory` the same, `tracker` still local.
  - `768-05-remove-ask`: "Remove docs from Claude Code's servers (project, repo)?", the backup
    line, Remove and Cancel.
  - `768-06-removed`: `docs` gone, `wiki` left, the toast; `.mcp.json` holds `wiki` alone.
  - `768-07-refused`: the form's status "search changed since the tab read it; Refresh and try
    again"; the outside change kept.
- **Fixes on the way:** the edit form's clear arguments (F-claude-768, a source fix, the build and
  the run again); the prompt's Remove recalibrated after its detail was shortened
  (L-claude-768-a-prompts-buttons-move-with-its-detail-001).
- **The last run** came after the green gate, on a fresh build of that tree: exit 0, 12 checks
  passing, the same shots.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #768); the guide's MCP Servers section ("Editing a CLI's
  servers") and the in-app MCP Servers card; `docs/marley_architecture/marley_workbench.md` (the
  rows' `stored`, and a new "Editing a CLI's MCP servers" section). No Zed path.
- **Knowledge appended:**
  - AD-claude-768-cli-mcp-servers-are-written-one-entry-at-a-time-through-the-safe-write-001;
  - F-claude-768-the-edit-form-showed-a-stored-argument-in-clear-001 with
    PR-claude-768-an-edit-form-masks-what-its-list-masks-001;
  - L-claude-768-a-prompts-buttons-move-with-its-detail-001.
- **Brain:** consultation 6c60b48b44eb4e6e8e1cc8a3fd29f299 closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed.

