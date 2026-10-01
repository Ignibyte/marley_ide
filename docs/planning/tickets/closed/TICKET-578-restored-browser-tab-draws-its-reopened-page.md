# TICKET-578 — A restored Browser tab sometimes draws nothing

- **Ticket:** LOCAL #578 (bug, prong 3, after #494)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/578-restored-browser-tab-draws-its-reopened-page.spec.md
- **Source ticket:** found in TICKET-576's Test, 2026-09-26 (the shot `576-03-a-again`)
- **Status:** closed

## Summary
In #576's scenario (three launches over two projects, Marley's Chromium stopped after each), the restored Browser tab of the third launch came back with its title, its URL and a loaded page per `browser_tabs`, but drew nothing, in the first two of six runs; the other four drew the page. The first guess, that the tab kept counting as a viewer of its dead saved page so `start_viewing` never ran for the page it reopened, is ruled out: `PageElement` takes a target only once its page is attached, and a logged run shows `show_page` finding `viewing` false and viewing then starting for the new page. The cause is still open; a reproduction that logs the screencast's frames and acknowledgements for the restored page, in a run that goes blank, is the next step. `script/e2e/578-…sh` (in the queued notes) checks the page area's pixel after a relaunch and passes on both builds so far.

## Acceptance
After Marley and its Chromium stop and Marley starts again, a restored Browser tab in front of its pane draws the page it reopened, in every run of a repeated scenario.
