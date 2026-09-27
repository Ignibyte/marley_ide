# A recording turned into a Playwright test — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-506-recording-to-playwright-test.md
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

## Chad's answer, 2026-09-26
- D3 confirmed: text typed into ordinary fields is recorded; secret fields keep a count and
  become `process.env` placeholders.

## Promotion (2026-09-27, at `5f84183d71`)
- **D3 confirmed** by Chad on 2026-09-26 (the answer above; the spec's D3 now says so). Promotion
  waited on it.
- **Seams re-read** (the draft's line numbers moved with #503 to #505; the shapes stand):
  - `recorder.rs`: `Entry` (41), `Recording` (134), `Recorder` (181) and `push` (191),
    `save_in` (347), `list_in` (389), `read_in` (428). As the draft read them.
  - `select.rs`: `WORLD` (17), `BINDING` (21), `LISTENER` (28), `watch_selects` (129).
  - `browser.rs`: `watch_selects` runs for the page's own session (938) and for each cross-site
    iframe's session (2043). The record listener's setup joins the first only, and the script's
    `window !== window.top` guard covers the rest. Then `select_requested` (1220, which returns
    for any binding but `select::BINDING`), `record_entry` (1652), `record_navigation` (1660),
    `record` (1676, now `pub fn`), `send_key` (2221), `insert_text` (2227), `mouse_press`
    (2289), the `Page.navigatedWithinDocument` arm (2714, which updates the history and records
    nothing), `follow_observed` (2778) and its `Runtime.bindingCalled` arm (2814, which calls
    `select_requested` alone), `key_entry` (2912), `recordings_dir` (2957), `record_this`
    (4235).
  - `browser_tools.rs`: `recordings` (442) and `recording` (456). `recording` hands the saved
    timeline back as it is, with no redaction: #516's redactor reached the console, the terminal
    tools and the picks, not recordings. #506 adds it, since fills now keep text.
  - `registry.rs`: the `recordings` (173) and `recording` (179) descriptions,
    `recordings_schemas` (576), `recording_schemas` (604).
  - `redact.rs`: `Redactor::new` (158), `redact` (175).
- **Landed since the draft:** #505's `interactive!` and `generated_name!` macros in `pick.rs`,
  which the record listener takes; #518's redaction pattern (`pick_for_agents`: redact whole,
  then cut); #504 and #580 (nothing on this seam).
- **Playwright on the box:** `@playwright/test` 1.63.0 in `/srv/stacks/rustal/node_modules`, with
  `chromium-1243` and `chromium_headless_shell-1243` in `~/.cache/ms-playwright`. The scenario
  uses them read-only and runs Playwright in the scratch repository, whose `test-results` stays
  in `$E2E_WORK`. Nothing is written into rustal.
- **Recall added:**
  - AD-claude-499 rejected "keeping typed text (a password is typed text)". D3 supersedes that
    clause for ordinary fields: an AD at Complete says so, and the CHANGELOG entry says plainly
    what the recorder now keeps.
  - AD-claude-516, PR-claude-redact-the-whole-text-before-cutting-it-001 and
    F-claude-516-a-cut-before-redaction-leaks-the-cut-secret-001: nothing an agent reads may be cut
    before the redactor has read it whole.
  - L-claude-495-a-listener-and-a-binding-catch-what-headless-chromium-hides-001: the world and
    binding pattern; events a world dispatches are untrusted, which is why selects are Out.
  - F-claude-495-the-list-opened-for-an-agents-click-in-a-focused-tab-001: an agent's trusted
    input is recorded as the user's is (D1). The action entry does not say whose it was; the
    minute's `agent` entries already do.
  - L-claude-516-fake-secrets-are-put-together-at-run-time-001: the scenario's password is built
    from pieces.
- **Brain consultation f1f29d315cbf4227a0a62a3121eab843:** nothing on this seam.

### Design changes at promotion
- **The fill's text cap:** the listener caps a fill's text at 4,096 characters for the trip,
  marked, instead of 1,000. `browser_recording` and `browser_draft_test` redact each string of the
  timeline whole, then cut a fill's text to 1,000 in what they answer (D6 and the prevention rule
  above).
