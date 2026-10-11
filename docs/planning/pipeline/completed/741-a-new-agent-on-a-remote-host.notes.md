# A new agent on a remote host — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-741-a-new-agent-on-a-remote-host.md
- **Pipeline spec:** 741-a-new-agent-on-a-remote-host.spec.md

## Phase 1 — Plan (queued by /spec, 2026-10-10)
- **Request:** Chad, 2026-10-10, the agents-anywhere plan
  (`docs/planning/design-notes/agents-anywhere-2026-10-10.md`), with the goal "lets make tickets
  and build it".
- **Classification:** feature.
- **Recall (§18.3):**
  - The guide's own note (`guide.md:2378`) and AD at `architecture-decisions.md:4444`: Views' commands run on this machine only.
  - `rh attach WORKSPACE` takes a workspace alias or id and needs a terminal; `seat start` waits up to 60 s for the seat's first report (TICKET-109).
  - #541's spawn-site rule: the ssh runs as a terminal's task, not a new spawn site.
- **Checklist:** this harness has no TaskCreate; the phase checklist lives here.
- **The design** is written at promotion (`/pipeline:plan`), when every cited seam is checked
  against the code again.

## Phase 1 — Plan (promoted 2026-10-10)
- **Pre-flight:** no active pipeline; README marker present; cargo idle.
- **Recall, at promotion:**
  - #740 shipped `harness_hosts` (`HarnessHost::base`, `ssh_arguments`, `shell_word`),
    `Slot`, `seat_command_of` and the seat form's `seat_slots`; PR-claude-740 says every word after
    an ssh destination is quoted for the host's shell.
  - Marley's terminal `ssh` wrapper (#526, `marley.bash:131`) passes an ssh with a remote command
    through as plain ssh, so the typed attach line is not turned into a Marley login.
  - #735's picker: `places_for`, `load_recent` splices recent projects before the last place
    (`Browse…`), `Start::run_in`; PR-claude-735: code in the picker's confirm defers modal swaps.
- **Brain:** no `rusty` MCP server in this repository's sessions; no `brain_ask`.
- **Seams re-verified:**
  - `agents.rs`: `show_picker` (721), `places_for` (745), `Place` (818), `load_recent` (910, its
    `places.len() - 1` assumes Browse… last), `Start` and `run_in` (985-1040), the delegate's
    `update_matches`, `confirm` and `render_match` (1102-1290).
  - `harness_seat.rs`: `SeatAgent` (47, `argument`, `shown`), `Seat::commands`, `run_seat`,
    `seat_slots`, `SeatStartFailed` and `show_app_notification`.
  - `harness.rs`: `HarnessView::surface` (1555, keeps `views` as kind and quoted line through
    `view_line`, 1737), `open_view` (1583, types the line into `agents::start_in_terminal`).
  - rustal-harness `harness-cli/src/seats.rs:215-245`: `seat start` answers `{id, title,
    profile, opened, state, views}`, `views` as `session_surface_to_human` gives them
    (`harness-runtime/src/mcp.rs:1258`: `native` = `rh view WS`, and `tmux` = `rh attach WS`
    for a tmux seat), each `{kind, claim, argv}`.
  - `marley_remote::remote_terminal_command` (391): a terminal's ssh is `ssh -t`, the keepalive
    and `ConnectTimeout`, the port, `--`, the destination.

### Design
- **`harness_hosts.rs`:** `terminal_line(host: Option<&HarnessHost>, argv)`: the argv's words
  through `shell_word`, joined; for a host over ssh, `ssh -t <keepalive> -o ConnectTimeout=10 [-p
  P] -- DEST` and that line as one more word, every word quoted for the local shell.
- **`harness.rs`:** `view_command(slot, argv, cx)` picks the host and calls `terminal_line`;
  `view_line` gives the kind and the argv; `surface` keeps each view's line from
  `view_command`, so Views shows, copies and opens the line that runs, through `ssh -t` for a host.
  The guide's "the commands run on this machine" line changes: a `marley.harness` command over SSH
  still runs them here.
