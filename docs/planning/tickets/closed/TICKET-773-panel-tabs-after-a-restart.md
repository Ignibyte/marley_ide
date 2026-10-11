# TICKET-773 — Panel tabs after a restart

- **Ticket:** LOCAL #773 (feature, the Marley layout: tabs everywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [773's spec](../../pipeline/completed/773-panel-tabs-after-a-restart.spec.md); the design is #747's D9
- **Source ticket:** split from #747 at its promotion (2026-10-11), to keep #747 one shippable slice.
  Plan: [tabs-everywhere shelf](../../design-notes/tabs-everywhere-2026-10-11.md).
- **Status:** closed

## Summary
#747's panel tab is a center item, but not yet a serializable one, so a restart leaves no Files,
Git or Outline tab. The item becomes serializable: its row (workspace id, item id, the panel's
persistent name) sits in a small table of its own, as the Graph tab's do, and the restore waits
for the workspace's panel of that name, as the thread tab waits for the Agent Panel. It refuses in
Zed's layout, for a name outside the moved list, and for a panel that never arrives, and the
cleanup drops the row.

## Acceptance
A Files tab and a Git tab open at the quit come back in their panes after a restart.
