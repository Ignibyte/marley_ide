---
pipeline_id: e1e3695c-c5a6-4204-9a02-24b6f23464e9
ticket: docs/planning/tickets/closed/TICKET-755-three-small-hardening-fixes.md
status: Phase 4 — Complete PASS
title: Three small hardening fixes
type: chore
slice: Marley's MCP server and the agent bar; the T3 Code survey's facts and smaller things (docs/t3code_architecture/README.md)
references:
  - docs/t3code_architecture/README.md
  - docs/t3code_architecture/05-terminal-browser-and-capture.md
  - docs/t3code_architecture/04-remote-and-mobile.md
  - docs/t3code_architecture/06-orchestration-mcp-and-automations.md
  - docs/planning/pipeline/completed/552-codex-and-opencode-notifications.spec.md
  - docs/planning/pipeline/completed/520-terminal-identity.spec.md
---

## Title
Three fixes the T3 Code survey found: Marley writes other tools' config files safely, each
terminal proves its identity to Marley's MCP server with a token of its own, and a renderer the
kernel kills for memory no longer takes a project's whole Chromium with it. Chad, 2026-10-11:
"8.) yes to all".

## Scope
### In
- **A. Safe config writes.**
  - A helper, `marley_workbench::safe_write`, that replaces a file another program reads:
    1. follow a symlink to the real file, so the link stays a link;
    2. read the bytes and the file's identity (device, inode, mode, length, modification time);
    3. make the new text; when it equals the old, write nothing;
    4. write a temp file in the real file's folder (`.<name>.marley-<uuid>.tmp`, `create_new`, the
       original's mode, or 0644 for a new file), and sync it;
    5. when asked, copy the old bytes to `<name>.marley-backup` beside the path as named, with the
       same mode, replacing the backup before it;
    6. read the file again: if its bytes or identity changed since step 2, remove the temp and
       refuse ("changed while Marley wrote it; nothing was written");
    7. rename the temp into place and sync the folder.
  - `agent_notify::write_codex_in` uses it, with the backup. `write_plugin_in` uses it without
    one: OpenCode's `marley.js` is Marley's whole file, versioned in its first line.
  - A refusal or failure shows as today, through `workspace.show_error`.
