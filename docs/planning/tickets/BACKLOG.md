# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-622](open/TICKET-622-a-task-blocks-rerun-and-pill.md) | feature | terminal (wave 3) · a task block's Rerun and its pill |
| [TICKET-623](open/TICKET-623-a-failed-blocks-errors-as-diagnostics.md) | feature | terminal (wave 3) · a failed block's errors as project diagnostics |
| [TICKET-624](open/TICKET-624-a-prompt-editor-at-the-shells-prompt.md) | feature | terminal (wave 3) · a prompt editor at the shell's prompt, on a key |
| [TICKET-625](open/TICKET-625-completions-in-the-prompt-editor.md) | feature | terminal (wave 3) · completions in the prompt editor |
| [TICKET-626](open/TICKET-626-a-commands-colours-at-the-prompt.md) | feature | terminal (wave 3) · a command's colours at the prompt |
| [TICKET-627](open/TICKET-627-the-prompt-editor-by-default.md) | feature | terminal (wave 3) · the prompt editor by default, with the raw-passthrough ladder |
| [TICKET-573](open/TICKET-573-english-at-the-prompt-second-stage.md) | feature | terminal (wave 3) · English at the prompt, second stage: a System One reading for the lines the rules leave open, on the prompt editor (#627) |
| [TICKET-466](open/TICKET-466-fish-shell-integration.md) | feature | terminal (wave 3) · shell integration for fish (fish 4.9.2 installed 2026-09-30) |
| [TICKET-628](open/TICKET-628-blocks-with-native-headers-and-ps1-hidden.md) | feature | terminal (wave 3) · blocks with native headers, PS1 hidden |
| [TICKET-629](open/TICKET-629-block-navigation-in-display-rows.md) | feature | terminal (wave 3) · block navigation in display rows |
| [TICKET-630](open/TICKET-630-block-density-and-two-line-headers.md) | feature | terminal (wave 3) · block density: two-line headers and gaps |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-534](open/TICKET-534-harness-sessions-in-the-rail.md) | feature | prong 2 C1 · harness sessions in the rail, the read side; its blockers (#533, #508, the harness's M9) are done, and it is built in wave 4 of `design-notes/remaining-work-2026-09-30.md` |
| [TICKET-548](open/TICKET-548-system-one-via-cloudflare.md) | feature | prong 2 · Jev through Cloudflare Workers AI (stated zero retention) as a provider setting, per project; waits for the System One layer itself |
| [TICKET-540](open/TICKET-540-session-resume-after-restart.md) | feature | prong 2 · Claude Code sessions resumed after a restart; waits on how the embedded harness keeps processes alive, so it cannot start a second Claude on a live session |
| [TICKET-475](open/TICKET-475-shell-tests-scratch-data-dir.md) | chore | prong 1 T0 · the shell PTY tests install Marley's scripts in a scratch data directory; moot while no gate runs the tests (#483), and wanted again only if they run |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then) |
