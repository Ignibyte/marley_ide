# Merge upstream Zed — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-695-merge-upstream-zed.md
- **Pipeline spec:** 695-merge-upstream-zed.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-08: "lets bring down upstream and ensure everything is good then ill
  restart it".
- **Trial merge:** `git fetch upstream main` gives `9ab0715969` (2026-10-08). The merge base is
  `78648aaf7d` (2026-09-18), with 319 commits and 812 files to bring in. `git merge --no-commit`
  conflicts in 8 files:
  - `Cargo.lock` (1 hunk);
  - `crates/agent_servers/src/acp.rs` (1);
  - `crates/markdown/src/markdown.rs` (4);
  - `crates/migrator/src/migrations.rs` (1) and `migrator.rs` (1);
  - `crates/terminal/src/terminal.rs` (4);
  - `crates/terminal_view/src/terminal_element.rs` (3) and `terminal_view.rs` (1).
- **Disk:** The shared build disk had 16G free. The cleanup recipe removed 1,732 old incremental variants of
  Marley's crates (133G), leaving 131G.
- **Design:** for each file, keep upstream's change, then re-apply Marley's hunk as its
  touchpoints row describes. `Cargo.lock` takes upstream's and lets cargo re-resolve. After that:
  build, fix what upstream's API changes broke in the Marley crates, run the full gate, the golden
  set, and install.

