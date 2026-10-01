# A restored Browser tab draws the page it reopens — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-578-restored-browser-tab-draws-its-reopened-page.md
- **Pipeline spec:** 578-restored-browser-tab-draws-its-reopened-page.spec.md

## Phase 1 — Plan
- **Request:** found in #576's Test (2026-09-26), queued first; Chad's goal of 2026-09-26, "lets
  continue completing tickets".
- **Classification / tier:** bug, prong 3. One Marley function. Size S.
- **Checklist (no TaskCreate in this harness):** all Plan steps done here.
- **Recall (§18.3):** #494's notes (`494-03-reopened` drew its pages because the scenario clicked a
  tab behind the one in front, whose first paint came after its page); #488's viewer rule (a tab
  streams its page from its first paint in front until it is deactivated). Brain consultation
  b0f5bbf415b84b14a59f263e9d018f63: nothing on this seam.
- **Discovery (at `9d9b8ffa66`):** `browser.rs` `show_page` (sets `target`, never touches
  `viewing`), `start_viewing` (returns early while `viewing`), `stop_viewing`, and
  `PageElement`'s paint, which calls `start_viewing` each paint.
- **Decisions:** D1 in the spec.

### Design
- **Approach.** `show_page` calls `self.stop_viewing(cx)` before it sets the new target:
  `remove_viewer` of the old page (none when the page is gone), `viewing` false; `cx.notify()`,
  already at its end, brings the paint that starts viewing the new page.
- **File manifest.** Marley: `crates/marley_workbench/src/browser.rs`;
  `script/e2e/578-restored-browser-tab-draws-its-reopened-page.sh` (Test).

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| — | the harness's `navigate` opens a Browser tab on a light page | `578-01-before` |
| REQ-001 | quit; Chromium's unit stopped; launch; the restored tab's page area sampled | `578-02-restored`; the pixel in the run log |
| — | the same on the build before (installed): red | the run log |
| REQ-002 | golden set with 578; the gate | `just regress`; `script/gates.sh --diff` |

## Phase 2 — Code
- **Built.** `show_page` calls `self.stop_viewing(cx)` first: the old page's viewer goes
  (`remove_viewer`, nothing to do for a page the browser no longer has) and `viewing` is false, so
  `PageElement`'s next paint, which `show_page`'s `cx.notify()` brings, starts viewing the new page.
