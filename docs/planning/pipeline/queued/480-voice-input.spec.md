---
pipeline_id: ff8d872f-595b-4f4c-909c-d8c11f021c3d
ticket: docs/planning/tickets/open/TICKET-480-voice-input.md
status: PARKED — Phase 2 Code PASS; Phase 3 waits for #483 (e2e visualization tests replace unit tests)
title: Voice input through Voxtype
type: feature
slice: prong 1 T7d
references: [docs/planning/pipeline/queued/477-agent-bar.spec.md]
---

## Title
A microphone button in the agent bar that drives Voxtype, the dictation daemon Omarchy ships.

## Scope
### In
- A microphone button in the agent bar (#477), shown when `voxtype` is on the PATH. A click runs
  `voxtype record toggle`, in an adapter that spawns it.
- The button follows `voxtype status --follow --format json` (its `class`: idle, recording or
  transcribing, as Voxtype reports them) and shows recording and transcribing.
- The terminal keeps the focus, so Voxtype's typing lands in it.

### Out (explicitly deferred)
- Marley capturing audio or running a model itself; hosted transcription.
- Key bindings of Marley's own: Omarchy binds Voxtype's keys system-wide.

## Reference (§20)
- **Warp:** the microphone in the input and the agent toolbelt; "Warp transcribes your speech
  and places the text in the input field"
  (https://docs.warp.dev/guides/agent-workflows/how-to-use-voice-and-images-to-prompt-coding-agents/).
  Warp records locally and transcribes through a hosted service
  (`docs/warp_architecture/crates/voice_input.md`); Marley does neither. No Warp code.
- **Upstream Zed:** none; Zed's microphone code is for calls.

### Prior art
- **Published material:** Voxtype 1.0.1's `record` (start, stop, toggle, cancel) and `status`
  (`--follow`, `--format json`); Omarchy's bindings for it; Claude Code's own dictation (hold
  Space, after `/voice`).
- **Code we already ship:** none for speech; process spawns go through `smol::process::Command`
  (Zed's clippy bans `std::process::Command::spawn`).

## UI proof
UI-AFFECTING: a button and its state.
- **Driven tests** (`marley_workbench`): with a fake `voxtype` on a scratch PATH that logs its
  arguments and prints status lines, the button shows; a click runs `record toggle`; a
  `recording` line turns the button red; without `voxtype` there is no button; the terminal
  keeps the focus.
- **Live drive:** the button in a capture; speaking needs Chad, so no dictation is driven.

## Locked-In Decisions
- D1 — Voxtype, local, over a hosted service: it is already on the box, private, and needs no
  keys.
- D2 — Marley drives the daemon and does not touch the audio.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE `voxtype` is on the PATH, the agent bar shall show the microphone button, and elsewhere none | driven |
| REQ-002 | WHEN the button is clicked, Marley shall run `voxtype record toggle` | driven |
| REQ-003 | WHILE Voxtype records or transcribes, the button shall show that state | driven |
| REQ-004 | WHEN the button is clicked, the focus shall stay in the terminal | driven |
| REQ-005 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; promotion confirms Voxtype's status classes and asks the brain.
- **P2 Code** — the button, the spawn adapter, the status follower.
- **P3 Test** — driven tests, negative checks, a capture, the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