- **B. A token per terminal.**
  - `marley_terminal::identity`: `TOKEN_VARIABLE` (`MARLEY_TERMINAL_TOKEN`), and a token minted
    for each terminal id the first time it is asked for (a v4 UUID's 122 random bits), kept in a
    process-wide map for the run; `token_matches(id, token)` compares in constant time.
  - Zed's terminal builder sets the token beside `MARLEY_TERMINAL_ID` and empties it wherever the
    id is emptied (tasks, remote terminals); the context server Marley registers for Zed's agents
    (#501) blanks it as it blanks the id.
  - The plugin's bridge and `marley-edit` send `Marley-Terminal-Token` with `Marley-Terminal`.
  - `marley_mcp`'s transport reads the header and, for a call on Marley's own bearer, drops the
    caller's terminal when its token is missing or wrong, as it drops an outside client's
    (`transport.rs:282-283`), and logs that it did. The project and folder headers stay.
  - The check comes from the app as a function handed to the server at start, so `marley_mcp`
    gains no dependency.
- **C. Chromium's OOM policy.** `systemd_run_args` adds `--property=OOMPolicy=continue`, with a
  comment saying why.
- **Docs:** the guide (the agent bar's setup and its backup; "How agents reach it" and the
  token; the Browser tab's unit), the architecture notes (`marley_mcp.md`, `terminal_blocks.md`
  for the terminal's identity, `marley_browser.md`, `marley_workbench.md` for the agent bar's
  setup), and an architecture decision replacing #520's "the id is no authority".

### Out (explicitly deferred)
- Showing the change before the write (T3's before-and-after card): the setup stays one click,
  as #552 made it. #720's edit slice, which writes servers the user chose, is where a preview
  belongs.
- Moving Marley's own files (`tasks.json` from #558, the programs `mcp.rs` writes, the fixed-name
  temp files in `clients.rs`, `service.rs` and `relay.rs`) to the helper: their own chore.
- A process of the same user reading another terminal's token from `/proc/<pid>/environ`: the
  token stops one terminal claiming another's id, not a hostile program running as the user.
- A per-session credential for every MCP client, scoped and revoked as T3's
  `McpSessionRegistry` does: the harness owns sessions (Chad, 2026-10-02).
- A crashed page's tab: `Target.targetCrashed` still closes it (`browser.rs:3574-3580`); a tab
  that stays with "This page crashed" and Reload is a follow-up.
- systemd-oomd, which acts on whole cgroups whatever `OOMPolicy` says.

## Reference (§20)
N/A — Marley-specific: each part hardens a seam only Marley has (its writes into Codex's and
OpenCode's config, its MCP server's caller headers, its Chromium unit). The protocols are T3
Code's, read from the survey: `CaptureShortcutConfig.ts:225-297` for the checked write (report 05
§3 item 6), `McpSessionRegistry.ts:124-150` for a credential per caller (report 06 §2.1), and
`bootService.ts:114-118` for `OOMPolicy=continue` (report 04 §2.2, §3 item 3). No Warp analog.

### Prior art
- **The code we ship:**
  - Zed's `Fs::atomic_write` (`crates/fs/src/fs.rs:977-992`): a temp file in the folder and
    `persist`, but it replaces a symlink with a file (`settings_store.rs:572-593` canonicalizes
    first for that reason), writes mode 0600 whatever the original was, keeps no backup and does
    not check the file. Not enough on its own; the helper does what it does plus those four.
  - Marley's own staged writes: `rustal_skill.rs:188-209` (a uuid-named staging folder, then
    `rename`) and `shared_plugin.rs:173-198`; `settings_change.rs:255-261` refuses `changed` when
    the file moved while the user was asked. The helper takes the uuid name from the first and the
    check from the second.
  - `agent_socket.rs:114-125` and `agent_reports.rs:198-213` already prove a terminal for
    `marley-agent`'s reports (`SO_PEERCRED`, then the process chain to the terminal's shell).
    The MCP server speaks HTTP on TCP, where no peer credential arrives, so a token is the
    cheaper proof there.
  - `marley_terminal::identity` (`AGENT_PROGRAM` 41-59, a process-wide static set once) and
    `marley_terminal::ide::variables` (19-50, a map read at spawn): the patterns for the token map.
  - `transport.rs`: `ClientTable::resolve` and `ct_eq` (`clients.rs:216-226`, `auth.rs:53`), the
    outside client's caller reset (282-283), `caller_terminal` (585-596).
  - `service.rs:340-357`: `systemd_run_args`, the only unit Marley starts.
- **Behavior maps:** the T3 survey's reports 04, 05 and 06 as cited above; Orca report 06 §2.5 on
  identifying the caller by its folder.
- **Published material:** systemd.service(5) and systemd.exec(5) on `OOMPolicy=` (the default
  `stop`, `continue` logs and leaves the unit running); rename(2) on atomic replacement within one
  filesystem.

