# T3 Code survey 03: checkpoints, worktrees, git and pull requests

Read from a local clone of `github.com/pingdotgg/t3code` at `17c0878941` (2026-10-06). Paths are relative to that
repo root unless they start with `crates/` (Marley). Line counts are non-test TypeScript.

## 1. Summary

Every T3 Code thread works either in the project's checkout or in a git worktree the server makes
for it under a temporary `t3/<hash>` branch, which a model renames in the background from the
first message. Each turn ends with a checkpoint: a commit of the whole tree written through a
private index and kept under a hidden ref, so the diff panel can show any turn, the uncommitted
changes or the branch, and **Edit from here** can rewind the conversation and, in an isolated
worktree only, the files. A thread's Git actions commit, push and open a pull request with a
generated title and body. A Pull Requests page is a review client for GitHub, GitLab,
Forgejo/Gitea, Bitbucket and Azure DevOps behind one provider contract. Threads link pull requests,
the server keeps them in sync and settles a thread when its pull request merges, and an agent can
ask the server to watch its pull request and wake it when checks fail or pass, someone comments, or
the branch starts to conflict. Cleanup policies remove idle or merged worktrees but keep the
branch, and the next turn recreates the checkout. The area is large: about 20,600 lines of host
providers, 8,400 of source-control discovery, 6,800 of the git driver, 3,500 of git workflow,
3,650 of checkpoint and pull-request orchestration, and 16,900 of review UI. Marley is ahead of
T3 Code on the local half of the worktree lifecycle: T3 Code has no local merge, its cleanup cannot
see squash merges, and it has no drift chips, port slots or teardown hooks. Worth taking: (1) undo
one turn's files from the Turns list, refused while another agent shares the checkout; (2) a
worktree's pull request and its checks on its rail row, with Open Pull Request and generated text;
(3) a pull-request watch that wakes an idle terminal agent when CI or a reviewer needs it; (4) new
worktrees started from the fetched base; (5) the branch named from the first prompt after the
worktree exists; (6) viewed marks in Review that a later turn clears; (7) review notes that carry
their hunk.

## 2. Features

### 2.1 Checkpoints as hidden refs

**What the user sees.** Nothing directly. Turns in a git project get diffs and a rewind point;
threads without git get neither.

**How it works.** `apps/server/src/orchestration-v2/CheckpointService.ts:25` names the refs
`refs/t3/orchestration-v2/checkpoints/<base64url scope key>/ordinal/<n>` (line 166). A scope is a
working directory; ordinal 0 is the thread's start and ordinal *n* the end of run *n*.
`CheckpointCaptureService.ts:106-125` materializes a missing baseline before the end checkpoint,
so the first turn of a thread always has a "before". The git work is in
`apps/server/src/vcs/GitVcsDriver.ts:806-1070`:

- a private index `t3-checkpoint-index-<uuid>` in the common git dir, with author and committer
  "T3 Code" (lines 816-824);
- the real index is copied in, `read-tree --reset HEAD` keeps its stat data only where it matches
  `HEAD`, and its racy timestamp is restored afterwards, so `git add -A` on a large tree does not
  rehash every file (lines 852-870);
- sparse checkouts are handled (cone mode only; a non-cone sparse index refuses rather than record
  false deletions), and an embedded repository with no commit, which makes `git add` fail, is
  retried with that folder excluded under one timeout (lines 959-1027);
- `write-tree`, `commit-tree -m "t3 checkpoint ref=…"`, `update-ref` (lines 1032, 1050, 1067), all
  with `core.fsync=objects,reference`.

Restore (`GitVcsDriver.ts:1077-1151`) runs `git restore --source <commit> --worktree --staged`,
then `git clean -fd` (line 1119), then `git reset` so the index matches `HEAD` again. Diffs are
`git diff --patch` or `--numstat -z` between two checkpoint commits, with an ignore-whitespace
option and an output cap (lines 1153-1217).

