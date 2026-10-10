---
pipeline_id: 1f8d4a4e-8540-48ed-8c82-2db5899d83a1
ticket: docs/planning/tickets/open/TICKET-725-marleys-agents-get-rustal-ste.md
status: Phase 4 — Complete PASS
title: Marley's agents get the Rustal STE skill by default
type: feature
slice: agents in Marley; the skill is rustal-ste (Rusty's store, cb90144)
references:
  - docs/planning/pipeline/completed/709-load-the-shared-claude-code-plugin.spec.md
---

## Title
Zed's agent, Claude Code and Codex in Marley get `rustal-ste` with no setup, behind one setting.

## Scope
### In
- **The skill files in Marley:** `crates/marley_workbench/agent_skills/rustal-ste/`, a copy of
  Rusty's store folder (SKILL.md, references, scripts, LICENSE, SOURCE.md; not the examples),
  embedded with `include_str!`.
- **The setting:** `marley.rustal_ste_skill` (default `true`), on the Marley page's Agents
  section. Off leaves no trace: no built-in skill, no plugin folder in terminals, and Marley's
  Codex copy removed.
- **Zed's agent:** a built-in skill. Zed's `agent_skills` gains an additive
  `register_builtin_skill(name, content)` and `unregister_builtin_skill(name)`, read by
  `builtin_skills` and `builtin_skill_content`. Built-in skills don't prompt.
- **Claude Code:** a plugin folder, `<data>/claude-code/rustal-ste/<digest>/`, written read-only,
  holding `.claude-plugin/plugin.json` and `skills/rustal-ste/`.
  - Local Marley terminals load it through `CLAUDE_CODE_PLUGIN_DIRS`, beside the shared plugin
    (#709's identity plumbing, which now takes several folders).
  - The Agent Panel's Marley entry on Claude Code gets the same variable.
- **Codex:** `$CODEX_HOME/skills/rustal-ste/` (default `~/.codex/skills`), with a
  `.marley-owned` marker holding the digest.
  - Marley writes or refreshes the folder only when it is missing or carries the marker.
  - Off removes it only when the marker is there.

### Out (explicitly deferred)
- Syncing from Rusty's store at runtime. The copy follows the store by hand, recorded in
  SOURCE.md.
- Other agent CLIs (Gemini, OpenCode).

## Reference (§20)
Upstream Zed's agent skills (`crates/agent_skills`: `builtin_skills`, `builtin_skill_content`,
`SkillSource::BuiltIn`; docs/src/ai/skills.md). Claude Code's plugin folders
(`CLAUDE_CODE_PLUGIN_DIRS`, #709). Codex's skills folder (`~/.codex/skills`, as Omarchy's links
and Codex's `.system` skills show).

### Prior art
- #709's `shared_plugin.rs`: a digest folder, a read-only write, and a staged rename.
- Zed's built-in `create-skill`.

## UI proof
`script/e2e/725-marleys-agents-get-rustal-ste.sh`, under `compositor sway`, with `CODEX_HOME` in
`$E2E_WORK`.

Shots:
- `725-01-zed-skills`: the Agent Panel's skills (the Skills page or the `/` menu) lists
  `rustal-ste`.
- `725-02-terminal`: a new terminal's `echo $CLAUDE_CODE_PLUGIN_DIRS` names the rustal-ste
  plugin folder.
- `725-03-off`: with the setting off, the same command in a new terminal no longer names it.

Checks:
- `$CODEX_HOME/skills/rustal-ste/SKILL.md` exists with the marker, and is gone once the setting
  is off.
- A folder Marley didn't write is left alone.

## Locked-In Decisions
- **D1:** one setting for all three agents, on by default (Chad: "install it by default").
- **D2:** never write over or remove a skill folder Marley didn't write.
- **D3:** the store is the source; Marley carries a copy, and SOURCE.md names both.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.rustal_ste_skill` is on, Zed's agent shall list `rustal-ste` as a built-in skill. | Shot 725-01 |
| REQ-002 | WHILE it is on, a new local Marley terminal and the Marley entry on Claude Code shall load the rustal-ste plugin folder. | Shot 725-02 |
| REQ-003 | WHILE it is on, Codex's skills folder shall hold Marley's copy; when off, Marley shall remove only its own copy. | The checks |
| REQ-004 | WHEN it is turned off, no agent shall get the skill from Marley. | Shot 725-03 and the checks |
| REQ-005 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan.**
- **P2 Code:** the files, `agent_skills` hook, `rustal_skill.rs`, identity, assistant, settings;
  gate.
- **P3 Test:** the scenario.
- **P4 Complete.**
