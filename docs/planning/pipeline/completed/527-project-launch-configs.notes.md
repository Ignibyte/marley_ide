# A project's launch configs — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-527-project-launch-configs.md
- **Pipeline spec:** 527-project-launch-configs.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-09-25, "yes please" to item 4 of the Warp once-over (Tab Configs), which
  the note pairs with Orca's repo-declared first tabs for new worktrees (report 05 §3 item 6). The
  brief for this draft: a file in the project declares what opens in one click (terminals with
  commands, Claude Code, a Browser tab on the dev server's URL, splits); a command runs only after
  the user approves its exact text by hash; it opens from the rail's + and, later, for each new
  worktree (#510); decide where the file lives and its format.
- **Classification / tier:** feature, M. Marley crates only (`marley_workbench`,
  `marley_browser`); no Zed path.
- **Split.** Slice 1 is this spec: the file, the approval, the rail's +. Slice 2 opens the configs
  marked `"new_worktree": true` in each worktree #510 creates, after the same approval; it belongs
  in #510's spec when #510 is promoted, or in its own ticket if #510 ships first.
- **Recall (§18.3):**
  - Ledger: nothing on `.zed/`, `tasks.json`, launch configs or approvals by hash.
  - L-claude-500-a-clicked-context-menu-starts-on-its-first-entry-001: a clicked + menu starts on
    its first entry, so one Down reaches the second; take the menu's shot after the moves.
  - L-claude-487-a-headless-seat-has-no-devices-until-a-client-adds-them-001: under sway the
    window fills the output, and a scenario reads its click targets from its first shot.
  - L-claude-439-a-popover-menu-takes-focus-only-on-a-platform-frame-001: a popover menu takes
    focus two frames after it opens; the e2e run has real frames, as #500's scenario showed.
  - Completed pipelines: #440 (`start_cli` writes only the program name, after the shell's ready
    handshake), #455 (a project opened for the first time gets one terminal; a config adds to
    it), #441 (tasks go to center terminals in the Marley layout), #500 (New Browser Tab in the +;
    its scenario found the + at 236, 96 on a 1600 by 1000 output).
  - Brain: not consulted in this drafting pass, which was read-only; `/pipeline:plan` runs
    `brain_ask` at promotion.
