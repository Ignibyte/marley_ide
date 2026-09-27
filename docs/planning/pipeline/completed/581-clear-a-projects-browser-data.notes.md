# Clear a project's Browser data — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-581-clear-a-projects-browser-data.md
- **Pipeline spec:** 581-clear-a-projects-browser-data.spec.md

## Phase 1 — Plan
- **Request:** TICKET-581, split from #507 at its promotion (2026-09-27): #507's D6 and REQ-006,
  Clear Browser Data for one project. Autonomous, in the run of Orca and Warp findings Chad asked
  to have built (2026-09-26).
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (cargo 1.98.1, the gate, e2e, shear, hooks
  wired, no active pipeline, the README marker, cargo idle, a clean tree at d9b895a342); recall
  ✓; mint ✓; the prior-art sweep ✓; the spec ✓; the design ✓.
- **Classification / tier:** feature, prong 3 (B7c). Marley crates and the e2e fixture only; no
  Zed path. Size S.
- **Recall (§18.3):**
  - AD-claude-507-a-chromium-and-a-profile-per-project-001: a profile per project under
    `browser/projects/<key>/`, the hub's browser per project; the clear works on that folder and
    that entry.
  - F-claude-507-a-stopped-unit-lost-the-cookie-set-before-the-stop-001 and
    PR-claude-close-chromium-over-cdp-before-stopping-its-unit-001: close Chromium over CDP
    before its unit stops. For a clear the cookies are deleted anyway, but a Chromium killed
    while writing its profile could hold files open as they go; `stop_chromium` is the path.
  - L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001 and
    L-claude-507-the-rails-project-row-is-its-header-001: the scenario right-clicks the header at
    y 95, moves with End and Up, and shoots the menu before Enter.
  - #550's close guard asks with `window.prompt` (`close_guard.rs:124`): Zed's renderer takes
    Enter for the active button, the first, and Escape for "Cancel"
    (`crates/ui_prompt/src/ui_prompt.rs:66`, `:70`).
  - Brain consultation 8b902af22b5c497bb50160729e81c492: nothing on this seam beyond #507's
    decision.
