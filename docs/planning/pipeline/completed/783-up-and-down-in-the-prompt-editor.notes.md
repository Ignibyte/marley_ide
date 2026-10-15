# Up and Down in the prompt editor — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-783-up-and-down-in-the-prompt-editor.md
- **Pipeline spec:** 783-up-and-down-in-the-prompt-editor.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-14: Up in the prompt editor under the terminal acts on the alacritty
  grid, not on the editor.
- **Classification / tier:** bug, prong 1 T3; one Marley crate, no Zed crate.
- **Recall (§18.3):**
  - #637 bound → in `MarleyShellInput > Editor` to `marley::AcceptSuggestion` and let it propagate
    to `editor::MoveRight`; the same fall-through, one binding further down, is this bug.
  - `PR-claude-an-editor-inside-a-terminal-rebinds-the-terminal-keys-it-must-keep-001` (#635): an
    editor inside a terminal view meets the Terminal context's bindings.
  - `L-claude-628-check-a-copy-through-the-prompt-editor-with-copy-on-select-001`: a scenario's
    keys land in the prompt editor, which takes the focus at every prompt.
  - The brain (`rusty-cli brain ask`, consultation d7f103213aa049e399701ca5752d6867) returned
    nothing on this seam.
- **Discovery:** `crates/marley_workbench/src/rich_input.rs` (the footer's `element`, `Prompt`,
  `edited`, `open_for`, `send`); `crates/marley_workbench/keymap.json` (the
  `MarleyRichInput > Editor && !showing_completions` block); the actions in
  `crates/marley_workbench/src/marley_workbench.rs`; the history in `autosuggest::history`. Zed:
  `Editor::move_up` and `move_down` propagate when the cursor stays put; `move_page_up` and
  `move_page_down` propagate only in single-line editors, so PageUp and PageDown do not leak from
  this auto-height editor.
- **Decisions:** D1 to D6 in the spec.

### Design
- **Approach.** Two actions in `marley_workbench.rs`'s `actions!` list, `RichInputUp` and
  `RichInputDown`, documented as the keys of the rich input. `keymap.json` binds `up` and `down` to
  them in the existing `MarleyRichInput > Editor && !showing_completions` block, whose comment says
  why: Zed's editor passes these keys on when its cursor cannot move, and inside a terminal view the
  next binding sends them to the terminal. The footer in `rich_input::element` adds two
  `on_action` listeners that call `up` and `down` on the view.
- `Prompt` gains `walk: Option<Walk>`, where `Walk { commands: Vec<String>, index: usize, draft:
  String }`. `open_for` starts each opening with none.
- `up(view, window, cx)`: read the prompt; when it is the shell's, its editor has one selection and
  the selection's head sits on display row 0 of `editor.display_snapshot(cx)`, step the walk older:
  with no walk, build it from `autosuggest::history` (blank commands and repeats of a newer one left
  out) and keep the editor's text as the draft; show `commands[index]` with `set_text` and
  `move_to_end`. An empty history or the oldest command leaves the editor as it is. Otherwise call
  `editor.move_up(&MoveUp, window, cx)`. Then `cx.stop_propagation()` in every case, since
  `move_up` asks for propagation when its cursor stays put.
- `down`: the same with the last display row (`max_point().row()`); a walk steps newer, and past
  index 0 it ends and gives back the draft. With no walk the editor stays as it is.
- `edited` (the shell's `BufferEdited`) ends a walk whose recalled command no longer matches the
  editor's text. The walk's own `set_text` emits `BufferEdited` after the index has moved, so its
  text matches and the walk goes on.
- `send` and `close` leave the walk; the next `open_for` replaces the `Prompt`, so it starts empty.
- §14: no panics (indexing through `get`), errors none (no IO). No spawn.
- **File manifest:**
  - `crates/marley_workbench/src/rich_input.rs` — Marley crate.
  - `crates/marley_workbench/src/marley_workbench.rs` — Marley crate (the two actions).
  - `crates/marley_workbench/keymap.json` — Marley crate.
  - `script/e2e/783-up-and-down-in-the-prompt-editor.sh` — the scenario (Test phase).
  - No path outside the Marley-owned set, so no ledger row in `docs/marley/zed-touchpoints.md`.

### Visual check plan
`script/e2e/783-up-and-down-in-the-prompt-editor.sh`, `compositor sway`. Setup: `HOME` with
`.bashrc` setting `PS1='$ '` and a `stand_in` function (#481's), and `.bash_history` of
`git status`, `ls -la`, `echo hello world`, `ls -la`; a scratch repo opened. A click on the
terminal docks the editor.

| REQ | What the scenario does | Shot |
|---|---|---|
| 005 | Down in the empty editor | `783-06-down-alone`: editor empty, grid prompt line empty |
| 001 | type `draft words`, press Home, Up | `783-01-up`: editor `ls -la`, grid line empty |
| 002 | Up, Up | `783-02-older` (`echo hello world`), `783-03-oldest` (`git status`, the repeat skipped) |
| 003 | Up | `783-04-stays`: still `git status` |
| 004 | Down three times | `783-05-draft`: `draft words` back |
| 006 | Ctrl-C, Up, Up, Enter | `783-07-ran`; MCP `ran "echo hello world"` |
| 007 | type `first`, Shift-Enter, `second`, Up | `783-08-line-up`: both lines kept, cursor on the first |
| 008 | Ctrl-C, Up, type ` again`, Up, then Down | `783-09-walk-again` (`echo hello world`, the newest), `783-10-edited-back` (`echo hello world again`) |
| 009 | Ctrl-C, `stand_in`, Enter, Ctrl-G, type `after up`, Up, Down, Enter | `783-11-agent`; MCP: the stand-in got exactly `after up` |
| 010 | — | review of the diff: no scenario opens Zed's completion menu reliably at a known entry |

The shell's grid line is seen behind the editor in each shot: a leaked Up would show a history
command there. The stand-in reads in cooked mode, so a leaked Up would put `^[[A` in the line it
got.

### Risks
- gpui ranks bindings at the same depth by keymap order. Marley's keymap loads after Zed's, so
  `up` in `MarleyRichInput > Editor` outranks `Editor`'s `editor::MoveUp`, as #637's → outranked
  `editor::MoveRight`. The `!showing_completions` gate is what lets `editor::ContextMenuPrevious`
  keep the key while the menu shows; without it Marley's binding would win there too.
- `display_snapshot` takes `&mut App`; the handler reads it inside `editor.update`, so no second
  borrow of the editor.
- An agent whose own editor key Marley uses (#649) never opens this editor, so it is unaffected.

## Phase 2 — Code (2026-10-14)
- **Built:** `marley::RichInputUp` and `marley::RichInputDown` in `marley_workbench.rs`;
  `keymap.json` binds `up` and `down` to them; `rich_input.rs` gains `Prompt::walk`, `Walk`,
  `Toward`, `up`, `down`, `arrow`, `at_edge`, `walk`, `step`, `distinct` and
  `end_walk_if_edited`, the footer's two listeners, and the module doc's paragraph.
- **Deviations from the plan:**
  - The keys got a keymap block of their own, gated
    `!showing_completions && !showing_code_actions`: Zed gives Up and Down to its code-actions
    menu as well as the completion menu (`Editor && (showing_code_actions ||
    showing_completions)`), and the shared Enter/Tab block keeps its own gate unchanged.
  - The last line is the display row of the buffer's end (`max_point().to_display_point`), not
    the display snapshot's `max_point`: the history suggestion is an inlay after the text and can
    wrap onto a row of its own, which would make Down on the text's last row move instead of walk.
  - `Cargo.lock`: the gate's cargo-audit went red on 15 advisories RustSec published today against
    `rustls` 0.23.40 (RUSTSEC-2026-0346, -0348, -0349) and the wasmtime 48.0.5 family
    (RUSTSEC-2026-0352 to -0364), all in upstream's tree and none from this change. Fixed as #511
    and #642 did (L-claude-642): `cargo update` naming every wasmtime, pulley and cranelift crate
    (48.0.6 and 0.135.6; the wasm-tools crates it asks for were already at 0.254.2), and
    `cargo update -p rustls@0.23.40 --precise 0.23.46`, which also moved `rustls-webpki`
    0.103.15, `aws-lc-rs` 1.18.2 and `aws-lc-sys` 0.46.0. `cargo audit` then exits 0. The bump
    rides in this ticket's commit, since the receipt binds the lockfile.
- **Review of the diff:** against REQ-001 to REQ-010. No entity is updated inside its own update
  (`at_edge` and `walk` run from the footer's action listener; the history closure only reads).
  The walk goes back into `Prompts` before `set_text`, so the `BufferEdited` it raises sees the
  recalled command and keeps the walk; Down past the newest stores none first, so the draft's edit
  ends nothing. No indexing outside `get`. The agent's editor never walks. The walk follows the
  behavior of Zed's inline-input history (a draft kept, an edit ends the walk); no code of
  `cycle_inline_input_history` was carried over. Nothing from Warp.
- **Gate:** run 1 red on gate:7 only (the advisories above), every other gate green. Run 2, with
  the lockfile bump: `GATE GREEN [diff]`, all 17 rows PASS, receipt written. The clippy scope
  builds no wasmtime or aws-lc; the Test phase's `cargo build -p zed --bin marley` compiles them.

## Phase 3 — Test (2026-10-14)
- **Build:** `just build` at `nice 19`, 2 m 58 s; it compiled the bumped wasmtime 48.0.6 and
  aws-lc-sys 0.46.0 crates, so the lockfile bump builds.
- **Scenario:** `just e2e script/e2e/783-up-and-down-in-the-prompt-editor.sh`, a headless sway,
  `SHOT_DIR` in the scratchpad; exit 0. MCP checks: `ran "echo hello world"` pass; the stand-in
  got exactly `claude got: after up` pass. Focus report: Hyprland 1 Marley window before and 1
  after, 6 clients before and after, no rule added; sway stopped with the run's Marley.
- **Shots, each read (the prompt area cropped and enlarged):**
  - `783-06-down-alone` (REQ-005): the editor empty with its placeholder, the grid's `$ ` line
    empty.
  - `783-01-up` (REQ-001): `ls -la`, the newest, though Home had put the cursor at the start of
    `draft words`; the grid's line empty.
  - `783-02-older`, `783-03-oldest` (REQ-002): `echo hello world`, then `git status`: the file's
    second `ls -la` left out.
  - `783-04-stays` (REQ-003): still `git status`, the cursor at its end; the grid's line empty.
  - `783-05-draft` (REQ-004): three Downs give back `draft words`, with #557's hint after it.
  - `783-07-ran` (REQ-006): the block `echo hello world` with its output `hello world` and a green
    check; the rail's row `echo hello world · done`; the editor back, empty. Nothing else ran.
  - `783-08-line-up` (REQ-007): `first` and `second` unchanged after Up from the second line.
    `783-08-then-walks`: the next Up shows `echo hello world` (the session's command, newest), so
    the first Up had put the cursor on the first line. `783-08-draft-back`: Down gives back both
    lines.
  - `783-09-walk-again` (REQ-008): after `echo hello world` was recalled and ` again` typed, Up
    shows `echo hello world`, the newest, not the older `ls -la`: the edit ended the walk.
    `783-10-edited-back`: Down gives back `echo hello world again`.
  - `783-11-agent` (REQ-009): the stand-in's block reads `ready` with an empty line under it; the
    rich input holds `after up` after Up, Up, Down, Down. `783-11-agent-sent`: the block reads
    `after up` and `claude got: after up`, no `^[[A`. The toast is Ctrl-G's shortcut note,
    unchanged.
- REQ-010 is the review's: the keymap block is gated `!showing_completions &&
  !showing_code_actions`; no scenario opens a menu at a known entry in this editor.
- The editor's cursor shows in some shots and not others (its blink); the text in each shot
  carries the criterion.
- **Fixes:** none needed. Pre-existing: none seen.
- **Gate:** run 3, with the scenario in the tree (the fingerprint covers `script/e2e`):
  `GATE GREEN [diff]`, every row PASS, receipt written.

## Phase 4 — Complete (2026-10-14)
- **Docs (§21):** `CHANGELOG.md` Fixed ("Up and Down stay in the prompt editor") and Security
  ("rustls 0.23.46 and wasmtime 48.0.6"); `docs/marley_architecture/marley_workbench.md`, a #783
  bullet in "The prompt editor by default" and the scenario in the tests paragraph;
  `docs/marley/three-prong-plan.md`'s T3 row. No path outside the Marley-owned set needs a
  `zed-touchpoints.md` row (gate:16 green).
- **Ledger (§19):** `F-claude-783-up-in-the-prompt-editor-reached-the-shell-behind-it-001`,
  `PR-claude-783-an-editor-in-a-terminal-keeps-the-keys-its-actions-pass-on-001`. No lesson: the
  advisory bump repeats L-claude-642, and a scenario changing the receipt is already a lesson.
- **Follow-up:** the PR's sweep of the keymap found keys the Terminal context binds and the Editor
  context leaves unbound (`ctrl-r`, `ctrl-w`, `ctrl-e`, `ctrl-b`, `ctrl-o`, `ctrl-q`, `alt-b`,
  `alt-f`, `ctrl-enter` …), which from the prompt editor likely reach the shell. Not seen in the
  app; filed as #784 at the top of the Queue. PageUp and PageDown do not fall through: their
  editor moves propagate only in a single-line editor.
- **Brain:** consultation d7f103213aa049e399701ca5752d6867 closed with `brain decide`:
  `decisions/up-and-down-in-marleys-prompt-editor-walk-the-shells-history-and-never-reach-the-grid`,
  follow-up by 2026-10-28.
- **Ticket:** closed, moved to `tickets/closed/`; no BACKLOG row was ever added for #783.
- **Archive and commit:** the pair moved to `pipeline/completed/`; one commit carries the change,
  the lockfile bump, the scenario, the docs and this record.
