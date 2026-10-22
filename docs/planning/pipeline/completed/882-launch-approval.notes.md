# A launch config's approval as its board draws it — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-882-launch-remote-and-block-tools.md
- **Pipeline spec:** 882-launch-approval.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #881; split while planning: the approval here, the
  menu, remote terminals and the block tools TICKET-889.
- **Classification / tier:** feature, S. Marley crate only.
- **Recall (§18.3):** #592 (an approval shows a command as written; `verbatim`); #527's scenario,
  whose menu points are stale (its probe opened the Default group's menu).
- **Discovery:** probes of #528 (the block filter is close to the board already) and #527; the
  approval is `window.prompt` with the config's text as a code block.
- **Decisions:** D1 to D3 in the spec.

### Design
- `launch.rs`: `ApprovalRow { glyph, what, place }` from each item; `LaunchApproval` (a
  `ModalView` with a focus handle, `menu::Confirm` runs and `menu::Cancel` cancels, answering a
  oneshot); `run` opens it with `toggle_modal` and awaits the answer where it awaited the prompt.
- **File manifest:** `launch.rs`; the scenario.

## Phase 2 — Code
- **Built:** `launch.rs`: `ApprovalRow` and `approval_rows` (the glyph, from #851's `badge_of`
  for an agent, made `pub(crate)`; the title; what it runs; the folder and split);
  `LaunchApproval`, a `ModalView` with its focus handle, `menu::Confirm` and Run answering true,
  `menu::Cancel` and Cancel false, through a oneshot whose dropped sender reads as Cancel; `run`
  opens it with `toggle_modal` in place of `window.prompt`. A command is a plain text run in the
  buffer font that wraps (D2). The approval's hash and store are unchanged.
- **Review:** clippy asked for `&Context` in `new`; `verbatim` keeps its other callers
  (`turn_undo.rs`).
- **Gate:** green on the first run, 17 of 17.


## Phase 3 — Test
- **Scenario:** `script/e2e/882-launch-approval.sh` (sway): #527's repository, the project's +
  and End, Return; `offline_chromium` for the Browser tab Run opens.
- `882-01-approve`: "Run Dev?", "Dev runs these, once you say so."; `>_ dev server` with
  `python3 -m http.server 8766 --bind 127.0.0.1 # <!-- hidden --> *kept* ```` whole, wrapped, as
  written; `CC Claude Code` (split right); `www http://127.0.0.1:8766/` (split down); Cancel and
  "Run · Enter". REQ-001 met.
- `882-02-day`: Marley Day, the same. REQ-001 met.
- `882-03-ran`: Enter: the dev server running, the stand-in Claude Code to the right, the page
  "The dev server answers" below it. D1 seen.


## Phase 4 — Complete
- CHANGELOG (Changed), `marley_workbench.md`, look-plan W5 row.
- Knowledge: none new. Brain: `brain decide` on consultation 2bf892a4.
- Ticket closed; pipeline archived; TICKET-889 holds the menu, remote terminals and block tools.
