# A terminal's rail row holds two lines — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-796-terminal-rows-hold-two-lines.md
- **Pipeline spec:** 796-terminal-rows-hold-two-lines.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "the left terminal panes show what they are running underneath
  the names. But once they hit 2 lines it makes the pane jump. We need to probably figure out a way
  to handle this or make the height always assume 2 lines so its not". He chose to run it on the
  box ("Do it here"), and is not moving to the Mac ("that box just doesnt have enough memory").
- **Classification / tier:** bug, one Marley crate, one function's lines plus two stale test
  assertions.
- **Pre-flight:** cargo, the gate, e2e, shear and the hooks OK; no active pipeline; the README
  marker present; cargo busy with another project's `cargo test` (`/srv/stacks/elephc`), so the
  Code phase waits for it (`just idle`).
- **Recall (§18.3):**
  - AD-claude-468-the-rail-draws-its-own-rows-after-warps-tab-list-001: `row_card` drew terminal
    and thread rows at `h_11`, "the title over an optional second line", after Warp's tab list.
  - #519's notes: "`row_card` has one fixed-height subtitle and is shared with thread rows, and a
    test pins equal heights for rows with and without a second line"; #519 made a row with two
    lines 3.5 rem and left the others at `h_11`. That split is the jump.
  - #551 added the command's line under a plain terminal's folder line without touching heights.
  - The brain (`rusty-cli brain ask`, consultation 7919cb5b…) had nothing on rail row heights.
- **Discovery:**
  - `crates/marley_workbench/src/rail.rs:10324` `row_card`: `0 | 1 => h_11`, `2 => 3.5 rem`,
    `_ => 4.5 rem`, from `lines.len()`.
  - `rail.rs:6120` `render_terminal_row`: the lines are the subtitle (folder for a shell, status
    for an agent), activity, quota, the command (`command_line`), the running error, the
    Interrupted chip.
  - `marley_rail::working_directory_label` gives `None` at the project root, so a shell's folder
    line comes and goes with `cd`.
  - `rail_tests.rs:1524` pins a terminal row at 2.75 rem; `:1899` pins a thread row at a terminal
    row's height; `:2506` pins a shell's and an agent's rows at one height (still true after).
  - No golden scenario clicks the rail at fixed coordinates, so the taller rows move no click.
- **Decisions:** see the spec's D1-D3.

### Design
- **Approach:** in `render_terminal_row`, collect the lines into a `Vec<CardLine>` as now, then
  push blank lines (`RowLine::muted(String::new())`) until there are two. `row_card` then picks
  its two-line height, and the title keeps the top of the text column. An empty `Label` is one
  line high in gpui (its text shapes to one empty line); if the visual check shows otherwise, the
  blank becomes a no-break space. One comment says why the blanks are there.
- **File manifest:**
  - `crates/marley_workbench/src/rail.rs` (Marley crate): the padding.
  - `crates/marley_workbench/src/rail_tests.rs` (Marley crate): the terminal row's height to
    3.5 rem; the thread-row test compares icons and their left edge, not heights.
  - `script/e2e/796-terminal-rows-hold-two-lines.sh` (scenario).
  - No Zed crate, so no ledger row.

### Visual check plan
| REQ | What the scenario does | Shot |
|---|---|---|
| REQ-001 | Three terminals: T1 `cd sub`, T2 the stand-in `claude` before its first event, T3 idle at the root. | `796-01-before`: T1 `sub` over a blank, T2 status over a blank, all three rows one height |
| REQ-002 | T1 runs `sleep 30`. | `796-02-command`: T1 `sub` over `sleep 30 · running`; T1's title and T2, T3 at their `796-01` places |
| REQ-003 | T2's stand-in plays SessionStart, UserPromptSubmit, PreToolUse Bash. | `796-03-activity`: T2 status over the tool; T2's title and T3 at their `796-02` places |
| REQ-004 | T1: Ctrl+C, `cd ..`. | `796-04-root`: T1 `cd .. · done` over a blank, the row's height unchanged |

Comparing places across shots needs the order fixed: `marley.rail_order` = `window`. What no
scenario reaches: the quota line and the Interrupted chip (out of scope; they are the third-line
case).

### Risks
- A plain terminal at the root that has run nothing is now taller, with blank space under its
  title: the price of a row that does not jump. Thread and Browser rows beside it stay at
  2.75 rem, so a project's rows are no longer one height. Put to Chad at review: terminal rows
  only, or every row at two lines. He chose terminal rows only.

## Phase 2 — Code
- **Built:** `render_terminal_row` (`crates/marley_workbench/src/rail.rs`) collects its lines
  first, then pads them to two with `CardLine::from(String::new())`, so `row_card` takes its
  two-line height. One comment says why.
