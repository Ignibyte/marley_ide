# TICKET-523 — Saved Playwright scripts, run on a Browser tab

- **Ticket:** LOCAL #523 (feature, prong 3)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/523-saved-playwright-scripts.spec.md
- **Source ticket:** Chad, 2026-09-25: "playwright that can run scripts inside marley itself. that someone can keep saved inside marley." Decided the same day as its twin, Marley's browser exposed to trusted outside clients (#524). The plan already has Playwright attach to the endpoint in Marley's `DevToolsActivePort` (`docs/marley/three-prong-plan.md`, D16 and open decision 4); report 03 §2.15 found no scripts or recorder in Orca to take.
- **Status:** closed

## Summary
Chad repeats browser chores by hand: signing in to a local site, filling a form to reach a state, checking a page after a change. Marley keeps a library of Playwright scripts, some for one project and some for every project, in its own config folder, and runs one on the Browser tab the user picks: Playwright attaches to that tab's Chromium over its DevTools endpoint and drives the page while the user watches it in the tab. The run's output is a block in a terminal beside the tab, and when a script fails, Marley saves the tab's last minute as a flight-recorder recording, so the failure comes with what the page did.

## Acceptance
The Browser tab's Scripts tray lists the project's and the global scripts and makes new ones; Run drives the tab's page in front of the user with the output in a terminal block; the run's environment carries the endpoint and the tab's target id in `MARLEY_CDP_URL` and `MARLEY_TAB`; a failed run saves a recording that names the script.
