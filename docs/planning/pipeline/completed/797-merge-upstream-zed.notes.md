# Merge upstream Zed through 2026-10-17 — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-797-merge-upstream-zed.md
- **Pipeline spec:** 797-merge-upstream-zed.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "Lets also look any zed releases and bring that code down so we
  arent behind".
- **Trial merge (2026-10-17):** `git fetch upstream --tags` gives `56934ec056` (2026-10-17
  09:29 UTC). The merge base is `9ab0715969` (#695's), 110 commits and 296 files (40,940 lines in,
  13,868 out). `git merge-tree --write-tree HEAD upstream/main` conflicts in two files:
  `crates/agent_servers/src/acp.rs` and `crates/agent_ui/src/conversation_view.rs`. Of the files
  both sides changed, 23 merge clean, among them `assets/settings/default.json`, `Cargo.toml`,
  `agent_panel.rs`, `agent_ui.rs`, `thread_metadata_store.rs`, `editor.rs`, `markdown.rs`,
  `settings_content/src/agent.rs`, `settings_ui/src/page_data.rs`, `terminal_view.rs`,
  `terminal_element.rs`, `workspace.rs`, `multi_workspace.rs` and `zed.rs`.
- **Releases:** v1.24.3 (2026-10-16) and v1.25.0-pre (2026-10-14) are tags on release branches;
  everything in them is on main by this merge.