- **`browser_recording` redacts:** the tool walks the saved timeline and passes every string
  through `agent_redactor`: fills, console texts, snapshot texts and URLs. A fill it flags is
  marked `secret` for the draft, which writes it as a `secret(…)` placeholder.
- **Sharing #505's JavaScript:** `pick.rs` re-exports `interactive!` and `generated_name!` with
  `pub(crate) use`, and the record listener splices them in with `concat!`, as `DESCRIBE` does.
- **Phase 1 checklist** (no task tool in this session): pre-flight clean; D3 confirmed; the pair
  promoted; the BACKLOG row removed; the ticket in progress; the seams re-read; recall; the
  brain asked; the design changes written. Status: Plan PASS.

## Phase 2 — Code
- **Built** (Marley crates only):
  - `marley_browser/src/pick.rs`: a third shared piece, `css_path!` (the CSS path, walking past
    generated ids), which `DESCRIBE` now takes in place of its inline loop. The three macros are
    re-exported with `pub(crate) use` for the recorder.
  - `marley_browser/src/recorder.rs`:
    - `WORLD` (`marley-record`), `BINDING` (`marleyRecord`) and `LISTENER`, spliced from the
      shared pieces with `concat!`. The listener works in the top frame only, at the window, in
      the capture phase, on trusted events:
      - `pointerdown` with the primary button, walked up to the interactive ancestor;
      - `input` on a fillable field (text-like inputs, text areas, editable elements; a
        checkbox, radio or file input is left to its click), with the text capped at 4,096 for
        the trip unless the field is secret (a password or hidden input, or an `autocomplete` of
        `current-password`, `new-password`, `one-time-code` or `cc-…`);
      - `keydown` of Enter, Tab or Escape, on the focused element.
    - Locators in D2's order, each with `unique`: the test id (with its attribute); the role
      (explicit, else the implicit roles of ARIA in HTML) and the name (`aria-labelledby`,
      `aria-label`, an input button's value, the label, `alt`, the text, `title`, the
      placeholder); the label; the placeholder; the text, counted from the text nodes that hold
      its first word; the CSS path.
    - `ActionLocator`, `ReportedAction::parse` and `into_entry` (the URL through `redact_url`,
      each text held to its trip cap, no text for a secret field whatever the page sent, only
      Enter, Tab and Escape as keys).
    - `Entry::Action`, `Entry::Navigation { within }`, `Recording.project`.
    - `Recorder::push`'s fill merge: into the last action when it is a fill of the same first
      locator. The merged fill keeps its first time, so the minute stays in time order.
    - `Page::watch_actions`: the binding, the script for new documents, and a world in the main
      frame's loaded document.
  - `marley_browser/src/playwright.rs` (new, pure): `draft(timeline) -> Result<Draft { text, env,
    skipped, start }, DraftError>`. It writes the import; the `secret` helper when a secret
    field is filled; `test.use({ baseURL })`; `goto` the first action's path; a line per action
    with the first unique locator (`getByTestId` for `data-testid` and a `locator` for the other
    test attributes); an `expect(page).toHaveURL` for the last navigation after each action (a
    `RegExp` on the path when the URL holds a hidden value); and a comment for each skip.
  - `marley_workbench/src/browser.rs`: `watch_actions` beside `watch_selects` for the page's own
    session; `Runtime.bindingCalled` routed by name to `action_reported` or
    `select_requested`; navigations within the document recorded (`within: true`); `record`
    takes the project, which `record_this` reads from its workspace's first visible worktree.
  - `marley_workbench/src/browser_tools.rs`: `timeline_for_agents` redacts every string of a
    timeline whole. For a draft, a fill the redactor changed becomes secret; for
    `browser_recording`, fills are then cut to `FILL_BUDGET` (1,000). `browser_recording` now
    redacts, off the main thread. `browser_draft_test` works off the main thread and writes
    nothing. It reads the recording, redacts it and drafts, then finds `testDir` in the
    project's `playwright.config.*` (`test_dir_in`: a plain relative path inside the project,
    no `..`), else `tests`. It answers the test, the path (`<slug>-<id>.spec.ts`), the project,
    the variables, the skips, the start, `run` and a note.
  - `marley_mcp/src/registry.rs`: the `draft_test` read row and `draft_test_schemas`; the
    `recording` description names what fills keep, and that secrets are redacted.
  - `script/e2e/browser-fixture.sh`: `draft-test <id> [<json file>]`. `script/e2e/499-flight-recorder.sh`:
    its grep becomes two checks, neither the password nor the token in the recording, and the
    e-mail kept as a fill.
