---
pipeline_id: a41b1d50-4df5-4e66-b8c8-78afff62455c
ticket: docs/planning/tickets/open/TICKET-709-load-the-shared-claude-code-plugin.md
status: Phase 4 — Complete PASS
title: Load the shared Claude Code plugin
type: feature
slice: prong 2 C1; design note B2 (docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md), #652's next slice
references:
  - docs/planning/pipeline/completed/652-shared-claude-plugin-marleys-half.spec.md
---

## Title
Marley loads the Claude Code plugin it shares with rustal-harness, so a Claude Code in a Marley
terminal reports its exact state to the rail through `MARLEY_BIN` (#652).

## Scope
### In
- **The plugin's six files,** byte for byte as rustal-harness TICKET-108 left them (version 0.2.0,
  MIT OR Apache-2.0), under `crates/marley_workbench/claude_shared_plugin/`, embedded with
  `include_str!`. The digest is computed as `rh` does; it is
  `83d0bb8f5cd3041483c73b92a9d09ca01f4c5bd8763f8cf65b8433a713b10303` for these files.
- **Written by digest** into `<data dir>/claude-code/shared/<digest>/` the first time it is wanted:
  staged, then renamed into place, folders 0700 and files 0400. A folder already there is checked
  byte for byte, and an altered one is not loaded.
- **In local Marley terminals** (those `MARLEY_TERMINAL_ID` names), `CLAUDE_CODE_PLUGIN_DIRS` gets
  that folder first and the inherited folders after it, any other Marley copy removed (#652's D9).
  The terminals have it while `marley.claude_code_shared_plugin` is on, off by default, and #648's
  row `claude_shared_plugin` (Claude Code 2.1.287 and later) is on.
- **Settings:** the Agents section's Shared Claude Code Plugin toggle and the Agent Versions
  section's Shared Plugin on Untested Claude Code.

### Out (explicitly deferred)
- `marley-agent listen`/`delivered`/`approve` (prompts in, approvals): #652's later slices.
- Retiring the hook path and folding Marley's own plugin into the shared one.

## Reference (§20)
N/A — Marley-specific, with rustal-harness's plugin as the shared code: the files are the
harness's (TICKET-108), carried unchanged, and the digest and the read-only install follow `rh`'s
`claude/plugin.rs` (read, not copied; Marley's own code). Claude Code's documented
`CLAUDE_CODE_PLUGIN_DIRS`: `:`-separated absolute paths, loaded as `--plugin-dir`, an `@inline`
plugin that replaces a same-named installed one, from v2.1.280.

### Prior art
- **The code we ship:** `marley_terminal::identity::agent_environment` (#652), called by
  `terminal.rs` for every terminal, already sets `MARLEY_BIN` for named terminals, so the new
  variable needs no Zed hunk. `marley_agent::versions` holds the rows #648 gates on.
  `claude_ide.rs` (#653) is the model for a setting that is off by default, gated on a version
  row, and reconciled on settings and version changes. `sha2` is already a dependency.
- **rustal-harness:** `crates/harness-runtime/src/claude/plugin.rs`, with `FILES`, `digest`,
  `install` (staged and renamed, 0700/0400) and `check`.
- **Docs:** Claude Code's `env-vars.md` and `plugins__loading.md` (saved under the harness's
  TICKET-091 sources).

## UI proof
`script/e2e/709-load-the-shared-claude-code-plugin.sh`, under `compositor sway`. A stand-in
`claude` (#652's), first on the terminal's PATH, answers `--version` with 2.1.287. When run, it
prints `CLAUDE_CODE_PLUGIN_DIRS` and the first folder's plugin name and version, then acts out
the plugin's reports through `$MARLEY_BIN`. Real Claude Code never runs in a scenario (PR-687).

Shots:
- `709-01-on`: with the setting on, the stand-in shows the plugin folder first, rustal-harness
  0.2.0, and the rail's row shows the state it reported (waiting, with the tool).
- `709-02-off`: with the setting off, a new terminal's stand-in shows no Marley plugin folder.

## Locked-In Decisions
- **D1:** the variable is set in `agent_environment` (Marley crate), so no Zed path changes for it.
- **D2:** the files are carried unchanged, and the digest names the folder. An altered folder is
  logged and not loaded.
- **D3:** off by default, behind a setting and #648's version row, as #652 decided.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE `marley.claude_code_shared_plugin` is on and the installed Claude Code is 2.1.287 or later, a new local Marley terminal shall have the plugin's folder first in `CLAUDE_CODE_PLUGIN_DIRS`, holding the six files read-only. | Shot 709-01; the stand-in's log; the folder's modes |
| REQ-002 | WHEN the plugin's reports reach `MARLEY_BIN`, the rail's row shall show the state they report. | Shot 709-01 |
| REQ-003 | WHILE the setting is off, a new terminal shall have no Marley plugin folder in `CLAUDE_CODE_PLUGIN_DIRS`. | Shot 709-02; the stand-in's log |
| REQ-004 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the files, `shared_plugin.rs`, `agent_environment`'s variable, the version row,
  the settings; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check, every shot read.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close, archive,
  commit.
