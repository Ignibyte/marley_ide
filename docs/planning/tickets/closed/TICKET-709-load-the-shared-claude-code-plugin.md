# TICKET-709 — Load the shared Claude Code plugin

- **Ticket:** LOCAL #709 (feature, prong 2 C1; design note B2)
- **Owner:** 4a72b24c-9078-4b13-afe3-4d0e692d70e0
- **Pipeline doc:** [709-load-the-shared-claude-code-plugin.spec.md](../../pipeline/completed/709-load-the-shared-claude-code-plugin.spec.md)
- **Source ticket:** #652's deferred "next slice, loading the shared plugin"; unblocked by rustal-harness TICKET-108 (2026-10-09), which Chad asked for first
- **Status:** closed

## Summary
#652 built Marley's half of the Claude Code plugin it shares with rustal-harness: `MARLEY_BIN`
takes the reports. rustal-harness TICKET-108 (closed 2026-10-09) gave the plugin Marley's host.
- **The plugin:** version 0.2.0, six files under `crates/harness-runtime/src/claude/plugin/`
  (`.claude-plugin/plugin.json`, `hooks/hooks.json`, `hooks/register.js`, `LICENSE-MIT`,
  `LICENSE-APACHE`, `claude-code-versions.json`), licensed MIT OR Apache-2.0.
- **Its digest,** as `rh` computes it: `83d0bb8f5cd3041483c73b92a9d09ca01f4c5bd8763f8cf65b8433a713b10303`.
  It covers each file's path and contents, each preceded by its byte length as a little-endian
  u64, in that order.
- **The host:** chosen at `session.start`. Interactive sessions only; the harness when `RH_BIN`
  and `RH_STATE` are set, else Marley when `MARLEY_BIN` and `MARLEY_TERMINAL_ID` are set.

This slice, as #652 planned it:
- carry the six files at that revision;
- write them by digest into Marley's data directory;
- put them at the front of `CLAUDE_CODE_PLUGIN_DIRS` in local terminals, behind
  `marley.claude_code_shared_plugin` (off by default), with #648's row for the mod
  (`claude_shared_plugin`, Claude Code from 2.1.287).

A Claude Code in a Marley terminal then reports its exact state (working, waiting on the user with
the tool, idle) to the rail.

## Acceptance
With the setting on, a Claude Code started in a Marley terminal reports `waiting` with the tool
while its permission dialog stands, and the rail's row shows it; with the setting off nothing
loads.
