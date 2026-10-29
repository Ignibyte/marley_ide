# Home stays first in the Default group after a relaunch restores its tabs — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-794-home-missing-after-a-restored-tab.md
- **Pipeline spec:** 794-home-after-a-restored-tab.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-29: "lets do 795, 794, 947 then clean up all your builds etc and
  reinstall".
- **Classification / tier:** bug, S. One Zed hunk (`workspace`), one Marley file.
- **Recall (§18.3):**
  - F-claude-700: Home takes Zed's start workspace with its tabs; the start workspace is Home's.
  - #703 (in `ensure`): Home goes in before the active tab without taking it.
  - #912's relaunch (`912-06-next-start`) showed Home after a restart: there no saved tab was in
    Default, so the restored center held nothing to replace Home with.
  - The brain: consultation 6eb8ab9b297946ce80e687f704552b4c; nothing on this seam.
- **Reproduced on Linux:** `script/e2e/794-home-after-a-restored-tab.sh` at `57c5d4e5d4`. A terminal
  opened in Default beside Home, a quit, a launch with no folder: Default came back with its
  terminal ("1 row") and no Home, in the tabs or the rail. A launch that names a folder restores no
  session, so the scenario launches with none.
- **Cause:** `Workspace::load_workspace` (`crates/workspace/src/workspace.rs`) awaits the saved
  center's deserialization, then `remove_panes(self.center.root)` and installs the restored group.
  Home went into the first pane when the rail adopted the group (`fill_shown`); the swap dropped
  that pane, and no event Marley hears runs `ensure` after it.

### Design
- **Zed hunk:** `workspace::Event::CenterRestored`, a unit variant after `WorktreeCreationChanged`,
  with a `// Marley:` comment; `load_workspace` emits it inside the `if let Some((center_group, …))`
  arm, after `set_active_pane`, so it fires only when a center was swapped in. gpui delivers it
  after that update, when the new center is in place.
- **Marley:** in `home_page::init`'s `Workspace` subscription, the `ItemRemoved` arm gains a
  `CenterRestored` arm that calls `fill_restored(workspace, window, cx)`: in the Home workspace,
  `ensure`. `ensure` already returns when Home is there and puts it first without taking the active
  tab. The match on another crate's enum lists the two variants and lets `_` take the rest.
- **File manifest:**
  - `crates/workspace/src/workspace.rs` (Zed crate): the variant and the emit.
  - `crates/marley_workbench/src/home_page.rs` (Marley crate): the arm and `fill_restored`.
  - `docs/marley/zed-touchpoints.md`: the row for `workspace.rs`'s event, before the edit.
  - `script/e2e/794-home-after-a-restored-tab.sh` (Marley-owned).

### Visual check plan
- REQ-001: Default shown from the rail; `workspace: new terminal`; `echo kept`; `794-01-before`;
  `quit_marley`; `launch_marley_with` (no folder); `794-02-restored`: tabs Home, then the
  terminal; the rail's Default lists Home.
- REQ-002: `pane: close active item` with Home chosen (activate it first), then `pane: activate
  next item`: `794-03-closed`, Home gone and not back.
- REQ-003: the review of the diff.

### Risks
- A restored center that the user then empties still goes through `fill_emptied`, unchanged.
- Another Marley observer may later want `CenterRestored`; the event carries no data, so adding a
  listener changes nothing for Home.

## Phase 2 — Code
- **Built:**
  - `crates/workspace/src/workspace.rs`: `Event::CenterRestored` after `WorktreeCreationChanged`,
    emitted in `load_workspace`'s swap arm after `set_active_pane`, each with a `// Marley:` comment.
  - `crates/marley_workbench/src/home_page.rs`: the `Workspace` subscription matches `ItemRemoved`
    and `CenterRestored`, `_` for the rest; `fill_restored` calls `ensure` in the Home workspace.
  - `docs/marley/zed-touchpoints.md`: `workspace.rs`'s row extended before the edit.
- **Deviations:** none.
- **Review:** the emit sits inside the `if let Some((center_group, active_pane))` arm, so a workspace
  with no saved center emits nothing (REQ-003). gpui delivers the event after `update_in` returns,
  when the restored center is in place, so `ensure` reads the new active pane and nothing is
  updated twice. A workspace the rail has not adopted yet is not the Home workspace; `fill_shown`
  still runs when the rail adopts it. Style: the `match` on Zed's enum lists two variants and a `_`
  arm; the new function has its doc.
- **Gate:** `just gate-diff` green, 17 passed; run again after the scenario gained REQ-002's step.

## Phase 3 — Test
- **Scenario:** `script/e2e/794-home-after-a-restored-tab.sh` under sway, after `just build`.
- **Before the fix** (`57c5d4e5d4`): `794-02-restored` showed Default with its terminal alone, "1
  row" in the rail, no Home.
- **Shots:**
  - `794-01-before`: Default shown from the rail; tabs Home and the Default terminal (named for the home folder) with `echo kept`;
    the rail's Default "2 rows", Home and the terminal.
  - `794-02-restored` (REQ-001): after a quit and a launch with no folder, the same group: tabs
    Home then the terminal, the terminal still the chosen tab with its restored `echo kept` block
    and "Marley started again"; the rail's Default "2 rows" with Home first.
  - `794-03-closed` (REQ-002): Home chosen and closed by hand: one tab, the terminal; the rail's
    Default "1 row"; Home not put back.
- REQ-003: read in the diff; the emit is inside the swap arm.
- Not seen: the Mac's Onboarding case itself (Marley's Welcome replaced Zed's Onboarding in #912);
  a terminal is the restored tab here, and the restore path is the same for every saved item.
- Nothing pre-existing failed.

## Phase 4 — Complete
- Documented: `CHANGELOG.md` (Fixed), `docs/marley_architecture/marley_workbench.md` (Home's own
  page), `docs/marley/zed-touchpoints.md` (`workspace.rs`'s row, written before the edit and
  matching what shipped).
- Knowledge: F-claude-794-zeds-restore-dropped-the-pane-home-was-in-001,
  L-claude-794-a-scenario-restores-a-session-only-when-the-launch-names-no-folder-001.
- Brain: consultation 6eb8ab9b297946ce80e687f704552b4c closed with no decision (D1 is in the spec
  and the ledger row).
- Ticket closed; pipeline archived.
