# TICKET-416 — marley-web POC overlay drifts: wrap→clamp, maxHeight→content-driven

- **Ticket:** LOCAL #416 (chore, M20)
- **Tags:** parity, marley-web, 318-followup
- **Created:** 2026-08-12
- **Status:** closed

## Summary

Two POC drifts found at #318 (frozen-shell behavior is Marley-authoritative per MARLEY-PARITY.md):
(1) `OverlayShell.tsx`'s `useOverlaySelection` WRAPS while Marley's palette/finder/history CLAMP
(#318 D4 decided the split stays, structurally coupled to row windowing); (2) its
`maxHeight: 83.3333%` caps the card while Marley's height is content-driven with deliberate
bottom overflow (the component's own doc comment says "allowed to overflow the bottom edge" —
the CSS contradicts it). Fix both in marley-web; captures re-verified against Marley.

## Acceptance

Headline: the POC's shared overlay selection clamps; no maxHeight cap; parity captures match.
Full EARS at plan.
