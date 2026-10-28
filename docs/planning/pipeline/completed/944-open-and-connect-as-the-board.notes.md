# Remote folders, layouts and About as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-944-open-and-connect-as-the-board.md
- **Pipeline spec:** 944-open-and-connect-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11: OpenConnect).
- **Classification / tier:** feature, M. Hunks in four Zed crates (`recent_projects`, `remote`,
  `settings_ui`, `zed`) and Marley's look in `search_looks.rs`.
- **Recall (§18.3):**
  - AD-claude-445-marleys-remote-server-is-staged-by-the-install-and-matched-by-commit-001: the
    upload is Marley's own staged server on `dev`; the new status line sits on that path only.
  - AD-claude-763-marley-has-no-update-source-and-check-for-updates-says-so-001: About's update
    line repeats it; no update check is added.
  - AD-claude-460-the-fork-starts-in-the-marley-layout-001 and
    AD-claude-451-zeds-layout-presets-explain-themselves-in-the-marley-layout-001: the cards write
    `marley.layout`; Zed's presets keep their toast.
  - L-claude-860-a-closed-remote-group-in-a-run-001: a remote project in a run needs its host up;
    the scenario keeps both fake hosts up to the end.
  - F-claude-809-an-install-pruned-the-running-marleys-remote-server-001: the scenario stages the
    server in the run's own profile, never the user's data folder.
  - PR-claude-880-a-run-names-its-own-copy-of-every-user-store-001: #445's wrappers read nothing of
    `~/.ssh`.
  - F-claude-865-a-hook-called-from-a-render-read-its-own-view-001: the picker's look functions
    take the host's options and read no view.
  - The brain: to be consulted when the spec is promoted.
- **Discovery:** `remote_servers.rs` (`RemoteServerPickerDelegate`: `placeholder_text`,
  `update_matches`, `render_match`, `render_server_header`, `confirm`), `remote_connections.rs`
  (`open_remote_project`), `remote/src/transport/ssh.rs` (`upload_local_server_binary`),
  `remote/src/remote_client.rs` (`has_active_connection`), `settings_ui.rs` (the renderer table),
  `marley_page.rs`, `zed.rs` (`open_about_window`), `search_looks.rs`.
- **Decisions:** D1–D6 in the spec.

### Design
- **Remote look:** `pub struct MarleyRemoteLook;` (a `Global`) in `remote_servers.rs`. A new
  `RemoteMatch::OpenAddress { connection: SshConnectionOptions, path: String }`, pushed first by
  `rebuild_matches` when the global is set and `parse_ssh_address(query)` answers. The parser is a
  small function beside the picker taking `ssh://` URLs and the SCP form, as `parse_ssh_url` in
  `open_listener` does; `recent_projects` has no `url` dependency, so it splits the string itself.
  `confirm` on that row calls `open_remote_project` with `RemoteConnectionOptions::Ssh` and the
  path. `render_server_header` puts a `size_2` dot first when the global is set, its colour from
  `has_active_connection`. The add row's label checks the global.
