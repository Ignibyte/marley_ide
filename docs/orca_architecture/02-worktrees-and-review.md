# Orca survey 02: worktrees, git and review

Orca source: `/srv/stacks/orca-refs/orca` at `1c2cf120e3`. Orca paths below are relative to that
root; Marley paths are relative to `/srv/stacks/marley_ide`.

## 1. Summary

Orca gives each task a `git worktree` (default `~/orca/workspaces/<repo>/<name>`, branch
`<git-user>/<name>`) made in the background, runs a trust-gated `orca.yaml` setup script in a
visible terminal, copies `.worktreeinclude` files, and starts the agent there. Review is a Monaco
diff against the base plus line notes pasted into the agent's terminal as one prompt. Merging is
`gh pr merge`; Orca never merges locally. Removal deletes the branch only when git or a merge-tree
check proves it merged. "Worktree checkpoints" are a free-text note; per-line "attribution" has no
code. Worth taking: squash-aware removal (#511), notes into the agent terminal with readiness
gating (#509, #511), `.worktreeinclude` with setup and teardown hooks (#510), port discovery by
process cwd (#510). Zed already ships the plumbing (worktree service, git checkpoints, BranchDiff,
review comments, archive refs), so #509 to #511 are smaller than they look.

## 2. Features

### 2.1 Creating a worktree

**What the user sees.** A Create Workspace composer: project, "Run on" host, agent, prompt, name
(or a pasted GitHub/Linear/Jira/GitLab URL), a start-from picker (the repo's base ref, a local
branch, a commit, a remote branch, a PR or MR), and an Advanced drawer (explicit branch name,
parent workspace, sparse checkout). Submitting closes the dialog at once; the sidebar shows a
pending row with "fetching" then "creating", the worktree's tab shows setup status, and a failed
create offers Retry.

**How it works.**

- IPC `worktrees:create` (`src/main/ipc/worktrees/create/register-worktree-create-handlers.ts`)
  routes to `createLocalWorktree` or `createRemoteWorktree` in `src/main/ipc/worktree-remote.ts`
  (3,083 lines; the local path starts at `:2327`). SSH repos run the same steps through the relay
  (`src/relay/git-handler-worktree-ops.ts`).
- Path: `computeWorktreePath` (`src/main/ipc/worktree-logic.ts:103`) gives
  `<workspaceDir>/<repoName>/<sanitizedName>`; `workspaceDir` defaults to `~/orca/workspaces`
  (`src/shared/constants.ts:158`), `nestWorkspaces` defaults to true, a repo can override with
  `worktreeBasePath`, and SSH repos fall back to a sibling `<repo>-<name>`.
  `ensurePathWithinWorkspace` refuses traversal.
- Names: `sanitizeWorktreeName` keeps Unicode letters and digits, turns other runs into `-`,
  rewrites emoji to shortcodes and collapses `..`. `computeBranchName` prepends a prefix per the
  `branchPrefix` setting: `git-username` (default), `custom`, or `none`
  (`src/main/ipc/worktree-branch-name.ts`, `src/shared/branch-prefix.ts`). A collision with an
  existing path, a local or remote branch, or a branch that already has a PR retries with `-2`,
  `-3`. A Linear issue uses Linear's own `branchName`. Generated names are retired so a deleted
  worktree's name is never reused (`src/main/worktree-name-retirement.ts`).
- Base: the repo's `worktreeBaseRef` or the default (origin/HEAD). The remote-tracking base is
  refreshed with a fetch; offline it falls back to a local ref and reports `baseFallback`.
- Git: `git worktree add --no-track -b <branch> <path> <base>` (`src/main/git/worktree-add.ts:200`),
  then `git config --local branch.<branch>.base <base>` (`:231`) and `push.autoSetupRemote=true`
  when unset (`:239`). `--no-track` stops `git status` reporting "behind origin/main" before the
  first push; the base kept in git config is what branch cleanup and compare read back later.
- Pre-warmed checkouts: opening the composer calls `worktrees:prefetchCreateBase`. Main keeps up
  to 3 detached, locked checkouts under `<workspaceRoot>/.orca-preparing/<pid>-<uuid>`
  (`git worktree add --detach --no-checkout`, `reset --hard`, `worktree lock --reason
  orca-create-preparation:v1:<pid>:<id>`), each for 5 minutes
  (`src/main/worktree-create-preparation-pool.ts`, `src/main/git/worktree-create-preparation.ts`).
  On submit one is `git worktree move -f -f`'d to the final path and gets
  `git checkout --no-track -b <branch> <sha>`. The code cites a 4.1 s p50 cold `worktree add` on
  large repos as the reason.
- PR and MR start points: fetch `refs/pull/<n>/head` into
  `refs/orca/pull/<remote>-<urlhash>/<n>` (`src/shared/review-head-tracking-ref.ts`), branch from
  that SHA, compare against the PR's base branch, and push to a fork remote that is added lazily
  on first push.
- Metadata: `WorktreeMeta` in Orca's `orca-data.json` (`src/shared/worktree/meta-types.ts`):
  display name, `comment`, linked issue/PR/Linear/Jira/GitLab/Bitbucket/Azure/Gitea item,
  `baseRef`, push target, `createdWithAgent`, parent lineage, board status, `diffComments`,
  archived/pinned/unread flags, and provenance (desktop, CLI, automation).
- Agent start: `spawnLocalStartupAndSetupTerminals` (`worktree-remote.ts:397`) opens the agent's
  terminal with the prompt on its argv (`claude '<prompt>'`) and pre-trusts the folder for
  Cursor, Copilot and Codex.
- Also: sparse-checkout presets for monorepos (`src/main/git/worktree-sparse-add.ts`),
  `orca worktree create --agent --prompt --base-branch --parent-worktree` for agents that spawn
  agents (`src/cli/specs/core.ts:90`), and `src/main/folder-upgrade-worktree-path.ts`, which keeps
  a plain-folder project's registered path as its primary worktree after the folder becomes a git
  repo.

**Good and bad.** Keeping the base in `branch.<b>.base` and creating with `--no-track` are small
and right. Collision handling covers branches that already have PRs. The cost is size: the local
create function alone is about 750 lines and every step branches for WSL, SSH and runtime hosts.
The warm pool spends up to three full checkouts of disk to hide a few seconds.

**Size.** A subsystem: about 22,000 lines in `src/main` for worktree create, list and remove,
tests excluded. The warm pool is about 1,100.

**Marley today: has part of it.** Zed's `crates/git_ui_core/src/worktree_service.rs` (1,792 lines)
creates a worktree under `git.worktree_directory` (default `../worktrees`,
`assets/settings/default.json:1855`, giving `<parent>/worktrees/<project>/<name>/<project>` per
`crates/project/src/git_store.rs:9144`), names it with `worktree_names.rs`, opens it as a new
workspace in the MultiWorkspace, carries trust over from the main checkout
(`maybe_propagate_worktree_trust`, `:632`), records it as Zed-created keyed by the gitdir's
creation time (`crates/git_ui_core/src/created_worktrees.rs`), and runs `.zed/tasks.json` tasks
tagged `"hooks": ["create_worktree"]` (`crates/task/src/task_template.rs:95`,
`crates/workspace/src/tasks.rs:235`). It always makes a detached HEAD
(`worktree_service.rs:504` uses `CreateWorktreeTarget::Detached`). The git layer has
`CreateWorktreeTarget::NewBranch` (`crates/git/src/repository.rs:308`), reached outside tests only
through the remote-project RPC handler (`crates/project/src/git_store.rs:3905`). Zed's sidebar
and Agent Panel use this service; Marley's rail does not.

### 2.2 Setup, default tabs and teardown hooks

**What the user sees.** The first time a repo's `orca.yaml` scripts would run, a trust dialog
lists them. Setup then runs in a "Setup" terminal tab (or a split) beside the agent, so its
output and failure are visible. Per-repo settings: run policy (ask, run by default, skip by
default), whether the agent waits for setup, and a local script that replaces or adds to the
shared one.

**How it works.**

- `orca.yaml` at the repo root, parsed by `src/shared/orca-yaml.ts`. Keys: `scripts.setup`,
  `scripts.archive`, `setupAgentStartupPolicy` (`start-immediately` or `wait-for-setup`),
  `issueCommand`, `defaultTabs` (`title`, `color`, `command`), `environmentRecipes` (VM recipes)
  and `worktree.sharedDirectories`. Orca's own file runs `node
  config/scripts/run-internal-dev-setup.mjs` and `pnpm install`.
- The file is read from the new worktree, that is from the branch being checked out, not the
  primary checkout (`worktree-remote.ts`, around `:3000`, citing #1280).
- Trust: the renderer hashes the script text (setup plus `defaultTabs` commands) and compares it
  with `trustedOrcaHooks[repoId][kind].contentHash`; changed text asks again
  (`src/renderer/src/lib/ensure-hooks-confirmed.ts`). A repo-wide "trust all" exists.
- Setup never runs hidden. Main writes a runner into the per-worktree gitdir
  (`git rev-parse --git-path orca/setup-runner.sh`, so it never shows in `git status`) with
  `set -e` and the script's own shebang flags replayed, and the terminal runs `bash <runner>`
  (`src/main/worktree-runner-script.ts`, `src/main/setup-runner-script-text.ts`). With
  `wait-for-setup` the agent's command is wrapped to start after setup ends.
- Env: `ORCA_ROOT_PATH` (primary checkout), `ORCA_WORKTREE_PATH`, `ORCA_WORKSPACE_NAME`, and
  `CONDUCTOR_ROOT_PATH` plus `GHOSTX_ROOT_PATH` so scripts written for Conductor run unchanged
  (`src/main/setup-hook-env-vars.ts`). Git credential prompts are forced off in that terminal.
- `defaultTabs`: terminals created once for a new worktree, for example a dev server, trust-gated
  with setup.
- Archive hook: `scripts.archive` runs in the worktree before removal under `/bin/bash` as a
  process-group leader, with a 2-minute deadline and a 10 MB output cap (`src/main/hooks.ts`).
  A failure or timeout blocks the removal (nothing is stopped or deleted) unless the user
  explicitly allows it (`src/main/worktree-archive-hook-gate.ts`, CLI
  `--allow-failed-archive-hook`). A hook that traps SIGTERM and exits 0 after the deadline counts
  as failed. The CLI skips archive hooks unless `--run-hooks` is given.
- `issueCommand`: a template run in the first terminal of a worktree made from a GitHub or GitLab
  issue, default `Complete {{artifact_url}}`, with a per-user override in `.orca/issue-command`
  that Orca checks is gitignored (`src/main/issue-command-file.ts`).

**Good and bad.** A visible setup terminal, hash-based trust and a fail-closed teardown are the
right calls. Setup knows nothing about ports or other per-worktree resources; a script has only
`ORCA_WORKSPACE_NAME` to derive them from.

**Size.** About 1,400 lines (hooks, parser, runner, trust).

**Marley today: has part of it.** Zed's `create_worktree` task hook covers setup, and in the
Marley layout tasks run in center terminals (`crates/marley_workbench/src/routing.rs`). There is
no teardown hook, no hash-based re-prompt when a hook changes, and no root-path variable
(tasks get `ZED_WORKTREE_ROOT` and friends).

### 2.3 Gitignored files: copy and share

**How it works.**

- `.worktreeinclude` at the repo root is Claude Code's convention (opencode reads it too, per
  Orca issue #7549). Orca supports literal paths only; globs and negation are skipped with a
  warning. At most 1,000 entries and a 256 KB file. Only paths that exist in the primary
  checkout and that `git check-ignore` reports as ignored are copied; symlinks are resolved and
  their content copied (`src/main/git/worktree-include-file.ts`). A copy budget refuses oversized
  trees before writing and reports skipped entries as a create warning
  (`src/main/ipc/worktree-include-copy-budget.ts`). macOS gets APFS clones.
- `worktree.sharedDirectories` in `orca.yaml`: gitignored directories symlinked from the primary
  checkout, for `node_modules` or `.cache` (`src/main/git/worktree-shared-directories.ts`).
- Per-user "Worktree Shared Paths" (`repo.symlinkPaths`): symlinks, or APFS clones on macOS.
- Before `git worktree remove`, Orca unlinks its own symlinks so git does not refuse on
  "untracked" links (`src/main/ipc/worktree-symlinks.ts:363`).

**Good and bad.** The gitignored-only rule keeps copies out of the diff. A symlinked
`node_modules` is shared mutable state: an agent running `pnpm add` in one worktree changes every
worktree's dependencies, and Orca does nothing about it.

**Size.** About 1,100 lines.

**Marley today: lacks it.**

### 2.4 Ports

**How it works.**

- Orca allocates no ports. It discovers them: on Linux it reads `/proc/net/tcp` and `tcp6` for
  listeners, maps socket inodes through `/proc/<pid>/fd`, and gives each port to the worktree whose
  path contains the owning process's cwd, else its command line
  (`src/main/ports/local-workspace-platform-port-scanner.ts`,
  `src/main/ports/local-workspace-port-attribution.ts`).
- The worktree card lists live ports with Open (in Orca's browser), Copy and Stop. Stop sends
  SIGTERM only after a fresh scan proves the pid still owns that port and belongs to a workspace
  (`src/main/ports/workspace-port-ownership.ts`,
  `src/renderer/src/components/sidebar/WorktreeCardPorts.tsx`).
- Terminal output is watched for dev-server URLs (`src/main/ports/advertised-url-watcher.ts`).
- Labeled URLs: a loopback HTTP proxy on a random port maps
  `http://<project>-<worktree>.orca.localhost:<proxy-port>/` to the worktree's port, WebSocket
  upgrades included, responses streamed untouched (`src/main/localhost-worktree-label-proxy.ts`).
  Browsers scope cookies by host and not by port, so apps on `localhost:3000` and
  `localhost:3001` share one cookie jar; distinct `*.localhost` hostnames give each worktree its
  own. Routes are accepted only for loopback or scanned workspace ports (commit `a56da57bb8`).

**Good and bad.** Discovery works with any dev server and no config. It does not prevent
collisions: the second `next dev` fails with EADDRINUSE or moves to another port. Orca issue
#19602 asks for Conductor-style "spotlight testing" (one dev environment, swap a worktree's changes
into it) because a full environment per worktree is slow for large apps.

**Size.** About 2,600 lines.

**Marley today: lacks it.** #510 plans a port offset.

### 2.5 Worktrees in the UI

- Sidebar: projects as top rows, worktrees under them, the main checkout as one of those rows.
  Filters (sleeping, default branch, automation- or CLI-created, detached HEAD), pinning,
  drag order, multi-select actions, double-click rename, `Cmd-J` jump palette, and a board of
  workspace statuses (todo, in progress, in review, completed).
- A card shows name, branch, agent rows with status dots, unread in bold, PR or MR chip with
  checks, linked issue, the note, live ports, and provenance
  (`src/renderer/src/components/sidebar/worktree-card-display-property-options.ts`).
- Parent and child worktrees nest (orchestration, `--parent-worktree`); sleep and delete can
  include descendants.
- Worktrees made outside Orca appear behind a "hidden worktrees" card. Claude Code's own
  worktrees under `<repo>/.claude/worktrees/` and `.gsd-workspaces` are hidden by default
  (`src/shared/agent-scratch-worktrees.ts`, `src/shared/worktree/visibility-sources.ts:23`).
- First-prompt rename: a worktree made with a generated name gets a branch named from the first
  prompt by a headless agent CLI. It refuses when the branch has an upstream, since `git branch
  -m` would orphan the remote branch and its PR (`src/main/agent-hooks/first-work-branch-rename.ts`,
  `src/main/git/branch-rename.ts`). The folder is renamed best-effort.
- Sleep closes a worktree's terminals and browser tabs to free memory; agents resume later from
  saved launch config.

**Size.** The sidebar alone is about 70,000 lines of TypeScript; Marley should not copy that
weight.

**Marley today: has part of it.** The rail lists projects, but `ProjectGroupKey` is the main
worktree's path (`crates/project/src/project.rs:6590`), so a linked worktree's workspace is a
member of its main project's group, and the rail lists its terminals under that project's row
(`crates/marley_workbench/src/rail.rs:1740-1800`). There is no per-worktree row, branch,
ahead/behind or ports.

### 2.6 Removing, archiving and cleaning up

**How it works** (`src/main/ipc/worktrees/removal/execute-worktree-removal.ts`,
`remove-registered-local-worktree.ts`):

1. Refuse a locked worktree; re-list and confirm the git registration still matches.
2. Run `scripts.archive`; a failure stops here.
3. Check the tree is clean, ignoring Orca's own symlinks. Dirty needs Force Delete.
4. Fence file watchers, then kill every process in the worktree (PTYs, runtime terminals,
   structured agent sessions) and verify they exited
   (`src/main/ipc/worktrees/removal/worktree-removal-ownership.ts:18`).
5. Unlink shared symlinks.
6. Rename the directory into a sibling `.orca-worktree-trash/wt-<ms>-<nonce>`, deregister with
   `git worktree remove --force` on the now-missing path, and delete the tree after the IPC
   returns (`src/main/worktree-trash.ts`; the comment cites 8 to 35 s spinners on multi-GB
   `node_modules`).
7. Branch: `git branch -d`. If git refuses, test whether it merged anyway against
   `branch.<b>.base`, `origin/HEAD` and `HEAD`, after a `fetch --prune` of the target's remote:
   `git merge-tree --write-tree <target> <branch>` equals the target's tree, or `git cherry` shows
   every commit patch-equivalent, or the branch's net patch-id matches a squash commit on the
   target (`src/shared/git-branch-cleanup.ts:109-282`). Proven merged means
   `git update-ref -d refs/heads/<b> <expected-oid>`, a compare-and-swap that will not delete a
   branch that moved (`src/main/git/worktree-branch-removal.ts:131`). Not proven means the branch
   stays and a "Review N Branches" toast lists it (`PreservedBranchBatchReviewDialog.tsx`).
8. Drop the metadata, localhost label routes, terminal history and PR refresh state.

A worktree created on a pre-existing branch sets `preserveBranchOnDelete`. "Archive" is an
`isArchived` flag; it only feeds cleanup (archived and idle 7 days). Resource Manager's "Clean up
workspaces" scans every worktree for size, last activity and git state, with blockers for dirty
files, unpushed commits, unknown base, a live agent, a running terminal, an unsaved editor
buffer, pending diff notes, pinned and main (`src/shared/workspace-cleanup.ts`); 30 days idle and
clean makes a candidate.

**Good and bad.** Everything fails closed, and squash-merge detection makes "delete the merged
worktree" one click without losing unmerged work. The price is complexity, with bugs like #21954
(a failed PTY kill leaves finished worktrees undeletable).

**Size.** About 1,800 lines in the removal handlers, 840 in git removal and branch cleanup, 4,500
in the cleanup scanner.

**Marley today: has part of it.** Zed's `crates/agent_ui/src/thread_worktree_archive.rs` (1,747
lines), used when a thread is archived: it saves the staged and unstaged state as two detached
commits held by `refs/archived-worktrees/<id>` on the main repo (`:499`), releases the worktree
from every open project, checks that Zed created it, and runs `git worktree remove --force`;
restore recreates it (`:645`). It has no branch step because Zed's worktrees are detached. The
rail exposes none of it.

