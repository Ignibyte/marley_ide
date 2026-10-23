---
pipeline_id: 4c4d3adc-a1ff-4d4a-a4e9-868465ecf002
ticket: docs/planning/tickets/open/TICKET-917-terminal-prompt-and-session.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: The terminal prompt as a card, the sticky header, session chips
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/A-core.md, docs/warp_architecture/subsystems/03-terminal-session-core.md]
---

## Title
The shell's prompt editor becomes the boards' prompt card: chips for the folder, the branch, the
changed files and the last exit; `❯` before the line; "→ to accept" while a suggestion shows; a
row of key hints with "Start an agent here". The sticky header reads "<command> is above you ·
<state>" with a "Jump to it" button. The tab row's right end names the terminal's session, folder
and branch. A running block's header offers "Tell me when done".

## Scope
### In
- The prompt card (Marley: `agent_bar.rs` `footer_without_agent`, `rich_input.rs` `element`):
  - One card for the prompt editor and its place row: 14 px radius, a border, `editor_background`,
    a margin from the terminal's edges.
  - Its top row of chips: the folder (compact path) in the theme's `terminal_ansi_cyan`, the
    branch in `terminal_ansi_magenta`, "N changed" (the files the folder's repository reports
    changed, none at 0) muted, and "last: exit N" in the error colour while the newest block
    failed.
  - `❯` in the accent before the editor.
  - "→ to accept" at the line's right end while the history suggestion (#637) shows.
  - A hints row: "Enter runs", "Ctrl Shift Enter asks the agent", "↑ history", "Ctrl G a bigger
    editor", each key read from the keymap for its action (`SendRichInput`, `AskAgent`,
    `RichInputUp`, `RichInput`), so a rebound key shows as bound; and at the right "Start an agent
    here", which runs `marley::NewAgent { folder }` with the terminal's folder.
- The sticky header (Marley: `sticky_header.rs`): the command, then "is above you ·" and the
  block's state: "running · <elapsed>" in the running colour, "exit N" in the error colour, or
  "done in <duration>"; at the right a "Jump to it" text button, which scrolls to the block as the
  row's click does now. The elapsed time ticks once a second while the block runs (the per-view
  ticker TICKET-916 adds in `block_headers.rs`; this ticket adds it if 916 has not shipped).
