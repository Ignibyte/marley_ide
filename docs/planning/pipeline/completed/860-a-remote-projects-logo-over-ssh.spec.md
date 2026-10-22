---
pipeline_id: 6ed6fde9-49c3-4cd6-9d22-64e0058500d1
ticket: docs/planning/tickets/open/TICKET-860-a-remote-projects-logo-over-ssh.md
status: Phase 4 — Complete PASS
title: A remote project's logo, read once over SSH
type: feature
slice: The Marley look, W1.2 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
A remote project's mark is its own logo: Marley finds a candidate among the remote worktree's
files, reads it once through the project (as Zed's image viewer reads a remote image), keeps the
bytes in its data folder keyed by host and folder, and draws them from there while the connection
is down. A change to the file in the worktree reads it again.

## Scope
### In
- `project_icons.rs`: a remote entry (the project while connected, the kept file), its search,
  and `want_remote`.
- `rail.rs`: `follow_icons` asks for remote groups' marks too, open or closed.

### Out (explicitly deferred)
- The icon an `index.html` declares, for a remote project: reading the page is a second fetch;
  the named candidates (`logo.png` and the rest) are read.
- A logo the user picked (`marley.project_logos`) stays a local file, as today.

## Reference (§20)
N/A — Marley-specific: #564's and #831's project marks, over Zed's remote project
(`Project::open_image`, which reads an image through the remote server).

### Prior art
- `project_icons` (#564, #831): `want`, `search`, `files_changed`, the `ProjectMarks` cache.
- `Project::open_image` and `ImageItem::image` (Zed's image viewer, remote included); the remote
  worktree's synced entries say which candidate exists without a round trip.
- #445's fake SSH hosts (an sshd of the run's own, the staged remote server).
- The brain (consultation 168e2606): nothing on this seam.

## UI proof
Scenario `script/e2e/860-a-remote-projects-logo-over-ssh.sh` (sway): #445's fake host with
`logo.png` in its project; `E2E_REMOTE_SERVER` the server staged for the build's commit.
- `860-01-remote`: the remote project open: its logo in the rail, the strip off (REQ-001).
- `860-02-down`: `local` opened beside the remote project in its window, the host's sshd and
  server stopped, Marley started again with its last session: the remote project's closed row
  still draws the logo, read from the kept copy (REQ-002).

## Locked-In Decisions
- D1 — The kept copy is `<data dir>/project_logos/<hash of host and folder>.<extension>`; a
  toolchain that hashes differently reads the file again once.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a remote project with `logo.png` opens, Marley shall draw that logo as the project's mark. | `860-01-remote` |
| REQ-002 | WHILE the remote host cannot be reached, Marley shall draw the logo it kept for that project. | `860-02-down` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
