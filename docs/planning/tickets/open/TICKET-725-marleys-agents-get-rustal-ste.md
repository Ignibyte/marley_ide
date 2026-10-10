# TICKET-725 — Marley's agents get the Rustal STE skill by default

- **Ticket:** LOCAL #725 (feature; agents in Marley)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** (set at promotion)
- **Source ticket:** Chad, 2026-10-09: "for the skill lets adapt it and take it and use it for rustal
  specific instructions across agents", then "have the agents in zed install it by default as well".
- **Status:** open (waits on the skill: `rustal-ste`, adapted from ASD-STE100 with Rustal's glossary)

## Summary
Every agent Marley runs gets `rustal-ste` with no setup, behind a setting that is on by default
(off leaves no trace):
- **Zed's agent:** a built-in skill (`agent_skills::builtin_skills`, which do not prompt), through
  a small additive registration point in Zed's `agent_skills` so the content stays in a Marley
  crate.
- **Claude Code:** the skill in Marley's own plugin (`claude_plugin/marley/skills/`), for Claude
  Code in Marley's terminals and the Agent Panel's Marley entry; #724 brings the shared plugin's
  copy.
- **Codex:** a Marley-owned copy at `~/.codex/skills/rustal-ste`, written read-only, updated when
  Marley's copy changes and removed when the setting is off; never over a folder Marley did not
  write.

The skill ships in this public repository, so it holds terms and rules only: no hosts, addresses
or accounts.

## Acceptance
With the setting on, Zed's agent lists `rustal-ste` among its skills without a prompt, Claude Code
in a Marley terminal lists it, and Codex lists it; with it off, none does and the Codex folder is
gone.
