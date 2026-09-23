# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/pipeline:plan` with no argument
takes the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-451](open/TICKET-451-marley-layout-fixes.md) | feature | W6b · Marley layout fixes: Zed's layout presets, the right dock across a switch, a first terminal |
| [TICKET-452](open/TICKET-452-rail-rename-and-close.md) | feature | W6c · rename and close terminals from the rail |
| [TICKET-453](open/TICKET-453-rail-keyboard-and-filter.md) | feature | W6d · keyboard navigation, a filter and project reorder in the rail |
| [TICKET-454](open/TICKET-454-rail-switcher.md) | feature | W6e · a switcher over recent terminals and threads |
| [TICKET-448](open/TICKET-448-zed-dylint-lints.md) | chore | quality gates · Zed's dylint lints (`tooling/lints`) on the Marley crates, as gate:21 |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-446](open/TICKET-446-marley-crate-license-files.md) | chore | licensing: `LICENSE-APACHE` and `LICENSE-MIT` in every Marley crate; waits for Chad's copyright line for the MIT text |
| [TICKET-445](open/TICKET-445-marley-release-identity.md) | chore | packaging: Marley's own keyring label, updater, app id and URL scheme; waits until Marley ships a package or needs a non-`dev` build (the `dev` channel keeps it safe until then) |
| [TICKET-417](open/TICKET-417-318-after-capture-battery.md) | chore | gpui-era, macOS harness; obsolete in the Zed fork (the overlays it captured no longer exist) — close or re-scope when the block terminal's overlays land |
| [TICKET-271](open/TICKET-271-headless-pixel-capture-gpui.md) | chore | gpui-era; the fork builds gpui from the tree, so the wait is over — re-scope as "headless pixel captures for validate" when a UI slice needs it |
