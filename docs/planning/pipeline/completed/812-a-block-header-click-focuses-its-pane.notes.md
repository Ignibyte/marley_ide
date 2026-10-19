# A click on a block header focuses its terminal — Notes

- **Local ticket doc:** ../../tickets/closed/TICKET-812-a-restored-blocks-header-takes-the-panes-click.md
- **Pipeline spec:** 812-a-block-header-click-focuses-its-pane.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-19 (`/goal`): "lets do 812, 814, 809, 806". Found by #806's triage of
  575-terminal-id-across-restore: its second launch clicked the right pane on a restored block's
  header, and the command ran in the left pane.
- **Classification / tier:** bug; one Marley crate (`marley_workbench`), one closure.
- **Recall (§18.3):**
  - #628's notes: the header stops the press so a click on it starts no selection; nobody checked
    focus in a split.
  - PR-claude-a-button-in-a-card-that-takes-the-focus-stops-its-click-001 is the reverse case.
  - The brain (consultation ffeaeeb3163f468bb5a4f81b8897c761): nothing on this seam.
- **Discovery:** `crates/marley_workbench/src/block_headers.rs`, `header(_view, terminal, ..)`:
  `.on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())`. The view arrives as
  `_view` and is unused.
- **Decisions:** D1, D2 in the spec.

### Design
- Approach: rename `_view` to `view`, take `view.focus_handle(cx)` (gpui's `Focusable` on the
  entity) when the header is built, and in the mouse-down closure call
  `window.focus(&focus_handle, cx)` before `cx.stop_propagation()`. The release handler is
  unchanged.
- File manifest: `crates/marley_workbench/src/block_headers.rs` (Marley crate);
  `script/e2e/812-a-block-header-click-focuses-its-pane.sh` (new). No Zed path.

### Visual check plan
- REQ-001, REQ-002: under sway, the repository's terminal runs `P=left; echo left`, `pane: split
  right` and `workspace: new terminal` give a right terminal that runs `P=right; echo right`. A
  click in the left pane's body, then a click on the right pane's block header (its place
  measured from a first run's shot), then `where` typed: `where` appends `typed in $P` to a log.
  `812-02-header-clicked` shows `typed in right` in the right pane; `expect` reads the log.
- REQ-003: the stops stay in the diff.

### Risks
- The header's place moves with the bottom-anchored grid; the scenario's click point is measured
  from its own first run, and the shot shows where it landed.

## Phase 2 — Code
- Built to the manifest: `header` takes `view` (was `_view`), reads `view.focus_handle(cx)` when
  it builds the header, and its left mouse-down calls `window.focus(&focus_handle, cx)` before
  `cx.stop_propagation()`; `gpui::Focusable` joins the imports. The release handler and both stops
  are unchanged (REQ-003). The comment on the stops now says why the press still focuses.
- `script/e2e/812-a-block-header-click-focuses-its-pane.sh` written to the plan, with the left
  body and the right header's points as variables to measure from its first run.
- Review: the focus is the handle the terminal element focuses on a press on the grid
  (`TerminalElement::new(.., self.focus_handle.clone(), ..)`), so the pane activation and the
  prompt editor (#627) follow the same way; nothing reads or updates an entity in the closure;
  no Zed path.
- The gate: `just gate-diff` green, 17 of 17.
- Second pass (REQ-004, after the reopening): `sticky_header.rs`'s `header` reads
  `view.focus_handle(cx)` before it clones the view, and its left mouse-down calls
  `window.focus(&focus_handle, cx)` before it stops the press and jumps; `gpui::Focusable` joins
  its imports. The scenario gained the pinned-header steps: with the left pane focused, the right
  terminal is wheeled back (`pointer_to`, `scroll -15`; Shift+Page_Up after the header click left
  it at the live screen in a check run), and the pinned row at y 77 is clicked. The gate:
  `just gate-diff` green again, 17 of 17.

## Phase 3 — Test
- `just build`, then `just e2e script/e2e/812-a-block-header-click-focuses-its-pane.sh` under
  sway, `SHOT_DIR` in the scratchpad. Focus report: "1 Marley windows before the run, 1 after; the
  run added no rule".
- The first run passed with the header's point a guess, (1100, 500): its first shot showed the
  headers at the panes' bottom (rows at y 853 and 871), so that click had landed in the right
  pane's empty body, which focuses in any build. The point moved to (1000, 871), the right
  header's command row, and the scenario ran again.
- Each shot of the second run read:
  - `812-01-left-focused`: both panes draw a header (`…/repo · header` over `P=left; echo left`
    and `P=right; echo right`); the caret is in the left prompt editor, the left pane carries the
    tab bar's buttons, and the rail highlights the left terminal's row.
  - `812-02-header-clicked`: the right pane shows a new block, `where` with `typed in right`; the
    right pane carries the buttons and the rail highlights its row (`where · done`). `expect`:
    "the command typed after the header's click ran in the right terminal: pass" (REQ-001,
    REQ-002).
- Before and after: the same scenario on the installed release build (`E2E_BINARY`, no #812)
  logged `typed in left` and failed that `expect`, the bug as #806's 575 run met it.
- REQ-003: the diff keeps both stops.
- The point change in the scenario followed the gate, so the gate runs again before the commit.
- Second pass, after the reopening (REQ-004), on the debug build after the second green gate:
  - `812-01-left-focused`, `812-02-header-clicked`: as in the first pass (left focused; `where`
    then `typed in right` in the right pane, the rail on its row); `expect` pass.
  - `812-03-pinned`: the left pane has the caret, the rail's highlight and the buttons; the right
    terminal is wheeled back, `seq 1 400` pinned over its top row with the check and the arrow,
    lines 314 to 356 under it.
  - `812-04-pinned-clicked`: the right pane jumped to the block's start (`…/repo · header`,
    `seq 1 400`, `1`, `2`, …); the caret, the rail's highlight and the buttons moved to it.
  - `812-05-typed`: `where`, then `typed in right`, at the bottom of the right terminal; `expect`
    "the command typed after the pinned header's click ran in the right terminal: pass" (REQ-004).
  - Without the fix (`E2E_BINARY`, the installed build): `typed in left`, the first `expect`
    failed and the run stopped there, before the pinned steps. The pinned header's half rests on
    `812-03` to `812-05`: the left pane held the focus until the pinned row's click.

## Phase 4 — Complete
- Reopened at Complete, 2026-10-19: the architecture note for the block headers sits beside the
  pinned command header's (#529, `sticky_header.rs`), whose left press also stops propagation and
  never focuses. Same bug, same fix; the scope gained it as REQ-004 and the status went back to
  Phase 1 PASS, so the Code and Test phases run again.
- Docs: `CHANGELOG.md` (Fixed, #812); `docs/marley_architecture/marley_workbench.md`, the block
  headers' and the pinned header's press bullets. No Zed path, no touchpoint row. No slice status
  moves.
- Ledger appends: `F-claude-812-a-header-that-stopped-a-press-kept-the-focus-elsewhere-001`,
  `PR-claude-812-an-element-that-stops-a-press-over-a-view-focuses-it-001`,
  `L-claude-812-a-click-point-is-checked-against-the-shot-and-a-build-without-the-fix-001`.
- Brain: consultation `ffeaeeb3163f468bb5a4f81b8897c761` closed with `brain no-decision` (a bug
  fixed by the grid's own rule; no choice made).
- Ticket closed; its `BACKLOG.md` row left at promotion.
