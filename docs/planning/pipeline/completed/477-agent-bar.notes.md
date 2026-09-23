# The agent bar, with the folder and branch — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-477-agent-bar.md
- **Pipeline spec:** 477-agent-bar.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23, the third of five things to bring over from Warp's bar: "has
  the location of what folder and which branch you are on on the bottom right". The bar is the
  container the other four go in.
- **Recall.** The rail already recognizes agents in terminals and gives them a waiting status
  (`rail.rs`, `marley_agent`); `docs/marley/workbench-shell.md` asks for the branch on the
  project header, not yet built.
- **Seams read:** `TerminalView::render` (a `div` root, one full-size container with the
  element); `Terminal::foreground_process_command_name` and `working_directory`; the git
  store's branch as the title bar reads it.

## Phase 1 — Plan, at promotion (2026-09-23)
- **Recall:** nothing on a terminal footer in the ledger; the brain (consultation
  990de556d6a84806ab2203a64f336ccf) returned only unrelated follow-ups.
- **Seams re-verified.** `TerminalView::render`'s root is a `div` with one full-size container
  child; the view keeps `terminal`, `project`, `workspace` and `focus_handle` private, with only
  `terminal()` public. `Terminal::foreground_process_command_name` reads the process info that a
  `Wakeup` refreshes, so a quiet foreground process is only seen after output. The git store's
  `repositories()` map to `Repository`s whose snapshot holds `work_directory_abs_path` and
  `branch`. `agents::cli_icon` and `AgentKind::display_name` name an agent; `PathExt::compact`
  writes the home directory as `~`.

### Design
- **Zed** (`terminal_view.rs`): `MarleyFooterContext` (the view's weak handle, its terminal,
  project, workspace and focus handle) and the global `MarleyTerminalFooter`, an `Arc`'d
  renderer. `render` calls it before building the root; the root becomes a flex column, the
  container gets `min_h_0` so it can give up rows, and the footer follows it.
- **Marley** (`marley_workbench`): a new `agent_bar` module. `init` sets the global.
  `contents(context, cx)` is what the bar shows: the agent in the foreground, the folder, and the
  branch from `branch_for`, a pure longest-match over the repositories' work directories. The
  renderer draws the icon and name at the left and the folder and branch at the right, with
  debug selectors for the tests. It is `flex_none`, with a top border, on the terminal's
  background.
- **Files:** `crates/terminal_view/src/terminal_view.rs` (Zed, row first);
  `crates/marley_workbench/src/agent_bar.rs`, `agent_bar_tests.rs`, `marley_workbench.rs`
  (Marley); the crate's `Cargo.toml` if it needs `project` or `util`.

### Test plan
| REQ | Test |
|---|---|
| 001, 003 | driven: a real PTY `exec`s `claude`, a link to `sleep` on a scratch PATH, in a scratch folder; after a keystroke's echo refreshes the process info, the bar shows with the agent's name and the folder |
| 002 | driven: a terminal running `sleep` shows no bar and keeps its rows; with the bar the grid has fewer lines |
| 004 | driven: the scratch folder is also a FakeFs repository on branch `main`, which the bar shows; unit: `branch_for` picks the innermost repository and none outside |
| 005 | `just gate-diff` |

### Risks
- A foreground process that prints nothing is only seen after its next output; real agents
  draw their screens at once.

## Phase 2 — Code (2026-09-23)
- **Built.** Zed, `terminal_view.rs`: `MarleyFooterContext` and the global `MarleyTerminalFooter`;
  `render` builds the context and calls the renderer before the root, the root becomes a flex
  column, and the footer follows the container. Marley, `marley_workbench::agent_bar`: `init`
  sets the global; `contents` reads the agent from `foreground_process_command_name`, the folder
  from `working_directory`, and the branch through `branch_for`, the innermost repository whose
  work directory holds the folder; `render` draws the agent's icon and name at the left and the
  folder (`~` for home) and the branch at the right, truncating, `flex_none`, with a top border
  on the terminal's background.
- **Deviation:** the plan gave the container `min_h_0` so it could give up rows; the negative
  check showed the column shrinks it without, so it was left out, and the ledger row says so.
- **Review.** Re-entrancy: the renderer reads the terminal and the project, never the view being
  rendered. Clippy: the module doc's first paragraph, an `Arc::clone`, two closures. Upstream: a
  struct, a global and three calls in `render`; the row was written first.

## Phase 3 — Test (2026-09-23)
- **Tests** (`marley_workbench/src/agent_bar_tests.rs`), over a real PTY in a real scratch folder
  that the project's FakeFs also holds as a repository on `main`, with `claude` a link to `sleep`
  first on the PATH:
  - `the_bar_shows_the_agent_its_folder_and_its_branch` (REQ-001, REQ-003, REQ-004);
  - `without_an_agent_there_is_no_bar_and_every_row_is_the_terminals` (REQ-002): the shell
    waits in `read` with no bar, then becomes `claude`; the grid has fewer lines with the bar;
  - `the_branch_comes_from_the_innermost_repository` (unit).
  A quiet foreground process is only seen after output, so the tests send a space, which the tty
  echoes (L-claude-477-a-quiet-foreground-process-is-seen-only-after-output-001).
- **Negative checks**, each file restored by sha256:
  1. no agent ever detected: FAIL in both driven tests;
  2. the view's root not a flex column: FAIL, `55 lines with the bar, 55 without`;
  3. `min_h_0` removed from the container: PASS, so it was dropped (above);
  4. the outermost repository picked: FAIL, `left: Some("main")`, `right: Some("feature")`;
  5. every repository matched: FAIL, `left: Some("feature")`, `right: Some("main")`.
- **Live drive** (`OPEN=<a fresh repository on branch marley-demo> just shot bar-477` with a seed
  whose `.bashrc` becomes `exec -a claude bash -c 'while :; do printf .; sleep 1; done'`): the
  bar under the terminal with the Claude icon and "Claude Code" at the left and the repository's
  folder and `marley-demo` at the right; the stand-in's dots on the terminal's last row above
  it. A first stand-in that only slept printed nothing, so its terminal never read the
  foreground process again and showed no bar; a real agent draws at once.
- **The gate's first run was red** on my own lines: the branch test's name, which misspelled
  "repositories" (typos), and three `SharedString`s built from
  literals in the tests, which Zed's dylint lint `shared_string_from_str_literal` wants as
  `SharedString::new_static`. Fixed at the source.
- **Gate:** `just gate-diff` over `marley_workbench` and `terminal_view`: 20 passed, 0 failed,
  `GATE GREEN [diff]`; 584 tests in the scope and 148 in `marley_workbench` pass, coverage
  100% of lines.

## Phase 4 — Complete (2026-09-23)
- **Docs:** `CHANGELOG.md` (#477); the three-prong plan (T7a shipped); the `terminal_view.rs`
  ledger row, checked against what ships; `docs/marley_architecture/marley_workbench.md` (the
  agent bar module).
- **Knowledge:** AD-claude-477-a-footer-hook-in-zeds-terminal-view-and-the-bar-in-marleys-crate-001,
  L-claude-477-a-quiet-foreground-process-is-seen-only-after-output-001. No F-block: no bug was
  found.
- **Brain:** consultation 990de556d6a84806ab2203a64f336ccf, decided at Complete.
