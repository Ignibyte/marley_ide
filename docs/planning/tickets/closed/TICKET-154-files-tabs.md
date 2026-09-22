# TICKET-154 — M9 seq-5: Files as a left panel + open-file-as-a-tab

- **Forge ticket:** #154 `2953a9bf-11fe-495a-a7d1-f022ae2d1e08` (feature, M9; sprint #20)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `89d2ce11-b0ea-4e69-9788-fbe76889f7a0`
- **Pipeline doc:** ../../pipeline/active/files-tabs.spec.md
- **Status:** closed

## Summary
The file tree expands from the LEFT (toggled by 📁, scoped to the active project); opening a file opens a
full-screen CodeView TAB — never a mid-screen split. Pure: TabContent::CodeView + code_view accessors +
open_or_switch_code (cov/MSI 100). Shim: move the code-viewer + file-tree render blocks out of the tiled pane
`match kind` into a code_view_body (center full-screen) + a files_panel (left); retire the tiled panes. Deps
#150-153.

## Acceptance
The pure helpers at cov/MSI 100; driven captures — Files as a left panel, a file opens a full-screen code tab,
terminal returns; FULL gate GREEN. Full EARS in the spec.
