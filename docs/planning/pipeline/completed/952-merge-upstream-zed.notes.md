# Merge upstream Zed through 2026-11-19 — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-952-merge-upstream-zed.md
- **Pipeline spec:** 952-merge-upstream-zed.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-11-19: "lets bring down latest zed updates into our fork. its been a
  while. fix any conflicts".
- **Classification:** chore, upstream sync, after #695, #797 and #950.
- **Trial merge (2026-11-19):** `git fetch upstream main --tags` gives `3d3bd04fad` (2026-11-19
  16:16 UTC). The merge base is `0373366f43` (#950's): 266 commits, 715 files (68,832 lines in,
  17,637 out). New tags up to v1.29.2 and v1.30.1-pre. The trial ran in a scratch worktree
  (removed after): eleven conflicted files, 75 files changed on both sides (list in this session's
  `scratchpad/both-sides.txt`). The toolchain pin and Zed's alacritty rev did not move;
  tree-sitter's rev did, and async-task's source moved to `smol-rs`, which `deny.toml` already
  allows. Three new upstream crates: `language_model_selector`, `project_bookmarks`,
  `sandbox_linux`.
- **The conflicts:**
  - `.rules` (modify/delete): upstream's `a57e2fdc38` (#66716) moved it to a real `AGENTS.md`
    (the same text, four `.rules` mentions renamed), deleted `CLAUDE.md` and `GEMINI.md`, and
    moved `docs/.rules` to `docs/AGENTS.md`. Marley's 110 lines were two sections: "Marley" on
    top, Google's Rust style after Zed's Rust coding guidelines. D3 and D4.
  - `Cargo.lock`: PR-claude-695, Marley's kept.
  - `agent_servers/src/acp/v1.rs` (3): upstream reads `default_config_options()` from the settings
    enum, and sends initial config options with a new session when the agent supports them. Keep
    upstream's lines; Marley's default mode (#779) wraps upstream's `default_mode()` the way it
    wrapped the old expression; Marley's `meta` (#683) goes on the request upstream builds, as a
    `let request = request.meta(meta);` after upstream's config-options branch.
  - `agent_ui/src/config_options.rs`: upstream's new `response_controls` beside Marley's four
    accessors (#907, #915): keep both.
  - `agent_ui/src/conversation_view/thread_view.rs`: upstream's toolbar now draws the config
    options' response controls, falling back to fast mode and thinking without them. Keep
    upstream's `.map(…)` and add Marley's panel-width and text-width children before it.
  - `command_palette/src/command_palette.rs`: `match_strings_async` takes `StringMatchOptions`.
    Keep upstream's call; Marley's `case` goes in as the options' field if it has one, else the
    build says how Marley's case reaches it.
  - `git_ui/src/branch_picker.rs` (2): imports (upstream dropped `AnyElement` and `TaskExt`; keep
    what Marley's hunks use), and the placeholder: upstream's new select-only arm, Marley's board
    words arm (#942) beside it.
  - `picker_preview/src/picker_preview.rs` (3): upstream replaced `current_path` with
    `_worktree`; keep upstream's field and Marley's `marley_lines` (#921) beside it, and Marley's
    count where the buffer is set.
  - `recent_projects/src/recent_projects.rs` (2): upstream moved `RecentProjectEntry` out of the
    file; keep Marley's `MarleyRecentRow`, `MarleyRecentLook` and `marley_recent_look` (#854), drop
    the old struct. In the group row, upstream's `path_style` line and Marley's look after it.
  - `terminal/src/terminal.rs` (2): upstream's free `history_size(term)`; Marley's block folder
    (#619) on `process_hyperlink`, as its row already describes.
  - `workspace/src/workspace.rs` (3): `restoring_workspace` is `Rc<Cell<bool>>` with two new
    fields; Marley's `opened_from_saved_state` beside them. Upstream adds a pass that puts items
    opened while restoring back into the restored pane; Marley's `Event::CenterRestored` goes after
    it, so a listener sees the center as restored.
- **Removed or renamed symbols the Marley crates call** (the sweep #950's lesson asks for: every
  `pub` item a `-` line removes and no `+` line adds, grepped in `crates/marley_*`):
  - `ProjectDiff::deploy_at_project_path`, removed as dead code (`0c5af8c563`, #67149):
    `review_tools.rs:191`, `guide_tab.rs:167`. D6.
  - `Case::smart_if_uppercase_in`, removed (`36acea54c5`, #67151): `rusty/page_picker.rs:377`.
    The four-line choice moves into the caller.
  - `ConversationView::root_thread_has_pending_tool_call` renamed `root_thread_is_waiting_for_input`
    (`270df71b8d`, #66708), which also counts an open elicitation form: `tab_tiles.rs:145`,
    `thread_activity.rs:50`. Marley's tiles then show waiting for a form too, as Zed's sidebar does.
  - False hits: `is_disconnected` (a test network's method went; `Project::is_disconnected`
    stays), and common names (`show`, `model`, `scale`, `dismiss`, `select_last`).
- **Merge checklist on upstream's side:** `register_sidebar(` has the same five callers outside
  the Marley crates; the `workspace::Sidebar` trait gains no method; the default keymaps change
  no sidebar or multi-workspace binding; nothing in `keymap_file.rs` touches `reload_keymaps`.
  D1 and D7 of `workbench-shell.md` hold as far as the source shows; Code re-checks them merged.
- **Recall (§18.3):**
  - L-claude-950-merging-upstream-zed-001: upstream wins a conflict, additions only; grep for
    removed symbols before the build; prose in a Bash command can trip the commit-gate hook.
  - PR-claude-695-an-upstream-merge-keeps-marleys-lockfile-001: `git checkout HEAD -- Cargo.lock`,
    build, then `cargo audit`.
  - PR-claude-950-a-text-match-filter-matches-the-same-kind-of-line-001 and
    PR-claude-950-a-lint-nightly-gap-in-upstream-code-is-closed-in-the-gate-001: in force.
  - The brain (`rusty-cli brain ask`, consultation `4f392a5a75454acd835d846e4c6761c5`): nothing
    on upstream merges or on the rules file; only decisions due for follow-up.
- **Checklist (no TaskCreate in this harness):** pick ✓, pre-flight ✓ (cargo idle, no active
  pipeline, README marker present, #946's queued pair left queued), recall ✓, mint ✓ (#952; no
  BACKLOG row, picked by name), prior art ✓, spec ✓, design ✓.

### Design
- **Branch:** `marley/merge-upstream-2026-11-19` from `marley/workbench-shell` at `cacf1f61d8`;
  `git merge --no-ff --no-commit upstream/main` at `3d3bd04fad`; at Complete,
  `marley/workbench-shell` fast-forwards to the merge commit.
- **Before resolving:** count `Marley:` per Zed file on `HEAD`, for the check after.
- **Conflicts:** as listed above, each against its touchpoints row.
- **The rules file (D3, D4):** `AGENTS.md` from upstream, Marley's two sections inserted where
  they were; `git rm .rules`; `ln -s AGENTS.md CLAUDE.md`. `CONSTITUTION.md` and
  `.claude/commands/pipeline/{plan,code,complete}.md` say `AGENTS.md` where they say `.rules`.
  The touchpoints row for `.rules` becomes `AGENTS.md`, and `CLAUDE.md` gets a row.
- **The diff entry point (D6, Marley crate only):** one `pub(crate)` helper in
  `crates/marley_workbench/src/review_tools.rs` that calls `ProjectDiff::deploy_at(workspace,
  None, window, cx)`, then finds the workspace's `ProjectDiff` and calls
  `move_to_project_path`. `guide_tab.rs` calls it. `deploy_at` emits the telemetry event with
  source "Action" where the removed function said "Agent Panel"; accepted, since Marley's build
  sends no telemetry of its own.
- **After the merge:** `rg "Marley:" crates assets` against every touchpoints row and the counts
  taken before; each of the 75 both-sides files read for a Marley hunk a clean merge displaced;
  `cargo check -p zed`, then the API breaks fixed in the Marley crates; the "At an upstream merge"
  steps (the base in `lib-hook-helpers.sh` and the touchpoints doc, `.cargo/audit.toml`,
  `deny.toml`, the workbench traps D1 and D7, `test_action_namespaces`, `register_sidebar(`
  callers, the `workspace::Sidebar` trait); `script/gates.sh --diff`.
- **File manifest:**
  - Zed paths: the nine conflicted Zed crate files (resolution only); `Cargo.lock`; `AGENTS.md`
    and `CLAUDE.md` (with rows); `.rules` deleted.
  - Marley crates: `crates/marley_workbench/src/{review_tools,guide_tab,tab_tiles,thread_activity}.rs`,
    `crates/marley_workbench/src/rusty/page_picker.rs`, and whatever else the build names.
  - Marley docs and tools: `docs/marley/zed-touchpoints.md`, `.claude/hooks/lib-hook-helpers.sh`,
    `CONSTITUTION.md`, `.claude/commands/pipeline/{plan,code,complete}.md`; maybe `deny.toml` and
    `.cargo/audit.toml`; `CHANGELOG.md` at Complete.

### Visual check plan
- REQ-001 and REQ-002: the review and the gate, not a shot.
- REQ-003: the review (`ls -l CLAUDE.md`, `git diff upstream/main -- AGENTS.md` all `+` lines).
- REQ-004: `733-guided-review.sh` (sway): `733-02-chip`, the project diff at `alpha.txt`.
- REQ-005: `730-agent-findings-as-diff-notes.sh` (sway): `730-02-proposed`, the diff with the two
  proposals.
- REQ-006: `921-file-finder-as-the-board.sh` (`921-01-rows`, "src/rail.rs · N lines"),
  `854-open-recent.sh` (`854-01-recent`), `907-bottom-row-for-a-thread.sh` (`907-01-working`).
- REQ-007: `just shot 952-merged`.
- Each scenario runs first on the installed pre-merge build (`cacf1f61d8`, the branch's base, per
  `~/.local/share/marley/install.json`), then on a release build of the merged tree copied to the
  scratchpad. A check counts against the merge only when it passes before and fails after.
- Not reached by a scenario: #683's session meta, #779's default mode, #915's Plan / Act switch,
  #942's placeholder, #619's block links and `CenterRestored`'s listeners; the review checks them
  against their rows, as #797 and #950 did.

### Risks
- **Upstream's response controls beside Marley's bottom row.** Upstream's toolbar now draws an
  agent's config options as response controls; Marley's bottom row (#907) also shows the model,
  effort and mode from them. The two could show the same choice twice. 907's shots show it. If
  they do, the fix belongs to Marley's look, not to upstream's toolbar (D5), and may be its own
  ticket.
- **Restoring is reworked.** `restoring_workspace` is shared through `Rc<Cell<bool>>` and items
  opened while restoring are put back into the restored pane. `CenterRestored`'s one listener is
  Home's page (`home_page.rs`, #701, #794), which fills a restored center that has no tab; emitted
  before upstream's pass, Home could go into a pane upstream then fills, so the emit goes after
  it. A terminal opened during the restore (Marley's first terminal) can now come back as an item
  in the restored pane beside the saved tabs. `just shot 952-merged` shows the center after a
  start.
- **Clean merges can drop a Marley hunk.** 75 files changed on both sides; each is read.
- **The rules file lives in two places for agents.** Zed's agent and Codex read `AGENTS.md`;
  Claude Code reads `CLAUDE.md`. One link keeps a single text.

## Phase 2 — Code
- **Merge:** on `marley/merge-upstream-2026-11-19` from `cacf1f61d8`, `git merge --no-ff
  --no-commit upstream/main` (`3d3bd04fad`); the eleven conflicts the trial named. `Marley:`
  markers counted per Zed file before (109 files, `scratchpad/markers-before.txt`).
- **The rules file (D3, D4):** `AGENTS.md` is upstream's file with Marley's two sections where
  `.rules` had them (`git diff upstream/main -- AGENTS.md`: 110 lines in, none out); `.rules`
  removed; `CLAUDE.md` a link to `AGENTS.md`; `GEMINI.md` and `docs/.rules` gone as upstream has
  them. `CONSTITUTION.md` and the three pipeline commands say `AGENTS.md`. Touchpoints: the
  `.rules` row became `AGENTS.md`'s, and `CLAUDE.md` has a row.
- **Conflicts resolved under D5** (upstream's lines verbatim, Marley's added beside them):
  - `Cargo.lock`: `git checkout HEAD -- Cargo.lock` (PR-claude-695); cargo added upstream's.
  - `agent_servers/src/acp/v1.rs`: upstream's `set(default_mode, default_config_options())`
    unchanged; Marley's mode (#779) is now an `if let` after it that replaces the mode, where it
    used to wrap upstream's expression. Upstream's initial-config-options branch unchanged, then
    `let request = request.meta(meta);` (#683). One marker more (a "see `meta` above").
  - `agent_ui/src/config_options.rs`: upstream's `response_controls`, then Marley's four
    accessors (#907, #915).
  - `agent_ui/src/conversation_view/thread_view.rs`: Marley's panel- and text-width children,
    then upstream's response-controls `.map(…)`.
  - `command_palette/src/command_palette.rs`: upstream's `StringMatchOptions`, with Marley's
    `case` as one added field line (#920); upstream's default is `Smart`, Marley's other arm.
  - `git_ui/src/branch_picker.rs`: upstream's imports plus Marley's `AnyElement`; upstream's
    select-only placeholder arm first, then Marley's board words (#942), so a select-only
    picker keeps its own words.
  - `picker_preview/src/picker_preview.rs`: upstream removed `current_path` (its own field,
    which only Marley's header read). **Deviation:** Marley keeps its own `marley_path` beside
    `marley_lines` (#921), set in `update_from_buffer`, and the `RelPath` import back.
  - `recent_projects/src/recent_projects.rs`: upstream removed `RecentProjectEntry`; Marley's
    `MarleyRecentRow`, `MarleyRecentLook` and `marley_recent_look` kept, the old struct gone;
    upstream's `path_style` line, then Marley's look. `chrono`'s import back as a Marley line
    (upstream dropped it with the struct; the crate still depends on it).
  - `terminal/src/terminal.rs`: upstream's free `history_size(term)`, Marley's block folder
    (#619) on `process_hyperlink`.
  - `workspace/src/workspace.rs`: upstream's `Rc<Cell<bool>>` restore fields and Marley's
    `opened_from_saved_state` beside them; `Event::CenterRestored` after upstream's pass that
    puts items opened while restoring into the restored pane.
- **Hunks kept:** marker counts match per file except the two cross-reference markers added
  (`acp/v1.rs` 7→8, `command_palette.rs` 23→24). Of the lines Marley added to the 75 both-sides
  files, every one is in the merged file except those replaced above (the old default-mode wrap
  and request line in `v1.rs`, `branch_picker.rs`'s import line with `TaskExt`,
  `picker_preview.rs`'s `current_path` lines).
- **Merge checklist:** the base moved to `3d3bd04fad` in the touchpoints doc and
  `UPSTREAM_BASE_FALLBACK`. `register_sidebar(` has the same five callers; the `Sidebar` trait
  gains nothing; the keymaps move no sidebar binding (D1, D7 hold). `deny.toml` needed nothing
  (gate:8 green; async-task's `smol-rs` is allowed).
- **Upstream API changes fixed (`cargo check -p zed`, four runs):**
  - Zed crates, inside Marley hunks: `file_finder.rs`'s look match gains upstream's
    `Match::Absolute` beside the other rows with no project path (they keep Zed's look; row
    updated); `branch_picker.rs` reads `branch.upstream…tracking.status()` (upstream removed
    `Branch::tracking_status`, #67149); `config_options.rs` gets a private
    `marley_first_select_id` for the three accessors (upstream removed
    `first_config_option_id_matching` in its composer rework, #67190).
  - `marley_workbench`: `ProjectDiff::deploy_at_project_path` → `review_tools::open_project_diff_at`
    (D6), used by the guide too; `Case::smart_if_uppercase_in` → `StringMatchOptions` default
    (`Smart` ignores case without a capital, the removed helper's choice);
    `root_thread_has_pending_tool_call` → `root_thread_is_waiting_for_input`;
    `prompt_for_open_path` takes an `OpenPathPromptMode` and returns `OpenPathPromptResult`
    (five choosers take `SelectPath` and read `.paths`); `open_workspace_by_id` takes
    `keeps_saved_state_while_restoring` (`false`, the old behavior, for a projectless group's
    reopen); `agent_client_protocol::schema::v2` → `v2::schema` (ACP SDK 5.4); `ToolCallStatus::Paused`
    is an open call in `agent_activity.rs`; `PageElement`'s hand-written `IntoElement` removed
    (gpui's blanket impl, #64864); `CommitView::open` takes `CommitViewOptions`;
    `deploy_branch_diff_with_base_ref` takes a `branch_diff`; `Plan::entries()` is a method.
- **Review of the diff:** the helper's `items_of_type` iterator held `cx` across the update
  (a borrow error, bound first); a missing `;` (clippy). Google style: the new `bool` argument
  carries its name at the call; no tuple, no abbreviation; `marley_first_select_id` is private
  with a `Marley:` note. No panic on an input path, nothing from Warp, no Zed function body in a
  Marley crate (D6 calls upstream's public API).
- **Gate run 1 (`scratchpad/952-gate-1.log`):** red on gates 2, 7 and 21.
  - gate:2 and gate:21: `rail_tests.rs` still named `schema::{v1, v2}` (only the test build
    reaches it), and the `;`.
  - gate:7: five new advisories, all in upstream's lockfile at the same versions. ammonia
    4.1.9, quinn-proto 0.11.19 and quinn-udp 0.5.16 are patch releases in range: taken with
    `cargo update -p … --precise`. libsqlite3-sys 0.30.1 is upstream's workspace pin and the fix
    is 0.39: listed as inherited. Regenerating the list (`cargo audit --json` without the config)
    dropped seven ids the tree no longer matches (rkyv ×2, rustls 0.23.40, zip 0.6.6 ×3,
    webpki-roots).
- **Gate run 2 (`952-gate-2.log`):** **GATE GREEN [diff]**, 17 passed.
- **Checklist:** merge ✓, eleven conflicts ✓, rules file ✓, API breaks ✓, audit list ✓, ledger
  and base ✓, review ✓, gate ✓.

## Phase 3 — Test
- **Build:** `cargo build --release -p zed --bin marley` (16m 29s), hard-linked to
  `scratchpad/952-build/marley` so a later build cannot replace it mid-run (a copy would take
  2.4 GB on `/mnt/fast`).
- **Baseline:** the five scenarios on the installed pre-merge build (`cacf1f61d8`, the branch's
  base), `SHOT_DIR=scratchpad/base/<name>`. 921, 854 and 907 pass as written. 733 and 730 failed
  before their criterion's shot: the ask's "Allow for This Session" now sits at (1460, 929) where
  the scenarios click (1225, 933), between Deny and "Always for This Project", and 733's chip
  `alpha.txt:2` at (340, 306) where it clicks (333, 278). **Pre-existing — not in scope**: the
  click points predate the ask's and the guide's current layout. Both ran with
  `ALLOW_X=1460 ALLOW_Y=929` (733 also `CHIP_X=340 CHIP_Y=306`) and `STOP_AT` at the criterion's
  shot, before and after alike.
- **Red, and the fix (`script/e2e.sh`):** the merged build's first runs drew the run's window
  beside two or three others holding copies of Chad's session (his rail's own groups),
  from the run's copy of his database. Upstream's `f99c53f4f6` (#65463) restores the last
  session when a start opens a path, which every scenario's first launch does; before it, a path
  start restored nothing. The tiled window was narrower, so 733's chip click landed in another
  window and 730's diff drew unified without its proposals. Deleting `session_id` from the copy
  was not enough: with no last session, `restorable_workspace_locations` falls back to the last
  workspace (Chad's most recent project came up). The copy now gets a fresh session id, which names a session
  with no windows; `last_session_workspace_locations` then finds none and nothing restores, while
  a relaunch inside a run still restores the run's own session (#794, #912). Upstream's change
  itself stays (D5): Chad's own cold `marley <path>` now brings his last session back first. No
  shot with Chad's rail was kept; the reruns replaced them.
- **Merged build** (`E2E_BINARY=scratchpad/952-build/marley`, `SHOT_DIR=scratchpad/merged/<name>`):
  - REQ-004, `733-02-chip`: the guide's chip opens Uncommitted Changes at `alpha.txt` (2 files,
    +2 −2, the cursor at 1:1 in alpha's excerpt), the Guide tab beside it, as the baseline shows.
  - REQ-005, `730-02-proposed`: the diff at `notes.txt` with "Ten should stay in lower case" on
    line 10 and "Twelve may stay as it is" on line 12, each from scenario-lint with Keep and Drop,
    as the baseline shows.
  - REQ-006: `921-01-rows`, the finder's rows with their tiles, M and U badges and the preview
    headed "src/rail.rs · 1 201 lines", as before. `854-01-recent`, This Window's repo with its
    mark and "this window", the recent projects with their marks and ages, as before.
    `907-01-working`, the bottom row "Marley · Opus 5.5 · High · Act · Working · Edit notes.md ·
    7 s · Context 48k of 200k · 1 file +4 −0 · Full width", as before. The composer above it is
    upstream's new layout (#67190): the thought level ("High") is a response control by Readable
    column, the mode and model selectors on the right; Marley's Plan / Act switch is unchanged.
    No choice shows twice, so the risk the plan named did not happen.
  - REQ-007, `just shot 952-merged` (`COMPOSITOR=sway`, `scratchpad/shots-952/952-merged.png`):
    the Marley layout, the rail with Default and the repo's group and its bash row, the terminal
    at its prompt in the center, and Zed's trust prompt over the new project, as after #950.
- **Focus:** every run reported one Marley window on Hyprland before and after, no rule added;
  sway stopped with the run's Marley.
- **Not reached by a scenario:** #683's session meta, #779's default mode, #942's placeholder,
  #619's block links, `CenterRestored`'s listener, `Match::Absolute` rows; the review in Phase 2
  checked them against their rows.
- **Gate:** `script/e2e.sh` changed after gate run 2; the gate runs again at Complete before the
  commit.
- **Checklist:** REQ-001 to REQ-003 (Phase 2), REQ-004 to REQ-007 above.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`, "Upstream Zed through 2026-11-19" (each upstream claim read in
  its commit; the Linux sandbox left out, since its release note says it changes nothing yet).
  `docs/marley_architecture/marley_workbench.md`: the guide's chip and `place` go through
  `open_project_diff_at`. The touchpoints doc: the base moved to `3d3bd04fad`; the `.rules` row
  became `AGENTS.md`'s and `CLAUDE.md` has a row; the `acp/v1.rs`, `branch_picker.rs`,
  `file_finder.rs`, `picker_preview.rs` and `workspace.rs` rows describe the merged hunks (gate:16
  green). `CONSTITUTION.md` and the pipeline commands say `AGENTS.md`. `.cargo/audit.toml`
  regenerated. No plan slice moves: an upstream sync belongs to no slice.
- **Knowledge appended:**
  - F-claude-952-a-scenarios-first-start-restored-the-users-session-001
  - PR-claude-952-an-upstream-merge-reads-the-startup-changes-against-the-harness-001
  - L-claude-952-merging-upstream-zed-001
  - AD-claude-952-claude-md-links-to-upstreams-agents-md-001
- **Brain:** `brain decide` on consultation `4f392a5a75454acd835d846e4c6761c5`:
  `decisions/marleys-agent-rules-live-in-upstreams-agentsmd-with-claudemd-a-link-to-it`,
  follow-up by 2026-12-19.
- **Gate:** run 3 before the commit, for `script/e2e.sh` and the docs changed since run 2.
- **Commit:** `76148f22b3`, the merge (parents `cacf1f61d8` and `3d3bd04fad`), after gate run 3
  (**GATE GREEN [diff]**); the range scanned for private content (two project names in the
  notes and a failure block were taken out before the push); `marley/workbench-shell`
  fast-forwarded and pushed (`cacf1f61d8..76148f22b3`).
- **Install, a red:** `script/install-marley` failed at the remote server: upstream's
  `2c2f737ab2` named the binary `remote-server`, and the script stripped `release/remote_server`.
  Fixed in the script (F-claude-952-the-install-looked-for-the-remote-server-by-its-old-name-001;
  PR-claude-952 now covers Marley's scripts' build outputs too), committed on its own, and the
  install run again.
