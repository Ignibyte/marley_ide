# The prompt editor by default, with the raw-passthrough ladder — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-627-the-prompt-editor-by-default.md
- **Pipeline spec:** 627-the-prompt-editor-by-default.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, 2026-09-30: "lets spec out the remaining tickets and we can begin everything except the cloud flare ones" (wave 3 of `docs/planning/design-notes/remaining-work-2026-09-30.md`).
- **Batch:** wave 3, second batch (#624 to #627): T3 and T6, the prompt.
- **Recall (§18.3):**
  - Plan D4 and its risk note ask for driven-keystroke checks of the ladder.
  - #481 already stops special keys and chords from reaching the terminal while its editor is open.
  - The order is #624, #625, #626, then this; #573 plugs into this editor's idle point.
- **Discovery:** one Explore sweep for these slices (2026-09-30); the spec's Prior art cites what
  applies here, and the Plan phase re-verifies each seam at promotion.

## Phase 1 — Plan (promoted 2026-09-30)
- **Promoted;** seams re-verified: #624's `rich_input::open_for(view, Target::Shell)` opens and
  focuses the footer editor and #625/#626 give it completions and colours; the agentless footer
  draws it; `AnchoredBlocks::at_prompt()` holds from a Precmd to the next Preexec; readline turns
  bracketed paste on at every prompt, so the gpui-era ladder's bracketed-paste rung would keep the
  editor away from bash entirely, and it is dropped; a Marley setting runs through
  `MarleySettingsContent`, `default.json`, the Settings page and `MarleySettings`.
- **Recall:** the queued notes stand. The brain (`rusty-cli brain ask`, consultation 44b9ef811fdc4613bcef339d0db3361e):
  nothing on this seam.
- **Scope cut:** ghost text and #557's hint inside the editor, and Ctrl+D, move to a follow-up
  (the spec's Out).

### Design
- **Setting** `marley.prompt_editor` (default true): `settings_content/src/marley.rs`,
  `default.json`, `settings_ui/src/marley_page.rs` (the Layout section), `MarleySettings.prompt_editor`.
- **`rich_input.rs`:** for each terminal view, an observer of its terminal (re-armed when the view's
  terminal changes, as #623's watch) opens the shell's editor and focuses it when the shell arrives
  at a prompt off the alternate screen while the view holds the focus, and closes it (focus back to
  the terminal) when a command starts or a full-screen program shows; the view's on-focus moves the
  focus into the editor while the shell waits. Escape marks the editor dismissed until the next
  prompt; Ctrl+C (`marley::ClearRichInput`, in the shell's editor) empties it.
- **Manifest:** the four setting files, `rich_input.rs`, `marley_workbench.rs` (the action),
  `keymap.json`, the ledger clauses for the three Zed paths, the scenario.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | bash at a prompt; `echo hi` typed | `prompt.png` |
| REQ-001 | Enter | `ran.png` |
| REQ-002 | `vim` typed and Enter; `:q` and Enter | `vim.png`, `back.png` |

## Phase 2 — Code (2026-09-30)
- **Built:** `marley.prompt_editor` (content, `default.json`, the Settings page's Layout section,
  `MarleySettings.prompt_editor: PromptEditor`); in `rich_input.rs`, the `Shells` global
  (`at_prompt`, `dismissed`), an observer of each view's terminal (`follow_prompt`) and its
  on-focus (`take_focus_at_prompt`), `close` marking the shell's editor dismissed, `clear` and
  `ClearRichInput` under a `MarleyShellInput` key context; `keymap.json`'s Ctrl-C.
- **Deviation:** no re-arm when a view's terminal changes (#623's watch): only task terminals are
  replaced, and they have no prompt.
- **Review:** both hooks run inside the view's update, so every open, close and focus change is
  deferred to the window; the focus goes back to the terminal only when the editor held it.
- **Clippy found:** a fifth bool in `MarleySettings` (now the `PromptEditor` enum),
  `needless_pass_by_ref_mut`, `from_settings` past 100 lines (`PromptEditor::from_setting`), the
  `Settings` trait's import; rustc: the Layout section's array length.
- **Gate:** GREEN, 17 PASS.


## Phase 3 — Test (2026-09-30)
- **Scenario:** `script/e2e/627-the-prompt-editor-by-default.sh` (sway). Two runs.
- **First run, red:** `ran.png` showed the block `echo hi` and the shell's cursor in the grid, with
  no editor. `echo hi` starts and ends between two of the terminal's notifies, so `follow_prompt`
  saw the shell at a prompt both times and never saw the prompt come again; the send had marked
  the editor dismissed, and nothing cleared it.
- **Fix:** `ShellState` keeps the terminal's block count too; a new block while the shell waits
  counts as a new prompt, which clears the dismissal and docks the editor. Gate GREEN, 17 PASS.
- **Second run:**
  - `prompt.png` (REQ-001): the editor docked under the terminal holding `echo hi` in the shell's
    colours; the shell's own line empty.
  - `ran.png` (REQ-001): the block `$ echo hi` / `hi` with its check, the rail's `echo hi · done`,
    and the editor back, empty, with its placeholder; the shell's cursor hollow.
  - `vim.png` (REQ-002): vim full screen, no editor in the footer; `:q` went to vim raw.
  - `back.png` (REQ-002): vim gone, the blocks `echo hi` and `vim`, the editor back empty.
- **Not shot:** REQ-003 (Escape until the next prompt) is verified by review: `close` sets
  `dismissed`, `take_focus_at_prompt` skips a dismissed shell, and only a new prompt clears it.

## Phase 4 — Complete (2026-09-30)
- **Documented:** `CHANGELOG.md`; the guide (Rich input, the keys table, Settings); `marley_workbench.md`;
  the plan's T3 row; the ledger rows for `settings_content`, `default.json` and `settings_ui`.
- **Knowledge:** F-claude-627-a-quick-commands-prompt-went-unseen-001,
  L-claude-627-count-what-happened-instead-of-sampling-a-flag-001.
- **Brain:** the consultation closed with `brain decide`.
- **Follow-up:** ghost text and #557's hint inside the editor, and Ctrl+D, stay out of scope
  (the spec's Out section).
