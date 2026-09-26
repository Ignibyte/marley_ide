# TICKET-572 — A running command's error: a notification when a dev server prints an error and keeps running

- **Ticket:** LOCAL #572 (feature, prong 1 T7b's follow-on, paired with #551; use 7 of the System One layer, on #565)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/queued/572-running-command-errors.spec.md
- **Source ticket:** Chad, 2026-09-26, approving the seven ranked uses of
  `docs/planning/design-notes/jev-system-one-2026-09-25.md` (use 7, "A running command's
  error", from the Warp second pass's finding 3, which #551 takes for a command's end and leaves
  this case to use 7) on the layer TICKET-565 builds, with his rules: "local first and then jev
  second", and "we need probably every aspect of this configurable and turned off / on where the
  system will use or wont use it. Otherwise this becomes a jev required system."
- **Status:** open

## Summary
#551 tells Chad when a long command ends. A dev server never ends: its useful signal is a failure
printed while it runs, and today nothing reads one. This ticket watches each plain terminal's
running block for new lines: the error shapes any build tool prints (`error:`, `error[E0308]`, a
Python traceback, `panicked at`, `EADDRINUSE`, `Failed to compile`) open a failure episode when the
block is still running five seconds later, and the recovery shapes (`Compiled successfully`,
`ready in`, `✓ built in`, `Finished`) close it. Each episode posts one desktop notification when the
terminal is not in front, and a second when it recovers; the rail row carries a red mark and the
first error line under #551's command line until then. A System One question goes through #565's
layer only for lines the shapes leave open (a bare stack frame, an "error" inside ordinary log
text), and it may mark, never more. Agent terminals and SSH keep their own banners. Off by default
as the use `running_error` in `marley.system_one.uses`; the `rules` provider runs the shapes with
no model at all.

## Acceptance
With the use on, a stand-in dev server that prints `error: Failed to compile` and keeps running
gets a notification and a red mark on its row, and `Compiled successfully` posts the recovery and
clears the mark; on the `replay` provider in `act`, a bare stack frame the shapes leave open is
flagged from the reading and a no-signal reading changes nothing; a command whose error is followed
by its exit within five seconds is left to #551; an agent terminal's `error:` is ignored; a focused
terminal gets the mark and no banner; the mode `off` does nothing.
