# Boards C: Threads, ThreadStates, Inbox, Review, DiffReview, Worktrees, Palette

Code paths are relative to /srv/stacks/marley_ide/crates. Shots are in `review/now/`.

## Threads

Built by #836 (filters, day groups, preview with Open, Open in a Split, Archive). These gaps remain:

- **Page header count.** Board: "48 conversations · 3 open" beside the H1. Marley: the "Threads" headline only (`marley_workbench/src/threads_page.rs:893`, tour-11-threads.png). Kind: missing. Size: S.
- **New Agent button in the header.** Board: an accent "+ New Agent" button at the header's right. Marley: New Agent appears only in the empty state (`threads_page.rs:602`). Kind: missing. Size: S.
- **Search placeholder and icon.** Board: a magnifier icon, "Search titles, folders, and what you asked". Marley: a plain box, "Search conversations by title or folder, and CLI sessions by their prompts…" (`threads_page.rs:316`, tour-11). Kind: look. Size: S.
- **WHO filter rows.** Board: each row has a coloured glyph tile (M, R, CA, CC) and a count at its right. Marley: plain subtle buttons with the name only, no tile and no count (`threads_page.rs:421-457`, tour-11). Kind: look. Size: S.
- **Thread state on cards.** Board: each card's tile has a state dot (needs you, working, done, idle). Marley: `Row` has no state field, and the start slot is a muted agent icon (`threads_page.rs:257`, `:665`). Kind: missing. Size: M (needs the live status per row, as the rail has).
- **Cards as cards.** Board: bordered, rounded cards with a 32 px glyph tile, title and time on one line, a snippet line, then agent and folder pills. Marley: a sparse `ListItem` with an icon, the title, an "Agent · folder" line and the time in the end slot (`threads_page.rs:653-704`, tour-11). Kind: look. Size: M.
- **Snippet line on every card.** Board: each card shows its last message ("You: …", "Done: …"). Marley: a snippet only when the search matched one (`threads_page.rs:673`). The store holds titles, not messages, so #836 left the conversation out of the preview, and that reason covers this line too. Kind: deferred (#836). Size: M.
- **Columns squeeze the list.** Board: the three columns wrap and flex (filters 230, list flex 999, preview 380). Marley: filters fixed at 200 px and the preview at 320 px. In a half-width pane the list shrinks to about 100 px and every title truncates ("Marley a…", "Program…") (tour-11-threads.png; `threads_page.rs:886`, `:923`). Kind: look. Size: S–M.
- **Preview: state line.** Board: under the title, "Needs you · allow kill-session?" in the state colour. Marley: "Last changed …" only (`threads_page.rs:530`). Kind: missing. Size: S (with the state above).
- **Preview: conversation excerpt.** Board: the user's bubble, the agent's answer and the waiting-command banner. Marley: none. Kind: deferred (#836: loading a thread to preview it would start its agent). Size: L.
- **Preview: facts grid.** Board: a 2×2 grid of Folder, Files changed (6 · +412 −198), Started and Mode (Act · bypass). Marley: none, only the agent · folder line (`threads_page.rs:510-545`). Kind: missing. Size: M.
- **Preview: Archive for CLI sessions.** Board: Archive on every preview. Marley: Archive only for a Zed conversation (`threads_page.rs:578`). Kind: deferred (#836 D3). Size: S.
- **Bottom row on the Threads page.** Board: "Threads · 48 conversations · 1 needs you … 3 open". Marley: the focus item knows File, Terminal and Thread, but has no Threads-page case (`marley_workbench/src/focus_row.rs:33-38`, tour-11). The global "N needs you" does exist. Kind: missing. Size: S.

## ThreadStates

Built by #825, #864, #798 and #717. These gaps remain:

- **Width button wording.** Board: "⅓" with the steps ⅓, ½, ⅔, Whole window. Marley: the same four steps, labelled in percent ("33%", "100%") (`marley_workbench/src/agent_panel_width.rs:90-102`). The button is hidden when the pane is alone in its row, so it is not seen in t2-12 or t2-13. Kind: look. Size: S.
- **ASKS label on a waiting tool call.** Board: a framed "ASKS" label, a tool-name chip and the command, then Allow / Always for This Project / Deny. Marley: the EDIT and RUN labels exist (`agent_ui/src/conversation_view/thread_view.rs:8841`, `:11010`), but there is no ASKS label. The buttons are Zed's ("Allow once", "Always for …", "Deny"; `agent/src/thread.rs:6186-6201`). Not seen in a run. Kind: look. Size: S.
- **Agent's HTML page card.** Board: a "www" tile, "report.html · The agent's page, open in a Browser tab of this project" and a Show button. Marley: only `component_show` gets a drawing of its own (`marley_workbench/src/component_tools.rs:31-40`), so an agent's browser page shows as a plain tool call. Kind: missing. Size: S–M.
- **New Thread with Context notice.** Board: a framed notice, "New Thread with Context, in a split to the right", with "1 request, 3 answers, 4 tool calls". Marley: the preamble is text in the new thread's message box ("Continuing … What follows is context …: history, not …", `marley_agent/src/handoff.rs:104`), with no counts. Not seen in a run. Kind: look. Size: S.
- **No-model state.** Board: a centred empty card with a "Z" tile, "Zed Agent needs a model", "Pick a provider and a model, and this thread can start." and Configure Providers. Marley: an Info callout above the conversation, "The Zed Agent has no model yet" / "Configure a language model provider, then pick a model under Select a Model." (`marley_workbench/src/thread_tab.rs:612-634`). Kind: look. Size: S.
- **Thread tab's right-click menu.** Board: Rename, New Thread with Context…, Review Last Reply, Split Right, Archive, Delete Thread…. Marley: Zed's close entries, then New Thread with Context…, then Split Right/Left/Up/Down (`thread_tab.rs:319-336`, `marley_workbench/src/pane_splits.rs:72-96`). Rename, Review Last Reply, Archive and Delete Thread… are only on the rail row's menu (`marley_workbench/src/rail.rs:6531-6550`). Kind: missing. Size: S.

## Inbox

Needs you was built by #840 and toasts by #869. These gaps remain:

- **Toast dot colours.** Board: the dot by kind: yellow for a warning (undo), green for a success (captured), red, the accent for a question, grey for neutral. Marley: red when the text has "Could not" or "failed", the accent for everything else (`marley_workbench/src/toasts.rs:21-35`). Kind: look. Size: S.
- **Notices Marley builds itself.** Board: every notice (plugin too old, trust this folder, agents can act again) drawn as a dot, title, body and actions. Marley: the `MessageNotification`s and app notifications keep their own content. Kind: deferred (#869 Out). Size: M.
- **The plan entry.** Board: "Plan to review: “<plan title>”" with an accent "Open the plan" button. Marley: the ask is "Plan to review" with no title, and it has no answers, so the button is the quiet "Open" (`marley_workbench/src/plan_review.rs:377-379`, `rail.rs:2576-2582`). Kind: look. Size: S.
- **Desktop notification wording.** Board: "cargo build finished · exit 0", "repo — bash ran 4 m 12 s while you were in main.rs." Marley: the title is the command and the body is "done in 45 s" or "exit 1 after 4 m 12 s". It does not say where you were (`marley_workbench/src/command_watch.rs:136-139`). Not seen in a run. Kind: look. Size: S.
- **Phone push layout.** Board: the event as the title ("Codex needs you", "Turn finished") and "project · detail" as the body. Marley: the title is always "Marley" and the body is one line, "marley_ide: Claude needs approval" (`marley_workbench/src/push.rs:257`, `marley_agent/src/marley_agent.rs:871`). The ask itself ("run cargo test?") stays out on purpose: #535 sends nothing the agent wrote. Kind: look. Size: S.

## Review

Built by #881 and #893 (cards, Recommended chip, Plan/Changes tabs, version line, turns). These gaps remain:

- **"Other…" as a button.** Board: Accept recommended, Other… and Skip at the card's foot. Marley: Accept recommended and Skip, plus an "Other…" text field (`plan_review.rs:1858-1868`, `:2007`). Kind: deferred (#881 Out). Size: S.
- **Send Answers beside Send Feedback.** Board: Send Answers and Send Feedback (1) side by side under the plan, with the hint "Select a passage and press c to comment". Marley: while answers are pending, Send Answers takes Approve's place and Send Feedback is gone. Send Feedback carries no count, and the buttons and hint sit in the right column (`plan_review.rs:1877-1910`, `:1598`). Kind: look. Size: S.
- **Notes column heading and Remove.** Board: a "Your notes" heading; each note has its quote in a mono chip and a "Remove" text button. Marley: no heading, the quote in italic, and an X icon (`plan_review.rs:1481-1530`). The Overall placeholder is "A note on the whole plan", not "Anything about the whole plan". Kind: look. Size: S.
- **Turn cards.** Board: the title, a coloured state pill (new, kept, injected) and per-file +/− ("rail.rs +12 −4 · rail_order.rs +3 −1"). Marley: "Turn N · title" and "N files · kept|undone|failed|to review" as muted text. It has no per-file counts and no "injected" state for a turn that came from a task notification (`plan_review.rs:1726-1745`). Kind: look. Size: S–M.
- **Send Review to Agent in the column.** Board: "Send Review to Agent (2)" at the foot of the notes column. Marley: none in plan review; notes go back through Send Feedback (`grep "Send Review" plan_review.rs`: none). Kind: missing. Size: S.
- **Title placement.** Board: the plan's title as an H1 in the main column, under the tabs. Marley: the title is a Large label at the top of the right column (`plan_review.rs:1824`). Not seen in a run. Kind: look. Size: S.

## DiffReview

Built by #730, #731, #888 and #893. These gaps remain:

- **"Changes since main".** Board: a header over the branch diff against main. Marley: "Uncommitted changes" over Zed's project diff (`marley_workbench/src/diff_header.rs:118`). Kind: deferred (#893 Out). Size: M.
- **Per-file +/− in the file header.** Board: each file header has "+31 −8" beside the path and the Viewed tick. Marley: the Viewed tick is added (`marley_workbench/src/viewed_marks.rs:213`), but there is no line count. grep found no Marley hook in the editor's file header. Not seen in a run. Kind: missing. Size: S.
- **Guide beside the diff.** Board: the guide is an aside to the right of the diff, in the same view. Marley: the guide opens as a tab of its own (`marley_workbench/src/guide_tab.rs:1-8`, `marley_workbench/src/review_tools.rs:331`). Kind: look (layout). Size: M.
- **Send to rows.** Board: agent rows with a green "ready" or yellow "working", then a "Copy notes · any agent" row. Marley: agent rows with a muted state and no Copy notes row; Copy Notes lives in the diff header (`guide_tab.rs:346-395`). Kind: look. Size: S.
- **Gutter mark for noted lines and the note's label.** Board: a ● in the gutter on noted lines, and the label "Your note · lines 212–214". Marley: Zed's review-comment drawing plus #522 and #730 hunks (`editor/src/git.rs:2921-3046`). Not checked in a run, so unconfirmed. Kind: look. Size: S.

## Worktrees

Rail rows (drift chips, ahead count) and the menu exist, and the New Agent in Worktree card was built by #851. These gaps remain:

- **Port slot fact.** Board: "Port slot · 2 · PORT 3020, 3021…3029" among the prompt's facts. Marley: Copied from .worktreeinclude, Permissions and Setup only (`marley_workbench/src/worktree_agents.rs:537-590`). Kind: deferred (#851 Out: the slot is assigned when the worktree is made). Size: S.
- **Merge prompt's checklist.** Board: the Merge dialog lists four ticked checks: the main checkout is on main, nothing uncommitted in either checkout, no file would stop the merge. Marley: the checks run first and a failure refuses with a message, but the prompt itself is a question and one detail line, with no list (`rail.rs:4137-4147`, `marley_workbench/src/worktree_git.rs:238-290`). Kind: look. Size: S–M.
- **Remove prompt wording.** Board: "It has 3 changes not committed. They go with the folder. The branch stays: main does not have its commits." The board states whether this branch stays. Marley: "Remove <name> and its 3 uncommitted changes?" with a generic "Its branch goes too once its commits are in its base; otherwise it stays." (`rail.rs:3883-3897`). Kind: look. Size: S.

## Palette

Built by #839 (groups on an empty query, agent cards, file finder preview). These gaps remain:

- **Groups while typing.** Board: the query "open" typed, and the results still under Recent, Marley, and Editor and workspace. Marley: headings only for an empty query, Zed's fuzzy order when typing (tour-02-palette.png; `command_palette/src/command_palette.rs:818-829`). Kind: deferred (#839 D1). Size: S–M.
- **Command glyph tiles.** Board: each command row has a two-letter tile (www, Th, Mc, Cl, St, Km). Marley: text only (`command_palette.rs:831-850`, tour-02). Kind: missing. Size: S–M.
- **Command names.** Board: "Open Browser", "Open Threads", "Open Settings". Marley: Zed's humanized action names, "marley: open browser", "zed: open settings" (tour-02; `command_palette.rs:131`). Kind: look. Size: M (a name map, or a change to the Zed hunk).
- **Scope chip in the input.** Board: a "Commands" chip at the field's right. Marley: none (tour-02). Kind: missing. Size: S.
- **Palette footer.** Board: "Enter to run · Ctrl Enter to bind a key · ? for help". Marley: Zed's footer, "Add Keybinding… Ctrl-Enter  Run Enter", with no "? for help" (tour-02). Kind: look. Size: S.
- **Agent grid.** Board: agent cards in a grid. Marley: a one-column list of two-line cards. Kind: deferred (#839 Out, D3). Size: M.
- **File finder rows.** Board: an extension tile (rs, md, sh), the name over the path on two lines, and a git badge ("M") at the right. Marley: Zed's row, a file icon with the name and path on one line and no status badge (tour-04-file-finder.png). Kind: look. Size: M.
- **File finder hint and preview header.** Board: a "Ctrl P" hint in the field, and the preview headed by "path · 3 412 lines". Marley: the field has Zed's two icon buttons, and the preview has no header line (tour-04). Kind: missing. Size: S.
