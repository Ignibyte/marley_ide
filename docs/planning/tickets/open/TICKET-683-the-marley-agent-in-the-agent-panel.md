# TICKET-683 — The Marley agent in the Agent Panel

- **Ticket:** LOCAL #683 (feature, prong 2 C)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** (set at promotion)
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 1, item 5
- **Status:** open

## Summary
A "Marley" entry in the Agent Panel: an agent that explains and configures Marley with
TICKET-681's and TICKET-682's tools and a short instructions text, and never edits code. Marley
finds what the user has, without touching a token: `claude auth status` (JSON, `loggedIn`),
`codex login status`, and the language-model providers Zed has configured. It uses the
`marley.assistant` setting's agent when set, else Claude Code, else Codex, else Zed's own agent.
Claude Code runs through the registry adapter with `_meta.claudeCode.options` carrying a
`systemPrompt` and `disallowedTools` (Edit, Write, NotebookEdit, Bash), which the adapter accepts
and Zed does not send today: a small hunk in `crates/agent_servers`, recorded in
`docs/marley/zed-touchpoints.md`. Codex runs in its read-only sandbox. Zed's agent gets a "Marley"
profile with no file tools. The layer ships off: the first time the Agent Panel opens with an agent
found, it offers the Marley agent once, and `marley.assistant` holds the switch.

## Acceptance
With Claude Code signed in, the Agent Panel offers the Marley agent once; turned on, a thread with
it answers from Marley's docs, proposes a setting change the user accepts as a diff, and cannot
edit a file or run a command. With Claude Code missing and Codex signed in, the same thread runs on
Codex read-only. Off, nothing of it shows.
