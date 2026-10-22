# The thread's plan, card labels, reply chips, terminal peek and checks — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-864-the-thread-plan-chips-terminal-peek-and-checks.md
- **Pipeline spec:** 864-thread-plan-labels-chips-peek-checks.spec.md

## Phase 1 — Plan
- **Request:** the Queue's next row after #863.
- **Classification / tier:** feature, M. Marley's details panel; one Zed hunk set in the thread
  view.
- **Recall (§18.3):** a probe with a scripted ACP agent (plan, edit diff, terminal through
  `terminal/create`) shot Zed's current thread; #825's details panel is Marley's (`thread_tab.rs`).
  The brain had nothing.
- **Decisions:** D1–D3 in the spec.

## Phase 2 — Code
- `thread_view.rs` (Zed): `MarleyThreadLook`; `marley_card_label`, a framed XSmall label in the
  buffer font; an edit's tool call label leads with EDIT, a terminal card's command slot with RUN;
  `render_thread_controls` stacks `render_marley_chips` (Continue, Explain, Undo the Edits) over
  the faded controls at the thread's bottom, each chip clearing the message box and inserting its
  words (`MessageEditor::set_text` is test-only). `agent_ui` re-exports the look.
- `thread_tab.rs`: `plan_of`/`plan_section` (PLAN, "N of M", a check, an arrow or a dot per
  entry), `terminal_peek`/`peek_section` (ITS TERMINAL: the newest terminal call's command and its
  last four lines), `checks_of`/`checks_section` (CHECKS: each changed file's errors or warnings
  from its buffer's diagnostics).
- `search_looks.rs` sets the look.
- Changed after the first shot: CHECKS read "1 errors"; the counts are singular for one.
- Gate: `just gate-diff` 17/17 green.

## Phase 3 — Test
Scenario `script/e2e/864-thread-plan-labels-chips-peek-checks.sh` (sway, #825's scripted ACP
agent, extended: a plan, `src/main.rs` written into the scratch crate by its path, an edit diff of
notes.md, a three-line build through `terminal/create`).
- `864-01-thread`: "EDIT  Edit notes.md", "RUN  cargo build", the chips Continue, Explain and Undo
  the Edits under "The build is green."; the panel's PLAN 1 of 3 (a check, an arrow, a dot), ITS
  TERMINAL "$ sh -c for i in 1 2 3; …" and its four lines, CHECKS "src/main.rs 1 error"
  (REQ-001 to REQ-004).
- `864-02-chip`: after Explain, the message box holds "Explain what you changed and why." (REQ-003).
- `864-03-day`: the thread in Marley Day (all).
- The first run wrote src/main.rs in Marley's agent folder, its config folder, which no language
  server reads (CHECKS said no problems); the scripted agent writes the scratch crate's file now.

## Phase 4 — Complete
- CHANGELOG (Added), `marley_workbench.md`, the ledger rows, look-plan W2.
- Brain decision recorded on consultation d031d50c.
