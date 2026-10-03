---
pipeline_id: ddf358d0-34fb-4e38-ab7f-7a029c380cc5
ticket: docs/planning/tickets/open/TICKET-649-rich-input-through-the-agents-editor-key.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Rich input through the agent's own editor key"
type: feature
slice: prong 1 T7 (T7e, the rich input); Claude Code and Codex on their own tools, B4
references: [docs/planning/design-notes/claude-and-codex-on-their-own-tools-2026-10-02.md, docs/planning/pipeline/completed/481-rich-input.spec.md, docs/planning/pipeline/completed/561-browser-env-opener.spec.md, docs/planning/pipeline/completed/537-git-credential-prompts-off.spec.md, docs/planning/pipeline/completed/594-enter-after-an-agents-paste.spec.md]
---

## Title
Claude Code, Codex, Gemini CLI and OpenCode each open their prompt in `$VISUAL` or `$EDITOR` on a
key of their own and read the file back when the editor exits. Marley gives the terminals it opens
for agents its own editor, `marley-edit`, which opens the file in a center tab of that terminal's
workspace and returns when the tab closes, as `zed --wait` does. With `marley.agent_editor_in_tab`
on, Ctrl-G and the agent bar's Rich Input button send the agent its own editor key instead of
opening Marley's overlay, so the prompt comes back through the agent's own path: no paste, no
200 ms wait, no Enter, nothing sent until the user sends it. The overlay stays where the agent's
key cannot reach Marley's editor. Chad, 2026-10-02, on B4 ("rich input through the agents' own
editor key (a `marley edit` command as `$EDITOR`; the overlay only for agents with no editor
key)"): "Yes".

