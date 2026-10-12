---
pipeline_id: 83bfb2ab-36ad-4312-a36b-d1e8f433c8e4
ticket: docs/planning/tickets/open/TICKET-768-edit-cli-mcp-servers.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: Edit each CLI's MCP servers
type: feature
slice: agents on their own tools; MonoCode findings item 7, the second of two slices (#720 the first)
references:
  - docs/planning/pipeline/completed/720-mcp-manager-and-cli-update-checks.spec.md
  - docs/planning/pipeline/completed/755-three-small-hardening-fixes.spec.md
  - docs/planning/design-notes/finish-line-2026-10-11.md
---

## Title
Add, change and remove Claude Code's and Codex's MCP servers from #720's MCP Servers tab, each
write through #755's safe write, with no secret value ever shown back. Chad, 2026-10-11: "17.)
build now if we can", and the writes follow #755's safe write.

## Scope
### In
- **Buttons:** Add Server under Claude Code and under Codex; Edit and Remove on each of their rows.
- **The form** (a modal over the tab):
  - Name (letters, digits, `-` and `_`); for Claude Code a Where choice: User, Local to an open
    project's folder, or Project (that folder's `.mcp.json`); Codex has the user scope only.
  - Kind: Command (the program and its arguments, the arguments split as a shell would) or URL.
  - Env variables (Command) or headers (URL): name and value pairs, Add Variable and Add Header.
  - Editing: the name and Where are fixed; the command, arguments or URL come back as stored; an
    existing env or header value never does. Its field reads "set; type to replace", it is kept
    unless a new value is typed, and Clear removes the pair.
  - Save writes and closes; a refusal stays in the form with its reason.
- **Remove** asks first ("Remove notes from Claude Code's user scope?"), then writes.
- **The writes**, each through `safe_write::replace_in` with a backup:
  - Codex's `config.toml` through `toml_edit`: the `[mcp_servers.<name>]` table, its comments and
    every other table kept; `enabled` and `bearer_token_env_var` left as they are.
  - `.claude.json` (user scope, `mcpServers`; local scope, `projects["<folder>"].mcpServers`) and
    a folder's `.mcp.json` through `serde_json` with `preserve_order`: only that server's entry
    changes; the file keeps its key order, two-space indentation and final newline.
  - A refusal when the entry changed since the tab read it (or an added name exists now): "notes
    changed since the tab read it; Refresh and try again". Other changes to the file, such as the
    ones Claude Code makes while it runs, do not refuse it.
- After a write: the tab reads the files again, and a toast says the CLI reads it at its next
  start.
- **Docs:** the guide's MCP Servers section and the in-app card; `marley_workbench.md`.

### Out (explicitly deferred)
- Zed's context servers: Zed's own page edits them, and the tab opens it.
- Moving a server between scopes or renaming one (remove, then add).
- Codex's `enabled`, `bearer_token_env_var`, `env_vars`, `env_http_headers`, timeouts and OAuth
  settings: kept as they are, not edited.
- Servers that Claude Code's plugins add.
- Running `claude mcp add-json` or `codex mcp add`: a value would sit in a process's arguments,
  where `ps` shows it.

## Reference (§20)
N/A — Marley-specific: no editor Marley builds on edits other CLIs' MCP configs. Upstream Zed's
MCP Servers page (`crates/settings_ui/src/pages/mcp_servers_page.rs`, `open_mcp_server_form`) is
the behavior kept for the form's shape: a name, a command or a URL, env and headers, Save. The file
formats are the CLIs' own, read from their docs and files (`claude mcp`, `codex mcp`). No Warp
analog.