- **`harness_seat.rs`:** `SeatAgent::of(AgentKind)` (Claude, Codex); `seat_slots` public to the
  crate; `start_agent_on(workspace, slot, agent, folder, window, cx)`: a toast "Starting Claude
  Code on box-2…", then `seat add` under `<agent>-<folder's last part, a-z0-9->`, skipping names
  the slot's sessions hold and trying `-2` … `-9` on `seat_exists`, then `seat start`; its `views`
  give the attach (`tmux`, else `native`) line, typed into a new terminal of the workspace with
  `agents::start_in_terminal`. Every refusal, and an answer with no view, is a notification
  ("Could not start Claude Code on box-2: …"), and no terminal opens.
- **`agents.rs`:** `Place::Harness { slot, name }`, listed after Browse… as **On box-2…** (**On
  the harness…** for the first harness), for each `seat_slots`; `load_recent` splices before
  Browse…'s own index. Where shows harness places only for a CLI choice whose kind is a
  `SeatAgent`. Choosing one moves the picker to a third step, `remote: Some(slot)`: the
  placeholder asks for a folder on that harness's host; the typed query is the one entry, "Start
  Claude Code in /srv/other on box-2", or "Type the folder's full path" when it does not start
  with `/`; Enter on a full path dismisses and runs `start_agent_on`.
- **File manifest (all Marley crate):** `agents.rs`, `harness_seat.rs`, `harness.rs`,
  `harness_hosts.rs`; `docs/marley/guide.md`; the scenario.

### Visual check plan
| REQ | The scenario does | The shot |
|---|---|---|
| 001 | `marley.harnesses` names `box-2` through #740's fake `ssh`; a fake `claude` on the PATH; Ctrl+Alt+N → Claude Code | 741-01-where |
| 002, 003 | On box-2… → `/srv/other` → Enter; the stand-in `rh` answers `seat add`, `seat start` with views, and `attach WS` by printing `attached: WS` | 741-02-attached, the ssh log |
| 004 | The seat's row under HARNESS · box-2 → its tab → Views → Open | 741-03-view, the ssh log |
| 005 | — | The review of the diff |

### Risks
- **Coordinates** for the session row, Views and its Open are found by the first run.

## Phase 2 — Code (2026-10-10)
- **Built:**
  - `harness_hosts.rs`: `terminal_line(host, argv)`; `ssh_arguments` lost its `tty` flag, which
    only the terminal line would have set and which builds its own.
  - `harness.rs`: `view_argv` (kind and argv) in place of `view_line`; `view_command(slot, argv)`;
    `surface` keeps each view's line from it, so Views shows, copies and opens the line that runs.
  - `harness_seat.rs`: `SeatAgent` for the crate with `of(AgentKind)`; `seat_slots` for the crate;
    `start_agent_on` (a toast while it starts), `add_and_start` (names from `seat_stem`, skipping
    the slot's session titles, the next on `seat_exists`, up to `-9`), `attach_line` (`tmux`, else
    `native`), `not_started` (a notification; the toast goes).
  - `agents.rs`: `Place::Harness { slot, name }` after Browse…, shown only for a CLI choice a seat
    can run (`Start::seat_kind`); `load_recent` splices before Browse…'s own index; the folder step
    (`remote`, `typed`), its placeholder, its one entry, and its confirm.
  - The scenario `script/e2e/741-a-new-agent-on-a-remote-host.sh`.
- **Deviations:** none from the design.
- **Review:**
  - Re-entrancy: `start_agent_on` runs in the workspace update the picker's confirm makes, as
    `Start::run_in` does; `show_app_notification` defers itself, so calling it there is safe.
  - The typed attach line runs as plain ssh in Marley's terminals: #526's wrapper passes an ssh
    with a remote command through.
  - REQ-005: every refusal of `seat add` or `seat start`, no reachable harness, and an answer
    without a view end in `not_started`; the terminal opens only after a view is found.
