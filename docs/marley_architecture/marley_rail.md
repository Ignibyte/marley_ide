# `marley_rail`

The Marley rail's row model, written in the fork for the workbench shell's W2 (#438). Pure and
gpui-free, MIT OR Apache-2.0, with no dependencies.

## What it decides

- **Input:** a `RailSnapshot`, the window as the rail sees it. `projects` holds the project
  groups in the window's order, each a `ProjectSnapshot` (display name, expanded, and its center
  terminals as `TerminalSnapshot`s: id, title, subtitle, bell). `focus` holds the displayed
  workspace's group index and its active terminal's id.
- **`rail_rows`** gives the rows in display order: each project's header, then its terminals
  when it is expanded. It marks the one selected row and a header's attention flag, which is
  set when a folded project hides a terminal whose bell rang.
- **`selection`** picks that row: the displayed workspace's active terminal when its row is
  visible, else that workspace's project header, else nothing (the window shows no project the
  rail lists). A stale focus never selects a row that is not there.
- **`has_attention`** is the rail's notification flag: any listed terminal's bell, folded or
  not.
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

`src/marley_rail.rs`, eight unit tests, including exactly one selected row over every
combination of fold, displayed project and active terminal in a two-project window.
