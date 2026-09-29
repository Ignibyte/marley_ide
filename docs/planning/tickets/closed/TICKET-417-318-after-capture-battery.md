# TICKET-417 — Run the #318 after-capture parity battery (needs a live session)

- **Ticket:** LOCAL #417 (chore, M20)
- **Tags:** overlay, selftest, 318-followup, deliberate
- **Created:** 2026-08-12
- **Status:** closed

## Summary

#318's REQ-002 after-captures were §7-blocked: the headless mini had no active user session, and
activation is unobtainable without one (the full diagnosis + substitute evidence are in the #318
notes, Phase 4). The BEFORE set is banked at `scratchpad/318/before/` (5 overlays, pixel-verified,
window 2384×1119) — NOTE: scratchpad is session-scoped; if it has been reaped, re-take a before set
from a pre-318 checkout first. In the next LIVE session (Chad at the desk, or an active CRD
session): rebuild, run the #318 capture protocol (activate → clickat-focus in the editor area →
chord → capture → probe; palette ⌘⇧P, finder ⌘P, history ⌘R, launcher ⌘⇧A, fleet ⌘⇧E; close-verify
between), and compare the sampled card-interior/border points per overlay against the before set.
Any delta = a #318 regression to fix.

## Acceptance

Headline: five before/after pairs compared by pixel sampling, zero card-rect deltas. EARS at plan.

## Resolution
Closed on 2026-09-28 by Chad's decision ("close both"). The five overlays it would have compared belonged to the gpui-era app, which the Zed fork replaced; none of them exists in Marley, so there is nothing left to capture.
