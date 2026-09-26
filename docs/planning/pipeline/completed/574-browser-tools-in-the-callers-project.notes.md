# Browser tools act in the caller's project — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-574-browser-tools-in-the-callers-project.md
- **Pipeline spec:** 574-browser-tools-in-the-callers-project.spec.md

## Phase 1 — Plan
- **Request:** cut from TICKET-520 at its promotion (2026-09-26); #520's design, items 9 and 10,
  its D6 and D7 and its queued REQ-007 and REQ-008, is the start. Chad's goal of 2026-09-25 covers
  it ("The rest go ahead and begin implementing it now").
- **Classification / tier:** feature, prong 3 with prong 2. Marley crates only; no Zed path.
  Size M.
- **Checklist (no TaskCreate in this harness):** pick · pre-flight · recall · mint · prior art ·
  spec · design · present: all done here.
- **Recall (§18.3):**
  - AD-claude-493-one-browser-tab-per-page-001: an agent's page never takes the focus; it opens
    behind the tab in front, or in a split beside a pane showing other work. The new placement
    keeps these rules and changes only the workspace.
  - AD-claude-520-each-terminal-names-itself-and-the-bridge-names-the-caller-001: the `Caller`
    each call carries; a caller is a default, never an authority.
  - #520's notes: the hub's focus value is filtered to live pages (`page_state`); the history keeps
    that filter.
  - Brain consultation 1a3ef1f18c0a4cbaacbb1b919c7dd868: nothing on this seam.
- **Discovery (at `81d5f15cda`):** `browser_tools.rs:53` (`answer`, which spawns `run` without
  the caller today), `:115` (`page_of`: the named tab, else `hub.focused()`), `:143` (`tabs`),
  `:432` (`navigate`: a new page when asked or when the browser has none, else `page_of`);
  `browser.rs:398` (`BrowserHub`, its `focused: Option<String>`), `:520` (`focused`), `:527`
  (`set_focused`, called from the view's focus at `:3305`), `:535` (`tabs`), `:2798`
  (`place_tab`), `:3005` (`BrowserView::workspace`); `multi_workspace.rs:849`
  (`project_groups`), `workspace.rs:7685` (`root_paths`); `mcp.rs` (`terminals`, each terminal
  with its workspace; `Terminal::marley_terminal_id`).
- **Decisions:** D1 to D5 in the spec.

### Design
- **Approach.**
  1. `BrowserHub`: `focus_history: Vec<String>` replaces `focused`; `set_focused` moves the target
     to the end (bounded, say 64); `focused()` is the newest live one, as today;
     `focused_among(&HashSet<String>)` the newest live one in the set. `place_next_in:
     Option<WeakEntity<Workspace>>`, set before an agent's `new_page`, taken by `place_tab` for a
     page with no opener: its tab goes to that workspace's active pane (split beside it when the
     pane has the focus and shows other work, AD-claude-493), in that workspace's window.
  2. `browser_tools`: `answer` passes `call.caller()` down. `Scope { workspaces:
     Vec<WeakEntity<Workspace>>, home: WeakEntity<Workspace>, name: String }` from
     `caller_scope(caller, cx)`: the terminal's workspace's group, else the group whose
     workspaces' `root_paths` hold `Marley-Project`, else `Marley-Cwd` (longest root wins); `home`
     the group's last active workspace (`last_active_workspace_for_group`, as the rail reads it),
     else its first. The tabs in scope: `live_views` whose `workspace` is among them, by target.
  3. `page_of(hub, named_tab, scope)`: a named tab as today; else with a scope the newest focused
     tab in it, and with none a refusal naming `browser_navigate`; with no scope, as today.
     `navigate`: with a scope and no tab of its own, `place_next_in(home)` and a new page;
     `tabs`: `project` per tab (its view's workspace's group name) and `default` for this caller.
  4. `registry.rs`: the browser tools' descriptions: "with no `tab`, the tab the user focused
     last in the calling terminal's project; `browser_navigate` opens one there when it has none".
