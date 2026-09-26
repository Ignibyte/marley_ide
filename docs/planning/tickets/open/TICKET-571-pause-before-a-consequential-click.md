# TICKET-571 — A pause before a consequential click in the Browser tab

- **Ticket:** LOCAL #571 (feature, prong 3 with prong 2's agent tools; use 6 of the System One layer, on #565)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/571-pause-before-a-consequential-click.spec.md
- **Source ticket:** Chad, 2026-09-26, approving the seven ranked uses of
  `docs/planning/design-notes/jev-system-one-2026-09-25.md` (use 6, "A pause before a
  consequential click") on the layer TICKET-565 builds, with his rules: "local first and then jev
  second", and "we need probably every aspect of this configurable and turned off / on where the
  system will use or wont use it. Otherwise this becomes a jev required system."
- **Status:** open

## Summary
An agent driving the Browser tab through `browser_click` can press Place order, Delete account or
Send as easily as Next. Claude Code asks before each MCP call, so its clicks have a first check; a
Zed thread with tool actions always allowed, a harness actor, or any client Marley cannot name has
none. This ticket holds such a click and asks in the Browser tab: a card naming the element and the
page, Allow (Enter) or Refuse (Esc), a workspace toast with Show, and a refusal the agent can read
after 25 seconds. Rules computed in code class the consequential clicks from the element's role and
name, its form and the page's URL: pays, deletes, sends in the user's name, changes an account. A
System One question goes through #565's layer only for what the rules leave open, and may add a
pause, never remove one. By default the pause applies only to agents running without their own
prompts; a setting widens it to every agent. Off by default as the use `click_consequence` in
`marley.system_one.uses`; the `rules` provider runs the classes with no model at all.

## Acceptance
With the pause set for all agents on the `rules` provider, the stand-in agent's click on Place
order shows the card and clicks nothing; Enter makes the click, Esc or 25 seconds answers the tool
with a refused result; a click on a plain link goes at once; on the `replay` provider in `act`, a
generic Continue on a page that says it will charge is paused too, and a no-signal reading lets the
click go; a page that changed under the pause is refused at Allow; other writes on the tab are
refused while the pause waits; the mode `off` clicks with no card.
