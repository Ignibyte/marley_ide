# TICKET-451 — Marley layout fixes: Zed's layout presets, the right dock and a first terminal

- **Ticket:** LOCAL #451 (feature, workbench shell W6b)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (split from #442 at its promotion)
- **Source ticket:** ../../pipeline/completed/442-rail-persistence.spec.md (Out) · ../../../marley/workbench-shell.md
- **Status:** open

## Summary
Three faults of the Marley layout that #438 and #441 carried forward.
- **Zed's Panel Layout presets** misread the Marley layout. With `agent.dock` patched to the
  right, the title bar's menu shows "Custom"; Classic then writes the other panels to the left
  but not `agent.dock`, and Agentic writes `agent.dock: left`, which overrides the Marley layout
  for good (`agent_settings.rs:92-111`, `:338-396`; `title_bar.rs:1289-1292`). The title bar
  re-shows its entries on every settings change, so a Marley-side hide would be undone: catch
  `UseClassicLayout` and `UseAgenticLayout` in the capture phase while the layout is `marley`,
  as #441 catches the terminal actions, with a message instead of the write.
- **The right dock across a layout round trip.** With the Agent Panel open, a switch to the Zed
  layout and back can close the right dock: the panel becomes the right dock's active panel,
  and moving it back closes the dock (`dock.rs:638-700`, `:895-918`). Record each dock's
  visibility across the swap and restore it (#438 inspect S9).
- **A first terminal.** A project first shown in the Marley layout with no center terminal gets
  one at its root. A restored workspace adds its terminals after the rail's first read, which
  would double them: seed only once the workspace's items are restored, or only for a project
  opened fresh.

## Acceptance
In the Marley layout, choosing Classic or Agentic from the title bar changes no setting and says
why; a layout round trip leaves each dock open or closed as it was; a project opened with no
terminal shows one at its root, and a restored project gains no extra terminal. The EARS
criteria come at promotion.
