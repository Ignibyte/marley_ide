# A recording turned into a Playwright test — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-506-recording-to-playwright-test.md
- **Pipeline spec:** 506-recording-to-playwright-test.spec.md

## Phase 1 — Plan (drafted overnight, 2026-09-25)
- **Request:** Chad, 2026-09-25: "lets do 1 through 8", item 3 of the list after the browser
  waves: record each click and typing target as a locator, and draft a Playwright test from a
  recording. His direction the same day, relayed with the night's brief: the app being built
  keeps its Playwright regressions in its own code, runnable in Marley or outside it, so the draft
  test is a file in the project, not Marley's. The Orca survey's report 03 item 9 gives the rules:
  locators computed at event time; `getByTestId`, then role and name, label, placeholder, text,
  CSS; passwords as `process.env` placeholders; `toHaveURL` after navigations; `baseURL`.
- **Classification / tier:** feature, prong 3, the flight recorder's second half (B5). Marley
  crates only (`marley_browser`, `marley_workbench`, `marley_mcp`) and two scenarios.
- **Recall (§18.3):**
  - #499's spec, D3: "No typed character is ever recorded; keys by name, text by count", and its
    REQ-002 and scenario grep (line 120 of `script/e2e/499-flight-recorder.sh` looks for the
    e-mail, the password and the token). D3 here narrows that rule; flagged for Chad (Risks).
  - AD-claude-499-the-flight-recorder-keeps-a-drawn-pages-minute-in-memory-001: the minute lives
    in memory, reaches disk only by Record this, and only while a tab draws the page; the listener
    reports always and the hub keeps entries only then, as `record_entry` does now.
  - L-claude-495-a-listener-and-a-binding-catch-what-headless-chromium-hides-001: the listener,
    binding and world pattern; events a world dispatches reach the page as untrusted, which is why
    Marley's own select choices are Out.
  - L-claude-492-chromium-shows-a-password-by-length-001: a password must be known in the page to
    be kept out; the listener checks `type` and `autocomplete` where the element is.
  - L-claude-499-a-screencast-sends-frames-only-when-the-page-changes-001: frames mark changes;
    the draft is built from entries, never frames.
  - L-claude-490-page-navigate-answers-at-the-commit-001: navigations are known at commit; the
    draft's `toHaveURL` waits by itself (Playwright retries until its timeout).
  - Brain: not consulted in this drafting run (read-only brief); promotion runs `brain_ask`, and
    D3 goes to Chad.
- **Discovery:**
  - `crates/marley_browser/src/recorder.rs`: `Entry` (41 to 111, tagged by `kind`: `click` with
    viewport coordinates, `typed` with a count, `key` by name, `navigation`, `console`,
    `request`, `snapshot`, `agent`, `frame`); `Recording` (134, id, tab, URL, title, time, length,
    frames, entries, snapshot); `Recorder::push` (191, merging typing and wheel turns);
    `save_in` (347), `list_in` (389), `read_in` (428, the timeline as JSON).
  - `crates/marley_workbench/src/browser.rs`: the page's attach task calls
    `watch_selects(page.session_id())` (771), where the record listener's setup joins it;
    `select_requested` (1040) returns early for any binding but `select::BINDING`;
    `record_entry` (1363, only while `viewers > 0`); `record_navigation` (1371, from
    `Page.frameNavigated`'s main-frame arm); `record` (1387, builds the `Recording` from the
    minute); `send_key` (1907) and `insert_text` (1913) push counts; `mouse_press` (1975) pushes a
    click by coordinates before the press is sent; the `Page.navigatedWithinDocument` arm (2398)
    records nothing today; `follow_observed` (2462) and its `Runtime.bindingCalled` arm (2498);
    `key_entry` (2596); `recordings_dir` (2641); `record_this` (3761), which knows the tab's
    workspace and so its project.
  - `crates/marley_browser/src/select.rs`: `WORLD` (17), `BINDING` (21), `LISTENER` (28),
    `watch_selects` (129: binding, script for new documents, a world per existing frame).
  - `crates/marley_browser/src/pick.rs`: `DESCRIBE` (43 to 100), whose locator and `unique`
    code the listener reuses, with #505's generated-name helper.
  - `crates/marley_workbench/src/browser_tools.rs`: `recordings` (212) and `recording` (226).
    `crates/marley_mcp/src/registry.rs`: the `recording` description (171) and
    `recording_schemas` (539). Both files are moving under #516's Code phase tonight.
  - Playwright on the box: `@playwright/test` 1.63.0 in `/srv/stacks/rustal/node_modules` (its
    Chromium build 1243 cached in `~/.cache/ms-playwright`); 1.61.1 in `creative_simulator` wants
    build 1228, which is not cached; `toHaveURLWithPredicate` in both calls `urlMatches(baseURL,
    …)`; `playwright-core/lib/coreBundle.js` holds `generateSelector` and `asLocator`.
