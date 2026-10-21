---
pipeline_id: d34447ba-1810-4e1c-9749-5859f1601498
ticket: docs/planning/tickets/open/TICKET-871-trust-and-asking-notifications.md
status: Phase 4 — Complete PASS
title: The agents' questions as the boards draw them
type: feature
slice: The Marley look, W3.4 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
The questions agents put to the user through Marley's notifications (a settings or keymap change,
a harness seat, `agent_start` and Marley's other sensitive tools, a pairing, Claude Code's
folder-trust question) read as the Dialogs and MoreForms boards draw them: a title, the facts as
label and value rows, a muted footnote, and buttons in a row at the right by role, as #870's
prompts.

## Scope
### In
- One Marley card for every ask: `settings_change.rs` (also the seats' asks), `agent_control.rs`,
  `clients.rs`' pairing, `agent_trust.rs`' trust question.
- #870's button roles and drawing, shared.

### Out (explicitly deferred)
- Zed's trust modal (`SecurityModal`): TICKET-872.
- The notification's frame (Zed's `MessageNotification`, its close button and elevation): the
  card fills it.
- The other MoreForms forms (MCP server form, memory edit, new skill, pickers): their screens'
  W5 tickets.

## Reference (§20)
N/A — Marley-specific: the asks are Marley's own notifications (#692, #703, #811, #833), drawn
inside Zed's `MessageNotification` built with `new_from_builder`; the boards are Dialogs ("A
setting an agent proposes") and MoreForms ("An agent asks to start another agent").

### Prior art
- The code we ship: the four builders named above; #870's `prompts::render_button` and its
  `Role`; `MessageNotification::new_from_builder`.
- The brain (consultation 99b28d7a): nothing on this seam.

## UI proof
Scenario `script/e2e/871-asks.sh` (sway): a stand-in MCP client through the plugin's bridge (as
#491) calls `settings_change` and `agent_start`; each waits on the user.
- `871-01-setting`: the settings ask as a card: "e2e wants to change your settings", the change
  and the file as rows, Decline quiet, Apply primary at the right (REQ-001).
- `871-02-agent`: the `agent_start` ask: its title, Agent / Where / Prompt rows, Deny quiet and
  the allow buttons, the footnote (REQ-001).
- `871-03-day`: Marley Day, both cards (REQ-001).

## Locked-In Decisions
- D1 — The card draws in both layouts: the asks are Marley's own, with no Zed look to keep.
- D2 — Roles: a refusal (Decline, Deny, Refuse) is quiet, the choice that grants the least is
  primary, the others plain; buttons in a row at the right, the refusal first.
- D3 — The card is the notification's content; Zed's frame and close button stay.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an agent's question reaches the user through a notification, Marley shall draw it as a card: a title, its facts as label and value rows, a muted footnote, and its buttons by role in a row at the right, in Marley Night and Marley Day. | `871-01-setting`, `871-02-agent`, `871-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code** (`asks.rs` and the four builders), **P3 Test**, **P4 Complete**.
