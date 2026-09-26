# TICKET-567 — `browser_find` and `terminal_find` for agents

- **Ticket:** LOCAL #567 (feature, prong 2 with prong 3; the Jev note's use 2, on #565's layer)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/567-find-tools.spec.md
- **Source ticket:** Chad, 2026-09-26, approving the seven ranked uses of
  `docs/planning/design-notes/jev-system-one-2026-09-25.md`; use 2, "`browser_find`,
  `terminal_find`", under his rules "local first and then jev second" and every use off by
  default with its own switch and mode
- **Status:** open

## Summary
An agent that wants one element of a page reads a whole `browser_snapshot` today (up to 30,000
characters, about 7,500 tokens), and one that wants one line of a command's output reads the
whole block. Two read tools take a query in words instead. `browser_find` takes a snapshot,
keeps its refs in the hub as the snapshot tool does, and answers with the ref that matches;
`terminal_find` answers with the block's line. Code matches first: a ref or a line holding
every word of the query answers alone; several matches are ranked by #565's layer among
themselves; no match sends the tagged lines (`e12| button "Sign in"`, `L0042| …`) as options
of a choice with `none`, plus a `present` noul, the pattern of TypeSafe's semantic-find
cookbook. With the layer down or the project unlisted the tools answer from the local match
alone and say so; with the use off they are not listed. Off by default; shadow answers locally
and logs the model; suggest returns candidates to verify; act returns the top ref.

## Acceptance
With the use in `act` and the layer on `replay`, `browser_find "sign in"` on the fixture page
answers the button's ref with no call, `browser_find "submit"` on a page with two submits
answers the replay's pick, a query nothing matches answers `sure: false` with
`browser_snapshot` as the next step, and `browser_click` on the answered ref clicks the element;
`terminal_find` finds the line of a 300-line block by its words and, for a query in other
words, by the replay's row; in `shadow` the answers come from the local match and the model's
row is logged; with the use `off` neither tool is listed and a call by name is refused.
