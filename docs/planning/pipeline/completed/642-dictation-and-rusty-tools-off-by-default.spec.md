---
pipeline_id: 57fd9b86-facc-463a-b3c8-3b952a658754
ticket: docs/planning/tickets/open/TICKET-642-dictation-and-rusty-tools-off-by-default.md
status: Phase 4 — Complete PASS
title: "Dictation and Rusty's tools wait to be turned on"
type: feature
slice: prong 1 T7d and prong 2 C2 (their defaults); design note Part 3, V1
references: [docs/planning/design-notes/herdr-and-hermes-2026-10-02.md, docs/marley/three-prong-plan.md]
---

## Title
The agent bar's microphone (#480) and Rusty's tools for Zed's agents (#633) stop switching on
wherever their program is installed. Dictation gets `marley.voice.enabled`, off by default, the
master switch of the `marley.voice` block the design note sketches; `marley.rusty_tools` defaults
to false. Off, the microphone is hidden, Marley starts no `voxtype`, and `marley: toggle
dictation` says dictation is off and where to turn it on. Chad, 2026-10-02: "Both off by default".

## Scope
### In
- **The voice switch:** `marley.voice: { enabled: Option<bool> }` in `settings_content`
  (`MarleyVoiceSettingsContent`), `"voice": { "enabled": false }` in `default.json`, resolved as
  `MarleySettings::dictation`, a `Dictation { On, Off }` read "off unless on", as
  `embedded_harness` is.
- **The microphone:** `agent_bar::microphone` draws nothing while dictation is off, before it
  follows Voxtype's status, so off starts no `voxtype status --follow`.
- **The action:** `marley::ToggleDictation` checks the switch first; off, a workspace toast says
  "Dictation is off. Turn it on in the Voice section of the Marley settings." and runs nothing, the
  way System One's check answers while the layer is off. On, it behaves as #480 ships it.
- **Live, no restart:** an observer of the settings store in `voice.rs` drops the status follower
  when the switch turns off (its `kill_on_drop` child ends) and leaves the next drawn microphone to
  start one when it turns on; every `TerminalView` already redraws on a settings change.
- **Rusty's tools:** `marley.rusty_tools` defaults to false (`default.json`, the content's doc,
  `RustyTools::from_setting` reading "offered only when on"); `rusty.rs`'s observer already offers
  and withdraws while Marley runs.
- **The Settings window:** the Marley page gains a Voice section, after Push and before System
  One, with one toggle, Voice (`marley.voice.enabled`), whose description names Voxtype; Rusty
  Tools for Agents stays in the Agents section and shows off.
- **The e2e harness:** `script/e2e.sh` keeps writing `marley.rusty_tools: false` into each run's
  copy of the user's settings, which now matters only for a user who turned it on, and also writes
  `marley.voice.enabled: false`; `script/e2e/480-voice-input.sh` sets Voice back on for its fake
  with the harness's `profile_setting` (#635).
- **The in-app guide page** (`crates/marley_workbench/guide/index.html`): its dictation article and
  agent bar line say Voice must be on. It sits under `crates/marley_*`, which the commit receipt
  binds, so it changes in the Code phase, before the gate.
- `script/e2e/642-dictation-and-rusty-tools-off-by-default.sh`.

