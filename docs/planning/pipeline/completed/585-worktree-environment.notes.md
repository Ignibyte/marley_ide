# A new worktree agent's worktree gets the main checkout's gitignored files, an offered setup command, and Marley's paths in its hook tasks — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-585-worktree-environment.md
- **Pipeline spec:** 585-worktree-environment.spec.md

## Phase 1 — Plan
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (cargo busy with #511's release install, so
  no cargo ran and no crate or scenario was edited); recall ✓; mint ✓ (a new pair from the
  templates, `pipeline_id` e5dfb461-ce75-40ac-bf70-5edb0accabfe; the BACKLOG row removed; the
  ticket in progress); the prior-art sweep ✓; spec and design ✓; TICKET-590 filed ✓.
- **Request:** TICKET-585, #510's split slice 2, "the worktree's environment", with the Orca
  second pass's setup suggestion folded in (#510's notes, "The split" and "Folded in from the Orca
  second pass"); Chad's standing word of 2026-09-26: build every remaining finding.
- **Classification / tier:** feature, prong 2. As filed it held four parts; the port offset needs
  two Zed touches, a slot registry and persistence, so it goes to TICKET-590 and this slice keeps
  what happens before the agent starts (M). No Zed touch.
- **Recall (§18.3):**
  - #510's notes: `.worktreeinclude` "read with the `ignore` crate's
    `gitignore::GitignoreBuilder` … as Claude Code reads it … only paths that also are gitignored …
    copied … before the agent starts, with a size cap and the skipped entries named in a toast";
    the setup line `Setup: pnpm install (pnpm-lock.yaml found)` off by default and remembered in
    `git config marley.worktreeSetup` (the command, or `none`); other ecosystems only when a real
    repository asks.
  - #511 (just shipped): `worktree_git::git` is the adapter for git in the main checkout, and its
    `text()` trims (PR-claude-column-significant-git-output-is-read-untrimmed-001 applies to
    `ls-files -z` output); the seed's mark (`worktree_agents::skip_seed`), L-claude-511 on the
    first terminal racing an opener's item.
  - #587: the trust watch ends a minute after the launch line while Claude Code's question has
    not shown (`agent_trust.rs`), which a setup run first would outlast.
  - The brain (consultations c80c1a66b2024647a44a969074a8e5f0 for the question and
    f72931c768b84a2489def904a44b2b1e for the keywords): nothing on this seam.