### 2.7 "Worktree checkpoints"

It has nothing to do with git. It is the `comment` field of `WorktreeMeta`, which agents set with
`orca worktree set --worktree active --comment "..." --workspace-status in-progress`
(`src/cli/specs/core.ts:141`), shown as Notes on the card. The docs tell agents to read it first
with `orca worktree current --json` and merge rather than overwrite. Orca has no git snapshots at
all: no `write-tree`, `commit-tree` or stash; its only private refs are `refs/orca/pull/*`,
`refs/orca/merge-requests/*` and short-lived `refs/orca/rebase/<uuid>`.

**Marley today: lacks the note.** It has real git checkpoints through Zed (item 4 of section 3).

### 2.8 Diff viewer

- Monaco diff editors. "View all" is a virtualized list of per-file sections loaded on demand,
  with a file tree (`src/renderer/src/components/editor/combined-diff/`, about 5,400 lines).
- Scopes: unstaged, staged, and branch, computed as `merge-base(base, HEAD)..HEAD`
  (`src/main/git/source-control/branch-compare.ts`), with a selectable compare base,
  ahead/behind counts, and line totals split into source, tests and generated by path.
- Image diffs (side by side, swipe, onion skin), HTML preview in a browser split, a three-way
  conflict view, F7 navigation, and a "Changes" mode on a normal editor tab.
