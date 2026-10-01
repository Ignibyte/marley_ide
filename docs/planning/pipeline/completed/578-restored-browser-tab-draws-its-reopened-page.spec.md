---
pipeline_id: 3dd1a382-211c-4d51-b0c8-67c944f7eb25
ticket: docs/planning/tickets/open/TICKET-578-restored-browser-tab-draws-its-reopened-page.md
status: Phase 4 — Complete PASS
title: "A restored Browser tab sometimes draws nothing"
type: bug
slice: prong 3 (B1c, #494's restore)
references: [docs/planning/pipeline/completed/494-browser-restore.notes.md, docs/planning/pipeline/completed/576-browser-tabs-table-item-id-not-unique.notes.md]
---

## Title
A restored Browser tab sometimes drew nothing: its title, its URL and `browser_tabs` said the
page had loaded (2 of 6 runs of #576's scenario, 2026-09-26). The first diagnosis, a tab stuck
counting as its dead page's viewer, was ruled out in its Test phase; the blank has not come back
in 14 runs since (2026-09-30, 8 of them with the stream logged), and its cause is unproven.

What the code does allow is a page whose stream never draws: Chromium sends one frame when a
screencast starts, and on a still page none after it (L-claude-499, and a CDP probe on
2026-09-30: one frame 5 to 17 ms after the start, then nothing). A start that fails is only
logged, and the hub's `screencasting` flag stays on, so it is never asked again; a first frame
that fails to decode, or arrives for no page the hub shows, is the only one. Any of these leaves
the tab blank for good. The hub now watches for it: a viewed page whose stream has drawn nothing
two seconds after it started has its stream started again, up to three times.

## Scope
### In
- `crates/marley_workbench/src/browser.rs`, `BrowserHub`: each stream's start notes that nothing
  is drawn yet; a decoded frame notes that it drew; two seconds after a start, a page still viewed,
  still streaming and still undrawn has its stream stopped and started again, with an info log,
  at most three times until a frame draws.
- The debug logs added in Plan to find the blank: a stream's start and stop, a page's layout, each
  frame, and a frame for no page shown.

### Out (explicitly deferred)
- The rest of the restore.
- A cause for the 2026-09-26 blank beyond the class above, which a run that goes blank with these
  logs on would show.

## Reference (§20)
- **Upstream Zed:** N/A (the Browser tab is Marley's).
- **Warp:** N/A.
- **N/A — Marley-specific:** #488's hub streams a page while a tab draws it; the watchdog keeps
  that rule and only asks again when a stream that should draw has not.

### Prior art
- **Behavior maps.** None apply.
- **Published material.** CDP `Page.startScreencast`: frames come with `Page.screencastFrame` and
  each waits for `Page.screencastFrameAck`; a still page sends one frame at the start (the probe).
- **The code we already ship.** `sync_screencast`, `show`, `follow` in `browser.rs`; no crate owns
  a retry of a screencast.

## UI proof
`script/e2e/578-restored-browser-tab-draws-its-reopened-page.sh` (`compositor sway`; an offline
Chromium): #576's flow of three launches, Marley's Chromium stopped after each, with the stream
logged; the restored tab's page area sampled at the end (`page_drawn`, each channel above 200).
Run ten times on the fixed build:
- `576-03-a-again` in each run: the restored tab drawing page A;
- the logs: each start's frames, and whether the watchdog ever started a stream again.
No scenario can make Chromium withhold its first frame, so the watchdog's restart itself is
proven by review and by its log line in any run where it fires.

## Locked-In Decisions
- D1 — The fix is a watchdog on the stream, not a guess at the 2026-09-26 cause: it covers every
  way a first frame can be lost.
- D2 — Two seconds, three restarts until a frame draws: a first frame comes within 20 ms of a
  start (the probe), so two seconds is far past a slow one; three bounds a page that cannot draw.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a viewed page's stream has drawn no frame two seconds after it started, the hub shall start the stream again, up to three times until a frame draws. | Review; the info log line |
| REQ-002 | WHEN Marley and its Chromium stop and Marley starts again, the restored Browser tab in front of its pane shall draw the page it reopened, in each of ten runs. | Ten runs' `576-03-a-again` and their pixel |

## Phase Plan
- **P1 Plan:** the reproduction attempts, the probe, this spec.
- **P2 Code:** the watchdog; the logs kept; a review; the gate.
- **P3 Test:** the scenario ten times; every shot's pixel; the logs read.
- **P4 Complete:** CHANGELOG (Fixed), the crate note, knowledge, close, archive, commit.