## Scope
### In
- **The helper:** `crates/marley_workbench/bin/marley-edit`, Python 3's standard library like
  `marley-open-url`, written to `<data_dir>/mcp/marley-edit` (0755) at start, beside the opener.
  - The file is its last argument; options before it (`+12`, Gemini CLI's `-i NONE`, a `--wait`)
    are ignored. It sends the file's absolute path, resolved against its working directory, so
    git's relative `.git/COMMIT_EDITMSG` works too.
  - It reaches the Marley that wrote it through `mcp-endpoint.json` beside its own data
    directory, `$MARLEY_MCP_ENDPOINT` winning
    (PR-claude-a-program-marley-writes-finds-the-marley-that-wrote-it-001), with the
    `Marley-Terminal` header from `MARLEY_TERMINAL_ID`, as the bridge sends it.
  - It calls `editor_open`, prints one line on stderr (the file's name, open in a Marley tab;
    closing the tab hands it back), then calls `editor_wait` until the edit has ended, and exits 0.
  - A refusal, or a Marley it cannot reach: the reason on stderr and exit 1, the file untouched.
- **The editor family in Marley's MCP server** (`marley_mcp::registry`): `Family::Editor`, not
  served in `tools/list` (as `fleet` and `session` are not), with `editor_open` (Write, grant
  class `editor.write`, given to Marley's own bearer beside `browser.write` and `terminal.write`)
  and `editor_wait` (Read: `edit`, `wait_seconds` at most 20, under the server's 30 s). Outside
  clients are refused both (`clients::permits` lists neither).
- **The app side** (`marley_workbench::agent_editor`, new):
  - `editor_open` finds the caller's terminal (`mcp::caller_terminal`) and refuses a call from no
    Marley terminal, a relative path, or a folder. It opens the file in that terminal's workspace
    with `Workspace::open_abs_path` (`OpenVisible::None`, with the focus; `open_path` never makes
    a preview tab) and answers at once with the edit's id and the file's name.
  - The edit ends when the tab's item is released (`ItemHandle::on_release`, as Zed's CLI waits).
    Saving alone does not end it, and Zed's Save, Don't Save and Cancel on a dirty close stay as
    they are.
  - When it ends, Marley brings the terminal the edit came from to the front of its pane with the
    focus, deferred out of the pane's update
    (PR-claude-defer-a-pane-change-out-of-an-items-own-event-001).
  - `editor_wait` answers `closed: true` once the edit has ended, else `closed: false` after its
    `wait_seconds`; an unknown id is refused.
- **Agent terminals:** while the switch is on, `agents::agent_env` gives a terminal opened for an
  agent in a local project `VISUAL`, `EDITOR` and `MARLEY_AGENT_EDITOR`, each the helper's path,
  once the helper is written and when its path holds no whitespace. `start_in_terminal` records
  the terminal as one Marley gave its editor, by the `Terminal`'s entity, dropped with it.
- **Over the user's shell files:** Marley's shell integration exports `VISUAL` and `EDITOR` again
  from `MARLEY_AGENT_EDITOR` once the user's files have run (bash after `~/.bashrc`; zsh in
  `__marley_install`; fish at the first prompt), then takes `MARLEY_AGENT_EDITOR` out of the
  environment, as it does `MARLEY_AGENT_HISTORY`. The name lives in
  `marley_terminal::shell_integration::AGENT_EDITOR_VARIABLE`.
- **The key:** `AgentKind::editor_key()` in `marley_agent` gives each agent's default editor key as
  bytes: Ctrl-G (`0x07`) for Claude Code, Codex and Gemini CLI; Ctrl-X then E (`0x18 0x65`) for
  OpenCode. In a terminal Marley gave its editor, with the switch on and such an agent in the
  foreground, `marley::RichInput` (Ctrl-G) and the agent bar's Rich Input button write that key
  into the terminal, give the terminal the focus, and open no overlay and no shortcut note.
  Everywhere else both open the overlay as #481 does.
- **The switch:** `marley.agent_editor_in_tab: Option<bool>`, off by default, in `settings_content`,
  `default.json` and `MarleySettings`, with a toggle, Agent Prompts in a Tab, in the Settings
  window's Marley page, Agents section. A change applies to agent terminals opened after it.
- `script/e2e/649-rich-input-through-the-agents-editor-key.sh`.

### Out (explicitly deferred)
- Send selection, Attach File, review notes and Send to Agent into the open tab: they paste into
  the agent's prompt as before (#549 inserts into the overlay while it is open). B3 (#653) replaces
  send selection's typed reference.
- Agent terminals a restart restores (#540, #575): they come back as plain shells without
  `agent_env`'s variables (git's prompts are on there too), so Ctrl-G opens the overlay there.
  Giving a restored agent terminal `agent_env` is its own ticket.
- Remote projects: the helper and the file live on this machine, so an agent over SSH keeps the
  overlay.
- Each agent's own rebinding of its key (`~/.claude/keybindings.json`'s `chat:externalEditor`,
  Codex's `tui.keymap.global.open_external_editor`, Gemini CLI's `input.openExternalEditor`,
  OpenCode's `keybinds.editor_open` and `leader`): Marley sends the default.
- Gemini CLI's own `/editor` choice, which it takes before `$VISUAL`.
- The switch on by default: after this ticket's check, on Chad's word.
- Closing the tab when the agent or the helper ends first: the tab stays, and its close then
  changes nothing.
- A `marley edit` subcommand of the app binary, or a `marley` CLI (#513: Marley ships none).
- Retiring the overlay: it stays for the cases above and for agents Marley does not know.

## Reference (§20)
Upstream Zed, `zed --wait`: the `cli` crate's `-w/--wait` (`crates/cli/src/main.rs:68-72`) and the
app's `open_local_workspace` (`crates/zed/src/zed/open_listener.rs:1015-1090`) open each file as a
tab with `OpenVisible::None` and return when the tab's item is released
(`ItemHandle::on_release`, `crates/workspace/src/item.rs:1105-1111`), not when it is saved. A dirty
close asks Save, Don't Save or Cancel (`crates/workspace/src/pane.rs:2406-2424`). Zed's docs give
`zed --wait` as `EDITOR` and `VISUAL` and in `terminal.env` (`docs/src/reference/cli.md:53-61`,
`236-243`; `docs/src/terminal.md:86-92`). Marley keeps the wait's end and Zed's close. It changes
the transport, Marley's MCP endpoint as #561's opener uses it, since Marley builds no `cli`; and
the window, the workspace of the terminal the call comes from, where Zed's CLI picks one by
worktree match and a temporary file is in none. Warp's rich input
(https://docs.warp.dev/agents/cli-agents/rich-input/, cited in #481) stays as the overlay for the
agents and terminals this does not reach. No Warp code.

### Prior art
- **Behavior maps.** `docs/warp_architecture/` says nothing of an agent's external editor
  (searched for rich input, `$EDITOR`, external editor); `docs/orca_architecture/05` mentions rich
  input only for its brackets. Orca's source at `1c2cf120e3` sets no `VISUAL` or `EDITOR` (searched
  `src/`), and neither does rustal-harness. The design note's fights table ("Duplicates Claude
  Code's own Ctrl+G") and B4.
- **Published material.**
  - Claude Code, interactive-mode: "`Ctrl+G` or `Ctrl+X Ctrl+E` | Open in default text editor |
    Edit your prompt or custom response in your default text editor"; Show last response in
    external editor prepends Claude's reply as `#` lines it strips on save. Keybindings:
    `chat:externalEditor`, defaults Ctrl+G and Ctrl+X Ctrl+E, `null` unbinds.
  - Claude Code 2.1.288's bundle (the box moved from 2.1.287 on 2026-10-02), read for behavior
    only: `$VISUAL`, else `$EDITOR`, else the first of `code`, `vi`, `nano` found; the value split
    on single spaces and run with no shell, the file last; an editor it does not take for a
    graphical one runs on the terminal (alternate screen) and is waited for; the file is
    `claude-prompt-<id>.md` in its temporary folder; a non-zero exit shows `<name> quit unexpectedly
    (exit code N)` and keeps the draft; the text read back replaces the draft and is not
    submitted; its Bash tool runs commands with `GIT_EDITOR=true`.
  - Codex at the installed version, upstream tag `rust-v0.155.1` (`be2951e`), read in a shallow
    clone in the /spec session's scratchpad (the batch's `/srv/stacks/rustal-codex` was removed):
    - `codex-rs/tui/src/keymap.rs:1558`: `open_external_editor:
      default_bindings![ctrl(KeyCode::Char('g'))]`, read from
      `tui.keymap.global.open_external_editor` (`keymap.rs:649-652`).
    - `codex-rs/tui/src/app/input.rs:532-539`: the key starts the editor only with no overlay, no
      modal view or popup in the bottom pane (`bottom_pane/mod.rs:1631-1635`, so not while an
      approval is asked) and no edit in progress; otherwise it is taken and does nothing.
    - `codex-rs/tui/src/external_editor.rs:39-58`: `VISUAL`, else `EDITOR`, split with
      `shlex::split`; "neither VISUAL nor EDITOR is set" (`:20`). `run_editor` (`:174-230`): a
      `.md` file in an `editor` folder under the Codex home, `~/.codex` or `<cwd>/.codex`
      (`:185-198`), which the sandbox must not be able to write (`:60-160`); the program run with
      no shell, the file as its last argument (`:210-217`), waited for (`:221`); a failed exit is
      "editor exited with status …" (`:225`); the file read back (`:228`).
    - `codex-rs/tui/src/app/input.rs:146-198`: with neither variable set, "Cannot open external
      editor: set $VISUAL or $EDITOR before starting Codex." (`:152`); the editor runs inside
      `tui.with_restored` (`:170`; `codex-rs/tui/src/tui.rs:745-780` leaves the alternate screen
      and keeps raw mode); the text, trailing whitespace trimmed, replaces the draft
      (`:186-187`) and is not submitted. `codex-rs/tui/src/app.rs:279`: the footer reads "Save and
      close external editor to continue." while it waits.
    - The TUI process runs the editor, not the App Server, so the terminal's environment is the
      one that counts (Codex hooks run in the server's, herdr #4859). No crate under `codex-rs`
      sets `GIT_EDITOR` (searched; `cli/src/doctor` only reports `VISUAL` and `EDITOR`).
  - Gemini CLI 0.62.0's bundle (Apache-2.0; `packages/cli/src/ui/utils/editorUtils.ts` and its key
    table): `input.openExternalEditor` on Ctrl+G and Ctrl+Shift+G (Ctrl+X kept as deprecated); its
    `/editor` choice first, then `$VISUAL ?? $EDITOR`, then `vi`; split on spaces; `-i NONE` added
    when the program's path contains `vi`; `$TMPDIR/gemini-edit-*/buffer.txt`; the text set back,
    not submitted.
  - OpenCode 1.18.31's bundle and opencode.ai/docs/keybinds: `editor_open` is `<leader>e` with
    the leader `ctrl+x`, and `/editor`; `$VISUAL || $EDITOR`, nothing when neither is set; split on
    spaces, no shell; `$TMPDIR/<ms>.md`; exit 0 required; the text set back. Its own `ctrl+g` is
    `messages_first`, which `home` also does.
  - None of the four was run; what each does with `marley-edit` is read, not observed.
- **The code we already ship.** Zed's `cli` and `open_listener` (the wait above);
  `Workspace::open_abs_path` (`workspace.rs:5078`) and `open_path` (`5128-5137`, `allow_preview`
  false); `ItemHandle::on_release`. Marley's `marley-open-url` and `offer_browser_opener` /
  `write_program_in` (`mcp.rs:310-362`); `caller_terminal` (`mcp.rs:759-768`); the server's grants
  (`mcp.rs:96`) and `answer` (`mcp.rs:470-503`); unlisted families (`registry.rs:41-46`);
  `terminal_run`'s 20 s wait under `APP_CALL_TIMEOUT_SECONDS` 30 (`registry.rs:176-185`,
  `marley_mcp.rs:94`); `agent_env` (`agents.rs:341-356`); the shell integration's consumed
  variables (`marley.bash:22-29`), its `~/.bashrc` (`marley.bash:50-54`), zsh's `__marley_install`
  (`marley.zsh:93-97`), fish's first prompt (`marley.fish:50-51`). Does a crate we build own the
  seam? Zed's `cli` owns "open and wait", but Marley ships no CLI and that CLI cannot name the
  caller's workspace; `marley_mcp` owns the transport; the helper takes the opener's endpoint and
  session code. No new dependency.

