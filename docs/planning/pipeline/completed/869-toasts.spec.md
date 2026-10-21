---
pipeline_id: bc40791a-9ab5-4532-ba7c-6874565bb082
ticket: docs/planning/tickets/open/TICKET-869-toasts-and-prompts.md
status: Phase 4 — Complete PASS
title: Toasts as the Inbox board draws them
type: feature
slice: The Marley look, W3.4 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md, https://claude.ai/artifact/FHSqmm3AyGP4Dy7QuzBiFn]
---

## Title
In the Marley layout a toast (Zed's `Workspace::show_toast`, which Marley calls for most of its
notices) reads as the Inbox board draws it: a coloured dot, a title, the rest of the message as a
muted line under it, and its action.

## Scope
### In
- A Marley hook in `show_toast`: the message split at its first ". " or ": " into a title and a
  line; a dot red for a failure ("Could not …", "… failed …"), the accent otherwise.
- Zed's layout keeps Zed's toasts.

### Out (explicitly deferred)
- Notifications Marley builds as `MessageNotification`s itself (11) and app notifications (19):
  they keep their own content.
- The prompts (TICKET-870).

## Reference (§20)
Upstream Zed: `workspace::notifications` (`show_toast` builds a `MessageNotification` with the
message and an optional action; `MessageNotification` already draws a `content_icon` and a muted
`secondary_content`).

### Prior art
- The code we ship: `Workspace::show_toast` (notifications.rs:196), `MessageNotification`'s
  `content_icon`, `content_icon_color`, `secondary_content`; Marley's 43 `Toast::new` calls.
- Ledger: the Zed hook pattern of #834, #838, #839.
- The brain (consultation 05f29eab): nothing on this seam.

## UI proof
Scenario `script/e2e/869-toasts.sh` (sway): two toasts Marley raises on demand:
`marley: toggle dictation` with voice off ("Dictation is off. Turn it on in the Voice section of
the Marley settings.") and Panel Layout's notice.
- `869-01-night`: the dictation toast: an accent dot, "Dictation is off" over "Turn it on in the
  Voice section of the Marley settings." (REQ-001).
- `869-02-day`: Marley Day, the same toast (REQ-001).

## Locked-In Decisions
- D1 — The split is at the first ". " or ": "; a message with neither is a title alone.
- D2 — A failure's dot is red; every other toast's is the accent.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley shows a toast in the Marley layout, it shall draw a dot, the message's first sentence as its title and the rest as a muted line, with its action, in Marley Night and Marley Day. | `869-01-night`, `869-02-day` |

## Phase Plan
- **P1 Plan**, **P2 Code** (the hook, `toasts.rs`), **P3 Test**, **P4 Complete**.
