# Rich input through the agent's own editor key — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-649-rich-input-through-the-agents-editor-key.md
- **Pipeline spec:** 649-rich-input-through-the-agents-editor-key.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-03)
- **Request:** Chad, 2026-10-02: "we need to brain storm integration into claude and codex using
  their tools instead of fighting them". The design note
  (`docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md`) lists rich input
  as a fight ("Rich input draws a Zed editor over Claude Code's prompt | `rich_input.rs` |
  Duplicates Claude Code's own Ctrl+G, which opens the prompt in `$VISUAL`/`$EDITOR`") and offers
  B4. Chad's answer 4, B4 "(a `marley edit` command as `$EDITOR`; the overlay only for agents with
  no editor key)": "Yes". This batch, in build order, is #648 B7, #649 B4, #650 and #651 B1, #652
  B2, #653 B3.
- **Classification / tier:** feature, size S to M. Rust in `marley_workbench`, `marley_mcp`,
  `marley_agent` and `marley_terminal`, a Python helper, three shell scripts, and the switch in
  three Zed paths that extend existing ledger rows. One slice. Queue.
- **What the discovery changed.**
  - Marley has no CLI and no subcommand dispatch. `just install` builds only the app binary
    (`script/install-marley:75`); `~/.local/bin/marley` is a launcher that `exec`s it; a second
    launch hands its paths over the datagram socket and exits (`crates/zed/src/main.rs:394-403`,
    "since Marley ships no CLI (#513)"). Zed's `cli` crate is unmodified and unbuilt. Marley's
    helpers are programs it writes to `<data_dir>/mcp/` and names by path in a variable:
    `marley-open-url` as `BROWSER` (#561). So the brief's `marley edit` becomes `marley-edit`,
    named by its absolute path (D1).
  - `zed --wait` waits for the tab's item to be released, not for a save, and its exit code is 0
    either way; Zed's `docs/src/reference/cli.md:266` claims the code reflects a save, which the
    code does not do. D2 keeps the code's behavior.
  - Zed's CLI cannot name a window: it picks one by worktree match (`open_listener.rs:797-832`,
    `workspace.rs:10899-10985`), and a temporary file is in no worktree. Marley's endpoint carries
    the caller's `MARLEY_TERMINAL_ID`, which names the workspace (D3).
  - The MCP server answers an app call within 30 s, so the helper waits in rounds of at most 20 s,
    as `terminal_run` does (`registry.rs:176-185`).
  - "The terminals Marley opens for agents" are the ones `start_in_terminal` opens with
    `agent: Some(_)` (`agents.rs:393-396`): the New Agent picker, the project's `+`, a launch
    config's agent item (`launch.rs:436-444`). A terminal a restart restores is a plain shell, and
    #540's resume types into it (`resume.rs:188-240`), so it has no `agent_env` today, git's
    prompts included. Out of scope; it keeps the overlay (D6).
  - Agent terminals get their variables at spawn, and the agent's line is typed into the shell
    after the user's files run (`agents.rs:409-428`), so an `export EDITOR=…` in `.bashrc` would
    win over a spawn variable. Marley's shell integration already consumes variables after those
    files; it re-exports Marley's editor there (D4).
  - The four agents Marley knows (`marley_agent.rs:44-57`) all have an editor key, so the overlay
    stays for terminals and states, not for agents. OpenCode's key is `ctrl+x e`, not Ctrl-G, and
    its own Ctrl-G is `messages_first` (D5).
  - The batch named `/srv/stacks/rustal-codex` for Codex's source; it was removed in a cleanup.
    Codex was read at the installed version instead, upstream `rust-v0.155.1` (`be2951e`), in a
    shallow clone in the /spec session's scratchpad (`codex-rs/tui`; the spec's Prior art cites
    its paths). Its editor key works only while the bottom pane shows no modal view, so Marley's
    Ctrl-G cannot answer one of Codex's approval prompts.
  - Claude Code on the box is 2.1.288, not the 2.1.287 the batch names; it updated itself on
    2026-10-02. The editor path is documented and needs no version check of #648's kind.
