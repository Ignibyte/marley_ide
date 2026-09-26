---
pipeline_id: a3881d77-4bc1-41af-bc71-dda946bdd0ed
ticket: docs/planning/tickets/open/TICKET-527-project-launch-configs.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "A project's launch configs: a file names what opens in one click from the rail's +"
type: feature
slice: workbench shell (the rail's +), Warp once-over item 4; worktrees (#510) in slice 2
references: [docs/planning/pipeline/completed/500-browser-from-the-rail.spec.md, docs/planning/pipeline/completed/440-rail-agent-clis.spec.md, docs/planning/design-notes/warp-once-over-2026-09-25.md]
---

## Title
A project names its launch configs in `.zed/marley.json`. A config is a list of items: a
terminal that runs a command once its shell is ready, an agent CLI started the way the rail's
Agent CLIs entries start one, or a Browser tab on a URL. Each item opens as a tab in the pane the
item before it used, or in a new pane split to the right of that pane or below it. The project's
+ in the rail lists the configs by name. Choosing one shows its exact text for approval the first
time and whenever the text changes, then opens every item.

## Scope
### In
- **The file.** `<project root>/.zed/marley.json`, JSON with comments and trailing commas (Zed's
  parser for its own `.zed/` files), holding a `launch` object from config names to configs. A
  config is `{"items": [...]}`. Each item has exactly one of:
  - `"terminal": "<command>"`: a new center terminal whose shell gets the command typed once the
    shell says it is ready, as `agents::start_cli` types an agent's name. An empty string opens a
    plain shell.
  - `"agent": "claude" | "codex" | "gemini" | "opencode"`: that CLI, started through
    `agents::start_cli` as the rail's Agent CLIs entries start it (and so with #537's
    environment once #537 lands).
  - `"browser": "<URL>"`: a Browser tab on the URL, `http` or `https` only (#492's rule for
    agents' URLs). On a loopback host the page loads once the port accepts a connection, waiting
    at most 30 seconds, so a dev server that an earlier item starts has time to come up.

  Optional keys on an item: `"title"` (a terminal's or an agent's name on its tab and rail row),
  `"cwd"` (the folder a terminal or an agent starts in, relative to the project root),
  `"split": "right" | "down"` (a new pane split off the pane of the item before), and
  `"focus": true` (the item that has the focus at the end; without one, the first item). An
  unknown key, two kinds in one item, or a second `focus` is an error.
- **The menu.** A project's + menu in the rail gains a "Launch" header after the agent CLIs (and
  after #510's New Agent in Worktree once it exists), with one entry per config in the file's
  order. A file that does not parse shows a single disabled
  entry naming `.zed/marley.json` and the first error, and the log holds the whole error. A
  project without the file shows no header.
- **The approval.** Choosing a config shows Zed's prompt with the config's text, one line per
  item (for example `terminal "dev server" in web: npm run dev`), and Run and Cancel. Run records
  the SHA-256 of that text in Zed's key-value store, keyed by the project's main checkout and the
  config's name, and opens the items; the next choice of the same text opens at once. A changed
  text asks again and says that it changed since it was approved. Cancel, Escape or a dismissed
  prompt open nothing and record nothing.
- **Reload.** An edit to the file shows in the next menu: Marley reads the file again when the
  project's worktree reports that it changed.

### Out (explicitly deferred)
- Configs for each new worktree (#510), slice 2: a `"new_worktree": true` key read when #510
  creates a worktree, under the same approval. It goes into #510's spec, or its own ticket if #510
  ships first.
- Configs in the user's own settings for every project (Warp keeps them per user); parameters
  asked for at open (Warp's `{{param}}`); a palette command or a URI that opens a config; saving
  the current layout as a config.
- A "trust every config in this project" choice (Orca's repo-wide trust).
- Tab colors, a shell per item, and Zed tasks (`.zed/tasks.json`) as items.

## Reference (§20)
Warp, from its published docs only (docs.warp.dev/terminal/windows/tab-configs/, item 4 of the
once-over note): a Tab Config is a file of panes, each with a directory and startup commands,
split nodes between them and optional parameters, listed in the + menu and opened in one click.
Marley keeps the open from the +, the per-item command and directory, and the splits. It leaves
out parameters, colors and per-pane shells, and keeps the file in the project, as Orca does,
instead of a per-user folder. Orca (MIT, read at `1c2cf120e3`): `orca.yaml`'s `defaultTabs`
(`title`, `color`, `command`), created once for a new worktree
(`src/shared/orca-yaml-hook-types.ts`, `src/renderer/src/lib/worktree-default-terminal-tabs.ts`),
and its trust model: the SHA-256 of the exact command text compared with the hash approved for
the repository, a new prompt when the text changed, and a dismissed prompt counted as a refusal
(`src/renderer/src/lib/ensure-hooks-confirmed.ts`, `src/renderer/src/lib/orca-hook-trust.ts`).
Upstream Zed: `.zed/` as the project's own config folder, and Zed's prompt.

### Prior art
- **Behavior maps.** `docs/orca_architecture/05-terminal-and-workspace.md` §2.7 (`defaultTabs`)
  and §3 item 6 (repo-declared first tabs, nothing run until its hash is approved);
  `02-worktrees-and-review.md` §2.2 (the trust dialog, the file read from the new worktree's own
  branch, setup in a visible terminal); the Warp once-over note, item 4.
- **Published material.** docs.warp.dev's Tab Configs page: the schema (`name`, `[[panes]]` with
  `id`, `type`, `directory`, `commands`, split nodes with `split` and `children`, `is_focused`)
  and its caution that a long-running command never hands its shell back, which is why an item
  here runs one command.
- **Code we already ship.** Two Zed seams came close to owning this and neither fits; the pieces
  under them are reused:
  - Zed's tasks (`crates/task/src/task_template.rs`, `TaskTemplate`: `command`, `args`, `env`,
    `cwd`, `reveal_target`, and `hooks` with `TaskHook::CreateWorktree`, which
    `Workspace::run_create_worktree_tasks` in `crates/workspace/src/tasks.rs` runs for a new
    worktree) hold one command each and run it in a task terminal. They cannot say "an agent", "a
    Browser tab" or "split right", nor group items, and a task terminal is not a shell the user
    keeps typing in. serde drops unknown keys in `tasks.json`, and the schema Zed gives the file
    marks them wrong (`TaskTemplates::generate_json_schema` with `DefaultDenyUnknownFields`), so a
    grouping key there would be flagged in the editor and lost before Marley saw it.
  - Zed's project settings (`.zed/settings.json`) would bring reload and a schema, but a project
    file is parsed as `ProjectSettingsContent` (`crates/settings_content/src/project.rs`,
    `crates/settings/src/settings_store.rs:1111`), which has no `marley` key; the `marley` block
    is user-level (`SettingsContent`, `settings_content.rs:251`). A project-level Marley key would
    take three Zed touches, where a Marley file in `.zed/` takes none.
  - Reused as they are: `settings::parse_json_with_comments`; `project::Event::WorktreeUpdatedEntries`
    for the file's changes; `Fs::load`; Zed's prompt, which gpui draws inside the window on Linux
    (`crates/ui_prompt/src/ui_prompt.rs:22`); `db::kvp::KeyValueStore` for the approvals;
    `Workspace::split_pane` and `add_item_to_active_pane`; `TerminalView::set_custom_title`; the
    rail's `render_project_menu`; `agents::start_cli`; `browser::new_tab` and its `open_page_in`;
    `marley_browser::address::agent_url` for the URL rule; `sha2`, already in the lock through
    `marley_browser`.

## UI proof
UI-AFFECTING. `script/e2e/527-project-launch-configs.sh` (`compositor sway`, Chromium offline): a
scratch repository whose `.zed/marley.json` names "Dev stack" (a terminal titled "dev server"
running `python3 -m http.server <port>` on a page in the repository, `claude` split right, and a
Browser tab on the server's URL split down) and "Shell" (one terminal running `echo launched`),
with a stand-in `claude` first on the PATH. Steps: the project's + and its menu (`527-01-menu`);
"Dev stack" chosen, and the prompt with its three lines (`527-02-approve`); Run, then the three
panes, the Browser tab showing the page the terminal's server serves (`527-03-opened`); "Shell"
chosen and run, then chosen again, which opens a second terminal with no prompt
(`527-04-no-prompt-again`); the file's "Shell" command changed on disk, then chosen: the prompt
again, saying the text changed (`527-05-changed`); Escape, and nothing new opens
(`527-06-cancelled`); "Shell" chosen once more, and the prompt comes back (`527-07-asks-again`);
the file broken on disk, then the + again (`527-08-broken-file`).

## Locked-In Decisions
- D1: The file is `.zed/marley.json`, JSON with comments, beside Zed's own `.zed/` files.
  `.zed/launch.json` was the other candidate; VS Code's `launch.json` is a debugger config and
  Zed imports `.vscode/launch.json` as debug scenarios, so the name would mislead and could
  collide with upstream later. The top-level `launch` key leaves room for other Marley project
  keys.
- D2: One approval per config, over its whole text: every item counts, since an agent's program
  and a URL are things the config does too. The text shown is the text hashed. The key is the
  project's main checkout (the rail group's `ProjectGroupKey`) and the config's name, so a
  worktree of the same repository with the same text does not ask again.
- D3: A terminal item is a shell with its command typed after the shell's ready handshake, as
  `start_cli` does, so the shell stays when the command ends and Marley's shell integration keeps
  its blocks. One command per terminal: a second command would wait behind a server that never
  exits.
- D4: Items open in order, each once the one before exists, so a split lands off the right pane.
- D5: A Browser item on a loopback host waits for its port and, after 30 seconds, loads the page
  anyway so Chromium shows why it failed. The wait is a TCP connect, made from `marley_browser`
  (its adapter modules already hold the browser's sockets), never from the workbench.
- D6: No Zed crate changes.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user opens a project's + menu in the rail and the project's `.zed/marley.json` names launch configs, the menu shall list each config by name under a "Launch" header, in the file's order. | Shot `527-01-menu` |
| REQ-002 | WHEN the user chooses a config whose text is not approved, Marley shall show that text, one line per item, with Run and Cancel, and open nothing until Run. | Shot `527-02-approve` |
| REQ-003 | WHEN the user runs a config, Marley shall open its items in order: each terminal with its command typed once its shell is ready, each agent as the rail's Agent CLIs entries start it, each Browser tab on its URL, and each item split right or down where it says so. | Shot `527-03-opened` |
| REQ-004 | WHEN a Browser item's URL is on a loopback host, Marley shall load the page once the port accepts a connection, or after 30 seconds. | Shot `527-03-opened` (the page comes from the config's own server) |
| REQ-005 | WHEN the user chooses a config whose exact text was approved before, Marley shall open it without asking. | Shot `527-04-no-prompt-again` |
| REQ-006 | WHEN a config's text changed after it was approved, Marley shall ask again and say that it changed. | Shot `527-05-changed` |
| REQ-007 | WHEN the user cancels or dismisses the prompt, Marley shall open nothing and record nothing. | Shots `527-06-cancelled` and `527-07-asks-again` |
| REQ-008 | WHEN `.zed/marley.json` does not parse, the + menu shall show one disabled entry naming the file and the first error. | Shot `527-08-broken-file` |
| REQ-009 | WHERE a project has no `.zed/marley.json`, its + menu shall be as it was before this ticket. | #500's scenario run again, its menu shot unchanged |

## Phase Plan
- **P1 Plan:** promote the pair, recall, consult the brain, confirm the design in the notes.
- **P2 Code:** `launch.rs` in `marley_workbench` (the file's types and parse, the text and its
  hash, the per-project cache and its reload, the approval, the item runner); the rail's menu
  entries; `browser::new_tab_at`; the port wait in `marley_browser`; fmt and clippy clean; a
  review of the diff.
- **P3 Test:** write and run the scenario, read every shot; run #500's scenario again;
  `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; the file's format in `docs/marley/README.md`, and the + menu's
  section (D4) of `docs/marley/workbench-shell.md`; the ledger; close the ticket, archive, commit.
