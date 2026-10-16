# The agent editor when the data folder has a space — Notes

- **Local ticket doc:** [TICKET-790](../../tickets/open/TICKET-790-the-agent-editor-with-a-space-in-the-data-folder.md)
- **Pipeline spec:** 790-the-agent-editor-with-a-space-in-the-data-folder.spec.md

## Phase 1 — Plan
- **Request:** seen in #787's check on the Mac. Every start logs "agent editor: Marley's data
  directory has a space in its path …, so agents keep the Rich Input overlay".
- **Classification / tier:** bug, macOS. Marley crate only (`marley_workbench`).
- **Recall (§18.3):**
  - AD-649 (`architecture-decisions.md:3902`): the helper, off by default, is given as `VISUAL`
    and `EDITOR` to the terminals Marley opens for agents.
  - #755: `marley-edit` sends the terminal's token, which comes from the terminal's environment
    whatever the helper's path is.
  - The brain (consultation `94f0dcc7b2d948fc9a30cdb78e5b8050`): nothing more.
- **Discovery:**
  - `agent_editor::set_helper` (`agent_editor.rs:92-102`) refuses a path with whitespace.
  - `mcp::offer_agent_editor` (`mcp.rs:378-393`) writes the helper with `write_program_in` off the
    main thread, then calls `set_helper`.
  - `marley-edit:50-54` finds the endpoint through `realpath(__file__)`.
- **Decisions:** D1 to D3 in the spec.

### Design
- In `agent_editor.rs`, `pub(crate) fn spaceless_in(state_dir: &Path, helper: &Path) ->
  io::Result<PathBuf>`:
  - `helper` when its path has no whitespace;
  - else `state_dir/agent-editor/<hash>/marley-edit`, made by creating the folder, removing an
    old link, then `std::os::unix::fs::symlink`. On non-Unix the helper is returned as it is.
- `offer_agent_editor` runs `spaceless_in(paths::state_dir(), …)` in the same background task,
  after the write. A failure to link is logged, and the original path goes on, so `set_helper`
  warns as before. `set_helper` keeps its check, which still covers a state folder with a space.
- **File manifest:** `crates/marley_workbench/src/agent_editor.rs` and
  `crates/marley_workbench/src/mcp.rs` (Marley crate).

### Visual check plan
| REQ | What runs on the Mac | What shows it |
|---|---|---|
| 001 | install, then restart the test Marley | `grep -c "space in its path"` over the new log lines: 0 |
| 002 | — | `ls -l ~/.local/state/Marley/agent-editor/*/marley-edit`: a link to `…/Application Support/Marley/mcp/marley-edit` |
| 003 | `terminal_run "<link> ~/ports-788/note-790.md"` in the `ports-788` terminal (`MARLEY_TERMINAL_ID` and the token are in its environment) | `790-mac-agent-editor.png`: a `note-790.md` tab in ports-788's pane |
| 004 | Linux: `just shot` | Marley starts. The Linux data folder has no space, so the helper path is unchanged |

### Risks
- **The tab waits for its close.** `terminal_run` returns while the helper still waits, after its
  20 s. The tab is closed afterwards with an editor tool or left open; the helper exits with it.

## Phase 2 — Code
- **What was built:**
  - `agent_editor::has_whitespace`, which `set_helper` now shares;
  - `agent_editor::spaceless_in(state_dir, helper)`: the helper when its path has no whitespace,
    else `state_dir/agent-editor/<016x of DefaultHasher over the path>/marley-edit`, remade as a
    symlink on each start;
  - `mcp::offer_agent_editor` writes the helper, then calls `spaceless_in(paths::state_dir(), …)`
    in the same background task. A link that fails is logged, and the helper's own path goes on
    to `set_helper`, which turns it down as before.
- **Deviation:** no `cfg(unix)` branches. `marley_workbench` already uses `std::os::unix` without
  them (`codex_server.rs`), and the non-Unix arm would have needed a `let _ =`.
- **Review:**
  - REQ-002: the link's folder is named for the helper's path, so two data folders don't share
    one. `marley-edit` resolves `realpath(__file__)`, so its endpoint is the data folder's.
  - REQ-004: with no whitespace the helper itself is returned, which is Linux's case.
  - No `unwrap`, and no `let _ =`.
- **Checks:** box clippy on `marley_workbench` green; **GATE GREEN [diff]**, 17 of 17. Mac clippy
  `-D warnings` on `marley_workbench` green, then `just install` there.

## Phase 3 — Test
- **The run, on the Mac.** The install had this diff at `ae6b75d8af`. The test Marley was quit
  and reopened, and the log's line count kept as the start.
- **REQ-001:** 0 lines with "space in its path" since the restart, where every start had one
  before.
- **REQ-002:** `~/.local/state/Marley/agent-editor/6e7e3d0aaf054b20/marley-edit ->
  /Users/…/Library/Application Support/Marley/mcp/marley-edit`, a path without whitespace that
  leads to the helper.
- **REQ-003:** in the `ports-788` terminal, which carries `MARLEY_TERMINAL_ID` and its token,
  `terminal_run "<link> ~/ports-788/note-790.md"` answered block 3, still running, output
  "marley-edit: note-790.md is open in a Marley tab; closing the tab hands it back".
  - `editor_list`: `note-790.md` in project `ports-788`, active.
  - `790-mac-agent-editor.png`: the `note-790.md` tab in the pane, "Files 1 / note-7…" in the
    rail, and the terminal's row "running" while the helper waits for the tab to close.

  So the link reaches this Marley's endpoint, through `realpath`.
- **REQ-004:** `just build`, then `COMPOSITOR=sway just shot 790-linux-unchanged`.
  `790-linux-unchanged.png` is Marley as before, and the run's log has no "space in its path" or
  "link without spaces" line: the data folder has no whitespace, so the helper's own path is
  published.

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md`, Fixed: "Agent prompts in a tab on a Mac".
  - `docs/marley/guide.md`, On a Mac: an agent prompts item, and the agent editor off the lacks
    list.
  - `docs/marley/three-prong-plan.md`: the macOS bullet.
  - `docs/marley_architecture/marley_workbench.md`: `spaceless_in`.
- **Ledger:**
  - F-claude-790-the-agent-editor-never-turned-on-on-a-mac-001
  - L-claude-790-macos-paths-hold-a-space-by-default-001
- **Brain:** consultation `94f0dcc7b2d948fc9a30cdb78e5b8050` closed by
  `decisions/on-a-mac-the-agent-editor-is-a-link-in-marleys-state-folder`.