## UI proof
`script/e2e/649-rich-input-through-the-agents-editor-key.sh` (`compositor sway`, for the one click
on the bar's button). The scenario's `.bashrc` puts its `bin` first on the `PATH` and exports
`VISUAL=user-visual EDITOR=user-editor` unconditionally, as a user's own file would; `setup` sets
`marley.agent_editor_in_tab` true and `MARLEY_CLAUDE` to the stand-in, and binds a key to
`zed::OpenSettingsAt` `marley.agent_editor_in_tab`, as #633's scenario does for its page.

The stand-in `claude`, a bash script in `bin`, does what Claude Code does on its editor key and
nothing else, and never reaches a model: it exits at once when called with `plugin` or
`--version`, as Marley's own calls of `MARLEY_CLAUDE` are; otherwise it prints `VISUAL=…` and
`EDITOR=…` and `draft: from the agent`, reads its terminal a byte at a time and logs each byte in
hex to `keys.log`. Ctrl-G writes the draft to `$E2E_WORK/tmp/claude-prompt-<n>.md`, runs `$VISUAL`
split on spaces with the file, then prints `editor exited <status>` and `read back: <text>` and
keeps the text as its draft. Enter prints `submitted: <draft>`. It mirrors what it prints to
`stand-in.log`.

Shots:
- `649-01-agent-env`: Claude Code from the New Agent picker; the stand-in prints `VISUAL` and
  `EDITOR` naming `…/mcp/marley-edit`, not `user-visual`.
- `649-02-tab-open`: Ctrl-G; a tab `claude-prompt-1.md` in front holding `from the agent`, with the
  focus; `keys.log` holds `07`.
- `649-03-saved`: Ctrl-A, two lines typed, Ctrl-S; the tab still open, and `stand-in.log` holds no
  `editor exited`.
- `649-04-read-back`: Ctrl-W; the agent's terminal in front, `editor exited 0` and `read back:` with
  both lines, no `submitted`; `keys.log` holds only `07`, no paste marker and no `0d`.
- `649-05-submitted`: Enter, typed next with no click; `submitted:` with the edited text.
- `649-06-button`: a click on the bar's Rich Input button; the tab open again; `keys.log` gains a
  second `07`. Ctrl-W closes it.
- `649-07-plain-env`: a New Terminal; `echo "$VISUAL $EDITOR"` prints `user-visual user-editor`.
- `649-08-hand-run-overlay`: the stand-in run by hand there; Ctrl-G opens the overlay with its
  placeholder `A prompt for Claude Code`, and no tab opens. Escape, then the stand-in ends.
- `649-09-refused`: in that terminal, `MARLEY_TERMINAL_ID= <helper> note.md; echo "exit $?"`; the
  helper's reason and `exit 1`, no tab.
- `649-10-off`: the switch set off; a new Claude Code from the picker prints `VISUAL=user-visual`,
  and Ctrl-G opens the overlay.
- `649-11-setting`: the Settings window at Agent Prompts in a Tab, off.

## Locked-In Decisions
- D1 — The editor is a program in Marley's data directory, named by its absolute path in `VISUAL`
  and `EDITOR`, as #561's opener is named in `BROWSER`. Not a `marley edit` subcommand: Marley ships
  no CLI (#513), and a subcommand would start the whole app binary for every edit.
- D2 — The edit ends when the tab's item is released, as `zed --wait`'s does. Saving does not end
  it; Zed's close prompt decides what is on disk, and the agent reads that.
- D3 — The tab opens in the workspace of the terminal the call comes from, found by
  `MARLEY_TERMINAL_ID` through Marley's endpoint. Zed's CLI would pick a window by worktree, and a
  temporary file sits in none.
- D4 — Inside the agent terminals Marley opens while the switch is on, Marley's editor wins: over
  the value Marley inherited, `terminal.env`, and the user's shell files, which the integration
  follows with its own export. Every other shell keeps the user's. The switch is off by default,
  so turning it on is the user naming the agents' editor. An unconditional `export EDITOR=nvim`
  is common, and without the second export the agent's key would open that editor with no sign
  why. "Only when unset" would never fire on this box: Omarchy's `envs` exports
  `EDITOR="${EDITOR:-omarchy-launch-editor --inline}"`, and Marley inherits it. This differs from
  AD-claude-561, where the user's files win over the opener, because the opener is on by default.
- D5 — Ctrl-G keeps its Marley meaning, edit the agent's prompt in an editor. Marley writes the
  agent's own key into the terminal for Ctrl-G and the button alike: Ctrl-G itself for Claude Code,
  Codex and Gemini CLI, Ctrl-X then E for OpenCode, whose own Ctrl-G (first message, also on Home)
  stays out of reach as it is today.
- D6 — Only a terminal Marley gave its editor passes the key. An agent run by hand in a New
  Terminal, a remote project, a restored terminal, and every terminal while the switch is off keep
  the overlay, since the agent's key there would open the user's own editor in the terminal.
- D7 — `editor_open` and `editor_wait` are Marley's own: absent from `tools/list`, refused to
  outside clients, and `editor_open` acts only for a call from a Marley terminal.
- D8 — The helper's path goes into the variables only when it holds no whitespace: Claude Code,
  Gemini CLI and OpenCode split the value on spaces. Otherwise Marley sets nothing, logs why, and
  those terminals keep the overlay.
- D9 — A failure exits 1 and leaves the file alone, so every agent keeps its draft and shows its
  own message.
- D10 — Off by default, as the batch's rule asks of a change to behavior a user has: the switch
  changes what Ctrl-G does in every agent terminal (the overlay today) and which editor every
  program there gets (an agent's `git commit` with no `-m`, `crontab -e`).

## Acceptance Criteria (EARS)
One observable behavior per row, with a verification method: a named shot of the visual check,
the review of the diff, or the gate's exit code.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHERE `marley.agent_editor_in_tab` is on, WHEN Marley opens a terminal for an agent CLI in a local project, the system shall give the agent `VISUAL` and `EDITOR` naming `<data_dir>/mcp/marley-edit`, over the values the user's `.bashrc` exports. | Shot `649-01-agent-env` (bash); the review for zsh and fish |
| REQ-002 | WHERE the switch is on, the system shall leave `VISUAL` and `EDITOR` as the user's in a New Terminal. | Shot `649-07-plain-env` |
| REQ-003 | WHEN the user presses Ctrl-G in a terminal Marley gave its editor while an agent with an editor key runs there, the system shall write that agent's key into the terminal and open no Rich Input overlay. | Shot `649-02-tab-open`, with `keys.log` |
| REQ-004 | WHEN `marley-edit` runs with a file in a Marley terminal, the system shall open the file as a tab, not a preview, in that terminal's workspace, with the focus. | Shot `649-02-tab-open` |
| REQ-005 | WHILE that tab is open, the system shall keep `marley-edit` waiting after a save. | Shot `649-03-saved`, with `stand-in.log` |
| REQ-006 | WHEN that tab closes, `marley-edit` shall exit 0, and the agent shall read the file as its draft, with no paste and no Enter from Marley. | Shot `649-04-read-back`, with `keys.log` |
| REQ-007 | WHEN that tab closes, the system shall bring the terminal the edit came from to the front with the focus. | Shot `649-05-submitted` |
| REQ-008 | WHEN the user clicks the agent bar's Rich Input button in a terminal Marley gave its editor, the system shall write the agent's key into the terminal, as Ctrl-G does. | Shot `649-06-button` |
| REQ-009 | WHEN the user presses Ctrl-G while an agent runs in a terminal Marley did not give its editor, the system shall open the Rich Input overlay as before. | Shot `649-08-hand-run-overlay` |
| REQ-010 | WHERE the switch is off, the system shall give agent terminals no `VISUAL` or `EDITOR` of its own, and Ctrl-G shall open the overlay. | Shot `649-10-off` |
| REQ-011 | IF `marley-edit` cannot have Marley open the file (no endpoint, a call from no Marley terminal, a refusal), THEN it shall print why and exit 1, leaving the file as it was. | Shot `649-09-refused` |
| REQ-012 | The system shall send each agent its default editor key: Ctrl-G to Claude Code, Codex and Gemini CLI, Ctrl-X then E to OpenCode. | Review |
| REQ-013 | WHEN a tab with unsaved changes is closed with Don't Save, the agent shall read the file as it was before the edit. | Review (Zed's close kept; the release ends the edit either way) |
| REQ-014 | The system shall list neither `editor_open` nor `editor_wait` in `tools/list` and shall refuse both to an outside client. | Review |
| REQ-015 | The system shall show Agent Prompts in a Tab in the Settings window's Marley page, Agents section, off by default. | Shot `649-11-setting` |

## Phase Plan
- **P1 Plan** — promote; check that `pane::CloseActiveItem` on the tab releases its item at once
  (nothing else holds a strong handle), and which pane `open_abs_path` uses when the agent's
  terminal is the center pane's front item; `brain_ask` on the reversal of the #481 decision.
- **P2 Code** — the helper, the editor family, `agent_editor`, the key table, `agent_env` and the
  shell integration, the rich input's two entry points, the switch and its toggle; a review of the
  diff; `script/gates.sh --diff` green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
