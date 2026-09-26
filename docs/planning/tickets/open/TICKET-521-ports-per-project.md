# TICKET-521 — Ports per project in the rail

- **Ticket:** LOCAL #521 (feature, prong 3 with the rail; the `ports_list` tool is prong 2's MCP surface)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/queued/521-ports-per-project.spec.md
- **Source ticket:** The Orca survey of 2026-09-25, item 6, "Ports that belong to a project" (`docs/orca_architecture/README.md`, with its list of new tickets), drawn from report 03 §2.3, report 05 §3 item 5 and report 02 §3 item 7; specced because Chad asked on 2026-09-25 for every item decided that day to be specced.
- **Status:** open

## Summary
Nothing in Marley says which dev server belongs to which project, or which port a server took after Vite or Next moved it to a free one. A new pure crate, `marley_ports`, reads the machine's listening TCP sockets from `/proc/net/tcp` and `/proc/net/tcp6`, finds each socket's process through `/proc/<pid>/fd`, and gives the listener to the project whose folder holds the process's working directory. The rail lists a project's live ports under it, each with Open (a Browser tab of the project on that port), Copy and Stop, and a `ports_list` MCP tool gives agents the same list, so they stop guessing ports.

## Acceptance
A server started anywhere with its working directory inside a project shows as a row under that project within five seconds, and leaves when it stops; Open, Copy and Stop work from the row; `ports_list` returns each project's listeners; a server outside every project, another user's server, and Marley's own listeners show nowhere.
