# Rich input: a Zed editor for an agent's prompt — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-481-rich-input.md
- **Pipeline spec:** 481-rich-input.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23: "there is Rich input which im not even sure what that is? DO
  you know?" Answered from Warp's docs: Warp's own editor in place of the agent's prompt box.
- **Recall.** PR-claude-break-the-code-a-driven-test-guards-before-trusting-it-001 and
  L-claude-453-a-key-context-test-must-press-a-key-only-that-context-binds-001: the Ctrl-G test
  must fail when the context is gone, so it checks the key reaches the program without an agent.
  The inline assistant (`agent_ui/src/terminal_inline_assistant.rs`) is the template for an
  editor over a terminal.

## Phase 1 — Plan, at promotion (2026-09-23)
- **Recall.** The brain (consultation 848c208c54144bbbb0e152d1dc771858) returned only unrelated
  follow-ups. Since #483 the proof is an e2e scenario, and keys are drivable, so every
  criterion but the button's click is driven.
- **Seams re-verified.** gpui matches keybindings before key listeners
  (`Window::dispatch_key_event`), so the editor's own keys (Backspace, the arrows, word motion)
  and this editor's Enter, Shift-Enter and Escape are handled before `TerminalView::key_down`,
  which sits on the view's root and sends the keys it maps to the PTY. Keys no binding takes
  fall through to it, except the characters it leaves to text input, so the editor's container
  stops the rest. An action handler that calls `cx.propagate()` lets the key go on to the
  terminal. Zed's default Linux keymap binds `ctrl-g` in editor and sidebar contexts, not in
  `Terminal`. `Editor::auto_height(min, max, window, cx)` is the inline assistant's shape.

### Design
- **`rich_input.rs`** (a new module): a global of the editors per terminal view (by entity id),
  each with whether it is open. `marley::RichInput`, bound to `ctrl-g` in `Terminal` and handled
  on every workspace: with an agent in the focused terminal it opens that terminal's editor
  (made once: auto height, one to eight lines, soft wrap, a placeholder naming the agent) and
  focuses it; without one it calls `cx.propagate()`, and the key reaches the program.
  `element(context, window, cx)` is the open editor inside a `MarleyRichInput` container that
  handles `marley::SendRichInput` (Enter: `Terminal::paste` of the text, then `\r`; the editor
  cleared and closed, the terminal focused) and `marley::CloseRichInput` (Escape: closed, the
  draft kept, the terminal focused), and stops the keys the terminal would otherwise take.
- **`agent_bar.rs`**: the footer is the editor above the bar; the bar gets a Rich Input button.
- **`keymap.json`**: `ctrl-g` in `Terminal`; `enter`, `shift-enter` (`editor::Newline`) and
  `escape` in `MarleyRichInput > Editor`.
- **Files:** `rich_input.rs` (new), `agent_bar.rs`, `marley_workbench.rs` (the actions, `init`),
  `keymap.json`, `script/e2e/481-rich-input.sh`. Marley only.

### E2E plan
| REQ | Shot |
|---|---|
| 001 | `481-01-bar`: the bar with Rich Input; `481-02-open`: after Ctrl-G, the editor above the bar with its placeholder |
| 003 | `481-03-two-lines`: `hello rich input`, Shift-Enter, `second line` in the editor; nothing printed |
| 002 | `481-04-sent`: after Enter, the editor gone, `claude got: hello rich input` and `claude got: second line` printed |
| 004 | `481-05-escaped`: `draft` typed, Escape: the editor gone, nothing printed; `481-06-draft`: Ctrl-G shows `draft` |
| 005 | `481-07-ctrl-g-passes`: after `quit`, no bar; `cat -v`, Ctrl-G, Enter prints `^G` |
| 006 | `just gate-diff` |

