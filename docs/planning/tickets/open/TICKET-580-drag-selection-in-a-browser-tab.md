# TICKET-580 — A drag in a Browser tab leaves no selection

- **Ticket:** LOCAL #580 (bug, prong 3, after #489)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet; `/pipeline:plan` mints the pair
- **Source ticket:** found in TICKET-518's Test, 2026-09-26
- **Status:** open

## Summary
In #518's scenario, in headless sway, a drag across a paragraph in a Browser tab leaves nothing
selected once the button is up. The drag was a press, moves of 60 px with a pause of 0.1 s after
each, and a release. While the button is down, the page's own log shows each `mousemove` with
`buttons` 1 and the selection growing ("Sele" after the first move). Right after the release,
`browser_look` reads an empty selection, and a pick after it has no `selected_text`. A faster drag,
a press and two long moves, selects nothing at all. A double click works: the word it selects
stays selected, #489 copies it with Ctrl+C, and #518's scenario uses one for that reason. The
cause is still open. It could be the release Marley sends, the focus handling after it, or how
`browser_look` reads the selection. The next step is a run of that drag that logs each
`Input.dispatchMouseEvent` Marley sends around the release (type, buttons, click count, point)
next to the page's selection after each one. A drag by hand in Chad's session would show whether
only the headless pointer does this.

## Acceptance
WHEN the user drags across text in a Browser tab and lets go, the page shall keep the selection
the drag made, as `browser_look` and a pick read it and as Ctrl+C copies it.
