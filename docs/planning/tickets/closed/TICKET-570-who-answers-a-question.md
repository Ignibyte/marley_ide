# TICKET-570 — Who answers an agent's question: owner, manager, the agent proceeds, cannot tell

- **Ticket:** LOCAL #570 (feature, prong 2, C1's attention on #508's inbox with #568's chips; use 5 of the System One layer, on #565)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/570-who-answers-a-question.spec.md
- **Source ticket:** Chad, 2026-09-26, approving the seven ranked uses of
  `docs/planning/design-notes/jev-system-one-2026-09-25.md` (use 5, "Who answers a question") on
  the layer TICKET-565 builds, with his rules: "local first and then jev second", and "we need
  probably every aspect of this configurable and turned off / on where the system will use or
  wont use it. Otherwise this becomes a jev required system."
- **Status:** closed

## Summary
#508 gathers every agent's pending question into one list, and #568 marks each entry's risk and
orders by it. This ticket marks each entry with who should answer it: you (the owner), the
manager, the agent could proceed, or unclear. Rules computed in code class the owner's questions
first, from #568's chips (destroys, credentials, rewrites history, sends out, outside project) and
two facts of this ticket's (money, a message to people); a read-only tool inside the project is
marked as one the agent could proceed on. A System One choice (owner, manager, agent proceeds,
cannot tell) goes through #565's layer only for the rest. In `act` the route refines #568's order
within a level, the owner's first; in every mode Marley answers nothing. Routing a manager-class
question to rustal-harness's manager follows when M10 and #534 land; until then the mark is what
Chad checks against what he did. Off by default as the use `question_route` in
`marley.system_one.uses`; the `rules` provider runs the facts with no model at all.

## Acceptance
On the `rules` provider, an entry for `git push --force` reads `for you` and one for
`Read: src/lib.rs` reads `could proceed`, and the inbox keeps #568's order; on the `replay`
provider in `suggest`, an open entry shows the replayed class with a `?`; in `act` the entries
within a level order owner, unclear, manager, could proceed, oldest first within a class; a
`cannot_tell` reading marks `unclear`; nothing is ever answered by Marley; when Chad answers, the
call's outcome line says `owner`.