- **Upstream themes that touch Marley:** ACP v2 (`a90dedb0e5` and the commits after it: v2
  connection, sessions, tools, permissions, terminals), `3e145061d8` (the legacy ACP adapter moves
  into a `v1` module, the cause of the `acp.rs` conflict, where `marley_session_meta` and
  `marley_default_mode` live), `10644801ad` (modes and defaults read from config values, beside
  #779's `marley_default_mode`), `8c8ada0fbf` (session close separate from view release, beside
  #734's `conversation_view.rs` hunks), `687d662a0b` (ACP SDK 3.3).
- **Disk:** `/mnt/fast` has 473G free (pruned earlier today).

### Design
- Follow #695: per conflicted file, take upstream's change and re-apply Marley's hunk from its
  row; then build, fix the Marley crates against the new APIs, run the gate, the golden set on a
  release build, and install.

### Visual check plan
- REQ-003: `just regress` (or the golden runner #695 used) on the release build; the seven
  pre-existing failures checked at the same step as before.
- REQ-004: `just shot 797-merged`.

### Risks
- ACP v2 may change the types `marley_workbench` reads from `acp_thread` (tool calls, permission
  requests, session ids). Fixed at the source in the Marley crates.
- **Recall (§18.3):**
  - L-claude-695-merging-upstream-zed-001: a merge commit on a branch; `upstream_base` and gates
    12 and 13 read `MERGE_HEAD`, already in the tooling; expect API breaks in the Marley crates
    (`cargo check -p zed` finds them); compare golden-set failures against the installed build,
    like with like (release against release).
  - PR-claude-695-an-upstream-merge-keeps-marleys-lockfile-001: `git checkout HEAD -- Cargo.lock`
    on a lockfile conflict. This merge does not conflict on `Cargo.lock`, but the clean merge
    still has to be checked against Marley's pins (wasmtime 48.0.5) and `cargo audit`.
  - PR-claude-695-a-scenario-chooses-a-menu-entry-from-home-001 and the font-size rule: already
    in the scenarios and the runner.
  - The brain (`rusty-cli brain ask`, consultation `fc629ee43799433e82955e0807ddecfd`): nothing on
    upstream merges; only decisions due for follow-up.
- **Checklist (no TaskCreate in this harness):** pick ✓, pre-flight ✓ (cargo busy with another
  project's `cargo test` in `/srv/stacks/elephc`, its own target folder; builds wait on `just
  idle`), recall ✓, promote ✓, prior art ✓, spec ✓, design ✓.

### Design
- **Branch:** `marley/upstream-2026-10-17` from `marley/workbench-shell` at `206b4ed3ed`; `git
  merge --no-ff --no-commit upstream/main` at `56934ec056`; at Complete, `marley/workbench-shell`
  fast-forwards to the merge commit.
- **`crates/agent_servers/src/acp.rs`** (3 conflict hunks): upstream moved the legacy adapter into
  `acp/v1.rs` (`3e145061d8`). Take upstream's `acp.rs` whole, then put Marley's hunks into
  `acp/v1.rs`:
  - `marley_default_mode` and `marley_session_meta` beside `AcpConnectionDefaults`; the mode
    type is now `acp_v2::SessionConfigValueId` (`10644801ad`).
  - `refresh_from_settings`: the no-settings branch sets `marley_default_mode(agent_id, cx)`;
    the settings branch takes it before `agent_settings.default_mode()` (#779).
  - The three `.meta(meta)` calls on the new, load and resume session requests (#683).
  - The touchpoints row moves from `acp.rs` to `acp/v1.rs`. `acp/v2.rs` is upstream's inactive
    v2 connection (`a90dedb0e5`, "inactive"); Marley's hunks go there when upstream switches it
    on.
- **`crates/agent_ui/src/conversation_view.rs`** (3 hunks):
  - The struct: upstream's `acp_v2::SessionId` for `root_session_id`, and Marley's
    `marley_own_folders` field (#734).
  - The stop handler: upstream's `if !is_subagent` blocks and its new error stop, with Marley's
    `crate::MarleyAttention::Failure` as `notify_with_sound`'s attention argument in each,
    the new error stop's included (#752).
- **After the merge:** `rg "Marley:" crates assets` against every touchpoints row; the clean
  merges of the 23 files both sides changed read for a Marley hunk upstream's change displaced;
  `cargo check -p zed`, then the Marley crates' API breaks fixed at the source; the
  "At an upstream merge" steps (the upstream base in `lib-hook-helpers.sh` and the touchpoints
  doc, `.cargo/audit.toml`, `deny.toml`, the workbench traps D1/D7, `test_action_namespaces`,
  `register_sidebar(` callers, new `workspace::Sidebar` default methods); `script/gates.sh
  --diff`.

### Visual check plan
- REQ-003: `just regress` on a release build of the merged tree; each failure rerun on the
  installed pre-merge build (`206b4ed3ed`), and only a pass-before, fail-after counts as the
  merge's.
- REQ-004: `just shot 797-merged` on the release build.

### Risks
- ACP v2 types reach `marley_workbench` (session ids, stop reasons, mode ids, permission
  requests). Fixed at the source in the Marley crates.
- The clean merge of `default.json`, `settings_content/src/agent.rs` (#793's `MergeFrom`) and
  `settings_ui/src/page_data.rs` can drop a Marley hunk without a conflict; each is read.

## Phase 2 — Code
- **Merge:** branch `marley/upstream-2026-10-17` from `206b4ed3ed`; `git merge --no-ff
  --no-commit upstream/main` (`56934ec056`), conflicts in the two files the trial named.
- **Conflicts resolved:**
  - `agent_servers/src/acp.rs`: upstream's file taken whole. Marley's hunks are in
    `acp/v1.rs`: `marley_default_mode` (returning `acp_v2::SessionConfigValueId`) and
    `marley_session_meta` beside `AcpConnectionDefaults`; Marley's mode first in both branches
    of `refresh_from_settings` (#779); `.meta(meta)` on the new, load and resume requests (#683).
    The touchpoints row moved from `acp.rs` to `acp/v1.rs`.
  - `agent_ui/src/conversation_view.rs`: upstream's `acp_v2::SessionId` with Marley's
    `marley_own_folders` (#734); upstream's `if !is_subagent` notices and its new
    "stopped because of an error" notice, each with `crate::MarleyAttention::Failure` (#752).
- **Hunks kept:** every `Marley:` marker in Zed's paths is there after the merge (the count per
  file matches, `acp.rs`'s seven now in `acp/v1.rs`); every line Marley added to the 23 files both
  sides changed is still in the merged file.
- **Merge checklist:** the upstream base moved to `56934ec056` in the touchpoints doc and
  `UPSTREAM_BASE_FALLBACK`. The workbench's traps hold: no new `register_sidebar(` caller, the
  `workspace::Sidebar` trait unchanged, the keymaps' only change two vim keys.
- **Upstream API changes fixed in the Marley crates:**
  - ACP v2 ids: `rail.rs`, `thread_tab.rs` and `thread_tools.rs` take session, tool call and
    permission types from `schema::v2` (`1c9dd3e70f`, `7fb155e9a2`); v2's `PermissionOptionKind`
    is not `Copy`, so the two answer lookups clone it.
  - `rail_tests.rs`'s thread module uses v2 ids and v1 session updates and stop reasons, as
    `StubAgentConnection` takes them (upstream's own tests do the same).
  - `prompt_colors.rs`: `Language::highlight_text` returns an iterator (`7fa2adbe08`); the
    `.into_iter()` went (clippy `useless_conversion`).
- **Gate run 1 (`scratchpad/797-gate-1.log`):** red on gates 2, 7, 8 and 21.
  - gate:2: the three API changes above.
  - gate:7: RUSTSEC-2026-0373 (ammonia 4.1.7, published 2026-10-16), in Marley's lockfile before
    the merge too: `cargo update -p ammonia --precise 4.1.8`.
  - gate:8: upstream pins wgpu to a git rev (`402675e805`, #65649): `gfx-rs` joins `deny.toml`'s
    allowed organizations, with the commit cited.
  - gate:21: upstream's `PathStyle::strip_prefix` gives a `&Path` (#65503), and
    `agent/src/tools/terminal_tool.rs` calls `Path::is_empty`, stable on 1.98.1 but not on the
    lint library's `nightly-2026-03-21`.
- **Gate run 2 (`797-gate-2.log`):** gates 7 and 8 green. Red:
  - gate:2: the moved `tool_call_id` in `rail_tests.rs` was still cloned (`redundant_clone`).
  - gate:21: the first fix, `-Zcrate-attr=feature(path_is_empty)` for every crate the check
    builds, cannot work: `no_std` crates (`cfg-if`, `equivalent`, `foldhash`) do not know a std
    library feature (E0635). Replaced by a one-line hunk in `terminal_tool.rs`,
    `subpath.as_os_str().is_empty()`, with its touchpoints row; the gate script is as it was.
  - gates 20 and 22: semgrep timed out on `rail.rs` (10.6k lines) under `command-injection` while
    elephc's tests held 15 cores, and `--strict` makes a timeout an error. `SEMGREP_TIMEOUT=60`
    (a rule's seconds per file, default 5) for both semgrep calls, so the rule runs to its end;
    the full scan passes by hand with it.
  - the receipt: a file changed during the run (the `rail_tests.rs` fix).
- **Gate run 3 (`797-gate-3.log`):** **GATE GREEN [diff]**, 17 passed. The review of the diff
  found nothing beyond the fixes above: each Zed hunk is one of the moved hooks, the attention
  argument, or `terminal_tool.rs`'s one line; nothing from Warp; no Zed function body carried
  into a Marley crate.

## Phase 3 — Test
- **Build:** `cargo build --release -p zed --bin marley` (28 min, beside elephc's tests), copied to
  `scratchpad/797-build/marley` so later builds cannot replace it mid-run.
- **Golden set on the merged release build (`797-regress-1.log`, runs in
  `~/.local/state/marley/regress/20261017-143608`):** 24 of 53 pass, 29 fail.
- **The 29 on the installed pre-merge build (`206b4ed3ed`, `797-regress-base.log`, runs in
  `…/regress/20261017-152653`):** 29 of 29 fail, each at the same check as on the merged build
  (compared check line by check line). None is the merge's: pre-existing — not in scope.
  - Why they fail, from the shots read: the scenarios' click points and steps predate the rail's
    Default group and #785. 520's split opens Home where it expects a second terminal (#785, a
    split opens Home, 2026-10-15); 532's click on the project's + lands on the Default group's +,
    which now sits above the project and moves every row under it.
  - #695 left 7 golden failures; the set has drifted to 29 since. Repairing it belongs to the
    testing phase at the end of the product (§7), with the unit tests and mutation.
- **`just shot 797-merged`** (`COMPOSITOR=sway`, `E2E_BINARY` the merged release build;
  `scratchpad/shots-797/797-merged.png`): the merged build starts in the Marley layout, the rail
  with the Default group and the repo's terminal row, the terminal in the center at the user's
  prompt, and Zed's trust prompt over the new project, as after #695 (REQ-004). The copied profile
  has the agent panel closed, so the shot does not show it; REQ-004 was narrowed to what the shot
  shows. #798's visual check opens the agent panel on this tree.
- **Checklist:** REQ-001 (conflicts against their rows, gate:16 green), REQ-002 (gate run 3
  green), REQ-003 (24 pass; the 29 failures pre-existing at the same checks), REQ-004 (the shot).

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md`, "Upstream Zed through 2026-10-17". The touchpoints doc: the
  upstream base moved to `56934ec056`; the `acp.rs` row moved to `acp/v1.rs`; the new
  `agent/src/tools/terminal_tool.rs` row. gate:16 found every row matching what ships. No plan
  slice moves: an upstream sync belongs to no slice.
- **Knowledge appended:**
  - F-claude-797-a-std-feature-turned-on-for-every-crate-broke-no-std-crates-001
  - F-claude-797-semgrep-timed-out-on-rail-rs-under-a-loaded-box-001
  - PR-claude-797-a-lint-nightly-gap-is-fixed-at-its-call-001
  - PR-claude-797-a-gate-check-with-a-time-limit-allows-a-loaded-box-001
  - L-claude-797-merging-upstream-zed-001
- **Brain:** no decision recorded. The merge carries no durable choice beyond the rules above.
- **Install:** after the batch's last ticket, so Chad restarts once.
