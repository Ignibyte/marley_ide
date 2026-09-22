---
pipeline_id: 0577cf42-6c08-4cd6-a8e3-c0f00d2c9806
ticket: forge#72 (a358706a-f0d8-4a21-b62f-e4a7dd3d5781) · local docs/planning/tickets/open/TICKET-072-send-to-agent.md
aar_id: 62e4730a-7fda-41cd-ba1a-66ad8dc0c8e1
status: Phase 5 — Complete PASS
title: send a line to a running agent
type: feature
milestone: M2.D
references:
  - crates/marley_agent/src/lib.rs (PURE: send_payload)
  - crates/marley_app/src/keymap.rs (cmd-shift-s → send-to-agent)
  - crates/marley_app/src/app.rs (SHIM: last_agent + the send dispatch)
---

## Title
cmd-shift-s sends the focused pane's composed prompt line to the last-launched agent (+ clears the
prompt) — the first CONTROL step: observe → act. Compose here, run it in the agent.

## Scope
### In
- PURE (`marley_agent`, cov/MSI 100): `send_payload(line: &str) -> Vec<u8>` = `"{line}\r"` bytes (\r =
  Enter for the running program).
- PURE (`keymap.rs`, cov/MSI 100): cmd-shift-s → `send-to-agent` + a keymap test.
- SHIM (`app.rs`, mutants::skip + cov-excluded): `RootView.last_agent: Option<PaneId>` (set in the
  `new-agent` dispatch to the launched pane); dispatch `send-to-agent`: collect the FOCUSED buffer's line,
  then (if `last_agent` present) `write_bytes(send_payload(line))` to that agent's session, then clear the
  focused prompt.

### Out
- A separate compose overlay (compose at the PROMPT — reuses the drivable cooked buffer). Broadcast to all
  (#73). Sending to a picked agent (uses the last-launched — a picker is later). Typing IN a focused agent
  (already reaches claude via #40).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — compose at the PROMPT (the cooked buffer, `state.buffer.text()`), NOT a key_char overlay — so the
  input is self-test-drivable (the terminal key path works; the finder's key_char doesn't).
- D2 — the write is a RAW `write_bytes` to the agent's PTY — CORRECT here (the agent runs claude,
  is_command_running) — the INVERSE of #59/#65's cooked-buffer rule (which is for a bare prompt).
- D3 — target = the last-launched agent (`last_agent`, set on cmd-shift-a). No agent → no-op.
- D4 — cmd-shift-s is free (no `"s"` binding, no hardcoded `key=="s"`).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `send_payload(line)` is called, it shall be `line` + a trailing `\r`; `""` → `"\r"`. | unit |
| REQ-002 | WHEN the keymap is queried, cmd-shift-s shall map to `send-to-agent`. | unit |
| REQ-003 (visual) | WHEN a line is composed at the prompt and cmd-shift-s is pressed, that line shall reach the last-launched agent (claude receives it) and the prompt shall clear. | self-test |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on send_payload + keymap; app shim excluded. | gate |

## Phase Plan
- **P2** — `send_payload` + the keymap + the last_agent/send-dispatch shim, mutation targets, test plan.
- **P3** — send_payload + keymap + the app.rs shim.
- **P3.5** — critic: send_payload, the keymap non-conflict, the raw-write-is-correct-here rationale, the
  borrow-safe collect-then-write-then-clear, last_agent lifecycle.
- **P4** — send_payload + keymap unit tests (cov/MSI 100) + the SELF-TEST (compose → cmd-shift-s → the
  agent receives + prompt clears) + gate GREEN.
- **P5** — docs, AAR, archive, close #72.
