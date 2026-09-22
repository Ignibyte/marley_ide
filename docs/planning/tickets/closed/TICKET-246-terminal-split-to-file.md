# TICKET-246 — Terminal pane can split-right to a read-only file view (not a new PTY)

- **Forge ticket:** #246 (ac5df912-b46a-4dd8-b2af-1f23c2fac7ab) (feature, M14 sprint #27)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 78f856d1-3ec4-4f05-b9db-b866b9b1f0b4
- **Pipeline doc:** ../../pipeline/completed/terminal-split-to-file.spec.md
- **Source:** the M14 round; the LAST in-range ticket. chad's "terminal + split-right-to-a-file" (#237 deferred).
- **Status:** closed

## Summary
Today `split_focused` always spawns a PTY, so you can't put a file beside a terminal. The pane model is already
content-agnostic (`PaneContent::CodeView(CodeViewState)` exists; `open_pane` is the shipped no-PTY split;
`code_view_body` renders it; resize/focus/close are PaneId-keyed). v1 revives the CodeView pane render arm, adds
a `split_file_pane` (the `open_file_in_viewer` guard ladder → `open_pane(CodeView)`), a trigger (the ⌘P finder in
a split mode), and a pure `pane_display_name` so the pane titles by filename. READ-ONLY (editable rides on #242);
NON-PERSISTED (the file pane drops on restart, matching existing FileTree/Git — a follow-up).

## Acceptance
Invoking "Split Right → File" splits the focused terminal and renders a chosen file read-only beside it (no PTY);
the pane resizes/closes like any pane and titles by filename; persist+restore with a CodeView pane doesn't crash.
Pure `pane_display_name` cov/MSI 100; driven (or env-blocked → units+mechanism). Full EARS in the pipeline spec.
