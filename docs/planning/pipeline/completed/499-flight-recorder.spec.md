---
pipeline_id: 3795148d-18ec-4308-85ed-86a6033510a0
ticket: docs/planning/tickets/open/TICKET-499-flight-recorder.md
status: Phase 4 — Complete PASS
title: "B5: Record what already happened in the Browser tab"
type: feature
slice: prong 3 B5 (wave 2)
references: [docs/marley/browser-handoff.md, docs/marley/three-prong-plan.md, docs/planning/pipeline/completed/492-browser-tools-for-agents.spec.md]
---

## Title
The flight recorder: each page keeps its last minute, and "Record this" saves it, so what Chad
just saw is already captured for the agent, with nothing typed and no secret in it.

## Scope
### In
- **The rolling minute**, per page, in memory, only while a tab draws the page: the input Marley
  sent (clicks with their place, keys by name, and typing as "typed N characters", never the
  characters), frames (at most two a second, as the JPEGs Chromium sent), console entries,
  requests and navigations (from #492's rings, through `redact_url`), and an accessibility
  snapshot at each navigation. Anything older than 60 seconds goes; the frames' bytes are capped.
- **Record this.** A toolbar button and `marley::RecordThis` save the page's minute, and a
  snapshot of the moment, as a recording in Marley's data directory
  (`<data dir>/browser/recordings/<id>/`: a timeline and the frames), never in a project; a
  toast names it. The tab shows that the recorder runs.
- **For agents.** `browser_recordings` (a read tool: each recording's id, page, time and length)
  and `browser_recording {id, frame?}` (the timeline, and a frame by its index as an image).

### Out (explicitly deferred)
- Replaying a recording in the tab; video; request bodies and headers; recording a page no tab
  draws; recordings pruned by age (they stay until removed by hand).

## Reference (§20)
Playwright's trace (actions, screenshots and network on one timeline, read in its viewer) and
the "record the last minute" of dash cameras and game capture, which `browser-handoff.md` pillar
C names: "a rolling 30–60 s trace of input, frames, console, network and AX snapshots; 'record
this' saves what already happened". Marley matches it with rings it already keeps and the input
it already forwards. Warp: N/A.

### Prior art
- **Published material.** Playwright's trace format (a timeline of actions with screenshots and
  network entries); CDP's `Page.screencastFrame` (the JPEG and its metadata), which Marley
  already receives.
- **Code we already ship.** #492's console and network rings with `redact_url`, `Page::
  accessibility_tree` and `snapshot::render` (which writes no values); #489's input path, where
  every click and key passes; #488's frames; `paths::data_dir`.
- **The security line (D15, `browser-handoff.md`):** "headers and tokens redacted from traces
  before they are written".

## UI proof
UI-AFFECTING. `script/e2e/499-flight-recorder.sh` (`compositor sway`, offline): a fixture sign-in
page that logs to the console and fetches with a token in its query. Steps: a click at a marked
spot, then a wait of more than a minute (the minute rolls past it); type an e-mail and a
password, click, scroll; Record this (`499-01-recorded`: the toast); the stand-in agent's
`browser_recordings` and `browser_recording` with a frame (the run log: the timeline's input as
counts and names, the console and the redacted request, a frame saved and shown); a grep of the
recording's files for the typed password, the e-mail and the token (the run log: none).

## Locked-In Decisions
- D1 — The minute lives in memory and reaches the disk only by Record this.
- D2 — Recordings go to Marley's data directory, never a project: the repository is public, and
  a recording holds what was on the screen.
- D3 — No typed character is ever recorded; keys by name, text by count.
- D4 — At most two frames a second, only while a tab draws the page.
- D5 — Frames are kept as the JPEGs Chromium sent, 16 MiB of them at most per page, the oldest
  going first; nothing is decoded for the recorder.
- D6 — A recording's id is the local time it was saved (`20260925-193045`), with a suffix when
  two land in one second; the tools reject any other name, so no id reaches outside the
  recordings' folder.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user runs Record this, Marley shall save the page's last minute of input, frames, console, requests and navigations as a recording and name it in a toast. | Shot `499-01-recorded`; the run log's files |
| REQ-002 | A recording shall hold no typed character, cookie, header or body, and its URLs shall be redacted. | The run log's grep |
| REQ-003 | WHEN an agent calls `browser_recordings` and `browser_recording`, the answers shall list the recordings and give one's timeline and a frame by its index. | The run log |
| REQ-004 | The recorder shall keep at most the last 60 seconds, and at most two frames a second, per page. | The run log's timeline |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design (the byte cap, the timeline's format).
- **P2 Code** — the rings, the input log, Record this, the two tools; fmt and clippy clean.
- **P3 Test** — the scenario, every shot; #489's and #492's scenarios again; the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