- The mobile app keeps per-file "reviewed" marks that reset when the file's diff identity changes
  (`mobile/src/session/mobile-diff-review-state.ts`); desktop does not.
- The native chat surface shows a per-turn "N changed files" roll-up summed from the file changes
  the agent reported into its session journal (`diff` items)
  (`src/renderer/src/components/native-chat/native-chat-turn-diffs.ts`), not from git.

**Marley today: has more.** Zed's multibuffer diffs: uncommitted changes, `BranchDiff` "Changes
since {branch}" with a base picker (`crates/git_ui/src/branch_diff.rs`), staged and unstaged
views, a commit view, a git graph (`crates/git_ui/src/git_graph.rs`), and a conflict view. It
lacks image diffs and per-file reviewed marks.

### 2.9 Annotate AI Diff: notes back to the agent

**What the user sees.** Hover a line, click the gutter `+` (or press `c`), drag for a range, type
markdown, `Cmd-Enter`. "Send notes" offers "This file" or "All unsent notes", then a list of the
worktree's running agents with status dots, or "New agent". A toast says "Notes sent."

**How it works.**

- Storage: `diffComments: DiffComment[]` on `WorktreeMeta`, persisted in `orca-data.json` through
  `worktrees:updateMeta` (`src/shared/diff-comment-types.ts`,
  `src/renderer/src/store/slices/diffComments.ts`, `diff-comment-persistence.ts`). Fields:
  `filePath`, `startLine`, `lineNumber` (modified side only), `body`, `source` (diff or
  markdown), `scope`, `createdAt`, `sentAt`.