**Good.** The index reuse and the fsync flags show care for big repositories and power loss. The
"materialize the baseline lazily" rule means no checkpoint is taken for threads that never edit.
**Bad.** `clean -fd` on restore deletes untracked files created after the checkpoint, which is
the point, and also any untracked file the user made meanwhile; the isolation rule in 2.3 is what
keeps that safe.
**Size.** 587 lines in `apps/server/src/checkpointing/`, about 1,250 more across the capture,
rollback and safety services.
**Marley today: has.** #509 (guide, "Per-turn diffs") takes a checkpoint of the innermost
repository at each prompt and turn end through Zed's `GitStore::checkpoint`, which also copies the
index into a temporary one (`crates/git/src/repository.rs`, `with_temp_index`), and pins each
changed turn as a commit under `refs/marley/turns/<session>/<n>`. Marley leaves out large and
binary untracked files, as Zed does. Only Claude Code's turns are recorded; Codex in a terminal
gets none.

### 2.2 The diff panel

**What the user sees.** A right panel with one scope menu: **Turn N** for any turn (the latest
first), **Uncommitted**, and **Branch** against the base. A file tree, collapsible files, an
ignore-whitespace toggle, and a comment box on any line range. A comment becomes a chip in the
composer at the cursor, sent with the next message.

**How it works.** `apps/web/src/components/DiffPanel.tsx:223-248` resolves the scope and titles
it; turn diffs come from the checkpoint refs, the other two from git. The comment record
(`packages/contracts/src/composerContext.ts:191-205`) carries the section ("Turn 3", "Branch"),
the file, the line range, the comment and the diff text of the commented range, bounded, so the
agent sees the hunk the reviewer saw even if the lines have moved since. Pull-request review
comments use the same record with the pull request attached.

