# The system pages as the board draws them — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-945-system-pages-as-the-board.md
- **Pipeline spec:** 945-system-pages-as-the-board.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-23: "we need to spec out the entire tickets then to fix this" — the
  design review's build order (item 11: SystemPages).
- **Classification / tier:** feature, M. Marley crates and Marley's install script; no Zed hunk.
- **Recall (§18.3):**
  - AD-claude-502-the-installed-marley-is-a-copy-and-a-launcher-001: the install replaces
    `~/.local/lib/marley/marley` by a rename, so the running process keeps its old file; D2 reads
    the running path with Linux's " (deleted)" suffix taken off.
  - L-claude-517-a-running-bash-script-reads-its-file-as-it-goes-001: never edit
    `script/install-marley` while an install runs; check by the process's own name, not
    `pgrep -f` in a one-shot shell.
  - L-claude-837-the-install-builds-the-working-tree-001: no install while this ticket's edits are
    half done.
  - F-claude-809-an-install-pruned-the-running-marleys-remote-server-001: the record is written
    after the remote server step and changes nothing the prune reads.
  - AD-claude-703-agent-control-starts-with-a-log-and-a-kill-switch-001 and L-claude-871: the
    scenario's refusal comes from the kill switch, its client run in Marley's terminal.
  - AD-claude-581-clearing-a-projects-browser-data-deletes-its-profile-001: the clear stays in the
    rail; the page only reads.
  - F-claude-903-a-stand-in-not-on-the-terminals-path-001: the MCP client is put first on the
    terminals' PATH through the scenario's `.bashrc`.
  - §14 file IO: the record is read through `read_install_in(dir)`, so the scenario's data folder
    is the one read.
  - The brain: to be consulted when the spec is promoted.
- **Discovery:** `script/install-marley` (the Linux tail, the macOS app writer), `system_pages.rs`
  (`install_rows`, `mcp_rows`, `chromium_rows`, `render_buttons`), `browser.rs` (`ProjectBrowser`,
  `identity`), `marley_browser/src/page.rs` (`Identity`), `mcp.rs` (grants, bridge),
  `marley_mcp` (`transport::MAX_BODY_BYTES`, `registry::tools_list`), `agent_activity.rs`,
  `clients.rs`, `guide.rs`.
- **Decisions:** D1–D6 in the spec.

### Design
- **The record:** `install-marley` keeps a step list in shell variables as it goes (the toolchain
  line from `rustc --version`, `SECONDS` round the build, the server's platform, the desktop entry
  or the app), then writes JSON with `printf` to `install.json.$$` and renames it. Keys:
  `commit`, `dirty`, `finished_at` (ISO 8601), `binary`, `steps: [{name, detail, result}]`.
- **Reading it:** in `system_pages.rs`, `#[derive(Deserialize)] struct InstallRecord { commit,
  dirty, finished_at, binary, steps: Vec<InstallStep> }` and `InstallStep { name, detail, result }`;
  `fn read_install_in(data_dir: &Path) -> Option<InstallRecord>` (unreadable is `None`, logged).
  Read off the main thread when the page opens, as Troubleshooting's checks are, and kept on
  `SystemPages`.
- **Stale build:** `fn running_is_stale(record: &InstallRecord, cx: &App) -> bool`: the
  current exe's path, " (deleted)" trimmed, equals `record.binary`, and `AppCommitSha` differs
  from `record.commit`.
- **MCP rows:** `clients::bridge_path()`, `mcp::served_tool_count(cx)` (the length of
  `tools_list`'s `tools`), `mcp::granted_write_classes(cx)` (the shared `ServerData`'s grants),
  `agent_activity::last_refusal(cx) -> Option<ActivityRow>`, `marley_mcp::MAX_BODY_BYTES`.
- **Chromium rows:** `browser::BrowserHub::version(cx) -> Option<SharedString>` from the first
  running project's `Identity::product()`; `browser::cookie_sites(project, cx) -> Task<Option<usize>>`
  sending `Storage.getCookies` on that project's connection and counting distinct domains,
  started when the page opens and kept on `SystemPages`.
- **Buttons:** `render_buttons` gains the arms for `Page::Shell`, `Page::Mcp`, `Page::Fleet` and
  `Page::Developers`, each through #870's `render_button`; Copy the endpoint writes the URL to the
  clipboard; Open the plan calls `cx.open_url` with the plan's GitHub URL.
- **File manifest:** `script/install-marley` (Marley-owned),
  `crates/marley_workbench/src/system_pages.rs`, `browser.rs`, `clients.rs`, `mcp.rs`,
  `agent_activity.rs` (Marley crate), `crates/marley_browser/src/page.rs` (Marley crate),
  `crates/marley_mcp/src/transport.rs`, `crates/marley_mcp/src/marley_mcp.rs` (Marley crate),
  `script/e2e/945-system-pages-as-the-board.sh` (Marley-owned).
- **Ledger rows:** none; no Zed path changes.

### Visual check plan
- REQ-001, REQ-002: setup writes `install.json` into the run's data folder with four steps, the
  debug binary's path and commit `0000000000`; open the System tab on Install and update;
  `945-01-install`.
- REQ-003, REQ-004: `marley: stop agent control`; the client in Marley's terminal calls
  `terminal_run` and is refused; open the MCP page; `945-02-mcp`; click Open Agent Activity;
  `945-03-activity`; `marley: resume agent control`.
- REQ-005: open the scenario's page in a Browser tab (it sets one cookie); the Chromium page;
  `945-04-chromium`.
- REQ-006: Shell integration, Fleet providers, For developers in turn; one shot of each page's
  foot, named `945-05a-shell`, `945-05b-fleet`, `945-05c-developers`.
- Day: `profile_setting theme '"Marley Day"'`, Install and update; `945-06-day`.
- REQ-007: the review of the diff reads the script's two paths; the format is the one the scenario
  writes.

### Risks
- An install that fails part way leaves the last good record; the record is written only at the
  end, so it never describes a half install.
- `Storage.getCookies` on a browser the user is using is a read; it runs once per page open, not
  in a loop.
- The script's JSON is written with `printf`; a path holding a quote is already refused by the
  script (`:130`, `:344`), so no escaping beyond that is needed.

## Phase 2 — Code

## Phase 3 — Test

## Phase 4 — Complete