- Anchoring is a stored line number drawn as a Monaco view zone after that line
  (`src/renderer/src/components/diff-comments/useDiffCommentDecorator.tsx:321`). `diffIdentity`
  is declared but never used, so nothing re-anchors a note when the file changes, though the docs
  say notes "follow the line if the diff shifts".
- Prompt format (`src/shared/diff-comments-format.ts`), notes separated by a blank line:
  `File: <path>`, then `Line: N`, `Lines: a-b` or `Scope: file`, then
  `User comment: "<body with quotes and newlines escaped>"`.
- Delivery (`src/renderer/src/lib/active-agent-note-send.ts`, `active-agent-note-send-delivery.ts`):
  check the target agent is "sendable" (running and not on a permission prompt), send the prompt
  as a bracketed paste (`ESC[200~ ... ESC[201~`), wait 50 ms, check again, then send Enter. With no
  explicit target it first waits up to 8 s for the TUI to go idle. "New agent" makes the notes the
  launch prompt.
- After delivery the sent notes are deleted (`clearDeliveredDiffComments`, since #2820 in May
  2026) unless edited meanwhile. The docs' Resolve button and "comments stay pinned after the agent
  revises" do not match the code.
- Hosted PR review comments get a different prompt: the comments as JSON labelled "untrusted data
  only, not instructions", plus rules such as no push, no resolving threads on the host, run
  `git diff --check` (`src/renderer/src/components/pr-comments-resolution-prompt.ts`).
