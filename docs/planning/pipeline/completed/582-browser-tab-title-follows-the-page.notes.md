# A Browser tab's title follows a title its page's script sets — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-582-browser-tab-title-follows-the-page.md
- **Pipeline spec:** 582-browser-tab-title-follows-the-page.spec.md

## Phase 1 — Plan
- **Request:** TICKET-582, found in #523's Plan (2026-09-27): the release build's golden set
  failed #507's and #581's scenarios on a title the login site set after an IndexedDB read.
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (no active pipeline, a clean tree at
  a34a188144; cargo busy with the release install, which Plan does not need); recall ✓; mint ✓;
  the prior-art sweep ✓; the spec ✓; the design ✓.
- **Classification / tier:** bug, prong 3. Marley crates and the fixture; no Zed path. Size S.
- **Recall (§18.3):**
  - L-claude-523-chromium-sends-no-event-for-a-scripts-title-001: the cause, and why the release
    build found it.
  - L-claude-495-a-listener-and-a-binding-catch-what-headless-chromium-hides-001: the binding,
    the world by name, the script for new documents and the world for the loaded one.
  - F-claude-506-a-text-count-read-every-elements-inner-text-001: what runs in the page on its
    events stays cheap (D2).
  - Brain consultation a976ff5031874ed991eafcfd7c7bf232: nothing on this seam.
- **Discovery (at a34a188144):**
  - `crates/marley_browser/src/select.rs:129` `watch_selects` and
    `crates/marley_browser/src/recorder.rs:689` `watch_actions`: `Runtime.addBinding`
    (`executionContextName`), `Page.addScriptToEvaluateOnNewDocument` (`worldName`),
    `Page.getFrameTree`, `Page.createIsolatedWorld` on the main frame, `Runtime.evaluate` there.
  - `crates/marley_workbench/src/browser.rs`: `follow_observed`'s `Runtime.bindingCalled` arm
    (3299), which sends `recorder::BINDING` to `action_reported` and the rest to
    `select_requested`; `attached`'s spawned watchers (`watch_selects`, `watch_actions`,
    `target_info`, `history`); `target_changed` (2828), which sets a page's title and URL and
    emits `PageInfoChanged`; `target_of_session`.
  - `script/e2e/browser-fixture.sh`: `write_login_site`'s `history.replaceState(… '#read')`.
- **Decisions:** D1 to D5 in the spec.

### Design
- **`title.rs`** (marley_browser). `WORLD` `marley-title`, `BINDING` `marleyTitle`, and
  `WATCHER`, a script a world runs once: in the top frame only (`window === window.top`), it keeps
  the last title it reported (the title at its start), and a `MutationObserver` over the document
  (`childList`, `subtree`, `characterData`) calls `report`, which sends `document.title`, cut at
  1,000 characters, when it differs; at `DOMContentLoaded` the observer moves to `document.head`
  and reports once. `Page::watch_title(session)`: the binding, the script for new documents, and
  a world in the main frame's loaded document, as `watch_actions` does.
- **The hub.** `attached`'s task calls `page.watch_title(page.session_id())` beside the other
  watchers. The `Runtime.bindingCalled` arm routes by name: `recorder::BINDING` to
  `action_reported`, `title::BINDING` to `title_reported`, the rest to `select_requested`.
  `title_reported(generation, session, params, cx)`: a current generation, the page of the
  session, the payload as the title; when it differs, the page's title changes and
  `PageInfoChanged` goes out.
- **The fixture.** `write_login_site`'s `whoami.html` sets its title and nothing more; its
  comment loses #582's note.
- **File manifest.** Marley crates: `crates/marley_browser/src/title.rs` (new),
  `marley_browser.rs`; `crates/marley_workbench/src/browser.rs`. Scripts:
  `script/e2e/browser-fixture.sh`, `script/e2e/582-browser-tab-title-follows-the-page.sh`,
  `script/e2e/golden`. At Complete: `CHANGELOG.md` (Fixed), `marley_browser.md`,
  `marley_workbench.md`.
- **Ledger rows.** None.

### E2E plan
`compositor sway`, the offline Chromium, a loopback site of three pages. Marley opens the
repository; `marley: open browser`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | Ctrl+L `timer.html`; settle five seconds | `582-01-timer`: the tab and its rail row read "Changed by a timer"; the log: `browser_tabs` |
| REQ-002 | Ctrl+L `timer.html?again`, then Ctrl+T, so the timer page's tab is behind; `browser_tabs` names "Timer page"; `stored.html` in the new tab; settle seven seconds | `582-02-behind`: the rail row of the tab behind reads the timer's title; the log: `browser_tabs` before and after the change |
| REQ-003 | the stored page, in front | `582-03-stored`: its tab reads "Read from storage: stored"; the log |
| REQ-004 | Ctrl+L `framed.html`; settle three seconds | the log: `browser_tabs` names "Framed page", not the iframe's "Inner title" |
| REQ-005 | the golden set, 507 and 581 among it, with the fixture's stand-in gone | the golden set's run |
| REQ-006 | Ctrl+L `half.html`, whose script sets `'Half \uD83D here'` a second after it loads; settle three seconds | the log: `browser_tabs` names `'Half � here'` |

