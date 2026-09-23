# The Marley layout by default — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-460-marley-layout-by-default.md
- **Pipeline spec:** 460-marley-layout-by-default.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (Chad asked for it).
- **Request:** Chad, 2026-09-23, after seeing the fork: "when the program is installed the user
  shouldnt have to choose Marley at first it should swap it over."
- **Classification / tier:** feature, small; `settings_content` (Marley's file), the `zed`
  crate's test init, `marley_workbench`.
- **Pre-flight:** no active pipeline, README marker present, cargo idle.
- **Recall (§18.3).**
  - The shell plan's Default paragraph (`workbench-shell.md:133-136`) chose Zed as the
    default and named the flip as the alternative.
  - #438 put the setting's default on the enum and kept `default.json` free of a `marley`
    block.
  - Brain: consultation `862df7124de5465093cad11b39fe810a`, nothing on this seam.
- **Discovery.**
  - Only `crates/settings_content/src/marley.rs` names `MarleyLayout` outside the Marley
    crates.
  - `zed`'s `init_test_with_state` ends in `initialize_workspace`, which calls
    `marley_workbench::init`: the only test path into the workbench from a Zed crate.
  - Marley tests leaning on the Zed default include
    `a_window_in_the_zed_layout_gets_zeds_sidebar_and_zeds_defaults`, which never sets the
    layout, and `the_layout_actions_write_the_choice_to_the_settings_file`, whose first "use
    marley" expects a write.

### Design
- `marley.rs`: `#[default]` moves to `Marley`, and the field's doc says `Default: "marley"`.
- `marley_workbench.rs`: the `from_settings` comment names the new default; the code is
  unchanged.
- `zed.rs`: in `init_test_with_state`, before `initialize_workspace`, a `// Marley:` hunk
  writes `settings.marley.layout = Some(MarleyLayout::Zed)` to the user settings.
- Tests: every Marley test that means Zed's layout says so with `set_layout(Zed)`; the
  layout-actions test starts from Zed; a new test reads the default.
- **File manifest.**
  - Zed crates: `crates/settings_content/src/marley.rs` and `crates/zed/src/zed.rs`, both
    rows updated in `docs/marley/zed-touchpoints.md` first.
  - Marley: `crates/marley_workbench/src/marley_workbench.rs` and `marley_workbench_tests.rs`,
    and any other test file the suite shows leaning on the default.
  - Docs at Complete: `CHANGELOG.md`, `docs/marley/workbench-shell.md`,
    `docs/marley_architecture/marley_workbench.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | unit: with default settings `MarleySettings::layout` is `Marley`; driven: a window opened with no layout set gets the rail and the Marley defaults |
| 002 | driven: the existing Zed-layout tests, now setting `zed` explicitly |
| 003 | `cargo nextest run -p zed` green; negative check: without the pin, a `zed` test that expects Zed's layout fails |
| 004 | `script/gates.sh --diff` |

### Risks
- **The `zed` crate's tests** are many and slow to build. The pin keeps them on Zed's layout,
  and the gate runs them because the change touches the crate.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] ledger rows first · [x] `marley.rs` · [x] `marley_workbench.rs` · [x]
  `zed.rs` · [x] the Marley tests · [x] fmt · [ ] clippy (with the gate).
- **Built.**
  - The two ledger rows (`settings_content/src/marley.rs`, `zed/src/zed.rs`) name the change
    and why, before either file changed.
  - `MarleyLayout`'s `#[default]` is `Marley`, and the field's doc reads `Default: "marley"`.
  - `MarleySettings::from_settings`'s comment names the new default; its code is unchanged.
  - `init_test_with_state` in `zed.rs` writes `marley.layout = zed` to the user settings just
    before `initialize_workspace`, with a `// Marley:` comment.
- **The Marley tests.** With the default flipped, 13 of 134 failed, all of them tests that began
  in Zed's layout without saying so. Eight set it now (`set_layout(MarleyLayout::Zed)` before
  `init`), and `open_with_docks` sets it for the five dock round-trip tests.
  `the_layout_actions_write_the_choice_to_the_settings_file` walks from the new default:
  Zed, then Marley, then Marley again, which leaves the file alone. New:
  `with_no_layout_chosen_a_window_opens_in_the_marley_layout`. 135 of 135 pass.
- **Deviations:** none from the design.

## Phase 3 — Test (2026-09-23)
- **Checklist:** [x] REQ-001 and REQ-002 tests · [x] REQ-003 with its negative check · [x] live
  drive · [x] gate.
- **Marley:** `cargo nextest run -p marley_workbench`: 135 passed, among them
  `with_no_layout_chosen_a_window_opens_in_the_marley_layout` (REQ-001) and the Zed-layout
  tests that now set `zed` (REQ-002). Before those tests were made explicit, 13 failed: the
  negative check for REQ-001, since the flip alone turned them.
- **Zed (REQ-003):** `cargo nextest run -p zed` with the pin: 93 passed, 1 skipped. Without the
  pin: 69 passed, 24 failed, among them `test_open_paths`, `test_multi_workspace_session_restore`,
  `test_reload_restores_project_windows_and_tabs` and eleven `open_listener` tests. Restored by
  sha256 checksum.
- **Live drive (REQ-001):** the rebuilt binary started with `--user-data-dir` on an empty
  scratch profile and a scratch folder, beside Chad's own window, with no input sent. The
  window came up in the Marley layout: the rail (PROJECTS, the filter), the folder with a
  terminal already under it, that terminal in the center (#455's first terminal, seen live for
  the first time), and the project panel on the right. Zed's own Restricted Mode prompt for an
  untrusted new folder sat on top: upstream's, not ours. SIGTERM ended that instance only, and
  the scratch profile was removed.
- **A cargo in the way:** rust-analyzer inside Chad's open Marley window started `cargo check
  --workspace` after the negative check touched `zed.rs`; the build waited for it.
- **Gate:** `script/gates.sh --diff`, scope the Marley crates plus `settings_content`,
  `marley_workbench` and `zed`: GATE GREEN [diff], 20 of 20. nextest over the scope ran 580
  tests (580 passed, 2 skipped); coverage at 100% of lines (2600) and functions (396). The
  receipt matches the tree.
- **Pre-existing:** none.

## Phase 4 — Complete (2026-09-23)
- **Checklist:** [x] document · [x] knowledge · [x] close the ticket · [x] archive · [x] commit.
- **Docs:** `CHANGELOG.md` (Changed); `docs/marley/workbench-shell.md` (the Default paragraph);
  `docs/marley_architecture/marley_workbench.md` (the setting's default and Zed's test pin);
  both ledger rows in `docs/marley/zed-touchpoints.md` describe what shipped.
- **Knowledge:** `AD-claude-460-the-fork-starts-in-the-marley-layout-001`,
  `L-claude-460-a-fresh-install-drive-beside-chads-own-window-001`. No `F-` block: no bug.
- **Brain:** consultation `862df7124de5465093cad11b39fe810a` closed with a decision, follow-up
  by 2026-10-07.
- **Ticket:** #460 closed; it never had a BACKLOG row (minted and promoted at once).