- **Upload line:** in `upload_local_server_binary`, when the release channel read through `cx` is
  `dev` (the channel #445's hunks test in `ensure_server_binary`), `format!("Uploading Marley's
  server for this build · {} MB", size.div_ceil(1_000_000))` goes to `set_status`; other channels
  keep Zed's string.
- **Layout cards:** `pub(crate) fn render_layout_cards(field, file, metadata, title, description,
  window, cx) -> AnyElement` in `marley_page.rs`, the same signature as `render_dropdown`;
  `settings_ui.rs` registers it for `MarleyLayout` in place of `render_dropdown`. Each card is a
  `v_flex` with a diagram of `div`s (a narrow rail beside a tab area; a left dock, a center and a
  bottom panel), a Label and a muted line; `MarleyLayoutCard { layout, name, line }` holds each.
  The click writes through `update_settings_file`, as the dropdown does.
- **About:** in `AboutWindow::render`, a Marley branch: the icon, `Headline::new("Marley")`, the
  mono build line (`AppCommitSha::short`; the time from `std::env::current_exe()`'s metadata,
  read once in `AboutWindow::new` and formatted with `chrono::Local`), the cat line, the update
  line, then the two buttons with Zed's `NavigableEntry` focus as OK and Copy have. Copy the
  command writes the string and closes the window; Open the log dispatches `workspace::OpenLog`
  and closes it.
- **File manifest:** `crates/recent_projects/src/remote_servers.rs` (Zed crate),
  `crates/remote/src/transport/ssh.rs` (Zed crate), `crates/settings_ui/src/settings_ui.rs` (Zed
  crate), `crates/settings_ui/src/marley_page.rs` (Marley-owned file in a Zed crate),
  `crates/zed/src/zed.rs` (Zed crate), `crates/marley_workbench/src/search_looks.rs` (Marley
  crate), `script/e2e/944-open-and-connect-as-the-board.sh` (Marley-owned).
- **Ledger rows (docs/marley/zed-touchpoints.md):** a new row for
  `crates/recent_projects/src/remote_servers.rs` (`MarleyRemoteLook`, the address row, the dots,
  the add row's name); extend `crates/remote/src/transport/ssh.rs` (the upload line on `dev`),
  `crates/settings_ui/src/settings_ui.rs` (`MarleyLayout`'s renderer), `crates/settings_ui/src/marley_page.rs`
  (`render_layout_cards`) and `crates/zed/src/zed.rs` (About's Marley branch).

### Visual check plan
- REQ-001, REQ-003: `projects: open remote` with host one saved; `944-01-remote`.
- REQ-002: type host two's `ssh://` address; `944-02-address`.
- REQ-004: Enter; the `sftp` wrapper's wait keeps the upload line up; `944-03-upload`.
- REQ-003 again: after host two opens, the dialog again; `944-04-connected`.
- REQ-005: `zed: open settings`, the Marley page; `944-05-layouts`.
- REQ-006: `zed: about`; `944-06-about`.
- REQ-007: click Copy the command; the steps run `wl-paste` on the sway's display and print it;
  the notes quote the line.
- REQ-008: `zed: about`, click Open the log; `944-07-log`.
- Day: `profile_setting theme '"Marley Day"'`, About and the Marley page; `944-08-day`.

### Risks
- The SSH upload may finish before the shot; the wrapper's wait is the fix. If Zed uses `scp`
  where `sftp` is missing, both wrappers wait.
- The address row must not swallow a query a user meant as a filter: only `ssh://` triggers it.
- About's window is 440 by 300; the Marley branch drops the Commit and Version labels (the build
  line carries both), so the extra lines fit.

## Phase 2 — Code
- **Built:** as the design names, with the ledger rows written first. `recent_projects` re-exports
  `MarleyRemoteLook`; the delegate keeps `marley` from when it was built, since `rebuild_matches`
  has no `cx`. `render_layout_cards` reads the store with `cx.global::<SettingsStore>()`.
- **Gate:** GREEN, and again on the final tree.

## Phase 3 — Test
- **Scenario:** `script/e2e/944-open-and-connect-as-the-board.sh` (sway).
  - It takes #445's `fake_host`, client wrappers and teardown, with `sftp` waiting five seconds.
  - Both hosts are saved with `$USER`, and the staged server for HEAD is copied into the run's
    profile.
- **Red, the code's, fixed:** host two was connected but its dot was muted. The connection pool's
  key is the whole `SshConnectionOptions`, nickname included, and the typed address had none
  (F-claude-944-the-connection-pool-keys-on-the-whole-ssh-options-001). The address row now opens
  through a matching saved host.
- **Red, the code's, fixed:** the 200 px layout cards squeezed the row's title and description to a
  column one word wide. The cards are now 132 px with shorter lines.
- **Red, the scenario's own, fixed:** the Day layout shot came after three windows tiled, leaving
  Settings about 370 px wide. It is now taken beside the Night one.
- **Shots:**
  - `944-01-remote`: the placeholder, "Add a host…", host-one and host-two with muted dots.
  - `944-02-address`: "Open …/two/home/project on 127.0.0.1" first.
  - `944-03-upload`: "Uploading Marley's server for this build · 40 MB".
  - `944-04-connected`: host-two's dot green.
  - `944-05-layouts`, `944-08-day-layouts`: the two cards, Marley's outlined in the accent.
  - `944-06-about`, `944-08-day`: the mark, "Marley", "dev · f82596b · built 2026-10-28 13:57", the
    cat line, the update line, Copy the command and Open the log.
  - The check "Copy the command copied it" passed (`git pull && just install`).
  - `944-07-log`: Marley's log in a tab.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`, `docs/marley/look-plan.md`, the architecture note, and the ledger
  rows.
- **Knowledge:** F-claude-944-the-connection-pool-keys-on-the-whole-ssh-options-001.
- **Brain:** consultation 18255aaa1c9249b1acedeeb1750fccf0 closed with `brain no-decision`.
- **Ticket:** closed.
