# New Agent in Worktree sits under New Agent Thread in the rail's + — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-598-worktree-agent-under-agent-thread.md
- **Pipeline spec:** 598-worktree-agent-under-agent-thread.spec.md

## Phase 1 — Plan (queued by /spec, 2026-09-30)
- **Request:** Chad, testing with `docs/marley/walkthrough.md`: move New Agent in Worktree up to
  sit below New Agent Thread.
- **Classification / tier:** feature, one Marley file, UI order only.
- **Recall (§18.3):** AD-claude-453 (the rail's header menu reorders through Zed's multi-workspace);
  #510's spec placed the entry "after the Agent CLIs"; #527's placed Launch after it. No failure or
  prevention rule touches the menu's order.
- **Discovery:** `crates/marley_workbench/src/rail.rs`, `render_project_menu` (about line 3397):
  the builder chains `entry("New Terminal")`, `entry("New Browser Tab")`,
  `submenu("New Agent Thread")`, then `agent_cli_entries` (a separator, the "Agent CLIs" header,
  one item per CLI), then `worktree_agent_entries` (the "New Agent in Worktree" submenu, about
  line 3529), then `launch_entries`. The fix calls `worktree_agent_entries` before
  `agent_cli_entries`.
- **Decisions:** see the spec's D1 to D3.

### Visual check plan
- REQ-001, REQ-002: a sway scenario with a scratch git repository (`git init`, one commit), a fake
  `claude` first on the PATH so the CLI list is not empty, and a `.zed/marley.json` with one
  launch config so the Launch header shows; click the project's `+`; `menu.png` must read New
  Terminal, New Browser Tab, New Agent Thread, New Agent in Worktree, Agent CLIs, Claude Code,
  Launch, the config.
- REQ-003: review; the condition code is untouched.

### Risks
- `script/e2e/510-worktree-agents.sh` may walk the menu by position with the arrow keys. It is
  not run per ticket (§7), but read it at Code and note it if it depends on the old order.

### Promotion (2026-09-30, `/pipeline:plan 598`)
- **Run mode:** Chad chose to run #598 to #605 back to back without a stop at each plan
  (answered 2026-09-30), so this plan did not wait for confirmation.
- **Pre-flight:** cargo 1.98.1, gates, e2e, cargo-shear 1.13.4 and hooks OK; no active pipeline;
  README marker present; cargo idle; `/mnt/fast` at 91% (watch it, see the target-dir cleanup
  rule before a build fills it).
- **Brain (`rusty-cli brain ask`, consultation 5c9916edba854ee99bb47c230f4229fc):** nothing on this
  seam; only unrelated due follow-ups came back.
- **Seams re-verified:** `render_project_menu` (`rail.rs:3397`) chains New Terminal, New Browser
  Tab, the New Agent Thread submenu, `agent_cli_entries` (separator, "Agent CLIs" header, the
  CLIs), `worktree_agent_entries` (`rail.rs:3529`, the "New Agent in Worktree" submenu), then
  `launch_entries`.
- **Found at promotion:** four scenarios walk the + menu by position (`WORKTREE_STEPS=7`, then a
  click on the submenu's Claude Code at `SUBMENU_CLAUDE_X/Y` 325/326, and 510's
  `WORKTREE_ENTRY_X/Y` 150/322): 510 and 585 are in `script/e2e/golden`, 587 and 589 are not. The
  spec's scope now keeps them in step.

### Design
- `rail.rs`, `render_project_menu`: call `Self::worktree_agent_entries(menu, &cli_workspace, …)`
  on the menu right after the New Agent Thread submenu, then `agent_cli_entries`, then
  `launch_entries`. `worktree_agent_entries`' doc comment: "after New Agent Thread".
- No separator of its own: the entry reads as the last of the plain entries, and the Agent CLIs
  separator still opens the CLI list.

### File manifest
- `crates/marley_workbench/src/rail.rs` (Marley crate): the call order, the doc comment.
- `script/e2e/510-worktree-agents.sh`, `585-worktree-environment.sh`,
  `587-claude-code-trust-in-a-new-worktree.sh`, `589-remove-a-worktree.sh` (scenarios): the step
  count 3 and the measured submenu points.
- `script/e2e/598-worktree-agent-under-agent-thread.sh` (new scenario).
- Docs at Complete: `CHANGELOG.md`, `docs/marley_architecture/marley_workbench.md`,
  `docs/marley/walkthrough.md` (stop 9.1 names the entry), `docs/marley/guide.md` (its + table).

