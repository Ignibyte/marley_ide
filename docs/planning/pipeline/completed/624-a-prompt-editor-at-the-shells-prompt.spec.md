---
pipeline_id: 8930fd9b-85b3-416c-b33b-84794990176a
ticket: docs/planning/tickets/open/TICKET-624-a-prompt-editor-at-the-shells-prompt.md
status: Phase 4 — Complete PASS
title: "A prompt editor at the shell's prompt, on a key"
type: feature
slice: prong 1 T3
references: [docs/marley/three-prong-plan.md]
---

## Title
At a shell prompt, Ctrl+G opens #481's footer editor with the line typed so far; Enter clears the
shell's line and sends the editor's text and a return (plan T3, opt-in step; T3c makes it the
default).

## Scope
### In
- `crates/marley_workbench/src/rich_input.rs`: the editor opens at a shell prompt too
  (`AnchoredBlocks::at_prompt()`), on #481's own `marley::RichInput` (`ctrl-g` in the terminal's
  key context), which so far passes the key on when no agent runs; prefilled from #484's
  `typed_text`.
- Enter: Ctrl+U to the shell (clearing its line), then the text and a return, as #481's `send`
  does; Escape closes the editor and leaves the shell's line as it was; Shift+Enter a new line.
- #484's ghost text and → stay on the shell's own prompt; the editor shows none yet.

### Out (explicitly deferred)
- Completions (T6a, #625) and colouring (T6b, #626).
- The editor by default and the raw-passthrough ladder (T3c, #627).
- Multi-line commands beyond Shift+Enter's literal new line.

## Reference (§20)
- **Warp (behavior):** a command is typed in an editor docked under the blocks, with the editor's
  keys, and Enter runs it (`docs/warp_architecture/subsystems/02-editor-and-text.md` 176-182, the
  input editor; `subsystems/03-terminal-session-core.md` 153-172, the PTY writes).
- **Upstream Zed:** the footer editor is a Zed `Editor`; the terminal's key contexts
  (`terminal_view.rs` `screen=alt`, `DECCKM`, `bracketed_paste`) are kept.

### Prior art
- **Behavior maps:** the notes above; plan requirement 4 and D4 (`docs/marley/three-prong-plan.md`
  55-57, 97-100: keys route to an auto-height `Editor` docked at the bottom).
- **Published material:** none needed.
- **Code we already ship:** `rich_input::init`, `send`, `element` (#481; its spec's Out names
  "the shell's prompt editor (T3), which this editor grows into"); `autosuggest::typed_text` and
  `AnchoredBlocks::at_prompt` and `note_input` (#484); `english::hint` (#557).

## UI proof
`script/e2e/624-a-prompt-editor-at-the-shells-prompt.sh` (Hyprland keys, or sway): bash at a
prompt with `ech` typed.
- `editor.png`: after Ctrl+G, the footer editor holding `ech`;
- `typed.png`: `echo hi` in the editor;
- `ran.png`: after Enter, a block `echo hi` with output `hi`, the shell's line clean, the editor
  closed.

## Locked-In Decisions
- D1 — Opt-in on a key first: the shell's own prompt stays the default until T3c.
- D2 — The send clears the shell's line with Ctrl+U before typing, so a half-typed line never
  joins the editor's.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses Ctrl+G at a shell prompt, Marley shall open the footer editor holding the line typed so far. | Shot `editor.png` |
| REQ-002 | WHEN the user presses Enter in it, the shell shall run the editor's text as one block. | Shot `ran.png` |
| REQ-003 | WHEN the user presses Escape in it, the shell's line shall be left as it was. | Review |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the key, the prefill, the send; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
