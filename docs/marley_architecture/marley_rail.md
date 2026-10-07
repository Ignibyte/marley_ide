# `marley_rail`

The Marley rail's row model, written in the fork for the workbench shell's W2 (#438) and grown
with agent threads in W3 (#439), agent CLIs in W4 (#440), the keyboard's row in W6d (#453), the
filter in W6h (#457), the switcher's order in W6e (#454), Browser tabs in B7a (#504), ports
(#521) and the inbox (#508). Pure and
gpui-free, MIT OR Apache-2.0; its one dependency is the equally pure `marley_agent`.

## What it decides

- **Input:** a `RailSnapshot`, the window as the rail sees it. `projects` holds the project
  groups in the window's order. Each is a `ProjectSnapshot`: its display name, whether it is
  expanded, its center terminals as `TerminalSnapshot`s (id, title, subtitle, bell, and the
  agent CLI in the foreground with its status, if any), its Browser tabs as `BrowserSnapshot`s
  (#504: the tab's view id, its title, the page's host and port, whether the main frame loads, the
  tray's picks, the page's annotations, the agent's mark and the icon's id), its
  agent threads as `ThreadSnapshot`s (key, title, status, attention), and its ports as
  `PortSnapshot`s (#521: the port and the pid that listens, which together name the row, the
  title, the URL and the tooltip, and since #603 the systemd service it runs in as a
  `PortService { unit, user }`, which `rail_rows` copies to `PortRow`), in the order the collector gives. `focus` holds the displayed workspace's group index, its active terminal's id and
  whether that terminal holds the window's focus, its active item's id when that is a Browser tab
  (`browser`), the thread its Agent Panel shows while the
  panel holds focus, and `cursor`, the row the keyboard is on while the rail holds focus. `filtering` says the filter holds text, and each project,
  terminal, Browser tab, thread and port carries `matched`: the byte offsets where the filter matched its name or
  title, or `None`. The gpui side computes them with Zed's matcher.
- **One walk** decides which rows show, and `rail_rows`, `selection` and the keyboard's
  functions all read it. Without a filter, a project shows its header, then its terminals, its
  Browser tabs, its threads and its ports when it is expanded. With one, a project shows when its name or a row under it
  matched. A name match shows every row under it and fold is ignored; otherwise only the rows
  that matched show.
- **`rail_rows`** gives the rows the walk shows, in its order. It marks the one selected row,
  the characters to highlight on each row while filtering, and a header's attention flag. The
  flag is set when a row the rail is not showing under the header needs the user: a terminal's
  bell, or a thread's dot or wait. A Browser row's counts and mark set no flag, and a port row has none.
- **`selection`** picks that row: the keyboard's row when it is shown, else the focused Agent
  Panel's thread, else the displayed workspace's active terminal, else its active Browser tab,
  else that workspace's project header, each only when the walk shows it. Otherwise nothing is selected: the window shows no
  project the rail lists, or the filter hides it. A stale focus never selects a row that is not
  there.
- **`step`, `first_row`, `last_row` and `parent`** move the keyboard's row (#453). `step` goes
  to the next or previous shown row and stays on the last or the first; with nothing selected
  it starts at the first going forward and the last going back. `parent` is a terminal's, a
  Browser tab's, a thread's or a port's project header, and a header is its own. `first_match` is the first shown row whose
  own name or title matched, where the keyboard's row goes as the filter changes (#457).
- **`cycle_project` and `cycle_row`** are Zed's Next and Previous Project and Thread (#459).
  Both go round the shown rows from the selected one, wrapping at the ends, and with nothing
  selected start at the first going forward and the last going back. `cycle_project` starts
  from the selected row's project header and reaches only headers. `cycle_row` reaches only
  terminals, Browser tabs and threads, since a port is no place to switch to (#521), so from a header it goes to the first row under it, or
  back to the last row above it. A lone row reaches itself; with none of the kind shown, nothing is
  reached.
- **`window_row`** is the terminal or thread row that holds the window's focus: the focused
  Agent Panel's thread, else the active terminal while it holds focus (#454). The rail notes
  each change of it for the switcher's order.
- **`switcher_rows`** gives every terminal and thread as a `SwitcherRow`, never a header, a
  Browser tab or a port, for the switcher: first the rows ranked by the `shown_at` the caller passes, most recent first,
  then the rest in the rail's order, whatever the fold and the filter.
- **`has_attention`** is the rail's notification flag: any listed terminal's bell, or thread
  dot or wait, folded or not.
- **`thread_status`** ranks what a live conversation reports: a pending confirmation over an
  error over a running agent, else done. **`ThreadStatus::label`** (#468) is the word a thread
  row's second line gives each: `idle`, `working`, `waiting`, `failed`.
- **`thread_attention`** says whether a thread's dot is lit after a rebuild. A run that just
  ended (running before, done or failed now) lights it unless the thread is shown, and it
  stays lit until the thread is shown.
- **`TerminalSnapshot::activity`** (#519) is a third line under an agent's status, from its
  own events: the tool in flight, what it waits on, its last message or its error. `rail_rows`
  and `switcher_rows` copy it to `TerminalRow::activity`; it decides no order or visibility.
- **`TerminalSnapshot::command`** (#551) is a plain terminal's last block as a
  `CommandSnapshot { text, running, exit_code, duration, password }`, for the row's command line;
  the builders copy it to `TerminalRow::command`. The workbench's snapshot, not this crate,
  matches the filter against its text when the title does not match. It is `None` for an
  agent's terminal and an agent's block.
- **`TerminalSnapshot::running_error`** (#572) is the failure a running command printed and kept
  running after, as `RunningError { line, questioned }`; the builders copy it to
  `TerminalRow::running_error`. The switcher's `SwitcherRow::Terminal` holds its row boxed, since
  a terminal's row is by far the larger variant.
- **The attention order** (#542), a setting since #671 made `RailOrder::Window` the default.
  `Attention { NeedsYou, DoneUnseen, Working, NotReporting,
  Idle }` classes a row: `terminal_attention` reads the agent's status with
  `TerminalSnapshot::reporting` (`Reporting::Timer` for the quiet timer, `Events` for #519's
  events, `Stale` once they stopped past `no update in N m`), so only an agent's own events can
  say it waits, and a bell or unread mark over an agent that is not working is done-unseen;
  `thread_attention_class` does the same for a thread, and `project_attention` takes a project's
  most demanding row. `RailSnapshot::order` (`RailOrder::Attention` or `Window`) and
  `RailSnapshot::held` pick the walk's order: a stable sort by class, ties in window order, or,
  while `held` is set, each item's place in the `Held` lists of project indexes, view ids and
  thread keys that `held_order` recorded from the walk (an item it does not name goes last).
  `ProjectRow::summary` counts a collapsed project's agents by state, most demanding first.
- **Closed projects** (#606). `ProjectSnapshot::closed` marks a group the window holds no workspace
  of: the workbench gives it no rows, and `cycle_project` passes over it, since going to it would
  open it.
- **Dragging** (#602). `run(snapshot, &Row)` gives the `Run` a dragged row may drop among:
  `Headers` (with the class under the attention order), a project's `Terminals` in one section
  (main checkout or one worktree, with the class), its `Browsers`, or its `Threads` (with the
  class); a worktree's or a port's row gets `None`. `place` sorts items by their place in a saved
  list (stable, the unplaced last), and `move_to` moves one item before or after another. The
  workbench orders the snapshot's projects and rows with them before the walk, so the attention
  sort breaks its ties by the order the user set.
- **`TerminalSnapshot::flag`** (#569) is the tooltip of a working agent's warning mark, when the
  stall kind flagged it `looping?` or `stalled?`; the builders copy it to `TerminalRow::flag`, and
  it too decides no order or visibility.
- **Worktree rows** (#510). `ProjectSnapshot::worktrees` lists a project's linked worktrees as
  `WorktreeSnapshot { path, name, branch, open, matched }`, and `TerminalSnapshot::worktree`
  tags a terminal with its worktree's folder. The walk puts a project's header, the main
  checkout's terminals, then each worktree's row with its terminals under it, then the Browser
  tabs, the threads and the ports; a shown terminal keeps its worktree's row above it, and under a
  filter a worktree shows for its name or branch, with its terminals. `Selection::Worktree` and
  `Row::Worktree` carry the folder; `parent` climbs from a worktree's terminal to its row and from
  the row to the project, and `cycle_project` climbs through it; `Focus::worktree` puts the
  displayed worktree's row before the project header in the selection. The terminals stay in the
  project's flat list, so every other reader of it is unchanged.
- **`WorktreeSnapshot::drift`** (#560), copied onto `WorktreeRow`, is `DriftSnapshot { ahead,
  behind, conflicts, base, base_commit }`: since #511 the commits on the branch its base lacks,
  the commits on the base the branch lacks, the files a merge would stop on (`None` when git
  cannot say), the base and its short tip. `ahead_words` is the row's count (`2 ahead of main`,
  none at 0). `words` is the chip's text (`2 behind`, `1 conflict`, `3 conflicts`; none up to
  date, and since #511 none while `ahead` is 0), `conflicted` its color, and
  `tooltip` its lines: `N commits behind main (main at <sha7>)`, or the short commit alone for a
  commit base, then `A merge would stop on:` and twenty files at most with `and N more`, or
  `It merges cleanly.`, or `This git cannot tell whether it merges cleanly.`
- **`TerminalAgent::mark`** (#532) is the agent's permission mark, `marley_agent`'s
  `PermissionMark`, when it runs without its permission prompts; the builders copy it with the
  agent, and it decides no order or visibility.
- **`TerminalSnapshot::turns`** (#509) are the turns of the terminal's Claude Code that changed
  the tree, newest first, as `TurnSnapshot { title, files, failed, injected, sha }`, and
  `turns_open` whether the rail lists them under the row. The builders copy both to
  `TerminalRow`; the turns are drawn inside the terminal's row, not as rows of their own, so they
  move no selection and decide no order.
- **The inbox (#508).** `RailSnapshot.inbox` holds what waits on the user as `InboxEntry`s: a
  key naming what waits and on what, the kind (`InboxKind::Thread` for an Agent Panel tool call
  waiting for confirmation, `Terminal` for an agent CLI waiting on a permission or a question,
  `Click` for an agent's click a Browser tab holds, `Harness` for a harness session waiting on a
  question, #534, `Codex` for a request a Codex App Server asks to approve, #651), the agent, the
  project, what it asks on one line, how long it has waited in words, and the answers it takes in
  place, as a list of `InboxAnswer`s (Allow, Allow session, Deny, Refuse, Stop turn, Dismiss, each
  with its words, its tooltip and whether it allows, #651). `waited_words` gives the
  words: `now` under a minute, then `3 m`, then `1 h 5 m`. The inbox decides no row's order or
  visibility, and the fold and the filter leave it whole. Since #568 an entry also carries its
  `chips` (`marley_agent::risk::Chip`: what the action would do, and whether Marley's rules or a
  model put it there) and its `level`, 1 to 5, which orders the inbox while the risk use is on;
  `inbox_suggests` says a model's chips show as suggestions, with a question mark, rather than
  dashed, so a change of the use's mode changes the snapshot and redraws. Since #570 an entry
  carries its `route` (`marley_agent::route::RouteMark`: who should answer it, and whether a rule
  or a reading said so), and `route_suggests` says a reading's route shows with a question
  mark.
- **`working_directory_label`** is a terminal row's second line: the directory relative to the
  project root, or with the home directory written as `~` outside it. It is empty at the root
  itself and when the terminal cannot tell (no path, or an empty one).

## Why a crate of its own

Every decision the rail makes is here, unit-tested in milliseconds. The gpui side's tests
build a binary that carries Zed's agent crates, so the less logic lives there, the cheaper
its tests.

## Consumers

`marley_workbench`'s `Rail` builds the snapshot from the live window, stores it, and draws the
rows this crate returns. It decides no ordering or visibility of its own. It hands in two
things it keeps: the keyboard's row, as `Focus::cursor` while it holds focus, and what the
filter matched. The one order it decides is the inbox's: it keeps when it first saw each entry
and lists the entries by that, the longest waiting first (#508).

## Tests

`src/marley_rail.rs`, thirty-three unit tests, including exactly one selected row over every
combination of fold, displayed project, active terminal and focused thread in a two-project
window, and `step` over every shown row of one in both directions.
