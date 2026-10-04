# TICKET-650 — Codex's state from its own App Server

- **Ticket:** LOCAL #650 (feature, prong 2 C1, a terminal's own agent state: Codex)
- **Owner:** claude-opus-5-5, 2026-10-03 (/spec)
- **Pipeline doc:** ../../pipeline/completed/650-codex-app-server-state.spec.md
- **Source ticket:** `design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md`, idea B1,
  part 1 (part 2, #651, is approvals and prompts); Chad, 2026-10-02, on B1: "Yes"
- **Status:** closed

## Summary
Codex in a Marley terminal gives Marley nothing typed today: its rail row guesses from 2 s of quiet
output, the close guard guesses the same way, and its permission chip reads its command line.
Codex's TUI is already a client of an App Server, and `codex --remote unix://PATH` names the server
it uses. With `marley.codex_app_server` on (off by default), Marley starts one `codex app-server
--listen unix://<socket>` for each Codex terminal it launches, types `codex --remote
unix://<socket>` into the terminal, and joins the same server as a second client once the TUI has
joined. Because the server belongs to one terminal, every thread it loads is that terminal's: no
guess maps a thread to its terminal. Marley subscribes to the TUI's thread and reads its status
(working, waiting on an approval, waiting on input, idle, failed), its token use and its sandbox and
approval policy. The rail's Codex row, the approvals inbox, the rail's attention order and the close
guard read that state instead of the quiet timer, and the chip reads the thread's sandbox instead of
the arguments. The integration registers its tested Codex range (0.155.1 to 0.158.0) with #648's
version table and stays off outside it. Marley sends Codex nothing that acts in this slice;
answering approvals and sending prompts are #651.

## Acceptance
With the setting off, or outside the tested range, Codex starts and reads as it does today.
On, a Codex Marley launches runs against its own App Server; its row says working, waiting on
an approval or on input, idle or failed from the thread's own status, even through a quiet
spell; its token use shows on the row; a wait enters the inbox as Codex's; the close guard
asks before ending a working one; the chip follows the thread's sandbox when it changes
mid-session; a server that stops shows as a failed row with the reason. Nineteen EARS criteria
in the spec.
