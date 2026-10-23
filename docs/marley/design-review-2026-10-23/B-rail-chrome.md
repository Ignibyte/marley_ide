# Board review B: rail and chrome

Boards: Rail, RailStates, RailMenus, ProjectLogo, TmuxSessions, Chrome, BottomRow, Splits.
Code paths are under `/srv/stacks/marley_ide/crates/`. Shots are in `.../review/now/`.

Notes on the shots:
- `t2-10-plus-menu.png` shows no open + menu, only the Knowledge tab.
- `t2-15-split.png` shows the split menu open (the pictures), not a split layout.
- The + menu, a split, the strip, the flyout, the Sessions view and page, the tmux close prompt and the debug row were not seen in any run. I judged those from the code.

One deliberate difference, not a gap: the boards draw the rail's view switch as the words Projects | Threads (| Sessions). Marley draws six icons in the title-bar row: Projects, Threads, Sessions, Files, Add Project, Shrink. Chad asked for that in #873 and #902 (`tour-01-start.png`).

## Rail

- **Tile glyphs.** Board: letter glyphs per kind (`CC`, `CX`, `>_`, `www`, `tx`). Now: the existing icons on the kind's tint (`tour-01-start.png`, `t2-13-thread.png`). Kind: deferred (#827 Out: "the icons stay"). Size S.
- **Browser tab states.** Board: Loading, A dialog is open, Loaded, each with a dot. Now: a Browser row has no state. Kind: deferred (#827 Out). Size S–M.
- **tmux row states.** Board: "a command runs", "bell", "command ended" and "1 window", with the four dots. Now: tmux rows are drawn with no state. #827's scope gave them none on purpose (`rail.rs` `tmux_row`). Kind: deferred (#827 scope). Size M.
- **Thread row second line.** Board: the line says what the agent does or wants: "Editing thread_view.rs", "Allow kill-session?", "3 files changed", "Opus 5.5 · idle". Now: always `<agent> · <status word>`, from `thread_subtitle` (`rail.rs:11286`). `t2-13-thread.png` shows "Marley · idle" in the done color with a green dot, so the word and the color disagree. Kind: look. Size M.
- **Shell row second line.** Board: a running build shows its progress ("1781 of 1813"); a failure shows "exit 101 · Ask an agent" with a link; idle shows the folder. Now: `<command> · running`, or `· exit N · 0 s` in red with no Ask an agent (`command_line`, `rail.rs:9694`). An idle shell's second line is blank (`tour-01-start.png`). Kind: look. Size S.
- **Folded group header.** Board: right-aligned pills, a pink "1 needs you" and an amber count. Now: a plain XSmall text "1 waiting, 2 working" after the name, in the conflict color when attention is set (`project_name`, `rail.rs:10764`; `marley_rail.rs:1344`). The wording differs too: "waiting", not "needs you". Kind: look. Size S.
- **Open group header.** Board: shows a "5 rows" count. Now: no summary while the group is open (`marley_rail.rs:1809`, `(!project.expanded).then(...)`). Kind: missing. Size S.
- **Default group header.** Board: a home tile and "quiet". Now: a list icon, "Default" and `+`, with no tile and no quiet note (`tour-01-start.png`). Kind: look. Size S.

## RailStates

- **Summary on an open header.** Board: each open group header carries its summary at the right ("1 needs you" in pink, "1 done"). Now: summary only when folded (see Rail). Kind: missing. Size S.
- **Filter field.** Board: a bordered, rounded field reading "Filter rows and projects" with a "Ctrl F" hint. Now: a flat row reading "Filter…" with "Ctrl-F" (`rail.rs:1277`, `tour-01-start.png`). Kind: look. Size S.
- **Strip foot.** Board: a `+` (add project) at the strip's foot. Now: not found. `render_strip` draws Open, Search, tiles and the Sessions tile only (`rail_strip.rs:259-320`). #830 deferred a gear there, not the `+`. Kind: missing. Size S.
- **Flyout.** Board: a compact flyout with the project's name, a "1 needs you" chip, and one line per row: dot, title, short state ("needs you", "12 min", "tmux"). Now: the open rail's full block, the header plus full row cards, 280 px wide (`render_strip_flyout`, `rail_strip.rs:460`). Not seen in a run. Kind: look. Size S.
- **Strip search popover.** Board: a match count in the field ("3 rows"), a flat result list with a tile, a title and a where-line ("marley_ide · tmux · 1 window"), and a footer "Enter opens · ↑ ↓ moves · Esc closes". Now: the rail's filter field and the matching rows grouped under project headers. No count, no where-line, no key-hint footer (`render_strip_search`, `rail_strip.rs:499`). Not seen in a run. Kind: look. Size S.
- **Threads in the strip.** Board: the strip shows only projects; nothing to add. Already deferred in #830 (no view switch in the strip). No gap.

## RailMenus

- **Threads view rows.** Board: an agent-colored glyph tile (`CC` amber, `M` orange, `R` teal, `CX` violet) and a line "Claude Code · marley_ide · 14:14", with "resumable" for a CLI session. Now: a plain muted icon in the circle slot with no kind tint, and a line `<agent> · <folder>` with no time (`render_thread_list`, `rail.rs:5494`; `threads_page.rs:268`). Kind: look. Size S.
- **New group.** Board: an inline "New group" field at the rail's foot, with an accent border and an "Enter" hint. Now: a modal prompt, "A name for the group" (`groups.rs:697-733`). Kind: look. Size S.
- **Empty-space menu wording.** Board: "New Terminal in Default", "New Browser Tab in Default". Now: "New Terminal" and "New Browser Tab", plus an extra "Reopen Thread…" (`rail.rs:4726-4785`). Kind: look. Size S.
- **Project header menu, missing entries.** Board: "Link to a Brain Page…". Now: not found (grepped "Brain Page", "link.*brain" in `marley_workbench/src`). Kind: missing. Size M (needs a brain-page picker and somewhere to keep the link).
- **Project header menu, Fold.** Board: a "Fold" entry. Now: not found (grepped `"Fold"`). Folding is only the chevron. Kind: missing. Size S.
- **Project header menu, Close Project.** Board: a red "Close Project". Now: "Remove Project" in the default color, at the end of a longer menu: Move Up/Down, logo choices, Archived Threads, Clear Browser Data… (`project_context_menu`, `rail.rs:5973-6110`). Kind: look. Size S.
- **Project header menu shortcuts.** Board: "Ctrl ~" on New Terminal and "Ctrl Alt N" on New Agent…. Now: the keys show, from #866's action-named entries. No gap.
- **Terminal row menu.** Board: Rename, Open Agent Here…, Review Last Reply, Turns (3), Send Block to Agent, Split Right, then a red Close. Now: Rename, Open Agent Here…, Review Last Reply, Move to Project ▸, Close (`rail.rs:6780-6835`).
  - Missing: "Turns (N)" as an entry (it exists only as the card's disclosure, `rail.rs:7062`), "Send Block to Agent" and "Split Right". Close is not red.
  - Kind: missing. Size S.
- **"Where a new agent works" picker.** Built (`agents.rs:914-951`: this project, the terminal's folder, open projects, Browse…, harness seats). No gap.

## ProjectLogo

- **"Has a logo" toast.** Board: a toast on first open, "marley_ide has a logo / Found logo.png in its top folder", with a "Use a letter" button. Now: no toast. Kind: deferred (#831 Out). Size S.
- **Settings window row.** Board: a "Project logo" row in Settings with the preview, "Choose a file…" and "Use a letter", and the key `marley.project_logo: "auto"`. Now: no Settings-window row. The pick lives in the rail header's right-click menu (Choose Logo…, Use a Letter, Use the Project's Logo; `rail.rs:6016-6035`) and in the map `marley.project_logos` (`settings_content/src/marley.rs:206`). Kind: deferred (#831 Out). Size S–M.
- **Mark on a tab.** Board: a project's tab shows the project's mark ("marley_ide — bash" with the cat). Now: tabs show the kind tile only (`tab_tiles.rs:27`, `glyph_of`). Not found: grepped `project_icons` in `workspace/src` and `tab_tiles.rs`. Kind: missing. Size S.
- **Title bar for a letter-only project.** Board: the mark beside the name in the title bar. Now: the mark shows only when the project has an image. A project with only a letter shows its name with no mark (`tour-01-start.png`: "repo", while the rail shows its "R" tile), following #831's scope. Kind: look. Size S.

## TmuxSessions

- **Filter in the Sessions view.** Board: a "Filter sessions" field. Now: none. Kind: deferred (#832 Out). Size S.
- **Detached rows.** Board: detached sessions fade (opacity 0.55). Now: every session row is drawn the same (`rail_sessions.rs:96-160`, `row_card` with no fade). Not seen in a run. Kind: look. Size S.
- **Host not looked on.** Board: the host sits in the list with a grey dot and "Not looked on yet. Add to tmux hosts" (a link). Now: such hosts appear only as checkboxes under "HOSTS TO LOOK ON" below the list (`rail_sessions.rs:269`). Kind: look. Size S.
- **Sessions page column headers.** Board: Session, Host, Windows, Folder · placed under, State. Now: none. `render_row` draws bare columns (`sessions_page.rs:158-252`, no header labels). Kind: missing. Size S.
- **Sessions page state.** Board: a colored chip, "attached here" (green), "attached elsewhere" (amber) or "detached". Now: two words only, "attached" or "detached", as plain text in accent or muted (`sessions_page.rs:162`). The rail's view does tell here from elsewhere. Kind: look. Size S.
- **Sessions page primary button.** Board: "Show" for a session attached here, "Attach" otherwise. Now: always "Attach" (`sessions_page.rs:230`). Kind: look. Size S.
- **Sessions page placement.** Board: "under <project>" under the folder. Now: the folder only. Kind: deferred (#861 Out). Size S.
- **Sessions page header summary and New Session.** Board: "… · 2 attached here · 4 detached" and "New Session…". Now: "… · N attached · N detached" and "New Session", which starts a session in the window's first folder at once, with no dialog (`sessions_page.rs:113-150`). Kind: look. Size S.
- **Sessions page host toggles.** Board: hosts with checkboxes to look on them. Now: the page's host list is read-only labels, "looked on" or "not looked on" (`sessions_page.rs:271-300`). The ticks are in the rail view only. Kind: look. Size S.
- **Bottom row tmux count.** Board: "tmux: 6 · 4 detached", and a click opens Sessions. Now: a muted "tmux 6" with no detached count and no click (`focus_row.rs:192`). Kind: look/missing. Size S.

## Chrome

- **Rusty and Marley buttons.** Board: R and M letter tiles after the pills, then Help and Sign In. Now: a person icon and the cat icon, placed after Help (`status_buttons.rs:520-545`, `t2-10-plus-menu.png`). The working and needs-you pills exist (#834) but were not seen in a run (no agents at work). Kind: look. Size S.
- **Extra title-bar items.** Board: the mark, the project and the branch, nothing more. Now: Zed's hamburger app menu and a worktree label before the branch ("main / main"), plus a user chevron after Sign In (`tour-01-start.png`). Kind: look. Size S.
- **Tab shape.** Board: rounded pills and a "+3 more" overflow button. Now: Zed's tab frames, and the bar scrolls (`tour-05-editor.png`). Kind: deferred (#866 Out). Size M.
- **Tab tile colors.** Board: each kind tinted (`CC` amber, `rs` orange, `tx` teal, `www` orange). Now: one neutral tile with a muted glyph for every kind (`tab_tiles.rs:105-121`; every tab in `t2-15-split.png`). Kind: look. Size S.
- **Tab glyphs.** Board: an agent thread tab shows its agent (`CC`, `M`); a tmux tab shows `tx`. Now: a thread tab shows the title's first letter ("W" for "What changed…", `t2-15-split.png`), and a tmux tab shows `>_` (`glyph_of`, `tab_tiles.rs:85`). Kind: look. Size S.
- **Tab state dots.** Board: a dot on every kind: thread working or needs you, Browser loaded, tmux idle. Now: only terminal tabs get a dot (`state_dot`, `tab_tiles.rs:38`). Thread and Browser tabs have none (`t2-15-split.png`, `t2-13-thread.png`). Kind: missing. Size S–M.
- **Doubled terminal icon.** Board: one tile per tab. Now: terminal tabs show the `>_` tile and Zed's terminal icon after it ("`>_` [icon] repo — bash", `tour-01-start.png`, `tour-05-editor.png`). Kind: look. Size S.
- **Nav arrows in the tab bar.** Board: none. Now: Zed's back and forward arrows at the left of the bar (`tour-01-start.png`). Kind: look. Size S.
- **+ menu tiles.** Board: a colored glyph tile per row. Now: Zed menu rows, text with keys. Kind: deferred (#866 Out). Size S.
- **+ menu items.** Board: "New Claude Code" as a direct item and "Open Folder… Ctrl K Ctrl O". Now: agents start through "New Agent…" or the "New Agent in tmux" ▸ submenu (#744's one way). "Open Folder…" is not in the project's + menu (`new_entries`, `rail.rs:6150-6265`). Kind: missing (Open Folder…). Size S.
- **Status bar words.** Board: words beside the icons ("Debug", "0 errors · 0 warnings"), and the language-server message as a dotted chip. Now: an icon-only Debug, an icon-only search, and Zed's diagnostics check icon. The server message is Zed's raw "⚠ (rust-analyzer) Error: Failed to discover workspac…" (`tour-05-editor.png`). Kind: deferred (#833 Out: Zed's left items stay). Size M.
- **tmux close prompt.** Board: a `tx` tile, the title "Close the tmux session "build"?", a detail line "On this machine · 1 window · cargo build is running", and buttons Cancel · Hide · Close Session. Now: a `window.prompt` with the title and a fixed explanation. No tile, no host, window or running-command line, and the buttons run in the order Close Session, Hide, Cancel (`close_guard.rs:184-200`). Not seen in a run. Kind: look. Size S.
- **Split menu.** Built (`t2-15-split.png`). The pictures are smaller (22×16) and the old pane is drawn in border grey; close enough. No gap.

## BottomRow

- **File focus.**
  - Board: blame ("t, just now"), the server message as a chip with a dot and "· Details", "0 problems", "UTF-8" and "LF", and a "Predictions off" chip.
  - Now: the `rs` chip, the path, then Zed's own items: search icon, server icon, check icon, the raw server message, "1:1", "Rust", and the prediction icon (`tour-05-editor.png`). Blame shows inline in the editor, not in the row.
  - Not found in the row: blame, UTF-8 and LF, "0 problems" words, the Predictions chip.
  - Kind: deferred in part (#833 Out: Zed's left items stay); the blame and the encoding are missing. Size M.
- **Terminal focus.**
  - Board: the folder ("~/…/repo"), "main · 1 changed", the failed block as a button "false · exit 1 · Ask an agent", "4 blocks · 1 bookmarked", and the size "132×38".
  - Now: no folder when the shell sits in the project root (`tour-01-start.png`). No branch or changes, no Ask an agent on the failed block (red text only), no terminal size (`focus_row.rs:105-196`).
  - Kind: missing. Size S–M.
- **Thread focus.**
  - Board: the agent tile and name, model and effort ("Opus 5.5 · High"), mode ("Act · bypass"), what it does with elapsed time ("Working · editing thread_view.rs · 12 min"), a context meter ("48k of 200k"), "3 files · +35 −2", and a "Full width" toggle.
  - Now: the agent's name, a status word and the thread title: "Marley idle What changed in the tmux rows?" (`t2-13-thread.png`; `focus_row.rs:198-240`).
  - Kind: missing. Size M.
- **Debugging row.** Board: shows the watched value (`name = "thread"`) and a yellow frame on the bar while paused. Now: neither. Kind: deferred (#901 Out). Size S. The rest of the debugging row is built (`debug_row.rs`); not seen in a run.
- **Home in the panel group.** Board: "Home" is the group's first labeled button. Now: an unlabeled Home icon before the group (`status_buttons.rs:240`, every shot). Kind: look. Size S.
- **Panel right-click menu.** Board: none drawn; nothing to add. Already deferred (#862 Out). No gap.
- **Language servers card.** Board: a card listing each server with a status word (failed, running), the failure's message inline, and Restart · Open log · Ask an agent buttons at the foot. Now: Zed's server menu, with Ask an Agent in a server's submenu (`language_tools/src/lsp_button.rs:488`). Not seen in a run. Kind: look. Size M.
- **Problems card.** Board: an Errors / Warnings / Hints count, and "Open in the drawer". Now: no Hints. Kind: deferred (#863 Out). Size S.
- **Panels card.** Board: three columns per row (name, where, keys), with Home in the list. Now: one "Files · beside the rail" label with the key, and no Home (`panel_group.rs:47-88`; `t2-11-status-sparkle.png`). Kind: look. Size S.
- **Edit predictions card.** Board: radio buttons for the provider (Off, Zeta · needs sign in, Copilot) and a "Never in .env files" checkbox. Now: Zed's own edit-prediction menu, unchanged. No Marley hook in `edit_prediction_ui` (grepped "Marley"). Not seen in a run. Kind: look. Size M.

## Splits

- **Pane headers.** Board: each pane's header is one tab pill with its glyph and a "+1" / "+2" count of the pane's other tabs. Now: each pane keeps Zed's full tab bar. Not seen in a run. Kind: deferred (#866 Out: the pill shape and "+N more"). Size M.
- **Ctrl Tab switcher tiles.** Board: tiles tinted by kind, with a state dot on the tile (needs-you pink, running amber, idle grey). Now: a neutral tile with no dot (`tab_tiles.rs:124-137`, `tile`). Kind: look. Size S.
- **Ctrl Tab switcher line.** Board: the line says the project and the state ("marley_ide · needs you", "repo · cargo run", "marley_ide · tmux"). Now: a file's path in its project, else the tab's tooltip (`switcher_row`, `tab_tiles.rs:124`). Kind: look. Size S.
- **Port rows.** Board: names the terminal that serves the port ("cargo run in repo — bash"). Now: the process's command line. Kind: deferred (#890 Out). Size M.
- **Built, no gaps.** The focused pane's accent outline (#853), the switcher footer, the port rows with Open and Copy, the Containers rows with Stop, and Collab's signed-out card (`collab_panel.rs:2733-2772`). None seen in a run.
