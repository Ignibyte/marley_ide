# Orca second pass, 2026-09-25

Chad asked for another look at Orca "just to make sure we didnt miss anything else cool. dont
stretch though." One agent read Orca's source (`stablyai/orca` at `1c2cf120e3`, 2026-09-25):
`src/main`, `src/renderer`, `src/shared` and `src/cli`, the 57 docs pages, the 52 reference notes,
the skills and skill guides, and the commit log (the feature commits of all 11,752, and every
commit since 2026-09-18). Each candidate was searched for in the seven reports of
`docs/orca_architecture/` under several terms before it was kept, and each one kept was read in
Orca's current source. The first survey missed little; Orca's week since it is structured chat,
the phone's web bundle, two more agent CLIs and fixes. Four findings help when several agents run
at once, and each is about a day of work: Orca asks before a quit or a tab close ends a live agent,
which Marley never does (ranked first); it lists which files a worktree branch would conflict on,
from a local `git merge-tree`; it routes programs that open a browser into its own through
`BROWSER`; and it treats a printed PR URL as the cue to look up that branch's PR. Five smaller
details fit work already queued. Orca is MIT, so code taken from it keeps its notice; in practice
Marley reimplements in Rust and cites the Orca path.

## 1. Quitting names the agents it would kill

**What it is:** Orca asks before it closes a terminal tab whose shell still has a live child
process, and before it closes the window or quits while a remote host still runs work. The first
survey's changelog list in report 07 carries it as two lines (Orca PRs #21569 and #18593), and no
report weighed it for Marley.

**Where it came from:** `probePtyRunningWork` asks each PTY's owner whether a child process is
alive and answers `live`, `unverifiable` or `exited`; a probe that times out counts as
`unverifiable`, never as idle (`src/renderer/src/components/terminal/pty-running-work-probe.ts`,
105 lines). The tab guard asks only on a close the user made, never on a bulk, CLI or lifecycle
close, and waits at most 4 s for the probe (`running-terminal-close-guard.ts`, 143 lines); a
setting turns it off (`skipCloseTerminalWithRunningProcessConfirm`). The window guard waits at
most 1.5 s (`window-close-running-work.ts`, 70 lines). Orca's quit asks only about remote work,
because its local shells and agents survive a quit in its terminal daemon (report 05, §2.2).

**Marley today:** Marley's terminals die with Marley, and nothing warns. Zed's `confirm_quit` is
off by default and asks only "Are you sure you want to quit?" (`crates/zed/src/zed.rs`). A
terminal counts as dirty only while a Zed task runs in it or after a bell
(`TerminalView::is_dirty` in `crates/terminal_view/src/terminal_view.rs`), and on quit Zed
serializes dirty items instead of asking. So a restart after `just install`, or closing the
window, ends every terminal agent, mid-turn or not, without a word.

**What Marley would do:** `marley_workbench` already builds a `TerminalAgent { kind, status }`
for each terminal from its foreground command (`crates/marley_workbench/src/rail.rs`). A Zed
touchpoint in quit and in the window-close path asks Marley for the terminals with a live agent
and, when there are any, asks "Quit Marley? 2 agents are running: marley_ide, Claude Code
(working); rustal-harness, Codex (waiting)". It stays silent when no agent runs. Closing one
terminal tab with a live agent asks the same question for that tab.

