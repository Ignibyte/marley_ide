# TICKET-190 — The "Files" panel is the tab navigator; the real file browser is unlabeled

- **Forge ticket:** #190 (7f2d0ce2-60eb-4a97-a83d-a1f922c6660f) (bug, M12.1)
- **Owner:** claude (session c104f25c)
- **AAR:** fe8cb735-d10e-46d7-bcb7-52c3191549c4
- **Pipeline doc:** ../../pipeline/active/files-panel-label.spec.md
- **Source ticket:** M12.1 sprint #24 — chad live-app feedback #3
- **Status:** closed

## Summary
chad reported he "can't click a file in the file browser to view it." Live repro shows the file-click path
actually works — the real bug is a stale label: the left dock is titled "Files" (`dock_title(Left)`) but since
#152/#154 it renders the Workspace→Tab navigator, while the actual file browser (`files_panel`) is unlabeled and
hidden by default. Fix: rename the left dock "Files"→"Workspace" and give `files_panel` a "Files" header, so the
panel named "Files" IS the file browser. Direction chosen autonomously (chad away) — flag for his review.

## Acceptance
The panel labeled "Files" is the file browser (with a "Files" header on `files_panel`); the left tab dock reads
"Workspace"; clicking a file in the browser opens its code view. Full EARS in the pipeline spec (REQ-001..003).
