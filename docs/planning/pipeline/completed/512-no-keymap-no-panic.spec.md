---
pipeline_id: 03c08d22-0f5f-46bc-a8b4-843117a265f7
ticket: docs/planning/tickets/closed/TICKET-512-no-keymap-no-panic.md
status: Phase 4 — Complete PASS
title: "A Wayland seat without a keymap no longer kills Marley"
type: bug
slice: platform (gpui's Wayland client), found in #502's Test
references: [docs/planning/pipeline/active/502-installed-release-build.spec.md]
---

## Title
gpui's Wayland keyboard handler tolerates a compositor that sends no usable keymap: `modifiers`
and `key` events before a keymap are skipped, and a keymap that fails to compile is logged and
ignored, so Marley keeps running and takes keys once a keyboard with a keymap arrives.

## Scope
### In
- `crates/gpui_linux/src/linux/wayland/client.rs`, the `wl_keyboard` dispatch: the three
  `keymap_state` unwraps (the `Modifiers` arm twice, the `Key` arm once) become early returns
  when no keymap has arrived; the `Keymap` arm's `.expect("Failed to create keymap")` becomes a
  logged error that leaves the earlier state as it was. Each hunk carries a `// Marley:` comment;
  the path gets its row in `docs/marley/zed-touchpoints.md` first.

### Out (explicitly deferred)
- The X11 client, which reads its keymap from the X server at start and has no such event.
- Where a `dev`-channel panic goes: to stderr only, since Zed installs no crash handler on that
  channel. Whether the installed build should keep stderr somewhere is its own question.
- Offering the fix upstream (a PR to zed-industries/zed): Chad's call.

## Reference (§20)
Upstream Zed, `gpui_linux`'s Wayland client, the same code at upstream `main` on 2026-09-25
(fetched through the GitHub API: lines 1857 to 1958 match the fork's). The behavior kept is
upstream's in every case where a keymap exists. The Wayland protocol (`wl_keyboard.keymap` with
the formats `no_keymap` and `xkb_v1`; `modifiers` and `enter` may follow at any time) is what the
fix matches. Warp: N/A.

### Prior art
- **Code we already ship.** The handler's own `Keymap` arm already refuses a non-`xkb_v1` format
  with a log line and a return, and `handle_keyboard_layout_change` (line 680) already reads the
  state through `if let Some`. The fix gives the other arms the same shape. xkbcommon's
  `Keymap::new_from_fd` returns an `Option`; the `expect` is gpui's.
- **Published material.** wayland.xml: `wl_keyboard.keymap_format.no_keymap` ("no keymap;
  client must understand how to interpret the raw keycode"); wlroots sends it when a seat has no
  keyboard.
- **Observed.** #502's scenario, run 1: after `quit_marley` the run's last `wtype` had exited,
  the headless sway's seat had no keyboard, and the Marley that `uwsm-app -- gtk-launch` started
  panicked at line 1921 (`Option::unwrap()` on `None`, in `<WlKeyboard as Dispatch>::event`).
  Run 2, with the seat's keyboard held first, started and stayed up. Reproduced at Plan with
  this ticket's scenario on the unfixed debug build: the same panic, before the window mapped.

## UI proof
UI-AFFECTING (a crash at start). `script/e2e/512-no-keymap-no-panic.sh` (`compositor sway`): the
debug build, whose first launch is made with no keyboard on the seat (the runner's held keyboard
released first). Steps: the run's Marley quits (the `wtype` keyboards that typed it exit), and a second
Marley starts on the same profile: the window maps and the process lives ten seconds
(`512-01-no-keyboard`); a keyboard types Ctrl+Shift+P and the palette opens
(`512-02-keys-arrive`). The run log shows no panic. Run first on the unfixed build, where it
failed at the first step, then on the fix; #500's scenario runs again for the keyboard as it
was.

## Locked-In Decisions
- D1 — Skip, do not guess: with no keymap there is no way to turn a keycode into a keystroke, so
  the events are dropped until one arrives, rather than interpreted with a made-up keymap.
- D2 — Keep the earlier state when a new keymap fails to compile: the user's keys keep working
  under the last good keymap.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley starts on a seat that sends no usable keymap, Marley shall open its window and keep running. | Shot `512-01-no-keyboard`; the run log (no panic) |
| REQ-002 | WHEN a keyboard with a keymap arrives later, Marley shall take its keys. | Shot `512-02-keys-arrive` |
| REQ-003 | WHERE a keymap exists from the start, the keyboard shall behave as upstream's. | Every other scenario of the sprint, unchanged |

## Phase Plan
- **P1 Plan** — promote, recall, the design.
- **P2 Code** — the ledger row, the three guards and the logged keymap failure; fmt and clippy
  on `gpui_linux`.
- **P3 Test** — the scenario on the unfixed and the fixed build; the gate.
- **P4 Complete** — docs, knowledge (the F- block from #502's Test), close, archive, commit.
