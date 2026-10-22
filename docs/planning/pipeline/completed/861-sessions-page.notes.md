# The Sessions page — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-861-a-sessions-page.md
- **Pipeline spec:** 861-sessions-page.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next ready row (#890 and #895 wait on Chad, #856 on a language server in
  the run, #891 and #893 are larger).
- **Classification / tier:** feature, M. Marley crate only, one new file.
- **Recall (§18.3):** #832's Sessions view; #880 gives each run its own tmux server.
- **Decisions:** D1, D2 in the spec.

### Design
- `sessions_page.rs`: `OpenSessionsPage`; `SessionsPage` observes `TmuxHosts`; `render_header`
  (counts, New Session through `new_session_in` in the first visible worktree), `render_table`
  (a row per session of each listed host; Attach through `attach`, Close… through a prompt then
  `kill`), the placing line, `render_hosts` (`host_choices`, `error_words`).
- **File manifest:** `sessions_page.rs` (new), `marley_workbench.rs`, the scenario.

## Phase 2 — Code
- Built as designed. Added after the first green: Open as a Page under the rail's Sessions view
  (`rail_sessions.rs`), since the ticket opens the page from there (REQ-003).
- Review: Close prompts before `kill` (D1); New Session reports a window with no project folder
  through `tmux_sessions::report`; the page holds only a weak workspace handle.
- Gate: `just gate-diff` 17/17 green before the link and again after it.

## Phase 3 — Test
Scenario `script/e2e/861-sessions-page.sh` (sway, the run's own tmux server, `build` and `notes`
detached).
- `861-00-rail`: the rail's Sessions view, both sessions under This machine, HOSTS TO LOOK ON,
  Open as a Page under it (REQ-003).
- `861-01-page`, from that link: "Sessions 2 on 1 host · 0 attached · 2 detached", New Session,
  the rows `build` and `notes` (this machine, 1 window, the folder, detached, Attach, Close…), the
  placing line, the hosts with This machine looked on (REQ-001).
- `861-02-closed`: Close… on `notes` and Return: "1 on 1 host", only `build`; `tmux list-sessions`
  lists only `build` (REQ-002).
- `861-03-day`: the same page in Marley Day (REQ-001).
- The hosts list in the shots names the ssh hosts the run's settings carry; shots stay out of the
  repository.
- Not shot: Attach and New Session, which open the terminal tabs #832 already shows.

## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md` (The Sessions page), look plan W1.3 already names #861.
- Brain decision recorded on consultation 4e3fb05a. No F/L/PR codes: nothing broke.