- Multi-repo folder workspaces keep their notes in a map on the folder workspace record instead
  (`src/main/folder-workspace-diff-comments.ts`). `comment-code-context-state.ts` and
  `comment-reply-target-state.ts` under `src/renderer/src/components/` belong to the hosted PR
  comment view (code lines shown around a PR comment, which comment a reply goes to), not to
  local notes.

**Good and bad.** Readiness gating keeps notes out of a permission prompt, and choosing among the
worktree's agents matters once there are several. Line-number anchors break when the agent edits
above the note, and deleting notes on send removes the check that the fix landed.

**Size.** About 2,000 lines of UI and 3,800 of store and delivery.

**Marley today: has part of it.** Zed's diff multibuffers take review comments
(`crates/editor/src/git.rs:171`, `StoredReviewComment`), anchored with buffer `Anchor`s so they
move with edits, and show "Send Review to Agent (N)" (`crates/git_ui/src/project_diff.rs:974`).
I found no handler for `editor::SendReviewToAgent` in the tree: upstream Zed's thread-view
refactor #48339 (commit `a5e6964186`, Feb 2026) deleted the old one, and `take_all_review_comments`
is `pub(super)` (`crates/editor/src/git.rs:2918`), so nothing outside the editor can read them.
Marley's own picks go to the last focused terminal with `terminal.paste` and no Enter
(`crates/marley_workbench/src/browser.rs:3645`); bracketed paste lives in
`crates/marley_terminal/src/keys.rs:188`.

### 2.10 Attribution