- **Review:** every new page reaches a tab through `show_page` (`page_created`, and a waiting tab
  adopting a start's page); it runs inside the tab's own update, from `open_page_in`'s window
  update or the hub subscription's flush, never inside the hub's update, so updating the hub there
  is safe. A tab behind others does not paint and so views nothing, as before.
- **Checks:** `cargo fmt --check`, `cargo clippy -p marley_workbench --all-targets -- -D
  warnings`: clean.

## Phase 3 — Test (stopped: the diagnosis ruled out)
- **What the runs showed.** #578's own scenario (one project, a relaunch with Chromium stopped,
  the page area's pixel sampled with ImageMagick) passed on the build before the change as well as
  after it: the restored tab drew its page both times. #576's scenario, blank in its first two
  runs, drew in its next four: once with the change and three times without it (the change set
  aside, logs on `restore`, `show_page` and `start_viewing`).
- **The diagnosis, ruled out.** `render` gives `PageElement` a target only once its page is
  attached (`self.target.clone().filter(|_| attached)`), so a restoring tab never starts viewing
  its dead saved page; each logged run shows `show_page` with `viewing=false`, then
  `start_viewing` of the new page. The change to `show_page` was reverted: nothing showed it
  mattered.
- **Next:** repeat #576's flow with logs on the restored page's screencast (each frame and each
  acknowledgement, `sync_screencast`, the page's window size) until a run goes blank; the scenario
  below is the check once a fix exists.
- **The scenario, kept here until then** (`script/e2e/578-restored-browser-tab-draws-its-reopened-page.sh`
  as written in Test): a light page opened by the harness's stand-in, a quit with the Chromium unit
  stopped, a launch, and `page_drawn`, which reads the shot's pixel at the page's point with
  `magick <shot> -format '%[pixel:p{1100,500}]' info:` and wants each channel above 200.

## Phase 1 — Plan, again (promoted 2026-09-30)
- **Promoted** from `queued/` with its diagnosis ruled out (above).
- **Recall:** L-claude-499 (a screencast sends frames only when the page changes);
  L-claude-494 (a created page keeps the window of its first size). The brain (`rusty-cli brain
  ask`, consultation 9dbc534d68ac40a1ad75508492ee2855): nothing on this seam.
- **Reproduction attempts:** debug logs added to the hub (a stream's start and stop, a page's
  layout, each frame, a frame for no page shown); #576's scenario run with
  `ZED_LOG=info,marley_workbench=debug` (`zlog` takes a crate's name, not a module path; the first
  try with `marley_workbench::browser` logged nothing). Two runs under `just install`'s load, then
  eight logged runs: all ten drew page A, and every start got its frames (two each), none for no
  page shown.
- **A CDP probe** (Node's own WebSocket against `/usr/lib/chromium/chromium --headless`): on a
  page left still for three seconds, `Page.startScreencast` gave one frame 5 ms later and no more
  in three seconds; with the viewport set just before, one frame at 11 ms; at once after load, one
  at 17 ms. So a start on a still page draws once, and if that frame is lost nothing else comes.
- **The design:** a watchdog in `BrowserHub` (the spec's Scope). `PageState` gains `drawn` (a
  frame decoded since the stream last started) and `restarts`. `sync_screencast` resets `drawn` at
  a start and, after the start's call, waits `FIRST_FRAME_WAIT` (2 s) on the executor's timer,
  then calls `restart_undrawn`: if the page is still viewed, still streaming, undrawn and under
  `RESTARTS` (3), it logs at info, clears `screencasting`, stops the stream and syncs again.
  `show` sets `drawn` and resets `restarts` on a decoded frame.
- **Manifest:** `crates/marley_workbench/src/browser.rs` (Marley crate); the scenario
  `script/e2e/578-restored-browser-tab-draws-its-reopened-page.sh` (#576's flow, logged, with the
  pixel check).
- **Risks:** a slow machine's first frame after two seconds gets one needless restart, which
  costs one more frame.

## Phase 2 — Code (2026-09-30)
- **Built (`browser.rs`):** `FIRST_FRAME_WAIT` (2 s) and `UNDRAWN_RESTARTS` (3); `PageState`'s
  `screencasting: bool` became `stream: Stream` (`Off`, `Started`, `Drawing`), with `restarts`.
  `sync_screencast` sets `Started` at a start and, after the start's call, waits on the executor's
  timer and calls `restart_undrawn`, which logs at info and stops and starts a still-`Started`,
  still-viewed stream again. `show` moves `Started` to `Drawing` and resets `restarts` on a
  decoded frame. Plan's debug logs stay.
- **Deviation from the plan:** the plan named a `drawn` flag beside `screencasting`; clippy's
  `struct_excessive_bools` made the two one enum, which also says the three states plainly.
- **Review:** the watchdog's update and the restart's resync run in spawned tasks, outside any
  update of the hub; a stream stopped meanwhile (`Off`) or drawn (`Drawing`) is left alone; a page
  gone returns early.
- **Clippy found:** `struct_excessive_bools` (above) and `needless_pass_by_ref_mut` on
  `restart_undrawn`'s `cx`.
- **The scenario** `script/e2e/578-restored-browser-tab-draws-its-reopened-page.sh` sources #576's
  and adds the stream's logs and the pixel check (`page_drawn`); written before the gate.
- **Gate:** `just gate-diff` GREEN, 17 PASS.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/578-restored-browser-tab-draws-its-reopened-page.sh`, ten runs on the
  fixed build, one after another.
- **Results:** every run passed `page_drawn`: the third launch's restored tab at the page's point
  read `srgb(242,238,252)`, page A's background, in all ten. No run logged "drew nothing" (the
  watchdog never had to restart a stream) and none logged a frame for no page shown.
- **Shots read:** run 10's `576-03-a-again.png`: repo-a open, the Browser tab "Page a" in the
  right pane drawing "Page a" on its light background, its row in the rail. The third launch's
  log: the page laid out at 550x860, its stream started, two frames.
- **Not reached:** the watchdog's restart itself: no scenario makes Chromium withhold a first
  frame. It rests on the review, and on its info line in any run where it fires.
- **In all, 2026-09-30:** 26 runs of #576's flow (16 on the build before, 10 after), none blank.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md` (Fixed); `docs/marley_architecture/marley_browser.md` (the
  screencast paragraph).
- **Knowledge:** L-claude-578-a-screencast-start-sends-one-frame-and-a-lost-one-is-not-sent-again-001,
  L-claude-578-zlog-filters-take-a-crates-name-001. No `F-` block: no bug was found in this
  pipeline's code; the watchdog closes a class the review found.
- **Brain:** consultation 9dbc534d68ac40a1ad75508492ee2855 closed with `brain decide`.