### Prior art
- **The code we ship:**
  - `marley_workbench::safe_write::replace_in` (#755): link followed, temp with the mode, backup,
    re-read before the rename. Its `edit` closure gets the current text, where the entry check
    runs.
  - `marley_workbench::mcp_servers` (#720): the readers (`claude_rows_in`, `codex_rows_in`), the
    `.claude.json` path, `Names`, the tab and its `refresh`.
  - `agent_notify::configure_codex` (#552): `toml_edit` editing that keeps comments.
  - `harness_seat::NewSeatModal` (#740): the form pattern, single-line editors, choice buttons, a
    status line, `menu::Confirm`.
  - `shlex` (workspace dependency) splits and joins arguments.
  - `serde_json` with `preserve_order` (workspace) keeps a JSON file's key order.
- **Behavior maps:** none for this; the MonoCode findings, item 7.
- **Published material:** Claude Code's MCP docs (the scopes, `.mcp.json`), `claude mcp add-json`
  and `codex mcp add` for the entry shapes. Claude Code 2.1.296 saves `.claude.json`
  read-modify-write under a lock, applying its change to a fresh read (read from the installed
  binary on 2026-10-11), so an entry Marley writes survives Claude Code's next save.

## UI proof
`script/e2e/768-edit-cli-mcp-servers.sh`, under `compositor sway`, with #720's fixtures
(`CLAUDE_CONFIG_DIR`, the runner's `CODEX_HOME`, `.claude.json` with `notes` and `tracker`, the
repository's `.mcp.json` with `docs`, Codex's `search` and `old-wiki`). Each write's file is
printed into the run's log beside the shots.
- `768-01-add-form`: Add Server under Codex, filled: `lint`, Command `lint-mcp --stdio`, env
  `LINT_TOKEN`.
- `768-02-added`: the tab after Save: `lint` under Codex; the log shows `[mcp_servers.lint]` in
  `config.toml` with the file's comment kept and a backup beside it.
- `768-03-edit-form`: Edit on `notes`: the command and arguments as stored, `API_TOKEN` "set; type
  to replace", `NOTES_DIR` likewise.
- `768-04-edited`: an argument changed and Save: the row's new arguments; the log shows
  `API_TOKEN`'s value kept.
- `768-05-remove-ask`: Remove on `docs`: the question.
- `768-06-removed`: `docs` gone; the log shows `.mcp.json` without it.
- `768-07-refused`: the scenario changes `search` in `config.toml` behind the form's back, then
  Save: the form says it changed since the tab read it.

## Locked-In Decisions
- **D1:** the form writes the files itself, through #755's safe write (Chad's rule). Rejected:
  `claude mcp add-json` and `codex mcp add` (values in a process's arguments; one program per
  write).
- **D2:** a conflict is the entry's, not the file's: Claude Code writes `.claude.json` often, so a
  whole-file check would refuse most writes while it runs.
- **D3:** values are write-only: an existing value is never read into the form, only kept or
  replaced (#720's D4 continued).
- **D4:** the name and the scope of an existing server are fixed in the form; moving is remove,
  then add.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user saves a new server for Codex, the system shall add its `[mcp_servers.<name>]` table to `config.toml` and keep the file's other content. | Shots 768-01, 768-02 and the log |
| REQ-002 | WHEN the user saves a new or changed server for Claude Code, the system shall write that entry in the scope chosen and change nothing else in the file. | Shots 768-03, 768-04 and the log |
| REQ-003 | WHEN the form edits a server, the system shall show each existing env and header name without its value, and keep a value the user did not replace. | Shots 768-03, 768-04 and the log |
| REQ-004 | WHEN the user removes a server, the system shall ask first and then remove only that entry. | Shots 768-05, 768-06 and the log |
| REQ-005 | IF the entry changed since the tab read it, THEN the system shall write nothing and say so in the form. | Shot 768-07 |
| REQ-006 | WHEN a write succeeds, the system shall leave a backup beside the file and read the files again. | Shot 768-02 and the log |
| REQ-007 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes, every cited seam checked
  again.
- **P2 Code** — the form and the writers in `mcp_servers.rs` (or a sibling module if it grows past
  the tab); the guide and the architecture note; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