**Hard parts:** The rail's "working" is the two-second quiet heuristic until #519's hook events
land, so the prompt lists every live agent with its state, not only the working ones. A kill from
outside (`hyprctl kill`, a crash) cannot ask; resuming the session (#540) covers that case. The
question must never fire for a plain shell, or it becomes a click dismissed unread.

**Size and ticket:** S. No ticket; the pass recommends one of its own now. The Warp second pass
reaches the same gap from Warp's docs (`warp-second-pass-2026-09-25.md`, finding 2) and adds a
60-second undo and the rule that a logout or shutdown is never held up, so one ticket can take
both.

## 2. Which files a branch would conflict on, before anyone merges

**What it is:** When GitHub marks a worktree's pull request as conflicting, Orca's Checks panel
lists the conflicting files and how many commits the branch is behind its base. When the merge is
clean locally, it says GitHub's verdict is stale and offers the two commands that make GitHub
recompute it.

**Where it came from:** `src/main/github/conflict-summary.ts` (313 lines), with a cache in
`conflict-summary-cache.ts` and the panel in
`src/renderer/src/components/right-sidebar/checks-panel/conflict-summary.tsx`. It makes no GitHub
API call. A throttled `git fetch origin <base>`, capped at 10 s, gets the live base tip rather than
the PR's pinned base. Then three read-only calls: `git merge-base <head> <base>`, `git rev-list
--count <head>..<base>`, and `git merge-tree --write-tree --name-only -z --no-messages
--merge-base <mb> <head> <base>`, which exits 1 on a conflict and still prints the merged tree's id
followed by the conflicting paths. None of the three moves a ref or touches the index or the
working tree. The result is cached on the pair of commit ids, so a refresh with nothing new runs
no git. On git older than 2.38, which has no `--write-tree`, Orca says the details are unavailable
rather than guess.

**Marley today:** Nothing. The first survey uses `merge-tree` only to prove a branch merged
before deleting it, which the removal slice planned after #511 ports. Its merge plan showed how
far the base had moved, which #511's draft defers, and nothing says what would collide.

**What Marley would do:** A chip on each worktree agent's rail row: "12 behind main, conflicts in
3 files", with the files in its tooltip, computed against the local base branch whenever the
branch head or the base tip moves. It needs no pull request, so it serves a local merge and a
workflow-managed one (#511) alike. Run between two sibling worktree branches, the same call says
which pair of agents is changing the same parts of the same files, the collision that grows with
the number of agents on one repository.

**Hard parts:** Few. Keep the git calls off the UI thread and debounce them, since an agent commits
often. `--write-tree` writes the merged tree's objects into the object store, where gc prunes them.

**Size and ticket:** S: a small git helper in a Marley crate and a chip in `marley_rail`. No
ticket. It belongs with #510's nested worktree rows, or in #531, which puts PR state and +/-
counts on the same rows.

## 3. Programs that open a browser land in Marley's Browser tab

**What it is:** Orca exports `BROWSER="orca open-url --url %s"` into the terminals of its
headless runtime, so a program on the server that opens a browser opens the page in Orca's
browser on the desktop that shows that terminal, among the tabs of the worktree the command ran
in. Orca sets the variable only when neither the user nor its own process has one.

**Where it came from:** `src/main/ipc/pty/host-env/assembly.ts`; the verb in
`src/cli/specs/browser-basic.ts` and `src/cli/handlers/browser-tab.ts`; `browserOpenUrlOnClient`
in `src/main/runtime/runtime-browser-commands-browser-tab-create.ts`, which takes only http and
https and focuses the new tab (Orca PR #17467, 2026-08-30).

**How it works:** `BROWSER` is the Unix convention for the program that opens URLs. `gh` reads it
(`gh help environment`: `GH_BROWSER`, then `BROWSER`), and so do Python's `webbrowser` module,
Vite's `--open` (which also takes it from `.env`) and `cargo doc --open`. `xdg-open` does not. It
takes the desktop's `x-scheme-handler/http` default first and reads `BROWSER` only when that fails,
so a tool that opens through `xdg-open`, such as Node's `open` package, keeps going to the system
browser.

**Marley today:** #503 catches the URLs a terminal prints and the ones Chad clicks. A program
that opens a URL itself still opens the system browser, where no agent tool sees the page.

**What Marley would do:** A small opener beside the bridge Marley already writes for Zed's agents,
exported as `BROWSER` in the environment `crates/marley_terminal/src/shell_integration.rs` gives
local shells, only when the user has no `BROWSER` of their own. It is a plain path that takes the
URL as its argument: Vite runs `$BROWSER <url>` through Node's `open`, and Orca's `%s` template
works only where the tool substitutes it, as Python's `webbrowser` does. The opener sends the URL
and its working directory to Marley's MCP endpoint with the bearer, and Marley applies #503's rule,
under which a loopback or `*.localhost` URL opens a Browser tab, with the focus, in the project that
owns that directory, and anything else goes to the system browser. With #507 that tab is in the
project's own Chromium, so a dev server's auto-open (Vite's `--open`, a Jupyter server's token URL)
arrives in the right profile, where an agent can then drive the page.

**Hard parts:** It catches only programs that read `BROWSER`; catching `xdg-open` would take a
Marley `.desktop` handler for `x-scheme-handler/http`, which would replace the desktop's default
browser, more than this is worth. The opener hands other URLs to `xdg-open` with `BROWSER` unset,
so a machine with no scheme handler cannot loop back into it. `gh auth login --web` must reach
Chad's real browser, where he is signed in; the loopback rule sends it there, and an OAuth redirect
to `localhost` happens inside the browser that showed the login page, so it never reaches the
opener. The opener uses the bearer because an in-band terminal frame could be printed by any
program's output, and a printed frame would steer Marley's Chromium without a click. `cargo doc
--open` hands over a `file://` URL, which the loopback rule sends to the system browser (open
question 2). A shell started before Marley's endpoint exists, and an SSH terminal, whose remote
shell never sees the variable, fall back to the system browser as today.

