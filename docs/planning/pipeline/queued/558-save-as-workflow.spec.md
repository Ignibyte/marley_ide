---
pipeline_id: 3ba05244-3602-4e62-a889-7c173e6230a1
ticket: docs/planning/tickets/open/TICKET-558-save-as-workflow.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Save as Workflow: a block's command becomes a task in tasks.json, with its parameters"
type: feature
slice: prong 1 T4 (tasks and runnables) with T1 (block actions); the Warp blocks note, recommendation 5
references: [docs/planning/design-notes/warp-blocks-and-natural-language-2026-09-25.md, docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md, docs/planning/pipeline/completed/474-block-hover-actions.spec.md, docs/planning/pipeline/completed/441-marley-terminal-routing.spec.md]
---

## Title
Warp's Save as Workflow keeps a command with named arguments for later. Chad chose Zed's
`tasks.json` over a Marley file so nothing conflicts: a workflow is a task, written by Marley with
`{{name}}` parameters that Zed's resolver leaves alone and Marley fills when the task runs from
Zed's own picker.

## Scope
### In
- **Reaching it.** A "Save as Workflow" button on a hovered block, beside Copy and Rerun, for a
  verified non-empty command (through the `MarleyBlockExtras` hook, #556 D7), and
  `marley::SaveAsWorkflow` in the palette, on the newest block with a row in view (#528's rule).
- **The editor**, a modal: Name (single line, prefilled with the command's first two words),
  Command (single line, prefilled with the block's command, the guessed tokens replaced by
  `{{name}}`), a Parameters list derived live from the `{{…}}` in the command, each row a Default
  (prefilled with the token it replaced) and a Description, and Where: this project
  (`<root>/.zed/tasks.json`) or global (`paths::tasks_file()`). Enter or Save writes; Escape
  cancels. A name that a task in that file already uses is refused with the reason.
- **The guesses**, pure rules run once when the editor opens (`marley_terminal::workflow`): a
  token that is a number (`port` when it follows `-p`, `--port` or `:`, else `number`), a URL
  (`url`), the block's git branch (`branch`, from `PromptInfo::git_branch`), and a token that names
  a path under the block's working directory that exists (`path`); repeats get `2`, `3`. Tokens
  after a `-` flag are fair game; the first word never is.
- **The file.** The task is appended to the JSONC array with
  `settings_json::append_top_level_array_value_in_json_text`, comments and indentation kept; a
  missing file starts as `[\n]` under a comment naming Marley, and a missing `.zed/` is made.
  The task: `label`, `command` (the parameterized text), `cwd` (the block's working directory,
  or `$ZED_WORKTREE_ROOT` when it is the root), and `marley: { parameters: { <name>: { default,
  description } } }`. Zed's `TaskTemplate` names no `deny_unknown_fields`, so it loads the task
  and ignores `marley`; the file watcher reloads the inventory, and the task shows in Zed's picker
  at once.
- **Running.** Zed's picker, `task::Spawn` and `task::Rerun` all end in Marley's
  `RoutedTerminals::spawn` with the resolved `SpawnInTerminal`. When its command or an argument
  holds `{{name}}`, Marley opens the parameter prompt (one field per placeholder, the default from
  the file's `marley.parameters` for that label, or the value last used this session), fills them
  in, and runs the task as the layout runs every task (the center, #441). A cancelled prompt runs
  nothing. A task with no placeholders runs as before.

### Out (explicitly deferred)
- Typing the filled command at the shell's prompt without Enter (Warp's run) instead of a task
  terminal: T4's "tasks as blocks" is where task output becomes blocks.
- Enum parameters, fixed or from a shell command's output; Warp's AutoFill by an agent (#573's
  System One use 6, "which tokens of a command are workflow parameters").
- Importing Warp's exported YAML, and a Marley workflows file (Chad's answer 2 rules it out).
- Multi-line commands (the editor is one line; a block whose command holds a newline gets no
  button).
- Editing or deleting a saved workflow from Marley: `zed::OpenProjectTasks` opens the file.

## Reference (§20)
- **Warp, workflows** (https://docs.warp.dev/knowledge-and-collaboration/warp-drive/workflows/,
  read 2026-09-26): Save as Workflow from the block's actions opens an editor with name, command,
  description and arguments; "Arguments use double curly braces: `{{argument_name}}`", names of
  `A-Za-z0-9`, `-` and `_`, not starting with a digit; text or enum arguments with defaults and
  descriptions; running "will paste the workflow into your active terminal input" without
  executing it, Shift+Tab cycling the arguments. Marley keeps the entry point, the `{{name}}`
  syntax, text arguments with defaults and descriptions, and the name; it stores the workflow as a
  Zed task (Chad) and runs it as a task, asking for the arguments in a prompt instead of in the
  input. Behavior map `docs/warp_architecture/subsystems/03-terminal-session-core.md` (blocks)
  was read for the shape only; no Warp code.
- **Upstream Zed:** the task model and its picker (`crates/task`, `crates/tasks_ui`), kept whole:
  `tasks.json` in `.zed/` and the config directory, `TaskTemplate`'s fields, the inventory's
  reload on file change, the picker on `alt-shift-t`, `task::Rerun`, and the spawn through
  `TerminalProvider`. Zed prompts for no value at run time (only the debugger's `$ZED_PICK_PID`),
  which is the one piece Marley adds, in its own provider.

### Prior art
- **Behavior maps and reports.** The Warp blocks note ("Save as Workflow (M)": the gpui-era
  module, rules guessing parameters first, storage as Chad's call), and Chad's answer 2.
  `docs/zed_architecture/subsystems/08-terminal-tasks-fusion.md` §3 (template → resolved → spawn
  spec), §5 (discovery, inventory, the modal) and §6 (the spawn pipeline and the provider seam).
  #441 (tasks in the Marley layout: the provider and the center).
- **Published material.** Warp's docs above. Zed's tasks documentation (zed.dev/docs/tasks): the
  file locations and the `$ZED_*` variables, with `${ZED_X:default}` for a default.
- **The code we already ship.**
  - `crates/task/src/task_template.rs`: `TaskTemplate` (`:24`, `label`, `command`, `args`, `env`,
    `cwd`, `use_new_terminal`, `allow_concurrent_runs`, `reveal`, `reveal_target`, `hide`, `tags`,
    `shell`, `show_summary`, `show_command`, `save`, `hooks`; no `deny_unknown_fields`);
    `resolve_task` (`:162`); `substitute_variables_in_str` (`:375`), whose unknown non-`ZED_`
    variables are left as they are (`:433`), and where `{{name}}` is never a variable at all, so
    a placeholder survives resolution verbatim; `unknown_variables` (`:312`), which drops a
    template naming an unknown `$ZED_*`, so parameters must not use that prefix.
  - `crates/paths/src/paths.rs`: `tasks_file` (`:309`, `<config>/tasks.json`, which is
    `~/.config/marley/tasks.json` since `APP_NAME` is Marley), `local_tasks_file_relative_path`
    (`:505`, `.zed/tasks.json`). `crates/project/src/task_inventory.rs`: `update_file_based_tasks`
    (`:762`, `parse_json_with_comments` then `serde_json::from_value::<TaskTemplate>` per entry).
    `crates/project/src/project_settings.rs`: the global file's watcher (`:1475`) and the
    worktree's `.zed/tasks.json` (`:1388`), so a write to disk is enough.
  - The writers: `settings_json::append_top_level_array_value_in_json_text`
    (`crates/settings_json/src/settings_json.rs:508`, the keymap editor's route, comment-preserving);
    `tasks_ui::insert_task_json_into_editor` (`crates/tasks_ui/src/tasks_ui.rs:22`) and the
    debugger's `save_scenario` (`crates/debugger_ui/src/debugger_panel.rs:1180`), the
    open-then-save route, which this ticket does not take (no editor tab for a save).
  - The picker and the run: `TasksModalDelegate::confirm` (`crates/tasks_ui/src/modal.rs:390`)
    → `Workspace::schedule_resolved_task` (`crates/workspace/src/tasks.rs:64`) →
    `TerminalProvider::spawn` (`crates/workspace/src/workspace.rs:328`), which Marley implements
    in `RoutedTerminals` (`crates/marley_workbench/src/routing.rs:96`, forcing the center in the
    Marley layout and moving a task's old terminal there); `schedule_task` (`tasks.rs:29`);
    `TerminalPanel::spawn_task` (`crates/terminal_view/src/terminal_panel.rs:632`) and
    `prepare_task_for_spawn` (`:1286`); `Project::create_terminal_task`
    (`crates/project/src/terminals.rs:64`), where the task's command is the terminal's program
    (`:227`), so a task terminal loads no shell integration and shows no blocks. Bindings:
    `alt-shift-t` → `task::Spawn`, `alt-shift-r` with `reveal_target: center`
    (`default-linux.json:733-734`).
  - The block: `marley_block` and its actions row (`terminal_element.rs:2320`, `:2366`);
    `AnchoredBlock::command`, `command_verified`, `prompt.pwd`, `prompt.git_branch`
    (`anchored.rs:38-58`, `block.rs:46`). `blocks::focused_terminal` (`blocks.rs:63`).
  - Modals: `RenameBranchModal` (`crates/git_ui/src/git_ui.rs:442`, a single-line `Editor` with
    Confirm and Cancel), `NewProcessModal` (`crates/debugger_ui/src/new_process_modal.rs`, a
    multi-field form that saves into `.zed/debug.json`), Marley's `NewAgentPicker`
    (`agents.rs:251`).
  - The gpui-era module, Marley's own code in the design record:
    `/srv/stacks/marley/crates/marley_app/src/workflows.rs` (297 lines, pure): `Workflow`,
    `substitute` (`{{name}}`, whitespace inside trimmed, a lone brace left), `params_of` (distinct
    names, first seen first), `ParamPrompt`. `substitute` and `params_of` move over as they are.
  - Does a crate we build own this seam? `task` owns the file, the model and the picker;
    `settings_json` owns the append; Marley's provider owns the run; only the guesses and the
    parameter prompt are new.

## UI proof
UI-AFFECTING. `script/e2e/558-save-as-workflow.sh` (`compositor sway`, for the hovered block's
button). Fixtures: a scratch repository on a branch `feature/one` with `web/` holding
`index.html`; the scenario's bash. Steps and shots: `python3 -m http.server 8123 --directory
web` typed and stopped with Ctrl-C; the pointer on its block, the Save as Workflow button
(`558-01-button`); a click, the editor with `{{port}}` and `{{path}}` in the command, their
defaults `8123` and `web`, the name "python3 -m" (`558-02-editor`); the name changed to "serve",
Enter, a toast, `.zed/tasks.json` in the log with the entry appended after a pre-seeded task and
its comment kept (`558-03-saved`); `alt-shift-t`, the picker listing "serve" (`558-04-picker`);
Enter, the parameter prompt with the defaults (`558-05-parameters`); the port changed to `8124`,
Enter, a center task terminal serving `web` on 8124, `ss -ltn` in the log (`558-06-ran`); a second
block `git status` saved with no guess, run from the picker with no prompt (`558-07-no-parameters`);
`ctrl-alt-r` (Rerun) on the parameterized task, the prompt again with `8124` prefilled
(`558-08-rerun`).

## Locked-In Decisions
- D1: A workflow is a Zed task in `tasks.json` and nothing else (Chad): the project's
  `.zed/tasks.json` by default, the global file by choice. Marley writes what Zed reads and adds
  one key, `marley`, that Zed ignores and Marley reads back from the file by the task's label.
- D2: Warp's `{{name}}` syntax, because Zed's resolver leaves it alone and `$ZED_*` names are
  validated at load; a parameter never starts with `ZED_`.
- D3: Parameters are filled in Marley's terminal provider, the one place every run passes
  (the picker, `task::Spawn`, `task::Rerun`), so a workflow runs from Zed's own picker with no
  change to `tasks_ui`. Defaults come from the file, and the last values of the session win
  over them.
- D4: A workflow runs as a task, in the center as #441 routes tasks, not typed at a prompt: that
  is what "runnable from Zed's task picker" means, and T4 turns task output into blocks later.
- D5: The rules guess numbers, URLs, the branch and existing paths, after the first word, and
  the editor shows every guess for the user to keep or undo by editing the command. No model
  (#573 may propose parameters later).
- D6: The file is edited as text with comments kept, never rewritten from a parsed value, and a
  name already in the file is refused rather than duplicated.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the pointer is on a finished block with a verified command, the system shall show a Save as Workflow button beside Copy and Rerun. | Shot `558-01-button` |
| REQ-002 | WHEN the user clicks it, the system shall open the editor with the command's numbers, URLs, branch and existing paths replaced by `{{name}}` placeholders, each with its token as the default. | Shot `558-02-editor` |
| REQ-003 | WHEN the user saves, the system shall append the task to the project's `.zed/tasks.json` with `label`, `command`, `cwd` and `marley.parameters`, keeping the file's other entries and comments. | Shot `558-03-saved`; the log's file |
| REQ-004 | WHEN the file is written, Zed's task picker shall list the workflow by its name without a restart. | Shot `558-04-picker` |
| REQ-005 | WHEN the user runs a workflow whose command holds placeholders, the system shall ask for each with its default before running. | Shot `558-05-parameters` |
| REQ-006 | WHEN the user confirms the prompt, the system shall run the filled command as a task in the center. | Shot `558-06-ran`; the log's `ss -ltn` |
| REQ-007 | WHEN the user runs a workflow without placeholders, the system shall run it at once. | Shot `558-07-no-parameters` |
| REQ-008 | WHEN the user reruns a workflow, the system shall ask again with the last values prefilled. | Shot `558-08-rerun` |
| REQ-009 | WHEN the user cancels the editor or the prompt, the system shall write and run nothing. | The log: the file unchanged after an Escape; no task terminal |
| REQ-010 | The diff gate shall be green. | `script/gates.sh --diff` |

## Phase Plan
- **P1 Plan:** this spec; at promotion, check the JSON schema Zed attaches to `tasks.json` for an
  unknown-key warning on `marley` (and, if it warns, keep the key and add it to the schema in a
  small `task` hunk with its ledger row); check whether `MarleyBlockExtras` has landed; ask the
  brain.
- **P2 Code:** the ledger rows first (`crates/terminal_view/src/terminal_element.rs` if the hook
  is new; `crates/marley_workbench/Cargo.toml` gains `settings_json` and `task`); the pure
  module (`substitute`, `params_of`, the guesses); the editor modal; the writer; the provider's
  prompt and fill; fmt and clippy clean; a review of the diff.
- **P3 Test:** write and run the scenario and read every shot; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley/three-prong-plan.md` (T4's status);
  `docs/marley_architecture/` for `marley_terminal` and `marley_workbench`; the ledger capture;
  close the ticket, archive, commit.
