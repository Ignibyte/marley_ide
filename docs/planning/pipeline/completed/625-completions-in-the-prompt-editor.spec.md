---
pipeline_id: 0636fe38-0438-490e-b49f-79ca97703c92
ticket: docs/planning/tickets/open/TICKET-625-completions-in-the-prompt-editor.md
status: Phase 4 — Complete PASS
title: "Completions in the prompt editor"
type: feature
slice: prong 1 T6
references: [docs/marley/three-prong-plan.md]
---

## Title
The prompt editor (#624) offers completions through Zed's menu: paths against the prompt's
folder, history, and the project's tasks (plan T6, first half). After #624.

## Scope
### In
- `crates/marley_workbench`: a `CompletionProvider` for the prompt editor: the word under the cursor
  completed as a path relative to the prompt's folder (`PromptInfo.pwd`), with folders ending in
  `/`; as the whole line, from #484's history (the terminal's verified commands, then the history
  file); and as a task's command from `TaskInventory::list_tasks`. Tab and Zed's `ShowCompletions`
  open it; Enter in the menu takes an entry without running the line.

### Out (explicitly deferred)
- Command and flag signatures (`docs/warp_architecture/crates/warp_completer.md`'s parser).
- Completions over the bare grid, without the editor.

## Reference (§20)
- **Warp (behavior):** completions at the input for paths, history and commands
  (`docs/warp_architecture/crates/warp_completer.md`: tokens, signatures, fuzzy ranking).
- **Upstream Zed:** `editor::CompletionsMenu` and `CompletionProvider`, kept as the menu.

### Prior art
- **Behavior maps:** the Warp note above; `subsystems/03-terminal-session-core.md` 153-172
  (native shell completions are a later idea).
- **Published material:** none needed.
- **Code we already ship:** `CompletionProvider` and `set_completion_provider` (`crates/editor`);
  `PromptInfo.pwd` (`anchored.rs`); #484's history lookup (`autosuggest.rs`, `suggest.rs`);
  `TaskInventory::list_tasks` (`crates/project`). The menu needs an editor, so this follows #624.

## UI proof
`script/e2e/625-completions-in-the-prompt-editor.sh`: a folder with `Cargo.toml` and `src/`; a
history file with `cargo test --workspace`; a task `build`.
- `paths.png`: `cat Car` then Tab, the menu with `Cargo.toml`;
- `taken.png`: Enter in the menu, the editor holding `cat Cargo.toml`, nothing run;
- `history.png`: `cargo t` then Tab, the menu with `cargo test --workspace`.

## Locked-In Decisions
- D1 — Zed's menu, not a menu of Marley's: the editor already hosts it.
- D2 — Paths complete against the prompt's folder, which a `cd` in the shell moves.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses Tab after a partial path in the prompt editor, Marley shall list the matching entries of the prompt's folder. | Shot `paths.png` |
| REQ-002 | WHEN the user takes an entry, the editor shall hold it and the line shall not run. | Shot `taken.png` |
| REQ-003 | WHEN the line so far starts a command in the history, the menu shall list it. | Shot `history.png` |

## Phase Plan
- **P1 Plan** — promote, re-verify the seams, the design.
- **P2 Code** — the provider and its three sources; a review; the gate green.
- **P3 Test** — the scenario, every shot read.
- **P4 Complete** — docs (§21), the ledger; close, archive, commit.