**Size and ticket:** S once #503's routing exists. No ticket; a follow-up slice of #503, whose
draft does not cover it.

## 4. A PR URL printed in a terminal ties the PR to its row

**What it is:** When an agent's `gh pr create` prints the new PR's URL, Orca's worktree card
shows that PR at once instead of at its next poll.

**Where it came from:** A scan over PTY output finds GitHub PR URLs, carrying up to 512 characters
between reads so a URL split across two writes still matches, and stripping colour codes and
trailing punctuation (`src/shared/terminal-github-pr-link-detector.ts`, 193 lines). A hit is only
a hint. It forces a lookup of the PR for the worktree's branch, and Orca links the PR only when
that lookup returns the printed number, because "terminal output can carry arbitrary PR URLs
(docs/agents/logs)"
(`src/renderer/src/store/slices/worktrees/session/worktree-unread-activity.ts`). A PR already
linked to the worktree is never replaced by a different printed number. Orca keeps a circuit
breaker per API bucket for its `gh` calls (`src/main/git/gh-rate-limit-breaker.ts`).

**Marley today:** No PR state anywhere yet; #531 adds the chip.

**What Marley would do:** In the output scanner #503 adds for dev-server URLs, a printed PR URL
triggers `gh pr view --json number,state,statusCheckRollup` for the project's branch, and #531's
chip appears when the numbers agree. The `gh` calls get a rate limit.

**Size and ticket:** S, a paragraph in #531, whose draft does not have it yet.

## Five smaller details

Three are details for work already queued or shipped, and two are extras; none needs a ticket of
its own.

