# T3 Code survey 02: threads and the composer

Read from a local clone of `github.com/pingdotgg/t3code` at `17c0878941` (2026-10-06). Paths are relative to that
repo root unless they start with `crates/` (Marley). Line counts are non-test TypeScript.

## 1. Summary

T3 Code is a chat GUI over headless agents, so it owns everything a coding-agent TUI owns in
Marley's terminals: the prompt box, its history, the queue of follow-ups and the question
panels. Its sidebar sorts threads into five shelves (pinned, active, an optional Working fold,
snoozed until a wake time, settled), and a server sweep settles threads by itself when their pull
request merges or after three idle days. Its composer is a Tiptap editor whose chips (files,
images, terminal excerpts, picked elements, review comments, other threads, pull requests) are
stored as `t3-context://` links plus typed records, and the server expands them into a
`<t3_context>` envelope at turn start. Around it sit thread and message search, ArrowUp recall,
a prompt stash, quoted citations of an earlier reply, pages an agent draws as its reply (checked
in a headless browser first), notification sounds, and server-published themes that Omarchy
already feeds on every theme switch. The UI is large: `apps/web/src/components/ChatView.tsx` is
11,751 lines, `components/chat/ChatComposer.tsx` 7,593, `composerDraftStore.ts` 4,600 and
`components/Sidebar.tsx` 5,430. Most of the composer has no place in Marley, because Claude Code
and Codex keep their own prompt, queue and history in the terminal, and Zed's Agent Panel already
queues and steers. Worth taking: (1) follow Omarchy's theme the way T3 does; (2) search every
agent session's messages; (3) another agent's session as a reference read on demand; (4) one
prompt to several agents, each in its own worktree; (5) pages an agent draws, shown in a Browser
tab; (6) large pastes handed over as files; (7) a folder of its own for each agent started
outside a project; (8) a Working fold, snooze and sounds as refinements of the rail.

## 2. Features

### 2.1 The sidebar's shelves

**What the user sees.** Threads sit under their project in up to five shelves: Pinned at the
top, the active list, a collapsed Working section (beta, off by default), Snoozed with a wake
time, and Settled at the bottom. Each row carries a status pill: Pending Approval, Awaiting
Input, Working (with a running duration), Connecting, Waiting, Plan Ready or Completed. Rows that
do not need the user (working, or finished and already seen) recede. Dragging a row between
shelves pins, unpins, settles or wakes it, and the dragged card names the verb before the drop.
Move Up and Move Down sit in the row menu, and the server keeps the order, so other devices see
it.

**How it works.** `resolveSidebarThreadSection` puts snooze first, then settled, then pinned
(`apps/web/src/components/Sidebar.logic.ts:138-147`). The status comes from pending approvals,
pending questions and the runtime state, with a usage-limit failure read as `limited`
(`Sidebar.logic.ts:983-1005`); the pill adds Plan Ready and Completed
(`Sidebar.logic.ts:1164-1242`). Completed means the latest run finished after the thread was last
visited (`hasUnseenCompletion`, `Sidebar.logic.ts:773-782`); the visit time is server-side
(`thread.visit` and `thread.mark-unread` in `packages/contracts/src`), so a thread read on the
phone is read on the desktop too. Rows recede by `shouldRecedeSidebarThread`
(`Sidebar.logic.ts:963-981`). The Working beta lives in
`packages/client-runtime/src/state/threadInbox.ts`: a thread folds away while it runs or waits
on background work and has no approval, question or plan prompt (`threadInbox.ts:20-34`); the
inbox then orders by when each thread last came back to the user (`threadInbox.ts:45-62`), and
the Working fold by the user's last message, so finishing and waking do not move rows
(`threadInbox.ts:74-89`). A tracker notes the moment a thread left the fold, since the server
stamps no such time (`threadInbox.ts:110-145`). Drag planning is `planSidebarThreadDrop` and
`applySidebarThreadDrop` (`Sidebar.logic.ts:305-549`); shared order keys are in
`client-runtime/src/state/threadSort.ts`, so web and mobile agree.

