# Marley checks the agent's version before an untested integration turns on — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-648-agent-version-check.md
- **Pipeline spec:** 648-agent-version-check.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02: "we need to brain storm integration into claude and codex using
  their tools instead of fighting them". On B7, asked "follow the user's installed versions, or the
  harness's pins?", he answered "Your install, with a check", written up in the design note as
  "follow the user's installed `claude` and `codex`, checked against a list Marley has been tested
  on, turning off only the parts it is unsure of outside that list and saying why". This is #648,
  first of the batch #648 to #653 (B7, B4, B1 in two parts, B2's Marley half, B3), after #640 to
  #647. B1 to B3 register their own rows in the table this ticket ships.
- **Classification / tier:** feature, small to medium. One new pure module
  (`marley_agent::versions`), one new workbench module (`agent_versions`, with the chip), a gate at
  three call sites of the prompt recognition, a settings key across three Zed files whose
  `zed-touchpoints.md` rows exist, one line in the e2e harness, the scenario. No new crate and no
  new external dependency: `marley_agent` gains `semver`, already in `Cargo.lock` and in
  `marley_workbench`. One slice; no split. What the brief listed and this plan changes: the brief
  named `terminalSequence` and the tags as today's undocumented Claude path; Claude Code's hooks
  reference documents `terminalSequence` now (fetched 2026-10-03), so the tags are the one row and
  the hook channel is not gated (D1).
