---
pipeline_id: ac49d1b4-4761-4a4f-8d26-c6368c3e6930
ticket: docs/planning/tickets/closed/TICKET-786-marley-compiles-on-macos.md
status: Phase 4 — Complete PASS
title: "Marley compiles on macOS"
type: chore
slice: macOS
references: []
---

## Title
Marley has only ever been built on Linux. Chad, 2026-10-16, is moving his daily driver to an
Apple Silicon Mac, with the Linux box kept as the engine ("lets make it happen"), so Marley has to
build for `aarch64-apple-darwin`. A survey of Marley's crates found one compile error there: the
agent socket's peer check calls `rustix::net::sockopt::socket_peercred`, which the locked rustix
1.1.4 compiles only for Linux, while `agent_socket` builds on every Unix. A first `cargo check` on
the Mac decides whether it is the only one.

## Scope
### In
- `crates/marley_mcp/src/agent_socket.rs`: the peer's user and process read per platform; on
  macOS through `LOCAL_PEERCRED` and `LOCAL_PEERPID`, so the socket still refuses a peer of
  another user and still hands the app the peer's process.
- Any other error that stops `cargo check -p zed --bin marley` on the Mac, as long as it is a
  compile error.

### Out (explicitly deferred)
- Features that compile on macOS and do nothing or fail there: agent reports' process chain and
  stall checks (`/proc`), the Ports and Containers lists, the Browser tab's `systemd-run`,
  `marley <verb>`, Codex's app server, the Omarchy theme. They are later tickets in the move's
  plan.
- The Mac app bundle and a Mac install path: #787.
- A macOS keymap.

## Reference (§20)
N/A — Marley-specific: the agent socket (#652) is Marley's own, and nothing in Zed reads a Unix
socket peer's credentials. The macOS form keeps the Linux behavior: a peer of another user is
refused before its line is read, and the app gets the peer's process.

### Prior art
- **The code we ship:** `nix` 0.30.1 is already in `Cargo.lock` and a workspace dependency
  (`Cargo.toml:729`; `util` and `zed` use it on Unix). With its `socket` feature it has, for Apple
  targets, `sockopt::LocalPeerCred` (an `XuCred` whose `uid()` is the peer's effective user) and
  `sockopt::LocalPeerPid` (the peer's pid), both through the safe `getsockopt`. `nix::unistd::getpeereid`
  gives the user but no pid. rustix 1.1.4 has only Linux's `SO_PEERCRED`. `std`'s
  `UnixStream::peer_cred` is unstable. `marley_mcp` denies `unsafe_code`, so `libc` directly is out.
- **Zed's crates:** no peer-credential code anywhere in `crates/` besides `agent_socket.rs`.
- **The behavior maps:** nothing applies; this is a platform call, not behavior.

## UI proof
N/A — no UI delta: Linux behavior is unchanged (the Linux arm is the existing call), and the
macOS proof is the check on the Mac, which no scenario reaches. Its run is `just shot`.

## Locked-In Decisions
- D1 — macOS reads the peer through `nix`'s `LocalPeerCred` and `LocalPeerPid`, added to
  `marley_mcp` as a macOS-only dependency with the `socket` feature. Not `getpeereid`, which has
  no pid; not `libc`, which needs `unsafe`.
- D2 — One `peer` function per platform returns a `Peer { uid, pid }` with no `Default`: Linux
  through rustix as today, macOS through nix, and any other Unix an `Unsupported` error, which
  refuses every peer. `serve` keeps one comparison against the user Marley runs as. No `Default`
  keeps cargo-mutants' `Ok(Default::default())` mutants of the per-platform functions unviable
  instead of unkillable on a Linux runner (the cfg-gated-functions failure in `failures.md`).
- D3 — Compile errors only. Anything that compiles on macOS and misbehaves is named in the move's
  plan and left for its own ticket.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `cargo check -p zed --bin marley` runs on the Mac (`aarch64-apple-darwin`), it shall finish with no error | the Mac's check log and its exit code |
| REQ-002 | WHEN a process of another user connects to the agent socket on macOS, the socket shall drop the connection before reading its line | review of the diff: the macOS arm's uid comes from `LOCAL_PEERCRED` and meets the same comparison as Linux's |
| REQ-003 | WHEN a process of the same user connects on macOS, the handler shall receive that process's pid | review of the diff: the pid comes from `LOCAL_PEERPID` |
| REQ-004 | WHILE Marley runs on Linux, the agent socket shall read the peer through `SO_PEERCRED` as before | review of the diff; `just shot` shows Marley starting; the gate green |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the per-platform peer read; a review of the diff; `script/gates.sh --diff` green
  on the box; `cargo check` on the Mac green.
- **P3 Test** — `just shot` on the box; the Mac's check log read.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