- **Decisions:** D1 to D6 in the spec.

### Design
- **Approach.**
  1. *The listener (`marley_browser::recorder`, a `LISTENER` constant and `Page::watch_actions`).*
     Modeled on `watch_selects`: `Runtime.addBinding {name: "marleyRecord", executionContextName:
     "marley-record"}`, `Page.addScriptToEvaluateOnNewDocument` in that world, and
     `Page.createIsolatedWorld` plus `Runtime.evaluate` for the main frame's document already
     loaded. The script returns at once when `window !== window.top` or when it has run in the
     world before. Listeners at the window in the capture phase: `pointerdown` (trusted, button
     0) reports a `click` for the interactive ancestor of the target; `input` (trusted) reports a
     `fill` for the target field with its value or `secret: true`; `keydown` (trusted, Enter, Tab
     or Escape) reports a `press` with the key and the focused element's locators. Each report
     carries `location.href`. `locate(element)` builds the list in D2's order, each entry with
     `kind`, `value` (and `role` and `name` for the role entry) and `unique` (a
     `querySelectorAll` count for the CSS kinds, a count over candidates with the same computed
     role and name for the role kind, the same text for the text kind).
  2. *The hub.* `follow_observed` sends `Runtime.bindingCalled` to a dispatcher that routes by
     binding name: `marleySelect` to `select_requested`, `marleyRecord` to `action_reported`,
     which parses the report, redacts its URL with `redact_url`, and calls `record_entry(target,
     Entry::Action { … })`. `Recorder::push` merges a fill into the fill right before it when their
     first locators are equal, keeping the last text. The `Page.navigatedWithinDocument` arm
     records a `Navigation { url, within: true }`. `record(target, dir, project, cx)` gains the
     project root, which `record_this` reads from its workspace's first visible worktree.
  3. *The draft (`marley_browser::playwright`, pure).* `draft(timeline: &Value) -> Result<Draft,
     DraftError>` with `Draft { text, env: Vec<String>, skipped: Vec<String> }`. It reads the
     timeline as saved (so older recordings without actions give a draft of skips and a note).
     The start page is the first action's `url`; `baseURL` is its origin; paths are relative. A
     navigation counts as caused by an action when it follows one; each gives one `toHaveURL`
     (a string path, or `new RegExp('^' + escaped path)` when the URL holds the recorder's hidden
     value `…`). A secret fill writes `await page.<locator>.fill(secret('<NAME>'))`, `NAME` from
     the field's name, id or label in upper snake case, unique within the test; `secret` is a
     four-line helper after the import that throws `set <NAME> to run this test` when the
     variable is unset. Strings are written as JavaScript string literals with single quotes
     escaped. `.press(key)` keys keep Playwright's names (`Enter`, `Tab`, `Escape`).
  4. *The tool (`browser_tools.rs`).* `browser_draft_test {id}`: `read_in(recordings_dir(),
     id)`, `draft`, then the suggested path: the recording's `project` root, the first
     `playwright.config.{ts,js,mjs,cjs,mts,cts}` there read for `testDir:\s*['"]([^'"]+)['"]`
     (the `regex` crate), else `tests`, joined with `<slug of the title>-<id>.spec.ts`; a project
     without a config gets a note that `npm init playwright@latest` sets one up. The answer:
     `test`, `path`, `env`, `skipped`, `run` (`npx playwright test <path>`), after the
     `Redactor` (a flagged fill becomes a `secret(…)` before the text is assembled). The registry
     row is a `browser_read` with its schemas; `browser_recording`'s description changes from
     "typing as counts, never the characters" to name what is kept.
  5. *#499's scenario.* Line 120's grep keeps the password and the token and drops the e-mail;
     a second check expects the e-mail inside a `fill` action.
