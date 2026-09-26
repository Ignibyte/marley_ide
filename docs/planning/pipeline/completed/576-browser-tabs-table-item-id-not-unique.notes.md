# A Browser tab's saved row survives another workspace's tab of the same item id — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-576-browser-tabs-table-item-id-not-unique.md
- **Pipeline spec:** 576-browser-tabs-table-item-id-not-unique.spec.md

## Phase 1 — Plan
- **Request:** found in #575's Plan (2026-09-26) and queued first; Chad's goal of 2026-09-26, "lets
  continue completing tickets".
- **Classification / tier:** bug, prong 3. One Marley file. Size S.
- **Checklist (no TaskCreate in this harness):** all Plan steps done here.
- **Recall (§18.3):** L-claude-494-zed-item-ids-change-at-each-launch-001 (item ids repeat across
  launches); AD-claude-494-browser-tabs-reattach-or-reopen-001 (the tab's row holds its page, URL
  and title); #575's run showed restored items' entity ids nearly the same from run to run, so a
  collision across launches is likely in a scripted sequence. Brain consultation
  28b969832d7341bc987c3de50d763c16: nothing on this seam.
- **Discovery (at `474071d6a4`):** `crates/marley_workbench/src/browser.rs:5020-5038`
  (`MarleyBrowserTabsDb`, one migration with `item_id INTEGER UNIQUE`), `:5040-5063` (`save_tab`,
  `INSERT OR REPLACE`; `get_tab`), `:4966-4988` (`deserialize` fails with "no Browser tab was saved
  for the item" when the row is gone); `crates/terminal_view/src/persistence.rs:426-445` (Zed's
  rebuild).
- **Decisions:** D1 and D2 in the spec.

### Design
- **Approach.** `MIGRATIONS` gains a second `sql!` entry: `CREATE TABLE marley_browser_tabs2`
  (the same columns, `item_id INTEGER`, the same primary and foreign keys, `STRICT`), `INSERT INTO
  marley_browser_tabs2 SELECT … FROM marley_browser_tabs`, `DROP TABLE marley_browser_tabs`,
  `ALTER TABLE marley_browser_tabs2 RENAME TO marley_browser_tabs`.
- **File manifest.** Marley: `crates/marley_workbench/src/browser.rs`;
  `script/e2e/576-browser-tabs-table-item-id-not-unique.sh` (Test). Zed: none.

### E2E plan
Fixtures: an offline Chromium and a site with `a.html` and `b.html`; repo-a and repo-b, alike;
Chromium's unit stopped after each quit.

| REQ | Scenario part | Shot or log |
|---|---|---|
| — | launch on repo-a; the harness's `navigate a.html` opens a Browser tab; quit | `576-01-a` |
| — | launch on repo-b; `navigate b.html`; quit; the saved rows, read-only | `576-02-b`; the run log |
| REQ-001 | launch on repo-a; `browser_tabs`: a tab at a.html, `project repo-a` | `576-03-a-again`; the run log |
| REQ-002 | the table's schema and rows, read-only, after the new build's first launch | the run log |
| — | the same on the build before (installed, `d0939a6fc4`): red when the item ids meet | the run log |
| REQ-003 | the golden set with 576 added; the diff gate | `just regress`; `script/gates.sh --diff` |

### Risks
- The red needs the two tabs' item ids to meet, which startup's near-fixed order makes likely but
  not certain; the run log prints the ids. The fix holds either way.

## Phase 2 — Code
- **Built.** `MarleyBrowserTabsDb::MIGRATIONS` is an array of two: the first as it was, and a
  second that builds `marley_browser_tabs2` (the same columns, `item_id INTEGER`, the same primary
  and foreign keys, `STRICT`), copies the rows in, drops the old table and renames the new one.
- **Review:** the first migration is untouched but for its indentation inside the array; sqlez
  compares an applied migration's stored text with the current one after `sqlformat` formats
  both, and `sql!` builds its string from tokens, so the check holds. The rebuild is SQLite's
  documented way to drop a constraint, as Zed's `terminals` migration does. No Zed path.
- **Checks:** `cargo fmt --check`, `cargo clippy -p marley_workbench --all-targets -- -D
  warnings`: clean.

## Phase 3 — Test
- **Checklist (no TaskCreate in this harness):** 576-01 · 576-02 · REQ-001 · REQ-002 · the red on
  the old build · golden set · gate.
- **Scenario:** `script/e2e/576-browser-tabs-table-item-id-not-unique.sh` (`compositor sway`; an
  offline Chromium serving `a.html` and `b.html`; repo-a and repo-b). Three launches, Chromium's
  unit stopped after each quit: repo-a, where the harness's stand-in navigates to page A (a new
  Browser tab); repo-b (`open_path`, trusted), page B; then `collide` saves repo-b's tab again
  under repo-a's item id with the app's own `INSERT OR REPLACE`, on the run's copy of the profile
  while Marley is stopped (it does nothing when the ids met already); `saved_tabs` prints this
  run's workspaces' rows (a join on `workspaces.paths`, since the profile is a copy of the user's)
  and the table's schema, read-only; repo-a again, `browser_tabs`, a click on the tab, the shot.
- **Red on the build before the fix** (`E2E_BINARY=~/.local/bin/marley`, `d0939a6fc4`): the two
  tabs met on item id 12884902150 by themselves; after the second launch one row was left, repo-b's
  (`8|12884902150|…/b.html`), the schema still `item_id INTEGER UNIQUE`, and repo-a's tab did not
  come back: the bug.
- **Green on the fix:** the rows `7|12884902151|…/a.html` and `8|12884902151|…/b.html` side by
  side after `collide`, the schema rebuilt with `item_id INTEGER`; repo-a's tab back at `a.html`,
  `project repo-a`. Of two green runs, one had the ids one apart by themselves, which is why
  `collide` makes the meeting certain.
- **Shots, read:**
  - `576-01-a`: repo-a alone in the rail, a Browser tab `Page a` in a split beside its terminal,
    the Agent chip `went to …/a.html`.
  - `576-02-b`: the next launch, repo-b alone in the rail (a path given at launch opens that
    project only), a Browser tab `Page b`.
  - `576-03-a-again`: the third launch, repo-a with its Browser tab `Page a` back at `a.html`
    (the tooltip shows the URL). The page area is blank: a separate bug, TICKET-578 (a restored
    tab that paints before its page is back counts as the viewer of its old, dead page and never of
    the page it reopens; `browser_tabs` shows the page loaded).
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule
  and did not reload it".
- **Golden set:** 576 added; `just regress`: all 19 passed.
- **Gate:** `script/gates.sh --diff`: 16 passed, 0 failed, `GATE GREEN [diff]`, receipt written.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Fixed: a project's Browser tabs come back after another
  project's); `docs/marley_architecture/marley_workbench.md` ("Saved and restored": the table keyed
  by the pair, the second migration). No Zed path.
- **Knowledge appended:** F-claude-576-the-browser-tabs-table-kept-a-unique-item-id-001.
- **Follow-up minted:** TICKET-578 (a restored Browser tab in front of its pane draws nothing
  when its page is reopened), queued first.
- **Brain:** `rusty-cli brain decide 28b969832d7341bc987c3de50d763c16` →
  `decisions/marleys-browser-tabs-table-is-keyed-by-workspace-and-item-only`.
- **Ticket:** closed; the pipeline archived to `completed/`.
