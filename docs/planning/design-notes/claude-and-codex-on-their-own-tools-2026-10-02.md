# Claude Code and Codex on their own tools, 2026-10-02

Chad: "also we need to brain storm integration into claude and codex using their tools instead
of fighting them as well."

What was read: every place Marley and rustal-harness touch Claude Code or Codex (Marley's
`marley_agent` and `marley_workbench`, its Claude plugin, Zed's `agent_servers`; the harness's
`CLAUDE_CODE.md`, `CODEX_*.md`, D161, D162), Claude Code's documentation (hooks, plugins, mods,
MCP and channels, the IDE integration, headless use), and Codex 0.158.0's source in
`/srv/stacks/rustal-codex` (stock Codex plus a rename) with its documentation, now at
learn.chatgpt.com. On this box: Claude Code 2.1.287, so mods are available; Codex 0.155.1, which
has `codex app-server`, `codex agents` and `--remote`.

The short answer: Marley observes Claude Code well and acts on it badly. It reads typed state
from Claude Code's hooks, then acts by typing into the terminal (prompts, selections, review
notes, with a fixed 200 ms wait between a paste and Enter), by reading the screen and pressing
keys (the folder-trust question), and by leaning on things Claude Code does not document (its
internal prompt tags, `installed_plugins.json`; `terminalSequence` has since been documented). Codex in a Marley
terminal gets nothing structured: a 2-second quiet timer for state, argv parsing for its
permission chip, OSC 9 for notifications. Both agents now ship the official channels that remove
every one of these: Codex's TUI already runs as a client of a shared App Server any other client
can join, and Claude Code 2.1.287's mods can submit prompts, decide tool calls and report state
from inside the process. The harness already uses the official headless channels and plans a
Claude Code mod of its own (D162, its M13), so the two should share one.

## Where Marley fights them today

