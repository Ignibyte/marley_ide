# TICKET-750 — Codex resumes, and a cut-off turn continues

- **Ticket:** LOCAL #750 (feature, the finish line: restarts that lose nothing, first half)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [750-codex-resumes-and-a-cut-off-turn-continues.spec.md](../../pipeline/queued/750-codex-resumes-and-a-cut-off-turn-continues.spec.md)
- **Source ticket:** Chad, 2026-10-11, on the survey list's first item ("Restarts that lose
  nothing"): "1.) this is nice". The T3 Code survey's item 1 (`docs/t3code_architecture/README.md`),
  reports 01 (item 2) and 06 (item 5); its open question 3, default a chip. Plan:
  [the finish line](../../design-notes/finish-line-2026-10-11.md), Now, after the tabs batch
  (#743 to #749).
- **Status:** open

## Summary
#540 brings a Claude Code session back after a Marley restart and leaves Codex behind. A turn the
restart cut off comes back idle, with nothing to say so. With this ticket a Codex that ran on
Marley's own App Server (#650) comes back too, as `codex resume <thread id>` in its folder. A resumed
agent whose turn was still open when Marley stopped shows "Interrupted · Continue" on its rail row,
and Continue types "Continue where you left off." into it once it is at its prompt. Marley never
continues by itself.

## Acceptance
After a relaunch, a Codex terminal that ran on Marley's App Server runs `codex resume <id>` in its
folder; a Claude Code or Codex whose turn was open at the quit shows the chip on its row, and
Continue sends the prompt; one whose turn had ended shows no chip.
