# The Marley agent in a terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-684-the-marley-agent-in-a-terminal.md
- **Pipeline spec:** 684-the-marley-agent-in-a-terminal.spec.md

## Phase 1 — Plan
- **Request:** the queue's top, phase 1 item 6, on Chad's "just have it go" (2026-10-07).
- **Classification / tier:** feature, prong 2 C; Marley crate only.
- **Checklist (no task tool offered):** pick ✓ · pre-flight ✓ (#683's install compiling; planning
  only) · recall ✓ · mint ✓ · prior art ✓ · spec ✓ · design ✓.
- **Recall (§18.3):** #683's `assistant.rs` holds the instructions and the tool list; Rusty's
  `filter_palette` (#661) hides a layer's commands while it is off; #532's `start_in_terminal` is
  how every agent CLI starts; `L-claude-682-no-source-edit-while-an-install-compiles-001`.
- **Decisions:** D1, D2 in the spec.

### Design
- `assistant.rs`: `actions!(marley, [OpenMarleyAgentInTerminal])` with a doc comment; registered on
  each workspace (`observe_new`), its handler writing `INSTRUCTIONS` to
  `paths::data_dir()/assistant/instructions.md` off the main thread (`std::fs` in
  `futures::future::lazy` on the background executor), then `agents::start_in_terminal` with
  `AgentKind::Claude` and the line `<claude> --append-system-prompt-file '<file>' --disallowedTools
  Bash Edit Write NotebookEdit MultiEdit\n` (the path shell-quoted). `filter_palette(on)` hides
  the action type while the switch is off, from `sync` and at start.
- Scenario: fake `claude` writing `"$@"` one per line to `$E2E_WORK/claude-args.txt`; the palette
  searched for "open marley agent" with the switch off (shot, nothing), the switch on through the
  copy's settings, the command run (shot), the files checked.

**File manifest:** `crates/marley_workbench/src/assistant.rs` (Marley);
`crates/marley_workbench/src/agents.rs` only if `start_in_terminal`'s visibility needs widening;
`script/e2e/684-the-marley-agent-in-a-terminal.sh`.

### Risks
- Marley's MCP tools reach this Claude Code through its Marley plugin (#547); a Claude Code
  without the plugin gets the instructions and the limits but not the tools, which the
  instructions then name to no effect. Recorded; `--mcp-config` would add a second `marley`
  server beside the plugin's.

## Phase 2 — Code
- **Built** (`crates/marley_workbench/src/assistant.rs`): `OpenMarleyAgentInTerminal`
  (`#[derive(Eq)]`, as the crate's other actions), registered on each workspace;
  `open_in_terminal` writes `INSTRUCTIONS` to `<data dir>/assistant/instructions.md` on the
  background executor (`write_instructions`), then types `'<claude>' --append-system-prompt-file
  '<file>' --disallowedTools Bash Edit Write NotebookEdit MultiEdit` into a new agent terminal
  through `agents::start_in_terminal` (`AgentKind::Claude`); `quoted` shell-quotes the two paths;
  `filter_palette` shows the action while the switch is on, called from `sync` and from each new
  workspace. The scenario.
- **Deviation:** the line names the `claude` Marley found (`claude_program`) by its full path, not
  the bare `claude` Marley's other launches type: the same program #683 runs, and a scenario's
  fake reaches the terminal without a PATH change.
- **Review:** the command does nothing while the switch is off (a bound key included); the
  instructions file is rewritten each time, so a new Marley's text replaces an old one; the
  terminal is an ordinary agent terminal, the plugin's hooks and the rail treat it as any Claude
  Code. Clippy: `#[derive(Eq)]` on the action and shared references for the handler.
- **Checks before the gate:** the scenario green on its first run, 1 of 1, both shots right
  (`scratchpad/684-e2e-1.log`).
- **Gate:** `just gate-diff` green, 17 of 17 (`scratchpad/684-gate-1.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/684-the-marley-agent-in-a-terminal.sh` (`compositor sway`), run 2,
  the Test phase's: 1 of 1 (`scratchpad/684-e2e-2.log`). The fake's arguments:
  `--append-system-prompt-file <profile>/assistant/instructions.md --disallowedTools Bash Edit Write
  NotebookEdit MultiEdit`; the file holds "You are Marley's own agent".
- **Shots** (`scratchpad/shots-684/`, Marley only):
  - `684-01-hidden` (REQ-001): the palette, "open marley agent" typed, "No matches".
  - `684-02-terminal` (REQ-002, REQ-003): a second center tab "repo — claude /mnt/fast/tmp/clau…"
    and its rail row, the block of the typed line marked running, the fake printing the
    instructions file and the five tools.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it".

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Added, #684); `docs/marley/guide.md`'s Marley agent section;
  `docs/marley_architecture/marley_workbench.md`'s `assistant.rs` entry; the plan doc's item 6.
- **Knowledge (§19):** none new: no bug, and the decisions are #683's
  (`AD-claude-683-…`); the brain: no consultation opened, as the ticket decided nothing beyond
  #683's (recorded here as `brain_no_decision`'s reason).
- **Ticket:** TICKET-684 closed.
