# The test suites run and green — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-634-the-test-suites-green.md
- **Pipeline spec:** 634-the-test-suites-green.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-01)
- **Request:** wave 5 (Chad, 2026-09-30); the workflow's "unit/regression/mutation at the end".
- **Recall (§18.3):** AD-claude-483; L-claude-465; the shared target dir and one cargo at a time
  (`/mnt/fast` near full: watch it, per the memory's rule).

## Phase 1 — Plan (promoted 2026-10-01)
- **Classification / tier:** chore; tests and whatever code a failing test exposes. Zed crates
  only where a Marley hunk causes a Zed test's failure.
- **Recall (§18.3):** #475 found seven `marley_workbench` failures that predate it (five
  non-deterministic, one focus assertion since #627, one missing `RequestedDirectories`);
  F-claude-443-e (a PTY child can outlive its test process: check for orphans after the run);
  Zed's nextest profile ends a test after 60 s. The brain had nothing on the question.
- **Discovery:** Marley's tests in Zed crates, counted from the fork point `78648aaf7d`: `paths`
  2, `terminal` 10, `terminal_view` 13, `zed` 1. Zed crates whose Rust Marley changed: 21 (the
  spec's Tier 2 list).
- **Disk:** `/mnt/fast` was at 100% (5.9G free) at promotion; Marley's incremental variants
  older than a day beyond each crate's four newest (2,806 folders) were dropped, per the memory's
  rule: 193G free.

### Design
- **Approach:** two runs per tier. First run, everything with `--no-fail-fast`, log kept in the
  scratchpad; a triage table here with one row per failure; fixes (a test's expectation
  following its ticket, `allow_parking` for a test that runs real IO, a harness setup brought up
  to date, or a code fix); the failing tests again until green; then the whole tier again.
  Tier 1 first, as one `cargo nextest run` over the eleven Marley crates plus `paths`,
  `terminal`, `terminal_view` (their whole suites, which hold Marley's tests) and `zed`'s one
  Marley test by name. Tier 2 next, the remaining Zed crates' suites in one run.
- **File manifest:** decided by the failures: `crates/marley_*` test files and sources (Marley);
  a Zed test or hunk only where a Marley change broke it, its touchpoint row first.

### Visual check plan
| REQ | What runs | What shows it |
|---|---|---|
| REQ-001 | Tier 1's second whole run | its summary line: all passed, or each skip named |
| REQ-002 | Tier 2's run | its summary, and the table's Tier 2 rows |
| REQ-003 | the first runs | the triage table, one row per failure |
| REQ-004 | `git diff` | no test function added |
| §7 | `just shot` | Marley starts as before; a fixed bug in UI code is seen in its own scenario |

### Risks
- Tier 2's test binaries (`editor`, `project`, `workspace`, `zed`, `agent_ui`) are large: a long
  build and tens of gigabytes; `df` between the tiers.
- A test that leaves a real shell behind (F-claude-443-e): `pgrep` for orphans after each run.
- A non-deterministic failure can pass on a rerun; it is fixed at its cause, never retried away.

## Phase 2 — Code
### Tier 1, first run (2026-10-01)
`cargo nextest run` over the eleven Marley crates, `paths`, `terminal`, `terminal_view` and
`zed`'s Marley test: **724 run, 596 passed, 128 failed**, 93 skipped (log in the scratchpad).

| Class | Count | Tests | Cause | Fix |
|---|---|---|---|---|
| non-deterministic | 103 | most of `rail::tests`, `marley_workbench_tests`, `agents::tests` | Real IO in a gpui test: an open rail's ports scan reads `/proc` and asks `docker` and `systemctl` (#521, #614, #615: `async-process`, `async-io`); `smol::unblock` in the project icon search (#564), the search path's commands (#557) and Codex's and OpenCode's notification setup (#552) (`blocking-N`). The tests' assertions passed: gpui fails a test at its end when a foreign thread woke one of its tasks. | The three `smol::unblock` calls use gpui's background executor with `futures::future::lazy`, as the crate's other blocking reads do (`voice`, `claude_plugin`, `ports`' own `/proc` read): the same work off the main thread, and a test drives it. The ports scan reads its root from `Ports::proc_root`; the shared test setups point it at a folder with no sockets, so a test never reads the machine's ports. `smol` leaves `marley_workbench`'s dependencies. |
| harness | 2 | `agents::tests::choosing_an_agent_cli_starts_it_in_a_new_center_terminal`, `rail::tests::agents::an_agent_cli_starts_in_a_new_terminal_in_its_project` | #596 binds ssh's passphrase socket before an agent's terminal opens in a local project, real IO the test never finishes, so no terminal opened. | `Launcher` (the test seam for agent starts) gains `passphrase_dialog`; the shared test setups skip the dialog. |
| stale (#491 to #586, #567) | 3 | `marley_mcp` `registry_is_exactly_the_l1_set_with_correct_tiers`, `tools_list_derives_from_registry_with_input_and_output_schemas`, `dispatch::tests::tools_list_returns_two_tools` | The registry grew from L1's five tools to 31; #567 lists the two finds only while enabled. | The expected list and count; the conditional tools left out of the expected `tools/list`; dispatch's list compared with the registry's. |
| stale (#526) | 2 | `marley_terminal` `bash_starts_with_marleys_rcfile_…`, `zsh_starts_with_marleys_zdotdir_…` | #526 hands every integrated shell `MARLEY_SSH_COMMAND`. | The variable in the expected environment. |
| stale (#536) | 2 | `agent_bar::tests::attach_file_types_the_chosen_paths_as_a_drop_does`, `the_action_attaches_to_the_focused_terminal_only` | With an agent CLI in the foreground, #536 writes each path on its own, quoted where needed, a space after it. | The expected writes. |
| stale (#554) | 2 | `blocks::tests::the_block_keys_walk_the_focused_terminals_blocks`, `the_block_keys_work_in_the_terminal_panel` | The block keys select the block before and scroll only when it is off screen, in place of scrolling each block's start to the top. | The tests follow the selection and check the selected block shows. |
| stale (#538) | 2 | `notifications::tests::a_notification_from_a_terminal_out_of_focus_goes_to_the_desktop`, `the_focused_terminal_of_the_active_window_goes_to_no_desktop` | #538's five seconds without a second banner per project: of a burst's two notifications, one shows. | One expected. |
| stale (#606) | 3 | `rail::tests::a_group_with_no_open_workspace_is_not_listed`, `a_project_that_loses_its_last_folder_leaves_the_rail`, `enter_with_no_row_highlighted_does_nothing` | Every project group of the window is a header, a closed one too; Zed keeps the group of a workspace whose last folder goes (`handle_project_group_key_change`). | The expected names; the first two renamed to say what they now check (`…_is_listed_closed`, `…_stays_closed`). |
| stale (#500) | 3 | `rail::tests::threads::the_agent_submenu_lists_…`, `agents_take_their_name_and_icon_from_the_registry`, `new_agent_thread_starts_one_in_that_projects_panel` | The helper opened New Agent Thread from the row under New Terminal, which is New Browser Tab since #500. | The helper finds the row under New Browser Tab. |
| stale (#621) | 5 | `terminal` `marley_shell_hooks_leave_a_finished_block_with_its_output`; `terminal_view` `marley_blocks_draw_their_pills_…`, `marley_a_hovered_blocks_buttons_…`, `marley_short_content_sits_on_the_bottom_edge`, `marley_a_click_on_shifted_content_reports_its_own_row` | A local task terminal opens its task's own block at index 0, and these tests run their scripts as tasks: every block index moved up one. | `finished_block` skips the task's block; the two-block tests name blocks 1 and 2 and wait for three finished (four blocks where a third runs), the passing ones too, which had been checking the wrong block. |
| **bug** | 1 (3 once the IO was gone) | `routing::tests::a_folder_opened_fresh_starts_with_a_terminal_at_its_root`, then `toggle_and_toggle_focus_switch_between_the_code_and_its_terminal`, `toggle_goes_back_to_the_terminal_used_last` | See "The focus bug" below. | `rich_input::holds_focus`, used by every check of whether a terminal view has the keys. |

### Runs 2 to 4
- Run 2 (the IO, harness and stale fixes): **724 run, 719 passed, 5 failed**, in 10.6 s where
  run 1 took 49.8 s (the tests no longer wait on real processes). Left: two more stale (below),
  and three routing tests on the terminal's focus.
  - stale (#520): `tools_list_derives_from_registry_…` read the blocks tool's `terminal` as
    required; since #520 it is optional (the calling terminal when left out). The test checks
    the argument exists.
  - stale (#526): `marley_a_hovered_blocks_buttons_copy_and_rerun_it` found no Rerun: since
    #526 a Rerun is offered only while a known shell waits at its prompt, and the script's
    prompt frames were unsigned. `MARLEY_TWO_BLOCKS` signs its three prompt frames with
    `nonce=`, as the shell integration does.
- **The focus bug.** A temporary probe (removed) showed the focus on the shell's prompt editor
  (#627), drawn in the terminal view's footer (#477), while
  `view.focus_handle(cx).contains_focused(..)` was false. Zed's `TerminalElement` tracks the
  view's focus handle on the grid's own node (`.track_focus(&focus)`), gpui maps a handle to the
  last node that tracked it, and the footer is the grid's sibling, so a focused prompt editor is
  not inside the view as gpui counts. Upstream has no focusable footer, so only Marley meets it.
  Effects in the app while the prompt editor has the keys (by default at every prompt since #627):
  the terminal toggle reads the terminal as unfocused and refocuses it instead of going back to
  the code; the block keys find no focused terminal; `notifications::looking_at` is false, so a
  banner, a phone push (#535), a long command's note (#551) or a running error's question (#572)
  goes out for the terminal the user is typing in; the rail's selection does not follow it.
  `rich_input.rs` itself had worked around it (`contains_focused(..) || editor_focused`).
  Fixed: `rich_input::holds_focus(view, window, cx)`, true when the view's handle contains the
  focus or the open editor of its terminal holds it, used by `routing::toggle_terminal`,
  `blocks::focused_terminal`, `notifications::looking_at` and the rail's focused terminal. The
  routing tests ask the same (`terminal_focused`).
- Run 4: **724 run, 724 passed**, 93 skipped (Zed's `#[ignore]`d and platform tests). No test
  shell was left running afterwards (F-claude-443-e).

### Tier 2 (the 18 other Zed crates Marley changed)
- First run: **3,499 run, 3,488 passed, 11 failed**, all in `zed` (bin `marley`), all gpui's
  "not deterministic" on `async-io`: `test_new_empty_workspace`, `test_open_add_new`,
  `test_open_file_in_many_spaces`, `test_open_non_existing_file`,
  `test_open_remote_from_existing_connection_reuses_window`,
  `test_setting_language_when_saving_as_single_file_worktree`,
  `test_window_edit_state_restoring_disabled`, and `open_listener`'s
  `test_add_flag_prefers_focused_window`, `test_open_workspace_with_directory`,
  `test_open_workspace_with_nonexistent_files`, `test_reuse_flag_functionality`.
- **Harness, caused by Marley's hunk:** the `zed` tests run `initialize_workspace`, which runs
  `marley_workbench::init` (#443). That offers Zed's agents two context servers by default,
  Marley's own bridge (#501) and `rusty-mcp` where it is installed (#633), and Zed's context
  server store starts every configured server for each project (`maintain_servers`), so each
  test that opened a project started real processes. Upstream has no default servers. Fixed in
  `init_test_with_state`'s Marley hunk, beside `marley.layout = zed`: `marley.rusty_tools` off
  and a disabled `context_servers.marley` of the user's.
- **Hygiene, same cause:** the same run wrote the user's data folder: it added
  `shell_integration/fish/vendor_conf.d/marley.fish` (the tree's script, as the installed build
  writes on its next launch, so no harm this time), opened the prompts database and touched
  `logs/telemetry.log`. #475's ctor did not cover the `zed` binary, whose Marley init writes the
  data directory (the MCP server's bridge and opener scripts, the endpoint file); it does now
  (`terminal` with `test-support` and `ctor` among `zed`'s dev-dependencies).
- `zed` alone after both: 93 passed. Second Tier 2 run: **3,499 run, 3,499 passed**.

### Not in scope: Zed's own tests and Zed's data folder
Most Zed crates' tests touch `logs/telemetry.log` (empty, Zed's client telemetry) under the data
folder, and `agent_ui`'s open the prompts database there: checked crate by crate (`agent_ui`,
`workspace`, `git_ui`, `project`, `editor`, `settings_ui`; `title_bar` touches nothing). No Marley
hunk is involved; upstream's tests do the same under `~/.local/share/zed`. Left as they are.

### Phase 2 summary
- **Built:** the test-only changes in the triage tables; three code changes: gpui's background
  executor for the three `smol::unblock` reads (`smol` leaves `marley_workbench`), the
  `Ports::proc_root` and `Launcher::passphrase_dialog` seams the shared test setups set, and
  `rich_input::holds_focus` (the focus bug). The `zed` test binary gets #475's ctor and keeps
  Marley's offered context servers off.
- **Deviations from the plan:** Tier 1 took four runs and Tier 2 two, as planned; the `zed`
  binary's data folder (#475's gap) was not foreseen.
- **Review:** no test function added (two renamed to say what they now check); every stale
  expectation names its ticket; each code change keeps the shipped behavior (same work off the
  main thread; the seams default to the shipped values; `holds_focus` is the old check plus the
  footer's editor). Zed hunks: test modules and dev-dependencies of `terminal`, `terminal_view`
  and `zed`, each row extended first.
- **Gate:** `GATE GREEN [diff]`, 17 passed, on the first run.
