---
pipeline_id: c607389c-0629-4d9f-b921-655acb525d3d
ticket: docs/planning/tickets/closed/TICKET-593-write-card-clear-of-its-toast.md
status: Phase 4 — Complete PASS
title: Keep the terminal write card's Allow and Deny clear of its toast
type: bug
slice: T-series, agent terminal writes (#525's follow-up)
references: [docs/planning/pipeline/completed/525-agent-drives-a-running-program.spec.md]
---

## Title
The card that asks before an agent types into a running program puts Allow and Deny at the left
of its row, where the toast that announces the same write does not cover them.

## Scope
### In
- `terminal_drive.rs` `footer`: the pending card's children reordered, Allow and Deny before the
  text.

### Out (explicitly deferred)
- The toast itself (Zed's `Toast`, drawn at the workspace's bottom right) and its Show.
- The drive bar after a write (its Take Over stays at the right): no toast shows then.
- A terminal pane narrower than the toast, in which no row placement escapes it.

## Reference (§20)
Upstream Zed: `workspace`'s notifications draw toasts in a stack at the workspace's bottom right
(`crates/workspace/src/notifications.rs`), over whatever is there. Marley keeps Zed's toast and
moves its own card's buttons out from under it.

### Prior art
- The code we ship: Zed's toast has no placement option, and a `Toast` carries one action, so
  Allow and Deny cannot move into it without a custom notification. The bottom left of a center
  terminal is never under the bottom-right stack while the pane is wider than a toast.
- #571's paused click shows its card in the Browser tab's own chrome, away from the toast, so
  it never met this.
- Behavior maps: nothing on this seam in `docs/zed_architecture/` or `docs/warp_architecture/`.

## UI proof
The scenario `script/e2e/525-agent-types-into-a-terminal.sh` (#525's visual check, under
`compositor sway`), a Python REPL and a stand-in agent through the plugin's bridge:
- `525-02-ask`: the card with Allow and Deny at the left of its row, the toast at the right.
- `525-03-typed`: after a click on Allow with the toast still up, `42` in the REPL.
- `525-08-denied`: after a click on Deny, the write refused and nothing typed.

## Locked-In Decisions
- D1 — Allow comes first, then Deny, then the text, which still truncates.
- D2 — The toast stays as it is: it tells a user looking elsewhere that a write waits.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a write waits for the user, the card shall show Allow and Deny at the left of its row, before the text. | Shot 525-02-ask |
| REQ-002 | WHILE the write's toast shows, a click on Allow shall type the write. | Shot 525-03-typed and the stand-in's answer |
| REQ-003 | WHILE the write's toast shows, a click on Deny shall refuse the write and type nothing. | Shot 525-08-denied and the stand-in's answer |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the reorder in `footer`; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — #525's scenario on the new build, every shot read.
- **P4 Complete** — CHANGELOG, the crate note, an `F-` block, close the ticket, archive, commit;
  #525's scenario and the fixture's commands go with #594.
