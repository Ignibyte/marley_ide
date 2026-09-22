---
pipeline_id: 7277eea3-d902-4010-bf5b-a39bb2d36489
ticket: forge#62 (2ee9f90a-67c0-4a1e-889c-f99a78419a5f) · local docs/planning/tickets/open/TICKET-062-agent-launch.md
aar_id: 8a763e09-847a-471f-af02-bddf152e6e65
status: Phase 5 — Complete PASS
title: launch + tag an agent pane
type: feature
milestone: M2.B
references:
  - crates/marley_agent/src/lib.rs (PURE: launch_command)
  - crates/marley_app/src/keymap.rs (cmd-shift-a → new-agent)
  - crates/marley_app/src/app.rs (SHIM: dispatch new-agent → split + run claude + tag)
  - crates/marley_app/Cargo.toml (add marley_agent dep)
---

## Title
cmd-shift-a launches an agent CLI (`claude`) in a new split pane, tagged as an `AgentRun` — Marley
starts + (later) observes an agent terminal.

## Scope
### In
- PURE (`marley_agent`, cov/MSI 100): `pub fn launch_command(kind: AgentKind) -> &'static str` —
  Claude→"claude", Codex→"codex" (the CLI to spawn; the inverse of `agent_kind_of`).
- PURE (`keymap.rs`, cov/MSI 100): `cmd-shift-a` → `"new-agent"` + a keymap test.
- SHIM (`app.rs`, mutants::skip + cov-excluded): `RootView.agents: HashMap<PaneId, AgentRun>` (the tag
  map). Dispatch `"new-agent"`: `split_focused(Horizontal, After, spawn_session)` → new PaneId (focused);
  write `"{launch_command}\r\n"` to the new pane's session (auto-run claude); tag `agents.insert(id,
  AgentRun::new(Claude, "claude"))`.
- `marley_app/Cargo.toml` += `marley_agent`.

### Out
- SURFACING the tag (an agent indicator / the cockpit view) — a later ticket. Observing the session
  delta → `agent_status_from` — later. Control (sending input to the agent). Configurable agent command.

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 (FORK) — agent CLI = `claude`; launch-AND-observe (auto-run). Configurable command + more kinds later.
- D2 — the tag lives in `RootView.agents` (a shim map keyed by PaneId), NOT threaded through the pure
  `workspace` module — keeps the pane-tree model unchanged; the tag is app state.
- D3 — cmd-shift-a is a free chord (cmd-a is unbound; cmd-shift-a distinct from the cockpit chords).
- D4 — auto-run via `write_bytes("claude\r\n")` (a RUN, so `\r\n`) to the new (focused) pane's session.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `launch_command(kind)` is called, it shall return the agent CLI program for that kind (`Claude`→"claude", `Codex`→"codex"). | unit |
| REQ-002 | WHEN the keymap is queried, `cmd-shift-a` shall map to `"new-agent"`. | unit |
| REQ-003 (visual) | WHEN cmd-shift-a is pressed, a new pane shall split and the agent CLI (`claude`) shall launch in it. | self-test (drive cmd-shift-a → capture the split + claude) |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on launch_command + keymap; app shim excluded. | gate |

## Phase Plan
- **P2** — `launch_command` + the keymap + the dispatch/split/run/tag shim, mutation targets, test plan.
- **P3** — launch_command + keymap + the app.rs new-agent shim + the Cargo dep.
- **P3.5** — critic: launch_command arms, the keymap non-conflict, the split→run→tag shim, mutants.
- **P4** — launch_command + keymap unit tests (cov/MSI 100) + the SELF-TEST (cmd-shift-a → split + claude)
  + gate GREEN.
- **P5** — docs, AAR, archive, close #62.
