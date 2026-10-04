---
pipeline_id: 60f8edbd-56da-4403-8031-76a95f24facd
ticket: docs/planning/tickets/closed/TICKET-648-agent-version-check.md
status: Phase 4 — Complete PASS
title: "Marley checks the agent's version before an untested integration turns on"
type: feature
slice: agents on their own tools B7 (design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md); T-series agent terminals
references: [docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md, docs/planning/pipeline/completed/519-claude-code-events-in-the-rail.spec.md, docs/planning/pipeline/completed/552-codex-and-opencode-notifications.spec.md, docs/planning/pipeline/completed/547-claude-code-events-slice-2.spec.md]
---

## Title
Marley follows the user's own `claude` and `codex`. It reads each one's version with `--version`,
once at start and again when the file on its search path changes, and compares it with a table of
the versions each integration that rests on an undocumented surface was tested on. Outside its
range such an integration stays off, and a chip in the agent bar under the agent's terminal says
why: the version and path found, the range tested, and the setting that turns it on anyway,
`marley.allow_untested_versions`, off by default. Today the table has one row, the injected-prompt
tags the rail uses to keep the user's prompt (#519). Claude Code's hooks reference now documents
`terminalSequence`, so the hook channel itself is not gated. B1 to B3 add their own rows when they
land. Chad, 2026-10-02, on B7: "Your install, with a check".

## Scope
### In
- **The table:** `marley_agent::versions` (new module, pure: no gpui, no IO). A row is an
  `Integration`: its id (the key under `marley.allow_untested_versions`), its agent, its name, what
  it rests on, and the range it was tested on (`from`, inclusive, and `before`, exclusive or open).
  `INTEGRATIONS` holds the rows; a range test that judges a prerelease by its release numbers;
  `parse_version` for `--version`'s output; `verdict` from what was found and the setting. One row:
  `claude_prompt_tags`, Claude Code from 2.1.283, before 2.2.0 (D3).
- **Detection:** `marley_workbench::agent_versions` (new module). `claude` is the program
  `MARLEY_CLAUDE` names, else the first on the launcher's search path; `codex` is `MARLEY_CODEX`,
  else the search path. Each file's identity (its canonical path, size and modification time) is
  read off the main thread; `--version` runs only when the identity changed, through
  `process::output` with the program's own folder first on `PATH`, bounded at 5 s, its first
  4 KiB read. It runs at start, and again when an agent bar for that agent draws 10 s or more after
  that agent's last check. Each read writes one log line.
