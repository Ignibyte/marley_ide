# TICKET-413 — LSP: drain every host's outcomes per pump tick (route by owning root)

- **Ticket:** LOCAL #413 (chore, M-unset)
- **Tags:** lsp, inspect-found, 332-followup
- **Created:** 2026-08-11
- **Status:** closed (2026-08-11, pipeline 413 complete — gate green, receipt written)

## Summary

Filed from #332 inspect (state critic, finding 2). `consume_lsp_responses` drains
`take_responses` ONLY for the active root (`app.rs` pump: one `consume` call with
`active_root`), while every host `drain()`s each tick — so a non-active
workspace's queued outcomes (Answered AND #332's Abandoned) defer unboundedly
until switch-back. Consequences: a backgrounded rename/resolve abandonment
flashes "Language server didn't respond" minutes late, at switch-back rather
than within the timeout window; answered outcomes likewise wait for focus; and
the ⌘T `workspace/symbol` fan-out (which SENDS to every Ready host) structurally
can't complete its merge for non-active roots — only the active host's answers
ever drain (pre-existing, adjacent).

THE BLAST RADIUS THAT MAKES THIS A TICKET, NOT A QUICK FIX: all 12 consumer
arms' guards read the ACTIVE editor state (focused uri, live buffer version,
active caret). Draining a non-active root's outcomes changes WHEN every arm
fires — e.g. a rename `WorkspaceEdit` for root A would apply while B is focused
(rename deliberately has no focused-file guard: it edits files by uri), instead
of on switch-back as today. Each of the 12 arms needs its own
non-active-root reasoning per
PR-claude-critic-shared-fix-safety-claim-needs-per-consumer-check-001 — that
per-consumer sweep IS the work. (#332's contained fix already stopped the worse
half: `on_connection_lost` no longer destroys a backgrounded root's queued
outcomes, so nothing is LOST anymore — only deferred.)

## Acceptance

Headline: every host's queued outcomes drain within one pump tick of arrival,
routed with the owning host's root; each of the 12 arms documents (and tests)
its non-active-root behavior; the symbol fan-in merges answers from every Ready
host. Full EARS at plan.
