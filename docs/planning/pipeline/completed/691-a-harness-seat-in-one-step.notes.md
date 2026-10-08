# A harness seat in one step — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-691-a-harness-seat-in-one-step.md
- **Pipeline spec:** 691-a-harness-seat-in-one-step.spec.md

## Phase 1 — Plan
- **Request:** the cron's item 6, unblocked by harness TICKET-109 (closed 2026-10-07).
- **Recall:**
  - #689: `writes_on`, `call_write`.
  - #690: L-689 (invoke `/pipeline:test` first).
  - The harness's final shape for TICKET-109: refusal codes, a Claude start that waits up to
    60 s, and `opened: false` for a live seat.
- **Design:**
  - `harness::seat_command(cx) -> Option<(PathBuf, Vec<String>)>`:
    - `Source::Command`: the program and its arguments before the first `mcp`;
    - `Source::Embedded`: `Harness.rh`, set by `embed` once found, with `--state` and the root.
  - **`harness_seat.rs`** (new, Marley crate):
    - `NewHarnessSeat` (the action) and `NewSeatModal` (Name, Folder and Role editors, the
      agent, the status, and the run).
    - `create`: `process::output` for `seat add`, then `seat start`. `run_seat` reads stdout as
      JSON on success and stderr's last line otherwise.
    - The tab opens through `harness::open`.
  - **Manifest:** `harness.rs`, `harness_seat.rs`, `marley_workbench.rs` (mod and init), the
    scenario, and the docs.
- **Visual check plan:**
  - The stand-in `rh` logs its arguments. `seat add` prints the add JSON, and `seat start` runs
    the real `rh actor new NAME` and prints its fleet id. Any other call is the real `rh`.
  - REQ-001: the form filled in for `builder`, Codex.
  - REQ-002: the tab of `builder` opens, and the log holds both commands.
  - REQ-003: a second seat with role `foreman` shows `seat_role_reserved`.
- **Risks:** a Claude seat's start takes up to a minute. The form says it is starting, and Escape
  leaves the run going. Its tab opens when the start finishes, if the workspace is still there.

## Phase 2 — Code
- **Built:**
  - `Harness.rh`, set by `embed`.
  - `Harness::seat_command`.
  - `writes_on` made `pub(crate)`. The palette filter and the workspace action cover
    `NewHarnessSeat`.
  - `harness_seat.rs`: `open_form`, `NewSeatModal` (`create`, `render_field`) and `run_seat`.
- **Review:**
  - An empty name or folder is refused before anything runs.
  - A form closed mid-run leaves the run going, and the tab opens if the workspace is still there.
  - stderr's last line without its `rh: ` is the shown refusal.
- **Gate:** runs 1 to 3 were red on clippy:
  - `redundant_pub_crate`, then `unreachable_pub`. The module is now `pub mod`, as its siblings
    are.
  - `unused_self`.
  - The module doc's first paragraph was too long.

  Run 4 (`scratchpad/691-gate-4.log`): **GATE GREEN [diff]**.

## Phase 3 — Test
- **Scenario:** `script/e2e/691-a-harness-seat-in-one-step.sh` (`compositor sway`).
  - Its stand-in `rh` answers `seat` from the script. Its `seat start` opens a real actor, and
    every other call goes to the harness's `rh`.
  - Run 1 measured the form. Run 2 passed, but its tab opened in the repo project (fixed below).
  - Run 3, after the fix and a green gate (`scratchpad/691-gate-5.log`), passed 1 of 1
    (`scratchpad/691-e2e-3.log`). #689's scenario was rerun for the shared change, 3 of 3
    (`scratchpad/689-e2e-5.log`).
- **Shots** (run 3):
  - `691-01-form` (REQ-001): "New Harness Seat" with its line on profiles and the manager role.
    Name is builder, and Agent shows Codex selected. Folder is the run's repo, Role is empty,
    and Create is at the bottom right.
  - `691-02-opened` (REQ-002): the tab builder, showing "the seat is up", under the Home group,
    and the rail's harness section lists builder working. The log holds
    `--state <root> seat add builder --agent codex --cwd <repo>` and `--state <root> seat start
    builder`.
  - `691-03-refused` (REQ-003): the form again with chief, Claude Code and role foreman. It shows
    "seat_role_reserved: the foreman's seat waits for TICKET-113" in red and stays open. Opened
    from Home, its folder defaults to the home folder.
  - `689-06-opened`: greeter's tab under Home, unchanged.
- **Fix, run 2 to run 3:** `harness::open_in_home` sends a tab to the Home group as the rail's row
  does (#676). Both the form and #689's open-session picker now use it, where before a session
  opened from a project went into that project.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it".

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md` (#691).
  - The guide's "A seat in one step".
  - `marley_workbench.md`'s `harness_seat.rs` entry.
  - The plan's item 6, marked done for the form. Its tool is TICKET-692.
- **Knowledge:** `AD-claude-691-a-seat-is-set-up-through-the-followed-harnesss-own-command-001`.
- **Ticket:** TICKET-691 closed.