### Out (explicitly deferred)
- The rest of the `marley.voice` block (`dictation`, `commands`, `speak`, `speech_output`) and the
  voice slices V2 to V4: Chad shelved voice on 2026-10-02 ("shelve this for later but keep it
  open").
- The `marley.knowledge` block and Rusty inside Marley: Chad asked for that plan separately
  (`docs/marley/rusty-in-marley.md`).
- Moving `marley.rusty_tools` into `marley.rusty.agent_tools`: that plan's R-D0 and slice R1 move
  it when the `marley.rusty` block lands. This ticket changes the key's default where it is; with
  the default off, R1 only has to carry the users who turned it on.
- Hiding `marley: toggle dictation` from the command palette while off, Zed's
  `CommandPaletteFilter` way (D4).
- A notice to users who had either feature on by default: the CHANGELOG's Changed entry and the
  guide say how to turn each on; no key moves, so there is nothing to migrate.
- Finding a Voxtype installed after Marley started: the PATH lookup stays at start, as #480 ships.
- `crates/zed/src/zed.rs`'s test setup, which sets `marley.rusty_tools = Some(false)` (#634): it
  now matches the default and stays, since removing it is a Zed touch for nothing.
- A Settings window toggle for `marley.embedded_harness` (#632 left it for a harness section).

## Reference (§20)
Upstream Zed: the Settings window's toggles (`settings_ui`'s `SettingField`, which shows and
resets to `default.json`'s value through `raw_default_settings`) and the settings store's
observers, which let a setting take effect while Zed runs, are kept as they are; Marley adds a
section and two defaults to its own Marley page and settings block. The off-by-default shape of a
feature block is Marley's own `marley.system_one` (#565). For the dictation switch itself, Orca's
behavior map: "Settings → Voice (off by default, no model chosen)"
(`docs/orca_architecture/05-terminal-and-workspace.md` §2.16); Warp wires its voice input through
its AI settings module (`docs/warp_architecture/crates/voice_input.md`, "Used by"). No Warp code.

### Prior art
- **Behavior maps:** Orca §2.16 puts dictation under Settings → Voice, off by default, and records
  that its shortcut takes Ctrl-E from readline and Claude Code while dictation is enabled, a cost
  of a voice feature left on. `docs/warp_architecture/crates/voice_input.md` names the settings
  module Warp wires voice through and, under "Marley relevance", argues for voice off where no
  transcription service is set up. `docs/zed_architecture/subsystems/07-workspace-panes-palette.md`
  §3.1 describes `CommandPaletteFilter`, which Zed uses to hide a disabled feature's actions.
- **Published material:** the design note's Part 2 read of Hermes Agent's voice: each mode is its
  own switch and the wake word ships off (`tools/voice_mode.py`); Voxtype 1.0.1's `record` and
  `status` (as #480 used them). Nothing in the MCP specification bears on a default.
- **Code we already ship:** `settings_content::SystemOneSettingsContent` (a block whose `enabled`
  is the master switch, the shape `marley.voice` copies); `EmbeddedHarness::from_setting` ("off
  unless it is on", `marley_workbench.rs:463-479`), which `Dictation` copies and `RustyTools` turns
  into; `system_one::check_toast` and `show_toast` (`system_one.rs:1225-1257`), the toast an off
  feature answers with; `block_headers::init`'s settings observer (`block_headers.rs:41-48`) and
  `TerminalView`'s own (`terminal_view.rs:507`, which notifies at `:956`), which make the bar's
  redraw live; `rusty::init`'s observer (`rusty.rs:34-54`), already live, proven by #633's
  `633-02-off`; `util::command`'s `kill_on_drop`, which ends the status follower with its task;
  Zed's `agent_ui::update_command_palette_filter` (`agent_ui.rs:790-860`), which hides the agent's
  actions while `disable_ai` is on: considered for the action and not taken (D4). No new
  dependency: `which` is already used.

## UI proof
`script/e2e/642-dictation-and-rusty-tools-off-by-default.sh` (`compositor sway`: it types into the
Settings window, a second window). Fixtures: a scratch repository; a terminal HOME whose `.bashrc`
becomes a stand-in `claude` (#480's); a fake `voxtype` first on the PATH that logs each call and
whose `status` follows a file (#480's, plus the log); a stand-in `rusty-mcp` (#633's); the
MCP Servers page bound to Ctrl+Alt+Shift+M in the run's keymap (#633's). Setup checks that the
harness's copy turned both switches off, then deletes `marley.voice.enabled`, `marley.rusty_tools`
and any `context_servers.rusty` from it, so the run starts on the shipped defaults. Every later
change is an edit of the run's settings file from outside while Marley runs; Marley itself never
writes it (L-607). Shots:
- `642-01-no-microphone`: the agent bar under the stand-in Claude Code with no microphone, Voxtype
  on the PATH; the fake's log empty.
- `642-02-dictation-is-off`: after `marley: toggle dictation` from the palette, the toast; the log
  still empty.
- `642-03-voice-toggle-off`: the Settings window's Marley page searched for "Voxtype": the Voice
  section's Voice toggle, off.
- `642-04-rusty-toggle-off`: searched for "Rusty": Rusty Tools for Agents, off.
- `642-05-no-rusty`: the MCP Servers page: `marley` alone, though `rusty-mcp` is on the PATH.
- `642-06-rusty-on`: `marley.rusty_tools` set true: `rusty` listed and running.
- `642-07-microphone-on`: the Settings window closed, `marley.voice.enabled` set true: a muted
  microphone; the log shows `status --follow`.
- `642-08-recording`: `marley: toggle dictation`: the microphone red; the log shows `record
  toggle`. A second toggle and the fake's idle line follow.
- `642-09-microphone-off`: `marley.voice.enabled` set false: no microphone; the fake's status
  process is gone.

## Locked-In Decisions
- D1 — The switch is `marley.voice.enabled`: a `marley.voice` block with one key now. The design
  note's block (`enabled`, `dictation`, `commands`, `speak`, `speech_output`) grows inside it with
  no migration: `enabled` stays the master switch, and a later `dictation` key defaults to true, so
  a user who turned Voice on keeps dictation. Readers see `MarleySettings::dictation`, which then
  becomes `enabled` and `dictation` together without a reader changing. Rejected: a flat
  `marley.dictation` (moving it into `voice` later is a settings migration);
  `marley.voice.dictation` alone (a master switch added later, off by default, would turn
  everyone's dictation off or need a migration).
- D2 — Both off by default, as Chad answered. Each reads "off unless on" (only `Some(true)` turns
  it on), and `default.json` carries `false` for each, so the Settings window shows the default and
  its reset returns to it.
- D3 — Off means Marley shows no microphone and starts no `voxtype`: no status follower and no
  `record toggle`; a follower running when the switch turns off is dropped, which ends its process.
  The PATH lookup at start stays (it reads directories and starts nothing), so turning Voice on
  shows the microphone at once. Voxtype itself is untouched: Omarchy's keys still dictate, and a
  recording under way when the switch turns off is Voxtype's to finish.
- D4 — `marley: toggle dictation` stays in the palette and, while off, answers with a toast that
  says dictation is off and where to turn it on, as System One's check does. The switch is checked
  before Voxtype's presence; on and without Voxtype, #480's error stands. Rejected: hiding the
  action with Zed's `CommandPaletteFilter`, which leaves the user no pointer to the switch.
- D5 — A change applies while Marley runs. The settings store recomputes `MarleySettings` when
  `settings.json` changes (the Settings window writes it; Zed watches the file). The microphone is
  read at each render, and each `TerminalView` notifies on a settings change; `voice.rs`'s observer,
  registered at init and so run before the views' (L-456), drops the follower when the switch turns
  off. The action reads the switch each time it runs. Rusty's offer already follows the store.
- D6 — The Marley page's Voice section holds one toggle, titled Voice, for `marley.voice.enabled`;
  later voice uses add their items under it, as System One's uses sit under System One.
- D7 — `script/e2e.sh` writes both switches off in each run's copy. With the defaults off, the lines
  matter only for a user who turned a feature on: without them every scenario would start that
  user's real `rusty-mcp` or `voxtype status` (L-633), and the agent bar each scenario draws would
  depend on that user's choice. Scenarios that want either turn it back on: 480, 633 and 642.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE `marley.voice.enabled` is not set, the agent bar shall show no microphone, with `voxtype` on the PATH. | Shot `642-01-no-microphone` |
| REQ-002 | WHILE `marley.voice.enabled` is off, the system shall run no `voxtype` process. | The scenario's checks of the fake's log at `642-01` and `642-02` and of its status process at `642-09`; review |
| REQ-003 | WHEN `marley::ToggleDictation` runs while `marley.voice.enabled` is off, the system shall show a toast saying dictation is off and where to turn it on. | Shot `642-02-dictation-is-off` |
| REQ-004 | WHERE the user's settings do not set `marley.voice.enabled`, the Settings window's Marley page shall show a Voice section whose Voice toggle is off. | Shot `642-03-voice-toggle-off` |
| REQ-005 | WHERE the user's settings do not set `marley.rusty_tools`, the Marley page shall show Rusty Tools for Agents off. | Shot `642-04-rusty-toggle-off` |
| REQ-006 | WHERE `marley.rusty_tools` is not set, the system shall offer no `rusty` context server to Zed's agents, with `rusty-mcp` on the search path. | Shot `642-05-no-rusty` |
| REQ-007 | WHEN `marley.rusty_tools` turns on while Marley runs, the system shall offer `rusty` without a restart. | Shot `642-06-rusty-on` |
| REQ-008 | WHEN `marley.voice.enabled` turns on while Marley runs, the agent bar shall show the microphone without a restart. | Shot `642-07-microphone-on` |
| REQ-009 | WHEN `marley::ToggleDictation` runs while `marley.voice.enabled` is on, the system shall run `voxtype record toggle`. | Shot `642-08-recording`; the fake's log |
| REQ-010 | WHEN `marley.voice.enabled` turns off while Marley runs, the agent bar shall hide the microphone without a restart. | Shot `642-09-microphone-off` |
| REQ-011 | The e2e harness shall write `marley.rusty_tools` and `marley.voice.enabled` false into each run's copy of the user's settings. | The scenario's setup check; review of `script/e2e.sh` |

## Phase Plan
- **P1 Plan** — promote; re-verify the seams cited in the notes; ask the brain; confirm the Settings
  window's search takes typed keys when it opens (633 typed into it).
- **P2 Code** — the `README.md` marker first; the ledger rows of `settings_content/src/marley.rs`,
  `default.json` and `marley_page.rs` widened; the setting, `Dictation`, the microphone's and the
  action's checks, the observer, `RustyTools`' flip, the Voice section, `e2e.sh`, 480's scenario,
  the in-app guide page; a review of the diff; `script/gates.sh --diff` green.
- **P3 Test** — write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG (Changed) and architecture docs (§21), the user docs the notes list,
  ledger capture (§19), the brain's two decisions followed up as revised, close the ticket,
  archive, commit.
