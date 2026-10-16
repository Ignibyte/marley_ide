# TICKET-786 — Marley compiles on macOS

- **Ticket:** LOCAL #786 (chore, macOS)
- **Owner:** c70cc71d-da30-4123-80da-1db398eb7911
- **Pipeline doc:** [786-marley-compiles-on-macos.spec.md](../../pipeline/completed/786-marley-compiles-on-macos.spec.md)
- **Source ticket:** Chad, 2026-10-16: "see what it would take to get rusty running there +
  marley ... Im considering going back to that box as my daily driver", then "lets make it
  happen". The move's plan keeps the Linux box as the engine and builds Marley for an Apple
  Silicon Mac as the desk.
- **Status:** closed

## Summary
Marley has only been built on Linux. Upstream Zed builds on macOS, and a survey of Marley's own
crates found one thing that stops a macOS build: the agent socket's peer check
(`crates/marley_mcp/src/agent_socket.rs`) calls `rustix::net::sockopt::socket_peercred`, which
the locked rustix gates to Linux, while the module builds on every Unix. macOS gets its own peer
check (`getpeereid`), so the socket still refuses a peer of another user. Anything else the first
macOS build turns up is fixed here too, as long as it is a compile error; features that compile
and do nothing on macOS (`/proc`, systemd) are later tickets.

## Acceptance
`cargo check -p zed --bin marley` passes for `aarch64-apple-darwin` on a Mac, and the Linux build
and gate stay green.
