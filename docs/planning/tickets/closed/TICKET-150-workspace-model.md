# TICKET-150 — M9 seq-1: pure Workspace/Project/Tab model + algebra

- **Forge ticket:** #150 `ee02dfec-81e3-4549-83d3-abba18cf2cc1` (feature, M9; sprint #20)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `72c5fd9d-4940-458b-8c9d-e474a5ed0b9c`
- **Pipeline doc:** ../../pipeline/active/workspace-model.spec.md
- **Status:** closed

## Summary
The M9 foundation: a pure `Workspace → Project → Tab` hierarchy (`tabs.rs`) with a fully-tested add/close/switch
algebra + never-empty / active-follows-close / last-item invariants (mirroring M6 R28–R30), generic over the
session handle `S`. A Terminal tab wraps the existing pane grid, renamed `Workspace<S>` → `PaneGrid<S>`
(mechanical, behavior unchanged). No render (seq-2). Decisions: multi-project container; active tab drives
files/cwd/branch.

## Acceptance
The algebra at cov/MSI 100; the full existing suite green after the rename; FULL gate GREEN. Full EARS in spec.
