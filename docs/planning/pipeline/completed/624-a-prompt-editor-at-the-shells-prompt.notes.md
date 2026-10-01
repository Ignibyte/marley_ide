# A prompt editor at the shell's prompt, on a key — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-624-a-prompt-editor-at-the-shells-prompt.md
- **Pipeline spec:** 624-a-prompt-editor-at-the-shells-prompt.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, second batch (#624 to #627): T3 and T6, the prompt.
- **Recall (§18.3):**
  - #481's editor opens only while an agent runs; its spec defers the shell's prompt editor to T3.
  - #484 reads the typed line from the grid (`typed_text`), giving up when text follows the cursor or the view is scrolled back.
  - Plan D4: keys route to an editor docked at the bottom, with the proven raw ladder for everything else (T3c).
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted;** seams re-verified: `rich_input::init`'s `RichInput` handler opens the editor when
  `agent_in` finds an agent and otherwise propagates the key to the program; `Prompt { editor, open
  }` per view; `send` pastes with `terminal_drive::paste_then` and a carriage return; `element`
  docks the editor in the footer and stops special keys and chords from reaching the terminal;
  `autosuggest::typed_text(terminal)` gives the line typed at a prompt (none when text follows the
  cursor, the view is scrolled back, or on the alternate screen).
- **Recall:** the queued notes stand. The brain (`rusty-cli brain ask`, consultation 45f1ebc7b048456780bfd3a4bbd80bf7):
  nothing on this seam.

### Design
- **`crates/marley_workbench/src/rich_input.rs`:** `Prompt` gains `target: Target` (`Agent(kind)`
  or `Shell`). The handler opens the editor for an agent as before, else for the shell while its
  terminal is at a prompt off the alternate screen, else passes the key on. A shell open sets the
  placeholder ("A command for the shell") and replaces the text with the typed line when there is
  one. `send` for the shell types Ctrl+U first, then pastes the text and a carriage return.
- **Manifest:** `rich_input.rs`, the scenario.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | bash at a prompt, `ech` typed, Ctrl+G | `editor.png` |
| REQ-002 | `o hi` typed in the editor, Enter | `typed.png`, `ran.png` |
| REQ-003 | review only (Escape keeps the shell's line) | — |

## Phase 2 — Code (2026-09-30)
- **Built (`rich_input.rs`):** `Target` (`Agent`, `Shell`) on `Prompt`; `target_of`; `open` kept for
  the agent bar's button and `open_for` behind it, the placeholder set per target and the shell's
  typed line put in the editor with the cursor at its end; `send` types Ctrl-U first for the shell;
  `new_editor` takes no agent now.
- **Clippy:** clean the first time. **Gate:** GREEN.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/624-a-prompt-editor-at-the-shells-prompt.sh` (sway).
- **Run 1:** red: the shortcut toast said Ctrl-G opened Rich Input, but no editor showed, and `o hi`
  and Enter went to the shell (F-claude-624). Fixed in `agent_bar::footer_without_agent`; clippy
  clean, the gate GREEN, rebuilt.
- **Run 2, the shots read:**
  - `editor.png` (REQ-001): the footer editor under the terminal holding `ech`, its cursor at the
    end; the shell's line still `$ ech` with #557's hint.
  - `typed.png`: the editor holding `echo hi`.
  - `ran.png` (REQ-002): one block `$ echo hi` with `hi`, a clean prompt (the `ech` cleared), the
    editor closed; the rail's row `echo hi · done`.
- **REQ-003** (Escape leaves the shell's line): reviewed; `close` touches only the editor.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; the guide (Rich input); `marley_workbench.md`; the plan's T3 row.
- **Knowledge:** F-claude-624-the-shells-editor-opened-unseen-001.
- **Brain:** the consultation closed with `brain decide`.

