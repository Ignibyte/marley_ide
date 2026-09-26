# A Wayland seat without a keymap no longer kills Marley — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-512-no-keymap-no-panic.md
- **Pipeline spec:** 512-no-keymap-no-panic.spec.md

## Phase 1 — Plan
- **Request:** found in #502's Test on 2026-09-25 (see its notes, Phase 3).
- **Classification / tier:** bug, a Zed crate (`gpui_linux`); a small additive hunk.
- **Discovery:** `crates/gpui_linux/src/linux/wayland/client.rs` lines 1850 to 1965 (the
  `wl_keyboard` dispatch); the backtrace from #502's first run.

- **Recall (§18.3):** F-claude-502-a-seat-with-no-keymap-panics-gpuis-keyboard-handler-001 and
  L-claude-502-omarchy-starts-an-entry-through-uwsm-app-and-gtk-launch-001 (the finding and the
  seat state); nothing else in the ledgers on `wl_keyboard` or xkb (the gpui-era keymap entries
  are about macOS menus and Marley's old chord table). No row for `gpui_linux` in the ledger
  yet. Brain: consultation a2a4d9c85e154fc5b835ef805a107c16, no decision on this seam.
- **Reproduced before the fix** (Plan, 2026-09-25): `script/e2e/512-no-keymap-no-panic.sh` on
  the debug build at `HEAD` (built for #501): the second Marley panicked at
  `crates/gpui_linux/src/linux/wayland/client.rs:1921:64` before its window mapped ("no window
  from the second Marley within 90 seconds").
- **Checklist (no task tool in this session):** pick · pre-flight · recall · promote ·
  reproduce · prior art · spec · design — done.

### Design
- **Approach.** Three hunks in the `wl_keyboard` dispatch of
  `crates/gpui_linux/src/linux/wayland/client.rs`, each with a `// Marley:` comment:
  - `Keymap` arm: the `.expect("Failed to create keymap")` goes; a `None` keymap (a compile
    failure, or an I/O error `log_err` already logged) logs an error and returns, leaving the
    earlier `keymap_state` and `compose_state` as they were (D2).
  - `Modifiers` arm: `let Some(keymap_state) = state.keymap_state.as_mut() else { return; }`
    before the layout read. The second `as_mut().unwrap()` in the arm stays: after the guard it
    reads the same state with nothing between that could clear it, and leaving it keeps the hunk
    to one insertion (§14 upstream discipline).
  - `Key` arm: the same `let Some(..) = state.keymap_state.as_ref() else { return; }` after the
    focused-window check, so a key with no keymap is dropped (D1). The serial tracker's update
    above it stays, as upstream orders it.
- **Ledger row first:** `crates/gpui_linux/src/linux/wayland/client.rs` in
  `docs/marley/zed-touchpoints.md`.
- **File manifest:** `crates/gpui_linux/src/linux/wayland/client.rs` (Zed crate, three hunks);
  `docs/marley/zed-touchpoints.md` (the row); `script/e2e/512-no-keymap-no-panic.sh` (written at
  Plan to reproduce).

### E2E plan
| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | the run's Marley quits (its `wtype` keyboards exit), then a second Marley starts on the same profile; its window maps and it lives ten seconds | `512-01-no-keyboard`; teardown prints the second Marley's stderr (no `panicked`) |
| REQ-002 | a keyboard types Ctrl+Shift+P | `512-02-keys-arrive` (the palette) |
| REQ-003 | a keyboard-heavy scenario from the sprint before, unchanged: `script/e2e/500-browser-from-the-rail.sh` (typing an address) | its shots, as at #500 |

### Risks
- Dropping `modifiers` before a keymap means the first modifier state after a keymap arrives
  comes with the next `modifiers` event; wlroots sends one when it sends the keymap, so no stale
  state is expected.

## Phase 2 — Code
- **Built.** The ledger row for `crates/gpui_linux/src/linux/wayland/client.rs` first, then three
  hunks in the `wl_keyboard` dispatch, each with a `// Marley:` comment: the `Keymap` arm's
  `.expect("Failed to create keymap")` became a `let Some(keymap) = keymap else { log::error!(…);
  return; }` that leaves the earlier state; the `Modifiers` arm and the `Key` arm return when
  `keymap_state` is `None`. The second `as_mut().unwrap()` in the `Modifiers` arm stays, as
  planned: after the guard nothing between the two reads can clear the state.
- **Checks.** `cargo fmt -p gpui_linux -- --check` clean; `just clippy gpui_linux` (all targets,
  `-D warnings`) exit 0; `just build` rebuilt the debug `marley`.
- **Review.** Correctness: with no keymap a key cannot become a keystroke, so dropping it is the
  only honest choice (D1); the serial tracker still records key presses, as upstream orders it.
  Re-entrancy: none, the handler holds the client state's borrow as before and returns with it.
  Provenance: Zed code edited in place, no Warp. Upstream discipline: three insertions and one
  removed `.expect`, no reformat.

## Phase 3 — Test
- **Scenario:** `script/e2e/512-no-keymap-no-panic.sh` (`compositor sway`), written at Plan.
  - **Before the fix** (Plan, the debug build at `HEAD`): red as expected. The second Marley
    panicked at `client.rs:1921:64` before its window mapped ("no window from the second Marley
    within 90 seconds"; its stderr held the panic).
  - **After the fix** (the rebuilt debug build): green. "the second Marley (…) still runs after
    ten seconds"; its stderr holds no panic.
- **Shots, read:**
  - `512-01-no-keyboard` — the second Marley, started on the seat whose keyboard had gone, in the
    Marley layout on the scratch repository (branch `keymap`), alive after ten seconds. REQ-001.
  - `512-02-keys-arrive` — after a keyboard typed Ctrl+Shift+P, the command palette open (its
    history led by the run's `zed: quit`). REQ-002.
- **REQ-003, the keyboard as it was:** `script/e2e/500-browser-from-the-rail.sh` again on the
  fixed build, green: `500-02-opened` shows the address typed into a new Browser tab loaded ("Opened
  from the rail's +").
- **Focus:** sway, "0 Marley windows before the run, 0 after" on Hyprland.
- **Gate:** `just gate-diff`: `GATE GREEN [diff]`, every gate PASS with `gpui_linux` in scope
  (log `gate-512.log` in the scratchpad); the receipt matches the tree.

## Phase 4 — Complete
- **Docs:** `CHANGELOG.md` (Fixed: Marley no longer dies at start on a seat without a keyboard);
  the touchpoint row for `crates/gpui_linux/src/linux/wayland/client.rs` describes the three hunks
  as shipped. No Marley crate changed.
- **Knowledge:** PR-claude-an-event-handler-never-unwraps-what-another-event-sets-001;
  L-claude-512-show-the-bug-on-the-unfixed-build-first-001. The failure itself is
  F-claude-502-a-seat-with-no-keymap-panics-gpuis-keyboard-handler-001 (found in #502's Test).
- **Brain:** consultation a2a4d9c85e154fc5b835ef805a107c16 closed with
  `decisions/marley-patches-gpuis-wayland-keyboard-handler-to-tolerate-a-missing-keymap`.
- **Also in this commit:** #502's record corrected (Chad's debug Marley quit normally at 20:11;
  `telemetry.log` ends with `App Closed`), in `lessons.md` and #502's completed notes and spec.
- **Ticket:** closed; the pipeline pair archived.
