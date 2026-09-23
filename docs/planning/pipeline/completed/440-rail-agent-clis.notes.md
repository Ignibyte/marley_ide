# Agent CLIs in rail terminals — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-440-rail-agent-clis.md
- **Pipeline spec:** 440-rail-agent-clis.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-22)
- **Request:** the Warp-style agent Chad works with daily: `claude` or `codex` in a terminal,
  listed under its project.
- **Classification / tier:** feature, medium; `marley_workbench` plus the pure `marley_agent`.
- **Recall (§18.3):** the gpui-era agent cockpit (#72-#80, #187) shipped `agent_kind_of`,
  `agent_status_from` and the Waiting threshold as pump ticks; the fork has no 16ms pump, so
  the threshold becomes a duration. `PR-claude-raw-input-passthrough-must-filter-platform-chords-001`
  does not apply (no key routing here).
- **Discovery:** on this box `claude` is `~/.local/bin/claude` → `~/.local/share/claude/versions/2.1.280`
  (native), `codex` and `opencode` come from mise shims, `gemini` is a shell script in
  `~/.local/bin`. The terminal sweep of 2026-09-22 found the handshake pair and argv-based
  naming.
- **Human confirmation:** Chad's goal authorizes autonomous execution through commit
  (2026-09-22). No `TaskCreate` in this harness; checklists live here.

## Phase 1 — Plan (promoted 2026-09-22, after #439)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] promote and
  re-verify · [x] prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Pre-flight:** tools present, no active pipeline, README marker present. A cargo run was
  up at promotion: rustal's own gate, in its own target directory. Planning needs no cargo;
  Code checks again before its first build.
- **Recall.**
  - The gpui-era cockpit (#72 to #80, #187) gave `marley_agent` its tick-based waiting
    threshold. The fork has no 16 ms pump and no crate uses the crate, so the tick API gives
    way to a duration on gpui's executor clock (D4).
  - `L-claude-439-a-popover-menu-takes-focus-only-on-a-platform-frame-001`: menus in driven
    tests open with the pointer and entries are found by `MENU_ITEM-<label>`, which is part
    of why the CLIs became entries (D8).
  - `PR-claude-agent-output-size-not-a-completion-signal-001`, in spirit: a quiet spell is not
    proof an agent is done. The spinners Claude Code and Codex draw keep output flowing while
    they work, and the bell refines it. A silent thinker reads as waiting (a risk below).
  - Brain: consultation `a56ec3fdce2c4c36b301035d22afec18`, nothing on this seam.
- **Re-verified at promotion.** The spec's `crates/terminal` lines hold:
  - the handshake pair at `terminal.rs:2142-2225`;
  - `foreground_process_command_name` at `:2890`, which returns `None` without a PTY, so a
    display-only test terminal has no foreground process (D6);
  - `breadcrumb_text`, a public `String`, set from OSC at `:1638`.

  Zed's own launch sequence is `agent_panel.rs:2170-2212`, with a 5 s startup timeout
  (`TERMINAL_INIT_COMMAND_STARTUP_TIMEOUT`). `start_init_command_startup_handshake` returns a
  finished task on a non-PTY terminal, so one code path serves both kinds and the tests reach
  all of it. W3's `+` menu reads New Terminal, then New Agent Thread; the CLI entries go
  after them, so W3's pointer helper (the row under New Terminal) still finds its submenu.

### Design
- **`marley_agent` (pure).**
  - `AgentKind { Claude, Codex, Gemini, OpenCode }` with `ALL` in menu order, `program()` and
    `display_name()`.
  - `agent_kind_of` recognizes all four, by the program's file name.
  - `launch_input(kind)` is `send_payload(kind.program())`.
  - `AgentStatus { Working, Waiting }` with `label()`; `WAITING_AFTER` = 2 s;
    `agent_status(quiet_for, bell)` returns waiting on the bell, or once quiet for at least
    `WAITING_AFTER`.
  - `status_line(kind, status)`, for example "Claude Code · waiting".
  - Removed, since nothing uses them and the fork has no pump: `WAITING_TICKS`,
    `agent_status_from`, `AgentRun`, `launch_command`, and the `Idle` and `Exited` states.
- **`marley_rail` (pure).** `TerminalSnapshot.agent: Option<TerminalAgent { kind, status }>`,
  carried into `TerminalRow`. The crate gains a dependency on `marley_agent`, pure as well.
  Selection and attention are unchanged: a bell still lights the dot, and a quiet spell does
  not (D7).
- **`marley_workbench::rail`.**
  - **Seams.** `foreground_command: fn(&Terminal) -> Option<String>`, by default
    `Terminal::foreground_process_command_name`. `agent_search_path: Option<OsString>`, `None`
    meaning the process's `PATH`. Tests replace both, as they replace the terminal factory.
  - **State.** `terminal_output: HashMap<EntityId, Instant>`, the last output per terminal on
    the executor clock. `quiet_timers: HashMap<EntityId, Task<()>>`: re-armed on each output
    of an agent terminal, each timer refreshes the rail after `WAITING_AFTER`, and both maps
    are pruned to the terminals that exist.
  - **Snapshot.** For each terminal, the foreground command goes through `agent_kind_of`. An
    agent row's title is the breadcrumb if one is set, else the agent's display name; its
    second line is `status_line`; its status is `agent_status(now - last_output, bell)`, where
    no output yet counts as quiet.
  - **Render.** An agent row draws the agent's icon (`AiClaude`, `AiOpenAi`, `AiGemini`,
    `AiOpenCode`) and carries a `marley-rail-agent-{id}` selector.
  - **Menu.** After New Agent Thread: a header, then one entry per CLI that `which::which_in`
    finds on the search path, with the agent's icon. The section is absent when none is found.
  - **Launch** (`new_agent`): the terminal starts as New Terminal's does
    (`add_center_terminal` with the factory, in `default_working_directory`). Then
    `start_init_command_startup_handshake` races a 5 s timer (`futures::future::select`), and
    `write_init_command_after_startup(launch_input(kind))` runs. Errors reach
    `detach_and_prompt_err`.
- **Manifests.** `marley_rail` gains `marley_agent`. `marley_workbench` gains `marley_agent`,
  `which` and `futures`, plus `terminal`'s `test-support` in dev (`take_pty_write_log`).
  `Cargo.toml`'s `[workspace.dependencies]` gains `marley_agent`, under the existing
  `Cargo.toml` row (a Zed path; the row says a Marley crate joins that table when something
  depends on it).
- **File manifest.** All Marley-owned except `Cargo.toml` and `Cargo.lock`, which are covered
  by their rows:
  - `crates/marley_agent/src/marley_agent.rs` with its tests;
  - `crates/marley_rail/src/marley_rail.rs` and its `Cargo.toml`;
  - `crates/marley_workbench/src/rail.rs`, `rail_tests.rs` and its `Cargo.toml`;
  - docs: `docs/marley_architecture/marley_agent.md`, `marley_rail.md`, `marley_workbench.md`,
    `docs/marley/workbench-shell.md` and `CHANGELOG.md`.

### Test plan
| REQ | Test |
|---|---|
| 001 | driven: a temp search path holding executable `claude` and `codex` and a non-executable `gemini` lists Claude Code and Codex under the header, and not Gemini CLI or OpenCode; an empty search path shows no header |
| 002 | driven: clicking Claude Code in alpha's `+` opens a center terminal in alpha's directory (the factory's record) |
| 003 | unit: `agent_kind_of` over the four programs, paths, arguments and non-agents. driven: a terminal whose foreground command reads `claude` draws Claude's agent row |
| 004 | driven: a breadcrumb title becomes the row's title, and without one the display name does |
| 005 | unit: `agent_status` around `WAITING_AFTER` and with the bell. driven: output reads working; `advance_clock(WAITING_AFTER)` reads waiting; a bell after fresh output reads waiting |
| 006 | driven: the foreground command going back to the shell returns the row to a plain terminal row |
| 007 | driven: the new terminal's `take_pty_write_log` is exactly `claude` and a carriage return |
| 008 | `marley_agent`'s tests; `script/gates.sh --diff` |

The live drive needs clicks. It runs only while Chad is away from the desk, or it is recorded
as #439's was.

### Risks
- A thinker that writes nothing reads as waiting until it prints again: acceptable, since the
  bell and title refine it, and the spinners Claude Code and Codex draw keep output flowing.
- Timer churn: one timer per agent terminal, replaced on each output, so a streaming agent
  holds one pending timer, never a queue.
- False recognition needs another program named `claude`, `codex`, `gemini` or `opencode` in
  the foreground. That is possible but harmless: the row only relabels.

## Phase 2 — Code
- **Checklist** (no `TaskCreate`): [x] ledger row · [x] `marley_agent` · [x] `marley_rail` ·
  [x] seams and state · [x] snapshot · [x] row · [x] menu · [x] launch · [x] tests · [x] review.
- **Built.**
  - The `Cargo.toml` row names `marley_agent` among the `[workspace.dependencies]` entries;
    that entry was added after the row.
  - `marley_agent`, rewritten:
    - four kinds with `ALL`, `program()` and `display_name()`;
    - `agent_kind_of` over all four;
    - `send_payload`, and `launch_input` built on it;
    - `AgentStatus { Working, Waiting }` with `label()`;
    - `WAITING_AFTER` = 2 s, `agent_status(quiet_for, bell)`, and `status_line`.

    The tick API, `AgentRun`, `launch_command` and the `Idle` and `Exited` states are gone:
    nothing used them. 5 tests.
  - `marley_rail`: `TerminalAgent { kind, status }` on `TerminalSnapshot` and `TerminalRow`,
    and a dependency on `marley_agent`. One new test: an agent row carries its agent, and a
    quiet agent raises no attention.
  - `marley_workbench::rail`:
    - the seams `foreground_command` (`fn(&Entity<Terminal>, &App) -> Option<String>`) and
      `agent_search_path` (the process's `PATH` by default);
    - output times and quiet timers per terminal, pruned to the open terminals;
    - `note_output` on a view's `Wakeup`, which the view also sends for a bell;
    - `terminal_snapshot`: recognition, title, status line and status;
    - the agent icon in the row, with a `marley-rail-agent-{id}` selector;
    - `agent_cli_entries` under an "Agent CLIs" header, via `agents_on_path` over
      `which::which_in`;
    - `new_agent`: the center terminal, then the handshake raced against a 5 s timer, then
      `write_init_command_after_startup`, and an `ensure!` whose error reaches
      `detach_and_prompt_err`.
  - The manifest gains `futures`, `marley_agent` and `which`, and in dev `tempfile` and
    `terminal`'s `test-support`.
  - 5 driven tests and a unit test in `rail::tests::agents` (unix only, for the executable
    bit).
- **Deviations from the design.**
  - The foreground seam takes `(&Entity<Terminal>, &App)`, not `&Terminal`, so a test can
    look up its fake command by the terminal's entity.
  - clippy's `too_many_lines` split the collector further: `Watched::in_window` gathers what
    the rail follows, and `terminal_snapshot` builds one terminal's row.
  - Coverage found the `})?;` trap again
    (L-claude-438-the-coverage-floor-counts-lines-per-function-001) on the launch's two
    `terminal.update` calls. `--show-missing-lines` printed nothing while the per-function
    summary counted two lines. The closures are now bound first
    (`let startup = terminal.update(cx, handshake)?;`), which puts each `?` on an executed
    line.
  - The view's non-output branch (any terminal event but `Wakeup`) had no test; the status
    test now sends `SelectionsChanged` and checks that a quiet agent stays waiting.
- **Review of the diff.**
  - The launch writes a fixed program name only (REQ-007, the write-log test).
  - The quiet timer is replaced on each output and pruned when its terminal closes.
  - `note_output` refreshes its own entity inside its own subscription, so nothing re-enters.
  - `new_agent` updates only the workspace, and spawns the launch on it, so the launch
    outlives a layout switch.
  - Provenance: the launch follows the handshake's public contract (start, race a timeout,
    write after startup) in the rail's own shape; Zed's body uses `select_biased!` and a
    non-PTY branch, the rail neither.
  - Upstream: `Cargo.toml` and `Cargo.lock` only, both under their rows.
- **Checks.** Clippy is clean at deny level on `marley_agent`, `marley_rail` and
  `marley_workbench` with all targets. The three suites: 64 passed. `cargo llvm-cov` over the
  three crates: 100.00% of 1,948 lines. `cargo fmt --all --check` is clean.

## Phase 3 — Test
- **Checklist** (no `TaskCreate`): [x] REQ-001 to REQ-008 · [x] the gate · [x] the live drive
  (recorded below: not run, and why).
- **Tests** (written in Code, run here).
  - `marley_agent`, 5 units: programs, names and order; recognition of all four with paths
    and arguments, and rejection of others; launch input; status around `WAITING_AFTER` and
    with the bell; status lines.
  - `marley_rail`: an agent row carries its agent, and raises no attention.
  - `marley_workbench::rail::tests::agents`:
    - `the_plus_menu_lists_the_agent_clis_on_the_search_path` (REQ-001: executable
      `claude` and `codex` listed; a non-executable `gemini` and a missing `opencode` not; an
      empty search path adds nothing);
    - `an_agent_cli_starts_in_a_new_terminal_in_its_project` (REQ-002, REQ-007: alpha's
      directory, and the write log exactly `claude` and Enter);
    - `a_terminal_running_an_agent_cli_is_an_agent_row` (REQ-003, 004, 006: argv with a path
      and arguments, the display name, then the CLI's title, then back to a plain row);
    - `an_agent_works_while_output_flows_and_waits_when_it_stops` (REQ-005: waiting before
      any output, working on output, waiting after `WAITING_AFTER` on the executor clock, a
      non-output event changing nothing, a bell reading waiting at once);
    - `each_agent_cli_draws_its_own_icon`.
- **The gate** (REQ-008). `script/gates.sh --diff`, touched `marley_agent`, `marley_rail`
  and `marley_workbench`; **19 passed, GATE GREEN [diff]** after 68 s on the first run.
  gate:3: 332 tests passed. gate:4: 64 tests; TOTAL 1,948 lines, 0 missed, 100.00%.
- **Live drive: not run.** It needs clicks (the `+` menu, Claude Code), and input goes into
  Chad's session only while he is away. `rusty headless status` showed his Microsoft Teams
  fullscreen on workspace 3, which the headless output would take, with Nautilus and Sublime
  beside it. What the rows do is proven by the driven tests, which draw and click them. Their
  look (the agent icon, the status line) is owed to the next headless capture, along with
  #439's thread rows.
- **Pre-existing — not in scope:** none.

## Phase 4 — Complete
- **Checklist** (no `TaskCreate`): [x] document · [x] capture knowledge · [x] close the ticket ·
  [x] archive · [x] commit.
- **Documented.**
  - `CHANGELOG.md`: Added (agent CLIs in rail terminals) and Changed (`marley_agent` is the
    fork's agent-CLI model).
  - `docs/marley_architecture/marley_agent.md`, rewritten for the fork: the surface,
    consumers and tests, with the gpui-era cockpit moved into a history paragraph.
  - `marley_rail.md`: the agent on a terminal row, and the new dependency.
  - `marley_workbench.md`: an Agent CLIs section, the tests, and the known limits.
  - `docs/marley/workbench-shell.md`: W4 shipped as #440.
  - Rows: `Cargo.toml` names `marley_agent` in `[workspace.dependencies]` (changed in Code,
    before the edit); `Cargo.lock` is still described by its row.
- **Knowledge appended.**
  - `PR-claude-bind-a-multi-line-closure-before-its-question-mark-001`: the `})?;` coverage
    trap, the second time it bit (#438, #440).
  - `L-claude-440-testing-agent-clis-without-a-pty-001`
  - `AD-claude-440-agent-clis-are-read-from-argv-and-started-in-one-click-001`

  No F block: Code and Test found no bug in shipped code.
- **Brain.** Consultation `a56ec3fdce2c4c36b301035d22afec18` closed with
  `decisions/agent-clis-in-the-rail-are-read-from-argv-judged-by-output-started-in-one-click`,
  follow-up by 2026-10-15.
- **Ticket.** #440 moved to `tickets/closed/`. Its backlog row left at promotion.
