# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-713](open/TICKET-713-rail-cleanup.md) | chore | the rail: unregister a swapped sidebar, refresh only on row changes, a silent swap, save after restore |
| [TICKET-771](open/TICKET-771-codex-0162-in-the-tested-range.md) | chore | Codex 0.162 in the tested range, so #750, #752 and #753 reach the box's Codex |
| [TICKET-750](open/TICKET-750-codex-resumes-and-a-cut-off-turn-continues.md) | feature | Codex sessions resume after a restart, and a cut-off turn offers Continue |
| [TICKET-751](open/TICKET-751-restored-scrollback.md) | feature | restored scrollback: a terminal's earlier output, dimmed, after a restart |
| [TICKET-764](open/TICKET-764-restored-blocks.md) | feature | restored blocks over #751's scrollback |
| [TICKET-752](open/TICKET-752-codex-push-and-answers.md) | feature | push for Codex and Zed's threads, and Codex's questions answered from Needs you |
| [TICKET-753](open/TICKET-753-codex-turn-checkpoints.md) | feature | per-turn checkpoints for Codex |
| [TICKET-714](open/TICKET-714-undo-a-turn.md) | feature | Undo This Turn and Keep on a turn row, for Claude Code and Codex |
| [TICKET-715](open/TICKET-715-usage-meter-and-resume-at-reset.md) | feature | a usage meter on agent rows; Codex's limited state and Continue at Reset |
| [TICKET-765](open/TICKET-765-claude-codes-limited-state.md) | feature | Claude Code's limited state and Continue at Reset |
| [TICKET-754](open/TICKET-754-cli-sessions-in-threads.md) | feature | Claude Code and Codex sessions in Threads, reopened with `--resume` |
| [TICKET-712](open/TICKET-712-lsp-tools-for-agents.md) | feature | `editor_diagnostics`, `editor_definition`, `editor_references`, `editor_hover` for every agent, over Zed's LSP store |
| [TICKET-755](open/TICKET-755-three-small-hardening-fixes.md) | chore | Codex's config written safely, a token per terminal, Chromium units survive a killed renderer |
| [TICKET-720](open/TICKET-720-mcp-manager-and-cli-update-checks.md) | feature | a tab of every CLI's MCP servers, and warnings for CLIs too old |
| [TICKET-768](open/TICKET-768-edit-cli-mcp-servers.md) | feature | edit each CLI's MCP servers from #720's tab |
| [TICKET-756](open/TICKET-756-the-browser-tools-agents-lack.md) | feature | the browser tools agents lack: wait, hover, select, drag, upload, dialogs, device sizes, colour scheme |
| [TICKET-766](open/TICKET-766-mcp-annotations.md) | chore | MCP annotations on every Marley tool; the guide's tool tables caught up |
| [TICKET-767](open/TICKET-767-file-choosers-downloads-and-take-over.md) | feature | file choosers, downloads and Take Over in the Browser tab |
| [TICKET-759](open/TICKET-759-follow-omarchys-theme.md) | feature | follow Omarchy's theme (opt-in, Omarchy only) |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | Marley's own remote server, so SSH projects open (slice 1 of the release identity) |
| [TICKET-758](open/TICKET-758-a-remote-managers-thread.md) | feature | a remote harness's manager as a thread: a Manager entry per harness, seat threads from remote windows |
| [TICKET-760](open/TICKET-760-marley-calls-itself-marley.md) | chore | Marley calls itself Marley: app id, window class, name, socket (release identity slice 2) |
| [TICKET-761](open/TICKET-761-marleys-keyring-label.md) | chore | Marley's keyring label (slice 3) |
| [TICKET-762](open/TICKET-762-the-marley-scheme.md) | chore | the `marley://` scheme and the desktop entry (slice 4) |
| [TICKET-763](open/TICKET-763-the-updater-off-with-marleys-explanation.md) | chore | the updater off, with Marley's own explanation (slice 5) |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-717](open/TICKET-717-second-opinion-and-handoff.md) | feature | after the finish (Chad, 2026-10-11): New Thread with Context, then a second opinion and hand-off between agents |
| [TICKET-716](open/TICKET-716-delegation-tools.md) | feature | after the finish: agents that start agents, tracked and never lost; waits on a harness request (seat labels, an ended state) |
| [TICKET-769](open/TICKET-769-a-drafted-message-for-review.md) | feature | after the finish: a drafted message for the user to review (from #716) |
| [TICKET-770](open/TICKET-770-one-prompt-to-several-agents.md) | feature | after the finish: one prompt to several agents (from #716) |
| [TICKET-718](open/TICKET-718-failed-checks-to-an-agent.md) | feature | after the finish: a PR's failed checks sent to an agent; the GitHub inbox later |
| [TICKET-726](open/TICKET-726-plan-review-in-a-tab.md) | feature | after the finish, the review batch: plan review in a center tab (in slices) |
| [TICKET-727](open/TICKET-727-plan-versions-and-diff.md) | feature | the review batch: plan versions and a diff between them |
| [TICKET-728](open/TICKET-728-question-cards-in-plans.md) | feature | the review batch: question cards in plans |
| [TICKET-729](open/TICKET-729-notes-on-markdown-and-last-reply.md) | feature | the review batch: notes on any Markdown file or the agent's last reply |
| [TICKET-730](open/TICKET-730-agent-findings-as-diff-notes.md) | feature | the review batch: agent findings as diff notes over MCP |
| [TICKET-731](open/TICKET-731-viewed-marks-in-the-diff.md) | feature | the review batch: viewed marks in the project diff |
| [TICKET-732](open/TICKET-732-a-feedback-template-for-review-notes.md) | feature | the review batch: a feedback template for review notes |
| [TICKET-733](open/TICKET-733-guided-review.md) | feature | the review batch: guided review |
| [TICKET-757](open/TICKET-757-claude-code-remote-control-switch.md) | feature | shelved (Chad: "yes and shelve"): a switch for Claude Code's Remote Control |
| [TICKET-719](open/TICKET-719-scheduled-runs.md) | feature | moved to rustal-harness (Chad: "harness yes"); Marley shows the runs once the harness has a scheduler |
