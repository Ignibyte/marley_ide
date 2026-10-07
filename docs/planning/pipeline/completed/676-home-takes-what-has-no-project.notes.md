# Home takes the screens that belong to no project — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-676-home-takes-what-has-no-project.md
- **Pipeline spec:** 676-home-takes-what-has-no-project.spec.md

## Phase 1 — Plan (2026-10-07)
- **Request:** Chad, 2026-10-07 (quoted in the spec).
- **Classification / tier:** feature; `marley_workbench` only.
- **Pre-flight:** green; #675 committed; cargo idle.
- **Recall (§18.3):** #675's routing and its guard; #600's `in_home`; the intake note's decision 4.
- **Discovery:** `agent_tab::open_later` (`fleet.rs:661`), `system_one.rs:409`'s action
  (`system_one_calls::open`), `Rail::open_harness` (`rail.rs`, `harness::open`), `Rail::in_home`.

### Design
- **`groups.rs`**: `with_rusty_group` becomes `with_group(kind, …)` (Home or Rusty, found by
  `group_of_kind`); the waiting list keys on window and kind; `in_group(kind, asked_from, window,
  cx, open)` moves here from `rusty.rs`.
- **`rusty.rs`**: `in_rusty_group` calls `groups::in_group(GroupKind::Rusty, …)`.
- **`agent_tab.rs`**, **`system_one.rs`**, **`rail.rs`** (`open_harness`): through
  `in_group(GroupKind::Home, …)`; `in_home` makes Home through `with_group`.
- **File manifest:** those files, the guide, the scenario; docs.

### Visual check plan
| Criterion | What the scenario does | Proof |
|---|---|---|
| REQ-001 | `marley: open system one calls` from the project | `676-01-calls` |
| REQ-002 | clicks the project's terminal | `676-02-project` |
| REQ-003 | the action again from the project | `676-03-again` |

### Risks
- A Home group a user renamed is no longer Home (#600's `rename`); the next open makes a new Home,
  as the empty-space menu already does.

### Checklist (no TaskCreate in this harness)
- [x] Pick, pre-flight, recall; discovery.
- [x] Mint the pair; the ticket in progress.
- [x] Prior-art sweep; spec; design; visual check plan; risks.
- [x] Phase 1 PASS under Chad's request.

## Phase 2 — Code (2026-10-07)
### Built
- **`groups.rs`**: `group_of_kind`; `with_group(kind, …)` (from #675's `with_rusty_group`);
  `in_group(kind, …)` (from #675's `rusty::in_rusty_group`); `Groups::waiting` by window and kind.
- **`rusty.rs`**: `in_rusty_group` calls `in_group` with Rusty.
- **`agent_tab.rs`**: `open_later` through `in_group(Home)`; its `ResultExt` import went with the
  old body.
- **`system_one.rs`**: the `OpenSystemOneCalls` action through `in_group(Home)`.
- **`rail.rs`**: `open_harness` through `in_group(Home)`; `in_home` makes Home through
  `with_group`.
- **The guide**: the empty-space item says Home also takes these screens.

### Review
- The Settings page's System One Calls link dispatches the same action, so it lands in Home too.
- `in_home` still runs its action directly when Home exists (it is called inside the rail's own
  update); only the making goes through `with_group`.
- The Agent tab's and the harness tab's routing is the same call as System One calls'; the check
  drives System One calls, as the spec's D2 says.

### Gate
`just gate-diff` on the tree with the scenario: GATE GREEN [diff], 17 of 17.

## Phase 3 — Test (2026-10-07)
`script/e2e/676-home-takes-what-has-no-project.sh` (`compositor sway`), on the debug build.

| Criterion | Shot | What it shows |
|---|---|---|
| REQ-001 | `676-01-calls` | from `repo`: a `Home` group after it, `System O…` its row, highlighted, System One calls in front (off, as every run's copy has it) |
| REQ-002 | `676-02-project` | the terminal clicked: `repo — bash` in front, System One calls still under Home |
| REQ-003 | `676-03-again` | the action again from `repo`: Home shown with its one tab, one Home group |

The exploratory run before the gate showed the same; nothing changed after it. Focus: headless
sway; Hyprland's one Marley window (Chad's) before and after, no rule added.

## Phase 4 — Complete (2026-10-07)
- **Documented:** `CHANGELOG.md` (Changed: Home takes what belongs to no project); the in-app
  guide's empty-space item (before the gate); `marley_workbench.md` (a new section). The intake
  note's decision 4 is now this.
- **Knowledge:** `AD-claude-676-home-takes-the-screens-that-belong-to-no-project-001`.
- **Brain:** `decisions/home-takes-the-screens-that-belong-to-no-project`.
- **Ticket:** closed; the pair archived.