- **Deviations from the design, and why:**
  - `test_dir_in` is a small parser, not the `regex` crate, which `marley_workbench` does not
    build with.
  - The text locator's uniqueness is counted from the text nodes that hold the text's first
    word. Counting over every element read each one's `innerText`, which is quadratic, inside a
    `pointerdown` handler the page waits on.
  - `input` from a checkbox, radio, file, range or color input records no fill: Playwright cannot
    fill those, and their click is recorded.
  - The secret helper builds its message by concatenation rather than a template literal, whose
    `${…}` clippy reads as a Rust format argument in a plain string.
- **Review** (against each REQ and the security line):
  - No secret field's text anywhere: the listener sends none, `into_entry` drops any the page
    sent, the typing counts stay counts, the draft reads the variable, and the frames show dots
    (REQ-002).
  - Everything an agent reads from a recording passes the redactor whole before any cut
    (PR-claude-redact-the-whole-text-before-cutting-it-001).
  - Only trusted events count, so a page's scripted clicks are not recorded, while an agent's
    input, as trusted as the user's (#492), is (D1).
  - No entity is updated while it is updated: `action_reported` runs inside the hub's own
    update.
  - Recordings stay in Marley's data directory (AD-claude-499), and the draft tool writes
    nothing (D4).
- **Checks:** `cargo check`; `cargo fmt`; `cargo clippy -p marley_browser -p marley_mcp -p
  marley_workbench --all-targets -- -D warnings`, clean after six fixes (four string pushes of
  `format!`, a JavaScript `${…}` in a Rust string, a match as `map_or_else`, and a first doc
  paragraph); `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` on the three, clean; `cargo
  dylint`, exit 0; `node --check` on `DESCRIBE`, `REFIND` and the listener as `concat!` assembles
  them; shellcheck on the fixture and #499's scenario; the stand-in agent's Python compiles.

## Phase 3 — Test
- **The scenario**, `script/e2e/506-recording-to-playwright-test.sh` (`compositor sway`,
  offline Chromium):
  - The fixture app: `signin.html` (E-mail and Password fields, each with its label, and Sign in
    with `data-testid="sign-in"`, which goes to `welcome.html`) and `welcome.html` (a search
    field known only by its placeholder, and an Orders link that `history.pushState`s to
    `/orders`). A `broken` sign-in page's button only writes a status line.
  - The scratch repository has `playwright.config.ts` (`testDir: './e2e'`, its output in the
    repository). `@playwright/test` resolves through `$E2E_WORK/node_modules`, a link beside the
    repository to `/srv/stacks/rustal/node_modules` (or `E2E_PLAYWRIGHT_MODULES`). The setup
    stops, naming what is missing, without it, and nothing is written into rustal.
  - The password and the e-mail are made from pieces in the setup.