- **Recall (§18.3):**
  - Brain `decisions/marleys-rich-input-is-a-zed-editor-above-the-agent-bar` (#481) lists "Open the
    agent's own external editor (Claude Code's Ctrl-G)" as the rejected alternative. This ticket
    reverses it for terminals Marley gives its editor; Complete records the decision's follow-up as
    revised. `decisions/ctrlg-at-a-shell-prompt-opens-marleys-footer-editor-for-the-shell` (#624)
    is untouched: the shell's editor at a prompt stays.
  - AD-claude-561-marley-exports-its-opener-as-browser-in-every-local-terminal-001: the opener's
    shape (a program in the data directory, a variable, MCP with the endpoint's bearer, a
    fallback). Its "only when unset would never fire on this box" holds for `EDITOR` too (D4); its
    "a shell's own files still win" is reversed here, with the reason in D4.
  - F-claude-561-an-opener-on-the-default-endpoint-would-open-tabs-in-another-marley-001 and
    PR-claude-a-program-marley-writes-finds-the-marley-that-wrote-it-001: the helper reads the
    endpoint beside its own data directory, so an e2e profile's helper reaches the run's Marley.
  - F-claude-594-an-enter-sent-with-a-paste-was-read-as-part-of-it-001 and
    PR-claude-keys-after-a-paste-go-in-a-later-write-001: the reason for `AFTER_PASTE`. The
    editor path sends one key and no paste, so neither applies to it; the overlay keeps
    `paste_then`.
  - F-claude-549-a-rich-input-behind-another-tab-took-the-reference-001: text sent where the user
    cannot see it. The tab opens in front with the focus; the terminal comes back to the front
    when it closes.
  - PR-claude-defer-a-pane-change-out-of-an-items-own-event-001 and
    PR-claude-a-callback-zed-calls-mid-update-defers-its-entity-work-001: the release fires while
    the pane closes the item, so bringing the terminal to the front is deferred. The Ctrl-G action
    runs on the workspace, not inside the terminal view's update, so writing the key there is safe.
  - F-claude-595-the-footer-read-its-own-view-while-it-rendered-001: the bar's button runs from a
    click, not inside render; the check reads the `Terminal` it is given.
  - AD-claude-537 (git's prompts off in agent terminals) via `agent_env`: the same place gets the
    editor's variables. #537's scenario is the shape of this one (stand-ins first on the `PATH`,
    the New Agent picker, a New Terminal for contrast).
  - L-claude-481-gate-text-by-its-modifiers-not-prefer-character-input-001 and
    F-claude-481-the-rich-inputs-check-raced-the-echo-of-its-paste-001: about the overlay's keys
    and its scenario's reads; the overlay path is not changed.
  - The completed archive: #481, #549, #561, #537, #540, #575, #594, #624, #627 read for the
    paths above.
  - Brain search for `EDITOR`, `VISUAL` and the editor key found only the two decisions above.
    Promotion runs `brain_ask`.
- **Discovery:**
  - `crates/marley_workbench/src/rich_input.rs`: `init` 95-140, the `RichInput` action on the
    workspace 122-139 (`crate::shortcut_note::taken`, then `open_for`; `cx.propagate()` when no
    target); `target_of` 150-160; `open` 253-255 (the bar's button); `open_for` 259-313; `send`
    521-552 (`paste_then` and `\r`).
  - `crates/marley_workbench/keymap.json:26`: `"ctrl-g": "marley::RichInput"` in `Terminal`. It
    stays; the action decides.
  - `crates/marley_workbench/src/agent_bar.rs`: `agent_in` 140-145 (the foreground process's
    name); `rich_input_button` 290-306.
  - `crates/marley_workbench/src/terminal_drive.rs`: `AFTER_PASTE` 54, `paste_then` 313-325.
    Unchanged.
  - `crates/marley_workbench/src/agents.rs`: `Launcher` 57-91; `agent_env` 341-356 (git's prompts,
    ssh's askpass); `start_in_terminal` 363-430 (`env` chosen at 393-396, the terminal at 397-403,
    the typed line after the handshake at 409-428).
  - `crates/marley_workbench/src/mcp.rs`: the server's grants 96; `offer_browser_opener` 119 and
    325-338; `write_program_in` 351-362; `answer` 470-503; `caller_terminal` 759-768;
    `terminal_of` 770-786.
  - `crates/marley_workbench/bin/marley-open-url`: `endpoint_path`, `read_endpoint` with the
    loopback check, `request`, the session's `DELETE`.
  - `crates/marley_mcp/src/registry.rs`: `Family` 15-26 and `is_served` 41-46; `REGISTRY` 84; the
    terminal family's rows; `browser_open_url` 304-313 and its schemas 505-520; `tools_list_for`
    412-430; `tool_schemas` 436 (exhaustive per family). `crates/marley_mcp/src/clients.rs:79-93`:
    `permits`. `crates/marley_mcp/src/permission.rs:54-63`: a write tool needs its class granted.
    `crates/marley_mcp/src/marley_mcp.rs:94`: `APP_CALL_TIMEOUT_SECONDS`.
  - `crates/marley_agent/src/marley_agent.rs`: `AgentKind` 44-57, `program` 61-68,
    `display_name` 83-90, `GIT_PROMPTS_OFF` 149-153.
  - `crates/marley_terminal/src/shell_integration.rs`: the variable names 60-76
    (`AGENT_HISTORY_VARIABLE` 76). The scripts: `marley.bash` 22-29 (a consumed variable), 50-54
    (`~/.bashrc`); `marley.zsh` 34-36, 93-97 (`__marley_install`, the first prompt); `marley.fish`
    27-28, 50-51 (the first prompt, after `config.fish`).
  - `crates/terminal/src/terminal.rs`: `TerminalBuilder::new` 1219-1352; `MARLEY_TERMINAL_ID`
    1315-1333; `BROWSER` 1243-1250 and 1338-1343. `crates/project/src/terminals.rs:417-442`: the
    merge order, `terminal.env` (421) before the terminal's own extras (423), so `agent_env` wins
    over `terminal.env`. Neither file changes.
  - `crates/workspace/src/workspace.rs`: `open_paths` (method) 4097, `open_abs_path` 5078-5126,
    `open_path` 5128-5137 (`allow_preview` false), `open_path_preview` 5139 (the
    `last_active_center_pane`), `OpenOptions` 11008-11021. `crates/workspace/src/item.rs:1105-1111`:
    `on_release`. `crates/workspace/src/pane.rs:2406-2424`: the dirty-close prompt.
  - Zed's wait: `crates/cli/src/main.rs:68-72`; `crates/zed/src/zed/open_listener.rs:1015-1090`.
  - `crates/settings_content/src/marley.rs`: the block, `prompt_editor` 131-135 for the shape.
    `assets/settings/default.json:1674-1761`: the `marley` block.
    `crates/settings_ui/src/marley_page.rs`: the Agents section 114-435 (Rusty Tools for Agents at
    346).
  - The box: `~/.local/share/omarchy/default/bash/envs:2`,
    `export EDITOR="${EDITOR:-omarchy-launch-editor --inline}"`, and `:3`, `SUDO_EDITOR="$EDITOR"`.
    Marley's own environment carries `EDITOR=omarchy-launch-editor --inline` and no `VISUAL`.
  - Claude Code 2.1.288 (`~/.local/share/claude/versions/2.1.288`), Codex 0.155.1, Gemini CLI
    0.62.0 and OpenCode 1.18.31 installed; their editor paths are in the spec's Prior art.
- **Decisions:** D1 to D10 in the spec.

### Design
- **`marley_agent`** (pure): `AgentKind::editor_key(self) -> &'static [u8]`, the default key's
  bytes, with a doc line per agent naming where the default is documented. Every known agent has
  one; the type stays `&'static [u8]` rather than an `Option` until an agent without one is added.
- **`marley_terminal::shell_integration`:** `AGENT_EDITOR_VARIABLE = "MARLEY_AGENT_EDITOR"`. The
  scripts, each after the user's files:
  - bash, after the `~/.bashrc` block: when `MARLEY_AGENT_EDITOR` is set, `export
    VISUAL="$MARLEY_AGENT_EDITOR" EDITOR="$MARLEY_AGENT_EDITOR"`, then `unset MARLEY_AGENT_EDITOR`.
    A login shell's files (the remote branch) are not this case: the variable is set only for
    local terminals.
  - zsh: the value kept in a global at the top, as `__MARLEY_SSH_COMMAND` is, and exported in
    `__marley_install`.
  - fish: kept in a global at the top, as `__marley_nonce` is, and `set -gx` in the first-prompt
    block of `__marley_prompt`.
- **`marley_mcp::registry`:** `Family::Editor` (`"editor"`), `is_served` false. Rows:
  `editor_open` (Tier::Write, `editor.write`; input `{path}` absolute; output `{edit, file}`) and
  `editor_wait` (Tier::Read; input `{edit, wait_seconds}`, `wait_seconds` 1 to 20, default 20;
  output `{closed}`). Their arm in `tool_schemas`.
- **`marley_workbench::mcp`:** `"editor.write"` added to the grants at 96; `answer` sends
  `editor_*` to `agent_editor::answer`; the helper written beside the opener by
  `write_program_in`, its path published to `agent_editor` once written, and only when it holds no
  whitespace (D8), with a log line otherwise.
- **`marley_workbench::agent_editor`** (new, `src/agent_editor.rs`):
  - Globals: the helper's path (`Option<PathBuf>`); the terminals given the editor
    (`HashSet<EntityId>` of `Terminal`s, an `on_release` removing each); the edits by id
    (`HashMap<u64, Edit>`, an `Edit` holding the item's release subscription, the terminal view's
    weak handle, `closed`, and the waiters' senders).
  - `env_for(local, cx) -> Option<(&'static str, String)>` triples for `agent_env`; `wire(terminal,
    cx)` for `start_in_terminal`.
  - `takes_key(terminal, kind, cx) -> bool`: the switch on, the terminal given the editor, `kind`
    has a key. `press_key(view, kind, window, cx)`: `terminal.input(kind.editor_key())`, the view
    focused.
  - `answer(call, cx)`: `editor_open` resolves the caller with `mcp::caller_terminal`, checks the
    path, opens it with `open_abs_path` in the caller's window (`window_handle.update`), and on the
    item: `on_release` marks the edit closed, wakes its waiters, and with `window.defer` activates
    the terminal view in its pane and focuses it when it is still there. `editor_wait` answers at
    once when closed, else waits on a oneshot or the executor's timer for `wait_seconds`. An edit
    is dropped once its `closed: true` is given, or a minute after its tab closed with no waiter.
- **`marley_workbench::agents`:** `agent_env` takes the editor's variables from
  `agent_editor::env_for` when the project is local; `start_in_terminal` calls
  `agent_editor::wire` on the new terminal when it gave them.
- **`marley_workbench::rich_input`:** the `RichInput` action, for `Target::Agent(kind)` with
  `agent_editor::takes_key`, calls `press_key` and skips the shortcut note and the overlay; `open`
  (the button) does the same. Nothing else in the file changes; the shell's editor (#624, #627)
  and `send` stay.
- **`marley_workbench::marley_workbench`:** `mod agent_editor`;
  `MarleySettings::agent_editor_in_tab: bool` resolved from the content, false when unset.
- **The helper** (`crates/marley_workbench/bin/marley-edit`): `endpoint_path`, `read_endpoint` and
  `request` as the opener has them; one session from `initialize`; `editor_open`, the stderr line,
  `editor_wait` in a loop with a 25 s request timeout; the session's `DELETE` in a `finally`; exit
  codes 0 (closed), 1 (refused or unreachable, with the reason), 2 (no file argument).
- **Zed paths** (each extends its row in `docs/marley/zed-touchpoints.md`, with a `// Marley:`
  comment where the file takes code):
  - `crates/settings_content/src/marley.rs` (row 59): `agent_editor_in_tab: Option<bool>`, with its
    doc comment.
  - `assets/settings/default.json` (row 67): `"agent_editor_in_tab": false`, with a comment.
  - `crates/settings_ui/src/marley_page.rs` (row 63): the Agent Prompts in a Tab toggle in the
    Agents section, beside Rusty Tools for Agents.
- **File manifest:**
  - Marley: `crates/marley_agent/src/marley_agent.rs`;
    `crates/marley_terminal/src/shell_integration.rs` and
    `crates/marley_terminal/shell_integration/marley.{bash,zsh,fish}`;
    `crates/marley_mcp/src/registry.rs`; `crates/marley_workbench/src/agent_editor.rs` (new),
    `mcp.rs`, `agents.rs`, `rich_input.rs`, `marley_workbench.rs`;
    `crates/marley_workbench/bin/marley-edit` (new);
    `script/e2e/649-rich-input-through-the-agents-editor-key.sh` (new).
  - Zed: `crates/settings_content/src/marley.rs`, `assets/settings/default.json`,
    `crates/settings_ui/src/marley_page.rs`, each on its existing row. No new row, no new
    dependency, no change to `.config/spawn-sites.txt`: the helper is Python run by the agent,
    and Marley spawns nothing new.
- **At Complete:** CHANGELOG; `docs/marley_architecture/marley_workbench.md` (the Rich input
  section and the helpers beside the opener), `marley_mcp.md` (the editor family),
  `terminal_blocks.md` (the integration's variables); an AD for Marley's editor in agent terminals
  (D1, D4, D6); the brain decision's follow-up as revised.

### Visual check plan
- **Setup:** `compositor sway`. A HOME whose `.bashrc` sets `PS1='$ '`, puts `$E2E_WORK/bin` first
  on the `PATH`, and exports `VISUAL=user-visual EDITOR=user-editor`. The stand-in `claude` in
  `bin` (the spec's UI proof); `MARLEY_CLAUDE` names it. `profile_setting
  marley.agent_editor_in_tab true`. A profile keymap binding `ctrl-alt-shift-e` to
  `["zed::OpenSettingsAt", {"path": "marley.agent_editor_in_tab"}]`. A scratch repository opened
  with `open_path`. A scenario `set_setting` for the live change, as #633's has.
- **Rows:**
  - REQ-001: New Agent picker (Ctrl-Alt-N, `Claude Code`, Enter), settle; the stand-in's first
    lines read through `mcp_agent terminal-screen`; `expect` `VISUAL=*/mcp/marley-edit` and the
    same `EDITOR`; shot `649-01-agent-env`. zsh and fish by the review of the two scripts.
  - REQ-003, REQ-004: Ctrl-G, settle; shot `649-02-tab-open` (the tab `claude-prompt-1.md` in
    front, its text `from the agent`, the cursor in it); `expect` `keys.log` is `07`.
  - REQ-005: Ctrl-A, `edited in Marley`, Enter, `second line`, Ctrl-S, settle; shot
    `649-03-saved`; `expect` `stand-in.log` holds no `editor exited`.
  - REQ-006: Ctrl-W, settle; shot `649-04-read-back`; `expect` `editor exited 0`, both lines after
    `read back:`, no `submitted`, and `keys.log` still `07` alone.
  - REQ-007: Enter, no click; shot `649-05-submitted`; `expect` `submitted: edited in Marley`.
  - REQ-008: a click on the bar's pencil (`RICH_INPUT_X`, `RICH_INPUT_Y`, from the shots); shot
    `649-06-button`; `expect` a second `07`; Ctrl-W.
  - REQ-002: the palette's `workspace: new terminal`; `echo "$VISUAL $EDITOR"`; shot
    `649-07-plain-env`.
  - REQ-009: `claude` typed there; Ctrl-G; shot `649-08-hand-run-overlay` (the overlay's
    placeholder `A prompt for Claude Code`, the pane's tabs unchanged); Escape; Ctrl-C.
  - REQ-011: `MARLEY_TERMINAL_ID= <profile>/mcp/marley-edit $E2E_WORK/note.md; echo "exit $?"`;
    shot `649-09-refused`; `expect` `exit 1` and an unchanged `note.md`.
  - REQ-010: `set_setting marley.agent_editor_in_tab false`, settle; the picker again; Ctrl-G;
    shot `649-10-off` (`VISUAL=user-visual` and the overlay).
  - REQ-015: `ctrl-alt-shift-e`; shot `649-11-setting`.
  - REQ-012, REQ-013, REQ-014: the review. Only Claude Code's stand-in runs; Codex's, Gemini
    CLI's and OpenCode's keys and readers are taken from their source and bundles, and no
    scenario runs a real agent.
- **Not reached:** the four real agents with `marley-edit` (never run for a turn, and the
  scenario stays on stand-ins); a Don't Save close (Zed's own prompt, unchanged, by review).

### Risks
- A user's rebinding of an agent's key (Claude Code's `"ctrl+g": null`, Codex's keymap, Gemini
  CLI's key table, OpenCode's `keybinds`) makes Marley's key do nothing or something else. The
  switch off brings the overlay back; reading those files is Out.
- Gemini CLI takes its `/editor` choice before `$VISUAL`: a Gemini user who picked one gets that
  editor, not the tab.
- Gemini CLI tests the helper's whole path for `vi` and then adds `-i NONE`; the helper takes the
  last argument as the file, so a home such as `/home/david` still works.
- Programs in agent terminals read the editor too. Claude Code's Bash tool sets
  `GIT_EDITOR=true`. No crate in Codex 0.155.1's `codex-rs` sets `GIT_EDITOR`, so a Codex
  `git commit` with no `-m` and no `core.editor` runs `marley-edit`: a tab opens and git waits,
  or, inside a sandbox that closes loopback, the helper exits 1 and git stops the commit.
  `SUDO_EDITOR`, which Omarchy copies from `EDITOR` in the user's files, keeps the user's value.
  Recorded at Complete if the check shows more.
- The release fires inside the pane's close; anything that touches the pane or the workspace from
  the callback is deferred, or Marley panics (the gpui re-entrancy class of F-claude-595).
- A split of the tab makes a second item on the same buffer; the edit ends when the first item is
  released, as `zed --wait` does. Not handled.
- The helper killed (Ctrl-C in the agent's terminal) leaves the tab open; its later close ends an
  edit nobody waits for, which the expiry drops.
- An agent's draft with a trailing newline from Zed's `ensure_final_newline_on_save`: Codex trims
  it, Claude Code drops one, Gemini CLI keeps it. Seen, not fixed, unless the check shows a sent
  blank line.
- The scenario's Ctrl-A, Ctrl-S and Ctrl-W are Zed's Linux defaults (`editor::SelectAll`,
  `workspace::Save`, `pane::CloseActiveItem`); a profile keymap that rebinds them would break the
  run, so the scenario checks the tab's state in each shot.

### Checklist (no TaskCreate in this harness)
- [x] Read CONSTITUTION.md §3, §7, §14, §18, §19, §20, the brief and the design note whole.
- [x] Read what the ticket replaces: `rich_input.rs`, `terminal_drive.rs`'s paste wait,
  `agent_bar.rs`'s button, `keymap.json`'s Ctrl-G.
- [x] Discovery through Explore (§18.2): Marley's helpers and agent terminal variables, Zed's
  `--wait`, Codex's TUI editor.
- [x] Checked each agent's editor key and reader: Claude Code (docs and 2.1.288's bundle), Codex
  (upstream `rust-v0.155.1`'s `codex-rs/tui`), Gemini CLI (0.62.0's bundle), OpenCode
  (1.18.31's bundle, its keybinds page). Recorded what was not run.
- [x] Recall: knowledge ledgers, the completed archive, the brain.
- [x] Decided the user's own `EDITOR` (D4) and the switch's default (D10).
- [x] Reference (§20), Prior art's three legs, UI proof, Locked-In Decisions, EARS criteria,
  Phase Plan.
- [x] Design with the file manifest and the three ledger rows it extends.
- [x] Wrote the ticket, the spec and these notes; nothing else.
