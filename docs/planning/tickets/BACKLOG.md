# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-831](open/TICKET-831-a-project-mark-from-logo-png.md) | feature | A project's mark from its logo.png (W1.2) |
| [TICKET-832](open/TICKET-832-a-sessions-view-for-tmux.md) | feature | A Sessions view for every tmux session (W1.3) |
| [TICKET-833](open/TICKET-833-the-bottom-row-says-what-has-the-focus.md) | feature | The bottom row says what has the focus (W1.4) |
| [TICKET-827](open/TICKET-827-rail-rows-say-their-state.md) | feature | Rail rows say their state by color (W2) |
| [TICKET-825](open/TICKET-825-agent-thread-with-panels.md) | feature | The agent thread with panels around it (W2) |
| [TICKET-826](open/TICKET-826-terminal-blocks-with-an-agent-beside.md) | feature | Terminal blocks with the agent beside them (W2) |
| [TICKET-834](open/TICKET-834-the-window-chrome.md) | feature | The window chrome: title bar, tabs and menus (W2.1) |
| [TICKET-835](open/TICKET-835-home-one-prompt.md) | feature | Home: one prompt, then what needs you (W2.2) |
| [TICKET-836](open/TICKET-836-threads-with-a-preview.md) | feature | The Threads page with a preview (W2.3) |
| [TICKET-837](open/TICKET-837-side-panels-as-one-inspector.md) | feature | Side panels as one inspector (W3.1) |
| [TICKET-838](open/TICKET-838-the-bottom-drawer.md) | feature | The bottom drawer: Problems and Debug (W3.2) |
| [TICKET-839](open/TICKET-839-pickers.md) | feature | Pickers: palette, agent picker, file finder (W3.3) |
| [TICKET-840](open/TICKET-840-needs-you-toasts-and-prompts.md) | feature | Needs you, toasts and prompts (W3.4) |
| [TICKET-841](open/TICKET-841-forge-as-a-fleet-provider.md) | feature | Forge as a fleet provider (W4.1) |
| [TICKET-842](open/TICKET-842-forge-sprint-board.md) | feature | Forge's sprint board (W4.2) |
| [TICKET-843](open/TICKET-843-forge-around-marley.md) | feature | Forge around Marley (W4.3) |
| [TICKET-844](open/TICKET-844-settings-window.md) | feature | The settings window and every setting's control (W5) |
| [TICKET-845](open/TICKET-845-rusty-screens.md) | feature | Rusty's screens restyled (W5) |
| [TICKET-846](open/TICKET-846-mcp-servers-and-agent-access.md) | feature | MCP Servers, Agent Activity and what agents may do (W5) |
| [TICKET-847](open/TICKET-847-system-one.md) | feature | System One restyled (W5) |
| [TICKET-848](open/TICKET-848-fleet-and-harness.md) | feature | Fleet, harness sessions and scheduled runs (W5) |
| [TICKET-849](open/TICKET-849-components.md) | feature | The Component Library and Component tabs (W5) |
| [TICKET-850](open/TICKET-850-browser-tab-and-tools.md) | feature | The Browser tab and its tools (W5) |
| [TICKET-851](open/TICKET-851-worktrees-and-review.md) | feature | Worktrees, plan review, turns and diff review (W5) |
| [TICKET-852](open/TICKET-852-workflows-and-launch.md) | feature | Workflows, launch configs and remote terminals (W5) |
| [TICKET-853](open/TICKET-853-splits-and-ports.md) | feature | Splits, the switcher, ports, containers and collab (W5) |
| [TICKET-854](open/TICKET-854-search-and-more.md) | feature | Project search, recent, branches, history, extensions, the guide (W5) |
| [TICKET-855](open/TICKET-855-system-pages.md) | feature | New pages: this build, data folders, troubleshooting (W5) |
| [TICKET-856](open/TICKET-856-editor-pop-ups.md) | feature | The editor's pop-ups in the theme (W6) |
| [TICKET-857](open/TICKET-857-empty-and-error-states.md) | feature | Empty and error states (W6) |
| [TICKET-858](open/TICKET-858-motion-and-focus.md) | feature | Motion, attention and focus (W6) |
| [TICKET-794](open/TICKET-794-home-missing-after-a-restored-tab.md) | bug | Home goes missing when the Default group restores a tab on start (seen on the Mac with Onboarding) |
| [TICKET-795](open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md) | bug | The first command in a new zsh on a Mac has no block header |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
