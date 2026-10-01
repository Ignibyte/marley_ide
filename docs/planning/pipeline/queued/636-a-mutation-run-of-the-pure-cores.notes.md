# A mutation run of the Marley crates' pure cores — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-636-a-mutation-run-of-the-pure-cores.md
- **Pipeline spec:** 636-a-mutation-run-of-the-pure-cores.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-01)
- **Request:** wave 5 (Chad, 2026-09-30); the workflow keeps mutation for the end.
- **Recall (§18.3):** the mutation decisions and failures named in the spec's Prior art; the
  shared target dir fills `/mnt/fast` (the memory's rule: drop Marley's old incremental variants
  when it reaches 100%).
