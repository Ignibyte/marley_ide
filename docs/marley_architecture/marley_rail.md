# `marley_rail`

The Marley rail's row model, written in the fork for the workbench shell's W2 (#438) and grown
with agent threads in W3 (#439), agent CLIs in W4 (#440), the keyboard's row in W6d (#453), the
filter in W6h (#457) and the switcher's order in W6e (#454). Pure and gpui-free, MIT OR
Apache-2.0; its one dependency is the equally pure `marley_agent`.

## What it decides

- **Input:** a `RailSnapshot`, the window as the rail sees it. `projects` holds the project
  groups in the window's order. Each is a `ProjectSnapshot`: its display name, whether it is
  expanded, its center terminals as `TerminalSnapshot`s (id, title, subtitle, bell, and the
  agent CLI in the foreground with its status, if any), and its
  agent threads as `ThreadSnapshot`s (key, title, status, attention), in the order the collector
  gives. `focus` holds the displayed workspace's group index, its active terminal's id and
  whether that terminal holds the window's focus, the thread its Agent Panel shows while the
  panel holds focus, and `cursor`, the row the keyboard is on while the rail holds focus. `filtering` says the filter holds text, and each project,
  terminal and thread carries `matched`: the byte offsets where the filter matched its name or
  title, or `None`. The gpui side computes them with Zed's matcher.
- **One walk** decides which rows show, and `rail_rows`, `selection` and the keyboard's
  functions all read it. Without a filter, a project shows its header, then its terminals and
  its threads when it is expanded. With one, a project shows when its name or a row under it
  matched. A name match shows every row under it and fold is ignored; otherwise only the rows
  that matched show.
- **`rail_rows`** gives the rows the walk shows, in its order. It marks the one selected row,
  the characters to highlight on each row while filtering, and a header's attention flag. The
  flag is set when a row the rail is not showing under the header needs the user: a terminal's
  bell, or a thread's dot or wait.
- **`selection`** picks that row: the keyboard's row when it is shown, else the focused Agent
  Panel's thread, else the displayed workspace's active terminal, else that workspace's project
  header, each only when the walk shows it. Otherwise nothing is selected: the window shows no
  project the rail lists, or the filter hides it. A stale focus never selects a row that is not
  there.
- **`step`, `first_row`, `last_row` and `parent`** move the keyboard's row (#453). `step` goes
  to the next or previous shown row and stays on the last or the first; with nothing selected
  it starts at the first going forward and the last going back. `parent` is a terminal's or a
  thread's project header, and a header is its own. `first_match` is the first shown row whose
  own name or title matched, where the keyboard's row goes as the filter changes (#457).
- **`cycle_project` and `cycle_row`** are Zed's Next and Previous Project and Thread (#459).
  Both go round the shown rows from the selected one, wrapping at the ends, and with nothing
  selected start at the first going forward and the last going back. `cycle_project` starts
  from the selected row's project header and reaches only headers. `cycle_row` reaches only
  terminals and threads, so from a header it goes to the first row under it, or back to the
  last row above it. A lone row reaches itself; with none of the kind shown, nothing is
  reached.
- **`window_row`** is the terminal or thread row that holds the window's focus: the focused
  Agent Panel's thread, else the active terminal while it holds focus (#454). The rail notes
  each change of it for the switcher's order.
- **`switcher_rows`** gives every terminal and thread as a `SwitcherRow`, never a header, for
  the switcher: first the rows ranked by the `shown_at` the caller passes, most recent first,
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
filter matched.

## Tests

`src/marley_rail.rs`, thirty-three unit tests, including exactly one selected row over every
combination of fold, displayed project, active terminal and focused thread in a two-project
window, and `step` over every shown row of one in both directions.