- **Discovery (re-verified at d9b895a342):**
  - `browser.rs`: `BrowserHub::stop_browser` removes a project's `ProjectBrowser`, forgets its
    pages through `forget_pages` (each `PageClosed` closes its tabs) and spawns
    `stop_chromium(profile, connection)`, which sends `Browser.close`, waits up to
    `CLOSE_WAIT`, runs `service::stop` and checks the unit is down; with no connection it
    connects through the profile's `DevToolsActivePort` (`try_connect`), which reaches a
    Chromium an earlier Marley left running. `start` awaits the legacy move's `Shared` task
    before it writes `project.json` and opens the browser. `close_tabs(target)` forgets and
    removes each tab of a page; `live_views` lists every tab, and each `BrowserView` carries its
    `project`. `init` registers `OpenBrowser` and `NewBrowserTab` on every workspace, and the
    Record This toast shows the toast pattern (`Toast::new(NotificationId::unique::<…>(), …)`
    through `Workspace::show_toast`).
  - `service.rs`: `project_dir_in`, `profile_in`, `write_project_file_in` (the file is kept),
    `stop`.
  - `marley_workbench.rs:71`: the `marley` actions, `OpenBrowser` and `NewBrowserTab` among
    them (`OpenBrowser`'s doc still says "starting Chromium the first time", from before #507).
  - `rail.rs`: `render_project_row`'s menu (`ContextMenu::build`): Move Project Up and Down,
    a separator, Remove Project (#507); the row holds `group.workspace`.
  - Zed: `Window::prompt(level, message, detail, answers, cx) -> oneshot::Receiver<usize>`
    (`crates/gpui/src/window.rs:6448`).
- **Decisions:** D1 to D7 in the spec.

### Design
- **`service.rs`.** `remove_profile_in(project_dir) -> anyhow::Result<()>`: `remove_dir_all` of
  `profile_in(project_dir)`; not found is removed already; any other error names the path.
- **The hub.** `stop_browser` splits: `forget_browser(key, cx) -> Option<Connection>` removes the
  project's browser and forgets its pages, answering its connection; `stop_browser` calls it,
  logs, and spawns `stop_chromium`. `clear_browser_data(project, cx) ->
  Shared<Task<Result<(), SharedString>>>` calls `forget_browser`, then in a task awaits
  `stop_chromium(profile, connection)` and `remove_profile_in` off the main thread, the error
  as text; the hub keeps it in `clears` by key until it ends, and `start` awaits a project's
  pending clear before anything else, as it awaits the legacy move, so a tab opened during a
  clear starts its Chromium on the new, empty profile.
- **The tabs.** `close_tabs` keeps its job through a shared `close_views(views, cx)`, which
  forgets each view's page (so its removal closes nothing) and removes it from its pane;
  `close_project_tabs(key, cx)` passes every tab of the project, a tab still waiting for its page
  included.
- **The ask and the report.** `clear_project_browser_data(workspace, window, cx)` reads the
  project from the workspace, asks with `window.prompt(PromptLevel::Warning, "Clear the browser
  data of <name>?", detail, &["Clear", "Cancel"])` (detail: its Browser tabs close, and every
  site in it signs out, its cookies, local storage and IndexedDB deleted). On Clear it closes the
  project's tabs, starts the hub's clear and, when that ends, shows a toast in the workspace:
  "Cleared the browser data of <name>." or "Could not clear the browser data of <name>:
  <why>". Anything but Clear changes nothing.
- **The action and the rail.** `marley::ClearProjectBrowserData` ("Clears the browser data of
  this project, after asking: its Browser tabs close and every site in it signs out"),
  registered in `browser::init` beside `OpenBrowser`. The rail's menu gains "Clear Browser
  Data…" after the separator, before Remove Project; its handler runs
  `clear_project_browser_data` in the row's workspace, so the prompt opens in the rail's window.
  `OpenBrowser`'s doc says a project's Chromium.
- **The fixture.** #507's `write_site` moves to `browser-fixture.sh` as `write_login_site <dir>`,
  so #507's scenario and this one share the login site.
- **File manifest.** Marley crates: `crates/marley_browser/src/service.rs`;
  `crates/marley_workbench/src/browser.rs`, `rail.rs`, `marley_workbench.rs`. Scripts:
  `script/e2e/browser-fixture.sh`, `script/e2e/507-browser-context-per-project.sh` (the moved
  site), `script/e2e/581-clear-a-projects-browser-data.sh`, `script/e2e/golden`. At Complete:
  `CHANGELOG.md`, `docs/marley/three-prong-plan.md` (the B7c row),
  `docs/marley_architecture/marley_browser.md`, `marley_workbench.md`.
- **Ledger rows.** None: no Zed path changes.

### E2E plan
`compositor sway`, the offline Chromium, the login site. Marley opens alpha, which signs in as
`alpha`; beta, handed over (#513), signs in as `beta` in one tab and opens a second on
`whoami.html`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | right-click beta's header (y 95), End, Up: Clear Browser Data… selected; Enter | `581-01-menu`, `581-02-asked` (the prompt names beta) |
| REQ-002 | Escape | `581-03-cancelled`: beta's tabs still there; the log: beta's unit active, its tab still `beta` |
| REQ-003 | the menu again, Enter on Clear | `581-04-cleared`: no Browser tab of beta, the toast; the log: beta's unit inactive, `profile/` gone, `project.json` kept |
| REQ-004 | `marley: new browser tab` in beta, `whoami.html` | `581-05-beta-signed-out`: none, none, none |
| REQ-005 | the same moment: alpha's row and a look at alpha's tab | `581-05-beta-signed-out`; the log: alpha's unit active, alpha's tab still `alpha` |
| REQ-006 | beta's folder made read-only, the clear again | `581-06-failed`: the toast says what could not be removed; the log: `project.json` kept; the folder writable again after |

Not reachable by a scenario: nothing; a Chromium an earlier Marley left running is the same
`stop_chromium` path as #507's legacy move, which #507's scenario runs.

### Risks
- A tab of the project opened while its clear runs: `start` waits for the project's clear, so
  the new Chromium opens on the new profile.
- The prompt asks in one window, and gpui shows no second prompt over the first there. A clear
  of a project whose clear still runs, asked from another window, gets the running clear back,
  and its toast reports that clear's outcome.
- A pick or a recording of a cleared page stays (Out); `browser_check_pick` on it answers that
  its page is gone.
- REQ-006's read-only folder must be made writable again, or the run's cleanup fails; the
  scenario's `teardown` does it before `browser_teardown`.

## Phase 2 — Code
- **Checklist** (no task tool): `service.rs` ✓; the hub's clear ✓; the tabs ✓; the ask and the
  toast ✓; the action ✓; the rail's entry ✓; the fixture ✓; fmt ✓; clippy with `--all-targets` ✓;
  the review ✓.
- **Built.** `service::remove_profile_in`. In `browser.rs`: `forget_browser` (out of
  `stop_browser`, which now calls it); `clear_browser_data`, a `Clearing`
  (`Shared<Task<Result<(), SharedString>>>`) that closes the Chromium through `stop_chromium`
  and removes the profile off the main thread, kept in `clears` until it ends; `start` waits for
  the project's `clearing` first; `close_views`, shared by `close_tabs` and the new
  `close_project_tabs`; `clear_project_browser_data`, which asks with `window.prompt` (Warning,
  Clear and Cancel), and on Clear closes the project's tabs and runs the clear from the app
  (`cx.spawn` on the workspace, outside the window's update, since each tab's pane is updated in
  its own window), then shows the toast; the action registered in `init`. The action
  `ClearProjectBrowserData` in `marley_workbench.rs`, and `OpenBrowser`'s doc names the
  project's Chromium. In `rail.rs` the menu moved to `project_context_menu` (clippy found
  `render_project_row` a line too long) and gained Clear Browser Data…, which runs the clear in
  the row's workspace. The login site moved to the fixture as `write_login_site`, and #507's
  scenario calls it.
- **Deviations from the plan, and why.**
  - `clears` is a `Vec` of key and clear, not a map: Zed's `map_lookup_then_insert` lint
    (gate:21) flags a `get` then an `insert` of the same key, and a `Vec` searched by key matches
    the hub's `browsers` for the few projects there are.
  - The clear's type is named `Clearing` (clippy's `type_complexity`).
  - The menu entry calls the clear in the row's workspace directly, not through the rail entity,
    which it does not need.
- **Review of the diff.** REQ-001 and REQ-002: the prompt names the project; any answer but the
  first (Clear) returns before anything changes, and Escape picks Cancel. REQ-003: the tabs close
  first (each forgets its page, so its removal closes nothing), then the hub forgets the browser
  and pages, closes Chromium over CDP, stops the unit, waits for it, and removes `profile/`,
  keeping `project.json`; the toast reports it. REQ-004: the project's next start waits for a
  running clear, then makes a new profile. REQ-005: only the project's key is touched. REQ-006: a
  removal error comes back as text to the toast. The prompt focuses itself when it opens
  (`PromptHandle::with_view`), so the closing menu, which refocuses only while it holds the
  focus, leaves it the keys. No entity is read while it is updated: the tabs close from
  `AsyncApp::update`, outside any window's update. No Zed path changed.

## Phase 3 — Test
- **Checklist** (no task tool): the scenario, a row per REQ ✓; the run ✓; every shot read ✓; the
  golden set with 581 added, 507 among it (the moved login site) ✓; the gate (below).
- **The scenario.** `script/e2e/581-clear-a-projects-browser-data.sh`, `compositor sway`, the
  offline Chromium and the fixture's login site (`write_login_site`, moved from #507's scenario).
  alpha signs in as `alpha`; beta, handed over (#513), signs in as `beta` and opens a second tab
  on `whoami.html`. `clear_menu` right-clicks beta's header at y 95 and presses End and Up. The
  tabs' titles through the stand-in agent's `browser_tabs`, the units through `systemctl`, and
  the folders through `find` carry the checks.
- **The run, green on the first try:** 16 checks pass. The shots, read:
  - `581-01-menu`: beta's project menu: Move Project Up greyed, Move Project Down, a separator,
    Clear Browser Data… selected, Remove Project; `581-02-asked`: Zed's prompt over the window,
    "Clear the browser data of beta?", the detail (its Browser tabs close, every site signs out,
    its cookies, local storage and IndexedDB deleted), Clear active above Cancel. REQ-001.
  - `581-03-cancelled`: after Escape, beta's two tabs still there, still `beta`; the log: beta's
    unit active, its profile there. REQ-002.
  - `581-04-cleared`: beta's pane holds only its terminal, the rail lists no page of beta and
    still alpha's page, and the toast says "Cleared the browser data of beta."; the log: beta's
    unit inactive, `profile/` gone, `project.json` kept, alpha's unit active. REQ-003.
  - `581-05-beta-signed-out`: beta's new tab reads none three times, a new profile made; alpha's
    row and tab still read `alpha`. REQ-004 and REQ-005.
  - `581-06-failed`: with beta's folder read-only, the clear closes beta's tab and stops its
    unit, and the toast says "Could not clear the browser data of beta: removing
    …/browser/projects/<key>/profile: Permission denied (os error 13)"; the log: the profile
    folder left, `project.json` kept, the unit inactive. REQ-006.
- **Focus.** A headless sway: "0 Marley windows before the run, 0 after; the run added no rule
  and did not reload it".
- **Not reachable by a scenario:** nothing beyond #507's; a Chromium an earlier Marley left
  running goes through `stop_chromium`'s own connection, which #507's legacy move exercises.
- **The golden set,** with 581 added: 30 of 30 pass, 507 among them on the fixture's
  `write_login_site`.
- **The gate:** `script/gates.sh --diff`, `GATE GREEN [diff]`, 16 of 16 on the first run (the
  `map_lookup_then_insert` lint, gate:21, was met in Code with the `Vec` of clears).
- **Verdict:** PASS.

## Phase 4 — Complete
- **Documented.** `CHANGELOG.md` (Added: clear a project's browser data);
  `docs/marley/three-prong-plan.md` (the B7c row, shipped);
  `docs/marley_architecture/marley_browser.md` (`remove_profile_in`); `marley_workbench.md` (the
  rail's menu, and Clear Browser Data: the ask, the tabs, the hub's `Clearing`, the toast). No
  path outside the Marley-owned set changed; gate:16 found every touchpoint recorded.
- **Knowledge appended:** L-claude-581-a-prompt-opened-from-a-context-menu-keeps-the-keys-001;
  AD-claude-581-clearing-a-projects-browser-data-deletes-its-profile-001. No `F-…`: Code and
  Test found no bug.
- **Brain:** consultation 8b902af22b5c497bb50160729e81c492 closed with
  `decisions/clear-browser-data-deletes-a-projects-profile-after-its-chromium-closes`.
- **Closed:** TICKET-581 to `tickets/closed/`; its backlog row left at promotion. The pair is
  archived to `completed/`.
