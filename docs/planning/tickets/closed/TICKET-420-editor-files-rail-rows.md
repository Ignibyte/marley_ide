# TICKET-420 — Open editor files as per-file rail rows

- **Ticket:** LOCAL #420 (feature, M31)
- **Tags:** rail, editor, simple-rail
- **Created:** 2026-08-12
- **Status:** closed (2026-08-12 — shipped: RailLevel::File projection + close path; 2134 tests green; driven live; GATE GREEN [diff] 15/15)

## Summary

Opened files no longer appear under the project — deliberately, since #237/#240 (2026-07-10):
all files fold into ONE editor surface whose single rail row is frozen at "Editor"
(tabs.rs:557-570, :363, :1212; pinned tests tabs.rs:1464-1505). Chad wants them back. This
ticket changes the rail PROJECTION only: the Editor section lists each open file as its own
row (label from the surface's file path, disambiguated — spec sweep finding: ContentLabels is
terminal-only by design; click focuses that file in the editor surface, × closes it), and the
frozen "Editor" row retires. The one-surface editor model from #237 is
NOT reverted; the per-file strip inside the editor stays. Un-pin/replace the #240 tests.

## Acceptance

Headline: with N files open, the Editor section shows N per-file rows (no "Editor" row);
clicking a row focuses that file; × closes it; zero files → the section's empty state matches
#418's design. Full EARS in the queued spec
(`docs/planning/pipeline/queued/420-editor-files-rail-rows.spec.md`).
