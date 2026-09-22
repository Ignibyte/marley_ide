---
pipeline_id: 784149f0-9731-441d-9d60-257ee8f9570d
ticket: forge#181 (67543670-020d-4a13-87e3-c95d97a9cd72) · local docs/planning/tickets/open/TICKET-181-agent-launcher.md
aar_id: 1d238685-5698-4714-982f-24c8723bff12
status: Phase 5 — Complete PASS
title: M12 — agent launch picker: ⌘⇧A opens a chooser (kind + initial prompt)
type: feature
milestone: M12 — The Agent Cockpit
references:
  - crates/marley_app/src/agent_launcher.rs (NEW, PURE: launchable_kinds + AgentLauncherState)
  - crates/marley_app/src/app.rs (SHIM: the overlay open/key/render + launch_agent(kind, prompt))
  - crates/marley_app/src/lib.rs (register the new module)
---

## Title
The "brain CONTROLS agents" slice: ⌘⇧A (and the 🧠 icon) opens a small picker — choose the agent KIND
(↑/↓, wrapping) and type an optional INITIAL PROMPT — then Enter splits a pane, launches that kind, and
delivers the prompt as the first send. Esc cancels. Today ⌘⇧A launches `claude` directly with no choice.

## Scope
### In
- PURE `agent_launcher.rs`: `launchable_kinds() -> &'static [AgentKind]` ([Claude, Codex], menu order);
  `AgentLauncherState { selected, prompt }` — `new`, `selected`, `prompt`, WRAPPING `move_up`/`move_down`
  (a small fixed list cycles), `selected_kind`, `push`/`backspace` (prompt only; the kind + prompt are
  independent — typing does NOT reset the kind, unlike PaletteState's filter).
- SHIM `app.rs`: a `agent_launcher: Option<AgentLauncherState>` field; the "new-agent" verb OPENS the picker
  (so both ⌘⇧A and the 🧠 icon open it) instead of launching directly; a key-router branch →
  `handle_launcher_key` (esc closes; enter launches + closes; ↑/↓ move; backspace + printable edit the
  prompt); the picker overlay render (kinds list, selected row highlighted, + the prompt line); a generalized
  `launch_agent(&mut self, kind, prompt)` doing the existing new-agent split + `launch_command(kind)` + the
  AgentRun tag + last_agent, and (if prompt non-empty) the first send.

### Out
- Fuzzy-filtering the kind list (it's 2 items — ↑/↓ is enough); persisting the last-chosen kind; a
  multi-line prompt; adding new AgentKinds (Claude + Codex only, per launch_command).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | `AgentLauncherState` move_up/move_down shall WRAP over launchable_kinds, and selected_kind shall map the index to the kind. | unit + mutation |
| REQ-002 | push/backspace shall edit the prompt buffer only (leaving the kind selection unchanged). | unit + mutation |
| REQ-003 | launchable_kinds shall list Claude then Codex (the launch_command-supported set). | unit |
| REQ-004 (visual) | WHEN ⌘⇧A is pressed, a picker overlay shall show; typing a prompt + Enter shall launch the selected kind in a split with the prompt delivered. | driven capture |
| REQ-005 | gate GREEN; the pure module cov/MSI 100. | gate |

## Phase Plan
P2 folded. P3 the pure module + the shim overlay/key/launch. P3.5 1-2 critics (the wrap arithmetic + the
empty-list/underflow guard; the prompt-delivery timing; the key router order; reuse of the new-agent split).
P4 unit + driven (typed) + gate. P5 docs.