- **Recall (§18.3):**
  - AD-claude-482-claude-code-sends-marleys-notifications-through-a-plugin-001: "`terminalSequence`
    is in Claude Code 2.1.281's hook schema but not its public docs". No longer true; Complete
    appends the correction (the ledger is append-only).
  - AD-claude-519-claude-codes-hook-events-ride-in-band-into-marley-fleet-001: the frames, the
    fold, "harness-injected prompts keep the user's", and "the 2 s quiet timer stays the fallback
    for a terminal that sends no events". D1 keeps the frames on, so the fallback is not reached.
  - AD-claude-552-codex-configured-opencode-given-a-file-001 and its code: the agent bar's chip that
    re-reads files when a bar draws after a while; D6 and D8 copy its shape.
  - AD-claude-587-marley-brings-claude-codes-trust-question-to-the-user-001 and `trust.rs:1-9`:
    "its words have changed across releases"; the reader matches each published wording and does
    nothing otherwise, which is why the trust reader gets no row (spec, Out).
  - L-claude-531-marley-takes-its-path-from-the-login-shell-so-stand-ins-are-named-001,
    F-claude-547-a-scenarios-click-ran-the-real-claude-001 (high) and
    PR-claude-name-the-fakes-the-app-runs-001: a program Marley itself runs is named by a variable;
    hence `MARLEY_CLAUDE` (already the plugin's) and the new `MARLEY_CODEX` (D5), and a scratch
    `CLAUDE_CONFIG_DIR` in the scenario.
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001: the harness
    copies the user's settings and PATH; D11 allows the row in each copy so no scenario depends on
    the user's installed version.
  - L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001: every settings change
    in the scenario is an outside edit; the chip's click only opens the window.
  - L-claude-482-background-work-in-a-marley-crate-is-a-lazy-future-001: the stat and the run go in
    `background_spawn(futures::future::lazy(...))`.
  - L-claude-603-a-scenario-fakes-a-program-for-marley-through-setups-path-001: a variable exported
    in `setup` reaches Marley's own spawns.
  - L-claude-480-an-e2e-fake-acts-out-the-program-001 and #519's scenario: the stand-in runs the
    plugin's real `event.py`, so the frames are the ones Claude Code would write.
  - F-claude-594 (high, rich input's paste answered a permission dialog): `agent_events::waiting`
    rests on the seat, which is why D1 rejects turning the whole hook path off.
  - Completed pipelines 482, 519, 547, 552 and 587 (spec and notes); queued #642 and #643 (both
    edit `script/e2e.sh`'s settings block, see Risks).
  - Brain (`rusty-cli brain search "agent version check claude codex tested"`, read only): nothing
    on this seam.
- **Discovery** (line numbers as of 2026-10-03, `f9ef577284`):
  - `crates/marley_agent/src/marley_agent.rs`: the crate doc `:1-12`, the dylint block `:16-28`,
    the modules `:34-39`, `AgentKind` `:44`, `ALL` `:57`, `program` `:61`, `display_name` `:83`,
    `agent_kind_of` `:99`. `crates/marley_agent/Cargo.toml`: `base64`, `marley_fleet`,
    `procfs-core`, `serde`, `serde_json`; rustal's lint table.
  - `crates/marley_agent/src/claude_events.rs`: the module doc naming Orca's list `:17-20`; `fold`
    `:217`; its `UserPromptSubmit` arm `:405-422` (the continuation skipped, an injected prompt
    keeps the user's label and facts); `HARNESS_TAGS` `:710-730`; `COMPACT_CONTINUATION` `:733`;
    `HARNESS_PREFIXES` `:736-744`; `opening` `:747`; `is_harness_injected` `:758`;
    `is_compact_continuation` `:772`; `PromptOrigin` `:778`; `prompt_origin` `:795`. `fold` has 27
    call sites, most of them tests, hence D4's kept signatures.
  - `crates/marley_workbench/src/agent_events.rs`: `waiting` `:112`; `on_frame` `:132-183` (the
    foreground and remote checks `:137-143`, `decode` `:144`); `after_fold` `:188-224`, the outcome
    note behind `is_harness_injected` and `is_compact_continuation` `:200-207`.
  - `crates/marley_workbench/src/turns.rs`: `on_event` `:108`, `prompt_origin` `:135`, `title_of`
    `:191`.
  - `crates/marley_workbench/src/notifications.rs:105-122`: the `marley-event` title routed to
    `agent_events::on_frame`. `crates/marley_workbench/src/remote.rs:49`: `is_remote`.
  - `crates/marley_workbench/src/rail.rs:6759-6780`: a Claude row's status from its seat, else the
    quiet timer.
  - `crates/marley_workbench/src/agent_bar.rs`: `agent_in` `:140`; `render` `:202-287`, the left
    group with the plugin's chip `:255-258` and `agent_notify::chip` `:261`;
    `claude_plugin_chip` `:365-421` (a `Button` with a start icon and a tooltip, a debug selector).
  - `crates/marley_workbench/src/agent_notify.rs`: `FRESH` `:34`, the read and its refresh only on
    a change `:144-172`, `read_if_stale` `:177-185`, `chip(kind, context, cx)` `:189-245`.
  - `crates/marley_workbench/src/claude_plugin.rs`: `init` `:115-130` (`MARLEY_CLAUDE` `:122`),
    `set_up`'s `which::which("claude")` `:141`, `installed_plugins.json` `:181-208`.
  - `crates/marley_workbench/src/agents.rs`: `Launcher` `:57-74`, its default search path (the
    process's `PATH`) `:76-84`, `launcher` `:90`, `installed_clis` `:105`.
  - `crates/marley_workbench/src/process.rs`: the crate's one spawn module `:1-5`, `output`
    `:19-38` (an `env` list added to the child's; no `kill_on_drop`), `run_program` `:46`.
    `script/gates.sh:361` (`SPAWN_SITES_PIN=7`).
  - `crates/marley_workbench/src/marley_workbench.rs`: the modules `:23-92`, `MarleySettings`
    `:342`, `claude_code_worktree_trust` `:401` and `:637`, `from_settings` `:547`, `init` `:741`,
    `agent_notify::init` `:776`, `agent_bar::init` `:777`, `claude_plugin::init` `:778`.
  - `crates/settings_content/src/marley.rs`: `MarleySettingsContent` `:9-173`,
    `claude_code_worktree_trust` `:160-166`, `system_one` `:167-169` (its `uses` map, a
    `BTreeMap<String, SystemOneMode>`, `:282-292`).
  - `assets/settings/default.json`: the `marley` block from `:1665`; `claude_code_worktree_trust`
    and its comment `:1789-1792`; `system_one` `:1796`.
  - `crates/settings_ui/src/marley_page.rs`: `marley_page()` `:10-22`; `agents_section()`
    `:112-435`; the map item `marley.system_one.uses.check` `:771-800`.
  - `crates/zed_actions/src/lib.rs:147-156`: `OpenSettingsAt { path, target }`.
  - Zed, read for the shape: `crates/acp_thread/src/acp_thread.rs:2440-2445`
    (`LoadError::Unsupported`) and `:2457-2464` (its words);
    `crates/agent_ui/src/conversation_view.rs:2776-2800`
    (`render_unsupported`); `crates/agent_servers/src/acp.rs:1015-1016` (the protocol version
    check); `crates/extension_host/src/wasm_host/wit.rs:54-69` (`wasm_api_version_range`).
  - On this box: `~/.local/bin/claude` links to `~/.local/share/claude/versions/2.1.288`, with
    2.1.283, 2.1.284, 2.1.285 and 2.1.287 beside it (dated 2026-09-25 to 2026-10-02);
    `claude --version` prints `2.1.288 (Claude Code)` in under 10 ms. `codex` is mise's `latest`
    link to `0.155.1/bin/codex`, printing `codex-cli 0.155.1`. `gemini` is a mise shim
    (`/usr/bin/mise`). This session's own Claude Code is 2.1.280 while the path holds 2.1.288,
    which is the running-session case in Risks.
  - The tags, checked by hand: each of the 19 names in `HARNESS_TAGS` is in the 2.1.283 and 2.1.288
    binaries (`grep -a`, read and not run); 15 as a literal `<name>`, four (`agent-message`,
    `local-command-caveat`, `mcp-polling-update`, `mcp-resource-update`) only bare, with the same
    counts in both.
  - The Codex source the brief names at `/srv/stacks/rustal-codex` is not on this box; the version
    line's form comes from the run here and from the harness's pin. Not confirmed from source.
  - `script/e2e.sh:626-642` (the copy of the user's settings, `rusty_tools: false`), `:562-578`
    (`profile_setting`); `script/e2e/519-claude-code-events-in-the-rail.sh:62-103` (the stand-in
    that runs `event.py`); `script/e2e/547-claude-code-events-slice-2.sh:19-20`, `:138` (a chip's
    position kept as `CHIP_X`, `CHIP_Y`); seventeen scenarios export `MARLEY_CLAUDE`, and none of
    their stand-ins answers `--version`.
- **Decisions:** D1 to D11 in the spec. In short: only surfaces the agents do not document get a
  row, which today is the prompt tags, since `terminalSequence` is documented; the table is code in
  `marley_agent`; the tags count as tested from 2.1.283 through later 2.1 releases, a prerelease
  judged by its release; off, every prompt is the user's and nothing else of #519 changes; the
  program is the variable's, else the search path's, read again only when its file changes; the
  check runs at start and when a bar draws 10 s after the last; an unread version is untested; the
  why is the agent bar's chip with its tooltip and a click to the setting; the setting is a map
  keyed by row; remote terminals keep today's reading; the harness allows the row.

### Design
- **Marley, `marley_agent`:**
  - `Cargo.toml`: `semver.workspace = true`.
  - `src/marley_agent.rs`: `pub mod versions;` and a line in the crate doc.
  - `src/versions.rs` (new): the module doc (what earns a row, D1; the range policy, D3; the
    check's home in `marley_workbench::agent_versions`). `Range { from: semver::Version, before:
    Option<semver::Version> }` with `contains(&Version)`, which compares `Version::new(major,
    minor, patch)` of the version found, and `words()` for the tooltip (the three forms D8 names).
    `Integration { id, agent, name, rests_on, tested }`, all `&'static str` but `agent` and
    `tested`, built in `const`. `CLAUDE_PROMPT_TAGS` (id `claude_prompt_tags`, agent Claude, name
    "Prompt tags", rests on "the tags and openings of the prompts Claude Code injects, which its
    documentation does not name", tested from 2.1.283 before 2.2.0, its doc comment naming #519 and
    this plan's check of 2.1.283 and 2.1.288). `INTEGRATIONS: [Integration; 1]`,
    `integrations_of(agent)`, `integration(id)`. `parse_version(output: &str) -> Result<Version,
    String>` (D5's rule; the error quotes the first line, cut to 80 characters). `Found { Version,
    Unreadable(String), Missing(String) }`. `Verdict { On, Allowed, Off(Off) }` with `Off {
    Untested(Version), Unreadable(String), Missing(String), NotChecked }`; `verdict(integration,
    found: Option<&Found>, allowed: bool) -> Verdict`. `reasons(agent, path, found, offs) ->
    Vec<String>`: the tooltip's lines in Marley's words, shaped like Zed's `render_unsupported`.
  - `src/claude_events.rs`: `PromptReading { Recognized, AllTyped }` with `is_injected`,
    `is_continuation` and `origin`; `AllTyped` answers false, false and `PromptOrigin::User`. A
    `fold_with(seat, previous, event, now_ms, reading)` that `fold` calls with `Recognized`;
    `Moving::take` asks the reading. `is_harness_injected`, `is_compact_continuation` and
    `prompt_origin` keep their signatures as `Recognized`'s.
- **Marley, `marley_workbench`:**
  - `src/agent_versions.rs` (new): the `AgentVersions` global, per agent (Claude Code, Codex): the
    path found, its identity, the `Found`, when the last check ended and whether one runs. `init`:
    the global, then `check` for both. `check(kind)`: off the main thread, `find` (the variable,
    else `which::which_in(kind.program(), launcher.search_path, "/")`, its miss saying where it
    looked: "MARLEY_CLAUDE names <path>, which is not there", "no claude on Marley's PATH, and
    MARLEY_CLAUDE is unset"), the identity from `std::fs::canonicalize` and `metadata`, and, when it
    changed, `process::output(path, ["--version"], None, &[("PATH", <folder>:<PATH>)])` raced
    against a 5 s background timer, its stdout and stderr cut to 4 KiB, then `parse_version`. Back
    on the main thread: store it, log it, and `cx.refresh_windows()` when any row's verdict changed.
    `check_if_stale(kind)`: a new check when none runs and the last ended 10 s ago or more.
    `verdict(id, cx)`: the row's verdict from the global and `MarleySettings`'s allow map.
    `prompt_reading(view, cx)`: `Recognized` for a remote terminal and while the row is on or
    allowed, else `AllTyped`. `chip(kind, context, cx)`: `check_if_stale`, then nothing for a
    remote terminal or with no row off (`NotChecked` included); else a `Button` with
    `IconName::Warning`, the label of D8, `Tooltip::text` of the reasons joined by newlines, a
    debug selector `marley-agent-version-chip`, and a click that dispatches
    `zed_actions::OpenSettingsAt { path: "marley.allow_untested_versions.<id>", target: None }`.
  - `src/agent_events.rs`: `on_frame` gets the reading once (`agent_versions::prompt_reading`) and
    hands it to `fold_with` and to `after_fold`, whose outcome note asks it.
  - `src/turns.rs`: `on_event` takes the reading for its `prompt_origin`.
  - `src/agent_bar.rs`: `.children(crate::agent_versions::chip(agent, context, cx))` after the
    plugin's chip.
  - `src/marley_workbench.rs`: `pub mod agent_versions;`; `MarleySettings::allow_untested_versions:
    BTreeSet<String>` (the ids set true), read in `from_settings`; `agent_versions::init(cx)` before
    `agent_bar::init`.
- **Zed, `settings_content`** (`crates/settings_content/src/marley.rs`): after
  `claude_code_worktree_trust`, `pub allow_untested_versions: Option<BTreeMap<String, bool>>`,
  documented "The integrations Marley turns on even when the agent's version is outside the range
  it tested them on, by their id, such as `claude_prompt_tags` (#648). Default: {}".
- **Zed, `default.json`**: after `claude_code_worktree_trust`, `"allow_untested_versions": {}` with
  a comment: Marley reads the version of the `claude` and `codex` on its PATH and keeps off what it
  has not tested on that version, saying why in the agent bar; an id set true here turns that
  integration on anyway, for trying a new release; `claude_prompt_tags` is the tags that tell your
  prompts from the ones Claude Code adds.
- **Zed, `settings_ui`** (`crates/settings_ui/src/marley_page.rs`): `agent_versions_section() ->
  [SettingsPageItem; 2]`, chained after `agents_section()`: the header Agent Versions; Prompt Tags
  on Untested Claude Code (`marley.allow_untested_versions.claude_prompt_tags`, pick and write
  over the map as the System One uses do): "Whether Marley tells your prompts from the ones Claude
  Code adds itself (task notifications, system reminders) by their tags on a Claude Code version it
  has not checked them on. Off, it reads every prompt there as yours; the agent bar says which
  version it found and which it checked." A `// Marley:` comment on the hunk.
- **Marley, scripts:** `script/e2e.sh`: in the copy's settings, `allow_untested_versions` set to
  `{"claude_prompt_tags": true}`, with the comment (L-633, D11).
  `script/e2e/648-agent-version-check.sh` (new, Test phase).
- **File manifest.**
  - Marley: `crates/marley_agent/Cargo.toml`, `src/marley_agent.rs`, `src/versions.rs` (new),
    `src/claude_events.rs`; `crates/marley_workbench/src/agent_versions.rs` (new),
    `src/agent_events.rs`, `src/turns.rs`, `src/agent_bar.rs`, `src/marley_workbench.rs`;
    `script/e2e.sh`; `script/e2e/648-agent-version-check.sh` (new).
  - Zed: `crates/settings_content/src/marley.rs` (crate `settings_content`);
    `assets/settings/default.json`; `crates/settings_ui/src/marley_page.rs` (crate `settings_ui`);
    `Cargo.lock` (`marley_agent`'s dependency list).
- **The ledger rows** (`docs/marley/zed-touchpoints.md`, written before the code, §14):
  - `crates/settings_content/src/marley.rs` (`:59`): append "`allow_untested_versions:
    Option<BTreeMap<String, bool>>`, the integrations turned on outside the agent versions Marley
    tested them on, by id (#648)".
  - `assets/settings/default.json` (`:67`): append "`marley.allow_untested_versions: {}` with its
    comment (#648)".
  - `crates/settings_ui/src/marley_page.rs` (`:63`): append "an Agent Versions section after Agents,
    with Prompt Tags on Untested Claude Code (`marley.allow_untested_versions.claude_prompt_tags`)
    (#648)".
  - `Cargo.lock` (`:48`): its row already covers the Marley crates' entries. No new row.

### Visual check plan
The scenario `script/e2e/648-agent-version-check.sh`, `compositor sway`. Setup as the spec's UI
proof: the versions folder and the link, `MARLEY_CLAUDE`, `MARLEY_CODEX`, the scratch
`CLAUDE_CONFIG_DIR`, the HOME, the steps file; `expect` the copy holds
`marley.allow_untested_versions.claude_prompt_tags` true, then `profile_setting
marley.allow_untested_versions '{}'`. Checks read `$E2E_PROFILE/logs/Marley.log` with `holds` and
`grep -c`. A run of the stand-in is: `claude` and Enter in the project's terminal, Enter for step 1,
Enter for step 2, settle 2; Ctrl+D ends it.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001, REQ-002, REQ-005 | Trust the project; open a terminal; a run of the stand-in | `648-01-tested`: the row keeps "Add a README to the project"; no chip; the log holds both read lines |
| REQ-003, REQ-004, REQ-006, REQ-007 | Ctrl+D; `ln -sfn versions/2.2.0 bin/claude`; settle 11; a run | `648-02-untested`: the chip `Untested Claude Code 2.2.0`; the row shows the task notification; one `2.1.287` line and one `2.2.0` line in the log |
| REQ-008 | `pointer_to` the chip; settle 1 | `648-03-why`: the tooltip with the tags, the link's path, 2.2.0, "2.1.283 and later 2.1 releases", the setting's name |
| REQ-009, REQ-010 | `click` the chip; settle 2 | `648-04-setting`: Agent Versions, Prompt Tags on Untested Claude Code off |
| REQ-011 | Close the Settings window; `profile_setting marley.allow_untested_versions '{"claude_prompt_tags": true}'`; settle 2; Ctrl+D; a run | `648-05-allowed`: no chip; the row keeps the user's prompt |
| REQ-012 | `profile_setting marley.allow_untested_versions '{}'`; Ctrl+D; `ln -sfn versions/garbled bin/claude`; settle 11; a run; `pointer_to` the chip | `648-06-unreadable`: `Claude Code version unknown`, the tooltip quoting `garbled (Claude Code)` |
| REQ-005 (a later 2.1) | Ctrl+D; `ln -sfn versions/2.1.300 bin/claude`; settle 11; a run | `648-07-back-in-range`: no chip; the row keeps the user's prompt |
| REQ-017 | Setup's `expect` | its `pass` line in the run's output |

Not reached by the scenario, and why: a real Claude Code or Codex (the brief: stand-ins only; the
stand-in prints the line Claude Code prints and runs the plugin's real hook); a missing program
(REQ-013), the first check still running (REQ-014), a remote terminal (REQ-015), the 5 s bound
(REQ-012's timeout half) and the pure module (REQ-016): the review, and the gate for REQ-016.

### Risks
- **The rule for later releases is a call to confirm.** D3 accepts 2.1 releases newer than any
  checked. If Chad wants "tested" to mean a version Marley has run, the row's range closes at the
  newest checked and the chip shows after most of Claude Code's updates; the table makes either
  one line. Promotion asks.
- **A session older than the install.** Claude Code installs an update while a session runs, and
  the session keeps its old version: this session runs 2.1.280 beside a path at 2.1.288. The
  verdict follows the path, so a session can be judged by a version it does not run. The ranges
  move forward with releases, so the common case (an old session judged by a newer, still covered
  install) changes nothing; B7 part 2 can judge each session by its own version.
- **A shim hides an update.** A launcher that picks the version at run time (mise's, asdf's) keeps
  its path, size and time across updates, so the change shows at Marley's next start. On this box
  `claude` and `codex` are links, not shims.
- **A `--version` that hangs.** `process::output` does not kill on drop, so a run that loses the
  5 s race would linger. The Code phase sets `kill_on_drop` for that run, in `process.rs` beside
  `output` and through the same builder, so the spawn count stays at 7; if that cannot be done
  without a new spawn call, the pin moves in the same change (gate:22).
- **A bar that does not draw.** The recheck rides on a bar drawing, so a terminal in a window
  nobody shows reads its frames with the last verdict until a bar draws. That is the same moment
  `agent_notify` re-reads its files; nothing polls.
- **The scenario's timing.** Each move of the link waits 11 s, past the 10 s since the last check,
  before the stand-in runs; the bar then draws, the check runs (well under a second here), and the
  steps follow. A step's frames can land before the new verdict; the first prompt is then read
  with the old reading, which is why each run's injected prompt comes in its second step.
- **The settings block of `script/e2e.sh`.** #642 and #643 rewrite the same block (Voice off, Rusty
  off and its service URL); this ticket adds one key to it. Promotion re-reads it after both land.
- **Remote terminals.** D10 keeps today's reading for a remote host whose version Marley has not
  read, which is looser than D7's rule for a local version it cannot read; promotion confirms it
  with Chad.

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: the agent bar (`:506` on): the version chip, what it says and its click;
  the Claude Code plugin's part (`:802` on): the tags and what turning them off changes; the
  settings keys: `marley.allow_untested_versions`; the variables: `MARLEY_CODEX` beside
  `MARLEY_CLAUDE`.
- Architecture (§21): `docs/marley_architecture/marley_agent.md` (the `versions` module, the
  reading in `claude_events`); `docs/marley_architecture/marley_workbench.md` (`agent_versions`,
  the gate's call sites); the design note's B7 marked as started by #648; the three
  `zed-touchpoints.md` rows checked against what shipped; `CHANGELOG.md` (Added: the version check,
  the chip and the setting).
- Ledger: an AD for the table and its rule for what earns a row; the correction to AD-482
  (`terminalSequence` is documented); a lesson for checking a Claude Code binary for the tag names.

### Checklist (no TaskCreate in this harness)
- [x] Read the brief (both parts), CONSTITUTION §3, §7, §14, §18, §19, §20, and the ticket and
      pipeline templates.
- [x] Read the design note in full: the fights table, the tools, B1 to B7, Chad's answers and the
      harness's side.
- [x] Read what the ticket touches: `marley_agent` (`claude_events`, `trust`, the crate root),
      `agent_events`, `turns`, `notifications`, `agent_bar`, `agent_notify`, `claude_plugin`,
      `agents`, `process`, the plugin's `hooks.json` and `event.py`, the rail's status, the
      settings block, `default.json` and the Marley page.
- [x] Recall: the knowledge ledgers (AD-482, AD-519, AD-552, AD-587; L-480, L-482, L-531, L-603,
      L-607, L-633; F-547, F-594; PR-claude-name-the-fakes-the-app-runs-001), the completed
      pipelines 482, 519, 547, 552 and 587, queued #642 and #643, a read-only brain search.
- [x] Discovery with file:line, and on this box: the installed versions, `--version`'s output, the
      tag names in two binaries.
- [x] Prior-art sweep, three legs: Warp's, Zed's and Orca's maps and Orca's version probe; Claude
      Code's CLI and hooks references, the harness's pin docs; `semver`, `which`, `process`,
      `agent_notify`, Zed's unsupported-version callout and its extension API range.
- [x] Found and recorded the scope change: `terminalSequence` is documented, so the tags are the
      one row (D1).
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D11, seventeen EARS rows, the phase
      plan.
- [x] Design: approach, file manifest by crate, the ledger rows to widen, the visual check plan,
      risks, the docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

### Promotion (2026-10-04)
- Promoted into `active/`; the BACKLOG row removed; the ticket in-progress. Pre-flight green, no
  other active pipeline; #640 to #647 landed (the last 16e78a9944).
- **Brain:** `brain ask` (consultation `1e38bf4472a749f4814b1397de1f8ba8`) returned due follow-ups
  on other work only.
- **The hooks reference, read again** (`code.claude.com/docs/en/hooks`, 2026-10-04):
  `terminalSequence` is documented with its allowlist (OSC 0, 1, 2, 9, 99, 777 and bare BEL,
  ended by BEL or ST; anything else rejected and the field ignored), no length limit given, and
  UserPromptSubmit's input carries no version. D1 stands: the hook channel gets no row.
- **Seams re-read** (an Explore pass, every item of the discovery list): the shapes hold, at new
  lines. `marley_agent.rs`: modules `:34-39`, `AgentKind` `:43-53` (with `short_name` `:72`);
  `Cargo.toml` has no `semver` (the workspace's is `Cargo.toml:855`, `marley_workbench` already
  uses it). `claude_events.rs`: `fold` `:217`, `Moving::take` `:392` with the prompt arm
  `:405-422`, the tags and openings `:710-772`, `prompt_origin` `:795`; non-test callers of the
  recognition are `take`, `agent_events.rs:202-203` and `turns.rs:135` only. `agent_events.rs`:
  `on_frame` `:132-183` (the agent-or-remote test `:139-143`, `fold` `:161`), `after_fold`
  `:188-224`. `agent_bar.rs`: `agent_in` `:141`, the left group `:239-263` with the plugin's chip
  `:256-260` and `agent_notify::chip` `:262`, `claude_plugin_chip` `:370-426` (a `Button` with
  `start_icon`, `Tooltip::text`, in a `div` with a debug selector). `agent_notify.rs`: `FRESH`
  `:34`, the read that refreshes only on a change `:144-174`, `read_if_stale` `:178`, `chip`
  `:189`. `claude_plugin.rs:115-130` (`MARLEY_CLAUDE` `:122`). `agents.rs`: `Launcher` `:56`,
  `launcher` `:89`, `which_in` at `:105`. `marley_workbench.rs`: `MarleySettings` `:346-417`,
  `from_settings` `:558-659`, `init` `:753` (agent_notify, agent_bar, claude_plugin at
  `:788-790`). `settings_content/src/marley.rs`: `claude_code_worktree_trust` `:161`,
  `system_one` `:166`, `rusty` `:169`. `default.json`: `claude_code_worktree_trust` `:1786-1789`.
  `marley_page.rs`: `marley_page` `:34-48`, `agents_section` `:138` (`[_; 14]`, ending with the
  Worktree Trust Question `:418-422`), the `uses.check` map item `:809-839` (its write pattern
  copied). `zed_actions::OpenSettingsAt { path, target }` `:149-156`. `script/e2e.sh`: the copy's
  forcing block `:635-649` (Rusty off, Voice off), `profile_setting` `:558-578`. #519's stand-in
  `:71-98` and its steps `:113-152`.
- **Decided at promotion:**
  - **`process::output` kills its child when dropped** (`kill_on_drop(true)` on the builder it
    already makes): the 5 s race drops the losing `output`, and a `--version` that hangs must not
    linger. No new spawn call, so gate:22's pin stays 7. Every other caller awaits its output to
    the end, where the flag does nothing.
  - **D1, D3 and D10 go as drafted; Chad is asked at the end of the batch** (the goal runs without
    stops): `terminalSequence` ungated, the range accepting later 2.1 releases, remote terminals
    read as today. Each is one line of the table or one test in `prompt_reading` to change.

## Phase 2 — Code (2026-10-04)
- **Built.**
  - `marley_agent::versions` (new, pure, `semver` added to the crate): `Range` (`contains` judging
    a prerelease by its release, `words`), `Integration` (id, agent, name, what off means, the
    setting's title, the range), `CLAUDE_PROMPT_TAGS` from 2.1.283 before 2.2.0, `INTEGRATIONS`,
    `integrations_of`, `parse_version`, `Found`, `Off`, `Verdict`, `verdict`, `chip_label`,
    `reasons`.
  - `claude_events`: `PromptReading { Recognized, AllTyped }` with `is_injected`, `is_continuation`
    and `origin`; `fold_with` takes a reading and `Moving::take` asks it; `fold` and the three
    recognition functions keep their signatures as `Recognized`'s, so the tests build unchanged.
  - `marley_workbench::agent_versions` (new): the global, the checks, `prompt_reading`, the chip,
    `allowed_in`. `agent_events::on_frame` reads the reading once and hands it to `fold_with`,
    `after_fold` and `turns::on_event`. The agent bar draws the chip after `agent_notify`'s.
    `MarleySettings::allow_untested_versions`. `process::output` kills its child when dropped.
  - Zed: `settings_content`'s field, `default.json`'s key, `settings_ui`'s Agent Versions section;
    the three ledger rows widened first. The e2e harness allows the row and names a missing
    `MARLEY_CODEX`.
- **Deviations from the plan, and why.**
  - The default is `{"claude_prompt_tags": false}`, not `{}`: the Settings window's toggle reads
    the defaults file, so the row is listed off there, and `settings.json`'s readers see the key.
  - An allowed row is on even before the first check ends (the plan's `NotChecked` kept it off):
    the user turned it on whatever the version, and the e2e harness's allow keeps every other
    scenario's reading the same from the first frame.
  - `Integration` carries `off_means` and `setting`, the tooltip's words, beside the plan's fields.
  - The harness also exports `MARLEY_CODEX=$E2E_WORK/no-codex`, so no run executes the user's
    `codex --version`.
  - `from_settings` passed clippy's 100-line cap with the new field; `allowed_in` and a
    `rail_order` function took two readings out of it.
- **Review.**
  - Re-entrancy: the checks run in `cx.spawn` and `background_spawn(lazy)`; `store` updates the
    global and refreshes windows outside any entity update. `chip` runs while the footer draws and
    may start a check, which only spawns.
  - Errors reach the bar: a missing program, a failed or slow `--version` and an unreadable answer
    each become a reason in the chip's tooltip; the log says the same, once per change (a review
    fix: `store` logged a missing program at every recheck).
  - The 5 s race drops the losing `output`, which now kills its child; the spawn count stays 7.
  - Remote terminals read with the tags and draw no chip (D10).
  - No Zed function body copied; the chip's shape is Marley's own plugin chip.
- **Clippy rounds:** two first doc paragraphs and a missing backtick in `marley_agent`; an unused
  import, a temporary `env` array outliving its statement, `AsyncApp::update` returning its value
  directly; `from_settings` over the line cap.

## Phase 3 — Test (2026-10-04)
- **Scenario:** `script/e2e/648-agent-version-check.sh`, `compositor sway`: four copies of one
  stand-in under `versions/`, the link `bin/claude` moved between them, a stand-in `codex`, a
  scratch `CLAUDE_CONFIG_DIR` listing Marley's plugin at the shipped version. Every check passes:
  the harness's allow; both versions read at start; 2.1.287 read once and 2.2.0 once; the garbled
  answer logged.
- **Shots of the second run, each read:**
  - `648-01-tested`: the rail row keeps "Add a README to the…" through the task notification; no
    chip. REQ-001, REQ-002 (log), REQ-005.
  - `648-02-untested`: the chip "Untested Claude Code 2.2.0" with a warning icon after the plugin's
    place; the rail row shows `<task-notification> T…` as the latest prompt. REQ-003, REQ-004,
    REQ-006, REQ-007.
  - `648-03-why`: the tooltip: prompt tags off and what that changes, the link's path and 2.2.0,
    "Tested on 2.1.283 and later 2.1 releases.", and Prompt Tags on Untested Claude Code. REQ-008.
  - `648-04-setting`: the Settings window at Marley › Agent Versions, Prompt Tags on Untested
    Claude Code off. REQ-009, REQ-010.
  - `648-05-allowed`: the map set true from outside: no chip; the row keeps the user's prompt.
    REQ-011.
  - `648-06-unreadable`: "Claude Code version unknown"; its tooltip quotes `--version printed
    "garbled (Claude Code)"`. REQ-012.
  - `648-07-back-in-range`: 2.1.300: no chip; the row keeps the user's prompt. REQ-005.
- **Fixed in Test:** the first run's `648-06` showed the chip but no tooltip: the pointer still
  rested where `648-03` left it, and a hover needs a move. The scenario moves it off first.
- **Not reached by a scenario:** a missing program (REQ-013), the first check still running
  (REQ-014), a remote terminal (REQ-015), the 5 s bound and the pure module (REQ-016): review, and
  the gate for REQ-016. REQ-017 is setup's check.
- **Gate:** `just gate-diff` green after the Test phase's scenario fix: every gate on the scope and
  the receipt.

## Phase 4 — Complete (2026-10-04)
- **Docs:** `CHANGELOG.md` (Added); `docs/marley/guide.md` (the agent bar's version chip, the
  settings key and the Agent Versions section); `docs/marley_architecture/marley_agent.md`
  (`versions`, `PromptReading`) and `marley_workbench.md` (`agent_versions`); the design note's B7
  marked started; the three `zed-touchpoints.md` rows checked against what shipped (the default
  lists the row false).
- **Knowledge:** AD-claude-648-marley-turns-off-only-what-rests-on-an-untested-agent-surface-001,
  AD-claude-648-terminalsequence-is-documented-correcting-ad-482-001,
  L-claude-648-a-tooltip-needs-the-pointer-to-move-onto-it-001,
  L-claude-648-a-stand-in-can-print-its-version-from-its-own-file-name-001.
- **Brain:** `brain decide` on consultation `1e38bf4472a749f4814b1397de1f8ba8`:
  `decisions/marley-turns-off-only-what-rests-on-an-untested-agent-surface-with-a-chip-that-says-why-648`,
  follow-up 2026-11-04.
- **Open for Chad (asked at the end of the batch):** D1 (`terminalSequence` ungated), D3 (later
  2.1 releases count as tested) and D10 (remote terminals keep the tags).
- Ticket closed, pipeline archived, committed and pushed.

