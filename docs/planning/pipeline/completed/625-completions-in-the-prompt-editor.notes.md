# Completions in the prompt editor — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-625-completions-in-the-prompt-editor.md
- **Pipeline spec:** 625-completions-in-the-prompt-editor.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, second batch (#624 to #627): T3 and T6, the prompt.
- **Recall (§18.3):**
  - Zed's `CompletionsMenu` belongs to an `Editor`; over the bare grid it would need a menu of Marley's own.
  - #484 already reads history (the terminal's commands, then the file) and matches prefixes (`suggest.rs`).
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted;** seams re-verified: `editor::CompletionProvider` (`completions`, `is_completion_trigger`,
  `sort_completions`, `filter_completions`), with `keymap_editor`'s action provider as the plainest
  example (a replace range from the word before the cursor, `project::Completion` with
  `CompletionSource::Custom`, one `CompletionResponse`); Zed binds Enter to `ConfirmCompletion` and
  Tab to `ComposeCompletion` in `Editor && showing_completions`, while Marley's keymap binds Enter to
  `SendRichInput` in `MarleyRichInput > Editor`, which a later keymap would let win; #484's history
  is the terminal's verified commands then its history file (`autosuggest::suggestion`);
  `AnchoredBlocks` keeps the staged prompt's `PromptInfo` (with `pwd`) privately;
  `TaskInventory::list_tasks(None, None, Some(worktree), cx)` gives the templates.
- **Recall:** the queued notes stand. The brain (`rusty-cli brain ask`, consultation 165f76c66f4f445e96e94c6b6b9f6df1):
  nothing on this seam.

### Design
- **`crates/marley_terminal/src/anchored.rs`:** `prompt_folder()`, the staged prompt's `pwd`.
- **`crates/marley_workbench/src/autosuggest.rs`:** `history(terminal, cx)`, the commands
  `suggestion` already walks (newest first), which `suggestion` now uses.
- **`crates/marley_workbench/src/shell_completions.rs`** (new): `ShellCompletions { terminal,
  workspace }`, a `CompletionProvider`: the line before the cursor and its last word; the word as
  a path under the prompt's folder (its folder part listed off the main thread, entries starting
  with the rest, folders ending in `/`, hidden ones only for a word starting with `.`); the whole
  line as the start of a history command or a task's command (each once). It sorts and filters
  itself and marks its answer incomplete, so each key asks again.
- **`crates/marley_workbench/src/rich_input.rs`:** the shell's editor gets the provider, an agent's
  none.
- **`crates/marley_workbench/keymap.json`:** `tab` to `editor::ShowCompletions` in
  `MarleyRichInput > Editor && !showing_completions`, and Enter's `SendRichInput` there only while
  no menu shows.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | `repo` with `Cargo.toml` and `src/`; Ctrl+G; `cat Car`, Tab | `paths.png` |
| REQ-002 | Enter in the menu | `taken.png` |
| REQ-003 | the history file holding `cargo test --workspace`; the editor's text `cargo t`, Tab | `history.png` |

## Phase 2 — Code (2026-09-30)
- **Built:** `shell_completions.rs` (new); `AnchoredBlocks::prompt_folder`; `autosuggest::history`
  (which `suggestion` now uses); the provider set on the shell's editor in `rich_input::open_for`;
  the keymap's Tab and Enter while no menu shows.
- **Clippy found:** imports (`AppContext`, `language::Anchor` for `text::Anchor`), `map_unwrap_or`,
  `option_if_let_else`, `redundant_closure_for_method_calls`, a missing `Debug` and field docs.
- **The gate found (gate:21, Zed's dylint):** an `async` block with no `.await` around the folder
  read inside the background task; it reads in place now. GREEN after.

## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/625-completions-in-the-prompt-editor.sh` (sway): `repo` with
  `Cargo.toml` and `src/`, the history file holding `cargo test --workspace`. One run.
- **Shots:**
  - `paths.png` (REQ-001): the editor `cat Car` with Zed's menu over it listing `Cargo.toml`.
  - `taken.png` (REQ-002): the editor `cat Cargo.toml`, the shell's prompt empty, nothing run.
  - `history.png` (REQ-003): the editor `cargo t` with the menu listing `cargo test --workspace`.
- **Not driven:** a task's command in the menu (the scenario's project has no tasks), and a path
  with a folder part.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; the guide (Rich input); `marley_workbench.md`; the plan's T6 row.
- **Knowledge:** L-claude-625-zeds-dylint-runs-in-the-gate-not-in-just-clippy-001.
- **Brain:** the consultation closed with `brain decide`.

