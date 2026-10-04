# Dictation and Rusty's tools wait to be turned on — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-642-dictation-and-rusty-tools-off-by-default.md
- **Pipeline spec:** 642-dictation-and-rusty-tools-off-by-default.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-02)
- **Request:** Chad, 2026-10-02, with the herdr and Hermes research: "i would like these to be
  enabled rather than by default because some may not want". Asked whether the dictation mic and
  `marley.rusty_tools`, both on wherever their program is installed, move behind switches: "Both
  off by default". On voice as a whole: "shelve this for later but keep it open". The batch: "Queue
  all three". All in `docs/planning/design-notes/herdr-and-hermes-2026-10-02.md` (Part 3's
  Switches and V1, and the third round of answers).
- **Classification / tier:** feature, small. Two defaults, one gate in a Marley module, one new
  section on the Marley page. Zed-side changes are additive and sit in three files whose
  `zed-touchpoints.md` rows exist. No new crate, no new dependency.
- **Recall (§18.3):**
  - AD-claude-480-marley-drives-voxtype-and-follows-its-status-001: the status is followed from the
    first frame that draws a microphone, and an ended status restarts only on a toggle. So a check
    placed before `follow_once_drawn` makes off start nothing.
  - #480's notes: the follower task's drop kills `voxtype status` (`kill_on_drop`); REQ-001's "no
    microphone without Voxtype" was never shot, because Voxtype is in `/usr/bin` on every PATH. The
    switch gives this ticket a bar with no microphone while Voxtype is installed.
  - AD-claude-633-rustys-server-is-offered-where-installed-as-a-default-001 (`rusty_tools` "on by
    default"): this ticket revises that default. The offer and withdrawal path stays as it is, and
    `633-02-off` already proved the withdrawal live.
  - AD-claude-565-...-off-by-default-001: off means no request, no key and no file; the model for
    "off starts no `voxtype`".
  - L-claude-633-scenarios-copy-the-users-settings-so-defaults-reach-real-services-001: the
    harness turns such defaults off in each run's copy (D7).
  - L-claude-607-a-settings-edit-after-marleys-own-write-does-not-reload-001: the scenario edits
    the run's settings from outside only, and never lets Marley write the file first.
  - L-claude-456-acting-around-zeds-own-settings-observers-001: an observer registered in
    `marley_workbench::init` runs before every view's, so `voice.rs` stops the follower before the
    bars redraw.
  - #632's notes: no page toggle for `embedded_harness`, since there is no harness section yet.
  - `docs/marley/rusty-in-marley.md` (the Rusty-in-Marley plan, drafted the same day, not yet
    committed): its R-D0 makes `marley.rusty { enabled, connection, agent_tools }` and says
    `agent_tools` "is today's `marley.rusty_tools` (#633), off by default after #642, and moves
    into this block when R1 lands". So this ticket keeps the key where it is and flips its
    default; it does not pre-empt R1's block.
  - #633's notes: the Settings window's search found the Rusty toggle by its description; the MCP
    Servers page opens by a key bound in the run's keymap.
  - Brain (`rusty-cli brain search`, read only):
    `decisions/marley-drives-voxtype-and-follows-its-status` (follow-up due 2026-10-07) and
    `decisions/marley-offers-rusty-mcp-to-zeds-agents-where-it-is-installed`. This ticket revises
    both. Promotion asks the brain (`brain ask`); Complete follows both up as revised.
- **Discovery:**
  - `crates/marley_workbench/src/voice.rs:49-73`: `init` registers `ToggleDictation` on every
    workspace (`:52`): with `voxtype` it runs `toggle`, and without it `show_error`. It looks up
    `voxtype` once, off the main thread (`:65`). `follow_once_drawn` `:79`, `toggle` `:88`,
    `follow` `:104` (sets `following`, stores the follower; its end resets both).
  - `crates/marley_workbench/src/agent_bar.rs:329-352`: `microphone` returns `None` without
    `Voice` or `voxtype`, then calls `follow_once_drawn` (`:335`); called at `:212` and placed at
    `:254`, after Rich Input, before the plugin chip and the notify chip. It draws only under a
    terminal whose foreground is an agent.
  - `crates/marley_workbench/src/marley_workbench.rs`: `ToggleDictation` and its doc `:188-190`
    (the palette shows it); `MarleySettings` `:341-409`, `rusty_tools: RustyTools` `:375-376`;
    `EmbeddedHarness::from_setting` `:463-479` ("off unless it is on"); `RustyTools::from_setting`
    `:481-497` ("offered unless it is off"); `from_settings` `:544-649` (`rusty_tools` `:585`,
    `system_one` `:640`); `rusty::init` `:770`, `voice::init` `:785`.
  - `crates/marley_workbench/src/rusty.rs:1-10` (module doc: "`marley.rusty_tools` turns it off"),
    `init` `:34-38` observes the settings store, `offer` `:41-54` looks for `rusty-mcp` only while
    wanted, `settle` `:58-90` inserts or removes `context_servers.rusty` in Zed's defaults.
  - `crates/settings_content/src/marley.rs`: `embedded_harness` `:82-87`, `rusty_tools` `:88-92`
    ("Default: true"), `system_one` `:167-169`, `MarleyPushSettingsContent` (the derives a small
    block carries), `SystemOneSettingsContent` `:252-293` with `enabled`.
  - `assets/settings/default.json`: the `marley` block `:1664-1837`; `embedded_harness: false`
    `:1703-1706`; `rusty_tools: true` `:1751-1753`; `system_one.enabled: false` `:1793-1797`.
  - `crates/settings_ui/src/marley_page.rs`: `marley_page()` `:10-22` chains Layout, Agents,
    Terminal, Push, System One and Privacy; Rusty Tools for Agents `:345-363` (Agents);
    System One's master toggle `:643-670`, the pattern for a nested `enabled`.
  - `crates/settings_ui/src/settings_ui.rs:215-250`: a field's shown value and its reset come from
    `raw_default_settings`, so `default.json` must carry each key; `:5001` `render_toggle_button`
    handles any `bool` field, so no renderer is added (the `settings_ui.rs` row is not touched).
  - `crates/marley_workbench/src/system_one.rs:1225-1232` (`check_toast`: "System One is off. Turn
    it on in the System One section of the Marley settings.") and `:1252-1257` (`show_toast`,
    `NotificationId::unique::<SystemOne>()`): Marley's way of answering an action whose feature
    is off.
  - `crates/marley_workbench/src/block_headers.rs:41-48`: a settings-store observer comparing a
    cached value and refreshing windows on a change.
  - `crates/terminal_view/src/terminal_view.rs:507` and `:925-956`: each `TerminalView` observes
    the settings store and ends `settings_changed` with `cx.notify()`, so its footer, the agent
    bar, redraws on any settings change.
  - `script/e2e.sh:627-642`: copies the user's settings into the run's profile and writes
    `marley.rusty_tools: false`. With the default false, that line now matters only for a user
    whose own settings turn Rusty's tools on: without it, every scenario would start that user's
    real `rusty-mcp` on their data. It stays, with its comment updated.
  - `script/e2e/480-voice-input.sh` relies on the microphone showing whenever its fake is on the
    PATH; with Voice off by default it needs `marley.voice.enabled` set true (a `set_setting`
    helper like 633's). `script/e2e/633-rusty-tools-for-zeds-agents.sh` already sets
    `rusty_tools` true and is unaffected.
  - `crates/zed/src/zed.rs:6236`: Zed's test setup sets `marley.rusty_tools = Some(false)` (#634);
    it matches the new default and stays.
  - `crates/marley_workbench/guide/index.html:920`, `:975-983`, `:1648`: the in-app guide's agent
    bar line, its Dictation article and its palette row. The file is under `crates/marley_*`, so
    the commit receipt fingerprints it (`.claude/hooks/lib-hook-helpers.sh:52-54`): edit it in the
    Code phase, or run `--diff` again at Complete.
  - Scenarios that click the agent bar by coordinates measured with the microphone present:
    `script/e2e/547-claude-code-events-slice-2.sh:19-20` (in the golden set; the plugin chip, X 520,
    its left end at 452) and `script/e2e/552-codex-and-opencode-notifications.sh:19-20` (not in the
    golden set; X 460). See Risks.
- **Decisions:** D1 to D7 in the spec. In short: `marley.voice.enabled` as the master switch of the
  future voice block, so it grows with no migration; both off, read "off unless on", with
  `default.json` carrying `false`; off starts no `voxtype` and the follower is dropped; the action
  answers with System One's kind of toast instead of being hidden; live through the settings
  store's observers; a Voice section with one Voice toggle; the harness turns both off in each
  run's copy.

### Design
- **Approach.**
  - `settings_content`: `pub voice: Option<MarleyVoiceSettingsContent>` in `MarleySettingsContent`,
    after `push` or beside `system_one`, documented "Voice in Marley (#480, #642): off until it is
    turned on". `MarleyVoiceSettingsContent { enabled: Option<bool> }` with `with_fallible_options`
    and the derives `MarleyPushSettingsContent` has; `enabled`'s doc names the microphone and the
    action, and "Default: false". `rusty_tools`' doc says "Default: false".
  - `default.json`: `"voice": { "enabled": false }` in the `marley` block with a comment (off until
    you turn it on; on, the agent bar shows a microphone where Voxtype is on your PATH and `marley:
    toggle dictation` dictates through it; off, no microphone and no `voxtype` started), and
    `"rusty_tools": false` with its comment saying it is off until turned on.
  - `marley_workbench.rs`: `MarleySettings::dictation: Dictation`, `Dictation { On, Off }` with a
    `const fn from_setting(enabled: Option<bool>)` that is `On` only for `Some(true)`, read from
    `marley.voice.enabled`. `RustyTools::from_setting` becomes `Some(true) => Offered, _ => Off`,
    its doc "offered only when it is on". `ToggleDictation`'s doc adds "while Voice is on in the
    Marley settings" (the palette shows action docs).
  - `voice.rs`: the action checks `MarleySettings::get_global(cx).dictation` first; `Off` shows a
    toast, `Toast::new(NotificationId::unique::<Voice>(), "Dictation is off. Turn it on in the
    Voice section of the Marley settings.")`; `On` keeps #480's branches. `Voice` gains the switch
    it last saw. `init` adds `cx.observe_global::<SettingsStore>`: when the switch changes to off,
    the follower is taken out (dropping it ends `voxtype status --follow`), `following` is false and
    the state idle; when it changes to on, nothing starts until a microphone is drawn, since
    `follow_once_drawn` starts a follower when there is none. A dropped task never reaches the code
    after its `read_status`, which is why the observer resets `following` and the state itself.
    The module doc says the switch gates everything.
  - `agent_bar.rs`: `microphone` returns `None` while `dictation` is `Off`, before reading `Voice`
    and before `follow_once_drawn`; its doc says "where Voice is on and Voxtype is on the PATH".
  - `rusty.rs`: the module doc says `marley.rusty_tools`, off by default, turns it on.
  - `marley_page.rs`: `voice_section() -> [SettingsPageItem; 2]`: a `SectionHeader("Voice")` and
    a `SettingItem` titled Voice, `json_path: Some("marley.voice.enabled")`, `files: USER`, `pick`
    and `write` through `marley.voice` as System One's toggle goes through `marley.system_one`. Its
    description: dictate into a terminal through Voxtype, where it is installed, with the agent
    bar's microphone and `marley: toggle dictation`; off, Marley shows no microphone and starts no
    voxtype; Marley never touches the audio. `marley_page()` chains it after `push_section()`.
    Rusty Tools for Agents stays as it is.
  - `script/e2e.sh`: the inline Python writes `marley.rusty_tools = False` and
    `marley.voice.enabled = False` into the copy (`setdefault("voice", {})`, so later keys a user
    set stay); the comment says both are off by default since #642 and the copy turns them off for
    a user who turned them on.
  - `script/e2e/480-voice-input.sh`: a `set_setting` helper (633's) and `set_setting
    marley.voice.enabled true` in `setup`; its header says so.
  - `crates/marley_workbench/guide/index.html`: the agent bar line ("the microphone where Voice is
    on and Voxtype is installed"), the Dictation article's first step (turn on Voice on the Marley
    page), the palette row; in the Code phase (the receipt binds it).
- **File manifest.**
  - Marley: `crates/marley_workbench/src/marley_workbench.rs`, `voice.rs`, `agent_bar.rs`,
    `rusty.rs`; `crates/marley_workbench/guide/index.html`; `script/e2e.sh`;
    `script/e2e/480-voice-input.sh`; `script/e2e/642-dictation-and-rusty-tools-off-by-default.sh`
    (new, Test phase).
  - Zed: `crates/settings_content/src/marley.rs` (crate `settings_content`): `voice` and
    `MarleyVoiceSettingsContent { enabled }`, the `rusty_tools` doc's default.
    `assets/settings/default.json`: `marley.voice.enabled: false`, `marley.rusty_tools: false`.
    `crates/settings_ui/src/marley_page.rs` (crate `settings_ui`): the Voice section and its
    chain. `crates/settings_ui/src/settings_ui.rs`: no change (a `bool` toggle needs no renderer).
- **The ledger rows it extends** (`docs/marley/zed-touchpoints.md`, written before the code, §14):
  - `crates/settings_content/src/marley.rs` (`:59`): append "`voice:
    Option<MarleyVoiceSettingsContent>` and `MarleyVoiceSettingsContent { enabled }`, the master
    switch of Marley's voice features, off by default: the agent bar's microphone and `marley:
    toggle dictation` (#642); `rusty_tools` off by default (#642)".
  - `assets/settings/default.json` (`:67`): "`marley.rusty_tools: true` with its comment (#633)"
    becomes "... (#633), `false` since #642", and append "`marley.voice: { enabled: false }` with
    its comment (#642)".
  - `crates/settings_ui/src/marley_page.rs` (`:63`): append "a Voice section after Push with its
    Voice toggle (`marley.voice.enabled`, #642)".

### Visual check plan
The scenario `script/e2e/642-dictation-and-rusty-tools-off-by-default.sh`, `compositor sway`.
Setup: a scratch repository opened with `open_path`; `terminal_env HOME` with a `.bashrc` that
becomes `claude` by name (`exec -a claude`, a dot a second, as 480's); in `$E2E_WORK/bin`, first on
the PATH, 480's fake `voxtype` with one more line that appends `$*` to a `calls` file, and 633's
stand-in `rusty-mcp`; 633's keymap binding Ctrl+Alt+Shift+M to `zed::OpenSettingsAt { path:
"context_servers" }`. Before Marley starts: `expect` that the copy holds `marley.rusty_tools`
false and `marley.voice.enabled` false (REQ-011), then delete both and any `context_servers.rusty`
(a small `del_setting` beside 633's `set_setting`). The process check is a shell function that runs
`pgrep -f` on the fake's `tail -n +1 -f <bin>/status`, never `bash -c` with the pattern in its own
command line.

| REQ | Scenario | Shot |
|---|---|---|
| REQ-001 | Trust the project; the stand-in's agent bar | `642-01-no-microphone`: no microphone after Rich Input |
| REQ-002 | `calls` empty after 01 and 02; after 09, no status `tail` left | the checks' `pass` lines in the run's output |
| REQ-003 | `marley: toggle dictation` from the palette | `642-02-dictation-is-off`: the toast's words |
| REQ-004 | `marley: open settings`; type "Voxtype" in the window's search | `642-03-voice-toggle-off`: the Voice item, its switch off |
| REQ-005 | The search cleared; type "Rusty" | `642-04-rusty-toggle-off`: Rusty Tools for Agents, off |
| REQ-006 | Ctrl+Alt+Shift+M | `642-05-no-rusty`: `marley` alone |
| REQ-007 | `set_setting marley.rusty_tools true`; settle | `642-06-rusty-on`: `rusty` with a green dot |
| REQ-008 | Close the Settings window; `set_setting marley.voice.enabled true`; settle | `642-07-microphone-on`: a grey microphone; `calls` holds `status --follow` |
| REQ-009 | `marley: toggle dictation` | `642-08-recording`: red; `calls` holds `record toggle`; toggle again, wait for idle |
| REQ-010 | `set_setting marley.voice.enabled false`; settle | `642-09-microphone-off`: no microphone |
| REQ-011 | The setup's two `expect` lines | the checks' `pass` lines |

Not reached by a scenario: a real dictation (speech, §7) and a real Rusty's tool call (it would
write into the user's brain); both are #480's and #633's, unchanged. The Settings window's own
switches are not clicked: a click would be Marley writing `settings.json`, after which the
scenario's outside edits stop reloading (L-607), and the window's write path is Zed's
`update_settings_file`, which this ticket does not change.

### Risks
- **Chip coordinates.** 547 (golden) and 552 click the agent bar's chips at coordinates measured
  while the microphone showed (Voxtype is on every PATH here). With the microphone hidden, the
  chips move left by its width, about 28 px (a small `IconButton` and the bar's `gap_1p5`). X 520
  against 547's chip, whose left end was 452, and X 460 against 552's should still fall inside the
  chips, but neither runs per ticket (§7). The Test phase notes it, and `just regress` shows 547.
- **The search's focus.** The scenario types into the Settings window's search as it opens; 633's
  run did so, which is evidence but not proof. Promotion confirms it.
- **The process check.** `pgrep -f` wrapped in `bash -c` matches the wrapper (a known trap); the
  check runs as a function.
- **Users lose two features on update.** Anyone who used the microphone or Rusty's tools without
  setting anything finds them off, Chad included. The CHANGELOG's Changed entry, the guide and the
  walkthrough say how to turn each on.
- **`rusty_tools` moves later.** Rusty-in-Marley's R1 moves the key into `marley.rusty`. That
  move is R1's settings migration, made smaller by this ticket: with the default off, only users
  who set `rusty_tools: true` need carrying.
- **A recording under way when Voice turns off.** Marley stops following and shows nothing; the
  recording is Voxtype's and ends with Omarchy's keys or Voxtype's own stop. D3 accepts it.
- **The receipt.** The in-app guide page, `script/e2e.sh` and the scenarios are fingerprinted by
  the commit receipt; a change to any of them after the Code phase's green needs `--diff` again
  before the commit (Complete step 5 says so).

### User-facing docs to update at Complete (P4)
- `docs/marley/guide.md`: `:34` (the feature table: "rich input, Attach File and dictation"),
  `:53-54` (Voxtype as an optional install, for dictation), `:513` (the bar's left side: "the
  microphone where Voxtype is installed"), `:565-572` (The microphone: shown wherever Voxtype is on
  the PATH; the action's error), `:911-916` (Rusty's tools: offered wherever `rusty-mcp` is, and
  `"rusty_tools": false` turns it off), `:1707` (the palette table's toggle dictation), `:1716-1720`
  (the Settings page's sections) and the keys block `:1724-1744` (no `voice` or `rusty_tools` yet).
- `docs/marley/walkthrough.md`: `:301-302` (stop 1.4: "six sections: Layout, Agents, Terminal,
  Push, System One and Privacy" becomes seven, with Voice), `:72-78` (stop 0.2's check: `voxtype`
  for 5.8), `:789-792` (stop 5.1: "a microphone when Voxtype is installed"), `:847-851` (stop 5.8
  The microphone: needs Voice turned on first), `:1589` (the palette table). No stop covers Rusty's
  tools (#633 added none).
- `docs/marley/tutorial-outline.md`: `:21` (Voxtype for the microphone), `:257-262` (lesson 20's
  microphone).
- `crates/marley_workbench/guide/index.html`: `:920`, `:975-983`, `:1648`, changed in the Code
  phase (above).
- `README.md` and `docs/marley/README.md`: checked, nothing on the microphone or Rusty's tools.
- Architecture (§21): `docs/marley_architecture/marley_workbench.md` `:1254-1266` (Voice) and
  `:1821-1831` (Rusty's tools, including the e2e line); `docs/marley/three-prong-plan.md` `:128`
  (the T7 row's T7d) and `:259` (the C2 row); the three `zed-touchpoints.md` rows checked against
  what shipped; `CHANGELOG.md` under Changed.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION §3, §7, §14, §18, §19, §20 and the templates.
- [x] Read the request and Chad's answers in the design note (Part 3 and the third round).
- [x] Recall: the knowledge ledgers (ADs for #480, #565 and #633; L-633, L-607, L-456), the
      completed pipelines 480, 632 and 633, and a read-only brain search.
- [x] Discovery with file:line: voice, the agent bar, `MarleySettings`, `rusty.rs`, the settings
      content, `default.json`, the Marley page, the settings UI's default handling, System One's
      toast, the observers, the e2e harness, the scenarios and golden coordinates, the in-app guide.
- [x] Prior-art sweep, three legs: Orca's, Warp's and Zed's behavior maps; Hermes and Voxtype;
      Zed's crates (`settings_ui`, `terminal_view`, `agent_ui`'s palette filter) and Marley's own.
- [x] The setting's shape locked with its reason (D1), and how each change is live (D5).
- [x] Spec: scope, Reference (§20), Prior art, UI proof, D1 to D7, eleven EARS rows, phase plan.
- [x] Design: approach, file manifest by crate, the three touchpoint rows to extend, the visual
      check plan, risks, the user docs for Complete.
- [x] Docs only: the ticket doc and this pair; no BACKLOG.md, no `active/`, no source, no cargo.

## Phase 1 — Promotion (2026-10-04)
- **Promoted** from `queued/` into `active/`; the BACKLOG row is gone and the ticket is
  in-progress. Pre-flight green: cargo 1.98.1, the gate, the e2e runner, cargo-shear 1.13.4, the
  hooks, no other active pipeline, the README marker present, cargo idle, `/mnt/fast` 280G free.
- **Seams re-verified** against the tree at d70bb7fc21: `voice.rs` `init` `:49-73`,
  `follow_once_drawn` `:79`, `toggle` `:88`, `follow` `:104`; `process.rs`'s `following` keeps
  `kill_on_drop(true)` (`:95`); `agent_bar.rs` `microphone` `:331-352`; `MarleySettings`
  `:341-409`, `EmbeddedHarness` `:464-479`, `RustyTools` `:481-497`, `from_settings` `:547`,
  `rusty::init` `:770`, `voice::init` `:785`; `rusty.rs` `init` `:34-38`, `offer` `:41-54`;
  `settings_content/src/marley.rs` `rusty_tools` `:88-92`, `system_one` `:169`,
  `MarleyPushSettingsContent` `:243`, `SystemOneSettingsContent` `:256`; `default.json`
  `embedded_harness` `:1706`, `rusty_tools` `:1753`, `system_one` `:1796`; `marley_page.rs`
  `marley_page()` `:10-22`, Rusty Tools for Agents `:345-363`, `push_section` `:558`,
  `system_one_section` `:643`; `system_one.rs` `check_toast` `:1225`, `show_toast` `:1252`.
  All as the queued notes say, give or take a line.
- **One change to the design:** `script/e2e.sh` has had `profile_setting <key.path> <json>` since
  #635 (`:562-580`), the same Python 633's own `set_setting` carries. 480's scenario calls it
  (`profile_setting marley.voice.enabled true` in `setup`) instead of a helper of its own, and
  642's scenario calls it for its edits while Marley runs too; only deleting a key needs a
  helper of the scenario's own (`drop_setting`, a name the harness does not use).
- **The search's focus, confirmed:** `SettingsWindow::new` focuses the search bar as it builds
  (`settings_ui.rs:2064-2066`), so the scenario types its query as the window opens; Ctrl+A
  then selects the query for the next one.
- **Brain:** `rusty-cli brain ask` (consultations `91c1551f35fb489db5ba6d8bc42feb70` and
  `0f5d097a1b784df1bd851191423d2279`) returned only due follow-ups on other work;
  `brain search` finds the two decisions this ticket revises (above). Nothing new on the seam.

## Phase 2 — Code (2026-10-04)
- **Built, to the manifest:**
  - Ledger first: the three `zed-touchpoints.md` rows (`settings_content/src/marley.rs`,
    `marley_page.rs`, `default.json`) widened before any Zed path was written.
  - `settings_content/src/marley.rs`: `voice: Option<MarleyVoiceSettingsContent>` before
    `system_one`, and `MarleyVoiceSettingsContent { enabled }` with the push block's derives;
    `rusty_tools`' doc says off until turned on, "Default: false".
  - `default.json`: `"voice": { "enabled": false }` with its comment, before `system_one`;
    `"rusty_tools": false`, its comment saying so.
  - `marley_workbench.rs`: `MarleySettings::dictation: Dictation`, `Dictation { On, Off }` read
    "off unless on" from `marley.voice.enabled`; `RustyTools::from_setting` offers only on
    `Some(true)`; `ToggleDictation`'s doc says "while Voice is on in the Marley settings".
  - `voice.rs`: the action answers `Off` with the toast "Dictation is off. Turn it on in the Voice
    section of the Marley settings." (`NotificationId::unique::<Voice>()`) before looking at
    Voxtype; `init` observes the settings store and, while off, `stop_following` drops the
    follower (its `kill_on_drop` child ends) and resets `following` and the state, since the
    dropped task never reaches its own reset. The module doc says the switch gates it all.
  - `agent_bar.rs`: `microphone` returns `None` while `Off`, before reading `Voice` or calling
    `follow_once_drawn`.
  - `rusty.rs`: the module doc says the setting, off by default, turns the offer on.
  - `marley_page.rs`: `voice_section()`, a Voice header and a Voice toggle on
    `marley.voice.enabled`, chained after Push.
  - `script/e2e.sh`: the copy writes `marley.rusty_tools` and `marley.voice.enabled` false; the
    comment says why with both off by default. `480-voice-input.sh`: `profile_setting
    marley.voice.enabled true` in `setup`.
  - The in-app guide: the agent bar line, the Dictation article (`#480 · #642`, a first step that
    turns Voice on) and the palette row.
- **Deviations:** (1) 480's scenario uses the harness's `profile_setting` (#635) rather than a
  helper of its own (Promotion, above). (2) The visual check's scenario,
  `script/e2e/642-dictation-and-rusty-tools-off-by-default.sh`, was written in this phase, before
  the gate, because the commit receipt fingerprints `script/e2e/*.sh` and gate:11 lints them: one
  gate run covers it instead of a second `--diff` at Complete. The Test phase runs it and may still
  change it, which would need `--diff` again.
- **Review of the diff** against REQ-001 to REQ-011: every path to `voxtype` is behind the switch
  (the microphone, its click, and the action are the only callers of `follow_once_drawn` and
  `toggle`; the PATH lookup reads directories and starts nothing). The observer touches only the
  `Voice` global, no entity, so nothing re-enters; the action handler shows the toast on the
  workspace it was handed. `rusty::offer` with the switch unset finds nothing and `settle(None)`
  returns at once, so no `rusty` entry is ever added. No test in the tree reads either default
  (`zed.rs:6236` sets `rusty_tools` false, now the default). Provenance: Marley's own patterns
  (`EmbeddedHarness`, System One's toast and toggle); nothing from Warp. Upstream discipline: the
  three Zed files are Marley's own additions with rows. No defect found.
- **Checks:** `cargo check -p marley_workbench -p settings_ui` green (46 s); `shellcheck -S info`
  on the changed scripts and the new scenario clean.
- **Gate, run 1** (`just gate-diff`, log in the scratchpad): RED on three. gate:2, two clippy
  lints in `marley_workbench`: `too_long_first_doc_paragraph` on `voice::init`'s doc (split into
  a one-sentence summary and a second paragraph) and `too_many_lines` on
  `MarleySettings::from_settings` at 104 of 100 (the five-line read of `marley.voice.enabled`
  moved into `Dictation::from_content(marley)`, the shape `ResumeAgents::from_content` already
  has; the function is now 100). gate:7, `cargo-audit`: seven advisories published 2026-10-02
  against wasmtime and wasmtime-wasi 48.0.3, Zed's extension host runtime (RUSTSEC-2026-0321 to
  -0327), not from this change. Fixed at the source as #511 did: the family's patch release
  48.0.5, which needed cranelift 0.135.5 and the wasm-tools crates (`wasmparser`,
  `wasm-encoder`, `wasmprinter`, `wasm-metadata`, `wit-component`, `wit-parser`) at 0.254.2 in
  the same `cargo update` (wasmtime 48.0.5 asks `^0.254.1`, which is why a family-only update
  left it at 48.0.3 with "available: v48.0.5"); lockfile only, `cargo audit` exit 0 after it. The
  `Cargo.lock` row and the CHANGELOG's Security entry say so. A separate commit for the bump was
  refused by the commit hook (the lockfile is gate-defining, so it needs a receipt too), and a
  receipt binds HEAD, so the bump rides in #642's commit, as #511's did. The receipt step was red
  because files changed during the run. The other 14 gates passed.
- **Gate, run 2** (`just gate-diff` on the settled tree): `GATE GREEN [diff]`, 17 passed, the
  receipt written. The gate's clippy scope does not build wasmtime; the Test phase's `just build`
  compiles 48.0.5 into the debug `marley` the scenario runs.

## Phase 3 — Test (2026-10-04)
- **Build:** `just build`, the debug `marley` with this change and wasmtime 48.0.5 (ten wasmtime
  crates compiled; the gate's clippy scope never builds them), 2 m 22 s.
- **Run 1:** `just e2e script/e2e/642-dictation-and-rusty-tools-off-by-default.sh` (`compositor
  sway`) stopped at the check after `642-07`: "the status process runs: FAIL". Two faults of the
  scenario, none of Marley's: (1) the process check's `pgrep -f "tail -n +1 -f …"` reads a
  pattern, in which ` +1` means one or more spaces then `1`, so the literal `+` never matched,
  though `calls` already held `status --follow`; the `+` now sits in a bracket. (2) `642-03`
  showed the Settings window's search empty: `marley: open settings` opens the window with its
  page list focused (the Marley entry outlined), so the typed "Voxtype" went nowhere; the
  promotion's reading of `SettingsWindow::new` (the search focused as it builds) is overridden by
  the page the action opens. The scenario now presses Ctrl+F (`search::FocusSearch` in the
  `SettingsWindow` context) first. Shots 01, 02 and 05 to 07 of run 1 already showed what run 2
  shows.
- **Run 2:** exit 0, every check passed (the copy's two switches off; no `voxtype` with Voice
  unset and after the action; the status followed, its process running, `record toggle` run; the
  process ended after Voice turned off). `calls` read `status --follow --format json`, `record
  toggle`, `record toggle`. Focus report: "hyprland: 0 Marley windows before the run, 0 after; the
  run added no rule and did not reload it"; the run's own sway stopped with its Marley. Every
  shot is Marley's window in the run's sway; none was deleted.
- **The shots, read:**
  - `642-01-no-microphone` (REQ-001, REQ-002): the stand-in Claude Code's agent bar reads Claude
    Code, `+`, the pencil, then Connect Claude Code to Marley and the folder: no microphone, with
    the fake `voxtype` first on the PATH; `calls` empty.
  - `642-02-dictation-is-off` (REQ-003, REQ-002): after `marley: toggle dictation`, a toast at the
    bottom right: "Dictation is off. Turn it on in the Voice section of the Marley settings."; the
    bar unchanged; `calls` still empty.
  - `642-03-voice-toggle-off` (REQ-004): the Settings window searched for "Voxtype": Marley ›
    Voice in the list; the page's Voice header and its one item, Voice, with the description
    naming Voxtype, the microphone and `marley: toggle dictation`; its switch off.
  - `642-04-rusty-toggle-off` (REQ-005): searched for "Rusty": Marley › Agents, Rusty Tools for
    Agents, its switch off.
  - `642-05-no-rusty` (REQ-006): User / AI / General / MCP Servers: `marley` alone, though the
    stand-in `rusty-mcp` is first on the PATH.
  - `642-06-rusty-on` (REQ-007): `marley.rusty_tools` set true from outside: `rusty` listed under
    `marley`, its dot green (cropped and enlarged to read it), no restart.
  - `642-07-microphone-on` (REQ-008): the Settings window closed and `marley.voice.enabled` set
    true: a muted microphone after the pencil; `calls` holds `status --follow`.
  - `642-08-recording` (REQ-009): `marley: toggle dictation`: the microphone red (cropped);
    `calls` holds `record toggle`.
  - `642-09-microphone-off` (REQ-010, REQ-002): `marley.voice.enabled` set false: the bar back to
    Claude Code, `+`, the pencil and the chip; the fake's status `tail` gone.
  - REQ-011: the setup's two `expect` lines passed (`marley.rusty_tools` and `marley.voice.enabled`
    false in the harness's copy).
- **Seen, not a criterion:** the dictation toast stays until closed, as System One's does (Zed's
  `Toast` without `autohide`). Kept, to match System One's answer to a feature that is off.
- **Not reached:** a real dictation (speech) and a real Rusty tool call (the user's brain), both
  #480's and #633's and unchanged here.
- **Pre-existing — not in scope:** none.
- **The receipt:** the scenario's fix changed a fingerprinted file, so `just gate-diff` runs again
  before the commit (run 3).
- **Gate, run 3** (after the scenario's fix): `GATE GREEN [diff]`, 17 passed, the receipt
  written over the tree that is committed.

## Phase 4 — Complete (2026-10-04)
- **Documented (§21):** `CHANGELOG.md`, Changed ("Dictation and Rusty's tools wait to be turned
  on") and Security ("wasmtime 48.0.5"); `docs/marley_architecture/marley_workbench.md` (Voice:
  the switch, the observer, `stop_following`; Rusty's tools: the default and the harness's two
  lines); `docs/marley/three-prong-plan.md` (T7d and C2); the three `zed-touchpoints.md` rows
  checked against what shipped, and the `Cargo.lock` row for 48.0.5; the user docs the plan
  listed: `docs/marley/guide.md` (the feature table, the optional install, the bar's left side,
  The microphone, Rusty's tools, the palette row, the Settings page's sections and keys),
  `docs/marley/walkthrough.md` (1.4's seven sections and both switches off, 5.1's bar, 5.8 with
  the toast and Voice turned on, the palette row), `docs/marley/tutorial-outline.md` (the
  install line, lesson 20); the in-app guide in the Code phase.
- **Knowledge (§19):** `AD-claude-642-voice-and-rustys-tools-wait-to-be-turned-on-001`;
  `L-claude-642-marley-settings-from-settings-is-at-clippys-line-cap-001`,
  `L-claude-642-marley-open-settings-focuses-the-page-list-001`,
  `L-claude-642-pgrep-reads-its-pattern-as-a-regular-expression-001`,
  `L-claude-642-a-wasmtime-patch-moves-with-the-wasm-tools-it-asks-for-001`. No `F-…`: the two
  faults Test found were the scenario's, and the clippy reds were lints.
- **Brain:** `brain decide` on consultation `91c1551f35fb489db5ba6d8bc42feb70`:
  `decisions/marleys-dictation-and-rustys-tools-for-zeds-agents-wait-to-be-turned-on` (follow-up
  2026-10-18); `decisions/marley-drives-voxtype-and-follows-its-status` and
  `decisions/marley-offers-rusty-mcp-to-zeds-agents-where-it-is-installed` followed up as
  revised, with it as successor.
- **Closed:** the ticket moved to `tickets/closed/`, its link at `completed/`; no BACKLOG row was
  left (promotion removed it). The pair archived to `pipeline/completed/`.