- **The run** (`506-run1.log`, the first): 16 checks pass, none fail.
  - REQ-001: the recording's actions are click, fill, click, fill, click, click, fill, press,
    click, each with its locators. Sign in's first locator is `{test id, data-testid, sign-in,
    unique}`.
  - REQ-003: the fills' texts are the e-mail, none and `4512`, one fill each for the run of
    typing.
  - REQ-002: the password's fill is `secret` with no `text`.
  - REQ-006: the link's move is a navigation with `within: true`.
  - REQ-004, REQ-005, REQ-008: `browser_draft_test` answered the test, the path
    `<repo>/e2e/welcome-20260927-003737.spec.ts` (the page's title at the save), `env:
    ["PASSWORD"]` and the start. The test has `baseURL: 'http://127.0.0.1:40915'` and
    `page.goto('/signin.html')`, and it locates each target by the first unique locator:
    `getByRole('textbox', { name: 'E-mail', exact: true })`, `getByLabel('Password', { exact:
    true })` (a password field has no ARIA role), `getByTestId('sign-in')`,
    `getByRole('searchbox', { name: 'Search orders', exact: true })` and `getByRole('link', {
    name: 'Orders', exact: true })`. Its checks are `toHaveURL('/welcome.html')` and
    `toHaveURL('/orders')`, and the password is `fill(secret('PASSWORD'))`.
  - REQ-007: Playwright 1.63.0 ran the written test with `PASSWORD` set: `1 passed (701ms)`,
    exit 0.
  - REQ-009: against the broken sign-in page it failed at line 24, `await
    expect(page).toHaveURL('/welcome.html')`, exit 1.
  - REQ-010: without `PASSWORD` it failed with `set PASSWORD to run this test`, exit 1.
  - REQ-002: no file of the recording, the draft's answer or the test holds the password.
  - Nothing reached the system browser.
- **The shots**, read:
  - `506-01-recorded`: the Welcome page at `/orders`, "4512" in the search field, "Orders" drawn
    by the link, the toast "Saved this page's last minute as recording 20260927-003737; agents
    read it with browser_recording."
  - `506-02-draft` (REQ-005): the drafted test open in Marley's editor at
    `e2e/welcome-20260927-003737.spec.ts`: the import, the `secret` helper, `test.use({
    baseURL })`, and the thirteen steps as above. The editor's two TypeScript diagnostics (no
    `@types/node` in the scratch repository) are not the test's, which Playwright runs as it is.
- **Focus:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule and did
  not reload it".
- **#499's scenario, rerun:** its two new checks pass: neither the password nor the token is in
  the recording, and the e-mail is kept as a fill. Its timeline shows the actions beside the
  typing counts, which now stand on either side of a fill.
- **#495's scenario, rerun** as well, since its binding now shares `Runtime.bindingCalled`'s routing:
  its shots are as before. The list opens with its groups, Plum and then Lime are chosen by click
  and by keys, the cross-site iframe's list opens under it, and an agent's click opens nothing.
- **Not reached here:** a real dev server's hot reload and the agent's own file write (the
  scenario writes the file the way the agent would), and selects (out of scope).
- **The golden set**, with `506-recording-to-playwright-test.sh` added: `just regress` ran all
  28 against the debug build, and all 28 passed (#506's in 63 s).
- **The gate:** `script/gates.sh --diff` (`gate-506.log` in the session's scratchpad): 16
  passed, 0 failed, `GATE GREEN [diff]`, with the receipt.
- **Verdict:** Phase 3 PASS. Nothing pre-existing was excluded.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: "A recording drafted as a Playwright test"; Changed:
  "The flight recorder keeps what you type into ordinary fields", the recorder's rule said
  plainly); the plan's row B5b in `docs/marley/three-prong-plan.md`;
  `docs/marley_architecture/marley_browser.md` (the action listener in the recorder's section,
  and "Drafting a Playwright test"); `docs/marley_architecture/marley_workbench.md` (the
  recording tools' redaction, `browser_draft_test`, and the flight recorder's #506 additions,
  among them agents' input now recorded as actions);
  `docs/marley_architecture/marley_mcp.md` (the tool counts, now eleven read and nine write
  tools, with #505's `browser_check_pick`, which #505 left out, and #506's `browser_draft_test`).
  No Zed path was touched, so no ledger row.
- **Knowledge appended:** F-claude-506-a-text-count-read-every-elements-inner-text-001,
  L-claude-506-a-macro-shares-javascript-between-const-page-functions-001,
  L-claude-506-prove-a-generator-by-running-what-it-makes-001,
  AD-claude-506-a-recording-becomes-a-test-the-project-keeps-001, which supersedes
  AD-claude-499's typing clause.
- **Brain:** consultation f1f29d315cbf4227a0a62a3121eab843 closed with
  `decisions/a-recording-becomes-a-playwright-test-the-project-keeps-506`, follow-up by
  2026-10-27.
- **Closed and archived:** the ticket in `docs/planning/tickets/closed/`, this pair in
  `docs/planning/pipeline/completed/`.
