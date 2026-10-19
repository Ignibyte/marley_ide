---
pipeline_id: 53a143c6-b403-4785-ac98-ba4336153976
ticket: docs/planning/tickets/closed/TICKET-813-rusty-runs-commands-and-edits-files.md
status: Phase 4 — Complete PASS
title: Rusty's agent runs commands and edits files
type: feature
slice: Prong 2, Rusty in Marley (#696's Rusty entry, #779's permission modes)
references: [683, 687, 696, 698, 779]
---

## Title
Rusty's entry in the Agent Panel runs commands and edits files, as the agent it runs on does on
its own, while the Marley agent stays read-only. Chad asked Rusty to check a system process and it
refused, because #696 gave Rusty the Marley agent's limits: an instruction not to run commands or
edit files, and the tools for both turned off on all three agents.

## Scope
### In
- Rusty on Claude Code: its session `_meta` disallows Marley's tools (`mcp__marley`) alone; Bash,
  Edit, Write, NotebookEdit and MultiEdit come back.
- Rusty on Codex: no `INITIAL_AGENT_MODE=read-only` and no `sandbox_mode: read-only`, so its
  threads start in codex-acp's own default mode, as a Codex thread does; #779's `full_access`
  still starts them in full access.
- Rusty on Zed's agent: the `rusty` profile turns on the built-in tools of Zed's Write profile,
  `terminal` and `edit_file` among them.
- `RUSTY_INSTRUCTIONS`: the "You do not edit files, run commands or change code here" paragraph
  goes; a line says Rusty may run commands and edit files on the user's machine when asked, and
  changes a brain page, task, note or memory only through the rusty server's tools.
- The doc comments in `assistant.rs` that describe the limits, and the guide's "Rusty in the Agent
  Panel" section.

### Out (explicitly deferred)
- The Marley agent: its instructions, its disallowed tools, its read-only Codex and its profile
  stay as they are.
- A setting for Rusty's access: Chad chose none (it comes with `marley.rusty.enabled`).
- Marley's own tools (`terminal_run`, the browser, the editor) for Rusty on Claude Code: still kept
  out, as today.
- New permission modes or prompts: the agents' own asking and #779's settings decide.
- `marley: open marley agent in terminal`, which is the Marley agent's alone.

## Reference (§20)
Upstream Zed, `agent_settings` and `agent_servers`: the behavior kept is how each agent asks
before acting on its own. Zed's native agent asks before `terminal` and `edit_file` under its
default tool permissions; claude-acp's `default` mode asks before Bash and edits; codex-acp's
default `agent` mode ("Auto review") asks for actions it judges unsafe. Rusty takes each agent's
own behavior, with Rusty's prompt and server added, instead of a Marley-made limit.

### Prior art
- Behavior maps: `docs/zed_architecture/` (the Agent Panel and agent servers). Nothing on an
  entry's tool limits in Warp's or Orca's maps.
- Published material: ACP session modes; claude-agent-acp's `disallowedTools` in the session
  `_meta` (L-claude-696, its `dist/acp-agent.js`); codex-acp 2.2.2's modes, read from the package
  (`npm pack @agentclientprotocol/codex-acp@2.2.2`, `dist/index.js` near line 34265):
  `read-only`, `workspace-write`, `agent` (the default, `DEFAULT_AGENT_MODE`) and
  `agent-full-access`; `INITIAL_AGENT_MODE` picks the first, else the default.
- The code we ship: `agent_settings::builtin_profiles::WRITE` and the Write profile's tool list in
  `assets/settings/default.json` (`agent.profiles.write.tools`), which the `rusty` profile takes
  from the default settings instead of a copied list; #779's `marley.agent_default_modes`, which
  already starts Rusty's threads in bypass or full access; #696's `session_meta`, `codex_config`,
  `set_entry` and `profile` in `marley_workbench::assistant`, which only change what they put in.
  No new seam: every piece exists, and this ticket narrows #696's limits to the Marley entry.

## UI proof
`script/e2e/813-rusty-runs-commands-and-edits-files.sh`, on #696/#738's set-up: Rusty on through
`marley_rusty`'s stand-in `rusty-mcp`, and `MARLEY_ASSISTANT_ADAPTER` running a scripted ACP agent
that answers a prompt with what its session was given (the disallowed tools, whether the
instructions hold the no-commands line and the Rusty-tools line, `INITIAL_AGENT_MODE`, the
`CODEX_CONFIG` sandbox). Conversations start from the palette (`marley: new rusty conversation`,
`marley: new marley conversation`).
- `813-01-rusty-claude`: a Rusty conversation on Claude Code (REQ-001, REQ-002).
- `813-02-marley-claude`: a Marley conversation on Claude Code (REQ-003).
- `813-03-rusty-codex` and `813-04-marley-codex`: both entries after a restart with
  `marley.assistant.agent` set to `codex` (REQ-004).
- `813-05-rusty-profile`: `agent: manage profiles`, Rusty's built-in tools, after the setting
  moves to `zed` (REQ-005).

## Locked-In Decisions
- D1 — Rusty's entry is the agent it runs on, plus Rusty's prompt and Rusty's server: Claude
  Code's own tools, codex-acp's default mode, Zed's Write tools. No Marley-made tool list (Chad,
  2026-10-18: "Commands + file edits").
