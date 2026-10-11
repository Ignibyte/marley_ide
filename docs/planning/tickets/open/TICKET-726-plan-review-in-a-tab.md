# TICKET-726 — Plan review in a center tab

- **Ticket:** LOCAL #726 (feature; size large, in slices; the review batch after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [726-plan-review-in-a-tab.spec.md](../../pipeline/queued/726-plan-review-in-a-tab.spec.md) (slice 1)
- **Source ticket:** [plannotator-findings.md](../../intake/plannotator-findings.md), item 1; Chad, 2026-10-09: "take a look at https://github.com/backnotprop/plannotator and see what may be to add to the list"; Chad, 2026-10-11, on the [finish-line list](../../design-notes/finish-line-2026-10-11.md): "10.) yes"
- **Status:** open (deliberate: the review batch, after the finish)

## Summary
Claude Code's `ExitPlanMode` plan opens as rendered Markdown in a center tab beside the agent's
terminal. The user comments on selected passages and the whole plan, then answers Approve or Send
Feedback, and the answer reaches the waiting call; the plan also waits in Needs you. Off by default
(`marley.plan_review`). The call reaches Marley through the shared Claude Code plugin, so slice 1
needs one rustal-harness change: under Marley's host, the calls `MARLEY_APPROVE_TOOLS` names go to
`$MARLEY_BIN approve` with `rh approve`'s contract, and a failure leaves Claude Code's own dialog
in charge (the request is written out in the spec's notes).

Slices:
1. Claude Code's plan from a terminal: the switch, `marley-agent approve`, the tab, comments,
   Approve and Send Feedback, Needs you (this spec).
2. Delete and label marks, "Looks good", editing a note.
3. A plan in a Zed agent thread (Claude Code over ACP), whose answer carries no text: Send Feedback
   is "No, keep planning" and a post into the thread.
4. Codex's plan from the App Server; its "Implement this plan?" is the TUI's own popup, so the slice
   starts with how an approval reaches Codex.

## Acceptance
A plan from Claude Code opens in the tab; a comment and Send Feedback reach the agent as the
denial's reason, and Approve lets it go on.