- **Discovery** (the Explore report over f648bcb382, 2026-09-29):
  - The create flow, `worktree_agents::create` (367-460): `free_folder`, the `CreateWorktree`
    action, the seed's mark, `create_worktree_workspace_on_branch`, the mark removed, `write_base`,
    `agents::start_cli_with_prompt`, `agent_trust::watch`. The copy fits after the create resolves
    (430) and before 441, on the background executor. `plan()` is synchronous; the prompt modal
    (`render` 294-325) has a headline, the editor and the `"{branch} from {base}"` label, and the
    Setup line goes after it (`Checkbox::new(..).label(..).on_click(..)`, as `clients.rs:691-697`);
    the choice lives on the modal and passes through `confirm` (270-281) to `start`.
  - Zed's create (`worktree_service.rs`): the workspace exists long before the future resolves;
    `run_create_worktree_tasks` (1416) schedules the hooks with `detach_and_log_err` as the
    future's last step, and nothing awaits them, so hooks run alongside Marley's steps.
  - The launch: `agents.rs:280-329` types `marley_agent::launch_line(kind, mode, &prompt)` once
    through `write_init_command_after_startup`, which clears the screen and refuses if the user
    typed; a setup command goes in the same line, `<setup> && <launch line>`.
  - The hooks: `TaskHook::CreateWorktree` (`task_template.rs:92-98`), run only by
    `Workspace::run_create_worktree_tasks` (`workspace/src/tasks.rs:235-338`) with a hand-built
    `TaskContext` (`ZED_WORKTREE_ROOT`, `ZED_MAIN_GIT_WORKTREE`, an empty `project_env`);
    `resolve_task` puts every task variable into `SpawnInTerminal.env`; each hook goes through the
    workspace's terminal provider, Marley's `RoutedTerminals::spawn` (`routing.rs:101-136`). A
    `VariableName::Custom` prints as `ZED_CUSTOM_*`, so `MARLEY_` names cannot come from task
    variables. `Inventory::templates_with_hooks` (`task_inventory.rs:719-730`) counts the
    worktree's and the global tasks.
  - Terminal environments: shells from `create_terminal_shell_internal`, tasks from
    `create_terminal_task` (`crates/project/src/terminals.rs`), two functions; `BROWSER` (#561) and
    `MARLEY_TERMINAL_ID` (#520) go in `TerminalBuilder`, which does not know the project. This is
    what moves the port offset to TICKET-590.
  - `ignore` 0.4.24 is a workspace dependency (`Cargo.toml:668`); `GitignoreBuilder` is used by
    `worktree.rs:3722-3736` and `fs/src/fake_git_repo.rs:395-425`; no `WalkBuilder` anywhere. Zed's
    worktree snapshot does not load ignored directories, so the candidates come from git.
  - No reusable lockfile detection (`languages::typescript::detect_package_manager` is private and
    has no bun); `marley_workbench` does not depend on `languages`.
  - #537 (queued) plans `create_terminal_shell_with_env` and a REQ-004 "the typed line is the
    launch line": it must allow this slice's setup prefix for worktree agents.
- **Decisions:** D1 to D6 in the spec.

### Prior art
In the spec. The one owner found: the `ignore` crate for the matching; Zed's `Fs` for the copy.

### Design
- **Approach.**
  - *`crates/marley_workbench/src/worktree_include.rs` (new, Marley):*
    `copy_included(main, worktree, fs) -> anyhow::Result<Included { copied, skipped }>`.
    It reads `<main>/.worktreeinclude` through `Fs` (absent: nothing), builds a `Gitignore` with
    `GitignoreBuilder::new(main)` and `add_line` per line, lists the candidates with
    `worktree_git::ignored_entries(main)` (`ls-files --others --ignored --exclude-standard
    --directory -z`, read untrimmed, a trailing `/` marking a wholly ignored directory), and sorts
    each into:
    - a file the matcher matches (`matched_path_or_any_parents(path, false)` is an ignore match):
      copied;
    - a directory the matcher matches as a directory: copied whole;
    - a directory a pattern reaches: walked, its matching files copied. A pattern reaches a
      directory when, negations aside, it starts with `**/` or has no slash (read as `**/<p>`) and
      its first name after `**/` is one of the directory's names; or, anchored, when its literal
      leading components and the directory's are one a prefix of the other, the walk starting at
      the deeper of the two.
    Sizes come from `Fs::metadata` before anything is written; the entries take the budget
    (`BUDGET_BYTES` 100 MB, `BUDGET_FILES` 10,000) in `ls-files` order, and an entry that would
    pass it is skipped whole with its size; a walk stops once it passes the budget. Each file is
    copied with `Fs::create_dir` for its parent and `Fs::copy_file` with `CopyOptions { overwrite:
    false, ignore_if_exists: true }`; a failure is named with the skipped. A symlinked directory is
    not walked.
  - *`worktree_git.rs`:* `ignored_entries(main)`; `setup_choice(main)` and
    `remember_setup(main, value)` over `git config marley.worktreeSetup`.
  - *`worktree_agents.rs`:*
    - `create`: after the create resolves and before `write_base`, `copy_included` on the
      background executor, then a toast (`NotificationId` of its own) when anything was skipped or
      failed.
    - The setup offer: `setup_offer(main, fs) -> Option<SetupOffer { command, lockfile }>` (a
      `package.json` and exactly one manager's lockfiles: pnpm, bun (`bun.lock`, `bun.lockb`),
      yarn, npm). `open_prompt` builds the modal, then fills its `setup` from a task: the offer,
      the remembered choice, and `Inventory::templates_with_hooks(&TaskHook::CreateWorktree, ..)`
      for the main checkout's worktree (any hook: no offer). The modal draws `Setup: <command>
      (<lockfile> found)` with a `Checkbox`, checked when `marley.worktreeSetup` equals the
      command; `confirm` passes the command when checked; `start` writes the choice (the command,
      or `none`) in the background.
  - *`agents.rs`:* `start_cli_with_prompt` takes `setup: Option<String>`; `start_cli` passes none.
  - *`marley_agent`:* `launch_line_after(setup, kind, mode, prompt)`, `<setup> && <launch line>`,
    `setup` typed as the user would (it is Marley's table's command).
  - *`agent_trust.rs`:* `watch` takes how long to wait for the question; `create` passes 15
    minutes when a setup runs, else the minute it waits today.
  - *`routing.rs`:* `RoutedTerminals::spawn` adds `MARLEY_ROOT_PATH` from `ZED_MAIN_GIT_WORKTREE`
    and `MARLEY_WORKTREE_PATH` from `ZED_WORKTREE_ROOT` to `task.env`, unless the task sets them.
  - *`Cargo.toml`:* `ignore`.
- **File manifest** (all Marley crates; no Zed touch, no ledger row):
  `crates/marley_workbench/src/worktree_include.rs` (new),
  `crates/marley_workbench/src/marley_workbench.rs` (the module),
  `crates/marley_workbench/src/worktree_git.rs`, `crates/marley_workbench/src/worktree_agents.rs`,
  `crates/marley_workbench/src/agents.rs`, `crates/marley_workbench/src/agent_trust.rs`,
  `crates/marley_workbench/src/routing.rs`, `crates/marley_workbench/Cargo.toml`,
  `crates/marley_agent/src/marley_agent.rs`. Test: `script/e2e/585-worktree-environment.sh`,
  `script/e2e/golden`.

### E2E plan
`script/e2e/585-worktree-environment.sh`, `compositor sway`, on 510's fixture pattern (its fake
`claude`, its + menu steps and submenu click).

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | setup: `.gitignore` (`.env`, `secrets/`, `big/`, `node_modules/`, `vendor/`), `.worktreeinclude` (`.env`, `secrets/`, `big/`, `README`, `vendor/**/local.json`), the files (`.env`, `secrets/key.txt`, `vendor/a/local.json`, `node_modules/x/index.js`); steps: New Agent in Worktree, Claude Code, Enter with the box clear; `ls -a` and `find` in the worktree's terminal | `585-02-copied`; the run log's listing of the worktree |
| REQ-002 | the same listing: `README` as committed, no `node_modules/` | the run log |
| REQ-003 | setup: `big/blob`, a 150 MB sparse file (`truncate -s 150M`) | `585-02-copied` (the toast names `big/`) |
| REQ-004 | setup: `package.json`, `pnpm-lock.yaml`; the first prompt | `585-01-prompt` |
| REQ-005 | setup: a fake `pnpm` first on the PATH (logs its arguments and folder, exits 0); steps: the second worktree with the box clicked | `585-03-setup`; `pnpm.log` |
| REQ-006 | steps: `git config --get marley.worktreeSetup` in the run log after the second start; the third prompt | `585-04-remembered` |
| REQ-007 | steps: commit `.zed/tasks.json` with a task `"hooks": ["create_worktree"]`; the fourth prompt | `585-05-hook` |
| REQ-008 | the hook task writes `$MARLEY_ROOT_PATH` and `$MARLEY_WORKTREE_PATH` to a file in `$E2E_WORK` | `585-05-hook`; the file in the run log |

No scenario can reach: a real package install (the fake stands in, so nothing is fetched), and
Claude Code's trust question after a long install (#587's scenario covers the question; the 15
minutes are read in the review).

### Risks
- Zed's `create_worktree` hooks start alongside the copy, since Zed schedules them inside the create
  and never awaits them: a hook that reads a copied file may run before it lands. The copy is small
  and local and the hooks start shells first, so it usually lands first; copying before the hooks
  needs a callback in Zed's `do_create_worktree`, left for when a repository needs it. The guide
  says to run such steps as the setup command, which runs after the copy.
- `<setup> && <launch line>` needs a shell with `&&`: bash, zsh and fish 3 have it.
- The budget's walk of a large ignored directory a pattern reaches is bounded by the budget.
- The new worktree's `.zed/tasks.json` is its base's committed copy; the offer reads the main
  checkout's tasks, which differ only while `tasks.json` has changes not committed.
- #537's REQ-004 (the typed line is the launch line) must allow the setup prefix for worktree
  agents; a line in its queued notes at Complete.
- A remembered `marley.worktreeSetup` whose command no longer matches the lockfile leaves the box
  clear.

## Phase 2 — Code
- **Checklist** (no task tool): `worktree_include.rs` ✓; `worktree_git.rs` ✓;
  `worktree_agents.rs` ✓; `agents.rs` ✓; `marley_agent` ✓; `agent_trust.rs` ✓; `routing.rs` ✓;
  `marley_workbench.rs` and `Cargo.toml` ✓; fmt ✓; clippy ✓ (`just clippy marley_agent
  marley_workbench`, all targets: one finding on the first run and four on the second,
  `too_long_first_doc_paragraph` twice, `similar_names` (`matched` beside `matcher`),
  `duration_suboptimal_units` and `option_if_let_else`, each fixed at the source; clean on the
  third); the review ✓.
- **What was built:**
  - *`worktree_include.rs` (new):* `copy_included(main, worktree, fs)` reads `.worktreeinclude`
    through `Fs` (absent or a folder: nothing), builds a `Gitignore` rooted at the main checkout
    (a line that is no pattern is an error naming it), and takes each entry
    `worktree_git::ignored_entries` lists: a file a line matches
    (`matched_path_or_any_parents`); a directory a line matches, or one a line reaches
    (`Reach::Named` for `**/<name>` and slash-less lines, by the directory's names;
    `Reach::Under` for a line with a slash, by its leading literal names, the walk starting at the
    deeper of the two), walked with every file checked against the matcher so a `!` line keeps a
    file out; a folder holding a `.git` and a link to a folder are not walked. Sizes are summed
    before writing; an entry past 100 MB or 10,000 files in all is skipped whole and named with its
    size ("big/ (150 MB, past the 100 MB budget)"), a walk past 10,000 files stops. Each chosen
    file is copied with `Fs::create_dir` and `Fs::copy_file` (`overwrite: false,
    ignore_if_exists: true`); a failed copy is named.
  - *`worktree_git.rs`:* `ignored_entries` (`ls-files --others --ignored --exclude-standard
    --directory -z`, split on NULs, never trimmed, a trailing `/` marking a directory);
    `setup_choice` and `remember_setup` over `marley.worktreeSetup`.
  - *`worktree_agents.rs`:* `LOCKFILES` (Orca's table) and `setup_offer` (a `package.json` and one
    manager's lockfiles; checked when the key keeps that command); `has_create_worktree_tasks`
    (`Inventory::templates_with_hooks` for the workspace's first folder, the global tasks
    included); the prompt reads the offer in the background unless there are hooks, and draws
    `Setup: <command> (<lockfile> found)` as a `Checkbox`; `Launch { prompt, setup }` carries the
    prompt's state to `start` and `create`; `create` runs `copy_included` (a helper of the same
    name here, with its toast under its own `NotificationId`) after Zed's create and before
    `write_base`, then `remember_setup` (the command or `none`, only when an offer showed), then
    starts the agent with the command and waits 15 minutes instead of one for Claude Code's trust
    question when a setup runs. `show_toast` takes its id.
  - *`agents.rs`:* `start_cli_with_prompt(.., setup: Option<String>, ..)`; `start_cli` passes
    none.
  - *`marley_agent`:* `launch_line_after(setup, kind, mode, prompt)`, `<setup> && ` before
    `launch_line`'s bytes.
  - *`agent_trust.rs`:* `watch` takes its wait; `WATCH_FOR` (a minute) and `WATCH_AFTER_SETUP`
    (15 minutes).
  - *`routing.rs`:* `with_marley_paths` in `RoutedTerminals::spawn`, in both layouts:
    `MARLEY_ROOT_PATH` from `ZED_MAIN_GIT_WORKTREE` and `MARLEY_WORKTREE_PATH` from
    `ZED_WORKTREE_ROOT`, names from Zed's `VariableName`, a task's own values kept.
  - *`Cargo.toml`:* `ignore`; `collections` moved from the dev-dependencies (Zed's
    `templates_with_hooks` takes its `HashSet`).
- **Deviations from the plan:** none in behavior. `create` grew past pedantic's hundred lines, so
  the copy and the setup choice are helpers of their own; the prompt's two fields travel as
  `Launch`, since `create` would otherwise take eight arguments.
- **The review of the diff:** against each criterion:
  - REQ-001 to REQ-003: the copy is awaited before the launch; candidates come only from git's
    ignored listing, so a tracked file (README) cannot be one; a wholly ignored directory no line
    reaches (`node_modules/`) is never walked; the budget is read before any write.
  - REQ-004 to REQ-007: no offer while hooks exist (read on the main thread from the inventory, no
    IO); the offer's IO in the background; the choice written only when an offer showed; the box's
    state is the modal's, passed at Enter.
  - REQ-008: the variables are added where every hook task passes (`RoutedTerminals::spawn`).
  - gpui: the prompt's reading task updates only the prompt, from an async context;
    `has_create_worktree_tasks` reads the project, its task store and the inventory inside the
    workspace's update, none of them the workspace.
  - IO: git through async processes, files through `Fs`, the copy and the offer on the
    background executor (`blocking_io_on_foreground` holds).
  - §20: Claude Code's published rules and Orca report 02 are behavior; no source was read. No
    Zed touch, so no ledger row.
  - One thing noted, not changed: an ignored file whose metadata cannot be read is left out without
    a name, as git would have trouble listing it too; a failed copy is named.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario ✓; three runs, every shot read ✓; 510 and 587 alone
  ✓; 585 in `script/e2e/golden` ✓; the golden set stopped at 29 of 53, all passing, on Chad's
  call; the gate ✓ (GATE GREEN [diff]).
- **The scenario:** `script/e2e/585-worktree-environment.sh`, `compositor sway`, on 510's fixture
  (its fake `claude` with the plugin's SessionStart, its `+` steps and submenu click). Setup: a
  HOME of its own and a scratch identity; `main` with README, `.gitignore` (`.env`, `secrets/`,
  `big/`, `node_modules/`, `vendor/`), `.worktreeinclude` (`.env`, `secrets/`, `big/`, `README`,
  `vendor/**/local.json`), `package.json` and `pnpm-lock.yaml`, committed; then the ignored files
  (`.env`, `secrets/key.txt`, `big/blob` a 150 MB sparse file, `vendor/a/local.json`,
  `vendor/a/other.json`, `node_modules/x/index.js`); a fake `pnpm` first on the PATH that logs its
  folder and arguments. Steps: the first prompt; the first worktree with the box clear, listed from
  the main terminal and from the run log; the second with the box clicked (the editor clicked
  again for the prompt's text); its agent's row, found from the two folders' order; the third
  prompt, Escape; `.zed/tasks.json` with a `create_worktree` task writing `$MARLEY_ROOT_PATH` and
  `$MARLEY_WORKTREE_PATH` to `hook.txt`, committed; the fourth prompt and worktree.
- **Found in Test, fixed at the source** (then run again):
  - **REQ-007 failed** (run 2's `585-05-hook`): with the task committed, the fourth prompt still
    offered `pnpm install`. The offer asked the inventory about the first folder of the workspace
    whose `+` was clicked, by then a linked worktree made before the task existed. It now reads the
    `.zed/tasks.json` committed at the base (`worktree_git::committed_file`, `git cat-file blob
    <base>:.zed/tasks.json`, parsed with `settings::parse_json_with_comments::<TaskTemplates>`),
    the file the new worktree gets, and the inventory only for the user's global tasks
    (`TaskSourceKind::AbsPath`). An F-block at Complete.
  - The scenario's own: run 1's guessed box position lay outside the modal, so the click dismissed
    it (measured: the box at 550,205, the editor at 800,143); run 2's second agent's row was the
    first's, since the rail lists worktrees by folder and Zed names them at random (the row is now
    worked out from the folders' order).
- **The shots** (the third run; every one read):
  - `585-01-prompt` (REQ-004): the prompt, `agent/<name> from main`, and under it an unchecked box,
    "Setup: pnpm install (pnpm-lock.yaml found)".
  - `585-02-copied` (REQ-001 to REQ-003): the main terminal's `ls -a` of the first worktree:
    `.env`, `.gitignore`, `package.json`, `pnpm-lock.yaml`, README, `secrets`, `vendor`,
    `.worktreeinclude`, and `find`'s `secrets/key.txt` and `vendor/a/local.json`; the toast "Not
    copied into <name> from .worktreeinclude: big/ (150 MB, past the 100 MB budget)". The run log:
    no `vendor/a/other.json`, no `node_modules`, no `big`, `git status` clean (README as
    committed), no install run.
  - `585-03-checked` (REQ-005): the second prompt with the box checked.
  - `585-03-setup` (REQ-005): the second worktree's agent terminal: `$ pnpm install && claude
    'second'`, the fake's `Done in 0.1s`, then `started in <name>: [second]`. The run log:
    `pnpm.log` names the second worktree and `install`; `marley.worktreeSetup` is `pnpm install`.
  - `585-04-remembered` (REQ-006): the third prompt with the box checked from the kept choice.
  - `585-05-hook` (REQ-007): the fourth prompt with no Setup line. The run log (REQ-008):
    `hook.txt` holds the main checkout and the fourth worktree, both checked against `realpath`.
- **Focus:** each run in its own headless sway ("sway: stopped, with the run's Marley, pointer and
  keyboard"); nothing reached the user's session.
- **510 and 587 alone:** both pass (6 and 5 checks), the create flow and the trust watch as before.
- **Not reached by a scenario:** a real install (the fake stands in, so nothing is fetched), and
  Claude Code's trust question after a long install (#587's scenario covers the question; the 15
  minutes are read in the review).
- **The golden set:** with 585 added, 53 to run against the debug build; 29 had passed and none
  had failed when it was stopped (Chad, 2026-09-29: no golden runs and no scenarios for the rest
  of the queue; unit tests and mutation come after it). The scenario it stopped in, 523, left its
  headless sway running (the #588 case), stopped by hand.
- **The gate:** `just gate-diff` (log in the session scratchpad): 16 passed, 0 failed, `GATE GREEN
  [diff]`, the receipt written. cargo-deny's and dylint's warnings are the tree's, not this diff's.

---
## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented:** `CHANGELOG.md` (Added: a worktree agent's worktree gets its environment);
  `docs/marley_architecture/marley_workbench.md` ("A worktree agent's environment": the copy, its
  reach rule and budget, the setup offer and its choice, `with_marley_paths`);
  `docs/marley_architecture/marley_agent.md` (`launch_line_after`); `docs/marley/workbench-shell.md`
  (W4's record gains #585); `docs/marley/tutorial-outline.md` (585 shipped, 590 added);
  `docs/marley/guide.md` ("Worktree agents": `.worktreeinclude`, the setup command, the paths in
  tasks). No Zed crate changed; `Cargo.lock`'s row already covers a Marley crate's new dependency
  (`ignore`, built already).
- **Knowledge:** F-claude-585-the-setup-offer-read-the-wrong-worktrees-tasks-001,
  L-claude-585-a-new-worktree-gets-its-bases-committed-tasks-001,
  L-claude-585-the-rail-lists-worktrees-in-gits-order-001,
  AD-claude-585-a-new-worktree-copies-worktreeinclude-and-offers-one-setup-command-001.
- **Brain:** consultation c80c1a66b2024647a44a969074a8e5f0 closed with
  `decisions/marley-gives-a-new-worktree-its-environment-worktreeinclude-and-one-setup-command`
  (follow-up 2026-10-29); f72931c768b84a2489def904a44b2b1e, the keyword consultation, closed as a
  duplicate.
- **Filed:** TICKET-590 (the port offset), at planning, its row at the top of the queue. #537's
  queued notes gain a line: its REQ-004 allows the setup prefix a worktree agent's launch line
  may carry.
- **Closed:** TICKET-585 moved to `tickets/closed/`; it had no BACKLOG row.
- **The way of working from here:** Chad, 2026-09-29, after this ticket's golden run: no
  scenarios and no golden runs for the rest of the queue, the static gate only; unit tests and
  mutation come after it.
