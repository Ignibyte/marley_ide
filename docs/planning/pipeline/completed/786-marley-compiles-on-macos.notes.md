# Marley compiles on macOS — Notes

- **Local ticket doc:** [TICKET-786](../../tickets/open/TICKET-786-marley-compiles-on-macos.md)
- **Pipeline spec:** 786-marley-compiles-on-macos.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-16: "see what it would take to get rusty running there + marley and
  doing a full import over there. Im considering going back to that box as my daily driver",
  then "lets make it happen" and "lets make a game plan for this". The plan (in the private ops
  handbook) makes the Mac the desk and keeps the Linux box as the engine. This ticket is its
  first Marley step: Marley builds for the Mac at all.
- **Classification / tier:** chore, macOS. Marley crate only (`marley_mcp`), no Zed hunk.
- **Recall (§18.3):**
  - `failures.md`: a value split into two `#[cfg]`-gated functions leaves an unkillable mutant on
    a single-platform runner. Here, D2 gives the per-platform functions a return type with no
    `Default`, so their mutants are unviable rather than missed.
  - `prevention-rules.md` holds rules from the pre-fork Marley's macOS self-test harness
    (`screencapture` needs the display awake and unlocked; `activate()` does not make a gpui
    window key). They bear on #787's checks on the Mac, not on this compile fix.
  - Completed pipelines: #652 built the agent socket and its `SO_PEERCRED` check; #755 named it as
    the proof of a terminal for `marley-agent`'s reports.
  - The brain (consultation `edeaf418fafb4f61905b4da57ad20559`): nothing on a Mac build of the
    fork or on peer credentials.
- **Discovery:**
  - `crates/marley_mcp/src/agent_socket.rs:131`: `rustix::net::sockopt::socket_peercred(&stream)`,
    then `peer.uid != uid` (:138) and `peer.pid.as_raw_nonzero()` (:142).
  - The module is `#[cfg(unix)]` (`marley_mcp.rs:39`). rustix 1.1.4 gates `socket_peercred` with
    `#[cfg(linux_kernel)]` (`src/net/sockopt.rs:1689`).
  - `marley_mcp/Cargo.toml:11-12`: rustix under `cfg(unix)` with `net` and `process`. `[lints.rust]
    unsafe_code = "deny"` (:30).
  - nix 0.30.1 (`src/sys/socket/sockopt.rs:645-663`): `LocalPeerCred` (`SOL_LOCAL`,
    `LOCAL_PEERCRED`, `XuCred`) and `LocalPeerPid` (`SOL_LOCAL`, `LOCAL_PEERPID`, `c_int`), both
    `#[cfg(apple_targets)]`, behind the `socket` feature. `XuCred::uid()` (`src/sys/socket/mod.rs:530`).
  - The Mac's first `cargo check --keep-going -p zed --bin marley` (clone at `5020ee34f8`, toolchain
    1.98.1, `-j 6`, niced) started 2026-10-16. Its result is recorded in Phase 2.
- **Decisions:** D1 to D3 in the spec.

### Design
- **Approach.** In `agent_socket.rs`:
  - a private `struct Peer { uid: u32, pid: u32 }`, with no `Default`, and `fn peer(stream:
    &UnixStream) -> std::io::Result<Peer>` in three `#[cfg]` forms:
    - Linux: rustix `socket_peercred`, `uid.as_raw()`, pid `as_raw_nonzero().get().unsigned_abs()`;
    - macOS: nix `getsockopt(stream, LocalPeerCred)?.uid()` and
      `getsockopt(stream, LocalPeerPid)?`, a pid at or below zero is an `InvalidData` error, and
      nix's errors become `std::io::Error`;
    - every other Unix: `ErrorKind::Unsupported`.
  - `serve` calls `peer`, compares `peer.uid` with `rustix::process::getuid().as_raw()`, and hands
    `peer.pid` on as today. The accept loop keeps passing the user. The log lines stay as they
    are.
- **File manifest:**
  - `crates/marley_mcp/src/agent_socket.rs` (Marley crate): the `Peer` type and the three
    `peer` forms.
  - `crates/marley_mcp/Cargo.toml` (Marley crate):
    `[target.'cfg(target_os = "macos")'.dependencies] nix = { workspace = true, features = ["socket"] }`.
  - `Cargo.lock`: `marley_mcp` gains `nix`.
  - No Zed crate, so no ledger row.

### Visual check plan
- REQ-001: the Mac's `cargo check --keep-going -p zed --bin marley` with the change, its log
  read, exit 0. Run over SSH on the Mac's own clone, with the diff applied there before the
  commit.
- REQ-002, REQ-003: the review of the diff. No scenario reaches a second user or a macOS
  `marley-agent` today; agent reports on macOS also need the `/proc` process chain, a later
  ticket.
- REQ-004: `just shot` on the box: Marley starts, nothing else to see. The gate green.

### Risks
- **More errors on the Mac.** The survey read the code; only the Mac's compiler can say. More
  compile errors land here, and the scope is unchanged.
- **The Mac's memory.** A check of Zed on 16 GB, next to Chad's Codex and Claude sessions. It
  runs niced at `-j 6`, and `memory_pressure` is read while it runs.
- **One cargo at a time.** That holds on each machine. The box and the Mac have separate target
  directories and separate cargo homes.

