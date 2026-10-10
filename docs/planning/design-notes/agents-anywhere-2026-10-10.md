# The agents-anywhere shelf: nine queued specs (2026-10-10)

Chad, 2026-10-10, after trying the Marley agent: Agent Panel threads exist only in a project and
only in that project's rail rows; Marley and Rusty are agents for everything, not for one folder;
and with Zed's thread history gone with Zed's sidebar there is no way back to an old conversation.
His direction:

> Basically what i'd like with the agents is you can use cli or use the agent panel but it can be
> used anywhere with and panel on there and not isolated or forced to the folder. We need to
> specify where we want to open. Bonus points if we can open on a remote via the harness.

He also asked for most things to open in a center tab, from icons at the bottom right, and set the
goal "lets make tickets and build it".

| Ticket | One line |
|---|---|
| #734 | An agent thread starts in a tab of any group, on a folder Marley names |
| #735 | The New Agent picker: agent, thread or CLI, and where |
| #736 | Thread tabs come back after a restart |
| #737 | The Threads page |
| #738 | Marley and Rusty in tabs |
| #739 | Home, Rusty, Threads and Marley at the status bar's right |
| #740 | One harness per host (bonus) |
| #741 | A new agent on a remote host, through the harness (bonus) |
| #742 | A thread on a remote seat, through `rh acp --seat` (bonus; rustal-harness TICKET-116) |

## Order

734 → 735 → 736 → 737 → 738 → 739 → 740 → 741 → 742. #735 builds its thread choice on #734's tab;
#736 restores #734's tab; #737 opens threads with #734's tab; #738 is a #734 tab on Marley's own
folder; #739's buttons open #737's page and #738's tab. #741 needs #740's hosts in #735's where,
and #742 needs the harness's TICKET-116.

## The one technical catch

Zed answers an ACP agent's file reads and writes through the project
(`AcpThread::read_text_file` and `write_text_file` resolve with `project_path_for_absolute_path`),
and refuses a path outside its worktrees. A thread working in a folder the hosting group does not
hold would fail every file call. The folder therefore joins the group's project as a hidden worktree
(`find_or_create_worktree(path, false)`), which Zed already does for a file opened from outside a
project. Zed scans and watches a hidden worktree whole, so the home folder, or a folder holding
it, never joins: a thread there reaches files only in open projects, as Zed's own threads do. #734
proves this before anything builds on it.

## What changes from Chad's earlier answers

On 2026-10-10 he first chose fixed rows for Marley and Rusty under Home, and Threads as a button in
the rail's header beside the Rusty button. His later direction (everything opens in a tab, icons at
the bottom right) replaces both: the Threads page is where Marley's and Rusty's conversations live,
and the buttons sit in the status bar.
