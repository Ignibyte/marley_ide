# TICKET-658 — The Tasks tab: Rusty's to-do lists in a center tab

- **Ticket:** LOCAL #658 (feature, Rusty in Marley R7, its Tasks half;
  `docs/marley/rusty-in-marley.md` R-D3, R-D9 and R-D10)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/queued/658-rusty-tasks-tab.spec.md
- **Source ticket:** `docs/marley/rusty-in-marley.md` (R-D3's TasksPage row: "A **Tasks** tab over
  the twelve task tools"; R-D9's fixed row; R-D10's `sortablelist-reorderablelist` row; slice R7);
  the follow-up batch #654 to #659, 2026-10-03
- **Status:** open

## Summary
Rusty's Qt app has a Tasks page (`TasksPage.qml`): to-do lists on the left, the chosen list's tasks
on the right, keyboard first. This ticket rebuilds it as a Marley center tab drawn from
`rusty-mcp`'s twelve task tools. A task is added from a field above the list, checked done by its
checkbox or Space, renamed in its row, archived and restored, deleted for good after a prompt, and
moved by a drag or by Alt-Up and Alt-Down; lists are made, renamed and deleted the same way. Every
change is a Rusty tool call, read back, so Rusty stays the only writer. The tab opens from the rail
Brain view's fixed row (#644) and from `rusty: open tasks`. It reads again when Rusty sends
`list_changed`, when it takes the focus, and on its Refresh button: a task an agent's own
`rusty-mcp` writes is announced to no one until Rusty's TICKET-035 lands. The drag is the one
Marley's rail already draws (#602, on Zed's tab bar's pattern), so Ely's `SortableList` and
`Reorder` are read and not ported. It comes after #644 and builds on #643's client and crate and
#645's `rusty` actions. Linking a rail project to its list is #655; the Decisions half of R7 is
#659.

## Acceptance
With Rusty on and a stand-in `rusty-mcp` serving made-up lists, the fixed row's Tasks entry and
`rusty: open tasks` open one Tasks tab with the lists and the first list's tasks in Rusty's order.
Adding, checking, renaming, archiving, restoring and deleting a task, and making, renaming and
deleting a list, each go through the matching Rusty tool and show what Rusty reads back; deletes
ask first. A drag shows a preview and a line and drops the task in the target's place; Alt-Up and
Alt-Down move the selected task; both send `reorder_tasks`. A `list_changed` redraws the shown tab
and marks a hidden one to read when it shows; a write Rusty does not announce appears when the tab
takes the focus or on Refresh. A refusal shows Rusty's message. Rusty switched off empties the tab.