### Checklist (no TaskCreate in this harness)
- [x] Pick the item · [x] pre-flight · [x] recall · [x] promote · [x] prior art (unchanged, in the
  spec) · [x] spec · [x] design · [x] presented (autonomous run)

## Phase 2 — Code
- **Built:** `render_project_menu` now calls `worktree_agent_entries` before `agent_cli_entries`
  (`crates/marley_workbench/src/rail.rs`); the doc comments of `worktree_agent_entries` ("right
  after New Agent Thread (#598)") and `agent_cli_entries` ("after New Agent Thread and New Agent in
  Worktree") say where each sits. The four scenarios that walk the menu by position now press Down
  3 times (`WORKTREE_STEPS=3`); 510's comment names the new order. Their submenu click points are
  measured from the Test phase's `menu.png`. The new scenario
  `script/e2e/598-worktree-agent-under-agent-thread.sh` is written.
- **Deviations:** none from the design.
- **Review of the diff:** REQ-001 and REQ-002 follow from the call order (`ContextMenu` draws in
  call order); REQ-003 holds since `worktree_agent_entries` returns the menu unchanged when
  `worktree_agents::offered` is false or no CLI is installed, and no condition moved. No entity is
  read while updated; no Zed crate touched; nothing from Warp.
- **Gate:** `just gate-diff` (scope `marley_workbench`): every gate PASS, receipt written
  (log in the session scratchpad, `gate-598.log`).

## Phase 3 — Test
- **Scenario:** `script/e2e/598-worktree-agent-under-agent-thread.sh` (sway), on the debug build
  (`just build`, `dev` profile). A scratch git repository with a commit, a fake `claude` first on
  the PATH, and a `.zed/marley.json` with one config `Dev`; a click on the project's `+`; then Down
  three times and Return.
- **`menu.png`** (REQ-001, REQ-002): the menu reads New Terminal, New Browser Tab, New Agent
  Thread (with its chevron), New Agent in Worktree (with its chevron), a separator, the "Agent
  CLIs" header with Claude Code, Codex, Gemini CLI and OpenCode, a separator, the "Launch" header
  with Dev. New Agent in Worktree sits directly under New Agent Thread. PASS.
- **`submenu.png`** (D1, and the other scenarios' points): Down three times and Return open New
  Agent in Worktree's submenu with the same four CLIs; Claude Code in it at (325, 197) on the
  1600×1000 sway output, the entry itself at y 193. The first run's hover alone did not open the
  submenu, which is why the worktree scenarios open it with Return; the scenario does the same.
- **REQ-003** (no entry for a non-git project or no CLI): by review, the condition is unchanged.
- **Scenario upkeep:** 510, 585, 587 and 589 now use `WORKTREE_STEPS=3` and
  `SUBMENU_CLAUDE_Y=197` (510's `WORKTREE_ENTRY_Y=193`), measured from `submenu.png`. 510 (golden)
  was run once to check the edit (below).
- **Focus report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule
  and did not reload it"; the sway run stopped with its Marley.
- **510 run once to check the edited points** (`script/e2e/510-worktree-agents.sh`, golden):
  every self-check passed (the agent started with bypass and its prompt as one argument; one
  `agent/` branch with no upstream and base `main`; the worktree in Zed's layout; no second
  branch or launch on the refused create), exit 0. `510-01-menu.png` shows New Agent in Worktree's
  submenu open beside it under New Agent Thread; `510-02-prompt.png` shows "Claude Code in a New
  Worktree" with `agent/ideal-flint from main`, so the click at (325, 197) landed on Claude Code.
  585, 587 and 589 share the same points and were not run (§7: no regression per ticket).
- **Shots:** in the session scratchpad (`shots-598/`, `shots-510/`), none in the repository.

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` (Unreleased, Changed: #598);
  `docs/marley_architecture/marley_workbench.md` (the entry's place, and Launch after the Agent
  CLIs); `docs/marley/workbench-shell.md` (the W4 status line names #598). No Zed path touched, so
  no touchpoint row. The guide page (#599) is drafted with the new order.
- **Ledger:** `L-claude-598-a-menu-entry-moves-with-the-scenarios-that-count-to-it-001`
  (lessons.md). No bug found in Code or Test, so no `F-`.
- **Brain:** consultation 5c9916edba854ee99bb47c230f4229fc closed with `no-decision` (a menu move
  as asked; the ticket and the lesson hold what matters).
- **Ticket:** closed; archived with this pair; committed with the change.
