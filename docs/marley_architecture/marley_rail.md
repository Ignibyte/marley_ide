# `marley_rail`

The Marley rail's row model, written in the fork for the workbench shell's W2 (#438) and grown
with agent threads in W3 (#439) and agent CLIs in W4 (#440). Pure and gpui-free, MIT OR
Apache-2.0; its one dependency is the equally pure `marley_agent`.

## What it decides

- **Input:** a `RailSnapshot`, the window as the rail sees it. `projects` holds the project
  groups in the window's order. Each is a `ProjectSnapshot`: its display name, whether it is
  expanded, its center terminals as `TerminalSnapshot`s (id, title, subtitle, bell, and the
  agent CLI in the foreground with its status, if any), and its
  agent threads as `ThreadSnapshot`s (key, title, status, attention), in the order the collector
  gives. `focus` holds the displayed workspace's group index, its active terminal's id, and
  the thread its Agent Panel shows while the panel holds focus.
- **`rail_rows`** gives the rows in display order: each project's header, then, when it is
  expanded, its terminals and then its threads. It marks the one selected row and a header's
  attention flag, which is set when a folded project hides something that needs the user: a
  terminal's bell, or a thread's dot or wait.
- **`selection`** picks that row: the focused Agent Panel's thread when its row is visible,
  else the displayed workspace's active terminal when its row is visible, else that
  workspace's project header, else nothing (the window shows no project the rail lists). A
  stale focus never selects a row that is not there.
- **`has_attention`** is the rail's notification flag: any listed terminal's bell, or thread
  dot or wait, folded or not.
- **`thread_status`** ranks what a live conversation reports: a pending confirmation over an
  error over a running agent, else done.
- **`thread_attention`** says whether a thread's dot is lit after a rebuild. A run that just
  ended (running before, done or failed now) lights it unless the thread is shown, and it
  stays lit until the thread is shown.
- **`working_directory_label`** is a terminal row's second line: the directory relative to the
  project root, or with the home directory written as `~` outside it. It is empty at the root
  itself and when the terminal cannot tell (no path, or an empty one).

## Why a crate of its own

Every decision the rail makes is here, unit-tested in milliseconds. The gpui side's tests
build a binary that carries Zed's agent crates, so the less logic lives there, the cheaper
its tests.

## Consumers

`marley_workbench`'s `Rail` builds the snapshot from the live window, stores it, and draws the
rows this crate returns. It keeps no ordering or selection state of its own.

## Tests

`src/marley_rail.rs`, sixteen unit tests, including exactly one selected row over every
combination of fold, displayed project, active terminal and focused thread in a two-project
window.
