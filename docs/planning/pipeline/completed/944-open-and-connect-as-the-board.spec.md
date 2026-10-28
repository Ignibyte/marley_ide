---
pipeline_id: 3861f6e7-d93c-4e96-ab34-6b151b789463
ticket: docs/planning/tickets/open/TICKET-944-open-and-connect-as-the-board.md
status: Phase 4 — Complete PASS
title: Remote folders, layouts and About as the board draws them
type: feature
slice: The Marley look, design review 2026-10-23 (docs/marley/design-review-2026-10-23/README.md)
references: [docs/marley/design-review-2026-10-23/G-system.md, docs/marley/look-plan.md]
---

## Title
The OpenConnect board's three dialogs: the remote-folder dialog takes an `ssh://` address, shows a
status dot per host and names the upload with its size; the layout setting is two picture cards;
About shows the build, how Marley updates, Copy the command and Open the log.

## Scope
### In
- `crates/recent_projects/src/remote_servers.rs` (Zed crate): `MarleyRemoteLook`, a global. While
  it is set, the default picker:
  - reads "ssh://user@host/path" as its placeholder;
  - when the query is an `ssh://` address (`ssh://[user@]host[:port]/path`, or the SCP form
    `ssh://user@host:/path`), lists first a row "Open <path> on <host>", whose confirm connects
    and opens the folder as a saved server's project does (`open_remote_project`);
  - starts each server header with a dot: the success colour while the host has a live
    connection (`remote::has_active_connection`), muted otherwise;
  - names the add row "Add a host…".
- `crates/remote/src/transport/ssh.rs` (Zed crate): on the `dev` channel with a commit (#445's
  path), `upload_local_server_binary`'s status reads "Uploading Marley's server for this build ·
  N MB", N the staged file's size in whole megabytes.
- `crates/settings_ui/src/marley_page.rs` (Marley-owned file in a Zed crate) and
  `crates/settings_ui/src/settings_ui.rs` (Zed crate): `MarleyLayout`'s renderer becomes
  `marley_page::render_layout_cards`: two cards side by side, each a small diagram (Marley: a rail
  and tabs; Zed: docks with a bottom panel), the layout's name and one line. The chosen card is
  outlined in the accent; a click on the other writes `marley.layout`.
- `crates/zed/src/zed.rs` (Zed crate): on Marley's channel (`is_marley`), About shows the app's
  cat mark, "Marley", a mono line "dev · <short commit> · built <when>", the cat line (#829,
  #943), the line "Marley updates through its install script, never from zed.dev. Pull, then run
  just install.", and two buttons: Copy the command (copies `git pull && just install`) and Open
  the log (`workspace::OpenLog`). Escape closes it. Zed's About stays on the other channels.
- `crates/marley_workbench/src/search_looks.rs` (Marley crate): sets `MarleyRemoteLook` with the
  other looks.

### Out (explicitly deferred)
- The upload's running count ("18 of 31 MB"): `scp` and `sftp` give Zed's transport no progress.
  A follow-up ticket: poll the remote file's size while the upload runs, or upload through a
  reader that counts.
- A dot for a connection that is being made: the connection pool's `Connecting` entry is private
  to `remote`; the status line already says what is happening. Part of the same follow-up.
- The OpenConnect board's other two cards (the page picker's star and slug, the harness seat's
  footnote): small look gaps for a follow-up ticket.
- The first-run screen's layout step: #912 ("Welcome to Marley", build order item 4); it can reuse
  this ticket's cards.
- The install steps and the newer-build notice: #945.