- **Discovery (opened and checked):**
  - `crates/marley_workbench/src/rail.rs`: `render_project_menu` (line 1113) builds the + menu in
    a `ContextMenu::build` closure (New Terminal, New Browser Tab, New Agent Thread, then
    `agent_cli_entries` at 1206, which adds the "Agent CLIs" header); `new_terminal` (582),
    `new_browser_tab` (607), `new_agent` (619), `activate_workspace` (512); `GroupEntry` (148)
    holds the group's `ProjectGroupKey` (the main checkout's path) and its workspace;
    `changes_the_folders` (1536) is the project-event filter the rail already uses.
  - `crates/marley_workbench/src/agents.rs`: `TerminalFactory` (37), `Launcher` (50, its default
    `Project::create_terminal_shell` at 61), `start_cli` (188 to 221): the terminal from the
    factory in the workspace's default directory, `start_init_command_startup_handshake`, a
    5-second `STARTUP_TIMEOUT` (45), then `write_init_command_after_startup` with
    `marley_agent::launch_input`.
  - `crates/marley_agent/src/marley_agent.rs`: `agent_kind_of` (73) and `send_payload` (84, a line
    and a carriage return).
  - `crates/terminal_view/src/terminal_panel.rs`: `add_center_terminal` (835) adds the view to the
    active pane and leaves the focus alone while a modal is open (868).
  - `crates/terminal_view/src/terminal_view.rs`: `set_custom_title` (448).
  - `crates/workspace/src/workspace.rs`: `add_item_to_active_pane` (4956), `split_item` (4997),
    `split_pane` (6222, returns the new pane); `crates/workspace/src/pane_group.rs:1048`,
    `SplitDirection`.
  - `crates/marley_workbench/src/browser.rs`: `new_tab` (5744) opens `BLANK` through
    `open_page_in` with the focus in the address bar; `BrowserHub::create_page_task` (809) and
    `navigate` (843).
  - `crates/marley_browser/src/address.rs`: `agent_url` (29), `http` and `https` only; the private
    `host_kind` (65) knows loopback hosts. `browser.rs` keeps `BLANK` at 108 and `open_page_in`
    at 2960.
  - Zed's tasks: `crates/task/src/task_template.rs` (`TaskTemplate` at 24, `hooks` at 80,
    `TaskHook::CreateWorktree` at 95, the schema's `DefaultDenyUnknownFields` at 148);
    `crates/workspace/src/tasks.rs:235`, `run_create_worktree_tasks`;
    `crates/project/src/task_inventory.rs:721`, `templates_with_hooks`.
  - Zed's project files: `crates/project/src/project_settings.rs`, `update_settings` (1339):
    `.zed/settings.json` applies only to a trusted worktree (1360 to 1382), `tasks.json` loads
    either way (1385); `crates/settings/src/settings_store.rs:1111` parses a project file as
    `ProjectSettingsContent` (`crates/settings_content/src/project.rs:44`), which has no `marley`
    key; `crates/settings_content/src/settings_content.rs:251`, the user-level `marley` block;
    `crates/paths/src/paths.rs` (`.zed` at 488, `.zed/tasks.json` at 505, `.vscode/launch.json`
    read as debug scenarios at 535).
  - Parsing, events, storage, prompt: `crates/settings_json/src/settings_json.rs:753`,
    `parse_json_with_comments`, re-exported by `settings`; `crates/project/src/project.rs:374`,
    `Event::WorktreeUpdatedEntries`; `crates/db/src/kvp.rs` (`KeyValueStore` at 12, `scoped` at
    89); `crates/gpui/src/window.rs:6448`, `Window::prompt`; `crates/ui_prompt/src/ui_prompt.rs:22`
    (gpui draws prompts on Linux); `crates/ui/src/components/context_menu.rs` (`header` 450,
    `separator` 469, `entry` 488, `ContextMenuEntry::disabled` 186).
  - Dependencies: `indexmap` with `serde` and `sha2` in the workspace (`Cargo.toml` 687, 858);
    `collections::IndexMap` (`crates/collections/src/collections.rs:3`); `sha2` already in
    `marley_browser`.
  - Orca: `src/shared/orca-yaml-hook-types.ts` (`OrcaDefaultTabTemplate`: `title`, `color`,
    `command`; `PersistedTrustedOrcaHookEntry`: `contentHash`, `approvedAt`);
    `src/renderer/src/lib/worktree-default-terminal-tabs.ts` (`applyDefaultTerminalTabs`, once per
    worktree); `src/renderer/src/lib/ensure-hooks-confirmed.ts` (the trust text of setup plus each
    tab's command, the hash compared with the repository's, prompts serialized in one queue, a
    dismissed prompt counted as "skip"); `src/renderer/src/lib/orca-hook-trust.ts` (SHA-256 of the
    trimmed text).
- **Decisions:** D1 to D6 in the spec.

