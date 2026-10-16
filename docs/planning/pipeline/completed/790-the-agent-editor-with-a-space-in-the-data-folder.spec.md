---
pipeline_id: 885f2668-e9f0-4671-bf41-ad40f9a715c9
ticket: docs/planning/tickets/closed/TICKET-790-the-agent-editor-with-a-space-in-the-data-folder.md
status: Phase 4 — Complete PASS
title: "The agent editor when the data folder has a space"
type: bug
slice: macOS
references: [docs/planning/pipeline/completed/792-the-block-terminal-on-macos.spec.md]
---

## Title
The agent editor (#649) gives the terminals Marley opens for agents `marley-edit` as `VISUAL` and
`EDITOR`, written to `<data dir>/mcp/marley-edit`. Three of the four agents split `$VISUAL` on
spaces, so `agent_editor::set_helper` refuses a path with whitespace. On macOS the data folder is
always `~/Library/Application Support/Marley`, so every start logs "Marley's data directory has a
space in its path" and Ctrl+G never opens the prompt in a tab. Found in #787's check on the Mac.

## Scope
### In
- `crates/marley_workbench/src/agent_editor.rs`: when the helper's path has whitespace, a link to
  it in Marley's state folder, in a subfolder named for the helper's path. The agents get the
  link. `marley-edit` finds its endpoint beside its real path (`os.path.realpath(__file__)`), so
  the link reaches the right Marley.
- `crates/marley_workbench/src/mcp.rs` (`offer_agent_editor`): the link is made off the main
  thread, after the helper is written.

### Out (explicitly deferred)
- A home folder with a space. Then the state folder has one too; the warning stays, and so does
  the overlay.
- The agent editor's default, which stays off (`marley.agent_editor_in_tab`).

## Reference (§20)
N/A — Marley-specific: `marley-edit` (#649) and its `$VISUAL` are Marley's. The behavior kept is
#649's: a terminal Marley opens for an agent gets the helper as its editor while the switch is on.

### Prior art
- **The code we ship:**
  - `marley-edit` (`crates/marley_workbench/bin/marley-edit:50-54`): `$MARLEY_MCP_ENDPOINT`, else
    `mcp-endpoint.json` beside `dirname(realpath(__file__))`'s parent, so a link resolves to
    the data folder's helper.
  - `paths::state_dir()`: `~/.local/state/Marley` on macOS (`crates/paths/src/paths.rs:170-175`).
  - `mcp::write_program_in` writes the helper and the opener.
- **Behavior maps:** nothing applies.

## UI proof
On the Mac:
- The log after a restart: no "agent editor: … space in its path" line.
- The link: `~/.local/state/Marley/agent-editor/<hash>/marley-edit`, resolving to the data
  folder's helper.
- `790-mac-agent-editor.png`: the helper run through the link from a Marley terminal (with
  `terminal_run`) opens its file in a tab of that terminal's project, which shows the link reaches
  the right Marley's endpoint.

On Linux, nothing changes when the data folder has no whitespace; the run is `just shot`.

## Locked-In Decisions
- D1 — A symbolic link, not a second copy. `marley-edit`'s endpoint lookup follows its real path,
  so a copy elsewhere would look for the endpoint in the wrong folder.
- D2 — The link lives at `state_dir()/agent-editor/<16 hex of the helper path's hash>/marley-edit`,
  so two data folders get two links, each to its own helper.
- D3 — The link is remade on each start, which replaces a stale link to a moved data folder.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts on macOS, it shall log no "space in its path" warning from the agent editor | the Mac's log |
| REQ-002 | WHEN the helper's path has whitespace, Marley shall publish a whitespace-free link to it as the agents' editor | the link and its target on the Mac |
| REQ-003 | WHEN the editor runs through that link from a Marley terminal, it shall open its file in a tab of that terminal's workspace | `790-mac-agent-editor.png` |
| REQ-004 | WHILE the helper's path has no whitespace, Marley shall publish the helper itself, as before | the review of the diff; `just shot`; the gate |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the link; a review; the gate; clippy on the Mac.
- **P3 Test** — on the Mac: install, restart, the log, the link, the editor through the link; on
  Linux: `just shot`.
- **P4 Complete** — CHANGELOG, the guide's Mac section, the crate note, ledger, close, archive,
  commit.
