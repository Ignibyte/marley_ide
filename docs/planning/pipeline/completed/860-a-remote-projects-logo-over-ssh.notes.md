# A remote project's logo, read once over SSH — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-860-a-remote-projects-logo-over-ssh.md
- **Pipeline spec:** 860-a-remote-projects-logo-over-ssh.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "lets finish up the remaining tickets besides the mac issues".
- **Classification / tier:** feature, M. Marley crate only.
- **Recall (§18.3):** `Worktree::load_binary_file` refuses remote worktrees, but
  `Project::open_image` reads through the remote server; the rail asks only local projects for
  marks (`follow_icons`). The brain had nothing.
- **Design:** `Entry::remote` holds the project (weak) and the kept file; `search` takes the
  remote way for such an entry: the kept copy first (so a closed or down project has its mark),
  then, connected, the first candidate the remote worktree lists, read with `open_image`,
  decoded, and written to the kept file.
- **Manifest:** `project_icons.rs`, `rail.rs` (Marley).

## Phase 2 — Code
- `project_icons.rs`: `Remote`, `want_remote`, `remote_icon`, `Candidate`, `open_candidate`,
  `keep`, `forget`; `search` takes the remote way for a remote entry following the folder.
- `rail.rs`: `follow_icons` asks remote groups too, open or closed.
- Review: a host that answers without a logo first fell back to the stale kept copy; `Candidate`
  now tells "cannot reach" from "holds none", and the latter drops the copy.
- Gate: dylint refused `async` blocks with no `.await` in `background_spawn`; they are
  `futures::future::lazy`, as the local search's. Then `just gate-diff` 17 passed, 0 failed.

## Phase 3 — Test
- Scenario `script/e2e/860-a-remote-projects-logo-over-ssh.sh` under sway: #445's fake host with a
  green `logo.png`, `E2E_REMOTE_SERVER` the server install-marley staged for HEAD.
- `860-01-remote` (REQ-001): the remote project's window, its rail header the green logo.
- `kept.txt`: `<profile>/project_logos/<hash>.png`, 99 bytes.
- `860-02-down` (REQ-002): after a restart with the host down, the window's closed `project` row
  draws the green logo, dimmed; a fresh process could only read it from the kept copy.
- What it took: a restored window whose remote connection fails lists no group, so the scenario
  makes the remote project's window also hold `local` (the local window closed, `local` opened
  from Home's Recent Projects, Home through `marley home`); on restart that window's remote group
  comes back closed.
- Not driven: a host that answers without a logo (the copy dropped).

## Phase 4 — Complete
- CHANGELOG, `marley_workbench.md` (a remote project's mark), look-plan W1.2.
- Knowledge: L-claude-860 (a closed remote group in a run).
- Brain: decided on consultation 168e2606993146e49841a742afc865e9.
