# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-647](open/TICKET-647-brain-graph-tab.md) | feature | Rusty in Marley R5 · the Graph tab: whole or local, filters, decision edges, Ely's force layout off the main thread |
| [TICKET-648](open/TICKET-648-agent-version-check.md) | feature | B7 · the tested-version table for `claude` and `codex`: an untested integration stays off and the agent bar says why; `marley.allow_untested_versions` |
| [TICKET-649](open/TICKET-649-rich-input-through-the-agents-editor-key.md) | feature | B4 · rich input through the agent's own editor key: `marley-edit` as the agents' `VISUAL`/`EDITOR` opens the prompt in a Marley tab (`marley.agent_editor_in_tab`, off by default) |
| [TICKET-650](open/TICKET-650-codex-app-server-state.md) | feature | B1 part 1 · Marley runs a Codex App Server per Codex terminal and reads typed state, token use and sandbox from it (`marley.codex_app_server`, off by default) |
| [TICKET-651](open/TICKET-651-codex-approvals-and-prompts.md) | feature | B1 part 2 · Codex's approval requests answered from the inbox, only the request the user saw |
| [TICKET-652](open/TICKET-652-shared-claude-plugin-marleys-half.md) | feature | B2 · agent reports reach Marley through `$MARLEY_BIN report`; the rail and resume read them first (Marley's answer to rustal-harness MREQ-009) |
| [TICKET-653](open/TICKET-653-marley-as-claude-codes-ide.md) | feature | B3 · Marley as Claude Code's IDE: the lock file and link, diagnostics, selection and open file (`marley.claude_code_ide`, off by default) |
| [TICKET-654](open/TICKET-654-rusty-open-page-picker.md) | feature | Rusty in Marley R3a · `rusty: open page` (Ctrl+Alt+U): a picker over the vault's titles, create on a miss |
| [TICKET-655](open/TICKET-655-knowledge-panel-project-view.md) | feature | Rusty in Marley R6 · the Knowledge panel's project view: the project's brain page, follow-ups and task group, linked when none matches |
| [TICKET-656](open/TICKET-656-brain-page-outline-and-property-edits.md) | feature | Rusty in Marley R2b · the Page tab's outline, and the title, name and properties edited in place |
| [TICKET-657](open/TICKET-657-brain-graph-groups-and-forces.md) | feature | Rusty in Marley R5b · the Graph tab's colour groups, sliders and arrows, restored after a restart |
| [TICKET-658](open/TICKET-658-rusty-tasks-tab.md) | feature | Rusty in Marley R7 · the Tasks tab over Rusty's task tools |
| [TICKET-659](open/TICKET-659-rusty-decisions-tab.md) | feature | Rusty in Marley R7 · Rusty's Decisions tab; Marley's System One tab renamed "System One calls" |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-548](open/TICKET-548-system-one-via-cloudflare.md) | feature | prong 2 · Jev through Cloudflare Workers AI (stated zero retention) as a provider setting, per project; waits for the System One layer itself |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then). Chad, 2026-10-01: wait, so bugs found before the first release do not mean cutting several |
