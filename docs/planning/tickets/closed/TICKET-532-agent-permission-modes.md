# TICKET-532 — Agent permission modes: Claude Code's bypass or Codex's full access, as a setting

- **Ticket:** LOCAL #532 (feature, prong 2: starting agent CLIs; the Marley settings page)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/532-agent-permission-modes.spec.md
- **Source ticket:** Chad, 2026-09-25: "default to what the agents are doing but allow the dangerously bypass permissions config or codex full permissions." His answer to the Orca survey's open question 5, whose default was each CLI's own prompts (`docs/orca_architecture/README.md`, "Open questions for Chad"; report 01 §2.1 and §4).
- **Status:** closed

## Summary
Marley starts Claude Code and Codex with their own permission prompts, and there is no way to ask for anything else. A setting per agent, and per project in Chad's own settings, makes Marley start Claude Code with `--dangerously-skip-permissions` or Codex with its full access (`--sandbox danger-full-access --ask-for-approval never`). It is off by default, and a repository's `.zed/settings.json` cannot turn it on. Whenever an agent in a terminal runs with such a flag, however it was started, its rail row says so, so a bypass is never silent. The two defaults appear in the Marley settings page's Agents section.

## Acceptance
With nothing set, Marley starts both agents with no permission flag, even when the repository's own settings ask for bypass; with the setting on for a project, Claude Code starts with `--dangerously-skip-permissions`, and with it on for Codex, Codex starts with full access; a row running either reads "bypass" or "full access" in the warning color; the settings page shows both.