- **File manifest.** Marley: `crates/marley_workbench/src/browser.rs`,
  `crates/marley_workbench/src/browser_tools.rs`, `crates/marley_mcp/src/registry.rs`,
  `script/e2e/browser-fixture.sh` (the stand-in's `navigate` and `look` name no tab already;
  `tabs` prints `project` and `default`), `script/e2e/574-browser-tools-in-the-callers-project.sh`
  (Test). Zed: none.

### E2E plan
Fixtures: two scratch repositories A and B; an offline Chromium (`offline_chromium`) and a local
site with two pages (`serve_site`); a HOME whose `.bashrc` defines `agent` (the stand-in agent
through the plugin's bridge, as #520's scenario does). A opens at launch; B through a second
`$E2E_MARLEY B`, which #513 hands to the running Marley.

| REQ | Scenario part | Shot or log |
|---|---|---|
| — | open B; in B's terminal, `agent navigate <page one>` (B has no tab: a new one in B); focus it | `574-01-two-projects` |
| REQ-001 | in A's terminal, `agent look` with A holding no tab: refused (REQ-003); then REQ-002, then `agent look` again: A's tab, not B's | the run log |
| REQ-002 | in A's terminal, `agent navigate <page two>` | `574-02-scoped-navigate` |
| REQ-004 | in A's terminal, `agent tabs` | `574-03-tabs` |
| REQ-005 | the stand-in run from B's folder, variables blank, `navigate <page two>` | `574-04-cwd` |
| REQ-006 | the harness-side client, variables blank, from the repository root | the run log |

