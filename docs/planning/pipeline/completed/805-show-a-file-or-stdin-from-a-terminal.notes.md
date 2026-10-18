# Show a file or stdin from a terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-805-show-a-file-or-stdin-from-a-terminal.md
- **Pipeline spec:** 805-show-a-file-or-stdin-from-a-terminal.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17 (see #803).
- **Discovery (drafted by /spec; re-verified at promotion):** `crates/marley_workbench/bin/marley-agent`;
  `crates/marley_mcp/src/agent_socket.rs` (the credential check); `crates/marley_agent/src/report.rs`
  (`Report`, `Release`, `Approve`); the shell integration's scripts in
  `crates/marley_terminal/shell_integration/`. Escape sequences were ruled out: vte drops unknown
  OSCs, and Marley's DCS hooks carry a nonce the shell keeps from programs, so a printed request
  could only be trusted with a user's confirmation.
- **Open for promotion:** the command's name (`show` may clash with a user's own command).
- **Recall (§18.3):** #652's socket (one line of 64 KB, the peer's credentials, a 3.5 s answer)
  and `terminal_of`, which finds the terminal on the caller's process chain and checks its
  `MARLEY_TERMINAL_ID`; #726's `approve`, handled in `take` before `accept` because it answers
  later (a show answers after reading its file the same way); #526's ssh wrapper, defined only when
  the user has none (`declare -F`, `${+functions[ssh]}`), the pattern for `show`; #800's
  `ComponentTab` (reused by title) and #803's `BlockTable` and file preview.
- **Promoted 2026-10-17**; seams re-read on the tree #804 left; the name decided by default
  (spec).

### Design
- **`marley_agent/src/report.rs` (Marley):** `Request::Show { terminal, path, kind, title,
  stdin }`, `ShowKind { Table, Component, Markdown, Image, Json }`, `Refusal::Show`
  (`show_invalid`).
- **`bin/marley-agent` (Marley):** `show [--as …] [--title …] FILE|-`: a missing file or a type
  it cannot name is an error (stderr, exit 1); stdin copied to the runtime folder; no terminal id,
  no answer or another refusal prints the input and exits 0; `show_invalid` prints its reason and
  exits 1. Its deadline is 4 s.
- **`marley_workbench/src/terminal_show.rs` (Marley, new):** `arrived(request, workspace, view,
  show)`: reads the file off the main thread (16 MB at most), checks it by kind, answers, and opens
  the tab through the window: a table (Zed's preview for a readable file, else a `BlockTable`
  titled), a component (`component_tools::open_tab`), markdown (an editor over the buffer and
  Zed's `MarkdownPreviewView` in the terminal's pane), an image (`open_abs_path`, invisible
  worktree). JSON is a component when it parses as one, else a table, else the component's error.
- **`agent_reports.rs` (Marley):** `take` hands a show to `terminal_show::arrived`.
- **`block_tables.rs`, `component_tools.rs` (Marley):** `open_file` with an optional block key,
  a `show_table` that reuses a `BlockTable` of the same title; `open_tab` crate-visible.
- **Shell integration (Marley):** `show` in bash, zsh and fish when `MARLEY_BIN` is set and the
  shell has no `show`.

### File manifest
| File | Crate | Change |
|---|---|---|
| `crates/marley_agent/src/report.rs` | Marley | the request, the kind, the refusal |
| `crates/marley_workbench/bin/marley-agent` | Marley | `show` |
| `crates/marley_workbench/src/terminal_show.rs`, `marley_workbench.rs`, `agent_reports.rs` | Marley | the handler |
| `crates/marley_workbench/src/block_tables.rs`, `component_tools.rs` | Marley | shared openers |
| `crates/marley_terminal/shell_integration/marley.{bash,zsh,fish}` | Marley | the function |

