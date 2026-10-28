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
- **Built:**
  - `script/install-marley`: `write_install_record` (python3 writes `install.json` to a temporary
    name and renames it), timed builds, called at the end of the Linux install (toolchain, build,
    remote server, desktop entry) and the macOS one (toolchain, build, app). `--remote-server-only`
    exits before it, so it writes none (REQ-007).
  - `system_pages.rs`: `InstallRecord`, `InstallStep`, `InstallState`; `read_install_in` on the
    background executor when the tab opens; `install_rows` with the stale-build warning; `mcp_rows`
    with Endpoint, Bridge, Tools, Last refusal, Limits; `chromium_rows` with the version, pages and
    cookie sites, and the limits; `read_cookie_sites` when the Chromium page is chosen; the buttons.
  - `browser.rs` (`product_of`, `cookie_sites_of`), `marley_browser/src/page.rs`
    (`Identity::product`), `marley_mcp/src/transport.rs` (`MAX_BODY_BYTES` public), `mcp.rs`
    (`GRANTED_WRITE_CLASSES`), `agent_activity.rs` (`last_refusal_today`).
- **Deviations:**
  - Keys: `ended` and each step's `value`, not `finished_at` and `detail`, to match the rows' words.
  - The record is written by python3 rather than `printf`, so a path is escaped by `json.dump`.
  - `marley_mcp::served_tool_count` already existed; no `granted_write_classes(cx)` function: the
    grants are a constant list, read by its length.
  - `clients.rs` unchanged: the bridge row reads the path the page already had.
- **Review:**
  - The first gate failed on dylint `blocking_io_on_foreground`: `install_rows` canonicalized the
    running and installed binaries while drawing. The read and both canonicalizations moved into
    `read_install_in`, run on the background executor; the page keeps `InstallState`.
  - The Chromium row read "1 pages · cookies from 1 sites"; singular words for one.
  - The macOS call named `$app`, local to `install_mac` and empty after it returns; it names
    `$app_dir/Marley.app` now. `git status --porcelain | head -c 1` could fail under `pipefail`
    when git is cut off; the whole status is passed instead.
  - Clippy: `OpenSettingsAt` needed `target: None`; unused qualifications; `map_or_else`;
    `needless_pass_by_ref_mut` on `read_cookie_sites`.

- **Gate:** `just gate-diff` green, 17 passed (after the dylint fix). The installer's later fixes are
  outside the receipt; `shellcheck -S info script/install-marley` passes, and
  `write_install_record` run alone under `set -euo pipefail` wrote the record shown below.

## Phase 3 — Test
- **Scenario:** `script/e2e/945-system-pages-as-the-board.sh` under sway: a scratch repository, a
  loopback page that sets one cookie, `install.json` in the run's data folder naming the debug
  binary and commit `0000000…945`.
- **Shots:**
  - `945-01-install` (REQ-001, REQ-002): Installed "commit 0000000 · 2026-10-28T12:00:00-0500";
    toolchain, build "412 s", remote server "linux-x86_64", desktop entry, each "ok" with a green
    dot; "Running Marley · still on c210142 · restart" in the warning color; Open the log.
  - `945-02-mcp` (REQ-003, REQ-004): Endpoint "http://127.0.0.1:…/mcp · loopback only · up", Bridge
    with its path and "stdio", Tools "68 tools · 10 write classes granted", Last refusal "none
    today", Limits "request bodies ≤ 1 MiB · loopback only", Paired programs; Open Agent Activity
    and Copy the endpoint.
  - `945-03-activity` (REQ-004): the button opened the Agent Activity tab.
  - `945-04-chromium` (REQ-005): Chromium "Chrome 154 · running", repo's profile path with "1 page ·
    cookies from 1 site", Limits.
  - `945-05a-shell`, `945-05b-fleet`, `945-05c-developers` (REQ-006): Open the guide; Add a provider
    and Open Settings; Open the plan and Open the guide.
  - `945-06-day` (REQ-001): Install and update in Marley Day, dots and warning legible.
- **Not seen:** the plan's refusal before the MCP shot was not driven, so Last refusal is shown only
  in its empty state; the filled row's words come from `last_refusal_today` and were checked by
  reading the code. REQ-007's `--remote-server-only` path exits before the record (read in the diff).
- **Fixes from Test:** the first run showed "1 pages · cookies from 1 sites"; singular words added,
  gate re-run, scenario re-run.
- Nothing pre-existing failed.

## Phase 4 — Complete
- Documented: `CHANGELOG.md` (Added), `docs/marley/look-plan.md` (SystemPages row),
  `docs/marley_architecture/marley_workbench.md` (the System tab). No Zed path, so no ledger row.
- Knowledge: L-claude-945-a-row-that-compares-files-reads-them-when-the-page-opens-001,
  F-claude-945-the-macos-install-record-named-a-local-of-install-mac-001.
- Brain: consultation 557f3e8cf0a147fdae37c12dfa11fc46 closed with no decision (the spec's
  decisions follow the existing page shape).
- Ticket closed; pipeline archived.
