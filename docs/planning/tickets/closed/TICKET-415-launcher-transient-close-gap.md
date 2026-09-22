# TICKET-415 — close_transient_overlays omits the agent launcher

- **Ticket:** LOCAL #415 (bug, M20)
- **Tags:** overlay, keys, 318-found
- **Created:** 2026-08-12
- **Status:** closed

## Summary

Found writing #318's draw-smoke: `close_transient_overlays` clears palette/finder/history/find +
eleven Some-typed overlays but NOT `agent_launcher` (fleet is toggle-scoped by design — decide
whether that stays). Consequence: opening another overlay over a lingering launcher stacks them
(fleet drew over the launcher in the smoke), against the
PR-claude-mouse-opened-modal-must-close-all-transient-overlays-001 family. Decide membership
deliberately (launcher in the set; fleet's exemption recorded), add the launcher's dismissal, and
extend the #318 smoke's strict five-flag equality once closes are uniform.

## Acceptance

Headline: after any transient-close, no launcher survives; the smoke asserts strict five-flag
equality per iteration. Full EARS at plan.
