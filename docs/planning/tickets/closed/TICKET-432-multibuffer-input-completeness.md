# TICKET-432 — Multibuffer input completeness (click-column · paste · IME)

- **Ticket:** LOCAL #432 (feature, M33)
- **Tags:** editor, multibuffer, input, ime
- **Created:** 2026-08-15
- **Provenance:** the #428 recorded v1 seams (editor.md deferred list);
  shelf: ../../design-notes/m33-tail-and-wedge-shelf.md
- **Pipeline doc:** ../../pipeline/completed/432-multibuffer-input-completeness.spec.md
- **Status:** closed (2026-08-15 — the three #428 seams shipped + the fail-closed ladder; GATE GREEN [diff] 15/15)

## Summary
The editable multibuffer's three recorded v1 gaps close: a click places the
caret AT the clicked COLUMN (the measured mono advance — today it parks at
line end), ⌘V pastes through the ONE insert mechanism (single-line v1;
multi-line grows the window like Enter), and IME composition (dead keys,
marked text) rides the surface's own `handle_input` canvas end-to-end (the
#267 path, scoped per-surface at #428). All three are small arms on shipped
infra — no new machinery.

## Acceptance
Headline: click a mid-line char in an excerpt → the caret lands on that
column; ⌘V inserts the clipboard at the mb caret with write-through + one
undo group; a dead-key compose (⌥E e → é) lands in the target buffer. Full
EARS in the queued spec.
