# TICKET-435 — Run blocks: rerun + block-scoped jump-to-failure

- **Ticket:** LOCAL #435 (feature, M33)
- **Tags:** fusion, runnables, terminal-blocks, diagnostics, navigation
- **Created:** 2026-08-15
- **Provenance:** Phase C thread 2 (roadmap: "run a task → jump to the
  failure line"; per-block rerun); depends on #434's run-block identity;
  shelf: ../../design-notes/m33-tail-and-wedge-shelf.md
- **Pipeline doc:** ../../pipeline/active/435-run-block-rerun-jump-to-failure.spec.md
- **Status:** open

## Summary
The wedge's payoff loop: a #434 run Block carries its runnable identity, so
(a) a per-block RERUN affordance re-spawns the same command as a fresh
Block, and (b) a FAILED run block's file:line refs (the #196/#289 fold)
get a one-keystroke block-scoped jump-to-failure — first failing ref opens
in the editor at the line (NavStack push), cycling through the block's refs
on repeat. Edit → rerun → jump becomes one tight loop without leaving the
keyboard.

## Acceptance
Headline: a failing test block shows rerun + jump affordances; the jump
opens the failing file:line; after an edit, rerun spawns a fresh block and
a passing rerun clears the jump state. Full EARS in the queued spec.

## Follow-up (recorded at close)
Re-verify the literal Terminal-F8 keypress on the LIVE app on an idle
machine: at #435's validate, synthetic unmodified keys did not deliver
machine-wide (another interactive session owned focus; typing probes were
stopped as cross-session-injection risk). The binding, scoping, ladder
dispatch, and the handler's full behavior are unit/drive-pinned; the marker +
spawn + correlation were live-verified. Only the physical F8 leg remains.