## Phase 2 — Code
- **Conflicts resolved** (each re-applies the hunk its touchpoints row describes on top of
  upstream's change):
  - `agent_servers/src/acp.rs`: upstream rebuilt `session/new` around `prepare_request`. The
    `.meta(meta)` moves onto it (#683); load and resume kept theirs.
  - `markdown/src/markdown.rs`:
    - Marley's `code_block_action` field, default and setter sit beside upstream's new
      `input_focus_handle`.
    - The action now reads the block's text from upstream's `current_code_block_text` (#530).
  - `migrator`: upstream's 2026-09-16 and 09-29 migrations run before Marley's
    `move_rusty_tools_into_rusty` (#643).
  - `terminal/src/terminal.rs`:
    - `new_term` takes upstream's new `generation` (0) with Marley's starting bounds.
    - `mouse_move`, `mouse_drag` and `mouse_down` take upstream's `MouseInputMode` with
      Marley's `marley_local` position (#631).
  - `terminal_view/src/terminal_element.rs`:
    - Upstream's per-button mouse handlers. The left release still reaches a terminal whose
      press the prompt editor took (#631), and the link menu's handler follows with the mode
      (#579).
    - Upstream's loop now covers the middle button, so Marley's middle-button handler goes.
    - Marley's block tests close before upstream's new read-only test.
    - Prepaint reads the alternate screen from `last_content.mode`, since upstream stopped
      destructuring `mode`.
  - `terminal_view/src/terminal_view.rs`: upstream's read-only check comes first, then Marley's
    path drop for agents (#536).
- **Upstream API changes fixed in the Marley crates:**
  - `ContextServer::stdio` takes a stdin prefix, in fleet providers, the harness and Rusty.
  - `LanguageModelRegistry::default_model` gives a `LanguageModel` whose `provider_id` goes
    through `provider()`.
  - Permissions answer by `PermissionRequestId`: `authorize_permission_request` with
    `permission_request_for_tool`.
  - `AcpThreadEvent::Stopped { .. }`.
  - `ToolCall::kind()` is ACP v2's `ToolKind`.
  - `linked_worktree_short_name` takes a `PathStyle`.
  - The test for `ZedListener { sender, generation }`.
  - `toml_edit`'s `display` feature comes from `marley_workbench`'s own manifest, since
    upstream's workspace dropped it.
- **`Cargo.lock`:**
  - Taking upstream's file reverted Marley's own pins, wasmtime 48.0.5 back to 48.0.1 with five
    RUSTSEC advisories, which gate:7 caught.
  - The fix is Marley's lockfile from HEAD (`git checkout HEAD --`; `--ours` gave back the
    staged upstream file), with cargo adding upstream's new dependencies. Audit is clean.
- **Tooling fixed for merges** (a gap, not a bypass):
  - `lib-hook-helpers.sh` `upstream_base` takes the upstream `MERGE_HEAD` as the fork point
    while it is being merged. The ledger then judges only Marley's changes: 58 rows, none
    missing or stale.
  - `gates.sh`: gates 12 and 13 count a line as added only when it is new against both sides
    of an upstream merge. Upstream's new `unsafe` and `#[allow]` lines in gpui and elsewhere had
    been read as Marley's.
- **Gate:**
  - Run 1 (`scratchpad/695-gate-1.log`) was red on gates 2, 7, 12, 13 and 21. The causes are
    above.
  - Run 2 (`695-gate-2.log`) was red on gate:2 only: a redundant clone in Marley's link-menu
    handler, which is now the terminal's last use since the middle-button handler is upstream's.
  - Run 3 (`695-gate-3.log`): **GATE GREEN [diff]**, 17 passed.

## Phase 3 — Test
- **Build:** `just build`, debug, Zed v1.25.0 with Marley on top.
- **Golden set, run 1 (`695-regress-1.log`):** 30 of 53 failed. The installed pre-merge build
  failed the same 30 for the same reasons (`695-regress-baseline.log`). The common cause was the
  runner, not Marley:
  - `script/e2e.sh` copies the user's `settings.json`, and the user's `ui_font_size` is 24 (since
    2026-10-06).
  - The scenarios' click points are measured at the default 16. At 24, every row and button sits
    lower, so a click on the rail's + missed (shots 500-01 on 2026-10-01 and today, side by side).
  - Fix: the copy now drops `ui_font_size`, `ui_font_family`, `buffer_font_size`,
    `buffer_font_family`, `agent_ui_font_size`, `agent_buffer_font_size` and the terminal's
    `font_size` and `font_family`, so a run uses the defaults the points were measured at.
- **Golden set, run 2 (`695-regress-2.log`), with that fix:** 12 of 53 failed. The 12 were run on
  the installed pre-merge build (`695-regress-base-2.log`):
  - **Failed on both builds at the same check (pre-existing, not in scope):** 510, 516, 568, 569,
    585 and 587. 508 also fails on both builds at "the terminal read nothing but the scenario's
    Enters", once the menu fix below lets it get that far.
  - **Passed before the merge, failed after:** 500, 503, 532, 561 and 579.
- **The merge's cause for 500, 503, 508, 532 and 579 is upstream's menu change:**
  - Zed #64365 (9183ba4c55) stops `ContextMenu` choosing its first entry on open unless the
    window's accessibility is enabled.
  - `main.rs` builds the app with `Application::new_inaccessible` unless `ZED_EXPERIMENTAL_A11Y=1`,
    so every Marley menu, the user's included, now opens with nothing chosen. Down chooses the
    first entry, and Enter with nothing chosen closes the menu.
  - Upstream means this; Marley keeps it. Each scenario that counted Downs from a chosen first
    entry now presses Home (`menu::SelectFirst`, bound for every context) first: 500, 503 (two
    menus), 508, 532 (`plus_entry`) and 579 (`choose`).
  - Rerun (`695-regress-3.log`): 500, 503, 532 and 579 pass; 508 reaches its pre-existing failure.
- **561 fails on the merged debug build, twice, at "the shell's own browser got the URL":**
  - Shots 561-06: `cd …/repo` ran as `cd …/repoBROW` and then `cd …/repoBROWS`. The next
    command's first characters went into the prompt before its Enter was handled, about 1.1 s
    late (`settle 1` plus 20 ms a character).
  - It passed on the installed pre-merge release build, and it passes on a release build of
    the merged tree (`695-regress-561-release.log`). So the debug build's speed causes it, not the
    merge.
- **Golden set, run 4, on the merged release build (`695-regress-4.log`):** 46 of 53 pass. The 7
  that fail are the pre-existing 508, 510, 516, 568, 569, 585 and 587, each at the same check as on
  the pre-merge build. They are pre-existing and not in scope; they go to the testing phase at the
  end.
- **`just shot 695-merged`** (release build, headless sway; `shots-695/695-merged.png`): the
  merged build starts in the Marley layout, with the projects rail, a terminal tab in the center
  and the right panel. Zed's trust prompt shows over a new project, as before.
- **Gate after the scenario and runner fixes (`695-gate-4.log`):** GATE GREEN [diff], 17 passed.

## Phase 4 — Complete
- **Documented:** a `CHANGELOG.md` entry, "Merged upstream Zed through 2026-10-08". The rows of the
  conflicted files were checked against what ships: each still describes it, and gate:16 finds
  58 rows with none missing or stale.
- **Knowledge appended:**
  - F-claude-695-taking-upstreams-lockfile-dropped-marleys-security-pins-001
  - F-claude-695-the-users-font-size-moved-every-scenarios-clicks-001
  - F-claude-695-scenarios-counted-on-a-menu-opening-on-its-first-entry-001
  - PR-claude-695-an-upstream-merge-keeps-marleys-lockfile-001
  - PR-claude-695-a-setting-that-moves-the-layout-stays-out-of-test-runs-001
  - PR-claude-695-a-scenario-chooses-a-menu-entry-from-home-001
  - L-claude-695-merging-upstream-zed-001
- **Brain:** no decision recorded. The merge carries no durable choice beyond the prevention rules
  above.
