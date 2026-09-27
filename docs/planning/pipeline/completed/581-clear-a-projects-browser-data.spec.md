---
pipeline_id: 3e83fba1-d724-4b71-ac1d-9997c86d0d83
ticket: docs/planning/tickets/closed/TICKET-581-clear-a-projects-browser-data.md
status: Phase 4 — Complete PASS
title: "Clear a project's Browser data"
type: feature
slice: prong 3, B7c, after B7b (#507); split from #507 at its promotion
references: [docs/planning/pipeline/completed/507-browser-context-per-project.spec.md, docs/orca_architecture/03-browser-and-design-mode.md]
---

## Title
Clear Browser Data resets one project's browser: after it asks, it closes the project's Browser
tabs, closes and stops the project's Chromium, and deletes the project's profile, so every site
in the project is signed out and its next Browser tab starts a clean Chromium. The other
projects' browsers and logins are left alone. Since #507 each project has a profile of its own,
which is what makes a clear of one project possible.

## Scope
### In
- `crates/marley_browser/src/service.rs`: `remove_profile_in(project_dir)`, which deletes the
  project's `profile/` and keeps `project.json`; a profile that is not there is removed
  already.
- `crates/marley_workbench/src/browser.rs`: `BrowserHub::clear_browser_data(project, cx)`, a
  task that forgets the project's browser and pages, closes every Browser tab of the project
  (a tab still waiting for its page too), closes and stops its Chromium (`stop_chromium`,
  #507), including one an earlier Marley left running, and removes the profile off the main
  thread; `clear_project_browser_data(workspace, window, cx)`, which asks with Zed's prompt,
  runs the clear and says how it went in a toast; the action registered on every workspace.
- `crates/marley_workbench/src/marley_workbench.rs`: the action `marley::ClearProjectBrowserData`.
- `crates/marley_workbench/src/rail.rs`: "Clear Browser Data…" in the project row's menu, before
  Remove Project, for the row's project.
- `script/e2e/581-clear-a-projects-browser-data.sh`.

### Out (explicitly deferred)
- Clearing one site, or only cookies, or only storage (Chrome's own Clear Browsing Data options).
- Clearing every project at once.
- Clearing the profile of earlier builds when it stayed put (#507's `KeptBecauseAProjectHasOne`).
- An undo: the profile is deleted, not moved aside.
- Marley's recordings and picks of the project's pages: they are Marley's, kept under
  `browser/recordings` and in the hub, not the browser's, and a clear leaves them.

## Reference (§20)
Upstream Zed: the confirmation is Zed's own prompt (`gpui::Window::prompt`, drawn by
`ui_prompt::ZedPromptRenderer`: Enter picks the active button, the first; Escape picks
"Cancel"), as #550's close guard asks, and the outcome is a workspace toast
(`workspace::Toast`), as the Browser tab's Record This reports. The menu entry sits beside
Remove Project, the entry Zed's sidebar offers (`crates/sidebar/src/sidebar.rs`). Chrome's
"Clear browsing data" is the behavior matched for what a clear removes: every cookie and all
site storage of the profile. Warp: N/A, Warp has no browser.

### Prior art
- **Behavior maps and reports.** Report 03 §3 item 3 (a clear action per project, planned with
  the per-project profiles); #507's D6 and REQ-006, drafted there and moved here at #507's
  promotion; Orca's profile menu offers "Clear data" per profile (report 03 §2.8).
- **Published material.** CDP's `Storage.clearDataForOrigin` clears one origin's storage (its
  IndexedDB, local storage, service workers and caches with `all`), and
  `Network.clearBrowserCookies` every cookie, in a running browser; nothing clears every
  origin's storage at once, so a clear through CDP would have to know each origin the project
  ever visited. Deleting the profile once its Chromium stopped clears everything the profile
  holds, which is the behavior wanted.
- **The code we already ship.** #507 left the pieces: `BrowserHub::stop_browser` forgets a
  project's browser and pages (their tabs close through `PageClosed`), `stop_chromium` closes
  a Chromium over CDP and stops its unit (PR-claude-close-chromium-over-cdp-before-stopping-its-unit-001),
  and `service::profile_in` names the folder. Zed's `Window::prompt` and `Workspace::show_toast`
  own the ask and the report. Does a crate we build own the seam? Yes: `marley_browser::service`
  owns the profile's folder and `browser.rs` the hub; nothing new beyond one function in each.

## UI proof
UI-AFFECTING (the rail's menu, Zed's prompt, the toast, the Browser tabs).
`script/e2e/581-clear-a-projects-browser-data.sh` (`compositor sway`: it clicks the rail's
menu). Setup: the offline Chromium and #507's loopback site (`signin.html?as=<name>`,
`whoami.html`); scratch repositories alpha and beta. Marley opens alpha, which signs in as
`alpha`; beta, handed over, signs in as `beta`. Shots: beta's menu with Clear Browser Data…
selected (`581-01-menu`); the prompt naming beta (`581-02-asked`); after Escape, beta's tab
still signed in (`581-03-cancelled`); after Enter on Clear, beta's tabs gone and the toast
(`581-04-cleared`); beta's next tab signed out while alpha's tab still reads alpha
(`581-05-beta-signed-out`); a clear that cannot remove the profile says why in its toast
(`581-06-failed`). The run log carries the units' states and the project folders after each
step.

## Locked-In Decisions
- D1: The clear deletes the whole profile after its Chromium stopped, rather than clearing
  through CDP in a running browser: that removes every cookie, all site storage, the caches and
  the service workers the profile holds, and leaves nothing half-cleared.
- D2: It asks first, always, with Zed's prompt at warning level, naming the project and saying
  that its Browser tabs close and every site in it signs out; the buttons are Clear and Cancel,
  Clear first. Escape and Cancel change nothing.
- D3: It closes every Browser tab of the project, in every window, the tabs still waiting for
  their page included, before the Chromium closes: a tab left open would show a page of a
  browser that is gone.
- D4: The Chromium closes over CDP and its unit stops (`stop_chromium`, #507) before the profile
  is removed, and the clear waits for that; a Chromium an earlier Marley left running is closed
  through its profile's `DevToolsActivePort` the same way.
- D5: `project.json` stays, and so does the project's folder: the project's next Browser tab
  makes a new profile in it.
- D6: The outcome is a toast in the workspace the clear was asked from: that it is done, or why
  not. A clear that cannot remove the profile leaves the tabs closed and the unit stopped, and
  says what failed.
- D7: The entry is in the rail's project menu, after the separator and before Remove Project,
  for the row's project; the action `marley: clear project browser data` does the same for the
  workspace's project.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user chooses Clear Browser Data for a project, the system shall ask first, naming the project and saying that its Browser tabs close and its sites sign out. | Shots `581-01-menu`, `581-02-asked` |
| REQ-002 | WHEN the user cancels the question, the system shall leave the project's tabs, its Chromium and its logins as they were. | Shot `581-03-cancelled`; the log (the unit active, the tab's title) |
| REQ-003 | WHEN the user confirms, the system shall close the project's Browser tabs, stop its Chromium unit and delete its profile, keeping `project.json`, and say so in a toast. | Shot `581-04-cleared`; the log (the unit inactive, `profile/` gone, `project.json` kept) |
| REQ-004 | WHEN the project's next Browser tab opens after a clear, the system shall show its pages signed out: no cookie, `localStorage` or IndexedDB left from before. | Shot `581-05-beta-signed-out` |
| REQ-005 | WHILE one project is cleared, the system shall leave every other project's Chromium, tabs and logins as they were. | Shot `581-05-beta-signed-out` (alpha's row and tab); the log (alpha's unit active) |
| REQ-006 | IF the profile cannot be removed, THEN the system shall say why in the toast and keep `project.json`. | Shot `581-06-failed`; the log |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes.
- **P2 Code:** `service::remove_profile_in`; the hub's clear and the prompt and toast in
  `browser.rs`; the action; the rail's entry; fmt, clippy and the gate's dylint stage; a review
  of the diff against each REQ, gpui re-entrancy and errors reaching the toast.
- **P3 Test:** write and run the scenario and read every shot; rerun #507's scenario; the golden
  set with 581 added; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG, the plan's B7c row, `marley_browser.md` and `marley_workbench.md`,
  the ledger capture, close the ticket, archive, commit and push.
