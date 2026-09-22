# TICKET-259 — Editable split-file pane

- **Forge ticket:** #259 a8bcfc50-90ad-4afe-a255-76c3edb0bacc (feature, M15)
- **Owner:** 99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c
- **AAR:** 8d17f4a9-f400-4bae-8f5a-6b4f02f64f58
- **Pipeline doc:** ../../pipeline/active/259-editable-split-pane.spec.md
- **Source ticket:** M15 — the #258(a) fast-follow (the split pane persisted read-only at #258(b))
- **Status:** closed

## Summary
The #246/#258 split-right file pane shows a file beside your terminal — read-only. This makes a FOCUSED
split pane a real editing surface (type/save/caret/selection/clipboard/motion/undo), the first time "the
editor" stops being a singleton. The batch's one architectural ticket: it breaks four shipped single-slot
assumptions (the pane has no Buffer; one app-wide FocusHandle; per-tab KeyContext; one `editor_geom` Cell +
one text-input registration) and rewires the editor accessor to serve the FOCUSED editable surface.

**Phase-1 recon corrected the spec's premise:** the blast radius is ~52 editing feature-sites / ~95 raw
call sites (the "16+" was stale — #298-317's multi-cursor/fold/symbol/LSP work grew it). The choke point is
two fns (`active_editor()`/`active_editor_mut()`), and `PaneGrid.focused` + `focused_terminal()/_mut()`
already model focus-aware access. Scoped to a 2-slice split — this ticket is **SLICE 1 (core editable
pane)**; slice 2 (editor-feature parity: find/fold/symbol/LSP-in-pane) is a named follow-up.

## Acceptance
⇧-focused split pane edits like the editor tab (type/save/caret/selection/clipboard/motion/undo); the
terminal pane's keys stay byte-identical while an editable pane is unfocused; the same file in tab + pane =
two independent Buffers with the #275 conflict banner as the no-silent-clobber net. Full EARS in the spec.