Not reached: an Agent Panel thread itself (REQ-005 runs the same bridge the same way Zed does:
from the project's root with blank variables).

### Risks
- #513's handoff may open B in a new window rather than the same window's rail; the scopes work
  across windows, and the scenario then takes the shots of the window in front.
- The history grows with focus changes; it is bounded and filtered to live pages.

## Phase 2 — Code
- **Checklist (no TaskCreate in this harness):** browser.rs (history, placements, `place_tab`,
  `tab_workspaces`, `window_of`) · browser_tools.rs (scope, `page_of`, `navigate`, `tabs`) ·
  mcp.rs (`caller_terminal`) · registry.rs (descriptions, the tabs schema) · browser-fixture.sh
  (the stand-in's `tabs` line) · fmt, clippy, rustdoc · review: all done.
- **Built.**
  - `BrowserHub`: `focus_history` (each page once, the newest last, at most 64) replaces `focused`;
    `set_focused` moves a page to the end; `focused()` and the new `focused_among(&HashSet)` share
    `newest_focused`: the history newest first, then the pages newest first, the first live one
    the filter takes. `page_gone` drops the page from the history.
  - `placements: Vec<(target, WeakEntity<Workspace>)>`: `create_page_task` takes `place_in` and
    records it for the new page's id as soon as `Target.createTarget` answers, before the page can
    attach (several round trips later). `place_tab` takes the page's placement: with one, it looks
    only among that workspace's tabs (after the one the user focused last there, else its newest)
    and else opens in that workspace's active pane, in that workspace's window (`window_of`),
    splitting beside a pane with the focus that shows other work, as AD-claude-493 does. A
    placement goes with its page, a failed attach or a restart.
  - `tab_workspaces`: each tab in a pane with its page and workspace, for the tools.
  - `browser_tools`: `answer` works out the caller's `Scope` once (`caller_scope`): the workspace
    of the caller's terminal (`mcp::caller_terminal`, which `terminal_of` now shares), else the
    local workspace one of whose own folders (`root_paths`) holds `Marley-Project`, else
    `Marley-Cwd`, the longest folder winning (`holding`); the scope is that workspace's group in
    its window (every held workspace with its `project_group_key`), with the group's rail name.
    `default_tab` is `focused_among` the scope's tabs, or `focused()` with no scope. `page_of`
    takes the scope: none of the project's tabs means a refusal at once ("no Browser tab of the
    project <name> shows a page; browser_navigate opens one there"). `navigate` opens a new page
    placed in the caller's workspace when the project has no tab showing a page. `tabs` adds
    `project` (the rail's name for the tab's workspace) and `default` (flattened beside the hub's
    `TabSummary`).
  - `registry.rs`: `browser_tabs`, `browser_navigate` and the `tab` argument say what a call with
    no tab acts on; the tabs schema has `project` and `default`, and `focused` says what it means.
  - The stand-in's `tabs` prints `, project <name>` after the URL and `, default` last.
- **Deviations from the design, and why.**
  - A placement keyed by the page's id instead of a one-shot `place_next_in`: a page the user or a
    page opens between the call and the attach would take a one-shot slot and open in the
    caller's project.
  - `home` is the caller's own workspace (its terminal's, or the one whose folder held the path),
    not the group's last active one: a linked worktree's agent (#510) gets its tab beside its own
    terminal. With one workspace per group, as today, the two are the same.
  - The rail's `group_names` moved to the crate root (`crate::group_names`): an item that the
    private `rail` module shares trips `redundant_pub_crate` as `pub(crate)` or `pub(super)` and
    `unreachable_pub` as `pub`. The rail's tests import `PathBuf` themselves now.
- **Review.**
  - Criteria: REQ-001 `page_of` with the scope; REQ-002 `navigate`'s placed new page; REQ-003 the
    refusal; REQ-004 `tabs`; REQ-005 `holding(Marley-Cwd)`; REQ-006 no scope, `focused()` as
    before. The golden set's browser scenarios call from the harness's shell in the repository,
    which no scenario opens as a project, so they keep today's behavior; only #520's scenario runs
    the stand-in in a terminal, and it calls the terminal tools.
  - Re-entrancy: `caller_scope` runs in the call's `cx.update`; `place_tab` updates the hub where
    it read it before (from the hub subscription's flush, or a tool's `cx.update`); `window_of`
    reads windows outside any window update.
  - Found and fixed: placements kept for a page that never attached or across a restart (now
    dropped with a failed attach and at a start); three clippy findings (`option_if_let_else`,
    `redundant_closure_for_method_calls`, the visibility pair above).
  - Known limit: a project opened through a symlink does not match a `Marley-Cwd` the bridge reads
    with `getcwd` (the resolved path); Marley's own terminals match by id or by `Marley-Project`,
    which Marley sets from the path it opened, and a caller that matches nothing keeps today's
    behavior.
  - Provenance: Marley code only; no Zed crate touched; nothing from Warp.
- **Checks:** `cargo check`, `cargo fmt --check`, `cargo clippy -p marley_workbench -p marley_mcp
  --all-targets -- -D warnings` (exit 0), `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` on both
  crates: clean.
- **For Complete:** an `L-` block for the visibility pair (an item a private module shares with
  its crate belongs in a public module or the crate root).

## Phase 3 — Test
- **Checklist (no TaskCreate in this harness):** 574-01 · REQ-003 refusal · 574-02 · REQ-001 look ·
  574-03 · 574-04 · REQ-006 harness-side client · golden set · gate.
- **Scenario:** `script/e2e/574-browser-tools-in-the-callers-project.sh` (`compositor sway`;
  offline Chromium; a loopback site with pages one, two and three; a scratch HOME whose `agent`
  runs the stand-in through the plugin's bridge and tees to `agent.log`). repo-a opens at launch;
  a second launch hands repo-b to the running Marley (#513), which opens it in the same window,
  above repo-a in the rail. `run_in_terminal` types into the focused terminal and prints what the
  stand-in answered; `from_folder` runs the stand-in from the harness in a folder with both
  variables blank. The rail's terminal rows (repo-b at y 136, repo-a at y 229) switch projects;
  the coordinates came from the first run, whose click at y 160 fell between the groups.
- **Runs.** Four runs; the three reds were two faults of the scenario's, none of the code's: the
  rail click between the groups (the `look` went to repo-b's terminal), then the refusal check's
  wording, which a run printing the stand-in's answers showed (the refusal reads `browser_look: no
  Browser tab of the project repo-a shows a page; browser_navigate opens one there` inside the
  server's `refused` envelope). The fourth run: every check passes.
- **Red on the build before this ticket:** run with `E2E_BINARY=~/.local/bin/marley` (the installed
  release, at 236fd6999b), `agent look` from repo-a's terminal read repo-b's tab (`Page one`, the
  tab the user focused last), and the run failed at the refusal check: the failure this ticket
  stops.
- **Checks in the final run (all pass):** the look from repo-a with no tab refused with its next
  step (REQ-003); repo-a's navigate opened a tab of its own (REQ-002); the look from repo-a read
  repo-a's tab on page two, though the user had focused repo-b's (REQ-001); `tabs` from repo-a:
  repo-a's tab `project repo-a, default`, repo-b's `project repo-b, focused`, still on page one
  (REQ-004, REQ-002); the stand-in from repo-b's folder with no id and no project navigated
  repo-b's tab (REQ-005); from the repository's root, in no project, `navigate` drove the tab the
  user focused last, repo-b's, to page three, repo-a's tab stayed on page two, and `tabs` marked
  repo-b's `focused, default` (REQ-006).
- **Shots, read:**
  - `574-00-handed-off`: one window; the rail lists repo-b above repo-a, each with its terminal;
    repo-b in front, its terminal focused at the `$` prompt.
  - `574-00b-b-navigated`: repo-b's `agent navigate …/one.html` and its answer in the terminal;
    Page one in a new Browser tab in a split beside the terminal, the Agent chip reading `went to
    …/one.html`; the terminal keeps the focus (AD-claude-493's split).
  - `574-01-two-projects`: after the click in the page, the Browser pane is the active one (its
    toolbar buttons show, the terminal's cursor is hollow), the rail's selection on the repo-b
    group: repo-b's tab is the one the user focused last.
  - `574-01b-in-a`: repo-a in front after its rail row's click, its terminal focused, no Browser
    tab.
  - `574-02-scoped-navigate`: repo-a's terminal shows the refused look and the navigate's answer;
    Page two in a new tab in a split beside repo-a's terminal, the Agent chip on it, the terminal
    still focused.
  - `574-03-tabs`: the look's answer names repo-a's tab at two.html; `agent tabs` lists repo-b's
    tab at one.html `project repo-b, focused` and repo-a's at two.html `project repo-a, default`.
  - `574-04-cwd`: repo-b in front, its tab on Page two with the Agent chip `went to …/two.html`,
    from the stand-in run in repo-b's folder.
- **Not reached:** an Agent Panel thread itself. The stand-in runs the same bridge Zed's agents
  get (`claude_plugin::BRIDGE`, written by `write_bridge_in`) the way Zed's context server runs it,
  from the project's folder with both variables blank.
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule
  and did not reload it" (the headless sway; the user's session untouched).
- **Golden set:** 574 added (`script/e2e/golden`); `just regress`: all 16 passed (491, 484, 481,
  492, 500, 515, 516, 513, 544, 546, 519, 547, 535, 550, 520, 574), among them 492 and 500,
  whose harness-side clients call from outside every project and keep today's behavior.
- **Gate:** `script/gates.sh --diff`: 16 passed, 0 failed, `GATE GREEN [diff]`, receipt written.
  Its log's dylint warnings are in Zed's `title_bar` and `sidebar`, which keep Zed's lints at warn:
  pre-existing, not in scope.
- **Verdict:** PASS.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: browser tools act in the agent's own project);
  `docs/marley_architecture/marley_workbench.md` (the browser tools' scope and default,
  `browser_tabs`' `project` and `default`, the hub's focus history and placements, the rail's
  naming in `crate::group_names`); `docs/marley_architecture/marley_mcp.md` (the descriptions and
  the tabs output); `docs/marley/three-prong-plan.md` (the B2b row, shipped). No Zed path was
  touched, so `docs/marley/zed-touchpoints.md` needs no row.
- **Knowledge appended:** F-claude-574-a-placement-outlived-its-page-001 (low; the Code review's
  find); L-claude-574-an-item-a-private-module-shares-lives-in-a-public-module-001;
  AD-claude-574-browser-tools-default-to-the-callers-project-001 (it narrows AD-claude-493's
  default). No PR block: the placement's class is covered by the review that found it.
- **Brain:** `rusty-cli brain decide 1a3ef1f18c0a4cbaacbb1b919c7dd868` →
  `decisions/marleys-browser-tools-act-in-the-callers-project`.
- **Ticket:** closed; the pipeline archived to `completed/`.
