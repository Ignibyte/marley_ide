# TICKET-275 — External file-change detection (reload clean, flag dirty)

- **Forge:** #275 `f1b13464-6b27-4ce5-82fd-016a1b0c739c` (sprint #30, M17)
- **Type:** feature
- **Status:** closed
- **Pipeline:** `docs/planning/pipeline/active/275-external-file-change.spec.md` (8d65d10b-fcd2-45ea-8b96-592807f61de0)

## Summary
The agent workflow gap: an agent rewrites a file in the terminal pane
next to the editor and the open buffer never notices. Per-OpenFile
(mtime, len) snapshots at load/save; poll-at-interaction checks at
three choke points (window-focus-regained render edge, file-tab
activate, pre-⌘S) — no fs-watcher thread in v1. Pure decision table
(snapshot, disk, dirty) → Noop | CleanReload | Conflict | Deleted.
Clean → silent reload (clamped caret/scroll, nonce re-mint, flash).
Dirty → non-modal #221-card banner (Keep mine re-snapshots · Reload
discards). ⌘S under conflict arms on the first press, overwrites on
the second; deleted files keep the buffer and ⌘S recreates.

## Acceptance
See the spec's EARS table (REQ-001..005): the full decision table,
the silent clean reload, the banner flows, the double-⌘S, and the
no-watcher/no-per-frame-stat negative.
