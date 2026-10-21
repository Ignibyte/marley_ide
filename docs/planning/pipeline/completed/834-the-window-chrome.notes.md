# The title bar's palette field and agent pills — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-834-the-window-chrome.md
- **Pipeline spec:** 834-the-window-chrome.spec.md

## Phase 1 — Plan
- **Request:** the Queue's top row after #826 (Wave 3's first); Chad's overnight goal.
- **Classification / tier:** feature, L scoped to M: tabs and menus are TICKET-866.
- **Recall (§18.3):** PR-claude-831 (no workspace read in a hook view's `new`); #833's focus row
  reaches the rail from the window and observes it; the brain (consultation 8b48ed6b): nothing on
  this seam.
- **Discovery:** `title_bar.rs` (`MarleyTitleBarButtons` and `MarleyProjectMark` globals and
  fields; `children` pushed left then right, laid out by the platform title bar with
  `justify_between`); `status_buttons.rs` (`init` sets both hooks; `TitleButtons` renders Rusty
  and Marley); `Rail::agents_at_work` (`rail.rs:1553`, `AtWork { state, .. }`), `Rail::inbox`,
  `open_inbox_entry`; `home_page.rs:354` (`zed_actions::command_palette::Toggle`).
- **Decisions:** D1 to D3 in the spec.

### Design
- **`title_bar.rs`** (Zed): `MarleyTitleBarCenter(pub Arc<dyn Fn(WeakEntity<Workspace>, &mut
  Window, &mut App) -> AnyView>)` + `impl Global`; a `marley_center: Option<AnyView>` field made
  in `new` like the others; in `render`, `children.push(...)` of `marley_center` between the left
  group and the right group (a `div().flex_1().flex().justify_center()` around it so it centers
  in the free space). Row for `title_bar.rs` gains the clause.
- **`status_buttons.rs`** (Marley): `TitleCenter { workspace }` view: renders nothing outside the
  Marley layout (`MarleySettings::layout`); else a `ButtonLike`-style div (h 26 px, `max_w(520)`,
  `w_full`, `rounded_md`, `element_background`, border) with the magnifier, "Search, run a command,
  or ask an agent" muted, and `KeyBinding::for_action(&command_palette::Toggle, cx)`; on click
  `window.dispatch_action(Box::new(command_palette::Toggle), cx)`. Set in `init` as the third
  hook.
- `TitleButtons::render`: find the rail as #833's `FocusRow::rail` does (factor a small
  `window_rail(window, cx)` helper in `rail.rs` or `status_buttons.rs`, observing it), count
  `agents_at_work` by `state` ("working" vs "waiting"/"failed"); pills `h_6 px_2 rounded_full`
  with a dot, warning or conflict background at 15 % and text; needs-you click
  `rail.open_inbox_entry(first key)`.
- **File manifest:** `crates/title_bar/src/title_bar.rs` (Zed), `status_buttons.rs` (Marley),
  `docs/marley/zed-touchpoints.md`, the scenario.

### Visual check plan
| REQ | Setup and action | Shot |
|---|---|---|
| REQ-001/002 | #542's stand-ins: b waiting, a working | `834-01-night` |
| REQ-001 | click the field | `834-02-palette` |
| REQ-003 | Escape; click the needs-you pill | `834-03-opened` |
| REQ-001/002 | theme Marley Day | `834-04-day` |

### Risks
- The title bar can be narrow; the field shrinks (`min_w_0`) before the groups do.
- `TitleButtons` reads the rail in render: the rail is read, never updated there.

## Phase 2 — Code
- **Built:** `title_bar.rs` (Zed): `MarleyTitleBarCenter` (global, `marley_center` field made in
  `new`), its view placed in the children absolutely at `left: 50%` of the title bar's row,
  vertically centered. `status_buttons.rs`: `TitleCenter` (nothing outside the Marley layout; else
  a 440 px field shifted left by half its width, magnifier, words, the palette's binding; a click
  dispatches `command_palette::Toggle`), set as the third hook; `TitleButtons` gains `rail` (found
  and observed as #833's focus row does) and `agent_pills` (counts from `agents_at_work`: working,
  and waiting or failed; the pink pill opens `inbox().first()` through `open_inbox_entry`; the
  amber pill's tooltip lists the agents).
- **Deviation, found in the probe:** the planned `flex_1` middle slot got no width, because the
  title bar's left group takes the row's free width; the field drew squeezed over the pills. The
  slot is now absolute at the bar's middle with the view a fixed 440 px offset by half (the row
  clips overflow, so a narrow window cuts the field rather than the groups). An F block at
  Complete.
- **Gate:** clippy clean at the first run after two warnings fixed in check; `script/gates.sh
  --diff` 17/17.

## Phase 3 — Test
- **Scenario:** `script/e2e/834-the-window-chrome.sh` (sway, the gated tree), exit 0, four shots.
  Three probe runs before the gate (the slot's layout, the pill's point).
- `834-01-night`: the title bar: the cat, `a`, the branch; in the middle the field "Search, run a
  command, or ask an agent" with `Ctrl-Shift-P`; at the right "1 agent working" (amber) and "1
  needs you" (pink) before Sign In (REQ-001, REQ-002 met).
- `834-02-palette`: the field clicked: the command palette open over the center (REQ-001 met).
- `834-03-opened`: Escape, then the pink pill: b's waiting stand-in terminal in front ("b:
  waiting") (REQ-003 met).
- `834-04-day`: Marley Day: the field and pills in Day's colors (REQ-001, REQ-002 met).

## Phase 4 — Complete
- **Docs:** CHANGELOG (Added, #834); the guide's title bar; the architecture note's title bar
  section; look-plan W2.1 done with #866; the `title_bar.rs` row describes the absolute slot.
- **Knowledge:** F-claude-834-a-flex-slot-between-the-title-bars-groups-got-no-width-001,
  PR-claude-834-the-title-bars-middle-is-placed-absolutely-001.
- **Brain:** decision on consultation 8b48ed6b.
- **Ticket** closed; #866 filed; pipeline archived; committed on `marley/workbench-shell`.