- **Tests in the tree:** `a_terminal_row_is_a_fixed_height_with_a_round_icon` now pins 3.5 rem;
  `a_thread_row_is_drawn_as_a_terminal_row_is` pins the thread row at its own 2.75 rem instead of
  a terminal row's height. Neither runs in the gate (§7); both still build.
- **Scenario, written now:** `script/e2e/796-terminal-rows-hold-two-lines.sh`. The gate's
  fingerprint covers `script/e2e`, so it is in the tree before the receipted run. Deviation from
  the plan: the shell starts in `sub` through its `.bashrc` (the first shell to start takes a
  lock and changes folder), because a typed `cd sub` would itself be a block with its command
  line, and `796-01-before` needs the folder line alone.
- **Review of the diff:**
  - REQ-001-004: every terminal row now has at least two lines, blanks last. A row with the
    folder line alone, the status alone or nothing gets the same 3.5 rem as one with two real
    lines; rows with three or more keep 4.5 rem (D3).
  - An empty `Label` holds one line's height: gpui's `shape_text` splits `""` into one empty
    line (`crates/gpui/src/text_system.rs`, `text.split('\n')`), and `TextLayout` sums
    `line.size(line_height)` per line (`crates/gpui/src/elements/text.rs`).
  - No entity is read or updated; no error path; no Zed crate touched; nothing from Warp's
    source.
  - Older scenarios with fixed rail coordinates: `602` has terminal rows that already had two
    lines (`one` over `cd one · done`); `542` clicks a shell row whose height may change. Neither
    is in the golden set, and no golden scenario clicks the rail at a fixed point.
- **Gate:** `just gate-diff` GREEN [diff], 17 passed, 0 failed, receipt written (log in the
  session scratchpad, `796-gate.log`). It waited about an hour behind another project's
  `cargo test` (`/srv/stacks/elephc`). Dylint's warnings in Zed's `sidebar` crate
  (`SharedString` from a literal) are pre-existing, not in scope.

## Phase 3 — Test
- **Run:** `just build` (2 m 15 s), then `SHOT_DIR=<scratchpad>/796-shots just e2e
  script/e2e/796-terminal-rows-hold-two-lines.sh`, exit 0, under its own headless sway. Focus
  report: "hyprland: 1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it"; sway stopped with the run's Marley, pointer and keyboard. Every shot shows only
  Marley's window.
- **`796-01-before` (REQ-001):** under `repo`, `sub — bash` with `sub` over a blank line,
  the stand-in's row (titled with the shell's `user@host`) with `Claude Code · waiting` over a blank line, and the
  selected `repo — bash` with nothing under its title. The titles sit at about y 165, 223 and 281:
  three rows of one height (the selected card spans y 270-325, 3.5 rem at 16 px). Each title is
  at the top of its row, not centered.
- **`796-02-command` (REQ-002):** the first row reads `sub — sleep 30`, `sub`, then
  `sleep 30 · running`. Its title is still at y 165, the agent's at 223, and the idle shell's card
  at 270-325.
- **`796-03-activity` (REQ-003):** the agent's row reads `working · List the files` over
  `Bash: ls -la`. Its title is still at y 223, and the idle shell's row is unmoved.
- **`796-04-root` (REQ-004):** the first row reads `repo — bash`, then `cd .. · done · 0 s` over
  a blank line: the folder line went and the row kept its height, with the command's line moved up
  under the title.
- **Pixel check:** ImageMagick `compare -metric AE` on the idle shell's row (260×73 at 0,262) and
  on the agent's title band (160×15 at 60,216), each shot against `796-01-before`: 0 pixels differ
  in all six comparisons. Nothing under a changing row moved.
- **Not reached:** the third-line case (a running command's error, the quota line, the
  Interrupted chip), which still grows a row by design (D3).
- No fix was needed; the gate's receipt stands.

## Phase 4 — Complete
- **Docs (§21):** `CHANGELOG.md` under Fixed; `docs/marley/three-prong-plan.md` T7 (after #551's
  row line); `docs/marley_architecture/marley_workbench.md` at `row_card` and at the seat's third
  line. No Zed path, so no touchpoint row.
- **Ledger:** AD-claude-796-a-terminal-row-holds-two-lines-001 (amends AD-claude-468…). No bug was
  found in Code or Test, so no `F-` or `PR-` block. The one lesson met again, the scenario inside
  the receipt, is already L-claude-782-the-in-app-guide-is-inside-the-gates-receipt-001.
- **Brain:** `brain decide` on consultation 7919cb5b →
  `decisions/a-marley-terminals-rail-row-holds-two-lines` (follow-up by 2026-10-31); the duplicate
  consultation f567c309 closed with `no-decision`.
- **Ticket:** closed, moved to `tickets/closed/`; no BACKLOG row (minted and promoted at once).
