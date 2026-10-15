# Keys only the terminal binds, pressed in the prompt editor — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-784-terminal-keys-from-the-prompt-editor.md
- **Pipeline spec:** 784-terminal-keys-from-the-prompt-editor.spec.md

## Phase 1 — Plan (2026-10-15)
- **Request:** #783's sweep: keys the Terminal context binds and the Editor context leaves
  unbound likely reach the program behind the prompt editor. Chad approved the plain sentence on
  2026-10-15 and said to plan and carry it through.
- **Classification / tier:** bug, prong 1 T3; one Marley crate, no Zed crate.
- **Recall (§18.3):**
  - `PR-claude-an-editor-inside-a-terminal-rebinds-the-terminal-keys-it-must-keep-001` (#635):
    gpui ranks an unscoped binding at the deepest depth, so inside the editor it outranks the
    Terminal context's override. That is why Ctrl-Q quits and Ctrl-O opens files here.
  - `PR-claude-783-an-editor-in-a-terminal-keeps-the-keys-its-actions-pass-on-001`: list the
    Terminal context's keys, find each one's winner in the editor, bind the ones that fall through
    in the element's own context. Done below with a script over both keymaps.
  - `L-claude-491-ctrl-q-in-a-terminal-goes-to-the-shell-001`: on the grid Ctrl-Q goes to the
    shell; a scenario quits through the palette.
  - The brain (`rusty-cli brain ask`, consultations 7dd57795ef0849bf9a32e22abf20b132 and
    b9e0363d6f514822a859681d9d6ad4b8) returned nothing on this seam.
- **Discovery.** A script (scratchpad `k784/keys.py`) parsed `assets/keymaps/default-linux.json`
  and `crates/marley_workbench/keymap.json`, evaluated each binding's context the way gpui does
  (`depth_of`, `!` over the whole stack, `>` as descendant, no context as the deepest), and for
  each key of the Terminal blocks named the winning binding in `Workspace > Pane > Terminal >
  MarleyRichInput (> MarleyShellInput) > Editor{mode=auto_height}`. Chad's settings use the Zed
  base keymap and no keymap of his own. The keys that do not stay in the editor:
  - to the program: Ctrl-W, Ctrl-E, Ctrl-B, Ctrl-R (`SendKeystroke`), Alt-B, Alt-F (`SendText`),
    Ctrl-Shift-V (`terminal::Paste`);
  - to Zed's unscoped bindings: Ctrl-O (`workspace::OpenFiles`), Ctrl-Q (`zed::Quit`);
  - Ctrl-Shift-C runs `terminal::Copy`: the grid's selection, never the editor's;
  - harmless: the rest go to Zed's or Marley's own actions (`buffer_search::Deploy`,
    `terminal::RerunTask`, `pane::SplitRight`, the block keys, bookmarks, Filter, Ctrl-G, and
    `editor::SelectAll`, which the editor itself handles). The footer's `on_key_down` already
    stops every unbound chord from the terminal's `key_down`.
  - The Editor's own actions that propagate (`move_up`/`move_down` at an edge, fixed in #783;
    the rest only in a single-line editor, or with a menu or an LSP task) leave no other key.