| What | Where | Cost so far |
|---|---|---|
| Answers Claude Code's folder-trust question by reading the screen and pressing ↑/↓ and Enter | `marley_agent/src/trust.rs:12-160`, `agent_trust.rs:164-236` | Three wordings and a focus glyph hard-coded; 2.1.263 moved the focus to "No, exit" (AD-587); F-claude-587 |
| Types prompts into the TUI: rich input, send selection, send block, English lines, review notes (paste, wait 200 ms, CR) | `rich_input.rs:521-552`, `terminal_drive.rs:54,313-325`, `send_selection.rs:236-259` | F-claude-594 (high), F-claude-481; every path must check the seat isn't waiting, or a paste answers a permission dialog |
| Guesses Codex, Gemini and OpenCode state from 2 s of quiet output or a bell | `marley_agent.rs:589-599`, `close_guard.rs:204` | The rail and the close guard rest on a guess |
| Reads Codex's permission mode from argv | `marley_agent.rs:405-499` | Misses `config.toml` and profiles |
| Hook frames ride the hook output field `terminalSequence`, under a 4 KiB cap (now documented in Claude Code's hooks reference, OSC 777 on its allowlist; corrected 2026-10-03, #648) | `claude_plugin/marley/hooks`, `event.py:136-151` | AD-482's warning, now out of date; F-claude-519 |
| Tells user prompts from injected ones by 19 internal tags copied from Orca | `claude_events.rs:710-822` | A renamed tag silently becomes a "user" prompt |
| Checks its plugin by reading Claude Code's own bookkeeping file | `claude_plugin.rs:181-208` | F-claude-547 (high) |
| Rich input draws a Zed editor over Claude Code's prompt | `rich_input.rs` | Duplicates Claude Code's own Ctrl+G, which opens the prompt in `$VISUAL`/`$EDITOR` |
| Send selection types `@path#L12-40` | `send_selection.rs` | Duplicates Claude Code's IDE integration, which sends the selection with every prompt |

## Their tools

**Codex.** Since about 0.157 the TUI starts or reuses a shared App Server daemon for its
`CODEX_HOME` and talks to it over a Unix socket; `codex --remote unix://PATH` names the server
and never falls back. Any client on the same socket sees every thread's state
(`thread/status/changed` goes to every connection: idle, active, `waitingOnApproval`,
`waitingOnUserInput`), gets the live items, diffs, plans and token use by calling
`thread/resume` on the loaded thread, answers approvals (the first answer wins and the TUI closes
its own prompt), and sends prompts (`turn/start`, `turn/steer`, `turn/interrupt`, or
`thread/queue/add` to deliver when idle). Hooks (twelve events, `PermissionRequest` decides),
plugins (it also reads a `.claude-plugin/plugin.json` manifest), MCP servers in config, and
`codex resume <id>`. Traps: hooks run in the daemon's process with the environment of whichever
terminal started it, so a hook can't find its terminal by an environment variable (herdr #4859);
a `-c` override or `--profile` makes the TUI fall back to a private server nobody else can reach;
the managed daemon can update itself to a newer version than the TUI.

**Claude Code.** Settings hooks (a `PermissionRequest` hook can allow or deny; HTTP and
`mcp_tool` hook types reach a server directly). Mods, 2.1.287 and later: JavaScript inside Claude
Code whose hooks run in every kind of session; `tool.call` and `tool.check` can hold, deny,
answer or approve a tool call; `$.prompt.submit` submits a prompt as if typed; `$.session.send`
messages another session; `$.http.fetch` and `$.process` reach out; a mod cannot change what the
permission prompt shows. Channels (research preview): an MCP server pushes messages into a
session. The IDE integration: an editor runs a loopback WebSocket MCP server, writes
`~/.claude/ide/<port>.lock` (0600, a fresh token per start), and Claude Code connects, sending the
selection and the open file with each prompt, opening diffs in the editor and reading
`mcp__ide__getDiagnostics`; the dozen internal tools beyond diagnostics are undocumented by name.
`--resume`, `--fork-session`, and the headless `-p` stream-json the harness drives.

## Ideas

### B1. Join Codex's App Server as a second client

Marley starts `codex app-server --listen unix://<Marley's socket>` and launches Codex in its
terminals as `codex --remote unix://<that socket>`, so the TUI the user sees and Marley's client
share one server. The rail reads typed state, the approvals inbox answers Codex's requests in
place, send selection and review notes go in as `turn/start` or `thread/queue/add`, the Agent tab
gets diffs and plans, the fleet gets token use, and resume is `thread/resume`. Replaces the quiet
timer, the argv chip, OSC 9 and typing, for Codex. Owning the server also pins the version
(generate schemas from that binary) and keeps hooks from inheriting a stranger's environment.
The harness drives the same App Server for its seats (H9, H10); one Rust client could serve both.
Size: L. The biggest single gain.

### B2. One Claude Code plugin with a mod, shared with the harness

The harness's M13 builds a Claude Code plugin whose mod declares state from turn events, decides
tool calls, carries steering (`$.prompt.submit`) and aborts (`$.turn.abort`). Marley's plugin
today does the first job with settings hooks and an undocumented output field. One plugin, built
once: the mod reports typed state to whichever host started the session (Marley's MCP server or
the harness's), takes prompts from it instead of a pasted keystroke stream, and lets an approval
be answered from Marley's inbox through `tool.check`. Retires `terminalSequence`, the injected-tag
table and the 200 ms paste wait. Needs Claude Code 2.1.287 or later (installed). Size: L, split
with the harness.

### B3. Marley as Claude Code's IDE

Marley serves the IDE MCP server and writes the lock file, so Claude Code in a Marley terminal
gets the selection and open file with every prompt, Zed's language-server diagnostics, and diffs
opened in Marley's diff view. Replaces send selection's typed `@path`. Catch: only diagnostics are
documented by name; opening diffs and reading selections use internal tool names that could change
with any release. Size: M for selection, open file and diagnostics; the rest waits for
documentation.

### B4. Rich input through Claude Code's own editor key

Done by #649 (2026-10-04) for the terminals Marley opens for agents in local projects, behind
`marley.agent_editor_in_tab`, off by default: `marley-edit` as `VISUAL` and `EDITOR`, and Ctrl-G
there sends the agent its own key (Ctrl-G to Claude Code, Codex and Gemini CLI, Ctrl-X E to
OpenCode). Restored and remote terminals keep the overlay.

Claude Code's Ctrl+G opens the prompt in `$VISUAL` or `$EDITOR`. A `marley edit` command that
opens the file in a Marley buffer and waits until it closes gives rich input through Claude
Code's own path, for any agent that honours `$EDITOR`; Codex's TUI has an external-editor key
too (to confirm). Rich input today takes Ctrl-G while an agent runs, so the two collide now.
Size: S.

### B5. Resume through each agent's own command

`claude --resume <id>` today; `codex resume <id>` (or `thread/resume` under B1), and the
harness's resume table (its TICKET-091) for the rest. Size: S per agent.

### B6. The trust question

No official route was found apart from headless use, which has no trust gate. Options: keep the
screen read as the one documented exception, look for a settings key that pre-trusts a folder
(AD-587 refused writing `hasTrustDialogAccepted` into `~/.claude.json`), or ask Anthropic. Size:
research first.

### B7. Version policy

Started by #648 (2026-10-04): `marley_agent::versions`' table, the agent bar's chip and
`marley.allow_untested_versions`; its one row is Claude Code's prompt tags. B1 to B3 add their own
rows; B7 part 2 (a remote host's version) remains.

The harness pins exact Claude Code and Codex binaries and checks their digests; Marley runs
whatever is on the search path, which moved through four Claude Code versions in five days. With
B1 and B2 Marley depends on typed protocols, so it should at least check the version it talks to
and refuse what it hasn't been tested against, or adopt the harness's pins.

## Overlaps with the harness to settle

- Two Claude Code plugins would load in one session if Marley shows harness terminals; the
  harness passes an empty `--setting-sources`, so Marley's may not load at all. One owner.
- The report contract: the harness's TICKET-091 and the herdr note's item 2 describe the same
  thing. One contract.
- Codex: the harness on the App Server, Marley on the TUI. One client crate (B1).
- Resume: twice, keyed differently.

## Open questions for Chad

Asked one at a time, in this order:

1. B1: should Marley own Codex's App Server and join it as a second client?
2. B2: one Claude Code plugin with a mod, built with the harness (which already plans it), and
   Marley's own plugin retired into it?
3. B3: Marley as Claude Code's IDE, starting with the documented parts?
4. B4: rich input through the agents' own editor key?
5. B7: follow the user's installed versions, or the harness's pins?

## Chad's answers, 2026-10-02

1. B1, Marley owns Codex's App Server and joins it as a second client: "Yes".
2. B2, one Claude Code plugin with a mod, built with the harness, Marley's plugin folded in: "Yes, with the harness".
3. B3, Marley as Claude Code's IDE (connection, diagnostics, selection, open file; undocumented parts checked per supported version; diffs later): "Yes, with version checks".
4. B4, rich input through the agents' own editor key (a `marley edit` command as `$EDITOR`; the overlay only for agents with no editor key): "Yes".
5. B7, versions: follow the user's installed `claude` and `codex`, checked against a list Marley has been tested on, turning off only the parts it is unsure of outside that list and saying why: "Your install, with a check".

## The harness's side, 2026-10-02

The harness took B2 as D169: its M13 plugin (in `crates/harness-runtime/src/claude/plugin/`,
TICKET-100 done) is the shared one, "One plugin, two hosts" in its `docs/M13_PLAN.md`. The mod
picks its host from the launcher's variables (`RH_STATE`, `RH_BIN` for the harness) and reports
nothing when none is set; MREQ-009 asks Marley to name its own variable and how the mod reaches
Marley. One state contract: its TICKET-099 (`docs/AGENT_SEATS.md`). A harness seat runs under
`env -i` with an empty `--setting-sources` and only the harness's `--plugin-dir`, so nothing loads
twice. Its findings for Marley's half:

- A mod's `$.mcp.call` goes through Claude Code's permission check and waits on a dialog unless a
  rule allows the server; it reaches only a server that has finished connecting (the first
  report at `session.start` was lost that way); at `session.end` it is refused ("no session is
  bound in this process"). `$.env.get` and `$.process.run` work everywhere, which is why the
  harness's mod runs `rh report` and `rh release`. Marley's half should report through a
  `marley` command the same way, not `$.mcp.call`.
- The trust dialog (B6) comes before anything of the session runs, the mod included; "No, exit"
  is focused first; an answer within about a second of the first drawing is lost.
- `tool.check`'s `next(e)` gives Claude Code's own verdict (`allow`, or `ask` with the rule); a
  hook answering `allow` runs the call with no dialog; a hook that throws or passes 10 s of its
  own time is denied. TICKET-101 sends only `ask` verdicts to the host, shaped so another host
  can answer them.

## Tickets, 2026-10-03

Queued as #648 (B7, the tested-version table, `marley.allow_untested_versions`), #649 (B4,
`marley-edit` as the agents' editor behind `marley.agent_editor_in_tab`; Ctrl+G for Claude Code,
Codex and Gemini CLI, Ctrl+X E for OpenCode), #650 (B1 part 1, Marley's own Codex App Server and
typed state), #651 (B1 part 2, Codex's approvals from the inbox), #652 (B2, Marley's half: agent
reports through `$MARLEY_BIN report`, the answer to the harness's MREQ-009) and #653 (B3, Marley as
Claude Code's IDE behind `marley.claude_code_ide`). Split out, needing tickets: B1 part 3 (prompts
as `turn/start` and `turn/steer`), B1 part 4 (resume through `codex resume --remote`), B2's
plugin loading (waits on the harness adding Marley's host and a licence for its plugin files),
B2's prompts and approvals and the hook frames' retirement, B3's diffs (and a by-hand check in
Chad's own session of what Claude Code does with an edit while connected), B7 part 2 (remote
hosts' versions), and giving restored terminals the agent variables (found drafting #649).