## Reference (§20)
Upstream Zed for remote development and About. Kept: `recent_projects::remote_servers`' modal,
its saved servers, its project rows and its add-server form; `remote`'s SSH transport and the
upload Marley stages (#445); `open_listener`'s `ssh://` URL forms; `settings_ui`'s renderer per
setting type; Zed's About window and its `menu::Cancel`. The layout cards are Marley-specific:
`marley.layout` is Marley's setting (#460).

### Prior art
- `crates/recent_projects/src/remote_servers.rs:843` `RemoteServerPickerDelegate`; `:1096` its
  placeholder "Search remote projects…"; `:993` `render_server_header` (name and nickname, no
  state); `:1252` `render_match`, whose `AddServer` row reads "Connect SSH Server". No Marley hunk
  in this file yet; `recent_projects.rs:75` `MarleyRecentLook` (#854) is the pattern.
- `crates/recent_projects/src/remote_connections.rs:128` `open_remote_project`.
- `crates/remote/src/remote_client.rs:403` `has_active_connection(opts, cx)`, public: whether the
  pool holds a live connection for a host. `ConnectionPoolEntry::Connecting` (`:1248`) is private.
- `crates/remote/src/transport/ssh.rs:1262` `upload_local_server_binary`: the size is read
  (`:1286`) and logged, the status set to "Uploading remote development server" (`:1291`); the
  file already carries #445's three `dev` hunks.
- `crates/zed/src/zed/open_listener.rs:344` `parse_ssh_url`: the `ssh://` URL and SCP forms.
- `crates/settings_ui/src/settings_ui.rs:564` `.add_basic_renderer::<settings::MarleyLayout>(render_dropdown)`;
  `:5262` `render_dropdown`, whose Marley branch draws segmented buttons (#844);
  `crates/settings_ui/src/marley_page.rs:59` the Layout row.
- `crates/marley_workbench/src/marley_workbench.rs:1234` `UseMarleyLayout` and `UseZedLayout`.
- `crates/zed/src/zed.rs:1533` `open_about_window`: `AboutWindow` with OK and Copy (the version
  details), the app icon (#829's cat on `dev`), `is_marley`; `:228` the app-level `OpenLog`
  handler, reachable from the About window, which has no workspace.
- AD-claude-763: Marley has no update source; Check for Updates says so. About's update line says
  the same.
- `script/e2e/445-marley-release-identity.sh`: two fake SSH hosts on loopback (sshd with a
  ForceCommand, `ssh`, `scp` and `sftp` wrappers first on Marley's PATH, keys made at run time),
  and `E2E_REMOTE_SERVER`.
- No crate we build has an upload with progress over `scp` or `sftp`.

## UI proof
Scenario `script/e2e/944-open-and-connect-as-the-board.sh` (sway). Setup: #445's two fake hosts
and its wrappers, the `sftp` wrapper waiting five seconds before it copies so the upload line
stays up for a shot; `E2E_REMOTE_SERVER` staged; host one saved in `ssh_connections`; a scratch
repository.
- `944-01-remote`: `projects: open remote`: the address placeholder, host one with its dot, "Add a
  host…" (REQ-001, REQ-003).
- `944-02-address`: `ssh://` and host two's address typed: the "Open … on …" row (REQ-002).
- `944-03-upload`: Enter: the connection's status, "Uploading Marley's server for this build ·
  N MB" (REQ-004).
- `944-04-connected`: the dialog opened again after host two connected: its dot in the success
  colour (REQ-003).
- `944-05-layouts`: Settings, the Marley page: the two picture cards, Marley outlined (REQ-005).
- `944-06-about`: `zed: about`: the mark, "Marley", the build line, the cat line, the update line
  and the two buttons (REQ-006).
- `944-07-log`: Open the log clicked: Marley's log in a tab (REQ-008). Copy the command is
  checked from the run's log: the steps read the clipboard with `wl-paste` (REQ-007).
- `944-08-day`: Marley Day: About and the layout cards (REQ-005, REQ-006).

## Locked-In Decisions
- D1 — The address row is offered only for a query that parses as an `ssh://` address; any other
  query filters the saved servers as Zed's picker does.
- D2 — A host's dot has two states: connected (success colour) and not (muted). A "connecting"
  dot waits for the follow-up, since the pool keeps that state private.
- D3 — The upload line names the size, not a running count (see Out).
- D4 — "built <when>" is the running binary's modified time: "today HH:MM" for today, the date
  otherwise. Neither the build script nor the install writes a time; the binary's time is the
  build's for the debug build and the install's for the installed one, minutes apart.
- D5 — Copy the command copies `git pull && just install`, to run in Marley's checkout, as the
  update line says.
- D6 — The layout cards write the same setting as the dropdown did; `UseMarleyLayout` and
  `UseZedLayout` stay.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens the remote-folder dialog, its field shall read "ssh://user@host/path" and its add row "Add a host…". | `944-01-remote` |
| REQ-002 | WHEN the field holds an `ssh://` address, the dialog shall list first a row that opens that folder on that host. | `944-02-address`, `944-03-upload` |
| REQ-003 | The dialog shall start each saved host with a dot in the success colour while Marley holds a live connection to it, and muted otherwise. | `944-01-remote`, `944-04-connected` |
| REQ-004 | WHILE Marley uploads its server to a host, the connection's status shall read "Uploading Marley's server for this build · N MB". | `944-03-upload` |
| REQ-005 | The Settings window's Marley page shall show the Marley and Zed layouts as two picture cards, the chosen one outlined in the accent, in Marley Night and Marley Day. | `944-05-layouts`, `944-08-day` |
| REQ-006 | WHEN the user opens About in Marley, it shall show the mark, "Marley", "dev · <commit> · built <when>", the cat line, the update line, Copy the command and Open the log, in Marley Night and Marley Day. | `944-06-about`, `944-08-day` |
| REQ-007 | WHEN the user clicks Copy the command, Marley shall put `git pull && just install` on the clipboard. | the run's log (`wl-paste`) |
| REQ-008 | WHEN the user clicks Open the log, Marley shall open its log in a tab. | `944-07-log` |

## Phase Plan
- **P1 Plan** — this spec, and the design in the notes.
- **P2 Code** — the remote look, the upload line, the layout cards and About, with their ledger
  rows; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario for the change, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
