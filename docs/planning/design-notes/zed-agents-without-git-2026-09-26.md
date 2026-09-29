# Zed's agents without git, 2026-09-26

Chad said on 2026-09-26: "Zed has a new system where it removes git and allows for agents to i
believe work locally without it. Lets see if we can have this somehow work on this system." One
agent read Zed Industries' published material (the zed.dev blog, Zed's docs and release notes, the
zed-industries/zed issues and pull requests, and Delta's docs, roadmap and release notes at
delta.dev) and the fork's own crates at upstream `78648aaf7d`. Nothing was built, tested or
fetched. The system is Delta, a separate Zed Industries app for coding with agents, built on a new
version-control layer called DeltaDB and in public beta since 2026-09-16. Headlines called it a
git replacement, but Delta's docs say a project must be a git checkout and commits stay in git;
what Delta replaces are pull requests, as the place where review happens, and git worktrees. Its
agents run on the local machine, by default each in a checkout of its own, and its data-storage
page lists the repository's contents and every edit among what it keeps on Zed's servers, behind a
Zed account. None of it is in the Zed editor's code or in our fork. The editor runs agents in a
folder with no git and can reject their edits there, but its checkpoints and its worktree
isolation need git, and the community pull request that would give it checkpoints without git has
had no review since it opened on 2026-07-30. The note recommends Marley's own local form of the
idea first: a snapshot of the folder before each Claude Code prompt, in a shadow repository outside
the project, restorable from the terminal's rail row (M). Delta itself can run beside Marley on
this box once Chad decides which code may go to Zed's servers (S).

## What the feature is

Zed Industries' posts on it, in order:

| Date | Post | What it says |
|---|---|---|
| 2026-06-11 | "Software Is Made Between Commits", Nathan Sobo (zed.dev/blog/introducing-deltadb) | DeltaDB records the work between commits as a stream of fine-grained deltas, each with a stable identity; git and CI keep running checks and connecting to everyone else. A waitlist. |
| 2026-08-12 | "Introducing Delta", Nathan Sobo (zed.dev/blog/introducing-delta) | Delta, "a multiplayer environment for coding with agents and reviewing what they build", a separate app. "DeltaDB works with the git repository you already have." "We'll keep developing Zed, and DeltaDB will come to it, but Delta is where it begins." A private beta. |
| 2026-09-01 | "Xanadu Was Waiting for Agents", Nathan Sobo (zed.dev/blog/agentic-xanadu) | "Every thread is also a git branch, so teammates who never open Delta see a normal repo". |
| 2026-09-16 | "Replace PRs with Delta – Now in Public Beta", Nathan Sobo (zed.dev/blog/delta-public-beta) | The public beta, free while it lasts. The Delta team turned off pull requests on Delta's own repository; Zed's repository stays on GitHub. |

The press said it more strongly. AlphaSignal's headline on 2026-08-12 read "Zed Launches Delta to
Replace Git Where AI Agents Write Code", though its own text says DeltaDB works with the git
repository you already have. The New Stack's read "Zed launches Delta because agents made pull
requests obsolete". Chad's "removes git" matches the headlines. "Agents ... work locally" matches
Delta's isolated checkouts and its `local` git remote, which brings an agent's branch into the main
clone without a trip through GitHub.

Checked and set aside, since none of them lets agents work without git: Parallel Agents
(2026-04-22, isolation through git worktrees), Terminal Threads (2026-05-20, CLI agents in
terminal-backed threads), Sandboxing (2026-08-05, Cameron Mcloughlin: the Zed agent's terminal
commands cannot write `.git`), Container Use (2025-07-30, containers plus git worktrees), Zed's
stable notes from 1.17.2 to 1.21.0 and preview notes to 1.22.0 (2026-09-23), the upstream
commits of 2026-09-23 to 2026-09-26, and the pull requests merged from 2026-09-18 to 2026-09-26
that mention agents or worktrees. If Chad meant something else, those are the places already
searched.

## How Delta works

From delta.dev/docs. Delta is closed source, and none of it is in zed-industries/zed.

- **Projects:** "The folder you open as a project must be the top-level folder of a Git checkout."
  Delta imports the tracked files, and the untracked files that are not ignored, into DeltaDB. The
  repository stays a normal git repository (concepts/core-concepts, concepts/delta-and-git).
