# A port offset for each worktree agent's worktree — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-590-a-port-offset-per-worktree.md
- **Pipeline spec:** 590-a-port-offset-per-worktree.spec.md

## Phase 1 — Plan
- **Checklist** (no task tool): pre-flight ✓ (no active pipeline, the README marker present,
  #585's release install compiling, so no crate edits and no cargo until it ends); recall ✓; the
  pair minted ✓; the prior-art sweep ✓; spec and design ✓.
- **Request:** TICKET-590, split from #585 at its planning; Chad's standing word of 2026-09-26
  (build every remaining finding) and of 2026-09-29 (go fast: no e2e scenario and no golden run for
  the rest of the queue, the static gate only; unit tests and mutation after the queue).
- **Classification / tier:** feature, prong 2 (S to M). One Zed touch (`terminals.rs`, two hunks).
- **Recall (§18.3):**
  - #585's notes ("Discovery"): shells from `create_terminal_shell_internal`, tasks from
    `create_terminal_task`; `BROWSER` (#561) and the terminal id (#520) go in `TerminalBuilder`,
    which does not know the project. That is why the offset was split out.
  - #510's notes ("The split"): the offset was planned there, the seam left to this slice.
  - Orca report 02 §2.4 and §3 item 7: Orca discovers ports and allocates none; many servers ignore
    `PORT` or move on their own. The ticket records that the offset helps those that read it.
  - The brain (consultations 10e413ee0e0b4cd28fdac5c361fda2c1 and its repeat
    fa62f430db0d43508686a0d6b37f1449): nothing on this seam.
- **Discovery** (an Explore agent over 58736c3c8d, then read here):
  - `create_terminal_shell_internal` (`terminals.rs:323`): `marley_project` is
    `first_project_directory` for a local project (`:332-338`); in the async block
    `env = env_task.await`, `env.extend(settings.env)` (`:396-397`), then #520's and #575's inserts.
    Every shell goes through it: restored terminals (`create_terminal_shell_restoring`), the
    panel, the agent panel's, the workbench's center terminals, agent CLIs, the rail.
  - `create_terminal_task` (`terminals.rs:64`): `env = env_task.await`, `env.extend(settings.env)`
    (`:135-136`), later `env.extend(spawn_task.env)` (`:171`). Every task goes through it: panel
    tasks and reruns, `RoutedTerminals::spawn`, the Zed Agent's terminal tool and ACP terminals,
    the debugger's debug task and run-in-terminal.
  - Splits: `clone_terminal` rebuilds from the source's final env, so a split keeps the variables.
  - A worktree agent's workspace has the worktree as its first folder
    (`create_worktree_workspace_on_branch`), so `first_project_directory` is the worktree.
  - #561's store: `marley_terminal::shell_integration::BROWSER_OPENER`, a process-wide
    `RwLock<Option<PathBuf>>`, set by the workbench (`mcp.rs:325-343`). `marley_terminal` has no gpui.
  - The create flow (`worktree_agents.rs:540-600`): Zed's create, `copy_included`, `write_base`
    (`git config branch.<b>.base`, run in the worktree), `remember_setup`, then
    `start_cli_with_prompt`, the agent's terminal.
  - Marley-made worktrees: branch prefix `agent/`, and `branch.<b>.base` set. Removing a worktree
    leaves its branch and config keys, so a free slot is judged against live worktrees.
  - `git::repository::parse_worktrees_from_str` is `pub` (`crates/git/src/repository.rs:382`);
    `marley_workbench` already depends on `git`.
  - The ledger row for `crates/project/src/terminals.rs` is `zed-touchpoints.md:69`.

### Design
- **`marley_terminal::ports`** (new module, Marley crate): `PORT_OFFSET_VARIABLE`
  (`MARLEY_PORT_OFFSET`), `PORT_VARIABLE` (`PORT`), `PORT_BASE` 3000, `PORT_STEP` 10; `SlotReader`, a
  type alias for `Arc<dyn Fn(PathBuf) -> Pin<Box<dyn Future<Output = Option<u16>>>> + Send + Sync>`;
  a process-wide `RwLock<Option<SlotReader>>` with `set_slot_reader`; and
  `async fn variables(folder: Option<PathBuf>) -> Vec<(String, String)>`, empty with no folder, no
  reader or no slot, else the offset (slot times ten, in `u32`) and the port.
- **`crates/project/src/terminals.rs`** (Zed crate, two hunks, `// Marley:` comments, ledger row
  first): in `create_terminal_task`, `marley_project` computed as the shell builder does (local
  project, `first_project_directory`), and in the async block
  `env.extend(marley_terminal::ports::variables(marley_project).await)` between `env_task.await` and
  `env.extend(settings.env)`; in `create_terminal_shell_internal`, the same line in the same place,
  with `marley_project.clone()`. The user's `terminal.env` and a task's own `env` extend after it.
- **`worktree_git`** (Marley): `SLOT_KEY` (`marleySlot`); `async fn slot_of(folder)`: `None` unless
  `folder/.git` is a file (a linked worktree), then `git symbolic-ref --quiet --short HEAD` and
  `git config --get branch.<b>.marleySlot`, parsed as `u16`; `async fn assign_slot(folder, branch)`:
  `git worktree list --porcelain` parsed with `parse_worktrees_from_str`, the live linked worktrees'
  branches other than `branch`, `git config --get-regexp` for the `marleyslot` keys, the lowest free
  from 1, written with `git config`.
- **`worktree_agents::create`** (Marley): after `write_base`, `assign_slot`; a failure is a toast
  ("The worktree's port slot was not written: …") and the agent still starts.
- **`marley_workbench::init`** (Marley): `marley_terminal::ports::set_slot_reader` with a closure over
  `worktree_git::slot_of` (errors logged, `None`).
- **Manifest:** `crates/marley_terminal/src/ports.rs` (new) and `marley_terminal.rs` (`pub mod
  ports;`); `crates/project/src/terminals.rs` (Zed); `crates/marley_workbench/src/worktree_git.rs`,
  `worktree_agents.rs`, `marley_workbench.rs`; `docs/marley/zed-touchpoints.md` row 69.

### For the quality pass
- No tests (§7, since 2026-09-29). What a later scenario would show: two worktree agents'
  terminals printing `3010` and `3020` for `echo $PORT` and `10`/`20` for `$MARLEY_PORT_OFFSET`,
  the main checkout's empty; a task's own `PORT` kept; a third worktree after the first's removal
  taking slot 1.

### Risks
- A Marley started from a worktree's terminal inherits its `PORT`, and every terminal inherits
  Marley's environment besides the map, so the main checkout's would carry it; emptying `PORT`
  could break programs that parse it. Left as is (Out).
- Two creates at the same moment could pick the same slot; creates are one prompt at a time.
- Each terminal in a linked worktree runs two short git commands before its shell starts; the main
  checkout's run none.

## Phase 2 — Code
- **Checklist** (no task tool): the ledger row (at Plan) ✓; `marley_terminal::ports` ✓; the two
  hunks in `terminals.rs` ✓; `worktree_git::slot_of` and `assign_slot` ✓; `worktree_agents::record`
  and `give_slot_reader` ✓; the reader given from `init` ✓; the review ✓; the gate ✓.
- **Built as designed**, with these changes:
  - The reader is given for each new workspace, from `init`'s `observe_new` with
    `workspace.app_state().fs`, not once in `init`, so it holds the app's `Fs` whatever the order
    of start-up; it is the same reader each time. (At first I took the `Fs` global to be unset when
    `init` runs; `main` sets it earlier in the same closure. The comment says the real reason.)
  - `marley_terminal::ports::slot_reader` boxes an async function as a `SlotReader`; the boxed
    future is `Send` (clippy's `future_not_send` on `variables` otherwise).
  - The base write and the slot moved out of `create` into `worktree_agents::record` (clippy's
    `too_many_lines`), each failure its own toast.
- **The review**, against each criterion:
  - REQ-001: `record` runs after Zed's create and before `start_cli_with_prompt`, so the agent's
    terminal reads the slot; `assign_slot` counts the listed worktrees' branches other than the
    new one, reads their `marleyslot` keys (git prints the variable name in lower case, the branch
    as written), and takes the lowest free from 1.
  - REQ-002: both builders await `variables(first_project_directory)` in their async block; a
    worktree agent's workspace has the worktree as its first folder.
  - REQ-003: no folder for a remote project; `slot_of` answers `None` when `.git` is not a file
    (the main checkout, a plain folder) or on a detached HEAD, with no git run for the first two.
  - REQ-004: the variables go in before `env.extend(settings.env)` and, for tasks, before
    `env.extend(spawn_task.env)`, so those win.
  - REQ-005: only listed worktrees' branches count, so a removed one's key is ignored.
  - REQ-006: the slot is read as each terminal starts; nothing is cached.
  - No entity is read or updated inside another's update; nothing from Warp or a GPL body; the
    Zed hunks are additive, each with its `// Marley:` comment, and the row describes them.
- **The gate:** `just gate-diff` green (16 passed, 0 failed), then again after the comment fix:
  16 passed, `GATE GREEN [diff]`, the receipt written.

---
## Phase 3 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented:** `CHANGELOG.md` (Added: a port of its own for each worktree agent's worktree;
  #585's entry loses its "next"); `docs/marley_architecture/marley_workbench.md` (the port slot in
  "A worktree agent's environment"); `docs/marley_architecture/terminal_blocks.md` (`ports.rs`);
  `docs/marley/workbench-shell.md` (W4's record gains #590); `docs/marley/tutorial-outline.md`
  (590 shipped); `docs/marley/guide.md` ("A port for each worktree"). The row for
  `crates/project/src/terminals.rs` (written at Plan) describes both hunks as shipped.
- **Knowledge:** AD-claude-590-each-worktree-agents-worktree-gets-a-port-slot-read-as-its-terminals-start-001.
  No F-block: clippy's two findings were caught before the gate; the wrong reason in a comment was
  mine and fixed in Code.
- **Brain:** consultation fa62f430db0d43508686a0d6b37f1449 closed with
  `decisions/each-worktree-agents-worktree-gets-a-port-slot-read-as-its-terminals-start`
  (follow-up 2026-10-29); 10e413ee0e0b4cd28fdac5c361fda2c1, its repeat, closed with no decision.
- **Closed:** TICKET-590 moved to `tickets/closed/`; its BACKLOG row went at promotion.
- **No tests** (§7, since 2026-09-29): what a later scenario would show is in "For the quality
  pass".
