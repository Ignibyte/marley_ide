# TICKET-506 — A recording turned into a Playwright test

- **Ticket:** LOCAL #506 (feature, prong 3, after B5)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/506-recording-to-playwright-test.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 3 of the list after the browser waves)
- **Status:** open

## Summary
The flight recorder keeps clicks and typing as page coordinates, which no test can replay. Marley records each click and typing target as a locator as well, and a tool drafts a Playwright test from a recording, so a bug Chad reproduced becomes a test the agent can run.

## Acceptance
A recording's clicks and typing carry locators; the draft-test tool returns a Playwright test that replays the recording.