`docs/site/content/docs/review/attribution.mdx` describes a gutter marker on AI-written lines,
human edits flipping it back, and an export. I found no code for any of it: no range store and no
decoration, and the page has not changed since the docs import (commit `6aba202d5e`). What did
exist was a `git` and `gh` PATH shim in Orca terminals that added commit trailers and PR and issue
footers (`ORCA_GIT_COMMIT_TRAILER`, `ORCA_GH_PR_FOOTER`). It went off by default in April 2026
(`cfe247659a`) and was removed in August (#14255, `4882eeb8ac`); tombstone wrappers still
neutralize old installs (`src/main/pty/legacy-terminal-shim-dir.ts`). Orca knows who wrote a
change only per worktree (which agent ran there) and per native-chat turn (the transcript's edit
calls).

**Marley today: lacks it.** Zed's `action_log` tracks Agent Panel edits for Accept and Reject;
terminal agents get nothing.

### 2.11 Commit, push, AI text, PR and merge

- Source Control panel: stage, unstage, discard, commit (`Cmd-Enter`), a primary button that moves
  from Stage to Commit to Push, Pull or Sync, explicit amend, "Force push with lease" as a separate
  action showing how many commits it replaces, and abort for a merge or rebase.
- Generate with AI runs the chosen agent CLI headless, for Claude
  `claude -p --output-format text --model sonnet --permission-mode plan` with the prompt on stdin:
  fixed rules plus the staged diff cut to 200 KB, shared fairly across files
  (`src/shared/commit-message-agent-specs-primary.ts:31`, `src/shared/commit-message-prompt.ts`).
  PR title and body come the same way from the branch diff and commit list. This works on the
  user's agent login, with no API key.
- Action recipes: for each action (commit message, PR, branch name, fix commit failure, fix push
  failure, fix checks, resolve conflicts, resolve comments) the user picks the agent, CLI args
  and a template with `{basePrompt}`, `{branch}`, `{stagedFiles}`, `{stagedPatch}`,
  `{linkedIssue}`, `{baseBranch}`, `{commitSummary}`, `{changedFiles}`, `{patch}`, globally or
  per repo (`src/shared/source-control-ai-action-variables.ts`).
- Fix with AI starts an agent with the hook output, attempted message and staged files. Resolve
  with AI hands over the conflicts with rules: start with `git status`, the continue or skip
  command, no `reset --hard`, stash or abort, no push, run `git diff --check`
  (`src/shared/source-control-conflict-prompts.ts`). Fix broken checks sends failed check names
  and log tails.
- Update from base fetches the base into a private `refs/orca/rebase/<uuid>` and runs
  `git rebase --onto <ref> <fork-point>` (`src/main/git/remote-rebase.ts`).
- Base drift: status events `current`, `drift` or `base_changed` with a behind count and recent
  subjects (`src/shared/worktree/base-ref-drift-types.ts`).
- PR creation is template-aware, with stacked PRs on GitHub. Merge is
  `gh pr merge <n> --squash|--merge|--rebase` without `--delete-branch`, since that would delete
  the local branch the worktree has checked out (`src/main/github/client/merge/merge-pr.ts:84`).
  It refuses with reasons (review required, changes requested, merge queue, conflicts) and offers
  auto-merge and the merge queue.
- No local merge exists anywhere. The "race three agents" recipe
  (`docs/site/content/docs/recipes/parallel-agents.mdx`) ends with a PR from the winner and
  deleting the other two worktrees.

**Marley today: has much of it.** Zed commits, amends, signs off, and pushes with
`--force-with-lease` (`crates/git/src/repository.rs:2847`); Generate Commit Message uses Zed's
configured model (`crates/git_ui/src/git_panel.rs:3858`); Create Pull Request opens the host's
compare page (`crates/git_hosting_providers`); the conflict view and branch diff can open Agent
Panel threads (`ResolveConflictsWithAgent`, `ReviewBranchDiff`,
`crates/zed_actions/src/lib.rs:623,643`). It lacks a local merge of a worktree branch and any PR
status or merge in the app.

### 2.12 Hosted review and task integrations

| Provider | What Orca shows and does | Auth | Source |
|---|---|---|---|
| GitHub | PR per worktree (state, checks, reviews, threaded comments with replies and reactions, per-file viewed marks), issues with timeline, Actions logs, a Projects v2 "Tasks" board, stacked PRs, auto-merge | `gh` CLI (keyring or `GH_TOKEN`); optional per-project account binding that resolves a short-lived `gh auth token --user` into the child env only; a rate-limit circuit breaker per REST, GraphQL and search bucket | `src/main/github/` (about 19,000 lines) |
| GitLab | MRs, issues, pipelines including child pipelines, job logs | `glab` CLI | `src/main/gitlab/` (about 4,700) |
| Bitbucket Cloud | PRs, PR creation | email plus API token or access token, encrypted with Electron `safeStorage`; `ORCA_BITBUCKET_*` env wins | `src/main/bitbucket/` |
| Azure DevOps, Gitea | PR status in the sidebar and Checks | env only: `ORCA_AZURE_DEVOPS_TOKEN`/`_PAT`/`_ACCESS_TOKEN`, `ORCA_GITEA_TOKEN` | `src/main/azure-devops/`, `src/main/gitea/` |
| Linear | list and board views, edit fields, create; Linear's branch name for the worktree; issue images passed into the agent's prompt; `orca linear` CLI for agents | personal API key in `safeStorage`, GraphQL | `src/main/linear/` (about 6,800) |
| Jira | Cloud and Server/DC issues, transitions, comments, ADF to markdown | Cloud: email plus API token; Server/DC: bearer PAT or basic; `safeStorage` | `src/main/jira/` (about 3,100) |

Any task row (GitHub issue, PR or project card, GitLab, Linear, Jira) opens the composer
prefilled with name, link and branch; it never creates silently.

**Marley today: lacks it.** Zed's hosting providers build permalinks, blame PR links and
create-PR URLs; nothing shows PR status, checks or issues.

### 2.13 Other git details worth knowing

- Terminals whose launch command is a known agent get git credential prompts turned off
  (`GIT_TERMINAL_PROMPT=0`, `GCM_INTERACTIVE=never` and indexed git config), so an agent's
  `git push` fails instead of hanging on a prompt it cannot see
  (`src/shared/terminal-git-credential-guard.ts`).
- Claude Code hook events Orca registers (`src/main/claude/hook-settings.ts`): `SessionStart`,
  `UserPromptSubmit`, `Stop`, `StopFailure`, `SubagentStart`, `SubagentStop`, `TeammateIdle`,
  `PreToolUse`, `PostToolUse`, `PostToolUseFailure`, `PermissionRequest`, `PostCompact`,
  `SessionEnd`. The comments record that a turn ending in an API error fires `StopFailure` and no
  `Stop`, and a manual `/compact` ends without `Stop`. An Esc interrupt fires nothing for the
  cancelled turn: in Orca's recorded Claude Code 2.1.280 sessions
  (`src/shared/__fixtures__/claude-cancel-shell-hooks.jsonl`) the event after the cancel is the
  next `UserPromptSubmit`. Orca infers interrupts from keystrokes with a 500 ms settle window
  (`src/shared/agent-interrupt-intent.ts`). This matters for #509.

## 3. Bring to Marley

Ranked. "Queued" marks rows inside tickets #509 to #511.

1. **Worktree agent on its own branch, as its own rail row (queued: #510).** Two agents in one
   checkout overwrite each other's files; this is what lets them run side by side. Seam:
   `crates/marley_workbench` calls Zed's `create_worktree_workspace`
   (`crates/git_ui_core/src/worktree_service.rs:736`) and then starts the agent in the new
   workspace's first center terminal (routing already seeds one). Size M. What Orca teaches for
   the ticket:
   - Create a branch, not Zed's detached HEAD: add a `NewBranch` option that reaches
     `CreateWorktreeTarget::NewBranch`, a small additive Zed touch. #511 needs the branch.
   - Name it `<prefix>/<generated-name>`, create with `--no-track`, and write
     `branch.<branch>.base <base>` into git config so #511 and branch cleanup know the base
     without Marley state.
   - The rail groups by the main worktree path today, so the ticket must choose a nested row
     under the project or a separate top-level project (open question 1). Orca nests.
   - Hide Claude Code's own `.claude/worktrees/*` worktrees from the rail, as Orca does; its
     subagents make them.
   - Keep Zed's location (`../worktrees`); leave Orca's warm pool out unless creation turns out
     slow on the Zed-sized repo.
   - Hard part: Claude Code asks to trust a folder it has not seen. Orca writes trust into the
     agent's own config before launch for Cursor, Copilot and Codex (`worktree-remote.ts:442-446`).
     Check whether a trusted `/srv/stacks` covers `/srv/stacks/worktrees/...` on this box before
     the first worktree agent stalls on that prompt.
2. **Remove a worktree safely, with squash-aware branch deletion (queued: #511).** One-click
   cleanup that never loses unmerged work. Seam: a Rust port of `src/shared/git-branch-cleanup.ts`
   (about 280 lines of git plumbing) in a Marley crate, plus Zed's
   `thread_worktree_archive::remove_root` to release the worktree from every open project. Size M.
   Order to copy from Orca: stop the worktree's terminals and agents and wait for them to exit,
   refuse a dirty tree unless confirmed, `git worktree remove`, `git branch -d`; when git refuses,
   prove the branch merged with `git merge-tree --write-tree`, `git cherry` or a patch-id match
   against the base, then `git update-ref -d refs/heads/<b> <oid>`; otherwise keep the branch and
   say so. Hard part: an agent process that will not die holds the directory (Orca #21954).
3. **Local merge into the base (queued: #511).** The ticket asks for it and Orca has nothing to
   copy, since its only merge is `gh pr merge`. Seam: the rail row's Merge, running git in the
   main checkout. Size M. Hard parts: a dirty main checkout, and conflicts. Include: require the
   main checkout clean and on the base branch; show ahead count and how far the base moved since
   branching (Orca's drift event); merge with `--no-ff` or `--squash` (open question 2); on
   conflict run `git merge --abort` in the main checkout and send the worktree's agent an
   Orca-style conflict prompt (merge the base into the branch there, resolve, continue, no reset
   or stash), then retry. An "Update from base" step (fetch into a private ref, rebase onto it)
   is cheap to add.
   For three agents raced on one task, a compare list of sibling worktree rows with diff stats,
   one Merge, and Remove for the rest covers Orca's recipe.
4. **Per-turn git checkpoints for Claude Code in a terminal (queued: #509).** Chad sees what each
   turn changed and can review turn by turn. Seam: the plugin's
   `hooks.json` plus Zed's `GitStore::checkpoint` (`crates/project/src/git_store.rs:2133`) and
   `Repository::diff_checkpoints` (`:10027`). A checkpoint is a temporary-index `write-tree` plus
   `commit-tree` that touches neither the user's index nor any ref, and skips untracked files over
   2 MB (`crates/git/src/repository.rs:3063`, `:3774`). Size M. What Orca's hook notes teach:
   - Add `UserPromptSubmit` for the turn's start; today the plugin has only `Notification` and
     `Stop`.
   - Close a turn at the next `UserPromptSubmit` as well as at `Stop` and `StopFailure`, because
     an Esc interrupt fires nothing for the cancelled turn and an API error fires no `Stop`.
   - Send the event through the existing `terminalSequence` OSC channel so Marley knows which
     terminal and project it belongs to. The hook can forward its stdin (session id, prompt,
     transcript path) base64-encoded, so the script still parses nothing.
   - Pin checkpoints with refs such as `refs/marley/turns/<session>/<n>` if they must outlive gc
     or a restart (open question 5).
   - Title each turn with its prompt, list turns under the terminal's rail row, and open a turn
     in a diff multibuffer so the review comments of item 5 work on it.
   - A checkpoint diff holds every change in the tree during the turn, the user's and other
     agents' included. Orca's native chat avoids that by summing the file changes the agent
     reported, which misses anything the agent changed through shell commands. Worktrees (#510)
     remove the overlap; the view should say when the turn ran in a shared checkout.
5. **Review notes into the agent's terminal (not a ticket yet; it belongs with the review views
   of #509 and #511).** Notes back to the agent close the review loop, and Zed draws the "Send
   Review to Agent" button, but I found nothing that handles its action. Seam: a handler for
   `editor::SendReviewToAgent` in `marley_workbench`, a pub accessor for the stored comments (small
   Zed touch), and a target picker over the rail's agent terminals. Size S to M. Hard part: the
   agent's readiness, which Marley knows only from the plugin's hooks. Take from Orca: the plain
   `File:`/`Line:`/`User comment:` format; bracketed paste then Enter only when the agent's status
   is idle or waiting and never while it sits on a permission prompt (Marley's plugin already
   reports both); fall back to an Agent Panel thread. Leave out Orca's delete-on-send: keep the
   notes, marked sent, since Zed's anchors follow edits and they check the fix.
6. **`.worktreeinclude`, setup env, and a teardown hook (queued: #510 and #511).** A fresh
   worktree lacks `.env`, which breaks dev servers first. Seam: the create path in item 1 and
   Zed's `create_worktree` tasks. Size S each. Copy gitignored paths listed in `.worktreeinclude`
   (the Claude Code format, globs included, which beats Orca's literal-only subset) only when
   `git check-ignore` agrees, with a size cap and a warning for what was skipped. Export
   `MARLEY_ROOT_PATH` and `MARLEY_WORKTREE_PATH` to setup tasks. Add a teardown hook that runs
   before removal and blocks it on failure, with a deadline.
7. **Ports: offset plus discovery (queued: #510).** Two worktree agents that each start a dev
   server want the same port. Seam: the worktree's terminal and task env for
   the offset (`PORT`, `MARLEY_PORT_OFFSET`); a Rust port of Orca's `/proc` scan that pins
   listeners to rail rows by process cwd; the Browser tab to open them. Size M. Orca teaches that
   discovery matters more than allocation, since Vite and Next move to a free port on their own
   and many apps ignore `PORT`. It also teaches the cookie trap: two worktree apps on different
   localhost ports share cookies, so open them as `<worktree>.localhost:<port>` (Chromium
   resolves `*.localhost` to loopback) or in the per-project browser context of #507.
8. **Agents' credential prompts off.** Seam: `marley_terminal` env for agent launches from the
   rail and the agent bar. Size S. An agent's `git push` then fails with a message the agent can
   read instead of hanging.
9. **A status note agents can set.** Orca's "checkpoint". Seam: a `worktree_note` tool in
   `marley_mcp` shown on the worktree's rail row. Size S. Useful once five worktrees run at once.
10. **Commit and PR text through `claude -p`.** Only if Zed has no model provider configured on the
    box: Orca's spec runs on the Claude Code login. Seam: an alternative backend for Zed's
    Generate Commit Message. Size S.
11. **Branch named from the first prompt.** Creation needs no name, and the branch still ends up
    readable. Seam: the #509 `UserPromptSubmit` event plus a headless `claude -p --model haiku`
    call; rename only while the branch has no upstream. Size M. Later.
12. **PR status and merge for a worktree's branch through `gh`.** Seam: rail row chip and a Merge
    that calls `gh pr merge` without `--delete-branch`. Size M to L. Later, if Chad moves from local
    merges to PRs.

## 4. Skip

- The warm checkout pool: 1,100 lines and three spare checkouts of disk to save seconds.
- WSL, Windows, APFS and multi-host (SSH relay, runtime) branches: Marley runs local on Linux.
- GitLab, Bitbucket, Azure DevOps, Gitea and Jira: Chad's repos are on GitHub.
- GitHub Projects board, stacked PRs, merge queue, reactions: team features.
- Workspace board statuses, emoji names, parent and child nesting, pinned duplicates: sidebar
  weight (Orca's sidebar is 70,000 lines).
- The Resource Manager cleanup scanner: overkill for a handful of worktrees; reuse its blocker
  list in the remove dialog only.
- Per-line AI attribution: Orca never built it, and it removed its own git and gh trailer shim.
- Action-recipe templates for eight git AI actions: two fixed prompts (conflicts, review notes)
  cover Marley's needs.
- Deleting diff notes on send: it removes the check that the fix landed.
- Symlinking `node_modules` by default: one agent's install changes every worktree.

## 5. Open questions for Chad

1. Should a worktree agent be a row nested under its project (Orca's model, and how Zed already
   groups it) or its own top-level project as #510 says?
2. For #511's Merge into the main checkout: merge commit, squash, or rebase then fast-forward?
   Push afterwards, or leave that to you?
3. Worktree agents building Marley itself share the box's one cargo and one target directory.
   Queue them, or give each worktree its own target directory at the cost of disk?
4. Port offset: a fixed step per worktree exported as `PORT`, or discovery only?
5. Per-turn checkpoints: keep them for the session only, or pin them with refs so they survive a
   restart (objects pile up until pruned)?
6. Review notes: into the agent's terminal only, or also into Agent Panel threads?
