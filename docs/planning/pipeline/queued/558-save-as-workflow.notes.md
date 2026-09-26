# Save as Workflow: a block's command becomes a task in tasks.json, with its parameters — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-558-save-as-workflow.md
- **Pipeline spec:** 558-save-as-workflow.spec.md

## Phase 1 — Plan
- **Request:** the Warp blocks note (2026-09-25), "Save as Workflow (M)" and recommendation 5,
  with open question 2 (Marley YAML or `tasks.json`). Chad, 2026-09-26: entries in Zed's
  `tasks.json` ("if it works tasks.json it would make sense not to conflict"), Save as Workflow
  writes a task.
- **Classification / tier:** feature, M. A pure module, a modal, a JSONC append, and a prompt
  in the provider Marley already owns. Zed paths: the element's hook if new; possibly a schema
  hunk in `crates/task` if the editor warns on the `marley` key.
- **Recall (§18.3):**
  - F-claude-441-a-task-provider-read-the-workspace-inside-its-update-001: the provider runs
    inside the workspace's update; `RoutedTerminals::spawn` already defers to the window's next
    turn (`routing.rs:113-117`); the parameter prompt opens from there too.
  - F-claude-441-a-rerun-reopened-the-hidden-terminal-panel-001: a task reruns in its last
    terminal; `move_to_center` handles it, and a filled workflow keeps the task's `full_label`,
    so the rerun finds the same terminal.
  - PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001: the button needs
    `command_verified`; a command that output printed is never saved.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: a third hover button
    sits to the left of Copy in the actions row; the 474 scenario's driven tests are in the
    tree, not run (§7), and no e2e scenario clicks a block's buttons by coordinates yet.
  - Brain: no page on workflows (searched 2026-09-26); the gpui-era pipeline #204 wrote the
    original module.