**Good.** The status vocabulary separates "waiting on you" (approval, input, plan ready, failed,
limited) from "done and unseen" and from "busy". Folding busy threads away leaves a list of what
needs the user, ordered by when it came back.
**Bad.** Five shelves, an optional sixth fold and capability flags per server make the list hard
to predict; the beta turns off manual ordering while it is on.
**Size.** `Sidebar.tsx` 5,430, `Sidebar.logic.ts` 1,442, `threadInbox.ts` 145.
**Marley today: has part.** The rail lists terminals, agent CLIs and Zed threads per project with
their status ("The rail"), drags to reorder (#602), and orders by attention since #542: waiting
or failed unseen, then finished unseen, then working, then quiet, then idle, holding still under
the pointer. Rows light until shown (#538). There is no pin, no Working fold and no shelf for
finished work; an idle row stays until its terminal closes or its thread is archived (#605).

### 2.2 Settling, by hand and by rule

**What the user sees.** Settle Thread moves finished work out of the active list without
deleting it. By default the server settles a thread three days after its last activity, and a
thread whose pull request merged (or closed, when idle) settles at once. Work in progress, a
pending question or approval, and live background work hold a thread open. Settling closes the
thread's terminals that sit at an idle prompt and keeps those running a command. Auto-settle can
be disabled per thread. A drag along the Settle buttons settles a range.

**How it works.** `apps/server/src/orchestration-v2/ThreadSettlementService.ts` sweeps every
minute (`:572`) and after a merge event. `isAutoSettlementCandidate` rules out archived, pinned,
overridden, blocked, running, background-held and just-queued threads, and lets a snoozed thread
settle only if it woke on completion or error (`:138-163`). `pullRequestSettles` settles on a
merge or close only when the user wrote nothing after it; runs started by background work, a PR
watch or another agent do not count as the user resuming (`:110-136`). With several linked pull
requests, any open link holds the thread (`:174-195`). The inactivity rule is
`activityAtMs < now - days * DAY_MS` (`:205-207`). Idle shells close through
`terminals.closeIdle` after settlement (`:516-523`).

**Good.** Settlement is decided on the server from persisted state, so it runs with every client
closed, and a merge done by an agent with `gh pr merge` is caught when that run ends (see report
03). The rule that only the user's own messages count as resuming is exact.
**Size.** 597.
**Marley today: lacks**, and mostly does not need it. Marley's rows are live terminals: an idle
agent row is a running shell, and the attention order (#542) already sinks it. Zed threads
archive by hand (#605). The piece that does carry over is "a merged pull request finishes the
work": a worktree agent's row could say merged and offer Remove (#589). That belongs with report
03.

### 2.3 Snooze with a wake time

**What the user sees.** Snooze offers In 1 hour, In 3 hours, This evening, Tomorrow and Next
week, plus Custom (a date and time, or a duration). The thread waits on the Snoozed shelf,
soonest wake first, and returns to the active list at its wake time, marked as woken. Wake Now
cancels it. A snoozed thread that finishes or fails early may wake and settle.

**How it works.** Presets come from `resolveSnoozePresets`
(`packages/client-runtime/src/state/threadSettled.ts:255-300`); the evening preset is offered
only while it is more than an hour away. The client re-runs the classification at the exact
next wake boundary instead of on a minute tick (`Sidebar.tsx:2700-2706`). Snooze is a server
command (`thread.snooze`, `thread.unsnooze`).
**Size.** `threadSettled.ts` 354, `CustomSnoozeDialog.tsx` 226.
**Marley today: lacks.** For Chad's use the closest need is "keep this agent's row quiet until
tomorrow" while it holds a long-lived dev server or a parked session; with attention ordering,
an idle row already drops, so the gain is small.

### 2.4 Undo for sidebar actions

**What the user sees.** Unpin, Settle, Snooze, Archive and discarding a draft each show a notice
for five seconds; Undo, or `mod+z` outside a text field, reverses it, including a pinned
position. Several actions of the same kind in a row undo together.

**How it works.** `apps/web/src/hooks/showThreadUndoNotice.ts` keeps live undos in one list,
groups consecutive actions of the same kind (`:28-86`), consumes every claim before awaiting so a
double press cannot restore twice (`:58-61`), and commits them after five seconds (`:99-113`).
**Marley today: has part.** A working agent's terminal closed from its tab comes back with Undo
or Ctrl-Shift-T for a minute (#550). Archive and Move to Project have no undo; archived threads
come back through the project menu (#616).

### 2.5 Threads with no project

**What the user sees.** A thread can start with No Project. It works in a folder of its own,
named from the date, the first words of the first message and a short id, such as
`2026-09-25-convert-these-pngs-to-webp-a1b2c3d4`. The folder outlives the thread, and branch and
diff controls stay hidden there.

**How it works.** `apps/server/src/project/ManagedProjectFolders.ts` owns `<baseDir>/scratch` and
`<baseDir>/projects/<slug>` (`:5-7`). The folder name is built from the date, capped words of the
first message and an id (`:132`, `:344-351`). It is refused when the data directory itself sits
inside a git checkout (`:263-268`).
**Good.** An agent never runs in the home folder, and each scratch job's files stay together.
**Marley today: has part.** Groups with no folder (#600) start their terminals and agent CLIs in
the home folder ("Groups with no folder (#600)"). A Claude Code started in the home folder sees
every file there, and two scratch jobs share it.

### 2.6 Start in the background, and one prompt to several models

**What the user sees.** In a new thread, `mod+Enter` starts it and opens a fresh draft at once,
keeping the workspace mode and base branch; with New Worktree, each submission gets its own
worktree. Shift-click in the model picker selects several models; one send starts one thread and
one worktree per model, while the user stays in the composer.

**How it works.** The fan-out loop checks each selected provider is ready, formats the prompt
for that provider and model, and collects one target per selection
(`apps/web/src/components/ChatView.tsx:9035-9077`); each target becomes its own thread launch.
The worktree side is report 03.
**Good.** Comparing Claude Code's and Codex's answer to one task on the same base is a single
action.
**Marley today: has part.** New Agent in Worktree (#510) takes one agent per prompt, opens the
worktree's workspace behind the current one ("Worktree agents"), and gives each worktree its own
port slot (#590). There is no multi-agent send.

### 2.7 Search across threads and messages

**What the user sees.** The sidebar's search, and the command palette (`Cmd/Ctrl+K`) across
connected machines, match thread titles and linked pull requests at once and, after two
characters, the text of the user's messages and the agents' final replies. A hit shows a
snippet around the match.

**How it works.** `apps/server/src/orchestration-v2/ThreadSearch.ts` runs one `LIKE` query over
finished user and assistant messages of live threads in live projects (`:63-67`, `:83-137`),
keeps one best match per thread with user messages ranked above assistant ones, then newest
(`:117-123`), and cuts a 240-character snippet around the first match (`:46-61`). The default
limit is 50 (`:143`). The query text is user content and never goes into the error
(`:16-27`). The client merges title matches before content matches and keeps the list's order
(`apps/web/src/components/Sidebar.logic.ts:1080-1112`), and one query atom per environment
merges their answers (`packages/client-runtime/src/state/threadSearch.ts:50-80`).
**Good.** It is small (168 lines), needs no index, and answers "which conversation did I fix
this in" from the palette.
**Bad.** A `LIKE` scan over JSON text grows with history; there is no ranking beyond role and age.
**Marley today: has part.** The rail's filter matches names and titles only ("The filter").
Zed's thread archive searches Agent Panel thread titles by fuzzy match
(`crates/agent_ui/src/threads_archive_view.rs:104`, `:307`). Claude Code's own `/resume`
searches past sessions of the folder it runs in. Nothing searches the messages of every Claude
Code and Codex session Chad has run, across projects, from Marley.

### 2.8 Context chips and the provider envelope

**What the user sees.** Context lands at the cursor as a chip inside the prose: a file, an image
(also kept on a thumbnail shelf), a terminal excerpt, a picked page element, a preview
annotation, a review comment from a diff, a skill, a file mention, another thread (`@`), or a
pull request (`#`, with its state colour). Text around a chip stays editable; a chip deletes like
a character; copy and paste carry chips, with their payload, to another draft or thread. The
copy button gives other apps plain Markdown with a link per chip.

**How it works.** `docs/internals/composer-context-references.md` is the design. A chip is a
Markdown link `[label](t3-context://v1/<kind>/<contextId>)` in the message text, and its payload
is a typed record in `message.context.records`; records never hold bytes and bind to
attachments by id (`packages/contracts/src/composerContext.ts:105-232`). At turn start the
server replaces each link with a marker such as `[Image: shot.png; ref=ctx_1]` and appends one
`<t3_context version="1">` envelope with an entry per referenced record
(`packages/shared/src/composerContextReferences.ts:112-131`, `:265-300`). Captured text is
escaped so a terminal line or a PR comment that contains `</t3_context>` cannot close the
envelope and forge a record (`composerContextReferences.ts:134-142`). Unknown kinds decode to a
catch-all with their payload kept, so an older client never drops a newer record. The clipboard
carries a structured fragment beside the Markdown, and a paste re-fetches images from the source
environment.
**Good.** One send path for every kind of context, the persisted message stays readable, the
agent gets labelled data with an explicit boundary, and forging is closed off.
**Bad.** It needed a legacy upgrader for three earlier formats
(`packages/shared/src/composerContextLegacy.ts`), and the composer is now two editor models with
a coordinate mapping between them (`docs/internals/composer-editors.md`).
**Size.** Contracts and codecs about 620; the editor and draft store several thousand.
**Marley today: has part.** Marley sends context into the agent's own prompt as text: a pick as
`[browser pick 1: …; browser_pick id 1]` ("The element picker and the pick tray"), a selection
as `@src/auth.rs#L12-40` for Claude Code (#549), a block as its Markdown or, when long, as a
reference the agent reads with `terminal_read` (#555), review notes as a prompt with `File:` and
`Line:` (#522), files as quoted paths ("Attach File"). That suits a TUI, which cannot draw chips.
What Marley lacks from this design is the envelope boundary: a short block goes in as plain
Markdown, so its output is indistinguishable from the user's words.

### 2.9 Another thread as a reference

**What the user sees.** `@` and part of a title, or a thread dragged from the sidebar onto the
composer, adds that thread as a chip. Only a reference travels; the agent reads the other
thread's history when it wants to, so a long thread costs nothing until then.

**How it works.** The record carries the environment, thread id and a title snapshot
(`packages/contracts/src/composerContext.ts:225-232`); the agent reads it through the
`t3_thread_read` tool, paged by position and by text offset for long items
(`apps/server/src/mcp/toolkits/orchestrator/tools.ts:190-202`; report 06 covers the toolkit).
**Good.** Handing one agent's findings to another is a mention, not a paste.
**Marley today: has part.** Zed's Agent Panel can mention another Agent Panel thread
(`crates/acp_thread/src/mention.rs:35`). For terminal agents, `fleet_snapshot` lists each Claude
Code session and what it is doing (#547), and `terminal_read` reads a terminal's blocks, but no
tool reads another agent's conversation, and nothing in the rail sends "read that session" to an
agent.

### 2.10 Large pastes become files

**What the user sees.** A paste of 32 KiB or more, or one that would push the message over its
120,000-character limit, becomes a text-file attachment named `pasted-text.txt` (then
`pasted-text-2.txt`), so the agent can read parts of it. `Ctrl+Shift+V` keeps it inline. A
message takes up to 100 attachments: images to 10 MiB each and 80 MiB in all, other files to
50 MiB.

**How it works.** `pastedTextDisposition` measures both characters and UTF-8 bytes against the
threshold (`packages/client-runtime/src/textPaste.ts:1`, `:19-38`); file names come from
`nextPastedTextFileName` (`:40-52`). The limits are constants in
`packages/contracts/src/chatAttachment.ts:13-17`. Attachments live outside the workspace, and a
path in the prompt grants no access the provider's sandbox would refuse
(`docs/internals/providers.md`, "Attachments and stored history").
**Good.** A pasted log of 20,000 lines no longer fills the context window in one go.
**Marley today: lacks.** #536 sends a multi-line paste into an agent as one bracketed paste and
drops an image in as its path, but a large text paste goes in whole. Claude Code folds it into
`[Pasted text #1 +N lines]` on screen and still sends all of it.

### 2.11 Cite in composer

**What the user sees.** Select text inside one assistant reply and choose Cite in Composer; the
quote becomes a chip with an optional comment. Selecting the chip scrolls back to the source,
highlighted; the saved quote stays readable if the source changes.

**How it works.** The quote and its source identity travel inside the message text
(`packages/shared/src/assistantCitations.ts`, 186 lines); locating the source uses selectors over
rendered, whitespace-normalised text and refuses ambiguous repeats
(`docs/internals/assistant-citations.md`).
**Marley today: lacks**, with little loss. In a terminal the reply is text Chad can select and
copy (#536 drops the agent's gutter) into the same agent's prompt.

### 2.12 Prompt recall and the stash

**What the user sees.** ArrowUp in an empty composer brings back this thread's earlier prompts,
text only; ArrowDown walks forward and back to the draft. `mod+S` stashes the prompt and its
attachments; on an empty composer the same key restores one stash or opens a menu of them.

**How it works.** Recall is derived from the thread's user messages on each key press, with no
store (`apps/web/src/components/chat/composerPromptHistory.ts:4-11`). The stash keeps 20 entries
in browser storage, with an attachment budget per entry (`apps/web/src/promptStashStore.ts:11`,
`:21`, `:31`).
**Marley today: has part.** Claude Code's and Codex's own prompts recall with Up. Rich Input
keeps one draft per terminal on Escape ("Rich input"), and the shell's prompt editor suggests
from history (#637), but Rich Input for an agent has no recall of prompts already sent. Zed's
Agent Panel keeps drafts (`crates/agent_ui/src/draft_prompt_store.rs`).

### 2.13 Queue and steer, and the held queue

**What the user sees.** While a turn runs, Send either queues the message for the next turn or
steers the running one, by a per-client default; `mod+Enter` does the other. Queued messages
show above the composer, editable, reorderable by drag, removable, and promotable to a steer.
After a server restart the queue is kept but held: nothing runs until the user presses Resume.

**How it works.** Commands `queued-run.edit`, `queued-run.reorder` and `queued-run.cancel`
(`packages/contracts/src`). Delivery order puts automatic delegated completions first, then the
user's queue position (`apps/server/src/orchestration-v2/QueuedRunOrder.ts:12-31`). On recovery
every queued run is marked `queueHeld`, "require explicit consent before draining them"
(`apps/server/src/orchestration-v2/ProviderRuntimeRecoveryService.ts:287-299`).
**Good.** The hold after a restart: messages written for a context that may be gone do not fire
on their own.
**Marley today: has.** Claude Code and Codex queue messages typed while they work, in Marley's
terminals; Rich Input sends into that same queue. Zed's Agent Panel has a queue with
Send Immediately, Edit First Queued Message and a steer toggle
(`crates/agent_ui/src/agent_ui.rs:285-296`). Marley's restart resume (#540) re-runs
`claude --resume` and types nothing.

### 2.14 Question panels with attachments

**What the user sees.** When an agent asks a question that takes a free answer, the panel offers
Attach Files and image paste beside the options; each question keeps its own attachments.

**How it works.** `apps/web/src/components/chat/ComposerPendingUserInputPanel.tsx` (299 lines);
answers upload to the machine running the thread and reach the agent as paths
(`docs/user/question-attachments.md`).
**Marley today: has part.** The Needs You inbox lists Claude Code's questions with their options
(#570) and Codex's approvals with decisions (#651); a Claude Code question is answered in its
terminal, where Attach File and drops work. Answering from the inbox is report 01's subject.

### 2.15 Activity summaries

**What the user sees.** Consecutive tool calls fold into one line, such as "Ran 2 commands and
sent messages to 3 threads", naming at most two categories, commands and edits before reads.
Expanding shows inputs, status and exit codes; Open Diff opens a file change.

**How it works.** `packages/client-runtime/src/work-log/presentation.ts:644-731`
(`summarizeToolGroup`, priority by action kind).
**Marley today: has part.** The rail row shows the tool in flight (#519), and each turn under
Turns (N) shows its prompt and how many files it changed ("Per-turn diffs"). The TUI shows its
own tool calls.

### 2.16 Visual replies

**What the user sees.** Asked for a chart, a table, a diagram or a mockup, the agent answers with
a self-contained HTML page shown in the thread above its text reply, themed like the app and
following light and dark mode. Scripts run sandboxed from the app; links open in the browser;
local images are inlined when the page is published, so it survives moving the files.

**How it works.** Two MCP tools (`apps/server/src/mcp/toolkits/html/tools.ts`):
`html_preview` renders the page in T3's own headless Chrome and returns a PNG, the height the
page needs and its console output, so the agent can check its work (`:25-67`); `html_render`
publishes it to the thread with a title and height (`:70-108`). Pages are capped at 512,000
characters of HTML (`:18-20`), 10 MiB per image and 25 MiB inlined
(`apps/server/src/htmlRender/HtmlRender.ts:38-40`). The agent is told the CSS variables the app
injects (`--background`, `--foreground`, `--chart-1` to `--chart-6` and others) and how to lay
out a page inside a reply (`packages/shared/src/htmlRender.ts:250-267`). The frame and the page
talk over the MCP Apps protocol, JSON-RPC over `postMessage` (`htmlRender.ts:269-274`).
**Good.** A preview-then-publish pair of tools makes the agent look at its own output before
the user does.
**Size.** About 1,640 (server, shared and the frame).
**Marley today: has part.** An agent can write an HTML file and open it in a Browser tab of its
project with `browser_open_url` (#561, #586), then read it back with `browser_look` and
`browser_console` ("The tools"). There is no tool that takes the page itself, no theme variables
from Marley's theme, and the page opens in the project's own Chromium profile, with its logins.

### 2.17 The file viewer

**What the user sees.** A file chip or an agent's file link opens beside the conversation:
code and JSON highlighted; Markdown, HTML, CSV and TSV with rendered and raw views; HTML and PDF
as pages; audio with controls. Files outside the workspace open read-only, and an HTML file
there cannot load its neighbours.
**Marley today: has part.** Zed opens any file in an editor, with Markdown preview
(`crates/markdown_preview`), CSV as a table (`crates/tabular_data_preview`), images and SVG.
Local HTML opens in a Browser tab (#586). PDFs have no viewer.

### 2.18 Notifications

**What the user sees.** Per client: Off, Notifications only, Sound only, or both. Titles are
Thread completed, Approval needed, Input needed, Usage limit reached and Thread failed, with the
thread's title. While the app has the focus on another thread, a toast with Open Thread replaces
the desktop notification. Two sounds, one for completion and one for needing input. A badge
counts unseen notifications on the dock icon or the favicon.

**How it works.** `apps/web/src/components/ThreadNotificationCoordinator.tsx` compares each
thread's attention state and completion time with the previous render and fires on a change
(`:124-155`), picks the title (`:157-166`), shows a toast when focused (`:172-205`) or a desktop
notification otherwise (`:207-231`). Sounds and the badge are in
`apps/web/src/threadNotifications.ts` (`:7-12`, `:25-65`, `:76-100`).
**Marley today: has part.** Desktop notifications from terminals (#478) say what happened
(`repo: Claude needs input` over the question, #538), from Codex and OpenCode too (#552), and to
the phone through ntfy (#535). Marley plays no sound for a terminal agent. Zed plays
`Sound::AgentDone` for its own agent (`crates/audio/src/audio.rs:30`).

### 2.19 Keybindings and quitting

**What the user sees.** `keybindings.json` holds rules of `key`, `command` and an optional
`when` over context keys such as `terminalFocus`, `composerFocus` and `turnRunning`, with `!`,
`&&`, `||`; the last matching rule wins. Quit needs a 1.2-second hold or a double press.
**How it works.** `apps/server/src/keybindings.ts` (761) and `apps/web/src/keybindings.ts`
(522); the desktop main process intercepts the quit accelerator
(`apps/web/src/components/QuitHoldOverlay.tsx:10-11`).
**Marley today: has.** Zed's keymap contexts are more expressive, and #550 asks before a quit
ends a working agent.

### 2.20 Themes, and Omarchy's hook

**What the user sees.** Themes per light and dark mode, a theme editor that picks a colour by
clicking the app, VS Code and Open VSX theme import, and environment themes: a JSON file dropped
into `~/.t3/userdata/themes/` on the server is offered to every client, and `t3 theme set <id>`
switches connected clients to it. A short form with only `appearance`, `canvas` and `accent`
(and optional overrides) is enough; T3 derives the rest. The page says to write the file to a
temporary name and rename it into place.

**How it works.** `apps/server/src/environmentTheme.ts` (307) watches the directory and streams
the published set to clients, skipping invalid files (`:4-12`, `:138-190`). The palette
derivation is `packages/shared/src/themePalettes.ts`. Omarchy ships a hook for exactly this:
`omarchy-theme-set-t3code` renders its `t3code.json.tpl` template from the current theme's
colours on every theme switch and writes `omarchy.json` into T3's themes folder atomically, so
T3 Code retints with the desktop. Omarchy's theme switch also renders user templates from
`~/.config/omarchy/themed/` and has hooks for Claude Code, VS Code, Obsidian and others, but none
for Zed.
**Good.** A watched folder of small JSON files is the whole integration surface.
**Marley today: has part.** Zed has themes, a light and dark choice, theme extensions, and
reloads its themes folder when a file changes (`watch_themes`, `crates/zed/src/main.rs:1967`).
Marley does not follow Omarchy: a theme switch on the desktop leaves Marley in its old colours.

## 3. Bring to Marley

1. **Follow Omarchy's theme.** *Why.* An Omarchy theme switch retints its terminals, Chromium,
   Claude Code, Obsidian and T3 Code; Marley keeps its old colours. *Seam.* Ship a Zed theme
   template, `marley.json.tpl`, for Omarchy's user templates folder, and a small watcher in
   `marley_workbench` that copies the rendered file into Marley's themes folder (atomic rename)
   when it changes; Zed's `watch_themes` reloads it, and a theme
   setting naming "Omarchy" picks it up. Map Omarchy's `colors.toml` keys (`background`,
   `foreground`, `accent`, the shades and `color0` to `color15`) to Zed's theme keys, with the
   terminal's ANSI colours taken straight across. *Size.* S to M. *Hard.* Zed's theme schema
   has about 150 keys, so most are derived (T3's `themePalettes.ts` shows the derivation from a
   canvas and an accent). Omarchy has already moved its current-theme folder once, so read it
   through the template mechanism rather than a hard-coded path. Opt-in, since a user who picked
   a Zed theme should keep it.

2. **Search every agent session's messages.** *Why.* "Which session fixed the flaky test" is
   answered today by `/resume` in the right folder, one CLI at a time. *Seam.* A pure parser in
   `marley_agent` for Claude Code's `projects/<encoded cwd>/*.jsonl` and Codex's session files,
   reading user prompts and final replies only, plus Zed's own thread store; a picker
   (`marley: search agent sessions`) with T3's ranking (one best hit per session, the user's
   own words first, then newest) and a 240-character snippet around the match
   (`apps/server/src/orchestration-v2/ThreadSearch.ts:46-61`, `:83-137`). Enter opens a terminal
   running `claude --resume <id>` or `codex resume <id>` in the session's folder, as #540 does.
   *Size.* M. *Hard.* Both formats are undocumented and change between releases; keep to the
   two CLIs and fail soft on records that do not parse. Results show text that may hold
   secrets, so pass snippets through #516's redaction. This is also the unbuilt half of Orca
   report 01's item 7 (session history).

3. **Another agent's session as a reference.** *Why.* Chad runs Claude Code and Codex side by
   side; telling one to "look at what the other found" means copying text between terminals.
   *Seam.* A `marley_mcp` tool, `agent_session_read`, that takes a terminal id from
   `terminal_list` (or a session id) and pages through that session's prompts and replies, by
   position and by text offset for long items, as `t3_thread_read` does
   (`apps/server/src/mcp/toolkits/orchestrator/tools.ts:190-202`). In the rail, Send to Agent on
   an agent row types a one-line reference into the chosen agent, as #555 does for a long block.
   *Size.* M, less after item 2's parsers. *Hard.* Another agent's transcript is untrusted text
   to the reader: wrap it as data, redact it, and never include tool outputs by default.

4. **One prompt to several agents, each in its own worktree.** *Why.* Running Claude Code and
   Codex on the same task from the same base, then reviewing both, is the comparison Chad does
   by hand. *Seam.* The New Agent in Worktree prompt (#510) gains agent checkboxes; one Enter
   makes one worktree, branch and port slot (#590) per agent and starts each with the prompt on
   its command line. The drift chips and Review (#560, #511) then compare them. *Size.* S to M.
   *Hard.* Claude Code's trust question per new worktree (#587) arrives once per agent; name the
   branches so the pair reads as a pair (`agent/<name>-claude`, `agent/<name>-codex`).

5. **Pages an agent draws, in a Browser tab.** *Why.* Test timings, a dependency graph or a
   benchmark table read better as a chart than as terminal text, and the Browser tab is already
   there. *Seam.* Two `marley_mcp` tools after T3's pair: `page_check` (HTML in, screenshot,
   content height and console out) and `page_show` (HTML and a title in; Marley writes the page
   under its data folder, inlines local images, injects the active Zed theme as CSS variables,
   and opens it in a Browser tab of the caller's project without taking the focus). Reuse T3's
   agent-facing theme and layout guide as the tool description
   (`packages/shared/src/htmlRender.ts:250-267`). *Size.* S to M. *Hard.* The project's
   Chromium carries the project's logins (#507); open agent pages in a context with no profile,
   and keep them on `file://` or a Marley-owned origin.

6. **Large pastes handed over as files.** *Why.* A 5,000-line log pasted into Claude Code goes
   into the context whole. *Seam.* #536's paste path in `marley_workbench`: above 32 KiB (bytes
   or characters, `packages/client-runtime/src/textPaste.ts:19-38`), write the text to
   `pasted-text.txt` (then `-2`, `-3`) and paste its quoted path instead, with a key that keeps
   the paste inline. *Size.* S. *Hard.* Where the file lives: inside the project it shows in
   git status unless ignored; outside it, Claude Code may ask before reading. A folder under the
   project's `.git` (not tracked, inside the repository) is one answer; ask Chad.

7. **A folder of its own for each agent started outside a project.** *Why.* An agent from a
   group's `+` starts in the home folder (#600), where it can read everything and where two jobs
   mix their files. *Seam.* `marley_workbench`'s group launch creates a folder such as
   `~/scratch/<date>-<agent>-<id>` and starts there (T3's naming is in
   `apps/server/src/project/ManagedProjectFolders.ts:344-351`; Marley does not know the first
   prompt at launch, so the folder takes no words from it).
   *Size.* S. *Hard.* Claude Code asks to trust each new folder; #587's trust card covers
   worktrees only and would need to cover these.

8. **A Working fold, snooze and sounds for the rail.** *Why.* Refinements of #542 for days with
   many agents. *Seam.* `marley_rail`: a collapsible Working group per project for rows that are
   busy and need nothing (T3's test in `packages/client-runtime/src/state/threadInbox.ts:20-34`),
   with the rest ordered by when each came back to Chad (`:45-62`); Snooze on a row's menu with
   T3's presets (`threadSettled.ts:255-300`), hiding the row until its time; and a sound for
   "needs you" and another for "finished" through Zed's `audio` crate, which already plays
   `Sound::AgentDone`. *Size.* S each. *Hard.* The attention order and a fold must not move a row
   under the pointer, which #542 already guards.

Smaller things worth a day each: Up in an empty Rich Input recalls the prompts the plugin
reported for that terminal (`composerPromptHistory.ts`); a short block sent with #555 goes inside
a labelled envelope with T3's escaping, so its output reads as data
(`composerContextReferences.ts:134-142`); files dropped on an agent's rail row go to that agent
as paths (`apps/web/src/sidebarPendingFileDropStore.ts`); a turn row under Turns (N) counts its
commands as well as its files (`work-log/presentation.ts:644-731`).

## 4. Skip

- The composer as built: Tiptap, the chip node, two coordinate systems and a draft store of
  4,600 lines. The prompt belongs to the agent CLI in Marley's terminal, and Rich Input is a Zed
  editor that hands text to it; text references (#549, #555, picks) are the right form there.
- Queue and steer: Claude Code and Codex already queue in the terminal, and Zed's Agent Panel
  queues and steers (`crates/agent_ui/src/agent_ui.rs:285-296`).
- The prompt stash and Cite in Composer: Rich Input keeps a draft per terminal, Zed's panel keeps
  drafts, and a terminal reply copies cleanly since #536.
- Settling by inactivity days and the settled shelf: Marley's rows are live terminals that sink
  in the attention order; a merged worktree row is report 03's.
- Pins, the Move Up and Move Down menu items and drag between shelves: drag to reorder (#602)
  and attention order already cover arrangement.
- The keybinding `when` language, the keybindings page and hold-to-quit: Zed's keymap contexts
  and #550.
- The theme editor, VS Code and Open VSX import, and the panel animation slider: Zed's theme
  extensions and settings.
- Regenerating a title with a model: the CLI sets the terminal title, and rows rename (#452).
- Dock badges: Hyprland has no dock.

## 5. Open questions

1. **Omarchy's theme.** Follow it whenever Omarchy is present, or only when a setting says so?
   *Default: an opt-in setting, off, so a chosen Zed theme stays.*
2. **Session search scope.** Claude Code and Codex sessions in every folder, or only the open
   projects'? *Default: both CLIs, every folder, read on the machine only, snippets redacted.*
3. **Large pastes.** Turn it on, and where do the files go? *Default: off until Chad picks a
   place; 32 KiB when on.*
4. **Agent pages.** Open them in a context with no profile, or in the project's Chromium where a
   page could reach a logged-in dev server? *Default: a context with no profile.*
5. **Several agents from one prompt.** Which agents does the prompt offer, and with which
   permissions? *Default: the agent CLIs on the PATH, each with the project's permission mode
   (#532).*
