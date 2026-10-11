# TICKET-758 — A remote manager's thread

- **Ticket:** LOCAL #758 (feature, the control plane: agents anywhere)
- **Owner:** 5b7350d2-26f7-4126-a833-9a692beba9e2
- **Pipeline doc:** [758-a-remote-managers-thread.spec.md](../../pipeline/queued/758-a-remote-managers-thread.spec.md)
- **Source ticket:** Chad, 2026-10-11: "is there a way to do remote agent threads. meaning if manager
  lives on a remote server and we dont want to steer them through the cli. Using or not using our
  harness (preferrable our harness)", then "ok we can build it. once things are spec and then add
  the things for the missing manager." Research: [harness and Zed's remote server](../../design-notes/harness-and-zed-remote-2026-10-11.md).
- **Status:** open

## Summary
Marley follows a harness on every host it lists (#740), but the Agent Panel's Manager entry (#694),
the manager-thread follower (#688) and its Needs you entries serve only the first harness. A manager
on another machine can be reached only through the CLI. Every followed harness with a manager gets
its own entry, `Manager · <harness>`, its own notices and Needs you entries, and a thread tab that
talks to that manager through `ssh HOST rh --state ROOT acp`, which the harness already serves. A
Manager thread in a tab counts as in front, and a new one shows the thread's retained records.

The same change makes a seat's or a manager's thread open from any window. Their agent-server
entries live in this machine's in-memory settings, which a remote project never sees, so today such
a thread opened from a remote project's window likely fails. Harness threads are built on a local
project of Marley's own, whatever the window.

No harness change is needed.

## Acceptance
With a manager on a second harness, its report raises a notice and a Needs you entry naming that
harness; the click opens a `Manager · <harness>` tab showing the report; a message sent there
reaches that harness's thread only. A seat's thread opens from a remote project's window (checked by
hand on a real host).