- **Discovery:**
  - `crates/task/src/task_template.rs`: `TaskTemplate` (24 to 81), `TaskTemplates` and
    `FILE_NAME` (141 to 144), `resolve_task` (162), `unknown_variables` (312),
    `substitute_variables_in_str` (375), the unknown-variable rule (407 to 434). `task.rs`:
    `SpawnInTerminal` (42 to 80: `command`, `args`, `env`, `label`, `full_label`, `cwd`, `id`),
    `ResolvedTask` (114), `VariableName` (154).
  - `crates/paths/src/paths.rs`: `tasks_file` (309), `local_tasks_file_relative_path` (505),
    `task_file_name` (522). `crates/project/src/task_inventory.rs`: `TaskSourceKind` (147),
    `list_tasks` (432), `update_file_based_tasks` (762). `project_settings.rs`: the global
    watcher (1475), the local file (1180 to 1191, 1385 to 1395).
  - `crates/settings_json/src/settings_json.rs`: `append_top_level_array_value_in_json_text`
    (508 to 628; makes `[value]` when the text has no array), `infer_json_indent_size` (633),
    `to_pretty_json` (726), `parse_json_with_comments` (753). `crates/settings/src/keymap_file.rs:1098`
    uses the append. `assets/settings/initial_tasks.json` seeds an active "Example task", so a
    new file is not seeded with it (D6's `[\n]`).
  - `crates/tasks_ui/src/modal.rs`: `TasksModal::new` (131), `update_matches` (264),
    `confirm` (390 → `schedule_resolved_task`), `render_footer` (634). `tasks_ui.rs`: `init`
    (101, the `Rerun` handler at 106 to 179), `spawn_task_or_modal` (185). `zed_actions`:
    `Spawn` (737), `Rerun` (770). Bindings at `default-linux.json:730-734`.
  - `crates/workspace/src/tasks.rs`: `schedule_task` (29), `schedule_resolved_task` (64), the
    provider call (131 to 138). `workspace.rs:328` (`TerminalProvider`).
    `crates/marley_workbench/src/routing.rs`: `set_terminal_provider` (49), `spawn` (96 to 130),
    `move_to_center` (135), `run_task` after it.
  - `crates/terminal_view/src/terminal_panel.rs`: `spawn_task` (632), `spawn_in_new_terminal`
    (714), `add_center_terminal` (835), `prepare_task_for_spawn` (1286).
    `crates/project/src/terminals.rs`: `create_terminal_task` (64), `Shell::WithArguments` (227).
  - `crates/terminal_view/src/terminal_element.rs`: `marley_block` (2320), the actions row
    (2366 to 2377). `crates/marley_terminal/src/anchored.rs:38-58`, `block.rs:46-54`
    (`PromptInfo`). `blocks.rs:63` (`focused_terminal`).
  - Modals: `git_ui.rs:442-528` (`RenameBranchModal`: `Editor::single_line`, `cancel`,
    `confirm`, `ModalView`, `key_context`), `new_process_modal.rs:435-458`
    (`save_debug_scenario`), `agents.rs:233-289` (`NewAgentPicker` through `toggle_modal`).
  - The gpui-era module at `/srv/stacks/marley/crates/marley_app/src/workflows.rs`: `Workflow`
    (11 to 22), `substitute` (35 to 58), `params_of` (63 to 79), `ParamPrompt` (101 to 157).
    It never existed in this repository's history.
- **Decisions:** D1 to D6 in the spec.

### Design
- **`marley_terminal::workflow`** (pure): `substitute(template, values) -> Result<String,
  MissingParam>` and `params_of(template) -> Vec<String>` from the gpui-era module, copyright
  Marley's; `guess(command: &str, branch: Option<&str>, exists: impl Fn(&str) -> bool) ->
  Guessed { command: String, parameters: Vec<Parameter { name, default }> }` by the spec's rules
  (`shlex::split` for words; quoted tokens kept quoted around the placeholder); `is_name` (Warp's
  character rule). `Workflow { label, command, cwd, parameters }` and `to_task_json(&self) ->
  serde_json::Value`.
- **The editor** (`marley_workbench::workflows`, new): `WorkflowEditor: ModalView` with two
  single-line editors (name, command), a parameters list re-derived from the command's text on
  each edit (rows with a default editor and a description editor), a Where toggle, Save and
  Cancel; key context `MarleyWorkflowEditor`, `enter` → `marley::SaveWorkflow`, `escape` →
  `menu::Cancel`. `marley::SaveAsWorkflow` (palette) takes the newest block with a row in view of
  the focused terminal; the block button (`MarleyBlockExtras::buttons`) takes its block. The
  `exists` check runs `std::fs::metadata` on at most the command's tokens under `prompt.pwd`.
- **The writer.** `write_workflow_in(file: &Path, workflow) -> io::Result<()>`: read the text
  (or `"// Tasks and Marley workflows. Zed's tasks.json format; Save as Workflow appends here.\n[\n]\n"`
  when missing, after `create_dir_all` on `.zed/`), refuse a duplicate label (`parse_json_with_comments`
  over the array), `append_top_level_array_value_in_json_text` with the inferred indent, and an
  atomic write through the project's `Fs` off the main thread; a toast with the file's path, or
  the error.
- **The run.** In `RoutedTerminals::spawn`, before `run_task`: `params_of(&task.command)` (and
  each of `task.args`); when non-empty, the defaults: the session's last values for that
  `full_label` (a global map), else the file's `marley.parameters` (the project's `.zed/tasks.json`
  then the global file, by `label`), else empty; a `ParameterPrompt: ModalView` with one
  single-line editor per name, Enter runs with `substitute`, Escape resolves the spawn's task with
  `None`. The filled `SpawnInTerminal` keeps its `id` and `full_label`.
- **File manifest.** Marley crates: `crates/marley_terminal/src/{workflow.rs (new),
  marley_terminal.rs}`; `crates/marley_workbench/src/{workflows.rs (new), routing.rs,
  marley_workbench.rs}`, `crates/marley_workbench/{Cargo.toml, keymap.json}`. Zed paths:
  `crates/terminal_view/src/terminal_element.rs` (the hook, if new); `crates/task/src/task_template.rs`
  only if the schema needs the `marley` key (a `schemars` attribute on an ignored field).
  Scripts: `script/e2e/558-save-as-workflow.sh`.
- **Ledger rows.** `docs/marley/zed-touchpoints.md`: the `terminal_element.rs` row (the hook),
  and a `crates/task/src/task_template.rs` row only in the schema case.

### E2E plan
`setup` seeds `.zed/tasks.json` with a comment and one task ("hello", `echo hello`) so the
append has something to keep; the branch is `feature/one`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | type `python3 -m http.server 8123 --directory web`, Return; Ctrl-C; `pointer_to` the block's first row | `558-01-button` |
| REQ-002 | click Save as Workflow | `558-02-editor`: `{{port}}` and `{{path}}`, defaults `8123` and `web` |
| REQ-003 | select the name, type `serve`, Return | `558-03-saved`: the toast; the log: `cat .zed/tasks.json` holds the comment, "hello", then "serve" with `marley.parameters` |
| REQ-004 | `alt-shift-t` | `558-04-picker`: "serve" listed |
| REQ-005 | Return on "serve" | `558-05-parameters`: two fields with the defaults |
| REQ-006 | Ctrl-A in the port field, type `8124`, Return; settle | `558-06-ran`: a center task tab serving; the log: `ss -ltn` shows 8124 |
| REQ-007 | type `git status`, Return; save it as "status" with no guesses; run it from the picker | `558-07-no-parameters`: the task's output, no prompt |
| REQ-008 | `ctrl-alt-r` | `558-08-rerun`: the prompt with `8124` |
| REQ-009 | Escape in the prompt; Escape in a second editor | the log: the file's hash unchanged; `ss -ltn` shows no new port |

Not reachable by a scenario: the global file (the profile copy has its own config directory, so
a global save is the same code on another path; Test may add one step writing there and reading
it back in the log).

### Risks
- The `marley` key may draw a schema warning in the editor when the user opens `tasks.json`
  (Zed's schema for the file comes from `TaskTemplates::generate_json_schema`); a `schemars`
  attribute on an ignored field would need a `task` hunk. The Plan phase checks it first; the
  fallback of hiding the metadata in `env` is worse and is not taken.
- A task terminal loads no shell integration (`terminal.rs`, the `task.is_none()` gate), so a
  workflow's output is not a block until T4; the scenario's `558-06-ran` shows a plain task tab.
- Zed's picker resolves tasks with the worktree's context; a `cwd` outside every worktree is
  kept as written and Zed spawns there.
- The guesses can be wrong (a version number, a commit hash as `number`); the editor shows each
  one, and undoing is deleting the braces.

## Phase 2 — Code
- Not started.

## Phase 3 — Test
- Not started.

## Phase 4 — Complete
- Not started.
