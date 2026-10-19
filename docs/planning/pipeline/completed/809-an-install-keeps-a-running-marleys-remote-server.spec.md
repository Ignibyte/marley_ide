---
pipeline_id: bd12f8b0-e8c2-463a-86cb-aded9513a4b8
ticket: docs/planning/tickets/closed/TICKET-809-install-keeps-the-running-builds-remote-server.md
status: Phase 4 — Complete PASS
title: An install keeps the remote server of a Marley still running
type: bug
slice: the release install (#502) and the remote server (#445)
references: []
---

## Title
`script/install-marley` keeps the remote servers of the three newest commits it staged. A Marley
that runs through several installs uploads the server of the commit it was built at, so once that
server is pruned every Open Remote Folder from it fails ("no Marley remote server is staged for
linux-x86_64 at commit 3f1b2d6") until a restart. The install should also keep the server of
every commit a running Marley was built at.

## Scope
### In
- `stage_remote_server`'s prune keeps, beside the newest three, each staged server whose commit
  is in the binary of a Marley running on this machine (`running_commits`).

### Out (explicitly deferred)
- The Mac's copy (`fetch_remote_server`, newest three): macOS has no `/proc`, and a running app's
  replaced binary cannot be read there.
- Staging a server for a commit that was never staged (a debug build at an unstaged commit).

## Reference (§20)
N/A — Marley-specific: Zed's dev channel stages and uploads no server at all; Zed's stable
channel downloads the server of its own version from Zed's servers, so it never prunes a cache a
running build needs. Marley's staging (#445) is its own.

### Prior art
- `script/install-marley`, `stage_remote_server`: stages `<commit>.gz` under
  `${XDG_DATA_HOME:-~/.local/share}/marley/remote_servers/dev/linux-x86_64/` and keeps the newest
  three by modification time.
- The binary carries its commit: Zed's `crates/zed/build.rs` sets `ZED_COMMIT_SHA` from `git
  rev-parse HEAD`, and the installed binary of 2026-10-19 holds `ba5aacce78…` once (`grep -ac`).
- Linux keeps a replaced executable readable at `/proc/<pid>/exe` for as long as the process
  runs; `pgrep -x marley` names Marley's processes (its `comm`).
- Nothing in Zed's crates or Cargo.lock owns an install script's cache.

## UI proof
N/A — no UI delta: the install script's prune. Its run is `just shot` (§7), and the prune is shown
by `XDG_DATA_HOME=<scratch> script/install-marley --remote-server-only` over a fixture of staged
servers, one of them the running Marley's commit and the oldest.

## Locked-In Decisions
- D1 — A running Marley's commit is read from its binary through `/proc/<pid>/exe`, not from a
  file the launcher writes: it covers Marleys started before this change, debug builds and e2e
  runs, and needs no new state.
- D2 — Only staged commits are searched for (`grep -F -f` with their names), so a binary is read
  once per running Marley and an unrelated hash in it can only keep a file, never remove one.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an install stages a server and prunes, it shall keep the staged server of each commit a running Marley was built at, even when that server is not among the newest three. | The fixture run's directory listing, before and after |
| REQ-002 | WHEN no running Marley was built at a staged commit, the install shall keep the newest three staged servers and remove the rest, as before. | The same run: the oldest fake commit removed |
| REQ-003 | WHEN Marley starts with the change in the tree, it shall start and draw. | `just shot` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — `running_commits` and the prune; a review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the fixture run of `--remote-server-only`, and `just shot`.
- **P4 Complete** — CHANGELOG, the ledger, close, archive, commit.