- **Deltas:** every file edit, file-tree change, message and comment is recorded as it happens,
  with nothing staged or committed. Commits stay in git and happen only when someone commits. A
  thread can be reverted to an earlier point, files included, with no commit (Edit > Revert
  Conversation to Cursor, agents/threads).
- **Worktrees:** each thread gets "Delta worktrees", which are not git worktrees: "Delta does not
  use git worktrees, despite the similar name, to avoid limitations around how many threads can
  work on one branch." Each participant gets a checkout on disk, in a `.delta` folder of the local
  clone when one is connected, otherwise in a folder Delta manages. Archiving a thread removes its
  checkout and reopening makes it again. A thread may use the existing local checkout instead, and
  is then not isolated (concepts/worktrees).
- **Getting work back:** the agent commits in its checkout, and `git push local <branch>` puts the
  branch in the main clone; or the agent pushes to origin and opens a pull request
  (agents/review-and-sync).
- **Setup:** `.agents/prepare`, an executable run in each new checkout before the agent starts, and
  `.agents/linked`, paths symlinked to one shared copy (configuration/project-setup).
- **Agents:** Delta's own, with Worker, Scout and Reviewer subagents that take
  `worktree = "shared"` or `"isolated"`. Models come through API keys (Anthropic among them),
  ChatGPT, Copilot and Grok subscriptions, or hosted models on a paid Zed plan; signing in with a
  Claude subscription is "coming soon" (agents/models-and-providers). Its terminals start in the
  thread's checkout and run on the local machine with the user's tools and credentials
  (agents/terminals). "Delta also connects to
  third-party agent harnesses, starting with Claude Code" (Introducing Delta), and the roadmap has
  a Claude Code plugin in progress, to "Track Claude Code conversations and code changes in
  DeltaDB, then review them in Delta or on delta.dev" (delta.dev/roadmap).
- **What it needs:** a Zed account through GitHub sign-in and the Delta Early Access Agreement; a
  workspace does not open without network access (account/authenticate, troubleshooting). The
  repository's contents (git objects and file contents), the thread's deltas and metadata are kept
  on Zed's servers in Cloudflare R2, Durable Objects, KV and D1, and model requests go through Zed
  Cloud. Ignored files stay local, and recognized secrets are redacted before anything leaves
  (privacy-and-security/data-storage and /security).
- **Safety:** "Delta does not have an agent permission system." It does not sandbox agents, and it
  runs `.agents/prepare`, `.envrc` files and agent instruction files with no trust check
  (privacy-and-security/agentic-safety).
- **Linux:** a glibc build for x86_64 and aarch64, a tarball whose `install.sh` puts Delta at
  `~/.local/delta.app` (installation). Releases 0.1.1 to 0.17.0 came between 2026-08-24 and
  2026-09-23, with Linux fixes such as settings opening as floating windows on tiling window
  managers (0.12.0), and an `Open in Zed` action that "now opens the active worktree file at the
  current cursor position" (whats-in-the-latest).

## What the Zed editor does without git, in our fork

The fork's `main` is upstream `78648aaf7d` (2026-09-18, version 1.22.0 in development). Upstream
has merged nothing on this since: the 1.21.0 and 1.22.0-pre notes (both 2026-09-23) and the pull
requests merged to 2026-09-26 carry none of it, so an upstream merge would bring nothing today.

DeltaDB's only trace in the tree is the `path` crate, whose first line reads "Relative path types
for deltadb" (split out of `util::rel_path` by upstream #61029, 2026-07-15), and a test comment,
"This whole crate is going away with DeltaDB soon"
(`crates/collab/tests/integration/random_project_collaboration_tests.rs:1276`). There is no
DeltaDB client and no sync code.

