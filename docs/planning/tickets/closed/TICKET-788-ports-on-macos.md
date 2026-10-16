# TICKET-788 — The Ports list on macOS

- **Ticket:** LOCAL #788 (bug, macOS)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** [788-ports-on-macos.spec.md](../../pipeline/completed/788-ports-on-macos.spec.md)
- **Source ticket:** seen in #787's check on the Mac, 2026-10-16.
- **Status:** closed

## Summary
On macOS, `marley_workbench::ports` reads the listening sockets from `/proc/net/tcp` and
`/proc/net/tcp6`, which macOS does not have. Every 3 s it logs "reading the listening ports: …
reading /proc/net/tcp6: File not found" (115 lines in the first six minutes of a Mac run), and the
Ports list, the container ports and the served-URL footer stay empty. On macOS the listening
sockets and their processes come from the platform instead (`lsof -nP -iTCP -sTCP:LISTEN`, or
libproc), so the list works; at the least, a platform with no source logs it once and stops
polling.

## Acceptance
On the Mac, a program listening on a port shows in the Ports list with its process, and Marley's
log carries no repeating warning from `ports`.
