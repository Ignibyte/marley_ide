---
pipeline_id: 359da68f-195c-4b6a-b979-93e37cce9fd6
ticket: docs/planning/tickets/closed/TICKET-789-marleys-keys-use-cmd-on-macos.md
status: Phase 4 — Complete PASS
title: "Marley's own keys use Cmd on macOS"
type: feature
slice: macOS
references: [docs/planning/pipeline/completed/790-the-agent-editor-with-a-space-in-the-data-folder.spec.md]
---

## Title
Marley's keymap (`crates/marley_workbench/keymap.json`) binds its own actions on Ctrl: Undo Close
Terminal, Take Over, Ask Agent, the block keys, the bookmark, and the browser's address bar, new
tab, reload and element picker. On a Mac, Zed's own actions answer Cmd, so Marley's sit beside
them on Ctrl. The move's plan lists this as the first gap Chad will feel.

## Scope
### In
- `crates/marley_workbench/keymap-macos.json` (new), loaded after `keymap.json` on macOS only,
  with Cmd twins of Marley's app keys:
  - Workspace: `cmd-shift-t` Undo Close Terminal.
  - Terminal: `cmd-i` Take Over, `cmd-shift-enter` Ask Agent, `cmd-shift-b` Toggle Bookmark.
  - Terminal with a selected block: `cmd-shift-i` Reinput, `cmd-shift-enter` Send to Agent,
    `cmd-shift-f` Find in Block.
  - The rich input: `cmd-v` and `cmd-c`, Marley's paste (an image to the agent) and copy (the
    editor's selection, else the terminal's). The shell input: `cmd-shift-enter` Ask Agent.
  - The Browser tab: `cmd-i`, `cmd-l`, `cmd-t`, `cmd-r`, `cmd-shift-c`, and `cmd-[` / `cmd-]` for
    back and forward. Its editor: `cmd-shift-c`.
  - Rusty's page: `cmd-d` bookmark, `cmd-[` / `cmd-]` back and forward.
- `crates/marley_workbench/src/marley_workbench.rs`: `load_keymap` binds the overlay after the
  keymap under `cfg(target_os = "macos")`.

### Out (explicitly deferred)
- Keys that stand for a terminal's own, which stay Ctrl on a Mac as in Terminal.app: the prompt
  editor's `ctrl-w`, `ctrl-e`, `ctrl-b`, the `ctrl-r`, `ctrl-o` and `ctrl-q` nulls, the shell
  input's `ctrl-c`.
- `ctrl-g` (the rich input, the agents' own key), `ctrl-tab`, and the `alt-` and `secondary-`
  bindings, which already read right on a Mac.
- Removing the Ctrl bindings on a Mac. They stay, since a terminal sends `ctrl-i` as Tab's own key
  and none of them takes a key a Mac user types for something else.

## Reference (§20)
Upstream Zed (`assets/keymaps/default-macos.json` beside `default-linux.json`): one platform's
keymap for each, with app actions on Cmd on a Mac and Ctrl elsewhere. Marley keeps that split with
an overlay instead of a second whole file, since its Ctrl bindings stay valid on a Mac. Browsers
on a Mac use Cmd+L, Cmd+T, Cmd+R, Cmd+[ and Cmd+], with Cmd+Shift+C for the element picker
(Chrome's).

### Prior art
- **The code we ship:**
  - `load_keymap_from` (`marley_workbench.rs:1119`) loads a keymap as a default source after
    Zed's;
  - gpui's `secondary-` modifier (Cmd on macOS) was weighed and left for the overlay: Cmd+V and
    Cmd+C in the rich input have no Ctrl twin on Linux, where Ctrl+C is the shell's.
- **Zed's Mac defaults:** the Cmd keys taken in deeper Marley contexts are Zed's at Workspace or
  Editor depth (`cmd-t` project symbols, `cmd-r` right dock, `cmd-shift-b` outline,
  `cmd-shift-c` collab panel, `cmd-shift-f` search), so Marley's contexts win where they apply,
  as Ctrl does on Linux. `cmd-shift-t` (Zed's Reopen Closed Item) is taken at Workspace as
  `ctrl-shift-t` is on Linux (`default-linux.json:695`).

## UI proof
On the Mac, where no keys can be typed over SSH (no Accessibility):
- `actions_list` through Marley's MCP: each overlay action lists its Cmd key beside the Ctrl one.
- Marley's log after the restart: no "the Marley keymap did not load".

On Linux the overlay is not compiled in; the run is `just shot`.

## Locked-In Decisions
- D1 — An overlay file, loaded after `keymap.json` on macOS, rather than rewriting bindings as
  `secondary-`. Ctrl stays on Linux untouched, and Mac-only meanings (Cmd+V, Cmd+[) have a home.
- D2 — The Ctrl bindings stay on a Mac (scope-out).
- D3 — The overlay loads through `load_keymap_from`, all or nothing like the main one, and fails
  with its own error.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE Marley runs on macOS, each overlay action shall be bound to its Cmd key | `actions_list` on the Mac |
| REQ-002 | WHILE Marley runs on macOS, the shell keys of the prompt editor shall stay on Ctrl | `actions_list`: `editor::DeleteToPreviousWordStart` keeps `ctrl-w` in `MarleyRichInput > Editor` |
| REQ-003 | WHEN Marley starts on macOS, its keymap and the overlay shall load without error | the Mac's log |
| REQ-004 | WHILE Marley runs on Linux, its bindings shall be as before | the review of the diff (`cfg`); `just shot`; the gate |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the overlay and its load; a review; the gate; clippy on the Mac.
- **P3 Test** — on the Mac: install, restart, `actions_list`, the log; on Linux: `just shot`.
- **P4 Complete** — CHANGELOG, the guide's Mac section, the crate note, close, archive, commit.