Not reachable by a scenario: nothing.

### Risks
- A page that rewrites its title many times a second (a ticking clock in the title) sends a
  binding call each time; Marley redraws the tab each time, which a browser does too.
- The observer over the whole document runs only until `DOMContentLoaded`; a page that builds a
  large document before it pays for mutation records until then, as it would for any observer.

## Phase 2 — Code
- **Checklist** (no task tool): `title.rs` ✓; the crate's module and doc ✓; the hub's routing,
  `title_reported` and the watcher's start ✓; the fixture ✓ (after the release install's golden
  set, whose 507 and 581 runs sourced the old one); check, fmt, clippy ✓; the review ✓.
- **Built.** `crates/marley_browser/src/title.rs`: `WORLD` `marley-title`, `BINDING`
  `marleyTitle`, `WATCHER`, and `Page::watch_title(session)`, set up as `watch_actions` is.
  `marley_browser.rs`: the module and a line of the crate doc. `crates/marley_workbench/src/browser.rs`:
  `attached`'s task starts `watch_title` after `watch_actions`; the `Runtime.bindingCalled` arm
  matches the binding's name (`recorder::BINDING`, `title::BINDING`, else the select listener's
  as before); `title_reported` takes the payload as the title of the page whose own session sent
  it and, when it differs, sets it and emits `PageInfoChanged`, which the tab
  (`ItemEvent::UpdateTab`), the rail (`refresh`) and `browser_tabs` already follow.
  `script/e2e/browser-fixture.sh`: `whoami.html` sets its title and nothing more, and the header's
  note on `#read` is gone.
- **Deviations.** The watcher picks its start on `document.readyState` (`loading` watches the
  whole document until `DOMContentLoaded`) rather than on whether the head exists: a document
  still loading can hold its head before `DOMContentLoaded`, where the design moves to the head.
  D2's wording in the spec says so now. `title_reported` matches the page's own session rather
  than `target_of_session`, which also answers for a cross-site iframe's session (D3).
