# TICKET-506 — A recording turned into a Playwright test

- **Ticket:** LOCAL #506 (feature, prong 3, after B5 and #505)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/506-recording-to-playwright-test.spec.md
- **Source ticket:** Chad, 2026-09-25: "lets do 1 through 8" (item 3 of the list after the browser waves), and his direction the same day, as relayed with the night's drafting brief: the app being built keeps its Playwright regressions in its own code, runnable in Marley or outside it, so the draft test is a file in the project, not Marley's. The Orca survey's rules for #506 are folded in (`docs/orca_architecture/README.md`, "What it changes in the queued sprint"; report 03 item 9)
- **Status:** closed

## Summary
The flight recorder keeps clicks as page coordinates and typing as counts, which no test can
replay. Marley now records, in the page, the element each click, fill and key press landed on,
as locators computed at that moment (a test id, then role and name, label, placeholder, text, then
a CSS path, generated names left out), and keeps what was typed into ordinary fields; password
and other secret fields still keep only a count. A tool, `browser_draft_test`, drafts a Playwright
test from a saved recording: `getByTestId` first and the other locators in that order, secret
fields read from `process.env`, `toHaveURL` after each navigation, the recording's origin as
`baseURL`. The draft belongs to the project: the tool suggests a path from the project's
Playwright config, and the agent writes the file there, where it runs in a Marley terminal or
anywhere else. Marley keeps no copy.

## Acceptance
A recording's clicks, fills and presses carry locators; `browser_draft_test` returns a Playwright
test and a path in the project; the test, written there, replays the recording and passes, fails
when the page regresses, and holds no password.
