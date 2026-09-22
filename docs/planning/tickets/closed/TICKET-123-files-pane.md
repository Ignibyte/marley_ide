# TICKET-123 — the file explorer as a real pane [M6 seq-4]

- **Forge ticket:** #123 `e79c0d80-4794-48cf-bcb5-d9675a74d8b4` (feature, M6 seq-4; sprint #17)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `eb4f9bc5-5703-4c83-8594-f4834c2ce95d`
- **Pipeline doc:** ../../pipeline/active/files-pane.spec.md
- **Status:** closed

## Summary
Retire the duplicate left-dock "FILES" tree (sidebar → sessions-only); move the folder-toggle + file-click
into the #121 FileTree pane, so the explorer lives in one place. Shim-only cleanup (clears duplication #1).

## Acceptance
Sidebar sessions-only + the interactive Files pane (live capture); FULL gate GREEN. Full EARS in the spec.
