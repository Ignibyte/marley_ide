# History suggestions in the prompt editor — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-637-history-suggestions-in-the-prompt-editor.md
- **Pipeline spec:** 637-history-suggestions-in-the-prompt-editor.spec.md

## Phase 1 — Plan (promoted 2026-10-01)
- **Request:** Chad, 2026-10-01: "lets do 637, 631, 540", run autonomously (a session goal).
- **Classification / tier:** feature, prong 1 T3, Marley-owned files only.
- **Checklist:** pick ✓, pre-flight ✓ (gate, e2e, shear, hooks, no active pipeline, README marker
  present, cargo idle), recall ✓, mint ✓, prior art ✓, spec ✓, design ✓.
- **Recall (§18.3):**
  - AD-claude-484: the typed text read from the grid; the history is the terminal's verified
    commands newest first, then the history file; → bound to an action that propagates without a
    suggestion.
  - AD-claude-573 and #573's design: the editor's hint is an `Inlay::edit_prediction` with the
    reserved id `usize::MAX - 573`, painted from the editor's edit subscription; the prompt editor
    has no edit-prediction provider, so nothing else removes it.
  - F-claude-634 (focus containment) and F-claude-635 (Ctrl-Shift-W): an unscoped or Terminal
    binding can outrank the editor's; a binding for the shell's editor goes in
    `MarleyShellInput > Editor`.
  - Brain (`rusty-cli brain ask`, consultation 88387bf8ce064c3facf0087868bd8ad1): nothing on this
    seam.
- **Discovery:** `autosuggest.rs` (`suggestion`, `history`, the workspace's AcceptSuggestion),
  `rich_input.rs` (`new_editor`'s subscription, `paint_hint`, the footer's actions),
  `english.rs::hint_for`, `keymap.json`'s `MarleyShellInput > Editor`. gpui's `dispatch_key_event`
  dispatches each matching binding in turn while the handler propagates (`window.rs`, the
  `match_result.bindings` loop), so a propagating handler lets `editor::MoveRight` run.

### Design
- **`autosuggest.rs`:** `pub(crate) fn suggestion_for(text, terminal, cx) -> Option<String>`, the
  lookup over `history(terminal, cx)` for a given text; `suggestion` uses it with the grid's typed
  text. The workspace's AcceptSuggestion handler propagates while `rich_input::holds_line_of` the
  terminal: the editor has the line, and the grid's typed text (kept there until the editor sends)
  must not be completed behind it.
- **`rich_input.rs`:**
  - `paint_hint` (shell only): when the editor has one empty selection at the end of its text,
    the history suggestion for the text, else `english::hint_for`'s hint; the warning colour only
    with the hint. The same inlay id.
  - `new_editor`'s subscription repaints on `EditorEvent::SelectionsChanged` too (the hint follows
    the cursor), through a `repaint` that skips `typed_line::changed`.
  - The footer's `on_action(AcceptSuggestion)`: with the suggestion showing (recomputed, D2),
    `editor.insert(rest)`; otherwise `cx.propagate()`.
- **`keymap.json`:** `"right": "marley::AcceptSuggestion"` in `MarleyShellInput > Editor`.
- **`marley_workbench.rs`:** AcceptSuggestion's doc names the editor.
- **File manifest:** all Marley: `crates/marley_workbench/src/autosuggest.rs`, `rich_input.rs`,
  `marley_workbench.rs`, `keymap.json`; `script/e2e/637-history-suggestions-in-the-prompt-editor.sh`.
  No Zed crate, no ledger row.

### Visual check plan
| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | sway; trust; click the terminal (the editor docks); type `ech` | `637-01-ghost`: `ech` and dimmed `o hello world` |
| REQ-002 | → | `637-02-taken`: `echo hello world`, no ghost |
| REQ-003 | Enter | `637-03-ran`: the block `echo hello world` and its output; MCP `ran` |
| REQ-004, REQ-006 | type `zzz` | `637-04-nothing`: `zzz`, no ghost |
| REQ-005, REQ-006 | Ctrl-C; `git st`, Left | `637-05-left`: cursor before `t`, no ghost |
| REQ-005 | → | `637-06-right`: cursor at the end, `git st` + ghost `atus` (not taken) |
| REQ-007 | Ctrl-C; `echo from this session`, Enter; `ec` | `637-07-session-first`: ghost `ho from this session` |
| REQ-008 | review | the order in `paint_hint` |
Not reachable by a shot: that nothing reached the grid on → (the grid line under the docked editor
stays empty in `637-04`..`06`; the review covers the guard).

### Risks
- `Editor::insert` with a single cursor at the end inserts at the cursor; the review checks it
  does not auto-close brackets in a suggestion (it is `insert`, not `handle_input`).
- The Terminal context's own `right` binding is a later binding in the same dispatch; with the
  editor's MoveRight before it, it is never reached from the editor.

