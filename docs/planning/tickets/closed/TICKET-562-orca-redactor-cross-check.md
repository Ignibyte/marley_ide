# TICKET-562 — Orca's redactor as a cross-check for #516's rules

- **Ticket:** LOCAL #562 (chore, prong 2: the MCP server's tools, #516's redactor)
- **Owner:** spec-drafter-2026-09-26
- **Pipeline doc:** ../../pipeline/completed/562-orca-redactor-cross-check.spec.md
- **Source ticket:** The Orca second pass of 2026-09-25 (`docs/planning/design-notes/orca-second-pass-2026-09-25.md`), the five smaller details, item 3; Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Status:** closed

## Summary
#516's Test phase compared Marley's redactor with Orca's once, took two rules from it (a bare
token as a URL's userinfo, Slack's `xoxe-` and `xoxo-` prefixes) and left out its `.env` rule.
The comparison was done in a night and recorded in a paragraph. This chore does it rule by rule,
writes the table down, and adds what Marley still lacks: an `Authorization` header whose scheme
is `Basic`, `Token` or `Digest` (today only the `Bearer` form is hidden, and `Basic dXNl…` leaks
its credential), hyphenated secret names (`x-api-key`, `private-key`, `access-key`, which the
`API_?KEY` alternation does not match), and the labels `authorization`, `bearer`, `privkey` and
`cookie`. The `.env` rule stays out, for #516's reason: `env` output would lose `PATH`. Every
kind Marley hides today keeps its marker, so #516's golden scenario stays true.

## Acceptance
The spec's table names every Orca rule with Marley's answer (same, wider, added here, or left
out with the reason); a terminal block holding an `Authorization: Basic` header, an
`x-api-key` header and a `cookie=` assignment comes back to the stand-in agent with each value
as `[redacted: <kind>]`; `516-secret-redaction-for-agents.sh` passes unchanged; the gate is
green.
