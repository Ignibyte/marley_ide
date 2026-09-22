# Backlog — the ordered queue

The local ticket-next (TICKET-409 pivot, 2026-08-09). `/work` with no argument takes
the **top row of the Queue**; rows leave this file when their pipeline completes.
**Deliberate** rows are never auto-next — they are picked explicitly (idle-machine,
upstream-gated, or hardware-gated work). New tickets get a row when minted (`/spec`
or `/pipeline:plan`); keep each section ordered by intent, not number.

## Queue (top = next)

| Ticket | Type | One line |
|---|---|---|
| [TICKET-437](open/TICKET-437-marley-app-identity.md) | chore | W1 · Marley's own app identity: `APP_NAME`, the binary, Chad's settings copied once |
| [TICKET-438](open/TICKET-438-marley-layout-and-rail.md) | feature | W2 · the Marley layout switch and the first rail: projects, center terminals, New Terminal |
| [TICKET-439](open/TICKET-439-rail-zed-threads.md) | feature | W3 · Zed agent threads in the rail |
| [TICKET-440](open/TICKET-440-rail-agent-clis.md) | feature | W4 · agent CLIs in rail terminals |
| [TICKET-441](open/TICKET-441-marley-terminal-routing.md) | feature | W5 · terminal routing and keys in the Marley layout |
| [TICKET-442](open/TICKET-442-rail-polish.md) | feature | W6 · rail persistence and polish |
| [TICKET-444](open/TICKET-444-ported-discarded-results.md) | chore | port hygiene · handle the eight results the ported Marley crates discard with `let _ =` |

## Deliberate (picked explicitly, never auto-next)

| Ticket | Type | Why it waits |
|---|---|---|
| [TICKET-417](open/TICKET-417-318-after-capture-battery.md) | chore | gpui-era, macOS harness; obsolete in the Zed fork (the overlays it captured no longer exist) — close or re-scope when the block terminal's overlays land |
| [TICKET-271](open/TICKET-271-headless-pixel-capture-gpui.md) | chore | gpui-era; the fork builds gpui from the tree, so the wait is over — re-scope as "headless pixel captures for validate" when a UI slice needs it |
