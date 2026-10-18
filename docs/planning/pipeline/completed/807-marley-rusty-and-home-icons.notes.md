# Marley's, Rusty's and Home's own icons — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-807-marley-rusty-and-home-icons.md
- **Pipeline spec:** 807-marley-rusty-and-home-icons.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-17: "Also i have agent-icons folder. one for home, rusty, and marley.
  lets add those as well"; then "ok lets build 3 and 4 then reinstall".
- **Recall:** Home's icon is `IconName::ListTree` in `HomePage::tab_icon` and the status bar's
  `marley-status-home`; the Marley agent's `IconName::Sparkle` in `marley-title-marley`; Rusty's
  `RUSTY_ICON` (`IconName::Blocks`); threads fall back to the sparkle in `agents::thread_icon`
  (which the New Agent picker and the rail's thread rows use) and draw `ZedAssistant` in
  `ThreadTab::tab_icon`. `IconName::path` is `icons/<snake_case>.svg`; crates/icons'
  `test_all_icons_exist` wants a file per name.
- **Promoted 2026-10-17** (#806 parked).

### Design
- **Zed (additive, ledger rows first):** `crates/icons/src/icons.rs`, three names in order;
  `assets/icons/marley_cat.svg`, `marley_home.svg`, `rusty_butler.svg`, Chad's SVGs as they are.
- **Marley:** `assistant::assistant_icon(agent)` (Marley's or Rusty's entry → its icon);
  `agents::thread_icon` and `ThreadTab::tab_icon` use it first; `status_buttons` and
  `HomePage::tab_icon` name the new icons; `RUSTY_ICON` becomes `RustyButler`.
- **The folder:** `agent-icons/` stays out of the repository (the SVGs are copied; the PNGs, the
  previews and the zip are Chad's).

## Phase 2 — Code
- **Built:** as the design. Clippy clean on `icons` and `marley_workbench`.

## Phase 3 — Test
- **Scenario:** `script/e2e/807-marley-rusty-and-home-icons.sh` (`compositor sway`, #778's set-up).
  One run; every shot read:
  - `807-01-title-bar` (REQ-001 to REQ-003): the title bar's `?`, then the butler (Rusty), then the
    cat (Marley); the status bar's Home is the house.
  - `807-02-home` (REQ-001): `rusty: open home`: the Home tab and Home's rail row draw the house.
  - `807-03-picker` (REQ-002, REQ-003): the New Agent picker: Marley with the cat, Rusty with the
    butler; Zed Agent, Claude Agent and the CLIs keep theirs.
  - `807-04-thread` (REQ-002): `marley: talk to marley`, "hello": the thread's tab and its rail row
    ("Marley · idle") draw the cat. A Rusty thread takes the same path (`assistant_icon`).
  - `807-05-light` (REQ-004): One Light: every new icon drawn dark on the light theme.
- **Gate (`807-gate-1.log`):** **GATE GREEN [diff]**.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` ("Marley's own icons"); `docs/marley/zed-touchpoints.md` (the
  `crates/icons/src/icons.rs` row and the three `assets/icons/` rows, written before the change).
  The guide names no icon, so it is unchanged.
- **Knowledge:** none new.
- **Brain:** no decision recorded.
- Also filed in this commit: #810 (the Take Over bar can be closed), #811 (trusted outside
  clients), #812 (a block header takes its pane's click).