## Phase 2 — Code
- **The Mac's checks.** Each ran on the Mac's clone with `cargo check --keep-going -j 6 -p zed --bin
  marley`, niced, toolchain 1.98.1, logs under `/Volumes/Offload/Builds/marley-logs/` there.
  - check-0, unchanged tree at `5020ee34f8`: one error, E0425 `socket_peercred` not found in
    `marley_mcp`. Everything that does not depend on `marley_mcp` checked clean.
  - check-1, with the peer fix: `marley_mcp` clean. Two E0308 errors in
    `marley_workbench/src/codex_server.rs`:
    - `last_line` took `smol::process::ChildStderr`;
    - `watch` took `smol::process::Child`.

    `crate::process::serve` returns `util::command::Child`, which is smol's on Linux and on macOS
    Zed's own posix_spawn child (`util/src/command/darwin.rs:231`), whose stderr is
    `Async<std::fs::File>`.
  - check-2, with both fixes: **exit 0**. The `zed` crate and the `marley` binary check clean on
    `aarch64-apple-darwin`. One warning was left: `single_instance::hand_verb` is never used on
    macOS, since its one caller, `commands::hand_to_running`, is Linux and FreeBSD only (#775).
    `hand_verb` took the same `cfg`.
  - clippy-0 on the Mac, with all three fixes: `cargo clippy --keep-going -p` every `marley_*`
    crate `-- -D warnings` (lib targets), **exit 0**, no warning in a Marley crate. The one note
    left is cargo's future-incompatibility report for `block` 0.1.6, a dependency of Zed's macOS
    platform crates. It is not Marley's.
- **What was built:**
  - `agent_socket.rs`: `Peer { uid, pid }` and `peer_of` in three forms, as designed.
  - `codex_server.rs`: `last_line` takes any `futures::AsyncRead + Unpin`, and `watch` takes
    `util::command::Child`. Both platforms' child types have the `id`, `status` and `kill` that
    `watch` calls.
  - `marley_mcp/Cargo.toml`: nix with `socket`, for macOS only.
  - `single_instance.rs`: `hand_verb` gated to Linux and FreeBSD, as its caller is.
- **Deviation:** `codex_server.rs` was not in the manifest. The survey read `prepare`'s Linux-only
  refusal and missed that the launch path names smol's types where Zed's wrapper differs by
  platform. It is a compile error, so it is in scope (D3). On macOS the server is never started,
  since `prepare` refuses it off Linux, so nothing changes at run time.
- **Review of the diff:**
  - REQ-002: the macOS uid comes from `LOCAL_PEERCRED` and goes through the same `!=` against
    `getuid()` before any read.
  - REQ-003: the pid comes from `LOCAL_PEERPID`; zero or a negative value is an error, so the
    connection is dropped rather than handed pid 0.
  - REQ-004: the Linux arm is the old call, with `uid.as_raw()` compared against
    `getuid().as_raw()` in place of two `Uid` values. Same meaning.
  - No `unsafe`, and no `let _ =`. No Zed crate, so no ledger row.
- **Gate:** run 1 (the peer fix alone) was red on gate:17 only. cargo-sort wants the
  `cfg(target_os = "macos")` table before `cfg(unix)`; `cargo sort` put it there. Run 2, on the whole diff (the peer fix, `codex_server.rs`,
  `hand_verb`'s cfg): **GATE GREEN**, 17 of 17.

## Phase 3 — Test
- **The run:** `just build`, then `COMPOSITOR=sway just shot 786-linux-starts`, shots in the
  session scratchpad. Chad's Marley was open on Hyprland, so the shot ran in a headless sway of its
  own. Hyprland had one Marley window before and after, and nothing was added to it.
- **`786-linux-starts.png` (REQ-004):** Marley starts in the Marley layout with a `repo — bash`
  terminal at its prompt, and Zed's Unrecognized Project prompt over it, which every fresh
  scratch project gets. Nothing else is new to see. The run's Marley log has no `agent socket`
  line, so the agent socket bound and logged nothing. The Vulkan loader messages and the
  llama.cpp provider's 404 are the headless run's own: pre-existing, not in scope.
- **REQ-001:** the Mac's check-2 log (Phase 2), exit 0 on `aarch64-apple-darwin`, and clippy-0,
  exit 0 over every Marley crate.
- **REQ-002, REQ-003:** the review of the diff in Phase 2. No scenario reaches them: a second
  user on the Mac, or a `marley-agent` report on macOS, which also needs the `/proc` process chain
  that the move's plan leaves for a later ticket.

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md`, Added: "Marley compiles on macOS".
  - `docs/marley/three-prong-plan.md`, Cross-cutting: a macOS bullet (the desk and engine
    split, #786 done, #787 next, what does nothing on macOS, checks on Linux).
  - `docs/marley_architecture/marley_mcp.md`: the agent socket's `peer_of` per platform.
  - `docs/marley_architecture/marley_workbench.md`: Codex's process task holds
    `util::command::Child`; `hand_verb` is Linux and FreeBSD only.
  - No Zed crate touched, so no touchpoint row.
- **Ledger:**
  - F-claude-786-marley-did-not-compile-on-macos-001
  - PR-claude-786-code-holding-a-util-command-child-names-zeds-type-001
  - L-claude-786-keep-going-stops-at-a-failed-crate-so-a-port-takes-rounds-001
- **Brain:** consultation `edeaf418fafb4f61905b4da57ad20559` closed by
  `decisions/marley-builds-for-macos-the-mac-is-the-desk-and-the-linux-box-the-engine`,
  follow-up by 2026-10-30.
- **Ticket:** closed and archived with this pair.
