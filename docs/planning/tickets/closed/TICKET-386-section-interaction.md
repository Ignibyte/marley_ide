# TICKET-386 — Section interaction: active highlight, collapse-persist, footer vocabulary

- **Forge ticket:** #386 `8ac2f904-6043-4c1b-8b63-e101a6bd3084` (feature, M26)
- **Owner:** unclaimed (queued)
- **AAR:** — (opened at promotion)
- **Pipeline doc:** ../../pipeline/queued/386-section-interaction.spec.md
- **Source:** chad's 2026-07-22 sectioned-shell direction; depends on #385
- **Status:** closed

## Summary
The #385 sections become interactive: the section holding the active tab carries the accent-wash
active highlight (the workspace-centric rail-highlight at section level); a section-header click
collapses/expands its rows, and the collapsed set persists across relaunch (the existing
project-collapse + settings round-trip idioms); a tab in a collapsed section becoming active
auto-expands it (never an invisible active tab). Plus a small audit: reconcile the #382
footer-focus vocabulary with the section names where they diverge (cockpit vs Browser) — smallest
honest change, possibly a documented no-op.

## Acceptance
Active section visibly highlighted (others quiet); header click toggles collapse; collapsed set
survives relaunch; activating into a collapsed section auto-expands; footer audit decided +
recorded. Full EARS in the pipeline spec.
