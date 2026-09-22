# TICKET-295 — Diagnostics gutter aggregates across ALL terminal grids

- **Forge ticket:** #295 (8b6a333d-83b0-4d07-8164-fa01c5bd5c38) (bug, M18 origin → ships M22)
- **Owner:** 466e35ad-09f6-4b81-89e7-b7fd16c1e45d
- **AAR:** e78f23f9-f8b1-461e-83d7-9b6871fdb3da
- **Pipeline doc:** ../../pipeline/active/295-cross-tab-diagnostics.spec.md
- **Source ticket:** M22 IDE wrap-up train (#306, #295, #356, #357, #355)
- **Status:** closed

## Summary
`open_file_diagnostic_rows` iterates `self.workspace().states()` — the active-or-first terminal grid only — so
with the editor tab active, a failed command block in any OTHER terminal tab never lights the diagnostics
gutter. Fix: iterate every terminal grid in the active project via a new pure `Project::terminal_grids()`,
keeping the per-grid failed-block body + the `merged_rows` union unchanged.

## Acceptance
`open_file_diagnostic_rows` consults the last failed block of every terminal grid in the active project (not
just the active-or-first). Full EARS in the pipeline spec.