## UI proof
`script/e2e/755-three-small-hardening-fixes.sh`, under `compositor sway`, with #552's set-up
(`CODEX_HOME` and `XDG_CONFIG_HOME` in the run's folders, stand-in `codex` and `opencode`) and the
browser fixture (`offline_chromium`, `serve_site`, `browser_unit`). The file and MCP checks print
into a Marley terminal, so the shot shows them; the unit's properties and journal lines print in
the run's log beside the shots.

Shots:
- `755-01-codex-written`: after Turn on Codex notifications, with `config.toml` a symlink into a
  dotfiles folder at mode 0600: the toast, and the terminal's listing: still a link, the backup at
  600, the target at 600 holding the three keys and its comment, no temp file left, and the
  backup's first lines (REQ-001 to REQ-003).
- `755-02-plugin-written`: after Connect OpenCode to Marley: `marley.js` alone in its folder at
  644, no backup and no temp file (REQ-003).
- `755-03-honest`: two terminals side by side; the right one's environment holds a 32-hex
  `MARLEY_TERMINAL_TOKEN`, and its client's own row is marked `self` (REQ-005, REQ-006).
- `755-04-spoofed`: the right terminal's client with `MARLEY_TERMINAL_ID` set to the left
  terminal's id and its own token: no row marked `self` (REQ-007).
- `755-05a-calm`: a project's Browser tab on a calm page; the log shows the unit's
  `OOMPolicy=continue` (REQ-008).
- `755-05b-hungry`: a second tab on a page that eats memory, the unit's `MemoryMax` lowered at
  run time to its use plus 400 MiB and swap off.
- `755-06-after-oom`: the kernel has killed that renderer and Marley closed its tab: the calm tab
  still draws, and the log shows the unit `active` and the journal's OOM line (REQ-009).

## Locked-In Decisions
- **D1:** one helper for writes into other programs' files, in `marley_workbench`, used by both
  of `agent_notify`'s writes now. Rejected: Zed's `atomic_write` (above); `tempfile` (a
  dev-dependency only, and its 0600 file needs the mode put back anyway).
- **D2:** one backup, `<name>.marley-backup`, beside the path as the user names it, replaced each
  time. Rejected: a dated backup per write (piles files into the user's folders); a backup beside
  the symlink's target (it would land in a dotfiles repository as an untracked file).
- **D3:** the check is the T3 one: bytes and identity read again just before the rename. With no
  card between the read and the write the window is short, so the review of the diff proves the
  refusal rather than a shot.
- **D4:** the terminal token lives in `marley_terminal::identity`, minted at spawn and kept for
  the run; a restored terminal keeps its id and gets a new token. Rejected: a token derived as a
  hash of a per-boot secret and the id (a hash crate in `marley_terminal`, and the secret would
  have to exist before the first restored terminal spawns, which is before the MCP server
  starts).
- **D5:** a wrong or missing token drops the terminal from the caller; the call runs as one with
  no terminal, as #520's D7 treats a malformed header. Rejected: refusing the call (a Claude Code
  whose plugin copy predates the bridge's new header would lose every tool until the plugin
  updates).
- **D6:** `OOMPolicy=continue` alone. A renderer killed for memory closes its own tab as a crash
  does today; the unit, the relay and the other tabs stay.
- **D7:** the three parts land in the order C, A, B, each whole before the next. B is the
  largest (six places, a Zed crate hunk among them) and can leave as its own ticket without
  touching A or C (see the notes, "The split").

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user turns on Codex notifications and `config.toml` is a symlink, the system shall write the link's target and leave the link in place. | Shot 755-01 |
| REQ-002 | WHEN the system rewrites Codex's `config.toml`, it shall keep the file's mode and leave the old text in `config.toml.marley-backup`. | Shot 755-01 |
| REQ-003 | WHEN the system writes Codex's config or OpenCode's plugin, no temp file shall remain beside it. | Shots 755-01, 755-02 |
| REQ-004 | IF the file changed between Marley's read and its rename, THEN the system shall write nothing, remove its temp file and show the error. | The review of the diff |
| REQ-005 | WHEN Marley opens a local interactive terminal, its environment shall hold a `MARLEY_TERMINAL_TOKEN` beside `MARLEY_TERMINAL_ID`; a task, a remote terminal and Zed's agents' bridge shall get it empty. | Shot 755-03; the review of the diff |
| REQ-006 | WHEN a call names a terminal with its own token, the system shall take that terminal as the caller. | Shot 755-03 |
| REQ-007 | WHEN a call names a terminal without that terminal's token, the system shall treat the call as having no terminal. | Shot 755-04 |
| REQ-008 | WHEN Marley starts a project's Chromium, its unit shall carry `OOMPolicy=continue`. | Shot 755-05a and the run's log |
| REQ-009 | WHEN the kernel kills one renderer of a project's Chromium for memory, the system shall keep the unit running and the project's other Browser tabs drawing. | Shots 755-05b, 755-06 and the run's log |
| REQ-010 | The gate shall pass. | `script/gates.sh --diff` exit 0 |

## Phase Plan
- **P1 Plan** — this spec, and at promotion the design in the notes, every cited seam checked
  again.
- **P2 Code** — C: `service.rs`. A: `safe_write.rs` (new), `agent_notify.rs`. B:
  `marley_terminal/src/identity.rs`, Zed's `crates/terminal/src/terminal.rs` (the existing
  Marley hunk, its touchpoint row extended first), `mcp.rs` (the context server's environment, the
  check handed to the server), `marley_mcp` `transport.rs`, the bridge, `bin/marley-edit`. The
  guide, the architecture notes, the decision. A review of the diff; `script/gates.sh --diff`
  green.
- **P3 Test** — the visual check: write and run the scenario, read every shot.
- **P4 Complete** — CHANGELOG and architecture docs (§21), ledger capture (§19), close the ticket,
  archive, commit.