### Visual check plan
| REQ | Set-up and action | Shot |
|---|---|---|
| REQ-001 | `show data.csv` | `805-01-table` |
| REQ-001 | CSV from `python3 -c` piped to `show --as csv --title People -` | `805-02-stdin` |
| REQ-001 | `show card.json` | `805-03-component` |
| REQ-001 | `show notes.md`, `show logo.png` | `805-04-markdown`, `805-05-image` |
| REQ-002 | `env -u MARLEY_TERMINAL_ID "$MARLEY_BIN" show notes.md` | `805-06-outside` |
| REQ-003 | `show missing.csv; echo "exit $?"`, `show bad.json` | `805-07-error` |

### Risks and decisions
- The type is never guessed from content (D2), except `.json`, which the spec names as a tree
  when it parses as one, else a table.
- Markdown and image copies of stdin stay in the runtime folder until the session ends.

## Phase 2 — Code
- **Built:** as the design. Deviation from the drafted spec, recorded there: stdin goes as a
  private file named in the request, not in pieces, since the socket reads one line of 64 KB.
- **Review of the diff:** the file is read off the main thread and checked before the answer, so
  the program's exit code says whether a tab opened; the program is answered exactly once on
  every path (refusal, `ok`); a stdin copy is deleted when Marley has read what it needs, and by
  the program when Marley refuses or does not answer; the tab opens through the window's handle,
  outside any entity's update (F-claude-733); `show` is defined only where the shell has none.
- **Clippy:** `single_match_else`, `needless_pass_by_value`, `needless_pass_by_ref_mut`, each
  fixed at the source. The program was tried outside Marley first: a file printed (exit 0), a
  missing file (exit 1), stdin without `--as` (exit 1), stdin with `--as csv` printed (exit 0).
- **Gate (`805-gate-1.log`)**, on the tree after the scenario: **GATE GREEN [diff]**.

## Phase 3 — Test
- **Scenario:** `script/e2e/805-show-a-file-or-stdin-from-a-terminal.sh` (`compositor sway`).
  One run; every shot shows its criterion.
  - `805-01-table` (REQ-001): `show data.csv`: a tab "Preview data.csv" beside the terminal's tab,
    Zed's preview, 20 rows with file line numbers.
  - `805-02-stdin` (REQ-001): `python3 -c … | show --as csv --title People -`: a table tab "People",
    city and temp, three rows.
  - `805-03-component` (REQ-001): `show card.json`: a Component tab "card.json": the green Deploy
    card, its stat 4 "+1 since noon" and the Rollout bar at 80%.
  - `805-04-markdown` (REQ-001): `show notes.md`: "Preview notes.md": the heading, the bold list
    items and the table rendered.
  - `805-05-image` (REQ-001): `show logo.png`: a "logo.png" tab, the 240 × 120 gradient at 100%,
    the status bar reading `240x120 • 18.1KiB • 3 channels, 24 bits per pixel • PNG`.
  - `805-06-outside` (REQ-002): `env -u MARLEY_TERMINAL_ID "$MARLEY_BIN" show notes.md; echo
    "exit $?"`: notes.md printed unchanged in the block, `exit 0`. (#803 then offered that block's
    markdown table with its chip, as it should.)
  - `805-07-error` (REQ-003): `show missing.csv` printed `show: missing.csv: no such file` and
    `exit 1`; `show bad.json` printed `show: $.children[0]: The type gauge is not in the catalog.
    The types are: …` and `exit 1`. The block pill is the `echo`'s, so the exit code is shown by the
    `echo`.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` ("`show` a file or stdin in a tab"); `docs/marley/guide.md` ("Showing
  a file in a tab (#805)", and a pointer under "An agent's own reports");
  `docs/marley_architecture/marley_agent.md` and `marley_workbench.md`. No Zed path changed.
- **Knowledge:** L-claude-805-the-agent-socket-takes-one-short-line-001. No bug was found in Code
  or Test.
- **Brain:** no decision recorded: the command's name is the spec's default, put to Chad in the
  report.