- **The gate:** the injected-prompt recognition in `marley_agent::claude_events` (the 19 tags, the
  seven openings and the compaction's continuation) takes a `PromptReading`, `Recognized` or
  `AllTyped`. `agent_events::on_frame`, its `after_fold` and `turns::on_event` pass the verdict's
  reading for a local terminal; a remote terminal (#543) keeps `Recognized`. The existing
  functions keep their signatures and mean `Recognized`, so the tests in the tree build unchanged.
- **The why:** a chip in the agent bar, after the plugin's chip, under a local terminal whose
  foreground agent has a row off. Its tooltip names what is off, the path and version found, the
  range tested and the setting; a click opens the Settings window at that row's setting (D8).
- **The setting:** `marley.allow_untested_versions: {}` in `settings_content` and `default.json`,
  a map from a row's id to true; an Agent Versions section on the Settings window's Marley page,
  after Agents, with Prompt Tags on Untested Claude Code (D9).
- **The e2e harness:** `script/e2e.sh` writes `marley.allow_untested_versions:
  {"claude_prompt_tags": true}` into each run's copy, so a scenario's stand-in `claude`, which
  prints no version, and the user's own Claude Code version leave every scenario's reading as it
  was (D11).
- `script/e2e/648-agent-version-check.sh`.

### Out (explicitly deferred)
- **Gating `terminalSequence`.** Claude Code's hooks reference (fetched 2026-10-03) documents the
  field, its allowlist (OSC 0, 1, 2, 9, 99, 777 and BEL) and its limits, so the hook channel rests
  on a documented surface and gets no row. Its 4,096-byte cap is not documented; `event.py` keeps
  each summary under 2,900 bytes, and a lower cap would drop frames, which the rail's quiet timer
  already covers. AD-482's "not its public docs" is out of date; Complete appends the correction.
- **B1 to B3's rows** (Codex's App Server schemas, closed per tested range since they are
  generated per release; the mod from 2.1.287 on; the IDE link's internal tools): each ticket adds
  its row, its Settings item and its key in the e2e harness's allow map.
- **B7 part 2, a remote host's version:** a remote terminal's Claude Code is that host's install;
  reading its version over the terminal's SSH link, or carrying it in the plugin's frame (Claude
  Code passes its hooks `AI_AGENT` and `CLAUDE_CODE_EXECPATH`, both undocumented, which name the
  running version), is a slice of its own. Until then a remote terminal reads as today (D10).
- **The running session's own version.** A session started before an update runs its old
  version; the check judges the install on the search path (Risks).
- **The trust reader** (`marley_agent::trust`, #587): it acts only on a screen that matches a
  published wording exactly, so on a changed release it does nothing and the user answers; a row
  would add nothing until B6 settles the question.
- **`installed_plugins.json`** (`claude_plugin.rs:181-208`): it decides only whether the agent bar
  offers the plugin's install or update; B2's plugin retires it.
- Gemini CLI and OpenCode: not read until a row needs them.
- Pinning binaries or checking digests, the harness's way (Chad chose the user's install).
- A script that checks a Claude Code binary for the 19 tag names, for the ticket that moves a
  bound (the check done by hand for this plan is in the notes).
- Showing the version found when nothing is off; merging `ClaudePlugin`'s own `claude` lookup
  with the check's.

## Reference (§20)
Upstream Zed, kept as it is and read for the shape: `acp_thread`'s `LoadError::Unsupported {
command, current_version, minimum_version }` and `agent_ui`'s `render_unsupported`
(`conversation_view.rs:2776`), which name the program's path, the version found and the version
needed, and say "does not report a valid --version" when there is none to read. Marley's chip says
the same three things. `extension_host::wasm_host::wit::wasm_api_version_range` (`wit.rs:54-69`), a
`semver` range with `contains`, is the table's shape. Where Zed refuses to start an agent outside
its range, Marley keeps the agent and turns off one integration. What is checked is
Marley-specific: Zed runs Claude Code and Codex through its own adapters and never reads their
hooks or prompt tags. No Warp code; Warp's maps hold no agent version check.

### Prior art
- **Behavior maps:** `docs/warp_architecture/` (checked `crates/` and
  `subsystems/04-agent-ai-mcp.md`): no version check of an agent CLI;
  `LocalClaudeCodexChildHarnesses` is a feature flag. `docs/zed_architecture/` maps neither `agent_servers` nor `extension_host`, so
  their source is the map. `docs/orca_architecture/01-agents-and-sessions.md:134` (Orca installs
  SessionEnd "on CLI versions that support it") and `:498` (its structured lane pins SDK and CLI
  versions). Orca (MIT, read at `1c2cf120e3`),
  `src/main/claude/claude-session-end-hook-capability.ts`: `probeClaudeCliVersion` runs
  `--version` with a 5 s timeout and 4,096 bytes of output, with the program's own folder first on
  `PATH` so a `#!/usr/bin/env node` launcher finds its runtime, and takes the first `N.N.N` with an
  optional prerelease; `CLAUDE_SESSION_END_CAPABILITY_FLOOR = '2.1.261'` carries the note "the only
  version measured, not an established minimum". Marley takes the timeout, the cap, the `PATH` and
  the parse rule as ideas; no code is copied.
- **Published material:** Claude Code's CLI reference (`--version`, `-v`: "Output the version
  number"; no format given) and its hooks reference, fetched 2026-10-03 (`terminalSequence`
  documented with its allowlist; UserPromptSubmit's input carries no version). On this box
  `claude --version` prints `2.1.288 (Claude Code)` and `codex --version` prints
  `codex-cli 0.155.1`. rustal-harness (Ignibyte's own, read only, ideas reusable):
  `docs/CLAUDE_CODE.md:14-41` (its own Claude Code 2.1.287, pinned by digest and size, recorded as
  `--version` prints it; `claude_unsupported_profile`; "a new version is a new pin") and
  `docs/CODEX_PROTOCOL.md:39-80` (`codex-cli 0.158.0` bound by digest; the App Server's `initialize`
  answer carries no protocol version, so for B1 the binary's version is the one signal). Marley
  does not pin; it takes the compiled-in table and the named reasons for a refusal.
- **Code we already ship:** `semver` (Cargo.lock 1.0.28, already `marley_workbench`'s for
  `ClaudePlugin::needs_update`; `Version::new` is `const`); `which`'s `which_in`;
  `marley_workbench::process::output`, the crate's spawn site, so gate:22's count stays at 7;
  `agent_notify`'s re-read when a bar draws (`FRESH` `:34`, `read_if_stale` `:177-185`, `chip`
  `:189`), whose shape the check copies; `claude_plugin::init` (`:111-125`: `MARLEY_CLAUDE`, else
  the PATH); `agents::Launcher::search_path` (`agents.rs:57-92`); `zed_actions::OpenSettingsAt`;
  the Settings page's items over a map (`marley.system_one.uses.check`, `marley_page.rs:772-800`).
  The sweep's answer: `semver` owns the ordering, `which` the lookup, `process` the run, the
  Settings window the switch, and Zed's unsupported-version callout the wording; the table and the
  gate are the only new code.

## UI proof
`script/e2e/648-agent-version-check.sh` (`compositor sway`: it hovers and clicks the chip).
Fixtures: a scratch repository opened with `open_path`; a HOME whose `.bashrc` puts
`$E2E_WORK/bin` first (`terminal_env HOME`); `$E2E_WORK/versions/` with copies of one stand-in
named `2.1.287`, `2.2.0`, `garbled` and `2.1.300`, and `$E2E_WORK/bin/claude` a link to one of
them, moved with `ln -sfn` as Claude Code's own installer moves `~/.local/bin/claude`;
`MARLEY_CLAUDE` names the link; a stand-in `codex` printing `codex-cli 0.155.1`, named by
`MARLEY_CODEX`; a scratch `CLAUDE_CONFIG_DIR` whose `installed_plugins.json` lists Marley's plugin
at the shipped version, so the plugin's chip stays away. Setup checks that the harness's copy
allowed the row, then sets `marley.allow_untested_versions` to `{}`. The stand-in, given
`--version`, prints its own file's name and ` (Claude Code)`; otherwise it acts out #519's way: at
each Enter it runs the plugin's real `hooks/event.py` with the next step's payloads and writes each
`terminalSequence` to its terminal. Step 1 is SessionStart, the user's prompt "Add a README to the
project" and Bash starting; step 2 is an injected `<task-notification>` prompt. Every settings
change is an edit of the run's file from outside (L-607). Never the user's Claude Code or Codex.
Shots:
- `648-01-tested`: the link at `2.1.287`, both steps run: the rail row keeps "Add a README to the
  project" through the injected prompt; no version chip. The log holds `agent versions: Claude
  Code 2.1.287 at` and `agent versions: Codex 0.155.1 at`.
- `648-02-untested`: the link moved to `2.2.0`, the stand-in started again more than 10 s after the
  last check, both steps run: the chip reads `Untested Claude Code 2.2.0`; the row shows the
  injected prompt as the latest. The log holds one `2.1.287` line and one `2.2.0` line.
- `648-03-why`: the pointer on the chip: its tooltip names the tags, the link's path, 2.2.0, the
  tested range and Prompt Tags on Untested Claude Code.
- `648-04-setting`: the chip clicked: the Settings window at Agent Versions, Prompt Tags on
  Untested Claude Code off.
- `648-05-allowed`: the window closed, `marley.allow_untested_versions` set to
  `{"claude_prompt_tags": true}`, the stand-in run again: no chip; the row keeps the user's prompt.
- `648-06-unreadable`: the map back to `{}`, the link moved to `garbled`, the stand-in run again
  past the wait, the pointer on the chip: `Claude Code version unknown`, the tooltip quoting
  `garbled (Claude Code)`.
- `648-07-back-in-range`: the link moved to `2.1.300`, a later 2.1 release no row names: no chip;
  the row keeps the user's prompt.

## Locked-In Decisions
- D1 — A row is an integration that rests on a Claude Code or Codex surface the agent does not
  document. Today there is one: the injected-prompt recognition, Orca's observed tags and openings
  (`claude_events.rs:710-745`), which Claude Code never names. `terminalSequence` is documented now
  and gets no row. Rejected: one row for all of #519's hook events (off, it would take away the
  rail's state, the banners, resume and the paste guard that `agent_events::waiting` gives rich
  input and send selection (F-claude-594), over a field the docs cover); a row for the trust reader
  (it fails closed, Out).
- D2 — The table is code in `marley_agent::versions`, compiled in: a row's `id`, `agent`, `name`,
  `rests_on`, `tested: Range { from, before }` and a doc comment naming the evidence (the tickets
  and the versions checked). The build and its table cannot disagree. Rejected: a data file the
  user could edit (the setting is the user's way); the harness's pins (Chad).
- D3 — Today's row covers Claude Code from 2.1.283, the version #519 checked the list against, up
  to 2.2.0, so later 2.1 releases count as tested. Every one of the 19 tag names is in the 2.1.283
  and 2.1.288 binaries (read, not run, on this box). Claude Code updates itself several times a
  week: 2.1.283 to 2.1.288 between 2026-09-25 and 2026-10-02, and 2.1.287 to 2.1.288 since this
  batch was briefed. A range closed at the newest patch would turn the row off within days of each
  Marley build, and Chad's answer keeps the user's install working. A new minor or major is
  outside until a ticket checks it and moves `before`. Each later row picks its own bounds (B1's
  closed, B2's open from 2.1.287). A prerelease is judged by its release numbers, so 2.2.0-beta.1
  counts as 2.2.0, which `semver` alone would order below `before`.
- D4 — Off, the tags' row reads every prompt as the user's, which is what UserPromptSubmit means in
  Claude Code's hooks reference; the openings and the compaction's continuation go with the tags.
  Nothing else of #519 changes: the rail, the banners, resume and the stop kind keep their events.
  The row's effect is that a task notification or a system reminder shows as the latest prompt.
  `claude_events::PromptReading { Recognized, AllTyped }` reaches the fold, the outcome note and
  the turn's title; `fold`, `prompt_origin` and `is_harness_injected` stay and mean `Recognized`.
  Rejected: hiding every prompt Marley cannot place (the user's own would go in the usual case); a
  rule that any leading `<tag>` is injected (a heuristic nobody has tested).
- D5 — Finding and reading. `claude` is `MARLEY_CLAUDE`, else `which_in` on
  `agents::launcher(cx).search_path` (L-531, PR-claude-name-the-fakes-the-app-runs-001); `codex` is
  `MARLEY_CODEX`, else the same. The identity is the canonical path, the size and the modification
  time; `--version` runs only when it changed, through `process::output` with the program's folder
  first on `PATH`, raced against 5 s, its first 4 KiB kept. The version is the first word of the
  first line that parses as `N.N.N` with an optional prerelease, a leading `v` dropped: `2.1.288
  (Claude Code)` and `codex-cli 0.155.1` both read. All of it runs in a lazy future on the
  background executor (L-claude-482). Each read logs `agent versions: <name> <version> at <path>`,
  or the reason it failed, at info.
- D6 — When: at `init`, for both agents; then when an agent bar for that agent draws and its last
  check is 10 s old or more, as `agent_notify` re-reads its files, so a change shows within seconds
  of the agent's next run in a visible terminal. Only a changed verdict refreshes the windows, so a
  drawing bar does not redraw itself forever. Nothing polls in the background. Until an agent's
  first check ends, its rows are off and no chip shows.
- D7 — A verdict is `On` (in range), `Allowed` (out of range or unread, and the setting on) or
  `Off` with its reason: `Untested { version }`, `Unreadable { printed or error }`, `Missing {
  where Marley looked }` or `NotChecked`. A version Marley cannot read counts as untested: Chad's
  rule turns off what is not in the list.
- D8 — The why shows in one place, the agent bar's chip, beside the plugin's chip, the family of
  chips that already says when Marley's link to Claude Code is missing or old (#482, #547). It shows
  under a local terminal whose foreground agent has a row `Off` for any reason but `NotChecked`. Its
  label is `Untested Claude Code 2.2.0`, `Claude Code version unknown` or `Claude Code not on
  Marley's PATH`, with a warning icon. Its tooltip gives, for each row off, what is off and what
  that changes, the path and the version found or what `--version` printed, the range tested in
  words ("2.1.283 and later 2.1 releases", "2.1.287 and later", "0.155.0 up to 0.159.0"), and the
  setting. A click dispatches `zed::OpenSettingsAt` with the first row's key. Rejected: a rail
  row's tooltip (hidden until hovered, and a rail row is not the agent's install); the Settings
  window alone (seen only when looked for, and its live state would wait on #643's
  `MarleyPageViews`); a notification (gone once read, and repeated at each update).
- D9 — The setting is `marley.allow_untested_versions: Option<BTreeMap<String, bool>>`, `{}` by
  default, keyed by a row's id, as System One's `uses` map is keyed by a use's name. An Agent
  Versions section after Agents on the Marley page holds an item per row; today Prompt Tags on
  Untested Claude Code (`marley.allow_untested_versions.claude_prompt_tags`). Off by default: it
  exists for trying a new release, and the chip says when it would help.
- D10 — A remote terminal's prompts are read with the tags whatever the local version, and no chip
  draws under it: the `claude` on Marley's search path says nothing about the host's. B7 part 2
  (Out) judges a host by its own version.
- D11 — The e2e harness allows `claude_prompt_tags` in each run's copy (L-633): none of the
  stand-ins seventeen scenarios name in `MARLEY_CLAUDE` answers `--version`, and the user's own
  Claude Code moves on its own, so without it the prompt checks of #509's and #519's scenarios
  would change with the user's install. This ticket's scenario sets the map back to `{}`.

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts, the system shall read the version of the `claude` that `MARLEY_CLAUDE` names, else the first `claude` on the search path its agents use, with `--version`, off the main thread. | The scenario's log check (`agent versions: Claude Code 2.1.287 at`); review |
| REQ-002 | WHEN Marley starts, the system shall read the version of the `codex` that `MARLEY_CODEX` names, else the search path's first, the same way. | The scenario's log check (`agent versions: Codex 0.155.1 at`) |
| REQ-003 | WHILE the file found keeps its canonical path, size and modification time, the system shall not run its `--version` again. | The log's single `2.1.287` line at `648-02-untested`; review |
| REQ-004 | WHEN an agent bar for Claude Code or Codex draws 10 s or more after that agent's last check and the file found has changed, the system shall read its version again and judge that agent's rows by it. | Shot `648-02-untested`; the log's `2.2.0` line |
| REQ-005 | WHILE the Claude Code version found is in `claude_prompt_tags`' range, the rail shall keep the user's prompt through a prompt Claude Code injects. | Shots `648-01-tested`, `648-07-back-in-range` |
| REQ-006 | WHILE the version found is outside that range and the setting does not allow the row, the system shall read every prompt of a local terminal's Claude Code as the user's. | Shot `648-02-untested` |
| REQ-007 | WHILE a row of the agent in a local terminal's foreground is off, the agent bar shall show a chip naming the agent and the version found. | Shot `648-02-untested` |
| REQ-008 | The chip's tooltip shall name what is off, the program's path and the version found, the range tested, and the setting that turns the row on. | Shot `648-03-why` |
| REQ-009 | WHEN the user clicks the chip, the system shall open the Settings window at the setting of the first row that is off. | Shot `648-04-setting` |
| REQ-010 | The Settings window's Marley page shall hold an Agent Versions section with Prompt Tags on Untested Claude Code, off by default. | Shot `648-04-setting` |
| REQ-011 | WHERE `marley.allow_untested_versions` holds a row's id as true, the system shall turn that row on whatever version is found, and the chip shall not show for it. | Shot `648-05-allowed` |
| REQ-012 | IF `--version` fails, runs past 5 s or prints no version, THEN the agent's rows shall be off and the chip's tooltip shall say what it printed or why it failed. | Shot `648-06-unreadable`; review of the bound |
| REQ-013 | IF Marley finds no program for an agent, THEN that agent's rows shall be off and the chip shall say where Marley looked. | Review |
| REQ-014 | WHILE an agent's first check has not ended, the system shall keep its rows off and show no chip. | Review |
| REQ-015 | WHILE a terminal is remote (#543), the system shall read its Claude Code's prompts with the tags, whatever the local version, and draw no version chip under it. | Review |
| REQ-016 | `marley_agent::versions` shall be free of gpui and IO, each row naming its id, agent, what it rests on and its range, with a prerelease judged by its release. | Review; `script/gates.sh --diff` |
| REQ-017 | The e2e harness shall allow `claude_prompt_tags` in each run's copy of the user's settings. | The scenario's setup check; review of `script/e2e.sh` |

## Phase Plan
- **P1 Plan** — promote after #647; read the hooks reference again for `terminalSequence`; ask the
  brain (`brain_ask`) on D1, D3 and D10; confirm with Chad that `terminalSequence` goes ungated
  (D1), the range policy that accepts later 2.1 releases (D3) and that remote terminals keep
  today's reading until B7 part 2 (D10); re-read `script/e2e.sh`'s settings block after #642 and
  #643 change it.
- **P2 Code** — the `README.md` marker first; the three ledger rows widened before their files;
  the table and the reading in `claude_events`; `agent_versions` with the chip; the gate in
  `agent_events` and `turns`; the setting, its default and its section; the harness's allow line;
  a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG (Added) and the docs the notes list (§21), ledger capture (§19) with
  AD-482's correction, close the ticket, archive, commit.