**Size.** `DiffPanel.tsx` is 1,193 lines; the diff components another 2,000 or so.
**Marley today: has.** The three scopes are three views: the Turns list under a terminal's rail row
(#509), Zed's project diff for uncommitted changes, and Review on a worktree row for the branch
(#511). Notes made with Add Review go to an idle Claude Code with **Send Review to Agent** (#522,
guide "Review notes to the agent"); the prompt gives `File:`, `Line:` or `Lines:` and the comment,
without the hunk.

### 2.3 Edit from here: rewind with or without the files

**What the user sees.** **Edit from here** under a sent message asks "Edit from here?" with
**Revert files too** and **Revert and keep changes**
(`apps/web/src/components/ChatView.tsx:11705-11730`). The message and its attachments return to
the composer; later messages leave the thread and the provider's history. **Revert files too** is
offered only for a thread in its own worktree, and is refused when another thread or live session
uses that folder, a folder inside it or one around it: "File restore requires an isolated
worktree. This workspace may contain changes from another thread. Rewind the conversation without
restoring files instead."

**How it works.** `apps/server/src/orchestration-v2/CheckpointRestoreSafety.ts:13-88` resolves real
paths and walks every other thread, archived ones too: their worktree paths, every checkpoint
scope's cwd, the cwd of each live provider session that cannot be shared, and the project root for
threads without a worktree. Any containment either way makes the restore unsafe. The check runs at
admission and again before the provider rollback (`CheckpointRollbackService.ts:158-170`), then
the provider rewinds its conversation (`session.rollbackThread`, line 260) and only then are the
files restored (line 265). A provider that cannot roll back its conversation (ACP agents,
Antigravity) is refused before any file changes (`docs/internals/overview.md`, "Turn completion
and checkpoints").

**Good.** Pairing the conversation and the files, and refusing a file restore in a shared checkout,
is the right rule; the containment check in both directions catches nested worktrees.
**Marley today: lacks for files, has for conversations.** Marley's turns can be viewed but not
undone. In the terminal, Claude Code's own `/rewind` rewinds its conversation and the edits its
file tools made, but not what shell commands changed; Marley's turn commits hold the whole tree's
change, shell edits included. Zed's Agent Panel offers Restore Checkpoint and Edit for its own
threads (`crates/acp_thread/src/acp_thread.rs`, `restore_checkpoint`;
`crates/agent_ui/src/conversation_view/thread_view.rs`).

### 2.4 A worktree per thread, named after the work

**What the user sees.** The composer's workspace menu offers the current checkout, **New
worktree**, and the previous worktree (`mod+shift+l` reuses it directly). A new worktree starts on
a temporary branch; seconds later the branch carries a name such as `t3/fix-login-redirect`, or
`feat/fix-login-redirect` with semantic prefixes, or whatever the user's own naming instructions
ask for (Settings, Source Control, Worktree branch naming). Shift-clicking several models sends
one prompt to each, each in its own worktree. **Start from origin** is a stored default.

**How it works.** `apps/server/src/orchestration-v2/ThreadLaunchService.ts:306-316` invents
`t3/<hash>` when the client named no branch, "so the worktree never waits on name generation".
With **Start from origin** it fetches the base from `origin` and starts the worktree at the
remote-tracking commit when one exists, falling back to the local base (lines 334-372). After the
worktree exists, lines 427-466 ask the text-generation model for a name, rename the branch with
`git.renameBranch` and record it; failures are logged and the temporary name stays. The prompt
(`apps/server/src/textGeneration/TextGenerationPrompts.ts:188-219`) has three modes: a fragment
the app prefixes (`t3/` by default, `packages/shared/src/git.ts:14`), a fragment with a semantic
prefix chosen by the model, or a complete name from the user's instructions.

**Good.** Creating first and naming later keeps the turn's start off the model's latency, and the
rename only touches a branch that nothing else knows yet.
**Marley today: has part.** New Agent in Worktree (#510, guide "Worktree agents") makes
`agent/<name>` from a name Zed makes up, on the main checkout's branch, local. The Orca survey
proposed naming the branch from the first prompt (`docs/orca_architecture/02-worktrees-and-review.md`,
item 11); it was not built. Marley never fetches, by design (#560).

### 2.5 Worktree setup: `t3.json`, progress and submodules

**What the user sees.** A checked-in `t3.json` names project scripts. One marked
`runOnWorktreeCreate` runs in a terminal after each new worktree. The thread shows setup as stages
(fetch, checkout with a percentage, submodules, setup script with its last output lines, agent),
on every device and after a reload. A project setting chooses recursive, top-level or no
submodules for new worktrees, and **Storage, Worktree location** moves the worktrees folder.

**How it works.** `packages/contracts/src/t3ProjectFile.ts:29-67` defines a script: `name`,
`command`, `icon`, `runOnWorktreeCreate` (line 40), `async` (line 46: true by default, "the agent
starts while the script is still running"; false holds the agent until it exits), and
`previewUrl` with `autoOpenPreview` (lines 52-62) to open the in-app browser. The file can also
set `defaultThreadEnvMode` (worktree or local) and `worktreeSubmodules` (lines 84-95), resolved
project override first, then environment, then the file, then the built-in default (lines
104-122). T3 Code's own `t3.json` runs `scripts/setup-worktree.ts`, which installs, then symlinks
the main checkout's `.env` files into the worktree using `T3CODE_PROJECT_ROOT`. Progress is
`packages/contracts/src/worktreeSetup.ts:16-22` (stage ids) and
`apps/server/src/project/WorktreeSetupTracker.ts` (366 lines), with the checkout percentage parsed
from git's `Updating files` lines.

**Marley today: has.** `.worktreeinclude` copies ignored files, the setup command for a lockfile,
Zed's `create_worktree` tasks, `MARLEY_ROOT_PATH` and `MARLEY_WORKTREE_PATH`, a port slot each
(#585, #590), and a teardown task before removal (#591), which T3 Code lacks. The setup command
runs in the agent's terminal (`pnpm install && claude …`), so its output shows as blocks; Marley
always holds the agent until setup succeeds. A launch config's `browser` item does what
`previewUrl` does (#527, guide "Launch configs"). Zed's `git.worktree_directory` sets the
location. Marley has no submodule policy.

### 2.6 Worktree cleanup, and the checkout that comes back

**What the user sees.** Settings, Storage: per machine or per project, remove worktrees after N
inactive days, after their pull request merges, or when they have no commits beyond the default
branch; and **Delete worktrees with deleted threads**. Off by default. Branch and thread history
stay; the next turn in that thread recreates the checkout.

**How it works.** `apps/server/src/storageCleanup.ts` runs at startup, on a settings change and
hourly. A candidate is skipped when any of these hold: the thread is not idle, a terminal is open in
the folder, the folder is a symlink or outside the managed roots, it contains a project root, it is
a main checkout (its `.git` must be a file), the branch moved, the tree has changes, or it holds
ignored files other than `node_modules` ("Ignored files can contain secrets or local datasets",
lines 252-266). "Merged" and "no commits" both need `HEAD` to be an ancestor of the freshly fetched
`refs/remotes/<remote>/<default>` (lines 271-306), so a squash merge does not count; the docs admit
it and suggest the inactivity rule instead. Before removing, every check is read again after the
git and host calls, along with live provider sessions in or under the folder, pending deletion
effects, and the settings (lines 309-388). Then `git worktree remove` without force; the branch
and the recorded path stay (lines 390-393). `ProviderTurnStartService.ts:458-490` notices a
missing worktree at the next turn, prunes and runs `git worktree add <path> <branch>`.

**Good.** The double read of every condition and the ignored-files rule make an automatic delete
something a user can leave on. Treating the checkout as disposable and the branch as the record is
a sound model for disk.
**Bad.** Merge detection misses squash merges, which most hosted merges are.
**Size.** 518 lines.
**Marley today: has part.** Remove on a worktree row (#589) asks, names uncommitted changes, closes
the workspace, and deletes the branch only when it is merged, squash merges included. Nothing
removes idle worktrees on its own, and a removed worktree is gone rather than parked.

### 2.7 Worktree tools for agents

**What the user sees.** An agent in a thread can move itself into a new worktree mid-conversation
and keep going there.

**How it works.** `apps/server/src/mcp/toolkits/worktree/tools.ts:25-41` defines
`t3_worktree_handoff` (create the branch, optionally from origin, re-point the thread, run the
setup script, end the turn, and queue `continuationPrompt` as the next message in the worktree),
`t3_worktree_status` and `t3_worktree_list`. `apps/server/src/mcp/WorktreeMcpService.ts` is 509
lines.

**Marley today: has part, through Claude Code.** Claude Code makes worktrees of its own under
`.claude/worktrees/`, and Marley lists their terminals under the project (guide "Worktree
agents"). Marley's MCP server has no worktree tool.

### 2.8 Git actions in a thread

**What the user sees.** One control with Commit, Push, Create PR and the chained Commit and push,
Commit, push and create PR. Progress shows per phase. Commit messages, PR titles and bodies are
generated in a chosen style: concise, conventional commits, custom instructions, or **Repository
conventions**. An existing open PR for the branch is opened instead of a second one, and a created
PR is linked to the thread.

**How it works.** `packages/contracts/src/git.ts:18-25` lists the actions. `GitManager.ts:786-835`
builds the style: for repository conventions it reads the last 20 non-merge commit subjects
(`git log -n 20 --no-merges --pretty=format:%s`), the root `AGENTS.md`, and `CLAUDE.md` when the
writer is Claude, each capped at 20 KB and refused if it resolves outside the root (lines 755-784).
`runPrStep` (`GitManager.ts:2037-2140`) refuses a detached or unpushed branch, returns an existing
open PR, resolves the base, reads the range's commit summary, diff summary and patch (capped at
20,000, 20,000 and 60,000 characters), follows a GitHub PR template when one exists
(`apps/server/src/sourceControl/PrTemplateDetection.ts:12-16`), writes the body to a temp file and
calls the host CLI. `apps/server/src/git/linkCreatedPullRequest.ts` links the result.

**Good.** Recent subjects are the cheapest way to make generated messages match a repository.
**Size.** `GitManager.ts` is 2,897 lines; the control is 2,663.
**Marley today: has part.** Zed's git panel commits, pushes and generates commit messages from the
diff, the subject typed so far, and project rules (`crates/git_ui/src/git_panel.rs`,
`generate_commit_message`), with no recent subjects. Zed's Create Pull Request opens the host's
compare page in the browser (`build_create_pull_request_url`); nothing writes the PR text. A
worktree row offers Review, Merge and Remove (#511), with no Push or PR. The Orca survey proposed
commit and PR text through `claude -p` and PR status through `gh` (report 02, items 10 and 12);
neither was built. An agent in the terminal can run `gh pr create` itself.

### 2.9 The Pull Requests page

**What the user sees.** A list of pull requests across the project's repositories, filtered by
involvement (all, reviewing, authored), state, draft, review decision and checks, with Shift held
for quick actions and a drag across rows to close several. A detail panel with Summary, Timeline
and Code tabs: edit title and description, comment, reply, react, resolve threads, request
reviewers, set labels, submit a review (comment, approve, request changes), check the branch out,
mark ready or draft, merge with a chosen method, update the branch from its base, enable or
disable auto-merge, open a revert PR for a merged one, and approve a fork's waiting workflows.

**How it works.** `packages/contracts/src/pullRequest.ts:80-101` lists the actions;
`apps/server/src/pullRequest/PullRequestProvider.ts:339-702` is the contract each host implements,
with optional operations for what a host cannot do; `PullRequestService.ts` (3,410 lines) caches
reads and routes them. GitHub is `GitHubPullRequestCli.ts` (3,022 lines) over `gh`, with batched
GraphQL for background polling and ETag revalidation of checks
(`apps/server/src/pullRequest/gitHubConditionalChecks.ts`).

**Size.** About 20,600 server lines and 16,900 UI lines.
**Marley today: lacks.** The rail's project header shows the branch's PR number and state (#531);
everything else is GitHub's site or `gh` in a terminal.

### 2.10 Six hosts behind one contract

**What the user sees.** Settings, Source Control detects GitHub (`gh` 2.81 or newer), GitLab
(`glab`), Forgejo and Gitea (`fj`, falling back to `tea`), Bitbucket (an access token or an API
token saved on the server, or environment variables) and Azure DevOps (`az` with its DevOps
extension), with **Rescan**. Environments can share GitHub access with each other, read-only or
read and act.

**How it works.** `apps/server/src/sourceControl/` (8,420 lines) discovers hosts and accounts;
`apps/server/src/pullRequest/` holds one provider per host.
`docs/internals/pull-request-file-revisions.md` records a bug class worth knowing for anyone
writing such a layer: a path missing from a host's answer means "no version" below the provider
boundary and "could not say" above it, and mixing the two reports every viewed file as changed.

**Marley today: lacks**, and needs only GitHub (Chad's repositories are there).

### 2.11 Viewed-file marks

**What the user sees.** In a pull request's Code tab a tick collapses a file and the toolbar counts
the ticks. A file pushed to after the tick comes back marked **Changed**. On GitHub the ticks are
GitHub's own viewed marks, in both directions; on other hosts the server keeps them.

**How it works.** `apps/server/src/pullRequest/pullRequestViewedFiles.ts` (423 lines) stores, for
hosts without marks, the file's version at tick time and compares it with the head's version,
cached for 60 seconds.

**Marley today: lacks.** Zed's diffs have no per-file reviewed state. The same idea fits Marley's
Review of a worktree: an agent that answers review notes changes some files, and the files it did
not touch should stay ticked.

### 2.12 Linked pull requests, sync and settlement

**What the user sees.** A thread shows its PR badge (open, draft, merged, closed, and the checks).
The server finds the PR for each unsettled thread's branch even with every app closed; a thread
can link several PRs, from other repositories on the same host too, and agents link theirs with
`link_pull_request`. A merged PR settles the thread (moves it out of the active list) unless the
user wrote to it afterwards; a closed PR settles an idle one.

**How it works.** `apps/server/src/orchestration-v2/ThreadPullRequestService.ts` (408 lines),
`PullRequestSyncReactor.ts` (475 lines) and `ThreadSettlementService.ts:110-130`. A merge made from
a shell sends no notification, so the sync reactor watches runs whose commands match
`/\b(?:gh\s+pr|glab\s+mr)\s+(?:merge|close)\b/` (`PullRequestSyncReactor.ts:40`) and re-reads the
thread's links when such a run ends (lines 434-441).

**Marley today: has part.** #531 reads the main checkout branch's PR with `gh` when the branch or
its commit changes and every two minutes, and shows the number and state on the project header.
Worktree rows show drift and commits ahead (#560) but no PR, and no checks are shown anywhere.

### 2.13 Stacks

**What the user sees.** Each PR's position in its GitHub stack, **Merge stack** (the selected PR
and every unmerged layer below, in one submission that respects branch rules and merge queues) and
**Rebase stack** (remote branches, bottom to top, without touching the local checkout).

**How it works.** `apps/server/src/pullRequest/githubStackActions.ts` (418 lines), with errors that
say how many layers were updated before the stack changed.

**Marley today: lacks.**

### 2.14 Watching a pull request for the agent

**What the user sees.** Ask the agent to "watch" or "babysit" its PR. The thread then counts as
working between wakes. Every two minutes the server checks the PR and, when something new
happened, sends the agent a message such as:

```
Update on pull request #42 (https://…), which T3 Code is watching for you:
- Checks failed on 3f2a1c9:
  - test (linux) https://…
- 2 new comments:
  - reviewer on src/a.ts: "This leaks the handle" https://…
Look into each item and act on it as your task requires. T3 Code keeps watching and wakes you on
the next change, so end your turn when you are done. …
```

The watch ends when the PR merges or closes, after ten comment-only wakes in a row, after eight
failed reads in a row, when the user stops the thread, or when the agent calls
`unwatch_pull_request` on handing the work back.

**How it works.** `apps/server/src/orchestration-v2/pullRequestWatch.ts:43-116` compares the PR
with what the agent was last told: a check reported the moment it fails (so a check that never
finishes cannot hold the news back), "passed" once the base branch's required checks have all
passed (or all checks when none are required), new or edited comments from anyone but the agent's
own account, and a new conflict. Rerun checks leave the failed list, so a second failure is
reported again. `PullRequestWatchReactor.ts:40-55` sets the cadence: a two-minute sweep, one read
per PR for all threads of a project that watch it, a batched fingerprint read on GitHub so only PRs
that moved are read in full, and rereads at 10 or 30 minutes for news a fingerprint cannot see. A
rate limit pauses the watch. Lines 162-201 of `pullRequestWatch.ts` write the wake text and a
timeline notification.

**Good.** The wake conditions are the ones a person would want, and the loop guard (ten
comment-only wakes) stops a bot and an agent from talking to each other forever.
**Size.** 220 + 652 lines.
**Marley today: lacks.** Claude Code in a terminal can poll with `gh pr checks --watch` itself,
spending its own turns and tokens while it waits. Marley already knows when a terminal's Claude
Code is idle (#519) and how to paste a prompt only then (#522).

### 2.15 Remove agent credits when merging

**What the user sees.** A setting, off by default, that strips agent `Co-Authored-By` lines and
"Generated with …" footers from GitHub merge and squash messages, keeping human co-authors.

**How it works.** `apps/server/src/pullRequest/mergeMessage.ts:1-13` lists the agent addresses and
footer pattern; code fences and indented blocks are left alone.

**Marley today: lacks**, and should not take it (section 4).

### 2.16 Automatic pull of the default branch

**What the user sees.** **Automatically pull** keeps the default-branch checkout current, only when
it is on the default branch, has an upstream, no changes and no local commits.

**How it works.** `apps/server/src/serverRuntimeStartup.ts:187-230` at startup, and
`apps/server/src/vcs/VcsStatusBroadcaster.ts` on its remote refresh, which runs on a configurable
interval while a client shows the project and backs off up to 15 minutes on failure (lines 32-34,
531-570).

**Marley today: lacks**, by choice: drift reads never fetch (#560).

### 2.17 New projects, clones, publishing, and threads without a project

**What the user sees.** **New project** from a name makes a git repository in the T3 data folder
with a README, an icon and a first commit, optionally a private GitHub repository. **Add Project**
clones in the background, with the composer usable before the files land. **Publish Repository**
creates the hosted repository, adds `origin` and pushes. A thread with no project works in its own
dated folder under the T3 data folder, named from its first words.

**How it works.** `apps/server/src/project/ManagedProjectFolders.ts:376`,
`apps/server/src/project/ProjectCloneTracker.ts`,
`apps/server/src/sourceControl/SourceControlRepositoryService.ts:403-430`.

**Marley today: has part.** Zed clones (`crates/git_ui/src/clone.rs`). Groups with no folder start
in the home folder (#600, guide "Groups with no folder").

## 3. Bring to Marley

1. **Undo one turn's files from the Turns list.** *Why.* An agent's turn that went wrong is the
   most common reason to reach for git by hand, and Claude Code's `/rewind` misses whatever a shell
   command changed. Marley already keeps each turn as a commit of its end on its start. A row's
   right-click **Undo This Turn…** reverse-applies that commit's patch to the tree with a three-way
   apply, keeping later turns; a conflict aborts and names the files. It refuses, with T3 Code's
   wording adapted, while another agent's terminal works in the same checkout or a folder inside
   or around it (`CheckpointRestoreSafety.ts:13-88`), and it does not touch the agent's
   conversation, so the confirmation says to tell the agent, or to use `/rewind` for both.
   *Seam.* `marley_workbench` (the Turns rows from #509) and Zed's `GitStore`; the checkout check
   reads the rail's terminals and their folders. *Size.* M. *Hard.* Untracked files the turn
   created must go, files the user created since must not; the turn's own tree lists exactly what
   it added, so delete only those that still match it.

2. **A worktree's pull request on its row, and Open Pull Request.** *Why.* Once worktree branches
   go to GitHub, Chad needs their state where the worktree is, and a PR body written from the
   branch's commits and diff. Extend #531 to worktree rows, and add the checks rollup (passing,
   failing, pending) with a popover of failed checks and links. Add **Push and Open Pull
   Request…** to the row's menu: refuse a detached or unpushed branch the way `runPrStep` does,
   open the existing PR when there is one, otherwise generate the title and body from the range
   (`GitManager.ts:2037-2140`), follow `.github/pull_request_template.md`, show the text for
   editing, then `gh pr create --body-file`. *Seam.* `marley_workbench` (rail rows, #531's `gh`
   reads), Zed's language model for the text. Orca report 02 items 10 and 12. *Size.* M. *Hard.*
   `gh` rate limits with many worktrees; batch the reads into one GraphQL query per repository,
   as T3 Code did when it cut its background calls.

3. **Watch a pull request and wake the agent.** *Why.* An agent that pushes and stops leaves Chad
   to notice red CI and paste the log back. Marley can watch instead and spend no tokens while
   nothing happens. A `pr_watch` tool on Marley's MCP server, called by the agent for its own
   terminal (#520 says which), plus a Watch toggle on the PR chip from item 2. Every two minutes
   while any watch is live, compare checks, comments and mergeability with what was last told
   (`pullRequestWatch.ts:43-116` is the rule set to port), and when there is news, paste T3 Code's
   wake text into the agent's terminal only while it is idle, as #522 does; otherwise hold it
   until the plugin's Stop event. End on merge or close, after ten comment-only wakes, or after
   eight failed reads. The rail row says `watching #42`. A Codex on its own App Server (#650)
   could take the wake as a turn through that server. *Seam.* `marley_mcp`, `marley_workbench`.
   *Size.* M after item 2. *Hard.* The agent's own comments must not wake it: compare authors with
   `gh api user`.

4. **Start a worktree from the fetched base.** *Why.* A worktree cut from a stale local `main`
   starts the agent behind and makes the drift chip lie until the next fetch. A **Start from
   origin** box in the New Agent in Worktree prompt, remembered in git config as
   `marley.worktreeSetup` is: fetch the base from `origin`, start at the remote-tracking commit
   when it exists, else the local base (`ThreadLaunchService.ts:334-372`). Record that commit as
   `branch.<b>.base` as now. *Seam.* `marley_workbench` worktree creation, which calls Zed's
   worktree service. *Size.* S. *Hard.* A fetch can prompt for credentials; run it with
   `GIT_TERMINAL_PROMPT=0` as #537 does for agents, and fall back to the local base on failure.

5. **Name the branch from the first prompt.** *Why.* Rows that read `agent/fix-login-redirect`
   instead of a made-up name tell Chad which agent is which. Create as now, then ask Zed's
   configured model (or System One where it is on) for a fragment from the first prompt and rename
   the branch in the background with `git branch -m`, keeping the `agent/` prefix; leave the name
   if the call fails, and never rename a branch with an upstream. T3 Code's prompt and its three
   modes are `TextGenerationPrompts.ts:188-219`. *Seam.* `marley_workbench`; the rename must also
   move `branch.<b>.base` and `branch.<b>.marleySlot`. Orca report 02 item 11. *Size.* S.
   *Hard.* The agent may already have printed the old name; the rename happens within seconds,
   before its first commit.

6. **Viewed marks in Review that a later turn clears.** *Why.* Reviewing a worktree in rounds (read,
   send notes, the agent fixes three files) means rereading every file each round. A tick per file
   in Review's diff that collapses it and stores the file's blob id at tick time; a file whose blob
   changed comes back marked Changed, as T3 Code does against a host
   (`pullRequestViewedFiles.ts`). *Seam.* `git_ui`'s branch diff (a Zed touch, additive) or a
   Marley wrapper view; marks in Marley's database keyed by repository, branch and path. *Size.* M.
   *Hard.* Collapsing excerpts per file in Zed's multibuffer.

7. **Review notes that carry their hunk.** *Why.* A note on line 40 is ambiguous once the agent's
   previous turn moved line 40. Add the commented lines of the diff (bounded, as
   `composerContext.ts:191-205` bounds them) and the view's name ("Changes since main", "Turn 3")
   to each note in #522's prompt. *Seam.* The #522 handler in `marley_workbench`. *Size.* S.
   *Hard.* None worth naming.

8. **Refresh the PR after `gh pr merge` in a block.** *Why.* #531 learns of a merge up to two
   minutes late. Marley's blocks know each command, so a finished block matching
   `\b(?:gh\s+pr|glab\s+mr)\s+(?:merge|close)\b` (`PullRequestSyncReactor.ts:40`) in a project
   re-reads that project's PR at once, and the same for an agent's tool call seen through the
   plugin's events. *Seam.* `marley_workbench`. *Size.* S. *Hard.* None.

9. **Recent subjects in Zed's commit message generation.** *Why.* Generated messages should match
   the repository's own style. Add the last 20 non-merge subjects to the prompt beside the project
   rules Zed already includes (`GitManager.ts:769-784`). *Seam.* `crates/git_ui/src/git_panel.rs`,
   a small additive Zed touch, or a Marley setting that turns it on. *Size.* S. *Hard.* None.

10. **Offer to park idle worktrees.** *Why.* Worktrees with their own dependencies cost disk long
    after their agent finished. A rail menu entry, **Clean Up Idle Worktrees…**, lists the
    worktrees with no open terminal, no changes, no ignored files besides `node_modules` and no
    activity for N days, and removes their folders while keeping the branch, its base and its slot.
    The worktree row stays, marked parked, and a click recreates the checkout with
    `git worktree add <path> <branch>` (`ProviderTurnStartService.ts:458-490`). Apply T3 Code's
    rule of reading every condition again right before removing (`storageCleanup.ts:309-388`).
    *Seam.* `marley_workbench`, beside Remove (#589). *Size.* M. *Hard.* A parked row must not
    count for drift or ports.

## 4. Skip

- **The Pull Requests page as a review client** (about 37,000 lines across server and UI): GitHub's
  site and `gh` already do it, and Chad's repositories have one author.
- **GitLab, Forgejo, Gitea, Bitbucket, Azure DevOps and GitHub sharing between environments**:
  Chad's repositories are on GitHub, and Marley runs one environment.
- **Stacks**: a team workflow.
- **Remove agent credits when merging**: Chad's own rules require the `Co-Authored-By` line, and
  Marley's merges are local merge commits (#511).
- **Automatic pull of the default branch**: Marley's drift never fetches by design, the main
  checkout is often on a feature branch, and item 4 covers the case that matters.
- **Thread settlement on merge**: terminal agents have no thread lifecycle in Marley; Remove after
  a merge does the job, and #589 already proves the merge, squash merges included, which T3 Code's
  cleanup cannot.
- **Rewinding the conversation**: Claude Code's `/rewind` owns its conversation, and Zed's Agent
  Panel has Restore Checkpoint and Edit for its threads.
- **`t3_worktree_handoff`**: Claude Code's own worktrees cover an agent that wants to move.
- **Setup progress stages**: the setup runs as blocks in the agent's terminal, which show the same
  thing. The `async` choice (start the agent while setup runs) is not worth a setting.
- **A submodule policy**: add it when a repository with submodules shows up.
- **New project from a name and Publish Repository**: rare actions that `git init` and
  `gh repo create` do in a terminal.

## 5. Open questions

1. **Pull requests for agent work.** Do worktree branches go to GitHub as pull requests, or are they
   merged locally (#511) or by the Rustal workflow (`marley.merge workflow`)? Items 2, 3 and 8 pay
   off only with pull requests. *Default: build item 1 and items 4 to 7 first; items 2, 3 and 8
   wait for the answer.*
2. **Undo a turn: one turn or back to it.** Reverse-apply only that turn and keep the later ones,
   or restore the tree to the turn's start and drop everything after, as T3 Code does? *Default:
   reverse-apply one turn, three-way, refuse on conflict.*
3. **Who names branches.** Zed's configured model, System One, or a headless `claude -p` on Chad's
   Claude Code login? *Default: Zed's commit-message model; no rename when none is configured.*
4. **Watch wakes while the agent works.** Hold the wake until the turn ends, or show it in the inbox
   for Chad to send? *Default: hold until the Stop event, then paste; the inbox shows it meanwhile.*
5. **Parking worktrees.** Offered from a menu, or automatic after N days as T3 Code allows? *Default:
   offered only, never automatic.*
6. **Start from origin.** On by default for new worktrees? *Default: off, remembered per
   repository once ticked.*
