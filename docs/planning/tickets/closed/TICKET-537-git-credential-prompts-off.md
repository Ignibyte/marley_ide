# TICKET-537 — Git credential prompts off for the agents Marley starts

- **Ticket:** LOCAL #537 (feature, prong 1 T7: the agent CLIs Marley starts in terminals)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/537-git-credential-prompts-off.spec.md
- **Source ticket:** Chad, 2026-09-25: specced at his request with every item decided that day (the brief quotes no words for this one). From the Orca survey: `docs/orca_architecture/02-worktrees-and-review.md` §3 item 8, with §2.13, and the README's "Smaller things worth a day each"
- **Status:** closed

## Summary
When an agent runs `git push` to an HTTPS remote that no credential helper answers for, git asks
for a username on the terminal. The question lands inside the agent's TUI, git waits for an
answer the agent cannot type, and the push hangs until someone looks. The terminals Marley opens
for agent CLIs, from the rail's Agent CLIs entries and the New Agent picker, start with
`GIT_TERMINAL_PROMPT=0` and `GCM_INTERACTIVE=never`, so git fails at once with "terminal prompts
disabled", which the agent can read and report. Stored credentials keep working, and a New
Terminal, or any shell the user opens, keeps git's prompts.

## Acceptance
In a terminal Marley started for an agent CLI, a `git push` that needs credentials fails at once
with git's "terminal prompts disabled"; in a New Terminal the same push asks for a username, as
before.
