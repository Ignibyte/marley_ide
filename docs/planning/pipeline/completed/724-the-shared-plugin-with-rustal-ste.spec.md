---
pipeline_id: 90a09179-e324-44ce-97d1-e1a81efce4cb
ticket: docs/planning/tickets/open/TICKET-724-the-shared-plugin-with-ste100.md
status: Phase 4 — Complete PASS
title: The shared plugin with rustal-ste
type: chore
slice: #709's carry of rustal-harness's plugin, on harness TICKET-115
references:
  - docs/planning/pipeline/completed/709-load-the-shared-claude-code-plugin.spec.md
  - docs/planning/pipeline/completed/725-marleys-agents-get-rustal-ste.spec.md
---

## Title
Marley carries the shared Claude Code plugin at harness TICKET-115 (`cda7c24`). That is 16 files,
version 0.3.0, with the `rustal-ste` skill, at the digest
`eebd8a515080bb5c301f40121fe9ba0de41cd4b5afb11eb8c7adabecb213b1d0`.

## Scope
### In
- **The files:** `crates/marley_workbench/claude_shared_plugin/` takes the harness's
  `crates/harness-runtime/src/claude/plugin/` at `cda7c24`. `shared_plugin::FILES` lists the 16
  in the harness's order, so Marley's digest is the harness's.
- **One copy of the skill per terminal:** while the shared plugin is wanted, terminals load it
  alone, and its `rustal-ste` is the one they get. #725's skill folder steps aside, so a Claude
  Code doesn't list the skill twice. The Marley entry keeps #725's folder.
- **The docs:** #709's lines in the guide and the architecture note.

### Out (explicitly deferred)
- The harness's limit that Claude Code attaches skill listings only to typed prompts: harness
  D188's note, and not Marley's.

## Reference (§20)
N/A — Marley-specific. rustal-harness TICKET-115 (`cda7c24`, D188; `docs/CLAUDE_CODE.md#the-skill-rustal-ste`).

### Prior art
#709's `shared_plugin.rs` and its scenario; #725's `rustal_skill`.

## UI proof
`script/e2e/724-the-shared-plugin-with-rustal-ste.sh`: #709's scenario with the new values.
`724-01-on` shows a terminal's report. The checks:
- the plugin's version is `rustal-harness 0.3.0`;
- its folder is the digest `eebd8a51…`;
- `skills/rustal-ste/SKILL.md` is in it;
- `CLAUDE_CODE_PLUGIN_DIRS` names no second rustal-ste folder.

## Locked-In Decisions
- **D1:** Marley's digest must equal the harness's, so the file order is the harness's.
- **D2:** a terminal loads one `rustal-ste`, the shared plugin's when it is on.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the shared plugin is on, a new terminal shall load the plugin at the digest `eebd8a51…`, version 0.3.0, holding `skills/rustal-ste`. | Shot 724-01 and checks |
| REQ-002 | WHILE the shared plugin is on, the terminal shall load no second rustal-ste folder. | Check |
| REQ-003 | The gate shall pass. | `script/gates.sh --diff` exit 0 |
