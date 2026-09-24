# Voice input through Voxtype — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-480-voice-input.md
- **Pipeline spec:** 480-voice-input.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23: "Voice activation which would be great to have".
- **What the exploration found.** Voxtype 1.0.1 is installed and its daemon runs
  (`systemctl --user is-active voxtype`: active), model `base.en`, mode `type`. Omarchy binds F9
  to push-to-talk and Super+Ctrl+X to toggle. `voxtype status --format json` printed
  `{"alt": "idle", "class": "idle", …}`.

## Phase 1 — Plan, at promotion (2026-09-23)
- **Recall.** The brain (consultation de4e9df0258d4c0289bb168b9fa10e95) returned only
  unrelated follow-ups. From the ledger: L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001
  (finding a program at startup is a lazy background future; `smol::unblock` and other foreign
  wakers break gpui's test scheduler in tests that do not allow parking).
- **Voxtype, re-verified without recording.** `voxtype record toggle` starts a recording, or
  stops one and transcribes it. `voxtype status --follow --format json` prints a line per
  change, `{"text": …, "alt": "idle", "class": "idle", "tooltip": …}`. The states, from
  Voxtype's own Quickshell reader (`/usr/share/voxtype/quickshell/voxtype-shared/StateReader.qml`):
  idle, recording, streaming, transcribing; the binary also has "stopped". Voxtype types the
  transcript into the focused window.
- **Seams.** A ui `Button`/`IconButton` with a click handler prevents the default on mouse
  down (`button_like.rs`), so the terminal view's root does not take focus from a click on the
  bar: the microphone focuses its terminal itself. `util::command::new_command` spawns with a
  piped stdout and `kill_on_drop` on both platforms.

### Design
- **`voice.rs`**, the Voxtype adapter. `Voice` (a global): the `voxtype` program, found on the
  PATH at `init` in a lazy background future; the `VoiceState` (idle, recording, transcribing)
  from the status's `class`, with streaming counted as recording and anything else as idle;
  and the follower, a task running `voxtype status --follow --format json` with
  `kill_on_drop`. `toggle(voxtype, workspace, cx)` makes sure the follower runs and runs
  `record toggle`, and a failure shows in the workspace.
- **When the follower starts.** The first time a microphone is drawn, deferred out of the
  render, so the state is right for a dictation started with Omarchy's keys as well as with a
  click; and on a click, if it has ended. It is never restarted by a render, so a status that
  exits at once cannot loop. When it ends, the state is idle. Tests that run `crate::init`
  draw no agent bar, so none of them starts Voxtype.
- **`agent_bar.rs`.** A microphone after Attach File: muted while idle, red while recording,
  yellow while transcribing, with a tooltip for each; a click focuses its terminal and toggles.
- **The crate root.** `claude_plugin::run` becomes `run_program`, shared by both adapters.
- **Files:** `voice.rs` and `voice_tests.rs` (new), `agent_bar.rs`, `agent_bar_tests.rs`,
  `claude_plugin.rs`, `marley_workbench.rs`. Marley only.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven (bar): with a `voxtype` the bar shows the microphone; with none it does not |
| 002 | driven (bar): a click runs `record toggle` (the fake's log) |
| 003 | driven (bar): drawn once, the follower runs; `recording`, `transcribing` and `idle` lines move the state; unit: the look of each state; unit: `class` to state |
| 004 | driven (bar): with another pane focused, a click on the microphone focuses the terminal |
| — | driven (adapter): a status that ends leaves the state idle and a toggle follows again; a failed toggle shows its error; `init` finds `voxtype` on the PATH |
| 005 | `just gate-diff` |

## Phase 2 — Code (2026-09-23)
- **Built.** `voice.rs`: `Voice` (a global: `voxtype`, `state`, whether the status runs, and
  the follower task, whose drop kills the status with `kill_on_drop`), `VoiceState`, `init`
  (the PATH lookup in a lazy background future), `follow_once_drawn` (defers `follow` the first
  time a microphone is drawn), `toggle` (follows if the status does not run, then
  `record toggle`, with a failure shown by `Workspace::show_error`), `follow`, `read_status` and
  `state_of`. `agent_bar.rs`: `microphone` after Attach File, and `microphone_look` (muted,
  red, yellow, a tooltip each). The click focuses the terminal, then toggles.
  `claude_plugin::run` became the crate root's `run_program`, named in its errors by the
  program's file name.
- **Deviation.** `init` does the lookup itself, and tests set the global with `Voice::new`, so
  a test can say "no Voxtype" without the real one being found on this box's PATH.
- **Review.** A render only defers; it starts nothing itself, and after a status ends, only a
  click starts another, so a status that exits at once cannot spawn one per frame (a test
  pins this). `follow` is called outside any entity update. No input reaches the process: its
  arguments are constants. No leaked process: after the crate's 173 tests no `tail -f` from
  the fake is left; the only `voxtype status` processes are Quickshell's, which pass
  `--extended`.

## Parked (2026-09-23)
- Chad, mid-Test: "We are removing unit tests from the workflow entirely with instead doing e2e
  visualization tests only. Please update the workflow accordingly. No more unit tests". He
  chose to keep the tests already in the tree, building but run by no gate, and scripted live
  captures as the e2e test. #483 changes the workflow first; #480 comes back after it, drops
  the tests written in this pipeline (never committed), and is verified by an e2e scenario.
- The seven negative checks had all failed as they should before the park; they belong to the
  tests that go.

## Resumed under #483 (2026-09-23)
- **The plan, amended.** The tests this pipeline wrote (`voice_tests.rs` and its additions to
  `agent_bar_tests.rs` and `marley_workbench_tests.rs`, never committed) go, and so does what
  only they used: `Voice::new` and the crate-visible `following`. The proof is the e2e
  scenario in the spec's UI proof. To drive the toggle without a click, `marley::ToggleDictation`
  (in `voice.rs`, registered on every workspace) runs the same `toggle`; without `voxtype` on
  the PATH it says so in the workspace. The three clippy findings the #483 gate met in this
  code (`missing_const_for_fn` on `microphone_look`, the long first doc paragraph of
  `follow_once_drawn`, `&mut AsyncApp` in `read_status`) are fixed here.

### E2E plan
| REQ | Shot |
|---|---|
| 001 | `480-01-idle`: the bar with a muted microphone; `480-05-no-voxtype` (a second run, no fake): the bar without one |
| 002 | `480-02-recording`: after `marley: toggle dictation`, the microphone red (the fake turned to recording on `record toggle`) |
| 003 | `480-02-recording` red, `480-03-transcribing` yellow, `480-04-idle-again` muted |
| 004 | not driven (a click): the handler's `window.focus` call is reviewed |
| 005 | `just gate-diff` |

## Phase 2 — Code, resumed (2026-09-23)
- **Changed since the park.** The tests are gone with what only they used (`Voice::new`, the
  crate-visible `following`). `voice::init` registers `marley::ToggleDictation` on every
  workspace: with `voxtype` found it runs `toggle`, and without it the workspace shows "Voxtype,
  which Marley dictates with, is not on the PATH". `microphone_look` is a `const fn`,
  `read_status` takes `&AsyncApp`, and `follow_once_drawn`'s doc has a short first paragraph.
  `just clippy marley_workbench` is clean on every target, the tests left in the tree included.
- **Review.** The action reads the global and then updates only through `toggle`, outside any
  entity update. The toggle from the palette runs where the palette gave the focus back, the
  terminal it opened from.

## Phase 3 — Test (2026-09-23)
- **E2E** (`SHOT_DIR=<scratchpad>/e2e480 just e2e script/e2e/480-voice-input.sh`; focus report:
  "the user's window and workspace are as they were"):
  - `480-01-idle`: the rail's row reads "Claude Code · working"; the bar reads "Claude Code",
    `+`, a grey microphone, "Enable Claude Code notifications", and at the right the scratch
    folder and branch `voice`. The fake's status said idle (REQ-001, REQ-003).
  - `480-02-recording`: after `marley: toggle dictation` from the palette, the palette closed
    and the microphone red: only the fake's `record toggle` writes "recording" (REQ-002,
    REQ-003).
  - `480-03-transcribing`: after the second toggle, yellow (REQ-003).
  - `480-04-idle-again`: grey, after the fake's delayed idle line (REQ-003).
- **Not driven.** The click, and so its `window.focus` call (REQ-004): a click would move
  Chad's pointer; the action runs the same `toggle`. "No microphone without Voxtype" (REQ-001's
  second half): this box has Voxtype in `/usr/bin`, which every usable PATH includes, so no
  scenario can hide it; the render returns early when the global's `voxtype` is `None`.
- **Gate.** `just gate-diff`: `GATE GREEN [diff]`, 15 gates and the receipt, which matches the
  tree.

## Phase 4 — Complete (2026-09-23)
- **Docs.** CHANGELOG (Added: voice input through Voxtype); `docs/marley_architecture/marley_workbench.md`
  (the Voice section, the bar's order, and the note that tickets add e2e scenarios, not tests);
  the plan's T7 row marks T7d shipped. No Zed path changed.
- **Knowledge.** AD-claude-480-marley-drives-voxtype-and-follows-its-status-001,
  L-claude-480-an-e2e-fake-acts-out-the-program-001. The seven negative checks of the first
  Test run went with the tests.
- **Brain.** Consultation de4e9df0258d4c0289bb168b9fa10e95 closed with the decision
  `marley-drives-voxtype-and-follows-its-status`, follow-up by 2026-10-07.