| Agent feature | In a git repository | In a folder with no git |
|---|---|---|
| Restore Checkpoint on Agent Panel threads (Zed's agent, and ACP agents that support rewind) | A hidden commit before each prompt | Never offered |
| Reject an agent's edits; edit an earlier message and resend (where the agent supports rewind) | Works | Works for edits made through the agent's edit tool or ACP `fs/write_text_file`; what terminal commands changed stays |
| Isolation for parallel threads | A linked git worktree per thread | None; the folder is shared as it is |
| Sandbox for the Zed agent's `terminal` and `fetch` tools | `.git` read-only | Works, but an agent may create a `.git` |
| Terminal Threads (CLI agents in the Threads Sidebar) | No checkpoints | No checkpoints |

How the code gets there:

- **Checkpoints.** Before each user message `AcpThread` asks `GitStore::checkpoint`
  (`crates/acp_thread/src/acp_thread.rs:4002`), and after edits compares a new checkpoint with it to
  decide whether the button shows (`:4424` to `:4540`; the button is at
  `crates/agent_ui/src/conversation_view/thread_view.rs:6184`). `GitStore::checkpoint` loops over
  the project's repositories (`crates/project/src/git_store.rs:2133`). In a folder with none it
  returns an empty map, two empty maps compare equal, and the button never appears. For each
  repository, `RealGitRepository::checkpoint` (`crates/git/src/repository.rs:3063`) works in a
  temporary index: `add --update`, the untracked files that `ls-files --others --exclude-standard`
  lists minus the 76 patterns of Zed's `checkpoint.gitignore` and any file of 2 MB or more
  (`:3774`), then `write-tree` and `commit-tree` on HEAD. The objects land in the project's own
  `.git`. Restore is `git restore --source <sha> --worktree .` (`:3091`), which leaves files added
  since the checkpoint in place; a TODO there says the `git clean` step is off because it would
  delete the large and binary files that checkpoints no longer track.
- **Reject and rewind.** The action log keeps, for each buffer an agent edits, its text before the
  agent's edits (`diff_base` in `crates/action_log/src/action_log.rs`). Reject writes that text
  back, and `reject_all_edits` (`:924`) does it for every buffer. Editing an earlier message calls
  `AcpThread::rewind` (`thread_view.rs:2059`, `acp_thread.rs:4380`), which truncates the
  conversation and rejects the edits; it fails with "not supported" for an agent that cannot
  truncate, and Restore Checkpoint runs it first. An external ACP agent's writes arrive through
  `AcpThread::write_text_file` (`acp_thread.rs:4654`) as buffer edits the action log sees. None of
  this needs git, and none of it sees a file that a terminal command changed.
- **Parallel agents.** Zed's worktree service (`crates/git_ui_core/src/worktree_service.rs`) makes
  linked git worktrees under `git.worktree_directory` (`../worktrees` by default). Archiving a
  thread saves its worktree's state under `refs/archived-worktrees/`
  (`crates/agent_ui/src/thread_worktree_archive.rs:78`) and then removes the worktree, when no
  other active thread uses it. Zed's docs say "Non-Git folders in the same project are included in
  the new workspace as-is" (`docs/src/git.md:229`).
- **Sandbox.** On Linux, `build_bwrap_args_with_sandbox_paths`
  (`crates/sandbox/src/linux_bubblewrap.rs:249` to `:307`) binds the project folders read-write and
  then binds each `.git` read-only over them. It covers only the Zed agent's `terminal` and `fetch`
  tools, not external agents, Terminal Threads or ordinary terminals (`docs/src/ai/sandboxing.md`).
  Upstream re-enabled it on 2026-07-29 (#61711). The same page warns that "an agent given
  access to a non-Git-repo directory `/foo` could create `/foo/.git`".

The open upstream work that would change the last column: issue #61951, "Agent checkpoints are
unavailable in projects that are not git repositories" (2026-07-30, severity S2), and its pull
request #61952 by ArneshBanerjee, a community contributor, opened 2026-07-30, last touched
2026-08-09, with no review. It keeps a git directory for each folder with no repository under
Zed's data directory (`checkpoints/<hash>`) and runs the existing checkpoint, restore and compare
code against it with `GIT_DIR` and `GIT_WORK_TREE`, so the project gets no git metadata. It
changes `crates/fs/src/fs.rs`, `crates/git/src/repository.rs`, `crates/paths/src/paths.rs` and
`crates/project/src/git_store.rs`, for local projects only; in Marley the directory would be
`~/.local/share/marley/checkpoints/`. Next to it, all open: #62301 adds `agent.enable_checkpoints`
to turn checkpoints off, #64000 bounds the checkpoint fan-out across repositories
(`ZED_GIT_CHECKPOINT_CONCURRENCY`), and issue #62283 reports prompts that hang forever in
repositories with a large mass of untracked files.

## This box

- 15 of the 50 folders under `/srv/stacks` have no repository at their root. Three hold nested
  repositories that Zed finds (`ignibyte`, `orca-refs`, `rustal_themes`), and two are reference
  trees that stay closed. The other ten have none at any depth: the four `asset-gallery` folders,
  `calibre-web`, `daisy_ui_admin`, `dashboard`, `rustal-harness` (172G, 37G of it in `target/`),
  `self_development` and `stories`. Eight of the ten have no `.gitignore` either.
- `/srv/stacks` is btrfs (subvolume `@stacks`, on the same device as `/mnt/fast`), so
  `cp --reflink` copies there share data blocks. Home is btrfs on another device.
- `bwrap` 0.12.0, not setuid, with unprivileged user namespaces on: Zed's Linux sandbox works here,
  and bubblewrap's `--overlay` options (new in 0.11.0, 2024-10-30) are available.
- git 2.55.0.
- Upstream Zed is installed on its own at `~/.local/zed.app` and owns both `zed` on the PATH and
  the `zed://` handler (`dev.zed.Zed.desktop`). Delta is not installed.

## How Marley could use it

Marley's layout puts projects in the rail, terminals in the center, CLI agents such as Claude Code
in those terminals, and Zed's Agent Panel threads under their project rows. Delta brings two ideas:
agents that work in their own copy of the code, and every agent step recorded and revertible
between commits. For git repositories the first is planned already. #510 gives each terminal agent
a linked git worktree on its own `agent/<name>` branch, and #511 reviews and merges it. Delta
avoids git worktrees because git checks a branch out in only one worktree at a time; #510 never
hits that limit, since every agent gets a new branch. What is left is the second idea everywhere,
and both ideas in the ten folders with no git.

**Works today, with no Marley code:**
- Agent Panel threads in a folder with no git: Reject, and rewind by editing an earlier message,
  for the edits the agent made through its tools.
- Claude Code in a Marley terminal: its own `/rewind`, in any folder. It covers only its
  file-editing tools ("Checkpointing does not track files modified by Bash commands") and not
  background subagents' edits, and it keeps the 100 most recent checkpoints of a session
  (code.claude.com/docs/en/checkpointing).
- Delta as its own app beside Marley, on a repository whose contents may go to Zed's servers.

**Needs an upstream merge:** nothing that exists yet. The fork already has every agent feature Zed
ships. DeltaDB in the editor has no date, and #61952 is unmerged; if it merges, a routine upstream
merge brings it.

**Needs Marley code:** snapshots for terminal agents (recommendation 1), isolation in folders with
no git (4), and carrying #61952 early (3, as Zed touches).

**Risks:**
- Code leaves the box with Delta. `omarchy-ops` and the Rusty skill store name hosts and addresses
  and never go into a public repository; Delta would copy whatever it opens to Zed's storage, under
  Zed's retention.
- Delta runs `.agents/prepare`, `.envrc` and instruction files from a repository with no trust check
  and no sandbox, and it is a beta that asks for an update when its protocol changes.
- Snapshots in a folder with no `.gitignore` can walk build output and stall, the failure #62283
  reports, and a `UserPromptSubmit` hook gets 30 seconds by default (code.claude.com/docs/en/hooks).
- Restore has to delete what the agent added without touching what Chad changed since the
  snapshot, the case Zed's own restore sidesteps by deleting nothing.
- An agent in an overlay of a folder Chad keeps editing is unsafe: "If the underlying filesystem is
  changed, the behavior of the overlay is undefined" (docs.kernel.org/filesystems/overlayfs.html).

## Recommendations, ranked

1. **Snapshots before each Claude Code prompt, in a shadow repository (M).** Marley's Claude Code
   plugin already runs `event.py` on `UserPromptSubmit`
   (`crates/marley_workbench/claude_plugin/marley/hooks/hooks.json`), only in a terminal that sets
   `TERM_PROGRAM=zed` as Marley's do (`crates/terminal/src/terminal.rs:721`), and the prompt waits
   until the hook exits. `event.py` would snapshot `CLAUDE_PROJECT_DIR` before it answers, and the
   event it sends would carry the snapshot's id. The snapshot follows Zed's checkpoint recipe
   against a git directory outside the project: `GIT_DIR` one per folder under
   `~/.local/share/marley/checkpoints/`, named by a hash of the folder's path, and `GIT_WORK_TREE`
   the folder itself; a temporary index; the folder's own ignore files plus Zed's
   `checkpoint.gitignore` and the 2 MB cap; `write-tree`, `commit-tree`, and a ref for each session
   and prompt. `marley_agent::claude_events` keeps the id on the seat, and the terminal's
   rail row gets "Changes since this prompt" (Zed's `MultiDiffView::open` over pairs of old and new
   files, `crates/git_ui/src/multi_diff_view.rs:148`) and "Restore files to before this prompt". It
   works with or without git, never writes the project's `.git`, covers what Bash changed, which
   `/rewind` misses, and sends nothing off the box. It is #61952's layout, so if upstream merges
   that, Marley can share the directory. Gemini CLI does the same for its `/restore`, with a shadow
   repository under `~/.gemini/history/<project_hash>`.
   *Hard parts:* restore only while the agent is idle; snapshot the present first so the restore
   can be undone; delete only files that a later snapshot holds and the chosen one lacks, and never
   touch ignored or oversized files it never recorded. With no `.gitignore`, leave out `target/`,
   `node_modules/`, `.venv/`, `dist/` and `build/`, and give up with a note on the row when the
   walk would outrun the hook's 30 seconds. No git alternates into the project's object store, so
   a `git gc` there can never break a snapshot. Prune refs after 30 days, as Claude Code does.
   Codex, Gemini and OpenCode have no Marley plugin and would need another cue, such as the block
   terminal's command start, in a later slice.
2. **Try Delta beside Marley on one repository (S, no Marley code),** once Chad has answered the
   second question. Install the x86_64 tarball, sign in, open a repository that may leave the box,
   run a thread in an isolated checkout and bring its branch home with `git push local <branch>`.
   Note how it behaves on Hyprland, where its `.delta` checkouts land, and what `Open in Zed` opens
   (upstream Zed, on this box, untested). Then watch the roadmap's Claude Code plugin: once it
   ships, Claude Code sessions in Marley's terminals could sync into Delta threads with no Marley
   code, and a rail row could gain "Open in Delta".
3. **Checkpoints for Agent Panel threads in folders with no git: wait for #61952 (S once merged).**
   Carrying it now is S to M and four Zed touches (`crates/fs`, `crates/git`, `crates/paths`,
   `crates/project`), each a row in `docs/marley/zed-touchpoints.md`. It pays only if Chad runs
   Agent Panel threads in the ten folders; recommendation 1 covers the terminal agents Marley is
   built around.
4. **Isolation in folders with no git (S or M, by Chad's answer to question 5).** The small form is
   a "Start a git repository here" entry in a no-git project's + menu, after which #510 applies.
   The larger form, for folders that must stay without git, is "New Agent in a Copy": a reflink
   copy of the folder's files on the same btrfs filesystem, ignored paths left out and setup run in
   it as Delta's `.agents/prepare` does, the agent started there, Review through `MultiDiffView`
   between the folder and the copy, and Apply copying the changed files back once it has checked
   that the originals did not change since the copy.
5. **Leave out:** rebuilding DeltaDB or a sync service in Marley (L and more, for a closed design
   that Delta already ships), and running agents inside a bubblewrap overlay of the project (the
   kernel leaves the overlay undefined when Chad edits the real folder, and Claude Code would need
   its config, credentials and network inside the sandbox).

## Open questions for Chad

**Chad's answer on Delta, 2026-09-28:** "since this isnt open source and requires zed remote server
than im not sure we do this at all. maybe when its mature". Marley does not adopt Delta or send any
repository to Zed's servers for now; the note waits until Delta matures. The questions below stay
as they were written.

1. Is Delta what you saw, the "Replace PRs with Delta" launch of 2026-09-16? If it was something
   else, a link would settle it. Default: Delta.
2. May any repository's contents go to Zed's servers? Delta uploads git objects, file contents and
   every edit. Default: none of the private ones (`omarchy-ops`, the Rusty store); a trial only on a
   repository that is public already.
3. Which part of "work locally without git" matters first: undoing an agent's work in any folder
   (recommendation 1), parallel agents in folders with no git (4), or the history between commits
   that DeltaDB records? Default: undo first.
4. Should recommendation 1 cover git repositories too, with one shadow repository per folder that
   never writes the project's `.git`, rather than only folders without git? Default: yes, one
   mechanism for both.
5. Of the ten folders with no git, do any need parallel agents, and may those become git
   repositories so #510 covers them? Default: `git init` where parallel agents are wanted.
6. Upstream Zed owns `zed` and `zed://` on this box, so Delta's `Open in Zed` would open it rather
   than Marley. Is upstream Zed still in use here? Default: leave it as it is.

## Sources

Zed Industries, published (dates are publication dates):

- https://zed.dev/blog/introducing-deltadb (2026-06-11), https://zed.dev/blog/introducing-delta
  (2026-08-12), https://zed.dev/blog/agentic-xanadu (2026-09-01),
  https://zed.dev/blog/delta-public-beta (2026-09-16), https://zed.dev/blog/sandboxing
  (2026-08-05), https://zed.dev/blog/parallel-agents (2026-04-22),
  https://zed.dev/blog/terminal-threads (2026-05-20),
  https://zed.dev/blog/container-use-background-agents (2025-07-30), and the index at
  https://zed.dev/blog
- https://delta.dev, https://delta.dev/roadmap, https://delta.dev/download, and under
  https://delta.dev/docs/: installation, concepts/core-concepts, concepts/delta-and-git,
  concepts/worktrees, agents/threads, agents/terminals, agents/review-and-sync,
  agents/models-and-providers, configuration/project-setup, account/authenticate,
  account/plans-and-pricing, privacy-and-security/data-storage, privacy-and-security/security,
  privacy-and-security/agentic-safety, troubleshooting, whats-in-the-latest (0.1.1 to 0.17.0)
- https://zed.dev/releases/stable, https://zed.dev/releases/preview, https://zed.dev/roadmap,
  https://github.com/zed-industries/zed/releases/tag/v1.21.0 and
  https://github.com/zed-industries/zed/releases/tag/v1.22.0-pre
- https://github.com/zed-industries/zed/issues/61951, /pull/61952 and its files, /pull/62301,
  /pull/64000, /issues/62283; the commits on `main` from 2026-09-23 to 2026-09-26; merged pull
  requests from 2026-09-18 to 2026-09-26 searched for "agent" and "worktree"; pull requests and
  issues searched for "checkpoint"

Press: https://alphasignal.ai/news/zed-launches-delta-to-replace-git-where-ai-agents-write-code
(2026-08-12) and https://thenewstack.io/zed-delta-github-alternative/ (the headline only; the text
did not load).

Other published material: https://code.claude.com/docs/en/checkpointing,
https://code.claude.com/docs/en/hooks, https://docs.kernel.org/filesystems/overlayfs.html,
https://github.com/containers/bubblewrap/releases/tag/v0.11.0, and Gemini CLI's
https://github.com/google-gemini/gemini-cli/blob/main/docs/cli/checkpointing.md.

Code read, in the fork (upstream `78648aaf7d` plus the Marley branch at `ca70b6488d`):
`crates/acp_thread/src/acp_thread.rs`, `crates/agent_ui/src/conversation_view/thread_view.rs`,
`crates/agent_ui/src/thread_worktree_archive.rs`, `crates/action_log/src/action_log.rs`,
`crates/agent_servers/src/acp.rs`, `crates/project/src/git_store.rs`,
`crates/git/src/repository.rs`, `crates/git/src/checkpoint.gitignore`,
`crates/sandbox/src/linux_bubblewrap.rs`, `crates/path/src/path.rs`, `crates/paths/src/paths.rs`,
`crates/git_ui/src/multi_diff_view.rs`, `crates/terminal/src/terminal.rs`,
`crates/marley_workbench/claude_plugin/marley/hooks/{hooks.json,event.py}`,
`crates/marley_agent/src/claude_events.rs`, the `agent-panel`, `parallel-agents`,
`terminal-threads` and `sandboxing` pages in `docs/src/ai/`, `docs/src/git.md`,
`assets/settings/default.json`, and the #510
and #511 specs and notes in `docs/planning/pipeline/queued/`. Box checks, all read-only:
`findmnt`, `bwrap --version`, `git --version`, `git rev-parse` over `/srv/stacks/*`, `du` on the
ten folders, and `xdg-mime query default x-scheme-handler/zed`.
