# TICKET-565 — The System One layer: typed decisions, off by default

- **Ticket:** LOCAL #565 (feature, prong 2; the Jev note's use 0, the layer #566, #567, #568 and #548 build on)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/565-system-one-layer.spec.md
- **Source ticket:** Chad, 2026-09-26, approving the System One layer and all seven ranked uses of
  `docs/planning/design-notes/jev-system-one-2026-09-25.md`, with three rules: "local first and
  then jev second"; "we need probably every aspect of this configurable and turned off / on where
  the system will use or wont use it. Otherwise this becomes a jev required system"; TypeSafe
  direct first, and data leaves the box only for projects on an allow list
- **Status:** closed
- **As built:** six files in `marley_system_one`, not eight; the mode and provider enums only in `settings_content`; dynamic question options wait for #567, their first user; the palette names the view `marley: open decisions`. The live request with Chad's key did not run (no key at hand); his recorded Jev answers settled the shape.

## Summary
A `marley_system_one` crate asks a System One model (Jev, through TypeSafe's API, first) typed
questions about a state Marley builds from facts computed in code, and hands each use a reading it
may display, rank, route on or refuse with, never an approval. Providers: `typesafe`, `compatible`
(any server speaking the same `/v1/systemone` request), `rules` (the deterministic control arm)
and `replay` (recorded answers, which every use's e2e scenario runs on); Cloudflare Workers AI is
#548. Question sets are compiled in, named, versioned and insert-only, with a pinned model id;
the state is masked with #516's redactor before it leaves; every call, failures included, is a
JSON line under Marley's data directory, and a Decisions view lists the day's calls with the
day's spend against a budget. Settings under `marley.system_one` hold the switch, the provider
and endpoint, the model, the project allow list and the metadata-only list, each use's mode
(off, shadow, suggest, act) and the daily budget; the key comes from `MARLEY_SYSTEM_ONE_KEY` or
the keyring, its source shown and never its value. A `marley: system one check` command asks one
question about the last terminal so Chad can see what leaves and what comes back. Nothing on
screen changes for a user who leaves it off.

## Acceptance
With the layer off, Marley draws as before and no file appears under `<data>/system_one/`. With
it on and a project listed, the check sends the masked state to the configured provider with the
key as a bearer header, logs the call with its answer, latency and cost, and shows it in the
Decisions view; an unlisted project is refused before any request; a metadata-only project sends
code's facts alone; a slow, failing or unreachable provider reads as `Unavailable` with its reason,
a breaker opens after five failures in a row, and the daily budget stops calls when spent; the
`replay` provider answers from a file; the key's value appears in no log, row or page.