- D2 — It comes with Rusty being on; there is no new setting (Chad, 2026-10-18: "Comes with
  Rusty").
- D3 — The asking is the agents' own: Claude Code and Zed's agent ask before each command or edit,
  Codex's default mode before the ones it judges unsafe, and #779's `bypass` and `full_access`
  skip it as they do for every agent.
- D4 — Marley's tools stay out of Rusty's Claude Code sessions (`mcp__marley`), as #696 set.
- D5 — Rusty's data (brain pages, tasks, notes, memories) changes only through the rusty server's
  tools, so Rusty's index, link graph and history stay right; the instructions say so.
- D6 — The `rusty` profile takes the Write profile's tools from the default settings when Marley
  puts it in, so a tool Zed adds to Write reaches Rusty without a Marley change.
- D7 — The Marley agent is untouched.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a Rusty conversation starts on Claude Code, its session `_meta` shall disallow `mcp__marley` and none of Bash, Edit, Write, NotebookEdit or MultiEdit. | Shot `813-01-rusty-claude`; the scripted agent's log |
| REQ-002 | WHEN a Rusty conversation starts, its instructions shall leave out "You do not edit files, run commands or change code here" and shall say brain pages, tasks, notes and memories change through the rusty server's tools. | Shot `813-01-rusty-claude`; the log |
| REQ-003 | WHEN a Marley conversation starts on Claude Code, its `_meta` shall still disallow Bash, Edit, Write, NotebookEdit and MultiEdit, and its instructions shall still hold "You do not edit files, run commands or change code here". | Shot `813-02-marley-claude`; the log |
| REQ-004 | WHILE `marley.assistant.agent` is `codex`, Rusty's entry shall start its adapter with no `INITIAL_AGENT_MODE` and a `CODEX_CONFIG` with no `sandbox_mode`, and the Marley entry's shall keep `INITIAL_AGENT_MODE=read-only` and `sandbox_mode` `read-only`. | Shots `813-03-rusty-codex`, `813-04-marley-codex`; the log |
| REQ-005 | WHILE `marley.assistant.agent` is `zed`, Zed's agent's `rusty` profile shall turn on the Write profile's built-in tools, `terminal` and `edit_file` among them. | Shot `813-05-rusty-profile` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `assistant.rs` only; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — the guide's "Rusty in the Agent Panel", CHANGELOG and architecture docs (§21),
  ledger capture (§19), close the ticket, archive, commit; the stop for Chad to try after the
  install (ask Rusty to check a process).
