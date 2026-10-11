# TICKET-775 — A quiet handed verb

- **Ticket:** LOCAL #775 (chore; size small)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [775's spec](../../pipeline/completed/775-a-quiet-handed-verb.spec.md)
- **Source ticket:** found in #749's visual check (2026-10-11): each `marley <verb>` typed in a
  terminal prints Zed's log line "starting zed version …" and Zed's "zed is already running" around
  #513's own line.
- **Status:** closed

## Summary
A second launch that hands its paths or a verb to the running Marley prints three things: the
logger's start line (the logger is set up before the single-instance check), #513's line saying
what it handed, and Zed's "zed is already running". In a terminal, after `marley agent-thread`,
only #513's line is useful. At planning: the verb is handed off in #749's intercept itself, before
Zed's `main` sets up its log, so a handed verb prints #513's line alone; a plain `marley <path>`
keeps Zed's output (its flags are Zed's to read).

## Acceptance
`marley agent-thread` typed in a Marley terminal prints one line, "Marley is already running on
…; it was asked to run `marley agent-thread`.", and nothing else.
