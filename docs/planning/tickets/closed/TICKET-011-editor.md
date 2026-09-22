---
ticket: TICKET-011
forge: forge#15 (cdfc58bf-88f3-4262-b6a7-eee6f5f6a2f3)
status: closed
type: feature
milestone: M1
sprint: M1.A — The Usable Terminal (seq 4/5)
branch: ticket-011-editor
pipeline: docs/planning/pipeline/active/editor-input.spec.md
spec: docs/specs/SPEC-editor.spec.md
aar: b8c1f784-05e1-44af-97e3-3e1018b9c0d8
---

# TICKET-011 — marley_editor (M1.A input subset)

Sprint M1.A seq 4/5. The **M1.A subset** of SPEC-editor — the rope-backed **input-prompt buffer**: a
`ropey`-backed `Buffer` + the `CharOffset`/`ByteOffset` seam + range `edit` carrying `EditOrigin`
(Human vs Agent) + single cursor/selection + movement (char/word/line) + submit-on-Enter + `point_at`.
**UI-agnostic** (visual_acceptance N/A; the render is `app_shell` #16). The full editor (anchors,
multi-cursor, grouped undo/redo, find, diff, layout) grows in M1.B.

## Acceptance
- The pure surface (buffer ops over CharOffset/ByteOffset, edit+delta+EditOrigin, selection clamp,
  movement, point_at) — **cov 100 / MSI 100** (the offset arithmetic is the disjoint-index mutation trap;
  non-origin/multibyte fixtures).
- Deps `ropey` (MIT) + `marley_text_offsets`; NO gpui/ui_components/imara-diff/regex-automata.
- FULL `scripts/gates.sh` → `GATE GREEN` (gate-15 N/A); §21 CHANGELOG + arch doc.

## Notes
The EditOrigin (human-vs-programmatic) write-provenance seam is load-bearing for later agent injection +
the brain (AD-claude-brain-agent-session-supervision-001) — designed in (observable) even though its
undo-grouping USE is deferred to M1.B. See the pipeline spec/notes for the full M1.A cut + deferrals.
