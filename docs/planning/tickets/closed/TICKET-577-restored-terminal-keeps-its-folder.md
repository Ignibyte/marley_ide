# TICKET-577 — A restored center terminal keeps its saved folder

- **Ticket:** LOCAL #577 (bug, prong 1, after #575)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/577-restored-terminal-keeps-its-folder.spec.md
- **Source ticket:** found in TICKET-575's Test, 2026-09-26 (F-claude-575-the-terminal-panels-cleanup-deleted-the-center-terminals-rows-001)
- **Status:** closed

## Summary
The terminal panel restores its own terminals and then runs `TerminalView::cleanup` with the panel's items alone (`terminal_panel.rs`, "Since panels/docks are loaded outside from the workspace, we cleanup here"), which deletes every `terminals` row of the workspace that is not the panel's. In the Marley layout the terminals are the center's, and that cleanup can run before a center terminal's restore reads its row: the terminal then opens in the project's folder instead of the one it was in. #575 keeps the terminal ids apart from this through memory; the folder, Zed's own row, still meets it. The panel's cleanup should delete only the rows of the items it held before (its serialized layout's), so the center's rows stay for the center's restore and the workspace's own cleanup.

## Acceptance
A center terminal left in a subfolder at a quit comes back in that subfolder after a launch, in the Marley layout and in Zed's.
