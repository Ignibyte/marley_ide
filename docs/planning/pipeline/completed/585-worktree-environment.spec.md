---
pipeline_id: e5dfb461-ce75-40ac-bf70-5edb0accabfe
ticket: docs/planning/tickets/open/TICKET-585-worktree-environment.md
status: Phase 4 — Complete PASS
title: "A new worktree agent's worktree gets the main checkout's gitignored files, an offered setup command, and Marley's paths in its hook tasks"
type: feature
slice: prong 2, worktree agents, the worktree's environment (slice 2 of #510's split, part 1; the port offset is #590)
references: [docs/planning/pipeline/completed/510-worktree-agents.notes.md, docs/orca_architecture/02-worktrees-and-review.md]
---

## Title
A worktree that New Agent in Worktree makes is a clean checkout: no `.env`, no local config, no
dependencies. Marley now copies the main checkout's gitignored files that `.worktreeinclude` names
into it before the agent starts, as Claude Code does for its own worktrees; offers the install
command of the repository's one JavaScript package manager, off by default and remembered per
repository, run in the agent's terminal before the agent; and gives tasks in a linked worktree,
Zed's `create_worktree` hooks among them, `MARLEY_ROOT_PATH` and `MARLEY_WORKTREE_PATH`.

## Scope
### In
- **`.worktreeinclude`** at the main checkout's root, in `.gitignore` syntax, read as Claude Code
  reads it (published at code.claude.com/docs/en/worktrees, "Copy gitignored files into
  worktrees"): only files that match a pattern and are gitignored in the main checkout are
  copied, so tracked files are never duplicated. A `**/` pattern, and a pattern with no slash,
  reaches into a wholly ignored directory only when that directory itself matches, or when the
  first name after `**/` is one of the names in the directory's path; a pattern that names the
  directory (`vendor/**/config.json`) reaches it. The copy runs after Zed makes the worktree and
  before the agent's terminal starts, never overwrites a file already there, and stays within a
  budget, 100 MB and 10,000 files, measured before anything is written: an entry that would pass
  it is skipped whole. Skipped entries and failures are named in one toast; a copy with nothing
  skipped shows none.
- **The setup command.** When the main checkout's root has a `package.json` and the lockfile of
  exactly one package manager (`pnpm-lock.yaml` → `pnpm install`; `bun.lock` or `bun.lockb` →
  `bun install`; `yarn.lock` → `yarn install`; `package-lock.json` → `npm install`), and the
  repository has no `create_worktree` task, the worktree prompt shows one more line under the
  branch line, `Setup: pnpm install (pnpm-lock.yaml found)`, with a checkbox. It is off unless
  `git config marley.worktreeSetup` holds that same command. Starting with it checked types
  `<setup> && <the agent's launch line>` in the worktree's terminal, so the agent starts once the
  install succeeds; the choice is written to `marley.worktreeSetup` (the command, or `none`).
- **Hook paths.** Every task Marley's task provider starts whose environment holds Zed's
  `ZED_MAIN_GIT_WORKTREE` also gets `MARLEY_ROOT_PATH` (the main checkout) and
  `MARLEY_WORKTREE_PATH` (the task's worktree root, Zed's `ZED_WORKTREE_ROOT`), so a
  `create_worktree` hook written for Orca's or Conductor's names ports with one rename.

### Out (explicitly deferred)
- **The port offset** (`MARLEY_PORT_OFFSET`, `PORT`), a ticket of its own, **TICKET-590**: it needs
  a Marley hunk in both of Zed's terminal builders (`create_terminal_shell_internal` and
  `create_terminal_task`), a slot registry and its persistence. #521's port discovery already lists
  every worktree's dev servers.
- Copying before Zed's own `create_worktree` hooks start: Zed schedules them inside the create and
  never awaits them, so they run alongside the copy (a Zed touch in `do_create_worktree` would be
  needed; see Risks).
- Orca's imported setup files (`conductor.json`, `.superset/config.json`, `.cmux/cmux.json`,
  `.codex/environments/environment.toml`), other ecosystems' lockfiles (`uv.lock`, `Cargo.lock`,
  `Gemfile.lock`), the `packageManager` field; added when a real repository asks.
