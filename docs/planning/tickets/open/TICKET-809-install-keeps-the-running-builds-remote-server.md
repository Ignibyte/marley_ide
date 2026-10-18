# TICKET-809 — An install deletes the remote server of the Marley still running

- **Ticket:** LOCAL #809 (bug, install)
- **Owner:** b6416b37-a6bf-4172-91e4-8d05bc38cda5
- **Pipeline doc:** (set at Plan)
- **Source ticket:** Chad, 2026-10-17: Open Remote Folder failed with "no Marley remote server is
  staged for linux-x86_64 at commit 3f1b2d6".
- **Status:** open

## Summary
`script/install-marley` keeps the remote servers of the three newest commits it staged
(`~/.local/share/marley/remote_servers/dev/linux-x86_64/`). Chad's running Marley was built at
`3f1b2d6`; three installs later its server was gone, and every Open Remote Folder from that
Marley failed until a restart. The install should keep the server of every commit a running Marley
was built at, beside the newest three.

## Acceptance
After installs that would prune it, the running Marley's server is still staged, and Open Remote
Folder works from it.
