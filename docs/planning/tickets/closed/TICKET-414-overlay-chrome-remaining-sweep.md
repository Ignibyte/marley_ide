# TICKET-414 — Overlay chrome: convert the remaining 10 sites (+ the non-modal variant)

- **Ticket:** LOCAL #414 (chore, M20)
- **Tags:** overlay, refactor, 318-followup
- **Created:** 2026-08-12
- **Status:** closed

## Summary

#318 extracted `overlay_card_chrome` + `quarter_overlay_card` and converted 7 of the 17 verbatim
chrome sites. Convert the remaining 10: completion popup (15080-era), references, code_action,
file_symbols, symbols, search, problems, naming_workflow, naming_pane, fleet_dispatch_draft.
Two design obligations recorded at #318 inspect: (1) the chrome is MODAL (`.occlude()`) — the
completion popup (and hover, a shipped tension) sit over scrollable content and need a
`block_mouse_except_scroll` VARIANT per PR-claude-block-mouse-except-scroll-for-nonmodal-scroll-
overlays-001, not the modal fn; (2) the naming trio's 0.3/h4/0.4 geometry is its own recipe —
extract or leave deliberately, don't force-fit quarter geometry. Pixel-parity capture budget per
site, same protocol as #318 (before-set first, live session required —
L-claude-318-driving-the-live-app-needs-an-active-user-session-001).

## Acceptance

Headline: zero verbatim copies of the 8-call chain outside the helpers; every converted site
pixel-identical; the non-modal variant decision recorded and applied. Full EARS at plan.
