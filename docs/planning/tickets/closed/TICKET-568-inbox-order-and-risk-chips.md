# TICKET-568 — The approvals inbox: needs-you order and risk chips

- **Ticket:** LOCAL #568 (feature, prong 2 (attention); the Jev note's use 3, on #508's inbox and #565's layer)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/568-inbox-order-and-risk-chips.spec.md
- **Source ticket:** Chad, 2026-09-26, approving the seven ranked uses of
  `docs/planning/design-notes/jev-system-one-2026-09-25.md`; use 3, "Inbox order and risk
  chips", which "orders #508's inbox and marks entries; approves nothing", with Claude Code's own
  reviewer kept as the approver (his answer to the note's question 5) and his rules "local first
  and then jev second" and every use off by default with its own switch and mode
- **Status:** closed

## Summary
#508 lists every agent waiting on a permission at the top of the rail, oldest first, each entry
saying what it asks (`Bash: rm -rf build`). This use marks each entry with what the ask would do
and puts the entries that most need Chad first. Code classifies first, from the tool and its
preview against the project's folders: `destroys`, `outside project`, `sends out`,
`credentials`, `rewrites history`, `installs`, `claims approval`; each chip carries a level, and
the local order is by level, then age. #565's layer is asked only about entries code found
nothing on: a noul per chip class it may add, never remove, and an urgency score that orders
such entries among themselves. The state is the ask, the tool, the working directory relative to
the project and code's chips as facts, masked. Nothing here answers a prompt: Allow and Deny
stay Chad's clicks, and Claude Code's own reviewer keeps the verdict. Off by default; shadow
logs; suggest shows the model's chips with a question mark; act shows them plain and orders by
them.

## Acceptance
With the use in `act` and the layer on `replay`, four waiting entries show code's chips
(`rm -rf build` destroys, a Read of `~/.ssh/config` credentials, a `curl -X POST` sends out) and
sit above an `Edit README.md` with no chip, in level order then age; an entry code found nothing
on (`Bash: python3 scripts/cleanup.py`) gains `destroys` from the replay's row and moves above
the Edit; in `suggest` the model's chip reads `destroys?` and the order stays local; in `shadow`
no model chip shows and the Decisions view holds the row; in `off` the inbox is #508's, oldest
first with no chips; Allow and Deny work as #508 built them throughout.
