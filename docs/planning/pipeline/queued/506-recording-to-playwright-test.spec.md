---
pipeline_id: 22eec7fe-5cee-40c4-9cdf-64aea79a0317
ticket: docs/planning/tickets/open/TICKET-506-recording-to-playwright-test.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A recording turned into a Playwright test the project keeps: locators at event time, secrets as env placeholders"
type: feature
slice: prong 3 (after wave 2), item 3 of the list after the browser waves; after B5 (#499) and #505
references: [docs/orca_architecture/README.md, docs/orca_architecture/03-browser-and-design-mode.md, docs/planning/pipeline/completed/499-flight-recorder.spec.md, docs/planning/pipeline/completed/495-select-picker.spec.md, docs/planning/tickets/open/TICKET-505-pick-fix-check.md]
---

## Title
What the user did in a Browser tab becomes a Playwright test for the app being built: the
recorder notes each target as locators when the event happens, and a tool drafts the test from a
recording for the agent to put in the project, where it runs in Marley or anywhere else.

## Scope
### In
- **The record listener (`marley_browser`).** An isolated world `marley-record` in each page's main
  frame with a binding `marleyRecord`, set up as #495's select listener is
  (`Runtime.addBinding`, `Page.addScriptToEvaluateOnNewDocument` with `worldName`,
  `Page.createIsolatedWorld` for the document already loaded). Capture-phase listeners at the
  window take trusted `pointerdown` (the left button), `input`, and `keydown` of Enter, Tab and
  Escape, compute the target's locators at once, before the page's own handlers run, and report
  `{action, locators, text, secret, key, url}` through the binding. It runs in the top frame only.
  - Locators, in order, each marked when it finds the element alone: a test id (`data-testid`,
    else `data-test`, `data-cy`, `data-qa`); role and name (the explicit role, else the implicit
    one of buttons, links, text and search fields, checkboxes, radios, selects, headings and
    images, and none for a password field, as ARIA in HTML gives it none; the name from
    `aria-labelledby`, `aria-label`, a label, `alt`, `title`, the text, the placeholder); the
    label of a form field; the placeholder; the text (80 characters at most); a CSS path. A click
    walks to the interactive ancestor, as a pick does. Generated names stay out (#505's rule).
  - `text`: a fill's field value after the input, 1,000 characters at most. A secret field (a
    password or hidden input, or an `autocomplete` of `current-password`, `new-password`,
    `one-time-code` or `cc-…`) sends none and `secret: true`.
- **The recorder (`recorder.rs`, `browser.rs`).** A new entry kind, `action` (`click`, `fill` or
  `press`, the locators, the page's URL through `redact_url`, the text or `secret`, the key);
  consecutive fills of one field merge, the last text kept, as typing's counts merge now. The
  existing entries stay. `Page.navigatedWithinDocument` is kept as a navigation too (`within:
  true`). A saved recording names its tab's project root (`project`).
- **The draft (`crates/marley_browser/src/playwright.rs`, new, pure).** From a saved timeline:
  the `@playwright/test` import; `test.use({ baseURL })` from the first action's origin; one test
  named after the page's title and the recording's id; `await page.goto(<path>)` to the first
  action's page; per action one line with the first locator that found the element alone,
  `page.getByTestId(…)`, `page.getByRole(<role>, { name, exact: true })`, `page.getByLabel(…, {
  exact: true })`, `page.getByPlaceholder(…, { exact: true })`, `page.getByText(…, { exact: true
  })` or `page.locator(<css>)`, then `.click()`, `.fill(<text>)` or `.press(<key>)`; a secret
  field's fill as `secret('<NAME>')`, a helper at the top that reads `process.env.<NAME>` and fails
  the test naming the variable when it is unset; `await expect(page).toHaveURL(<path>)` after each
  navigation an action caused (a regular expression on the path when the URL had hidden values).
  An action it cannot write becomes a comment saying what was skipped and why.
- **The tool.** `browser_draft_test {id}`, a read tool: it reads the recording and the project's
  Playwright config and writes nothing. It answers the test, a suggested path in the recording's
  project (the `testDir` a `playwright.config.{ts,js,mjs,cjs,mts,cts}` sets as a plain string,
  else `tests/`, with `<slug>-<recording id>.spec.ts`), the environment variables it reads, what it
  skipped, and how to run it (`npx playwright test <path>`). The draft and `browser_recording`'s
  entries pass #516's `Redactor`; a fill the Redactor flags is drafted as a `secret(…)` too.
- **#499's scenario.** Its grep of the recording keeps looking for the password and the token and
  now expects the e-mail, which is typed into an ordinary field.
- **The fixture.** `mcp_agent draft-test <id> [<file>]` in `script/e2e/browser-fixture.sh`.

### Out (explicitly deferred)
- Choices in `<select>` lists: Marley draws the list and sets the choice in the select world
  (#495), whose events the page sees as untrusted, so the listener does not record them. A later
  ticket records them from `choose_option`.
- Actions in cross-site iframes and inside shadow roots (Playwright's `frameLocator` and
  piercing): skipped, with a comment in the draft.
- Assertions beyond the URL (text, visibility, values), which Playwright's codegen offers by hand.
- Marley writing the file itself, a button that drafts, running the test from Marley, and the test
  driving Marley's own Browser tab: the draft is the project's (Chad's direction), and Marley's
  browser for outside clients and Marley's own saved Playwright scripts are other tickets from
  Chad's decisions of 2026-09-25.

## Reference (§20)
Playwright's test generator (`npx playwright codegen`, playwright.dev/docs/codegen) records clicks
and fills into a test, picking locators by role, text and test id and tightening them until they
are unique. Its locator guidance (playwright.dev/docs/locators) ranks `getByRole` first and
`getByTestId` seventh; Marley takes the survey's order, test id first, because a test id is a
contract the app's authors wrote for tests, and Marley's role and name approximate Playwright's
own computation in the page. `toHaveURL` resolves a relative path against `baseURL` (read in
Playwright 1.61.1's and 1.63.0's `toHaveURLWithPredicate`, which calls `urlMatches(baseURL, …)`).
Orca has no recorder and no test generation (report 03 §2.15). Upstream Zed: N/A. Warp: N/A (the
once-over lists browser recordings as Marley's own).

### Prior art
- **Behavior maps and reports.** `docs/orca_architecture/README.md` (the #506 row), report 03
  item 9 (locators at event time; `getByTestId`, then role, label, placeholder, text, then CSS;
  passwords as `process.env`; `toHaveURL` after navigations; `baseURL` from the recording's
  origin; the snapshot's role, name and occurrence mapping to `getByRole(…).nth(n)`) and §2.11
  (Orca's old snapshot engine recovering a re-rendered ref by role, name and occurrence,
  `src/main/browser/cdp-ref-resolution.ts`, read at `1c2cf120e3`).
- **Published material.** Playwright's codegen and locator pages; playwright-core 1.61.1 on the
  box (`/srv/stacks/creative_simulator/node_modules/playwright-core/lib/coreBundle.js`, Apache-2.0)
  holds Playwright's own `generateSelector` and `asLocator`, considered and not vendored: a 3.3 MB
  bundle written for Playwright's injected runtime. The Test phase runs each draft through
  Playwright instead, which catches a locator Marley computes differently. CDP's
  `Runtime.addBinding` and `Page.addScriptToEvaluateOnNewDocument` with `worldName`.
- **Code we already ship.** `recorder.rs` (`Entry`, `Recorder::push` and its merging, `Recording`,
  `save_in`, `read_in`); `browser.rs` (`record_entry`, `record_navigation`, `record`,
  `record_this`, `key_entry`, which keeps counts, `mouse_press`, `insert_text`, the
  `Page.navigatedWithinDocument` arm, `follow_observed`'s `Runtime.bindingCalled` arm and
  `select_requested`, which filters by binding name); `select.rs` (`watch_selects`, the listener
  and binding, L-claude-495); `pick.rs` (`DESCRIBE`'s locators and `unique` checks, #505's
  generated-name rule); `observe.rs` `redact_url`; `browser_tools.rs` (`recordings`,
  `recording`); `registry.rs` (`recording_schemas`); #516's `Redactor`. For the scenario:
  `@playwright/test` 1.63.0 in `/srv/stacks/rustal/node_modules`, whose Chromium build (1243) is in
  `~/.cache/ms-playwright` (1.61.1 in `creative_simulator` wants build 1228, which is not).

## UI proof
N/A — no UI delta: the listener draws nothing, Record this and its toast are #499's, and the
draft reaches the project through the agent. The e2e run is a scenario rather than `just shot`,
since the proof is a drafted test that runs: `script/e2e/506-recording-to-playwright-test.sh`
(`compositor sway`, offline Chromium) signs in on a fixture page, searches, follows an in-app link
and records (`506-01-recorded`); the stand-in agent reads the recording and drafts the test; the
scenario writes it where the tool suggests, as the agent would, and opens it in Marley
(`506-02-draft`); Playwright runs it against the fixture with the password in the environment,
then against a broken fixture; greps look for the password. The run log is the proof; the shots
are for reading.

## Locked-In Decisions
- D1 — Locators are computed in the page at the event: capture-phase listeners at the window in an
  isolated world, reporting through a binding (#495's pattern, L-claude-495), before the page's
  own handlers change the DOM. Only trusted events count, so a page's scripted clicks are not
  recorded, and an agent's clicks and typing, sent as the same trusted input as the user's (#492),
  are.
- D2 — The order is the survey's: test id, role and name, label, placeholder, text, CSS path, and
  the draft takes the first that found the element alone at the event. Generated names never
  enter (#505's rule).
- D3 — Text typed into an ordinary field is recorded; a password, hidden, one-time-code or card
  field keeps a count only, and its fill is drafted as a `process.env` placeholder that fails the
  test by name when unset. This narrows #499's D3 ("no typed character is ever recorded") to secret
  fields: the recorder's frames already show an ordinary field's text on the screen, and a
  password field shows dots. Chad reviews this change before the ticket is promoted.
- D4 — The draft is the project's: the tool returns the test and a suggested path, the agent writes
  the file with its own tools and prompts, and Marley keeps no copy. It is plain Playwright with
  `baseURL` from the recording, so it runs in a Marley terminal or anywhere the dev server runs.
- D5 — The draft asserts the URL after each navigation an action caused, a move within the
  document included, as a path relative to `baseURL`; a URL whose values the recorder hid is
  asserted by its path alone.
- D6 — `browser_draft_test` reads and never writes, and its answer passes #516's `Redactor`; a
  fill value the Redactor flags is drafted as a placeholder, like a password's.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks, fills a field, or presses Enter, Tab or Escape in a page a tab draws, the recording shall hold an `action` entry with the target's locators as they were at the event. | The run log's `mcp_agent recording` |
| REQ-002 | WHEN the user types into a password field, neither the recording nor the draft shall hold any character of it. | The run log's grep |
| REQ-003 | WHEN the user types into an ordinary field, the recording shall keep the field's text, once per run of typing. | The run log's `mcp_agent recording` |
| REQ-004 | WHEN an agent calls `browser_draft_test` with a recording's id, the answer shall give a Playwright test, a path in the recording's project, and the environment variables the test reads. | The run log's `mcp_agent draft-test` |
| REQ-005 | The draft shall locate each target by `getByTestId` when the target had a unique test id at the event, and otherwise by role and name, label, placeholder, text or CSS path, in that order. | The run log's draft; shot `506-02-draft` |
| REQ-006 | The draft shall assert the URL after each navigation an action caused, a move within the document included. | The run log's draft |
| REQ-007 | WHEN the drafted test runs with Playwright against the dev server with its variables set, it shall pass. | The run log: the first Playwright run's exit code |
| REQ-008 | The draft shall set `baseURL` from the recording's origin and address every page by its path. | The run log's draft |
| REQ-009 | WHEN the page no longer does what was recorded, the drafted test shall fail at the step that differs. | The run log: the run against the broken fixture |
| REQ-010 | WHEN a variable the draft reads is unset, the test shall fail with a message naming it. | The run log: the run without `PASSWORD` |

## Phase Plan
- **P1 Plan** — promote this pair after #505 (its generated-name rule) and with D3 confirmed by
  Chad; re-verify the recorder's seams and #516's `Redactor` against HEAD.
- **P2 Code** — the listener and its binding, the `action` entries, the navigation within the
  document, the recording's `project`, `playwright.rs`, the tool and its registry row, the
  fixture's command, #499's scenario's grep; fmt and clippy clean; a review of the diff (no
  secret field's text anywhere, the listener before the page's handlers, only trusted events).
- **P3 Test** — write and run the scenario and read the log and the shots; rerun #499's scenario;
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG (the recorder's changed rule said plainly), the prong's slice status
  and `docs/marley_architecture/marley_browser.md` (§21), ledger capture (§19), close, archive,
  commit.
