---
pipeline_id: 0fc83695-9672-4e85-9fe7-d90bd9ecafba
ticket: docs/planning/tickets/open/TICKET-928-agent-terminal-as-the-board.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: An agent CLI's terminal as the board draws it
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/A-core.md, docs/planning/pipeline/completed/556-terminal-run.spec.md, docs/planning/pipeline/completed/810-the-take-over-bar-can-be-closed.spec.md]
---

## Title
The waiting command as the AgentTerminal board's card (who asks, the folder, the list status,
the command, Run · Enter, Refuse · Esc, Always run … here), the typing bar in the board's words
with its key, and the rich input's key hints.

## Scope
### In
- **The waiting-command card** (`terminal_drive.rs`, `footer` and `ask_to_run`; Marley crate):
  `Pending` keeps who asks, the command, the terminal's folder and the list verdict. The footer
  draws a rounded, bordered card: a warning dot, "<who> wants to run", "in <folder, compacted>",
  and the status ("not on either list", or "on the ask list"); the command in mono on its own
  line; then Run with its key, Refuse with its key and "Always run <prefix> here". A pending
  write to a program (`terminal_type`) keeps today's bar with its keys added.
- **Always run … here** (`terminal_drive.rs`; `marley_workbench.rs` and
  `settings_content/src/marley.rs`, the Marley settings file in a Zed crate): a new user setting
  `marley.agent_command_allowlist_by_folder`, a map from a folder to patterns. The button adds
  the command's prefix as an anchored pattern (`<prefix>(\s.*)?`, regex-escaped) under the
  terminal's project folder through `update_settings_file`, then runs the command. `run`'s
  verdict adds the patterns of every listed folder that holds the terminal's folder to the
  allowlist. The prefix is the program and, when the second word is not an option, that word
  ("cargo test").
- **The typing bar** (`terminal_drive.rs`, `footer`): "<who> is typing into this terminal's
  program" in place of "<who> typed into <program>: <text>"; Take Over and Hand Back carry their
  key from the keymap ("Take Over · Ctrl I"); the close button stays (#810).
- **The rich input's key hints** (`rich_input.rs`, `element`; Marley crate): while the editor is
  open for an agent, a muted row under it: "Enter sends · Shift Enter new line · Esc keeps the
  draft · Ctrl Shift V pastes an image", each key read from the keymap for its action.

### Out (explicitly deferred)
- The model and the context ("Opus 5.5 · 48k context") in the agent bar: an agent's report
  (`marley_agent::report::Report`, the harness's schema, `deny_unknown_fields`) carries tokens but
  no model, so this needs a field on the harness seam; a follow-up ticket.
- Claude Code's own permission prompt drawn as the card: it stays in its TUI and in Needs you
  (#840); the card is Marley's `terminal_run`.
- The card drawn inline in the terminal's grid: it stays in the footer slot under the grid.
- The agent terminal's bottom row (agent chip, model, mode, waiting chip, Turns, context): #908
  Out and #907 Out leave it as new behaviour; it waits on the model field above; a follow-up
  ticket.

## Reference (§20)
- **Warp, command execution under agent profiles** (published docs,
  https://docs.warp.dev/agents/capabilities/agent-profiles-permissions/, as #556's spec cites
  them): an allowlist runs a command without asking, the denylist always asks; the approval card,
  Enter to run and the take-over control (`Ctrl+I`, Full Terminal Use,
  https://docs.warp.dev/agents/capabilities/full-terminal-use/) are kept from #525 and #556. The
  behaviour map `docs/warp_architecture/subsystems/04-agent-ai-mcp.md` describes the agent's
  command loop, read for its shape only; no Warp code. The per-folder allow is Marley's.
- **Upstream Zed:** the settings file and `update_settings_file` are Zed's, kept; key labels come
  from Zed's keymap (`KeyBinding::for_action`).

### Prior art
- `terminal_drive.rs:127` `Pending { words, allow, deny, run, answer }`; `:1031` `ask_to_run`
  builds "<who> wants to run <cmd>"; `:666` `footer` draws the bar; `:836` the verdict from
  `agent_command_allowlist` and `agent_command_denylist`; `marley_terminal/src/agent_commands.rs`
  `verdict` and `Verdict { Allowed, Ask, Outside }`.
- `crates/marley_workbench/keymap.json:34–38`: `ctrl-i` Take Over, `enter` Run, `escape` Refuse
  (`cmd-i` on macOS).
- `rich_input.rs:1–31` documents the keys (Enter sends, Shift-Enter adds a line, Escape keeps
  the draft, Ctrl-Shift-V pastes, an image going on to the agent); `:367` `placeholder`.
- `agent_bar.rs:164` `footer_without_agent` stacks the drive's footer and the prompt editor.
- `settings_content/src/marley.rs:150` `agent_command_allowlist`; Marley's settings are
  user-level only (no project settings field), so "here" is a folder key in the user's settings.
- AD-claude-556-an-agent-runs-at-the-prompt-behind-two-lists-and-a-signed-prompt-001.
- The brain: not consulted at queueing; the promoting session asks.

## UI proof
Scenario `script/e2e/928-agent-terminal-as-the-board.sh` (sway): a scratch repository; the
stand-in Claude Code and a stand-in `cargo` first on the terminals' PATH through the
`.bashrc`; the stand-in MCP client in a second terminal (L-claude-871) calls `terminal_run` and
`terminal_type` on the first; `marley.agent_commands_outside_lists` set to ask.
- `928-01-card`: `terminal_run` with `cargo test -p demo`: the card (REQ-001).
- `928-02-always`: Always run cargo test here clicked: the command ran; a second `terminal_run`
  of `cargo test --doc` runs with no card (REQ-002).
- `928-03-typing`: `terminal_type` into `python3` in the first terminal: the typing bar with
  Take Over · Ctrl I (REQ-003).
- `928-04-hints`: the stand-in Claude Code running, Ctrl-G: the rich input with its hint row
  (REQ-004).
- `928-05-day`: the card and the hints in Marley Day (REQ-001, REQ-004).

## Locked-In Decisions
- D1 — The card stays in the terminal's footer; only its content and shape change.
- D2 — "Here" is the terminal's project folder (its worktree root), else its working folder; the
  pattern applies to that folder and everything under it.
- D3 — The prefix is at most two words, so "Always run" never allows more than the user reads on
  the button; a command whose first word is a shell or another denylisted program gets no Always
  button (the denylist still asks).
- D4 — Every key on the card, the bar and the hint row is read from the keymap, so a rebinding
  shows.
- D5 — The typing bar no longer repeats what was typed; the terminal shows it.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE an agent's command waits for the user, the terminal shall show a card with who asks, the folder, the list status, the command on its own line, Run and Refuse with their keys, and Always run <prefix> here. | `928-01-card`, `928-05-day` |
| REQ-002 | WHEN the user chooses Always run <prefix> here, the command shall run, and later commands with that prefix in that folder shall run without the card. | `928-02-always` |
| REQ-003 | WHILE an agent types into a terminal's program, the bar shall read "<agent> is typing into this terminal's program" and offer Take Over with its key. | `928-03-typing` |
| REQ-004 | WHILE the rich input is open for an agent, a row under it shall name the keys that send, add a line, keep the draft and paste an image. | `928-04-hints`, `928-05-day` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `terminal_drive.rs`, `rich_input.rs`, `marley_workbench.rs`, the
  `settings_content/src/marley.rs` field and its touchpoint row; a review of the diff;
  `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