- **Checklist:** harness_hosts.rs ✓, harness.rs ✓, harness_seat.rs ✓, agents.rs ✓, scenario ✓;
  the guide at Complete.
- **Gate:** `script/gates.sh --diff` GREEN, 17 passed (the first run red on gate:14: `<harness>`
  in the module doc read as an HTML tag; now in a code span).

## Phase 3 — Test (2026-10-10)
- **Scenario:** `script/e2e/741-a-new-agent-on-a-remote-host.sh` (sway). A runtime of the built
  `rh` on a scratch root is `box-2` (`marley.harnesses`, `ssh: box-2`); #740's fake `ssh` logs
  `$*` and runs the words after the destination in a shell; a stand-in `rh` answers `seat add`,
  answers `seat start` by opening an actor of the seat's name with `native` and `tmux` views, and
  answers `attach WS` with `attached: WS`; a fake `claude` makes Claude Code a CLI choice.
- **Runs:**
  1. Checks for REQ-002 and REQ-003 pass; the attach terminal worked in Marley's own folder
     (`/srv/stacks/marley_ide`), as `start_in_terminal` got no folder. Fixed: the workspace's
     `default_working_directory`. The seat row's place was a guess.
  2. The tab opened; the views' Open place was a guess.
  3. Every check passes; the shots below.
- **Shots:**
  - `741-01-where` (REQ-001): Where for Claude Code lists the guess, recent projects, Browse…,
    then **On box-2…** with "a seat, folder next".
  - `741-02a-folder`: the folder step, `/srv/other` typed, one entry "Start Claude Code in
    /srv/other on box-2".
  - `741-02-attached` (REQ-002, REQ-003): a new terminal in the project's folder (`…/repo`) typed
    `ssh -t -o ServerAliveInterval=5 -o ServerAliveCountMax=3 -o ConnectTimeout=10 -- box-2
    '…/bin/rh --state … attach ws-claude-other'` and shows `attached: ws-claude-other`; the rail
    lists `claude-other` under HARNESS · box-2. The ssh log holds `-T … -- box-2 … seat add
    claude-other --agent claude --cwd /srv/other` and `… seat start claude-other`.
  - `741-03a-views`: the `claude-other · box-2` tab, Views listing `native` and `tmux`, each line
    `ssh -t … -- box-2 '/srv/stacks/rustal-harness/target/debug/rh --state …'`.
  - `741-03-view` (REQ-004): Open on `native` started a terminal running the harness's observer
    view of the seat ("the seat is up"); the ssh log holds `-t … -- box-2 …/rh --state … view
    <workspace>`.
- **REQ-005** (by review): every refusal, no reachable harness and an answer without a view end in
  `not_started`'s notification before any terminal opens.
- **Pre-existing — not in scope:** Views → Open (#690) starts its terminal in the tab's group with
  no folder, so in Home it works in Marley's own folder.
- **Fix gate:** after the folder fix, `script/gates.sh --diff` GREEN (17 passed).
- **Focus:** 1 Marley window before and after on Hyprland; no rule added.

## Phase 4 — Complete (2026-10-10)
- **Documented:** `CHANGELOG.md` (Added: A new agent on another machine); `docs/marley/guide.md`
  (Where's **On box-2…**, Views over `ssh -t`, "Several harnesses" pointing at it);
  `docs/marley_architecture/marley_workbench.md` (A new agent on a harness);
  `docs/marley/workbench-shell.md` (#741's line). No Zed path touched.
- **Knowledge:** F-claude-741-the-attach-terminal-worked-in-marleys-own-folder-001,
  L-claude-741-a-line-for-a-remote-view-is-quoted-twice-001,
  AD-claude-741-a-remote-agent-is-a-harness-seat-with-a-terminal-attached-001.
- **Brain:** no `rusty` MCP server in this repository's sessions, so no consultation to close.
