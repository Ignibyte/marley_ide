# TICKET-757 — Claude Code's Remote Control switch

- **Ticket:** LOCAL #757 (feature; size small; shelved)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [757-claude-code-remote-control-switch.spec.md](../../pipeline/completed/757-claude-code-remote-control-switch.spec.md)
- **Source ticket:** [T3 Code survey](../../../t3code_architecture/README.md), item 13 and report 04 item 2; Chad, 2026-10-11: "16.) yes and shelve"
- **Status:** closed (built on Chad's word, 2026-10-12: "Build it now")

## Summary
Claude Code's `--remote-control` puts a session's prompts, approvals and pushes on the Claude phone
app. A setting, off by default, adds the flag to every Claude Code line Marley starts. A row whose
Claude Code runs with it shows a `remote` chip. Marley's own phone push (#535) skips that session,
so the phone does not ring twice. The session travels through Anthropic's service, which is why the
switch starts off.

## Acceptance
With the setting on, a Claude Code Marley starts runs with `--remote-control`, its row shows the
chip, and its events push nothing through Marley's ntfy; with it off, nothing changes.
