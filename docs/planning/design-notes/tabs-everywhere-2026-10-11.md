# The tabs-everywhere shelf: seven tickets (2026-10-11)

Chad, 2026-10-11, after using #734 to #742 in his daily Marley:

> 1.) The rusty panel shouldnt be there anymore on start up
> 2.) I wanted everything to open in the main panel now with a tab. THe includes everything in the
> bottom right. With the ability to split pane it and drag it all
> 3.) I dont see anywhere where i can create a new agent thread from a specific folder
> 4.) we should think about commands we can run like marley <command> where it can open things
> inside marley. for example i could cd /whatever/folder and do marley agent-thread
> --provider=claude and it would pop open a new agent thread

and, of Zed's Agent Panel: "The agent pane inside zed still has the issue of needing to be in a
project." (`AgentPanel::has_open_project`: with no visible worktree the panel draws Open Project
and refuses a new thread.)

His answers to the follow-up questions, the same day:

- The Rusty group leaves the rail ("i cant manually remove it. but rusty now is in the bottom panel
  which is what we want"). The Home group stays and is renamed Default.
- One New Agent: "if its a project then default to the project folder chosen but allow to switch.
  Remove New Agent THread with just New Agent." The sparkle does the same. "New Agent from Thread"
  reopens an old conversation ("my thought is that you would re open it").
- Every side panel opens as a tab: "everything opens in the main pane now then the user can just
  split panel it out themselves. that way its consistent".
- Threads come back to the top left, as a switch in the rail, and as an entry on the Home page.
- Marley and Rusty move to the title bar: "Rusty and Marley go to the top right next to (?) and
  Avatar instead. That gives them more spotlight without mucking with the left panel tabs that
  should only be for swapping."
- `marley <command>`: "yes to all of these".
- He will redraw the icons.

| Ticket | One line |
|---|---|
| #743 | The rail's top: the Rusty group leaves, Home becomes Default |
| #744 | One New Agent, and Reopen Thread… |
| #745 | Threads in the rail |
| #746 | Marley and Rusty in the title bar |
| #747 | Side panels open as tabs: the mechanism, Files, Git and Outline |
| #748 | Side panels open as tabs: Collab, Fleet, Knowledge and Containers; Debug to the left |
| #749 | `marley <command>` |

## Order

743 → 744 → 745 → 746 → 747 → 748 → 749. #745 and #746 move what #739 put in the status bar;
#748 reuses #747's tab; #749's `marley panel` opens #747's and #748's tabs.

## Debug

Chad first made it the exception ("debug which we should move to the bottom right"), then said
everything opens in the main pane. Asked whether Debug keeps its bottom panel: "yes debug moves to
bottom left and stays the exception". The debug panel stays in the bottom dock and its button moves
to the status bar's left end.
