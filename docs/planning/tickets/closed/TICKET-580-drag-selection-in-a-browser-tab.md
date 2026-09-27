# TICKET-580 — A drag in a Browser tab seemed to leave no selection

- **Ticket:** LOCAL #580 (bug, prong 3, after #489)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/580-drag-selection-in-a-browser-tab.spec.md
- **Source ticket:** found in TICKET-518's Test, 2026-09-26
- **Status:** closed (not a Marley bug)

## Summary
#518's Test reported that a drag across a paragraph in a Browser tab left nothing selected once
the button was up, and its scenario selected with a double click instead. The page's full event
log shows otherwise. The drags ran past the end of the text on #518's `select.html`, whose
paragraph and list are placed absolutely over a body with no height, and the move past the line's
end collapsed the selection in Chromium itself, before any release. A drag that ends inside the
text keeps its selection after the release, and so does a drag past the end of a paragraph in
normal flow. Marley's input path needs no change.

## Acceptance
WHEN the user drags across a Browser tab's text and lets go inside it, the page shall keep the
selection the drag made, as `browser_look` and a pick read it. #518's scenario proves it with a
drag again.

## Resolution
Not a Marley bug. #518's scenario drags again, the known limit is withdrawn, and the wrong
failure and lesson are replaced by
L-claude-580-a-drag-past-an-absolutely-placed-lines-end-collapses-the-selection-001.
