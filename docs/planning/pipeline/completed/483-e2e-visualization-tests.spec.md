---
pipeline_id: cc10c971-c620-4320-bd12-8cc2d3d13f85
ticket: docs/planning/tickets/closed/TICKET-483-e2e-visualization-tests.md
status: Phase 4 — Complete PASS
title: E2E visualization tests replace unit tests
type: chore
slice: the workflow (CONSTITUTION §0, §3, §7)
references: []
---

## Title
The workflow's tests become e2e visualization tests: scripted runs of the real Marley, shot step
by step. No ticket writes or runs unit or driven tests.

## Scope
### In
- `script/gates.sh`: gate:3 (the test suites), gate:4 (coverage), gate:6 (miri) and gate:19
  (empty suites) retire, and with them the coverage floor, its exclude list and the mutation
  mask check; `--full` retires with the heavy gates. gate:2 keeps `--all-targets`, so the tests
  in the tree still build.
- `script/mutation.sh` retires: it runs the tests.
- `script/e2e.sh <scenario>`, the runner: the debug `marley` on a copy of the profile on hidden
  workspace 9; the scenario's `setup` builds fixtures before the launch and its `steps` run
  `settle`, `press`, `type_text` and `shot` after the window maps. Keys go to Marley's window
  only; each shot is the window by its toplevel. `script/live-shot.sh` becomes a one-shot
  scenario, `script/e2e/shot.sh`.
- `script/e2e/483-e2e-runner.sh`, the runner's own scenario.
- `enforce-tests-ran.sh` wants an e2e run at `/pipeline:test`; the owned-path set and the
  receipt's fingerprint take the new scripts.
- CONSTITUTION §0, §3 and §7; the phase commands, `/spec`, the templates, the justfile.

### Out (explicitly deferred)
- Deleting the tests already in the tree: Chad keeps them.
- A Linux headless renderer for gpui, for in-process screenshots with clicks.
- Mouse input: a click would move the user's pointer.

## Reference (§20)
- N/A — Marley-specific: this is Marley's own workflow. The runner builds on
  `script/live-shot.sh` (L-claude-467-capture-one-window-by-its-toplevel-001) and on Hyprland's
  `hl.dsp.send_key_state`, which Omarchy's own bindings use.

### Prior art
- **Code we already ship:** `script/live-shot.sh` launches and shoots without input. Zed's
  `crates/zed/src/visual_test_runner.rs` and gpui's `HeadlessAppContext::capture_screenshot`
  render real pixels on macOS only: `gpui_platform::current_headless_renderer` returns `None`
  elsewhere.
- **Published material:** Hyprland 0.56's Lua dispatchers (`/usr/share/hypr/stubs/hl.meta.lua`):
  `send_key_state { mods, key, state, window? }`; Omarchy's `bindings/clipboard.lua` sends down
  and up separately because `send_shortcut` can leave a key stuck.
- **Proven here:** a prototype sent Enter to the hidden Marley's window (`window =
  "address:0x…"`): its trust prompt closed, and the user's active window and workspace stayed
  as they were.

## UI proof
UI-AFFECTING in the tooling sense: the runner drives and shoots the real app.
- **E2E:** `script/e2e.sh script/e2e/483-e2e-runner.sh`: a scratch repository opens with its trust
  prompt (shot 1); Enter closes it (shot 2); `echo` and Enter typed into the terminal print (shot
  3). Every shot is read, and the runner reports whether the user's focus moved.

## Locked-In Decisions
- D1 — The tests in the tree stay and must build (clippy `--all-targets`); no gate runs them.
- D2 — The e2e test is a scripted live capture, read by the agent; shots never go in the repo.
- D3 — Keys only, to Marley's window by address; no mouse.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | The gate shall run no test suite, coverage, miri or empty-suite step, and shall still build every target in its scope | negative smokes: a failing test leaves the gate green; a test that does not compile turns gate:2 red |
| REQ-002 | WHEN a scenario runs, `script/e2e.sh` shall run the debug Marley hidden on workspace 9, write each shot to `SHOT_DIR`, and remove its profile copy and scratch folder | the e2e run; the files after it |
| REQ-003 | WHEN a step presses or types, the keys shall reach Marley's window and the user's active window and workspace shall stay as they were | the e2e run's shots and its focus report |
| REQ-004 | WHEN `/pipeline:test` ends with no e2e run in the transcript, the Stop hook shall block | negative smoke on a synthetic transcript |
| REQ-005 | The constitution and the phase commands shall name e2e visualization tests as the only tests a ticket writes and runs | review |
| REQ-006 | The gate shall be green, with shellcheck over the runner and its scenarios | `just gate-fast` |

## Phase Plan
- **P1 Plan** — this spec; the design in the notes.
- **P2 Code** — the gate, the runner and its scenarios, the hook, the docs.
- **P3 Test** — the negative smokes, the runner's own e2e run, the gate.
- **P4 Complete** — CHANGELOG, knowledge, the brain, close, archive, commit (only #483's files:
  #480's parked work stays out).
