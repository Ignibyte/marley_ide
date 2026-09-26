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
- **Discovery (at `1dfedc01aa`):** `browser.rs` `show_page` (sets `target`, never touches
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
