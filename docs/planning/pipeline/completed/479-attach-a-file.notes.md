# Attach a file to an agent's prompt — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-479-attach-a-file.md
- **Pipeline spec:** 479-attach-a-file.spec.md

## Phase 1 — Plan (queued 2026-09-23)
- **Request:** Chad, 2026-09-23: "Attach a file which basically just provides the location of a
  file after choosing it in the claude session".
- **Recall.** The gpui-era Marley inserted paths from its file finder and its tree
  (`docs/marley/history/CHANGELOG-gpui-era.md`); not ported. Zed's file finder cannot return a
  path to a caller; its path prompt can.

## Phase 1 — Plan, at promotion (2026-09-23)
- **Recall:** the brain (consultation c39ebeb56c5442059e502c1b17133518) returned only unrelated
  follow-ups. #477's footer context carries the view and the workspace the button needs.
- **Seams re-verified.** `Workspace::prompt_for_open_path(PathPromptOptions, DirectoryLister,
  window, cx)` returns a receiver of the chosen paths, and uses the injected prompt when
  `use_system_path_prompts` is off, as tests set it. `DirectoryLister::Project` lists what the
  project sees, local or remote. `TerminalView::add_paths_to_terminal` is public and pastes the
  quoted paths. `blocks::focused_terminal` finds the focused terminal view for an action.

### Design
- `marley_workbench::attach` (a small module): `attach(view, workspace, window, cx)` opens the
  prompt for files, several allowed, and on a choice types them with `add_paths_to_terminal`;
  a cancel types nothing. The action `marley::AttachFile`, caught at the workspace's root as
  the block keys are, attaches to the focused terminal.
- The agent bar gets a `+` button, "Attach File", after the agent's name and chip.
- **Files:** `crates/marley_workbench/src/attach.rs` and its tests, `agent_bar.rs`,
  `blocks.rs` (`focused_terminal` shared), `marley_workbench.rs` (the action). Marley only.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: with a stand-in `claude` in the foreground, the bar shows Attach File |
| 002 | driven: an injected prompt returns two paths, one with a space; the PTY is sent ` '<a b>' <c> ` |
| 003 | driven: a cancelled prompt sends nothing |
| 004 | `just gate-diff` |

## Phase 2 — Code (2026-09-23)
- **Built.** `agent_bar::attach(view, workspace, window, cx)` opens
  `Workspace::prompt_for_open_path` for files, several allowed, with the project's lister and
  "Attach" as the accept label, and on a choice calls `TerminalView::add_paths_to_terminal`; a
  cancel (`None`) types nothing. The bar's `+` (`IconButton`, tooltip "Attach File" with any
  key bound to the action) sits after the agent's name and before the plugin's chip, so it
  stays put when the chip goes. `agent_bar::init` also registers `marley::AttachFile` on every
  workspace for the focused terminal (`blocks::focused_terminal`, now `pub(crate)`).
- **Deviation.** No `attach.rs`: the function and the action live in `agent_bar.rs`, since
  the button is the bar's and Zed's rules ask for no small files.
- **Seams checked.** With `use_system_path_prompts` on (the default) and a local project, the
  chooser is the desktop portal's (title "Open File", accept label from the prompt; this box has
  the gtk FileChooser portal). Otherwise, and for a remote project, it is Zed's own path prompt,
  which `file_finder::init` registers on every workspace.
- **Review.** Re-entrancy: the click handler updates the workspace, and nothing else is
  updating at that point; the action runs inside the workspace's listener and reads only other
  entities. A terminal closed while its chooser is open makes the update fail, and that is
  logged (`log_err`), as Zed does. Nothing is carried over from a Zed function body; the diff is
  Marley-only.

## Phase 3 — Test (2026-09-23)
- **Tests added** (`agent_bar_tests.rs`): `attach_file_types_the_chosen_paths_as_a_drop_does`
  (REQ-002: the click opens a chooser for files, several allowed, and two paths, one with
  spaces, reach the PTY as ` '<a b>' <c> ` in one write), `a_cancelled_chooser_types_nothing`
  (REQ-003), `the_action_attaches_to_the_focused_terminal_only` (the action, and with an empty
  pane focused it opens no chooser), and `marley-attach-file` added to the bar's selectors
  (REQ-001). The chooser is the test platform's, answered with `simulate_path_prompt_response`,
  not an injected prompt: the default settings keep system prompts on, so the default path runs.
- **Run.** `cargo nextest run -p marley_workbench -E 'test(/agent_bar/)'`: 10 passed.
- **Negative checks**, each restored by sha256: no button (3 tests fail), `multiple: false` (the
  paths test fails), a cancel that types a space (the cancel test fails), and a focused-terminal
  finder that takes any terminal while anything has focus (the action test fails).
- **Live drive.** `OPEN=<a scratch repo> just shot attach-479 agent-seed.sh`: the bar reads
  "Claude Code", a muted `+`, then "Enable Claude Code notifications", with the folder and
  `marley-demo` at the right. The chooser is not driven live: it needs a pointer in Chad's
  session.
- **Gate.** The first `just gate-diff` was red on gate:1 alone (rustfmt wanted a test helper's
  return tuple split over lines); after `cargo fmt`, `GATE GREEN [diff]`, receipt matching.

## Phase 4 — Complete (2026-09-23)
- **Docs.** CHANGELOG (Added: Attach File); `docs/marley_architecture/marley_workbench.md`
  (Attach File under the agent bar, and a paragraph on the agent bar's, the notifications' and
  the plugin's test files, which #477, #478 and #482 had left out); the plan's T7 row marks T7c
  shipped. No Zed path changed, so no ledger row.
- **Knowledge.** AD-claude-479-attach-file-types-paths-through-zeds-path-prompt-001,
  L-claude-479-drive-a-file-chooser-through-the-test-platforms-path-prompt-001. No failures: the
  one red was formatting.
- **Brain.** Consultation c39ebeb56c5442059e502c1b17133518 closed with the decision
  `marleys-attach-file-types-paths-through-zeds-path-prompt`, follow-up by 2026-10-07.