### Changed at promotion (2026-09-29; each item overrides the design below)
- **Checklist** (no task tool): pre-flight ✓ (no other active pipeline, cargo idle); recall ✓;
  the brain ✓ (nothing on this seam); promoted ✓; the seams re-verified by an Explore agent at
  73fd28a74d ✓ (every line the notes cite has moved; #510, #532, #585, #503 and #521 landed).
- **The menu:** the Launch header goes after New Agent in Worktree (#510 landed); entries read a
  cache filled asynchronously, since the menu builder is synchronous and `Fs::load` is not.
- **The cache** (`launch.rs`, a global by project root): filled when a workspace opens and when
  `project::Event::WorktreeUpdatedEntries` names `.zed/marley.json`, read with `Fs` and
  `settings::parse_json_with_comments` into an `IndexMap` (the file's order).
- **Terminals and agents:** `agents::start_cli_with_prompt` is split so a shared
  `start_in_terminal(workspace, directory, input, ..)` opens a center terminal in a given folder
  and types the input after the handshake; a terminal item types `send_payload(command)`, an agent
  item the launch line as before. The title goes through `TerminalView::set_custom_title`.
- **Browser items:** `browser::open_url_tab` (#503) returns the tab it opened or brought forward;
  a loopback URL (`marley_browser::address::local_url`) waits for its port through a new
  `marley_browser::address::wait_for_port` (a TCP connect every half second, 30 seconds at most).
- **Splits:** each item opens in the active pane, then, with `split`, moves into a new pane split
  off the previous item's pane (`Workspace::split_pane`, `workspace::move_item`), since the
  active pane follows a focus event that may not have landed between items.
- **The approval** goes in Zed's key-value store (`KeyValueStore::global(cx).scoped`), the first
  use in a Marley crate; `sha2` joins `marley_workbench`'s dependencies.

### Design
- **`launch.rs`, a new module of `marley_workbench`.**
  - The file's types, serde with `deny_unknown_fields`: `LaunchFile { launch:
    IndexMap<String, RawConfig> }`, `RawConfig { items: Vec<RawItem> }`, and `RawItem` with the
    optional `terminal`, `agent`, `browser`, `title`, `cwd`, `split`, `focus`. Validation turns
    them into `Config { items: Vec<Item> }` with `Item { kind: Terminal(String) | Agent(AgentKind)
    | Browser(String), title, cwd, split: Option<Right | Down>, focus }`: one kind per item, the
    agent's name through `marley_agent::agent_kind_of`, the URL through
    `marley_browser::address::agent_url`, no `title` or `cwd` on a Browser item, at most one
    `focus`. The first error becomes one line of text for the menu.
  - Pure functions, gpui-free: `parse(text) -> Result<Vec<(String, Config)>, String>`;
    `approval_text(name, &Config) -> String`, one line per item in a fixed form
    (`terminal "dev server" in web: npm run dev`, `agent claude, split right`,
    `browser http://127.0.0.1:8000/, split down`); `approval_hash(&str) -> String`, SHA-256 in
    hex.
  - The cache: a global from a project root to its parsed file (or its error), loaded with
    `Fs::load` when a workspace's project adds a worktree and again when
    `project::Event::WorktreeUpdatedEntries` names `.zed/marley.json`; a removed file drops the
    entry. `launch::init` subscribes through `cx.observe_new::<Workspace>`, so the cache does not
    depend on the rail being open.
  - The approval: `KeyValueStore::global(cx).scoped("marley-launch")`, keyed by the main
    checkout's path and the config's name (a newline between), holding the approved hash. The
    store is read and written off the foreground.
  - The runner: `open(name, config, workspace, window, cx)` asks when the hash differs (the
    prompt's detail is the approval text; the message says whether the text changed since an
    approval), then opens the items one after the other: a `split` item first splits the
    previous item's pane with `Workspace::split_pane`, which makes the new pane active; a
    terminal item opens through the launcher's factory with its `cwd`, gets its `title`, waits
    for the shell's handshake and types its command with `send_payload`, as `start_cli` does
    (the shared part moves into one helper in `agents.rs`, and `start_cli` keeps its behavior);
    an agent item goes through that same helper with the agent's launch input; a Browser item
    goes through `browser::new_tab_at`. At the end the `focus` item, or the first, takes the
    focus. When #537 has landed, the helper hands its credential-prompt environment to agent
    items and to terminal items whose command's program `agent_kind_of` recognizes.
- **`browser.rs`:** `new_tab_at(workspace, url, window, cx)`, `new_tab` with the URL in place of
  `BLANK` and the focus left to the caller. For a loopback URL the page opens once
  `marley_browser`'s port wait returns.
- **`marley_browser`:** the port wait, a TCP connect to the URL's host and port every 250 ms for
  at most 30 seconds, `localhost` tried on both 127.0.0.1 and `::1`. It sits in the crate's
  adapter code beside the CDP socket, not in the workbench (§14).
- **`rail.rs`:** after `agent_cli_entries`, `launch_entries` adds a separator, the "Launch" header
  and an entry per config for the group's root, or the one disabled error entry; an entry's
  handler shows the project (`activate_workspace`) and calls `launch::open`.
- **File manifest.** All Marley-owned: `crates/marley_workbench/src/launch.rs` (new),
  `marley_workbench.rs` (`pub mod launch`, `launch::init`), `rail.rs`, `agents.rs`, `browser.rs`,
  `crates/marley_workbench/Cargo.toml` (`sha2`, `collections`), `crates/marley_browser/src/`
  (the port wait), and `script/e2e/527-project-launch-configs.sh` at Test. No Zed path, so no row
  in `docs/marley/zed-touchpoints.md`.
- **Ledger rows at Complete.** An AD for the file's place and format (D1) and one for the approval
  (D2: what is hashed, the key, the refusal on dismissal).

### For the quality pass
- No tests (§7, since 2026-09-29): the drafted scenario waits for the quality pass.

### Risks
- Sibling tickets drafted the same night share seams (their queued specs, 2026-09-25): #510 adds
  New Agent in Worktree after the Agent CLIs, so the Launch section goes after it; #503 and #521
  both plan the same "new Browser tab at a URL" function as `browser::new_tab_at`, so whichever
  lands first owns it; #521's `marley_ports` will know listening ports, and the Browser item's
  wait could ask it, but a TCP connect needs no scan and stays; #532 gives agents a permission
  mode that the helper shared with `start_cli` applies to agent items too.
- The + menu reads the cache when it opens. A menu opened within moments of a project opening
  may show no configs until the load finishes; the next open shows them.
- `add_center_terminal` adds its view to whichever pane is active when its task completes, so two
  items in flight at once would land in one pane. The runner waits for each item (D4).
- A terminal item types its command after the shell's handshake or after 5 seconds, as
  `start_cli` does; a slow `.bashrc` delays it, and a command typed early would sit in the tty's
  buffer until the shell reads it.
- Running a config twice runs its commands twice, and a second dev server fails on its port. That
  is what the user asked for; the scenario's second run uses "Shell" so the shot stays readable.
- The approval key includes the main checkout's path, so a moved repository asks again.
- A server bound only to `::1` behind a `localhost` URL: the port wait tries both addresses.

## Phase 2 — Code
- **Checklist** (no task tool): `launch.rs` ✓; `agents::start_in_terminal`, `launch_mode` and
  `launch_input` ✓; `browser::open_url_tab` returning its tab ✓; `marley_browser::address::accepts`
  ✓; the rail's `launch_entries` and `Rail::launch` ✓; `sha2` in the manifest ✓; the review ✓; the
  gate ✓.
- **Built as the changes at promotion say.** Clippy asked for `start_cli_with_prompt` to take
  `&str` and `Option<&str>` (its two callers changed), `Arc::clone`, the approval text built from
  parts rather than pushed `format!`s, and `&App`/`&Window` where nothing mutates.
- **The review**, against each criterion:
  - REQ-001, REQ-008, REQ-009: the menu reads the cache by the group workspace's first folder; no
    file, no header; a broken file, one disabled entry.
  - REQ-002, REQ-005, REQ-006, REQ-007: the prompt shows the hashed text; Run writes the hash
    before anything opens; Cancel, Escape or a dismissed prompt return before any write.
  - REQ-003: items open one after another, each awaited; a terminal's command is typed after the
    handshake; a split item moves into a pane split off the previous item's pane.
  - REQ-004: a loopback URL tries a connect every half second for 30 seconds, then loads anyway.
  - A `cwd` with `..` or an absolute path is refused at parse time; an unknown key is refused by
    `deny_unknown_fields`.
- **The gate:** `just gate-diff` green: 16 passed, 0 failed, `GATE GREEN [diff]`, the receipt
  written.

---
## Phase 3 — Complete
- **Checklist** (no task tool): document ✓; capture knowledge ✓; close the ticket ✓; archive ✓;
  commit ✓.
- **Documented:** `CHANGELOG.md`; `docs/marley_architecture/marley_workbench.md` ("A project's
  launch configs"); `docs/marley_architecture/marley_browser.md` (`address::accepts`);
  `docs/marley/workbench-shell.md`; `docs/marley/guide.md` ("Launch configs"). No Zed path changed.
- **Knowledge:** AD-claude-527-launch-configs-live-in-zed-marley-json-and-run-after-their-text-is-approved-001.
- **Brain:** consultation 9a07053d30f340089c0f97e23a5bd445 closed with a decision (follow-up
  2026-10-29).
- **Closed:** TICKET-527 moved to `tickets/closed/`; its BACKLOG row went at promotion.
- **No tests** (§7): the drafted scenario waits for the quality pass.
