# TICKET-718 — Failed checks to an agent, then a GitHub inbox

- **Ticket:** LOCAL #718 (feature; size medium, then large; agents together, after the finish)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [718-failed-checks-to-an-agent.spec.md](../../pipeline/queued/718-failed-checks-to-an-agent.spec.md)
- **Source ticket:** [monocode-findings.md](../../intake/monocode-findings.md), item 5; Chad, 2026-10-09: "lets queue up the MonoCode findings for potential future use"; Chad, 2026-10-11: "14.) ok"
- **Status:** open (deliberate: the agents-together batch, after the finish)

## Summary
#531's pull request chip already asks `gh` for a project's pull request every two minutes. The same
lookup now reads its checks, and the chip shows whether they failed, are pending or passed, with
the failed ones named in its tooltip. When a check failed, the chip's menu offers **Send Failed
Checks to Agent…**. Marley fetches the failed steps' logs of each GitHub Actions job, writes their
ends to one redacted file, and pastes into the chosen agent of that project a short prompt naming
the failed checks and the file. Enter is left to the person.

The GitHub inbox (issues and pull requests, with Start Work opening a worktree agent) is a later
slice.

## Acceptance
A project whose pull request has a failed check shows it on the chip; Send Failed Checks to Agent…
pastes into the chosen agent a prompt naming the check and a file holding its failed log, redacted,
unsent.
