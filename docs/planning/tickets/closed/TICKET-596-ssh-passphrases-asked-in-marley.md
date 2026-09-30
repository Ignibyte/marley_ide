# TICKET-596 — An agent's ssh asks for a key's passphrase in Marley

- **Ticket:** LOCAL #596 (feature, prong 1 T7: the agent CLIs Marley starts; split from #537)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** docs/planning/pipeline/completed/596-ssh-passphrases-asked-in-marley.spec.md
- **Source ticket:** docs/planning/pipeline/completed/537-git-credential-prompts-off.notes.md (Chad's answer, 2026-09-26)
- **Status:** closed

## Summary
#537 turns git's terminal prompts off in the terminals Marley opens for agent CLIs, but ssh asks
for a key's passphrase on the terminal itself, where an agent cannot answer and the user does not
look. Chad, 2026-09-26: "passphrases have always been a pain so either the agents need
passphrase-less or a way the user can type it in, preferably". Agent terminals get `SSH_ASKPASS`,
with `SSH_ASKPASS_REQUIRE=force`, pointing at a small Marley helper; the helper asks Marley, which
shows the prompt ssh gave in a dialog naming the agent's terminal, and hands the typed passphrase
back to ssh, never to a log, a file or the agent. Cancel fails the ssh command at once. A
passphrase-less key for agents stays the fallback.

## Acceptance
An agent's `git push` over ssh with a passphrase-protected key opens Marley's passphrase dialog
naming the terminal; the typed passphrase lets the push through, and Cancel fails it; no terminal
waits on ssh's own prompt; a plain terminal's ssh prompts as before. The full EARS criteria live
in the pipeline spec.
