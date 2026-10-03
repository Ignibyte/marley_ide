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