## Phase 2 — Code (2026-10-01)
- **Built:** `autosuggest::suggestion_for` (the lookup for a given line; the grid's `suggestion`
  uses it) and the workspace handler's guard (`rich_input::holds_line_of` → propagate);
  `rich_input.rs`'s `shell_editor`, `shell_hint` (the history's suggestion, else `hint_for`),
  `history_suggestion` (one cursor at the end, nothing selected), `accept_suggestion` (the
  footer's `AcceptSuggestion`: `Editor::insert`, else `cx.propagate()`), and the subscription's
  `SelectionsChanged` repaint; `keymap.json`'s `right` in `MarleyShellInput > Editor`;
  AcceptSuggestion's doc; the scenario.
- **Deviations:** none.
- **Review:**
  - REQ-001: `edited` paints after the editor's update ends, when the cursor already sits after
    the typed text; the `SelectionsChanged` repaint covers a cursor move with no edit.
  - REQ-002/REQ-006: `accept_suggestion` recomputes the suggestion rather than trusting the inlay,
    so a stale inlay can never be typed; without one, the action propagates, the workspace's
    grid handler propagates too (the editor holds the line), and gpui dispatches the next
    binding, `editor::MoveRight`. Nothing reaches the terminal.
  - REQ-008: `shell_hint` asks the history first, `hint_for` only without a suggestion, the grid's
    order (#557); the suggestion carries no warning range.
  - Re-entrancy: the subscription runs after the editor's update; `history_suggestion` reads the
    editor and the terminal, and `paint_hint`'s update is the only write.
  - `Editor::insert` is `replace_selections` with block autoindent, no bracket autoclose; with the
    cursor at the end of one line it appends the rest as it is.
  - Provenance: Marley code only; the lookup is #484's.
- **Clippy found:** `option_if_let_else` in `shell_hint` (now `map_or_else`).
- **Gate:** `script/gates.sh --diff` GATE GREEN (17 of 17), the second run; the first was red on
  the clippy lint above. Both ran at `nice 19` while rustal-os ran a mutation pass in its own
  target folders (never `/mnt/fast/target`), so its timings were not taken from it.

## Phase 3 — Test (2026-10-01)
- **Build:** `cargo build -p zed --bin marley` at `nice 19` (run directly: `just build` waits for
  every cargo on the box, and rustal-os's mutation pass held one).
- **Scenario:** `script/e2e.sh script/e2e/637-history-suggestions-in-the-prompt-editor.sh`, a
  headless sway, `SHOT_DIR` in the scratchpad; exit 0, both MCP checks pass (`ran "echo hello
  world"`, `ran "echo from this session"`). Focus report: no Hyprland window before or after, no
  rule added; sway stopped with the run's Marley.
- **Shots, each read (cropped to the prompt where small):**
  - `637-01-ghost` (REQ-001): the editor under the grid reads `ech` in the command colour and
    `o hello world` dimmed after it; the grid's prompt `$ ` is empty.
  - `637-02-taken` (REQ-002): the editor reads `echo hello world` in the shell language's colours,
    nothing dimmed.
  - `637-03-ran` (REQ-003): the block `echo hello world …/repo` with its output `hello world` and a
    green check; the rail's row says `echo hello world · done`; the editor is back, empty, with its
    placeholder.
  - `637-04-nothing` (REQ-004, REQ-008's order): `zzz` and #557's hint `· ctrl-shift-enter asks
    the agent`, no history text: with no suggestion, the hint shows, as on the grid.
  - `637-05-left` (REQ-005, REQ-006): `git st` with nothing after it: the cursor left the end, the
    suggestion went; the grid's line stays empty.
  - `637-06-right` (REQ-005, REQ-006): `git st` and `atus` dimmed after it: → moved the cursor back
    to the end, which brought the suggestion back without taking it; the grid's line stays empty.
  - `637-07-session-first` (REQ-007): `ec` and `ho from this session` dimmed: the session's command
    over the file's `echo hello world`.
- The editor's cursor does not show in the shots (its blink was off at each capture); the
  suggestion's absence and return carry REQ-005 instead.
- **Fixes:** none needed. Pre-existing: none seen.

## Phase 4 — Complete (2026-10-01)
- **Docs (§21):** `CHANGELOG.md` (Added); `docs/marley/three-prong-plan.md` T3 (#637 shipped);
  `docs/marley_architecture/marley_workbench.md` (Autosuggestions, the prompt editor by default,
  the scenarios). No Zed path touched, so no touchpoint row.
- **Ledger:** L-claude-637-a-propagating-action-lets-the-keys-next-binding-run-001,
  AD-claude-637-the-prompt-editor-shows-the-history-suggestion-in-its-hint-inlay-001. No bug found,
  so no F- or PR- block.
- **Brain:** `brain decide` on consultation 88387bf8ce064c3facf0087868bd8ad1
  (`decisions/marleys-prompt-editor-shows-the-history-suggestion-in-its-hint-inlay`).
- **Ticket:** closed; archived to `completed/`; committed on `marley/workbench-shell`.