- **Reproduction** (scratchpad `k784/784-repro.sh`, `just e2e` on the #783 debug build, a
  headless sway): the shell's grid printed `LEAK ctrl-e`, `LEAK ctrl-b`, `LEAK ctrl-r`, `LEAK
  alt-b`, `LEAK alt-f` (readline's tty binding of Ctrl-W keeps it from the `bind -x`, so the shell
  showed no line for it); Ctrl-Shift-V put `PASTED-MARKER` on the grid's prompt line; Ctrl-Shift-C
  with the editor's text selected left the clipboard as it was. The stand-in agent got
  `$'\027'`, `$'\005'`, `$'\002'`, `$'\022'`, `$'\E'` `b`, `$'\E'` `f` and the pasted text, while
  the rich input kept `x`. Ctrl-Q in the agent's rich input quit Marley at once (`timed out
  waiting on app_will_quit` in its log, the next shot black), with the stand-in Claude Code on the
  rail. Ctrl-O showed nothing in the headless sway, which has no file chooser portal.
- **Decisions:** D1 to D5 in the spec.

### Design
- **Approach.** `keymap.json` gets one block, `MarleyRichInput > Editor`, with a comment on why:
  inside a terminal view the editor meets the Terminal context's keys and Zed's unscoped ones.
  It binds `ctrl-w`, `ctrl-e`, `ctrl-b`, `alt-b`, `alt-f` to Zed's editor actions with the
  arguments Zed's own Ctrl-Backspace, End, Ctrl-Left and Ctrl-Right use; `ctrl-r`, `ctrl-o`,
  `ctrl-q` to `null`; `ctrl-shift-v` to `marley::RichInputPaste`; `ctrl-shift-c` to
  `marley::RichInputCopy`. The existing block of that context gains the rows, so the keys sit
  with Enter, Escape and Ctrl-Shift-W.
- Two actions in `marley_workbench.rs`'s `actions!` list beside `RichInputUp` and
  `RichInputDown`, with doc comments.
- `rich_input::element` adds two `on_action` listeners, each calling a function on the view as
  `up` and `down` do:
  - `paste(view, window, cx)`: the prompt's editor and target; the clipboard read once
    (`cx.read_from_clipboard()`); its first entry an image with bytes and the target an agent:
    `cx.propagate()`, so the Terminal's `terminal::Paste` runs and forwards Ctrl-V; otherwise
    `editor.paste(&Paste, window, cx)` and stop. An image in the shell's editor pastes nothing.
  - `copy(view, window, cx)`: the editor has a non-empty selection: the editor's own
    `editor::Copy`, dispatched to the editor's focus handle (`Editor::copy` is `pub(super)`);
    otherwise `cx.propagate()`, so `terminal::Copy` copies the grid's selection.
- The module doc gains a paragraph on the keys.
- §14: no IO, no panics, no spawn.
- **File manifest:**
  - `crates/marley_workbench/keymap.json` — Marley crate.
  - `crates/marley_workbench/src/marley_workbench.rs` — Marley crate (the two actions).
  - `crates/marley_workbench/src/rich_input.rs` — Marley crate.
  - `script/e2e/784-terminal-keys-from-the-prompt-editor.sh` — the scenario (Test phase).
  - No path outside the Marley-owned set, so no row in `docs/marley/zed-touchpoints.md`.

### Visual check plan
`script/e2e/784-terminal-keys-from-the-prompt-editor.sh`, `compositor sway`. Setup: `HOME` whose
`.bashrc` sets `PS1='$ '` and `stty -ixon`, binds each key with `bind -x` to print `LEAK <key>`,
and defines `stand_in`, a stand-in Claude Code (`exec -a claude`) in raw mode that prints `claude
got: %q` for each byte. After each group, `mcp_agent terminal-screen ""` saves the screen's rows
and the check is that no `LEAK` line is there.

| REQ | What the scenario does | Shot or check |
|---|---|---|
| 001 | type `one two`, Home, Ctrl-E, type `X`, Ctrl-B, type `Y`, Alt-B, type `Z`, Alt-F, type `W` | `784-01-moves`: the editor reads `one ZtwoYXW`; no `LEAK` |
| 002 | Ctrl-W | `784-02-ctrl-w`: `one `; no `LEAK` |
| 003 | Ctrl-R, Ctrl-O, Ctrl-Q | `784-03-nothing`: `one ` kept, Marley's window up; no `LEAK` |
| 004 | `wl-copy PASTED`, Ctrl-Shift-V | `784-04-paste`: `one PASTED`, the grid's line empty; no `LEAK` |
| 005 | Ctrl-A, Ctrl-Shift-C | `wl-paste` gives `one PASTED` |
| 006 | — | review of the diff: nothing in a headless sway selects grid text at a known place reliably while the editor docks; the propagation is the one #637 and #550 use |
| 007 | Ctrl-C, `stand_in`, Enter, Ctrl-G, type `ab cd`, Ctrl-W, Ctrl-E, Ctrl-B, Alt-B, Alt-F, Ctrl-R, Ctrl-O, `wl-copy EF`, Ctrl-Shift-V | `784-06-agent`: the rich input holds the edited line; the stand-in printed only `ready` |
| 008 | a PNG on the clipboard (`wl-copy --type image/png`), Ctrl-Shift-V | `784-07-image`: the stand-in got `$'\026'` |
| 009 | Enter | the stand-in's bytes are the editor's text and `$'\r'` |
| 003 | Ctrl-G, Ctrl-Q | `784-08-still-running`: Marley's window, the agent still on the rail |

Ctrl-O's file dialog does not show in a headless sway, so its `null` rests on the review and on
the editor keeping its text after the key.

### Risks
- `null` in Marley's keymap suppresses a binding only of the same source or weaker: Marley's
  keymap is tagged `KeybindSource::Default`, as Zed's default keymap is, and loads after it, so at
  the same depth it comes first (`resolve_bindings_for_input`). A user binding for one of these
  keys still wins, as it should.
- `editor::Copy` dispatched from inside an action listener runs after the current dispatch
  (`Window::dispatch_action` defers); the editor holds the focus throughout, so it lands there.
- An agent whose own editor key Marley uses (#649) never opens this editor, so it is unaffected.

## Phase 2 — Code (2026-10-15)
- **Built:** `keymap.json`'s `MarleyRichInput > Editor` block gains the eleven rows with a comment
  on why; `marley::RichInputPaste` and `marley::RichInputCopy` in `marley_workbench.rs`;
  `rich_input.rs` gains `editor_of` (the prompt's editor and target, which `arrow` now uses too),
  `paste`, `copy`, the footer's two listeners and the module doc's paragraph.
- **Deviations from the plan:** none.
- **Review of the diff:** against REQ-001 to REQ-009.
  - `paste` reads the clipboard once; an empty clipboard falls to `Editor::paste`, which does
    nothing. An image stops in the shell's editor and propagates only from an agent's.
  - `copy` reads the selection inside `editor.update` (`display_snapshot` takes `&mut App`), and
    dispatches `editor::Copy` after the listener returns (`Window::dispatch_action` defers), so no
    entity is updated inside its own update. The terminal view's `editor_copy` is a bubble-phase
    listener on an ancestor, so the editor's own copy answers first.
  - The `null` rows and Marley's keymap share the default source, and Marley's loads later, so
    at the same depth they suppress the unscoped `zed::Quit` and `workspace::OpenFiles` and the
    Terminal's `SendKeystroke`. A user's own binding still wins.
  - Nothing from Warp; readline's meanings through Zed's own editor actions.
- **Gate:** run 1, `GATE GREEN [diff]`, all 17 rows PASS, receipt written.

## Phase 3 — Test (2026-10-15)
- **Build:** `just build` at `nice 19`, 1 m 00 s.
- **Scenario:** `just e2e script/e2e/784-terminal-keys-from-the-prompt-editor.sh`, a headless
  sway, `SHOT_DIR` in the scratchpad. Run 1 failed only its last check: the stand-in got the
  return as `$'\n'`, because bash's `read -n` turns the tty's CR-to-NL translation back on (the
  `-icrnl` in its `stty` did not hold). The scenario now expects the newline and drops `-icrnl`;
  no source change. Run 2: exit 0, all twelve checks pass. Focus report: Hyprland 1 Marley window
  before and 1 after, no rule added; sway stopped with the run's Marley.
- The reproduction's window check (`sway_marley_pid >/dev/null`) passed though Marley had quit:
  the helper prints nothing and exits 0 with no window. The scenario tests that it prints a pid.
- **Shots, each read (the prompt area cropped and enlarged):**
  - `784-01-moves` (REQ-001): the editor reads `one ZtwoYXW`: Ctrl-E went to the end (`X`),
    Ctrl-B back one (`Y` before `X`), Alt-B to the word's start (`Z`), Alt-F to its end (`W`).
    The grid's `$ ` line is empty, and the screen holds no `LEAK`.
  - `784-02-ctrl-w` (REQ-002): `one `, the word deleted; no `LEAK`.
  - `784-03-nothing` (REQ-003): after Ctrl-R, Ctrl-O and Ctrl-Q, `one ` with the cursor after it,
    Marley's window up; no `LEAK`.
  - `784-04-paste` (REQ-004): `one PASTED`; the grid's line empty; no `LEAK`.
  - REQ-005: after Ctrl-A and Ctrl-Shift-C the clipboard holds `one PASTED`.
  - `784-05-shell-image` (D3): with a PNG on the clipboard, Ctrl-Shift-V leaves `one PASTED` and
    the shell gets nothing (no `LEAK ctrl-v`).
  - `784-06-agent` (REQ-007): the rich input holds `abEF` after `ab cd`, Ctrl-W, Ctrl-E, Ctrl-B,
    Alt-B, Alt-F, Ctrl-R, Ctrl-O and a paste of `EF`; the stand-in shows only `ready`, and its
    file is empty.
  - `784-07-image` (REQ-008): a PNG and Ctrl-Shift-V: the stand-in got `$'\026'`; the rich input
    still holds `abEF`.
  - `784-07-sent` (REQ-009): Enter: the stand-in got `a`, `b`, `E`, `F`, a space and the return,
    after the Ctrl-V; the rich input closed.
  - `784-08-still-running` (REQ-003): Ctrl-G, then Ctrl-Q in the rich input: Marley's window,
    the rich input open on its placeholder, the stand-in's lines unchanged.
- REQ-006 is the review's: `copy` propagates to `terminal::Copy` when the editor has no
  selection; no scenario selects grid text at a known place while the editor docks.
- The toast in the agent shots is Ctrl-G's shortcut note, unchanged.
- **Fixes:** none to the source. Pre-existing: none seen.
- **Gate:** run 2, with the scenario in the tree (the fingerprint covers `script/e2e`):
  `GATE GREEN [diff]`, every row PASS, receipt written.

## Phase 4 — Complete (2026-10-15)
- **Docs (§21):** `CHANGELOG.md` Fixed ("Shell keys stay in the prompt editor");
  `docs/marley_architecture/marley_workbench.md`, a #784 bullet after #783's and the scenario in
  the tests paragraph; `docs/marley/three-prong-plan.md`'s T3 row; `docs/marley/guide.md`, a
  paragraph on Up, Down and the keys after the prompt editor's, and four rows in the key table.
  No path outside the Marley-owned set needs a `zed-touchpoints.md` row (gate:16 green). The
  in-app guide (`crates/marley_workbench/guide/index.html`) has no prompt editor section, so it
  is left alone; its Ctrl+Q row speaks of the grid, which still sends the key to the shell.
- **Ledger (§19):**
  `F-claude-784-terminal-keys-in-the-prompt-editor-reached-the-program-or-quit-marley-001`,
  `L-claude-784-sway-marley-pid-prints-nothing-when-the-window-is-gone-001`,
  `L-claude-784-bash-read-n-turns-crnl-back-on-001`. No new prevention rule: #635's and #783's
  name this class, and the sweep they ask for is the one done here.
- **Brain:** consultation b9e0363d6f514822a859681d9d6ad4b8 closed with `brain decide`:
  `decisions/marleys-prompt-editor-and-rich-input-keep-the-terminals-keys-readline-moves-ctrl-roq-do-nothing`,
  follow-up by 2026-10-29; 7dd57795ef0849bf9a32e22abf20b132 closed with `brain no-decision`.
- **Ticket:** closed, moved to `tickets/closed/`; its BACKLOG row left at promotion.
- **Archive and commit:** the pair moved to `pipeline/completed/`; one commit carries the change,
  the scenario, the docs and this record.