- Session chips (Zed crate `crates/workspace/src/pane.rs`, Marley `tab_tiles.rs` or a new
  `session_chips.rs`):
  - `pane.rs`: a hook `MarleyTabBarEnd`, a global function the tab bar asks for an element drawn
    before its end buttons, given the pane's active item.
  - Marley's answer for a terminal tab: the session chip, then the folder and the branch as
    chips. The session chip says the shell's name ("bash"), "tmux · <session> · N windows" for a
    tab attached to a tmux session (#821), or "ssh · <host>" for a remote terminal, followed by
    "connected" or "link lost" from the link's state (#641). Nothing for other tabs.
- "Tell me when done" (Marley: `block_headers.rs`, `command_watch.rs`): a small button in a running
  block's header. A click marks the block; when it ends, `command_watch` posts its notification
  (`<command> finished · exit N`, with how long it took) whatever `marley.long_command_seconds`
  says, unless the user is looking at the terminal. A marked block's button reads "Will tell you";
  a second click unmarks it.

### Out (explicitly deferred)
- The prompt card when `marley.prompt_editor` is off: the shell draws its own prompt in the grid.
- The tmux window shown ("window 1 of 2"): the listing gives the count only (#913 says the same).
- Link latency ("connected · 18 ms"): Marley does not measure it; a follow-up if wanted.
- The session's own chip in the prompt card's row (TerminalSession draws it there too): the tab
  row carries it.
- The block map and the exit-127 suggestion: TICKET-918. The selected block's labelled toolbar
  and the start time in a header: the TerminalSession follow-up named in TICKET-916.
- Block frames, dots, the action row and the progress line: TICKET-916.
- The bottom row's branch, changed count and tmux detail: TICKET-908.

## Reference (§20)
Warp's terminal input and blocks (`docs/warp_architecture/subsystems/03-terminal-session-core.md`
§6: a block carries its command, state and prompt info, which the pinned header and the prompt's
context read) as observed behaviour, reimplemented on Marley's prompt editor (#624), sticky header
(#529) and block times; no Warp source read. Upstream Zed's pane tab bar keeps its buttons; the
hook only adds before them.

### Prior art
- `agent_bar.rs:164-230` `footer_without_agent` (the prompt editor, the place row with
  `place_chip`), `:112` `branch_of` (the folder's repository by the longest work directory).
- `rich_input.rs:900-967` `element` (the strip, its key context `MarleyShellInput`), `:367`
  the placeholder, `:452` `paint_hint` and `:485` `shell_hint` (the suggestion as an inlay).
- `crates/marley_workbench/keymap.json`: `ctrl-g` `marley::RichInput`, `ctrl-shift-enter`
  `marley::AskAgent`, `right` `marley::AcceptSuggestion`, and the `MarleyRichInput > Editor`
  bindings for Enter and Up.
- `agents.rs:842` `NewAgent { folder }`: with a folder, the picker asks only which agent.
- `project::git_store::RepositorySnapshot::cached_status()` (`git_store.rs:7010`) and
  `status_summary()` (:6378): the changed files.
- `sticky_header.rs:30-105` `header` (the command, a state pill, an arrow; a press jumps through
  `blocks::reveal`).
- `crates/workspace/src/pane.rs:3606-3633` `configure_tab_bar_start` (`end_children` from
  `render_tab_bar_buttons`); `:4347-4357` `MarleyTabMark`/`MarleyTabLook`, the pattern for a pane
  hook.
- `tmux_sessions.rs:148` `TmuxHosts::attach_of`, `:49` `TmuxSession::windows`; `close_guard.rs:166`
  `tmux_attach` (a tab's attach from its task label); `remote.rs:122` `is_remote`, `:524`
  `link_overlay` (the link's lost state).
- `command_watch.rs` (#551): `Watch`, `Told`, `notify`; `notifications.rs:206` `notify` (skips
  when the user looks at the terminal).
- Grep for "window .* of", "connected ·", "Tell me when done", "is above you": none.

## UI proof
Scenario `script/e2e/917-terminal-prompt-and-session.sh` (sway): project `repo` with two changed
files; a HOME whose `.bashrc` sets the prompt and whose history holds
`git log --oneline -5 --graph`; `marley.prompt_editor` on; a tmux server of the run's own under
`$XDG_RUNTIME_DIR` with a session `build` in `repo`, `marley.tmux_hosts` naming this machine; a
private session bus whose notification server logs each banner (#551's).
- `917-01-prompt`: `false` ran, then `git log --oneline` typed: the card with the folder, branch,
  "2 changed" and "last: exit 1" chips, `❯`, the suggestion and "→ to accept", the hints and
  Start an agent here (REQ-001, REQ-002).
- `917-02-start-agent`: Start an agent here clicked: the New Agent picker asking only which agent
  (REQ-002).
- `917-03-sticky`: a command printing 300 lines then sleeping 8 s: the pinned row "… is above you
  · running N s" and Jump to it (REQ-003).
- `917-04-jumped`: Jump to it clicked: the block's start at the top (REQ-003).
- `917-05-chips`: the tab row's right end: "bash", the folder and the branch; then `build`
  attached: "tmux · build · 1 window" (REQ-004).
- `917-06-tell`: `sleep 5`, Tell me when done clicked ("Will tell you"), focus moved to another
  terminal; after the end, the notification server's log holds the banner (REQ-005).
- `917-07-day`: Marley Day, the prompt card and the tab row's chips (REQ-001, REQ-004).

## Locked-In Decisions
- D1 — The card wraps Marley's prompt editor; nothing changes in the grid's own prompt.
- D2 — Key hints read the keymap for their actions, so they never name a key that is not bound.
- D3 — "N changed" counts the files the repository reports changed (staged, unstaged and new),
  the number `git status --short` lists.
- D4 — Session chips are drawn by Marley through one pane hook; Zed's +, split and zoom buttons
  stay after them.
- D5 — Tell me when done uses #551's notification and its "not while you look" rule; it lifts
  only the length threshold.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHILE the shell waits at its prompt with the prompt editor, the prompt shall be a card with the folder, branch and changed-files chips, the last exit when the newest block failed, and `❯` before the line, in Marley Night and Marley Day. | `917-01-prompt`, `917-07-day` |
| REQ-002 | The prompt card shall show "→ to accept" while a suggestion shows, a row of key hints from the keymap, and "Start an agent here", which opens New Agent for the terminal's folder. | `917-01-prompt`, `917-02-start-agent` |
| REQ-003 | WHILE a block's start is above the view, the pinned row shall read "<command> is above you ·" with the block's state, and Jump to it shall scroll to the block's start. | `917-03-sticky`, `917-04-jumped` |
| REQ-004 | WHILE a terminal tab is active, the tab row's right end shall show its session (the shell, a tmux session with its windows, or an ssh host with its link), its folder and its branch. | `917-05-chips`, `917-07-day` |
| REQ-005 | WHEN Tell me when done was clicked on a running block, its end shall post a notification whatever the long-command setting, unless the user is looking at that terminal. | `917-06-tell` |
| REQ-006 | Each Zed path changed shall have its row in `docs/marley/zed-touchpoints.md`. | gate:16 |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the prompt card, the sticky header's words, the pane hook and the session chips,
  Tell me when done, the ledger row; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