1. **A terminal read that leaves out what someone is typing (for #525).** `orca terminal read`
   finds the agent CLI's input box by its prompt glyph (`❯`, `›` or `»`) at the cursor, keeps only
   the glyph in the returned lines and returns the typed text as a separate `draft` field,
   "UI-only composer text, excluded from `tail`" (`src/shared/terminal-composer-draft.ts`, 201
   lines; `src/main/runtime/orca-runtime-terminal-projection.ts`;
   `src/shared/runtime-terminal-contracts.ts`). An agent reading Chad's Claude Code terminal then
   never takes his half-typed prompt for output. #525's `terminal_screen` reads live screens and
   does not do this yet. It is a screen heuristic tied to each CLI's glyphs, so it breaks when a
   CLI changes its input box.
2. **A setup command suggested for a repository that has none (for #510's second slice, the
   worktree's environment).** A sidebar card offers a setup command found in `conductor.json`,
   `.superset/config.json`, `.cmux/cmux.json` or `.codex/environments/environment.toml`, or else
   the install command for the one lockfile present (`src/shared/setup-script-imports.ts`,
   `setup-script-package-manager-suggestion.ts`,
   `src/renderer/src/components/sidebar/SetupScriptPromptCard.tsx`), so a new worktree has its
   dependencies before the agent's first command runs.
3. **Orca's redactor as a cross-check for #516.** `src/main/observability/redactor.ts` (308 lines)
   holds ordered, tagged rules: labelled key and value pairs; Anthropic, OpenAI, GitHub, AWS and
   Slack tokens; JWTs; PEM blocks; URL userinfo; `.env` lines. The tags tell a reader what kind of
   value was hidden. #516, shipped the same night, records the comparison in its spec: the same
   provider shapes, and two rules Marley lacked are now in its set (a token standing alone as a
   URL's userinfo, as git prints one, and Slack's `xoxe-` and `xoxo-` prefixes). Its `.env` rule was
   left out, because it would hide `PATH` and every other harmless value in `env` output; Marley
   hides secret-named values only.
4. **A note the first time Marley takes a key a terminal program would have received.** Orca's
   "Terminal shortcut handled" toast names the action and the keys, once per action
   (`src/renderer/src/lib/terminal-shortcut-capture-notification.tsx`). Marley takes Ctrl-G for
   rich input whenever an agent runs, on purpose; the note would say so once.
5. **Project icons from the repository's own files,** for the rail's project headers: a favicon
   or logo at the usual paths, or the icon `index.html` declares (`src/main/repo-icon-autodetect.ts`
   and three siblings, 1,190 lines). Orca also sends the `package.json` homepage to Google's
   favicon service and fetches GitHub avatars, which Marley should not do.

## Looked at and left out

Closing a desktop banner once its agent has been seen (Orca's `notifications:dismiss`) would be a
few lines, since gpui has `dismiss_system_notification` and Marley tags each banner per terminal,
but the notification daemon on Chad's desktop drops a normal banner after 8 seconds anyway.
Also left out: the rim flash on the pane a row jumps to, a worktree path printed in a terminal as a
"switch workspace" link, keeping local `main` fast-forwarded, fork sync, Markdown templates from
`.orca/templates`, the composer's "Create more" toggle (it keeps the dialog open and does not
launch several agents), a per-pane attention highlight, and the editor's Copy Context (Zed's
`editor: copy file location` gives the path and the lines).

## Recommendations, ranked

1. The quit guard (finding 1). S, depends on nothing, and stops the restart after each `just
   install` from killing agents mid-turn. A ticket of its own now, together with the Warp second
   pass's finding 2.
2. The conflict chip (finding 2). S, with #510's nested rows or in #531, since both draw on the
   same row.
3. `BROWSER` into the Browser tab (finding 3). S, a follow-up slice of #503, since it reuses #503's
   routing rule.
4. The PR URL as a hint (finding 4), a paragraph in #531.
5. The smaller ones only as they come up.

## Open questions for Chad

1. Should the quit prompt list every running agent, or only those the rail shows as working?
   Default: every running agent, each with its state.
2. Should `file://` pages from the project (`cargo doc --open`, coverage reports) open in a
   Browser tab too? Plan D15 limits agent navigation to http and https. Default: no, they go to
   the system browser.

## Sources

Orca, MIT, read from source at `stablyai/orca` commit `1c2cf120e3` (2026-09-25): the paths named
above, its `docs/` pages and reference notes, and its commit log. Marley: the survey in
`docs/orca_architecture/` (reports 02, 05 and 07 and the README),
`crates/marley_workbench/src/rail.rs`, `crates/marley_terminal/src/shell_integration.rs`,
`crates/terminal_view/src/terminal_view.rs`, `crates/zed/src/zed.rs`,
`crates/workspace/src/workspace.rs`, and the #516 spec
(`docs/planning/pipeline/completed/516-secret-redaction-for-agents.spec.md`). Tools: `gh help
environment`.
