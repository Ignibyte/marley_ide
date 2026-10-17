# TICKET-796 — A terminal's rail row holds two lines

- **Ticket:** LOCAL #796 (bug, the rail)
- **Owner:** 88e2e50b-c483-4f95-8e5b-a35778613e8d
- **Pipeline doc:** [796-terminal-rows-hold-two-lines.spec.md](../../pipeline/completed/796-terminal-rows-hold-two-lines.spec.md)
- **Source ticket:** Chad, 2026-10-17: "the left terminal panes show what they are running
  underneath the names. But once they hit 2 lines it makes the pane jump. We need to probably
  figure out a way to handle this or make the height always assume 2 lines".
- **Status:** closed

## Summary
`row_card` (`crates/marley_workbench/src/rail.rs`) sets a row's height from how many lines sit
under its title: `h_11` (2.75 rem) for none or one, 3.5 rem for two, 4.5 rem for three or more. A
plain terminal's row shows its folder when it is outside the project root, and under it the
command it runs or ran last (#551). In a subfolder, the row grows when the first command starts,
and every row under it moves down. `cd` into and out of the root makes the folder line come and
go, with the same jump. An agent's row does the same when its activity line (#519) comes and goes
under its status.

## Acceptance
A terminal's row is two lines tall whatever it shows under its title, so a command's line or a
folder's line coming or going moves neither the row's title nor the rows under it.