- Symlinked or cloned shared directories (Orca's `worktree.sharedDirectories`).
- A key to toggle the setup checkbox; it is clicked.
- Worktrees made by Zed's own worktree picker or by hand: only New Agent in Worktree's.

## Reference (§20)
N/A for Warp: a git worktree's environment, outside Warp's terminal. The behavior followed is
Claude Code's published `.worktreeinclude` (code.claude.com/docs/en/worktrees, "Copy gitignored
files into worktrees", read 2026-09-29), so a repository set up for Claude Code's worktrees works
the same in Marley's; Orca report 02 §2.2, §2.3 and §3 item 6 for the setup command, the budget
and the root-path variables (`docs/orca_architecture/02-worktrees-and-review.md`). Upstream Zed:
the worktree service's create flow and its `create_worktree` task hooks, kept as they are.

### Prior art
- **Behavior maps and reports.** Orca report 02 §2.2 (setup: `ORCA_ROOT_PATH`,
  `ORCA_WORKTREE_PATH`, `CONDUCTOR_ROOT_PATH`; "with `wait-for-setup` the agent's command is
  wrapped to start after setup ends"), §2.3 (Orca's `.worktreeinclude`: literal paths only, at most
  1,000 entries and a 256 KB file, `git check-ignore` required, a copy budget that refuses
  oversized trees before writing and reports skipped entries), §2.4 (Orca allocates no ports), §3
  items 6 and 7. The Orca second pass's setup suggestion (the lockfile table, "lockfiles of two
  managers → no suggestion"), recorded in #510's notes.
- **Published material.** Claude Code's worktrees page (above): the file, the syntax, the
  gitignored rule, the `**/` rule for wholly ignored directories, no size limit, and that a
  `WorktreeCreate` hook replaces the copy. `git-ls-files(1)` (`--others --ignored
  --exclude-standard --directory`), `gitignore(5)`.
- **Code we already ship.** The `ignore` crate (0.4.24, a workspace dependency) owns the matching:
  `gitignore::GitignoreBuilder` and `Gitignore::matched_path_or_any_parents`, which Zed's
  `worktree` crate and `fs`'s fake git repository already use. Zed's `Fs::copy_file` with
  `CopyOptions` and `fs::copy_recursive` own the copy. Zed's worktree hooks
  (`TaskHook::CreateWorktree`, `Workspace::run_create_worktree_tasks`) give hook tasks
  `ZED_MAIN_GIT_WORKTREE` and `ZED_WORKTREE_ROOT` as environment variables, and every hook reaches
  Marley's `RoutedTerminals::spawn`, so the paths need no Zed touch. `Inventory::templates_with_hooks`
  says whether a repository has a `create_worktree` task. `ui::Checkbox` draws the choice. No
  crate in the tree detects a package manager from lockfiles in a reusable way
  (`languages::typescript::detect_package_manager` is private and has no bun). Git's untracked
  checkpoint skips files of 2 MB or more (`repository.rs:3778`), a precedent for a budget.

## UI proof
UI-AFFECTING. `script/e2e/585-worktree-environment.sh` (`compositor sway`: it clicks the + menu's
submenu and the checkbox). Setup: a scratch repository on `main` with `.gitignore` (`.env`,
`secrets/`, `big/`, `node_modules/`), a `.worktreeinclude` naming `.env`, `secrets/`, `big/`,
`README` (tracked, so not copied) and `vendor/**/local.json`, the files themselves (`big/` holding
a 150 MB sparse file), `package.json` and `pnpm-lock.yaml`; fakes first on the PATH for `claude`
(510's) and `pnpm` (logs its arguments and folder, then exits 0). Shots: `585-01-prompt` (the
prompt with `Setup: pnpm install (pnpm-lock.yaml found)` and its box clear), `585-02-copied` (the
first worktree's terminal after `ls -a` and the skipped-files toast naming `big/`),
`585-03-setup` (the second worktree started with the box checked: its terminal shows
`pnpm install && claude …` and the fake agent's start after the fake install), `585-04-remembered`
(the third prompt with the box checked from `marley.worktreeSetup`), `585-05-hook` (with a
committed `create_worktree` task: the prompt has no Setup line, and the hook's output shows both
paths).

## Locked-In Decisions
- D1 — The copy follows Claude Code's published `.worktreeinclude` rules, globs and negation
  included, matched with the `ignore` crate; the candidates are what `git ls-files --others
  --ignored --exclude-standard --directory -z` lists in the main checkout, so only gitignored
  paths can be copied.
- D2 — The copy runs in `worktree_agents::create` after Zed's create resolves and before the
  agent's terminal starts, never overwrites, and keeps a budget of 100 MB and 10,000 files measured
  before writing; what passes it is skipped whole and named in a toast.
- D3 — The setup command is offered only for a repository with no `create_worktree` task and one
  JavaScript package manager's lockfile beside a `package.json` (Orca's table); off by default,
  remembered in `git config marley.worktreeSetup` (the command, or `none`), typed as
  `<setup> && <launch line>`, so a failed install leaves the agent unstarted with the error on
  screen.
- D4 — `MARLEY_ROOT_PATH` and `MARLEY_WORKTREE_PATH` are added by `RoutedTerminals::spawn` to a
  task whose environment has `ZED_MAIN_GIT_WORKTREE`, from Zed's own variables: no Zed touch.
- D5 — With a setup command, #587's trust watch waits up to 15 minutes for Claude Code's question
  instead of one, since the question comes after the install.
- D6 — The port offset is TICKET-590.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN New Agent in Worktree makes a worktree of a repository whose main checkout has a `.worktreeinclude`, the system shall copy into it, before the agent starts, the main checkout's gitignored files the file's patterns match. | Shot `585-02-copied`; the run log's listing |
| REQ-002 | The system shall copy no tracked file a `.worktreeinclude` pattern names, and no file inside a wholly ignored directory that no pattern reaches under the published `**/` rule. | The run log's listing (`README` unchanged, `node_modules/` absent) |
| REQ-003 | IF the files to copy would pass 100 MB or 10,000 files, THEN the system shall skip the entries past the budget whole and name them in a toast. | Shot `585-02-copied` (`big/` named) |
| REQ-004 | WHERE the main checkout's root has a `package.json` and one package manager's lockfile, and the repository has no `create_worktree` task, the worktree prompt shall offer that manager's install command with its box clear by default. | Shot `585-01-prompt` |
| REQ-005 | WHEN the user starts with the setup box checked, the system shall run the command in the worktree's terminal and start the agent only after it succeeds. | Shot `585-03-setup`; the fake `pnpm`'s log |
| REQ-006 | WHEN the user starts, the system shall keep the choice in `git config marley.worktreeSetup`, and the next prompt shall check the box when it holds the offered command. | Shot `585-04-remembered`; the run log's `git config` |
| REQ-007 | WHERE the repository has a `create_worktree` task, the prompt shall offer no setup command. | Shot `585-05-hook` |
| REQ-008 | WHEN a task starts in a linked worktree, a `create_worktree` hook among them, its environment shall hold `MARLEY_ROOT_PATH`, the main checkout, and `MARLEY_WORKTREE_PATH`, the worktree. | Shot `585-05-hook`; the hook's output file |

## Phase Plan
- **P1 Plan** — this spec; the design and the E2E plan in the notes; TICKET-590 filed.
- **P2 Code** — `worktree_include.rs` (the patterns, the candidates, the budget, the copy) in
  `marley_workbench`; the create flow's copy and toast; the setup offer (detection, the prompt's
  line and checkbox, `marley.worktreeSetup`, the launch line's prefix, the trust watch's window);
  `RoutedTerminals`' two variables; `ignore` in the manifest; fmt and clippy clean; a review of the
  diff.
- **P3 Test** — write and run `script/e2e/585-worktree-environment.sh`, read every shot, run 510
  and 587 alone, the golden set; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, `docs/marley_architecture/marley_workbench.md` and
  `marley_agent.md`, the shell plan and the guide, ledger capture, close, archive, commit.
