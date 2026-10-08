# TICKET-693 — Any harness code passes through

- **Ticket:** LOCAL #693 (bug, from the harness's correction to #692, 2026-10-07)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** [693-any-harness-code-passes-through.spec.md](../../pipeline/completed/693-any-harness-code-passes-through.spec.md)
- **Source ticket:** TICKET-692
- **Status:** closed

## Summary
#692 passes the harness's refusal codes through from a fixed list of twelve, and the harness's
list has thirteen (`seat_surface` was missing). `seat start` also passes through the runtime's own
refusal names: `harness_*`, `operation_failed`, `admission_refused`. The harness asks Marley to
take any `[a-z_]+` token between `rh: ` and the next `:` as the code, and to show the rest as given.
A plain `rh: text` has no code, and exit 2 is a usage error from Marley's own arguments.

## Acceptance
- A refusal `rh: some_new_code: why` reaches the agent as `some_new_code` with `why`.
- `rh: state root is unavailable; start rh serve first` is `refused` with that text.
- An exit 2 reads as a bug in Marley's call.
