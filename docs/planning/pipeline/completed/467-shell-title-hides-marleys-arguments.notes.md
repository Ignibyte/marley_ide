# A Marley shell's title leaves out Marley's arguments — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-467-shell-title-hides-marleys-arguments.md
- **Pipeline spec:** 467-shell-title-hides-marleys-arguments.spec.md

## Phase 1 — Plan (2026-09-23)
- **Checklist** (no `TaskCreate`): [x] pick · [x] pre-flight · [x] recall · [x] mint · [x]
  prior-art sweep · [x] spec · [x] design · [x] present (autonomous).
- **Request:** found while capturing the rail before the restyle Chad asked for ("The left pane
  lets make it look like the warp.dev one"): the terminal row and the tab read
  `marley_ide — bash --rcfile /mnt/fast/t…`. A regression from #463, fixed before the restyle so
  its captures show real titles.
- **Classification / tier:** bug, small; `marley_terminal` and one hunk in Zed's `terminal`.
- **Pre-flight:** no active pipeline, README marker present, cargo idle.
- **Recall (§18.3).**
  - `AD-claude-463-bash-loads-marleys-hooks-through-rcfile-prompt-command-and-ps0-001`: why the
    rcfile argument exists and that a shell the user starts with arguments is left alone, so
    the integration's arguments are the shell's only ones.
  - The ledger has nothing on terminal titles.
  - Brain: consultation `94d884320dd5472299a152482877e2e5`, nothing on this seam.
- **Discovery:** `Terminal::title` (`terminal.rs:3060-3105`) joins `argv[1..]`;
  `marley_shell_integration` (`terminal.rs:82`) computes the directory
  `paths::data_dir().join("shell_integration")`.

### Design
- `marley_terminal::shell_integration::shown_arguments(argv, dir) -> Vec<&str>`: `argv` after
  the program, less the first contiguous run equal to `for_program(argv[0], dir)`'s arguments.
  An empty `argv`, a program with no integration, or no such run returns the arguments as
  they are.
- `terminal.rs`: `marley_integration_dir()`, the directory, shared by `marley_shell_integration`
  and `title`; in `title`, `(argv[1..]).join(" ")` becomes
  `shown_arguments(argv, &marley_integration_dir()).join(" ")`.
- **File manifest.**
  - Marley: `crates/marley_terminal/src/shell_integration.rs`.
  - Zed: `crates/terminal/src/terminal.rs` (the helper, the title hunk, a PTY test). The row for
    `terminal.rs` in `docs/marley/zed-touchpoints.md` gains the title hunk before the edit.

### Test plan
| REQ | Test |
|---|---|
| 001 | unit: `bash --rcfile <dir>/marley.bash` and `/usr/bin/bash --rcfile …` show no arguments; the run is removed from among other arguments |
| 001 | PTY: `marley_bash_reports_each_typed_command_as_a_block`'s shell, after its `.bashrc` marker: the loaded process info names bash, and `title(false)` holds no `--rcfile` |
| 002 | unit: a program with no integration keeps every argument; bash with an rcfile of its own, a lone `--rcfile`, or the pair split apart keeps every argument; an empty argv shows nothing |
| 003 | `script/gates.sh --diff` |

### Risks
- **A user's own `bash --rcfile <Marley's script>`** would lose the pair from its title too. Only
  Marley writes that path.

## Phase 2 — Code (2026-09-23)
- **Checklist:** [x] recall · [x] marker present · [x] ledger row first · [x]
  `shell_integration.rs` · [x] `terminal.rs` · [x] fmt · [x] review.
- **Built.**
  - `marley_terminal::shell_integration::shown_arguments`, with two unit tests.
  - `terminal.rs`: `marley_integration_dir()`, now used by `marley_shell_integration` too; the
    title's arguments come from `shown_arguments`; the PTY test
    `marley_bash_is_titled_without_the_integrations_arguments`.
- **Deviations:** none.
- **Review.**
  - `slice::windows` panics on a zero size, so an empty run is never searched for; a program
    with no integration yields an empty run and every argument.
  - The title keeps Zed's structure, the space after a bare shell's name included (D3); only the
    joined arguments changed.
  - `argv[0]` is the program as spawned (`bash` or a path), which `for_program` reads by its
    file stem, as the builder did. A login shell's `-bash` matches nothing and keeps its
    arguments, and Marley never starts one with its own.
  - Upstream: two hunks, each marked, with the ledger row updated first; the process info is
    untouched.

## Phase 3 — Test (2026-09-23)
- **Checklist:** [x] REQ-001 tests · [x] REQ-002 tests · [x] negative check · [x] live drive · [x]
  gate.
- **Tests.**
  - `marley_terminal`: `a_shell_started_with_marleys_integration_shows_only_its_own_arguments`
    (REQ-001), `any_other_process_shows_every_argument` (REQ-002); the crate's
    `shell_integration` tests, 4 passed.
  - `terminal`: `marley_bash_is_titled_without_the_integrations_arguments` (REQ-001), a test of
    its own rather than a step in `marley_bash_reports_each_typed_command_as_a_block`, so each
    names one behavior; the five `marley_` tests passed.
- **Negative check.** With the title's join put back to `(argv[1..]).join(" ")`, the PTY test
  fails: `left: ".tmpW4qYkg — bash --rcfile /home/cpeppers/.local/share/marley/shell_integration/marley.bash"`,
  `right: ".tmpW4qYkg — bash"`. The file was restored by checksum.
- **Gate.** The first run was red on gate:2: `clippy::too_long_first_doc_paragraph` on
  `shown_arguments`; the doc now opens with a one-line summary. The second run: `GATE GREEN
  [diff]`, 20 passed, receipt `48043aea…` matching the tree.
- **Live drive.** The debug `marley`, built after the gate, on a copy of Chad's profile on hidden
  workspace 9, captured by toplevel (`grim -T`) with no input sent
  (L-claude-467-capture-one-window-by-its-toplevel-001). The tab and the rail's terminal row
  both read `marley_ide — bash`; before the fix the same capture read
  `marley_ide — bash --rcfile /mnt/fast/t…`.

## Phase 4 — Complete (2026-09-23)
- **Docs (§21).** `CHANGELOG.md` (Fixed); `docs/marley_architecture/terminal_blocks.md`
  (`shown_arguments`); the `crates/terminal/src/terminal.rs` row in
  `docs/marley/zed-touchpoints.md`, written before the edit, describes the title hunk and the
  shared directory helper as shipped. The three-prong plan's T0 rows are unchanged: this fixes
  a slice already shipped.
- **Ledger (§19).** `F-claude-467-a-marley-shells-title-showed-its-rcfile-001`,
  `L-claude-467-see-a-process-marley-changes-where-zed-shows-it-001`,
  `L-claude-467-capture-one-window-by-its-toplevel-001`. No PR block: one occurrence, and the
  lesson carries the check.
- **Brain.** Consultation `94d884320dd5472299a152482877e2e5` closed by
  `decisions/marleys-terminal-title-leaves-out-the-arguments-its-shell-integration-adds`,
  follow-up 2026-10-07.
- **Ticket** closed; the pipeline archived; one commit.
