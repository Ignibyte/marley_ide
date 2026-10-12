# TICKET-727 — Plan versions and a diff between them

- **Ticket:** LOCAL #727 (feature; size small, after #726 slice 1; the review batch after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [727-plan-versions-and-diff.spec.md](../../pipeline/completed/727-plan-versions-and-diff.spec.md)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 2; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"; Chad, 2026-10-11, on the [finish-line list](../../design-notes/finish-line-2026-10-11.md): "10.) yes"
- **Status:** closed

## Summary
The plan tab keeps every plan its session sends. From the second on, the header shows "Version N"
and a `+A −R` count against the version before, and Changes shows Zed's read-only diff of the two;
Plan returns to the drawn Markdown. Versions live as long as the tab.

## Acceptance
A second plan shows its version and count, and Changes shows the diff from the first.
