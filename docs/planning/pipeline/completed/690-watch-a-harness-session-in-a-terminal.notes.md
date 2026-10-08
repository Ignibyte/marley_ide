# Watch a harness session in a terminal — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-690-watch-a-harness-session-in-a-terminal.md
- **Pipeline spec:** 690-watch-a-harness-session-in-a-terminal.spec.md

## Phase 1 — Plan
- **Request:** the cron's item 5, unblocked by harness TICKET-109 (closed 2026-10-07). The
  harness's message: `session_surface_to_human` now answers agent seats with `rh view WS` (after a
  claim) and, on tmux, `rh attach WS` (direct).
- **Recall:**
  - #689's views and its L-689: invoke `/pipeline:test` before the scenario.
  - #684's `start_in_terminal` with a typed line.
  - `intake/harness-session-live-terminal.md`: its stream observer stays an intake. The plan's
    item 5 runs the view's own command instead.
- **Design:**
  - `HarnessView { workspace: WeakEntity<Workspace> }`, set by `open`.
  - `open_view(index)` takes the view's argv, kept beside its shown line. It quotes each
    argument and has the workspace run `agents::start_in_terminal(workspace, None, None,
    Some(line), None, window, cx)`, logging an error.
  - `views` becomes `(kind, line, argv)`.
  - **Manifest:** `crates/marley_workbench/src/harness.rs`, the scenario, and the docs.
- **Visual check plan:** #689's root with one actor. In its tab, Views, then Open on tmux's
  `rh attach`. Shots `690-01-views` and `690-02-terminal`: the terminal shows the actor's output.
  The native `rh view` needs a claim to type, and it shows the same screen.
- **Risks:** `rh attach` takes over the terminal's screen, and ending it is Ctrl-b d. The guide
  says so.

## Phase 2 — Code
- **Built:** `HarnessView.workspace`, set by `open`, and `open_view`. The line typed is the shown
  one, already quoted by `view_line`. Each view row has an Open button before Copy.
- **Deviation:** no stored argv. `view_line`'s line is the shell form of it, so that line is typed.
- **Review:** a closed workspace makes `open_view` do nothing, and a failed start is logged.
- **Gate:** `scratchpad/690-gate-1.log`: **GATE GREEN [diff]**.

## Phase 3 — Test
- **Scenario:** `script/e2e/690-watch-a-harness-session-in-a-terminal.sh` (`compositor sway`, the
  harness's built `rh`, one actor). Runs 1 and 2 measured the row and Open. Run 3 is the whole of
  it (`scratchpad/690-e2e-3.log`).
- **Shots:**
  - `690-01-views` (REQ-001): native and tmux, each with Open, Copy, then its command line, cut
    with an ellipsis. Under them, the worker's three lines.
  - `690-02-terminal` (REQ-002): a new terminal, "marley_ide — rh -N -S /run/user/1000/r…", under
    the Home group. tmux's status line reads `[actor-9ce0:rh*`, and its pane holds "one from the
    pane", "two from the pane" and "three from the pane".
- **Not Marley's:** the three lines step right because the fixture's bytes have a bare LF and no
  CR, and the pane shows them as written. `session_read` renders logical lines, so the tab does
  not.
- **Focus report:** "1 Marley windows before the run, 1 after; the run added no rule and did not
  reload it".

## Phase 4 — Complete
- **Docs:**
  - `CHANGELOG.md` (#690).
  - The guide's Views bullet.
  - `marley_workbench.md`'s Writes entry.
  - The plan's item 5, marked done.
- **Knowledge:** `AD-claude-690-a-harness-view-runs-as-a-typed-line-in-a-marley-terminal-001`.
- **Ticket:** TICKET-690 closed.
