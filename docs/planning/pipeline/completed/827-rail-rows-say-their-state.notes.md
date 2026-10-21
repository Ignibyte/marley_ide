# Rail rows say their state by color — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-827-rail-rows-say-their-state.md
- **Pipeline spec:** 827-rail-rows-say-their-state.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #833 (Wave 2's first); Chad's overnight goal.
- **Classification / tier:** feature, M. Marley crate only (`rail.rs`, `rail_sessions.rs`).
- **Recall (§18.3):**
  - F-claude-603 / F-claude-750: a row's trailing parts squeezed its text; the tile keeps the
    icon slot's `size_7`, so the text's width does not change.
  - #542's states (`Attention`) and its scenario's stand-in give every state on demand.
  - The brain (consultation cc517746): nothing on this seam.
- **Discovery:** `row_card` (`rail.rs:10763`, the icon in a `size_7` `rounded_full` raised
  circle), its 14 callers, `RowLine { text, state, color, cut }` (:10694), `render_terminal_row`
  (:6391, `rail_terminal_icon`, the subtitle first), the thread row's card (~:6970/:7030),
  Browser rows (~:6162), `tmux_row` (:7968) and `rail_sessions.rs`'s row, `project_name`
  (:10490, the summary in Muted), `TerminalRow { agent, bell, command, … }`
  (`marley_rail.rs:697`).
- **Decisions:** D1 to D3 in the spec.

### Design
- In `rail.rs`, beside `RowLine`:
  - `enum RowState { Working, NeedsYou, Done, Idle }` with `dot(cx) -> Hsla` (warning, conflict,
    success, text_muted) and `line(cx) -> Color` (`Color::Warning`, `Color::Conflict`,
    `Color::Custom(text_accent)`, `Color::Muted`), and `from_attention(Attention)`.
  - `enum RowKind { Thread, AgentTerminal, Shell, Browser, Tmux }` with `tint(cx) -> Hsla`.
  - `struct RowIcon { element: AnyElement, look: Option<(RowKind, Option<RowState>)> }` with
    `From<AnyElement>` (no look).
- `row_card(.., icon: impl Into<RowIcon>, ..)`: with a look, the slot is `rounded_md` on the
  kind's tint, `relative`, with the state's 8 px dot `absolute().bottom(-1).right(-1)`, bordered
  in the panel's background so it reads on any tint; without, today's circle.
- Terminal rows: kind `AgentTerminal` when `row.agent` is some, else `Shell`; state from
  `row.agent` (Waiting / Failed with bell → NeedsYou, Working → Working, else Done when `row.bell`,
  else Idle), or for a shell `row.command.running` → Working, `row.bell` → Done, else Idle. The
  first line (the subtitle, or the command line when there is no subtitle) takes `line(cx)` unless
  it already has its own color (limited → Warning stays).
- Thread rows: kind `Thread`, state from `thread_attention_class` on the row's status and
  attention; the first line's color likewise.
- Browser rows: kind `Browser`, no state. tmux rows (`tmux_row`, `rail_sessions::session_row`):
  kind `Tmux`, no state.
- `project_name(.., summary, attention: bool)`: the summary in `Color::Conflict` while
  `attention`; the call in `render_project_row` passes `row.attention`.
- **File manifest:** `crates/marley_workbench/src/rail.rs` (Marley),
  `crates/marley_workbench/src/rail_sessions.rs` (Marley), the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001/002 | `a`, `b`, `c` with stand-ins working / waiting / finished; a `sleep 60` shell in `a`; a `false` shell in `c`; one tmux session | `827-01-night` |
| REQ-003 | `b` folded by its disclosure (point from the probe) | `827-02-folded` |
| REQ-001/002 | theme Marley Day | `827-03-day` |

### Risks
- A dot at the tile's corner can be clipped by the row's `overflow_hidden`; the probe shot shows
  it.
- The first line's color must not override the limited (#715) and running-error (#572) colors.

## Phase 2 — Code
- **Built:** in `rail.rs`, `RowState { Working, NeedsYou, Done, Idle }` (`from_attention`, `dot`,
  `line`), `RowKind { Thread, AgentTerminal, Shell, Browser, Tmux }` (`tint`), `RowIcon` (with
  `From<AnyElement>`, so the other ten callers are unchanged); `row_card` draws a looked row's
  icon on a `rounded_md` tile of the kind's tint with an 8 px state dot, bordered in the panel's
  color, at the bottom-right. Terminal rows: `terminal_row_state` (agent status, then a running or
  failed command for a shell, then the bell) and the first muted line in the state's color.
  Thread rows: `thread_row_state`, `thread_row_icon`, the subtitle in the state's color. Browser
  and tmux rows (`tmux_row`, `rail_sessions`) get their kind's tile. `project_name` takes
  `attention` and draws the summary in `Color::Conflict`.
- **Deviation:** a shell whose last command failed reads as needs you, as the board's `exit 101`
  row does (D1 extended).
- **Review of the diff:** the first line's color applies only to a muted line, so #715's limit
  (warning) and #572's running error (red) keep theirs.
- **Gate:** clippy at source (a 105-line `render_thread_row` → `thread_row_icon`; identical idle
  arms merged); `script/gates.sh --diff` 17/17.

## Phase 3 — Test
- **Scenario:** `script/e2e/827-rail-rows-say-their-state.sh` (sway, the gated tree), exit 0,
  three shots, read as rail crops. One probe run before the gate set b's disclosure point.
- `827-01-night`: `a`'s Claude Code on its tile with an amber dot and "working · Run the build"
  in amber; `a — sleep 60` with an amber dot and "sleep 60 · running"; `b`'s Claude Code with a
  pink dot and "waiting · Write the README" in pink; `c`'s finished Claude Code with a green dot
  and "idle · Tidy the imports" in the accent; `c — bash` with a pink dot and "false · exit 1"
  in red (its own error color kept); `build` (tmux) on its tile (REQ-001, REQ-002 met).
- `827-02-folded`: `b` folded: "1 waiting" beside its name in pink (REQ-003 met).
- `827-03-day`: the same rail in Marley Day, the tiles' tints and dots clearer on the light ground
  (REQ-001, REQ-002 met).
- A lead, not a red: in Marley Night the kind tints are faint, since the theme's
  `warning_background`, `info_background` and `hint_background` sit close to the panel. The dots
  and the lines carry the state; stronger tints would be a theme change (#828's tokens), noted
  for Chad's review.

## Phase 4 — Complete
- **Docs:** CHANGELOG (Changed, #827); the guide's rail rows; the architecture note's rail
  section; look-plan TICKET-827 done. No Zed path touched.
- **Knowledge:** no bug reached Code or Test; no new lesson beyond L-claude-831's probe-run
  practice, which this ticket followed.
- **Brain:** decision on consultation cc517746.
- **Ticket** closed; pipeline archived; committed on `marley/workbench-shell`.
