# TICKET-548 — Jev through Cloudflare Workers AI, as a provider setting

- **Ticket:** LOCAL #548 (feature, prong 2, the System One layer's second provider)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/548-system-one-via-cloudflare.spec.md
- **Source ticket:** Chad, 2026-09-26: "lets add a ticket to add cloudflare as a configurable
  option at some point", after choosing TypeSafe's own API for Jev
  (`docs/planning/design-notes/jev-system-one-2026-09-25.md`, "Chad's answers")
- **Status:** closed

## Summary
Marley's System One layer (`marley_system_one`, not yet ticketed; the Jev note's use 0) calls Jev
through TypeSafe's own API, which keeps requests "for as long as necessary". Cloudflare Workers AI
resells the same model and states zero retention
(developers.cloudflare.com/ai/models/typesafe/jev/). This ticket adds it as a provider chosen in
`marley.system_one`, globally and per project, so a project holding other people's data can go
through Cloudflare while the rest go direct, with no code change. The key's source shows on the
settings page, never its value.

## Acceptance
With the provider set to Cloudflare for a project, Marley's System One questions for that project
go to Cloudflare's endpoint and every other project's to TypeSafe's; the call log names the
provider of each call; a missing or refused Cloudflare key reads as `Unavailable` with its reason,
as TypeSafe's does.