- **File manifest.** Marley crates only: `crates/marley_browser/src/recorder.rs` (the listener,
  `watch_actions`, `Entry::Action`, `within`, `Recording.project`, the merge),
  `crates/marley_browser/src/playwright.rs` (new), `crates/marley_browser/src/marley_browser.rs`
  (the module), `crates/marley_browser/src/pick.rs` (the shared locator code with #505's rule);
  `crates/marley_workbench/src/browser.rs` (the dispatcher, `action_reported`, the navigation
  within the document, `record`'s project), `crates/marley_workbench/src/browser_tools.rs` (the
  tool); `crates/marley_mcp/src/registry.rs`. `script/e2e/browser-fixture.sh` (`draft-test`),
  `script/e2e/499-flight-recorder.sh` (its grep), `script/e2e/506-recording-to-playwright-test.sh`
  (Test). Docs at Complete: `CHANGELOG.md` (the recorder's rule said plainly),
  `docs/marley/three-prong-plan.md`, `docs/marley_architecture/marley_browser.md`.
- **Ledger rows.** None in `docs/marley/zed-touchpoints.md`: no Zed path. Knowledge at Complete
  (expected): an AD superseding the typing clause of AD-claude-499 (ordinary fields as text,
  secret fields as counts) and one for the draft living in the project; lessons from running
  drafts through Playwright.

### E2E plan
Setup: `offline_chromium`; a site with `signin.html` (label "E-mail" and an `email` input; label
"Password" and a `password` input named `password`; a button with `data-testid="sign-in"`, "Sign
in", whose handler sends the page to `welcome.html`), `welcome.html` (a heading "Welcome", a
`type="search"` field with the placeholder "Search orders" and no label, a link "Orders" that
calls `history.pushState` to `/orders` and draws "Orders"), and `signin-broken.html` for the
regression; the scratch
repository with `playwright.config.ts` (`testDir: './e2e'`) and its `node_modules` a link to the
Playwright project's (`/srv/stacks/rustal/node_modules`, or `E2E_PLAYWRIGHT_MODULES`); setup stops
with a message when that is missing. The password is assembled at run time.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-003 | open `signin.html`; click E-mail, type the e-mail; click Password, type the password; click Sign in; on `welcome.html` click the search field, type `4512`, press Enter; click Orders; Record this | `506-01-recorded` (the toast); `mcp_agent recording <id>`: `action` entries with locators, the e-mail and `4512` as fill texts, the password's fill `secret` |
| REQ-004, REQ-005, REQ-006, REQ-008 | `mcp_agent draft-test <id> <file>` | the log: the draft (`getByTestId('sign-in')` for the button; `getByRole('textbox', { name: 'E-mail', exact: true })` and `getByRole('searchbox', { name: 'Search orders', exact: true })`; `getByLabel('Password', { exact: true })` for the password field, which has no ARIA role; `getByRole('link', { name: 'Orders', exact: true })`; `toHaveURL('/welcome.html')` and `toHaveURL('/orders')`; `baseURL` the fixture's origin), the suggested path `e2e/sign-in-<id>.spec.ts`, `env` naming `PASSWORD` |
| REQ-005 | the scenario writes the draft to the suggested path, as the agent would, and opens it in Marley through the palette's file finder | `506-02-draft`: the test in Marley's editor inside the project |
| REQ-007 | `PASSWORD=<the password> node_modules/.bin/playwright test e2e/<file> --reporter=line` in the scratch repository | the log: the test passes (exit 0) |
| REQ-009 | the fixture's Sign in handler swapped for `signin-broken.html`'s (no navigation); the same run | the log: the test fails at `toHaveURL('/welcome.html')` (exit not 0) |
| REQ-010 | the run once more without `PASSWORD` | the log: the helper's `set PASSWORD to run this test` |
| REQ-002 | `grep -rF` for the password in the recording's folder and in the draft | the log: nothing found |
| (regression) | rerun `499-flight-recorder.sh` | its log: the password and the token absent, the e-mail present as a fill |

Not reachable here: a real dev server's hot reload and the agent's own file write (the scenario
writes the file the way the agent would); selects (Out).

### Risks
- D3 reverses part of a rule that shipped (#499's D3, and CHANGELOG's "typing as a count of
  characters (never the characters)"). The frames argument holds for what is on screen, but a
  recording is also read by agents; if Chad wants #499's rule whole, the fallback is to draft
  every fill as a placeholder named after its field and keep counts, at the cost of tests that
  need every value supplied.
- Marley's role and name are an approximation of the accessible name Playwright computes; where
  they differ, `getByRole` fails at run time. The draft prefers test ids, the Test phase runs each
  draft, and a mismatch found there moves that element's kind down the order.
- A `pointerdown` listener in the capture phase at the window runs before the page's own
  capture-phase listeners on the window only if it was added first; the script for new documents
  runs before the page's scripts, so it is, but on the document already loaded when Marley
  attaches it is not. An element the page replaces during its own earlier listener would be
  reported as replaced. Rare, and the draft still names what the listener saw.
- The draft replays one minute. A flow longer than the recorder's window starts mid-way; the
  draft's `goto` is the first recorded action's page, which may need a signed-in state the test
  does not create. The answer says where it starts.
- The scenario depends on another project's `node_modules` for Playwright; the setup names the
  path and stops when it is gone.