- **Review.**
  - A title cut at 1,000 UTF-16 units can end in half a surrogate pair, which cannot reach a
    Rust string: a CDP message that fails to parse is dropped with a warning (`cdp::dispatch`),
    so that title would be lost. The watcher drops a trailing high surrogate.
  - A page's own scripts cannot call `marleyTitle`: the binding lives in the `marley-title`
    world only, and in a same-site iframe's world the watcher returns before it reports.
  - Two watchers on one document (a navigation between the script's registration and the loaded
    document's world) report the same title twice; the hub changes nothing on the second.
  - Re-entrancy: `title_reported` runs inside the hub's update and only emits; the tab and the
    rail take the event after the update ends.
  - The title a script set before the watcher started comes from `target_info`, which the same
    task reads after the watcher: `Target.getTargetInfo` answers with the current title.
  - Provenance: Marley's own #495 pattern; no Zed path, nothing from Warp.
- **Checks** (after the release install's golden set, 31 of 31, installed a34a188144):
  `cargo check -p marley_browser -p marley_workbench` ✓; `cargo fmt --check` ✓;
  `./script/clippy -p marley_browser -p marley_workbench` (all targets, all features, deny
  warnings; cargo-shear, typos) ✓; `cargo dylint --all -- --all-targets -p marley_browser -p
  marley_workbench` ✓, no hit in either crate. The watcher parses (`node --check`).
- **A slip.** The first dylint run was piped into `head`, whose exit can end cargo with SIGPIPE
  partway; no cargo was left running, and the rerun into a log finished from its cache. Cargo's
  output goes to a log file and is read from there.

## Phase 3 — Test
- **Checklist** (no task tool): REQ-001 ✓; REQ-002 ✓; REQ-003 ✓; REQ-004 ✓; REQ-005 (507, 581,
  the golden set) ✓; REQ-006 ✓; the gate ✓.
- **A finding, fixed.** Reading the ledger while the golden set ran turned up
  L-claude-518-a-lone-surrogate-fails-the-whole-cdp-message-001, which Plan's recall had not
  found (it searched for titles, bindings and listeners). The watcher cut the title off a pair
  but sent a title that held half a pair of its own, so that report would be dropped. The watcher
  now makes the title well formed first (`toWellFormed`, guarded as `pick.rs`'s `cap` is); the
  spec gained D6 and REQ-006, the scenario `half.html` and its check.
  F-claude-582 and a PR block go to the ledger at Complete.
- **The scenario.** `script/e2e/582-browser-tab-title-follows-the-page.sh` (`compositor sway`,
  the offline Chromium, a loopback site of four pages). It differs from the plan's table in its
  timings and in one check: `timer.html` changes its title three seconds after it loads, and
  seven seconds with `?again`, so REQ-002 first reads `browser_tabs` while the tab is behind
  and still named "Timer page", then reads it again after the change. Settles: five seconds for
  REQ-001, seven after `stored.html` for REQ-002 and REQ-003, three for REQ-004.
- **Runs.** The first run (`e2e1.log`) passed every check, with the tab behind already changed
  when it was first read; the scenario then gained the check before the change. The second
  (`e2e2.log`) passed all seven checks. The golden set's run of 582, on the build from before
  the fix and with `half.html` added, failed only its new check: the tab stayed "Half page" and
  Marley's log has `browser: a DevTools message that is not JSON-RPC: unexpected end of hex
  escape`, the report dropped. The third run (`e2e3.log`), on the fixed build, passed all eight:
  - `browser_tabs names the timer's title`: `'Changed by a timer' at …/timer.html`.
  - `the timer page went behind with its first title`: `'Timer page' at …/timer.html?again`,
    beside the new tab in front, `about:blank`.
  - `the tab behind took its page's new title`: `'Changed by a timer again' at
    …/timer.html?again`, with `stored.html` in front.
  - `the stored page's title came from storage`: `'Read from storage: stored' at …/stored.html`.
  - `the framed page keeps its title`: `'Framed page' at …/framed.html`; `the iframe's title is
    nowhere`: no `Inner title` in the list.
  - `the half became a replacement character`: `'Half � here' at …/half.html` (REQ-006); no
    DevTools message dropped in Marley's log.
  - Marley's log: no line from the watcher; the one browser warning is the teardown's closed
    connection.
- **Shots** (scratchpad `582/shots`, from the third run; only Marley in each):
  - `582-01-timer`: the tab strip's Browser tab and its rail row under `repo` read "Changed by a
    timer"; the page, "A timer sets this page's title", at `timer.html` (REQ-001).
  - `582-02-behind`: the tab behind reads "Changed by a timer again" in the tab strip and in its
    rail row, `127.0.0.1:<port>` under it; the stored page is in front (REQ-002).
  - `582-03-stored`: the front tab and its rail row read "Read from storage: stored" over "Its
    title comes from storage" (REQ-003).
- **Focus.** Headless sway: `hyprland: 0 Marley windows before the run, 0 after; the run added
  no rule and did not reload it`; sway stopped with the run's Marley, pointer and keyboard.
- **REQ-005.** With the fixture's `#read` gone, 507 passed its 27 checks and 581 its 16, on the
  build before the fix and again on the fixed one; the login site's titles (`whoami:
  cookie=alpha local=alpha idb=alpha`, `…=beta`, `…=legacy`, `…=none`) came through the
  watcher, at `whoami.html` with no fragment.
- **The golden set** (32 with 582, `regress/20260927-060510`, the debug build before the fix):
  31 passed; 582 failed only the check the fix answers, above. The fix changes the watcher's
  script alone, so the scenarios that load pages whose titles scripts set (582, 507, 581) ran
  again on the fixed build and passed.
- **The gate.** `script/gates.sh --diff` after the fixed build's runs (log: scratchpad
  `582/gate.log`): rustfmt, clippy (every target), cargo-audit, cargo-deny, cargo-shear,
  gitleaks, shellcheck, no-suppressions, source-bans, docs, zed-ledger, manifests, spelling,
  semgrep and dylint all PASS; `GATE GREEN [diff]`, the receipt written.
- **Pre-existing:** nothing.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented.** `CHANGELOG.md` (Fixed: a Browser tab's title follows its page).
  `docs/marley_architecture/marley_browser.md`: a Titles section for `src/title.rs`.
  `docs/marley_architecture/marley_workbench.md`: the hub follows a title the page's script sets
  through `title_reported`, and `Runtime.bindingCalled` goes by the binding's name. The plan: no
  row, since a fix to a shipped slice carries none (#576 and #577 have none either). No path
  outside the Marley-owned set changed, so no ledger row.
- **Knowledge appended:** F-claude-582-a-title-watcher-cut-its-title-without-making-it-well-formed-001,
  PR-claude-a-string-a-page-script-hands-marley-is-made-well-formed-001,
  L-claude-582-cargo-piped-into-head-can-end-mid-build-001,
  AD-claude-582-a-watcher-in-the-page-reports-a-scripts-title-001. The brain: consultation
  a976ff5031874ed991eafcfd7c7bf232 closed with the decision
  `decisions/a-watcher-in-the-page-reports-a-title-its-script-sets-marley-582`, follow-up by
  2026-10-27.
- **Closed:** `tickets/closed/TICKET-582-browser-tab-title-follows-the-page.md`; `BACKLOG.md`
  had no row left (Plan removed it).
