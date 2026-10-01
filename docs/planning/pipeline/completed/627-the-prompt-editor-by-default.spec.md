---
pipeline_id: 792fa05b-50e3-4ee8-a257-57ab4d014ae9
ticket: docs/planning/tickets/open/TICKET-627-the-prompt-editor-by-default.md
status: Phase 4 — Complete PASS
title: "The prompt editor by default, with the raw-passthrough ladder"
type: feature
slice: prong 1 T3
references: [docs/marley/three-prong-plan.md]
---

## Title
The prompt editor becomes the default input while the shell is at a prompt, and a ladder sends keys
raw to the program everywhere else (plan T3, D4). After #624, #625 and #626.

## Scope
### In
- `crates/marley_workbench`: the footer editor docks whenever the terminal's shell is at a prompt
  (`at_prompt()`), focused with the terminal; a setting turns it off.
- The ladder: the alternate screen or a running command (no prompt) sends every key raw
  (`to_esc_str`), as today; otherwise keys go to the editor. Readline turns bracketed paste on at
  every prompt, so bracketed paste and application cursor mode do not count against the editor.
  Ctrl+C at the editor clears it; Escape closes it until the next prompt.

### Out (explicitly deferred)
- #484's ghost text and → inside the editor, and #557's hint under it: each needs Zed's edit
  prediction machinery or a placed overlay; a follow-up ticket.
- Ctrl+D passing EOF from an empty editor.
- #573's network reading (its own ticket, on the editor's idle point).
- Remote shells without Marley's hooks (no prompt known; raw as today).

## Reference (§20)
- **Warp (behavior):** the input editor owns the prompt, and a running program gets the keys
  (`docs/warp_architecture/subsystems/02-editor-and-text.md`, `subsystems/03-terminal-session-core.md`
  153-172).
- **Upstream Zed:** the terminal's key contexts and `to_esc_str` (`crates/terminal/src/mappings/
  keys.rs`), kept as the raw path.

### Prior art
- **Behavior maps:** the notes above; plan D4 and its risks (driven-keystroke checks); the
  gpui-era ladder, TICKET-432 (`docs/marley/history/CHANGELOG-gpui-era.md`).
- **Published material:** xterm's modes (DECCKM, bracketed paste, the alternate screen).
- **Code we already ship:** #481's key stopping (`rich_input::element`); `TerminalView::key_down`;
  the key context's `screen=alt`, `DECCKM`, `bracketed_paste`; `at_prompt`.

## UI proof
`script/e2e/627-the-prompt-editor-by-default.sh`: bash.
- `prompt.png`: at a prompt, the editor docked and holding typed `echo hi`;
- `ran.png`: after Enter, the block and the editor empty again;
- `vim.png`: `vim` running full screen, the editor gone, `:q` typed raw;
- `back.png`: after `:q` and Enter, vim gone and the editor back.

## Locked-In Decisions
- D1 — The ladder's order is the gpui-era one Marley proved.
- D2 — A setting turns the editor off, leaving #624's key.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the shell is at a prompt, Marley shall send typed keys to the prompt editor. | Shot `prompt.png` |
| REQ-002 | WHILE the alternate screen is on or a command runs, Marley shall send keys to the program raw. | Shots `vim.png`, `back.png` |
| REQ-003 | WHEN the user presses Escape in the editor, Marley shall give the keys to the shell until its next prompt. | Review |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the default dock, the ladder, the ghost text in the editor; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