## Phase 2 — Code (2026-09-23)
- **Built.** `rich_input.rs`: the `Prompts` global (an editor and whether it shows, per terminal
  view, dropped with the view through `observe_release`), `init` (`marley::RichInput` on every
  workspace: with an agent in the focused terminal it opens that terminal's editor, and
  otherwise `cx.propagate()` lets the key reach the program), `open` and `new_editor` (auto
  height, one to eight lines, soft wrap, "A prompt for <agent>"), `send` (one `Terminal::paste`
  and `\r`, then the editor cleared; an empty one only closes), `close` (the draft kept, the
  terminal focused) and `element` (the open editor in a `MarleyRichInput` container that handles
  `marley::SendRichInput` and `marley::CloseRichInput`). `agent_bar.rs`: `agent_in`, shared with
  `contents`; the footer is a column with the editor above the bar; a pencil button, "Rich
  Input" with the key the action has. `keymap.json`: `ctrl-g` in `Terminal`; `enter`,
  `shift-enter` and `escape` in `MarleyRichInput > Editor`.
- **Review.** Clippy's `option_if_let_else` turned the lookup-or-create into
  `unwrap_or_else(new_editor)`; a borrow of the view across the terminal's update was split.
  `send` and `close` run from the container's action handlers, outside any entity update.

## Phase 3 — Test (2026-09-23)
- **First e2e runs, two findings.**
  - The stand-in, started as a subshell from `.bashrc`, never led the foreground process group
    (job control is not on while bash reads its startup files), so no agent showed and Ctrl-G
    reached it as `^G`. The scenario now `exec`s the stand-in in place of the shell.
  - Then the editor opened and grew with Shift-Enter, but typed characters never appeared: the
    container stopped every key unless `prefer_character_input` was set, and gpui's Linux
    platforms (`gpui_linux`: Wayland and X11) never set it, so no key reached the editor's text
    input. Enter then sent an empty editor, and Escape kept no draft. Fixed: the container
    stops only keys the terminal maps, chords with Ctrl, Alt or Super and keys that type
    nothing (`to_esc_str` maps no plain character). F-claude-481 records it.
- **E2E** (`SHOT_DIR=<scratchpad>/e2e481 just e2e script/e2e/481-rich-input.sh`; focus
  report: "the user's window and workspace are as they were"):
  - `481-01-ctrl-g-passes`: at the plain `$ ` prompt, `cat -v` then Ctrl-G and Enter show `^G`
    twice, the echo and cat's line; no agent bar (REQ-005).
  - `481-02-bar`: after `stand_in`, the bar reads "Claude Code", `+`, the pencil, the
    microphone and the plugin's chip.
  - `481-03-open`: after Ctrl-G, the editor above the bar with "A prompt for Claude Code"
    (REQ-001).
  - `481-04-two-lines`: "hello rich input" and, after Shift-Enter, "second line" in the editor;
    the terminal shows nothing new (REQ-003).
  - `481-05-sent`: after Enter, the editor gone; the terminal shows the tty's echo of both lines
    and `claude got: hello rich input`, `claude got: second line` (REQ-002).
  - `481-05-focus-back`: "typed after" and `claude got: typed after`: the terminal had the
    focus back (REQ-002).
  - `481-06-escaped`, `481-07-draft`: after Ctrl-G, "draft" and Escape, the editor closed with
    nothing printed; Ctrl-G shows "draft" again (REQ-004).
- **Not driven.** The pencil button's click; Ctrl-G does the same.
- **Gate.** `just gate-diff`: `GATE GREEN [diff]`, 15 gates and the receipt, which matches the
  tree.

## Phase 4 — Complete (2026-09-23)
- **Docs.** CHANGELOG (Added: rich input); `docs/marley_architecture/marley_workbench.md` (the
  Rich input section, the bar's order and the footer's column, the scenario); the plan's T7 row
  marks T7e shipped. No Zed path changed.
- **Knowledge.** F-claude-481-the-rich-input-dropped-every-typed-character-on-linux-001,
  L-claude-481-gate-text-by-its-modifiers-not-prefer-character-input-001.
- **Brain.** Consultation 848c208c54144bbbb0e152d1dc771858 closed with
  `marleys-rich-input-is-a-zed-editor-above-the-agent-bar`, follow-up by 2026-10-07.
