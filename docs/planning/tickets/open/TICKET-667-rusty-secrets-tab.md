# TICKET-667 — The Secrets tab, behind Rusty's PIN

- **Ticket:** LOCAL #667 (feature, Rusty in Marley R8)
- **Owner:** claude-opus-5-5, 2026-10-06 (Chad: "we need to look into then adding the last things missing", after the R1 to R7b batch)
- **Pipeline doc:** (none yet)
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3, the slices table); the read-only survey of
  Rusty at `13249a8` and Marley on 2026-10-06
- **Status:** open

## Summary
Rusty keeps secrets in `~/.rusty/.secret` behind a PIN (`secret_list` names only; `secret_pin_status`, `secret_pin_set`, `secret_unlock` gives a token for `pin_timeout_minutes`, `secret_lock`, `secret_reveal { key, token }`, `secret_update { key, value, token }`, `secret_set`, `secret_delete`). This ticket adds a Secrets tab: the names, the lock (set, unlock, lock, change the PIN), and with the token Reveal, Copy, Replace and Delete; it locks on expiry and when Marley's window loses focus, as Rusty's app does. Two hazards come first: Zed's MCP client logs whole messages at trace level (`context_server/src/client.rs:282`, `:341`), so a PIN, a token or a revealed value could reach Zed.log, against R-D7; and Rusty's `secret_set` and `secret_delete` take no token. The first is Marley's to close in this ticket; the second goes to Rusty.

## Acceptance
Secret names list without a PIN; a value shows only after unlocking and is never written to a log; the tab locks on expiry and on losing focus.
