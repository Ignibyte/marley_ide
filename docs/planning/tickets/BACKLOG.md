# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-495](open/TICKET-495-select-picker.md) | feature | prong 3 B1d · Marley draws the page's `<select>` lists |
| [TICKET-486](open/TICKET-486-keep-the-terminal-size-across-launches.md) | bug | prong 1 T0 · the first terminals of a launch open at the last session's size (#485's limit) |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-475](open/TICKET-475-shell-tests-scratch-data-dir.md) | chore | prong 1 T0 · the shell PTY tests install Marley's scripts in a scratch data directory; moot while no gate runs the tests (#483), and wanted again only if they run |
| [TICKET-466](open/TICKET-466-fish-shell-integration.md) | feature | prong 1 T0c · shell integration for fish; waits for fish on a machine that can run its tests (the dev box has none) |
| [TICKET-446](open/TICKET-446-marley-crate-license-files.md) | chore | licensing: `LICENSE-APACHE` and `LICENSE-MIT` in every Marley crate; waits for Chad's copyright line for the MIT text |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then) |
| [TICKET-417](open/TICKET-417-318-after-capture-battery.md) | chore | gpui-era, macOS harness; obsolete in the Zed fork (the overlays it captured no longer exist) — close or re-scope when the block terminal's overlays land |
| [TICKET-271](open/TICKET-271-headless-pixel-capture-gpui.md) | chore | gpui-era; the fork builds gpui from the tree, so the wait is over — re-scope as "headless pixel captures for validate" when a UI slice needs it |
