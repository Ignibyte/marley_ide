---
pipeline_id: 9bf40bf4-4846-49e0-ad11-ad896d4f8fae
ticket: docs/planning/tickets/open/TICKET-545-activation-token-hand-off.md
status: Phase 4 — Complete PASS
title: "A second launch hands its launcher's activation token to the running Marley"
type: feature
slice: the Marley app, startup (#513's follow-up)
references: [docs/planning/pipeline/completed/513-one-marley-per-data-dir.spec.md]
---

## Title
Since #513 a second launch hands its paths to the running Marley, which asks the compositor to
bring a window forward with a token gpui requests itself, from a window that has no focus. A
compositor that checks tokens (sway, KWin, Mutter) declines it. The launcher's own token, which a
desktop launcher gives the second launch in `XDG_ACTIVATION_TOKEN`, now travels with the hand-off,
and the running Marley's next activation uses it.

## Scope
### In
- `marley_workbench::single_instance`: `keep_activation_token()`, called at the top of `main`
  before gpui's Wayland client takes the variable out of the environment, and `send` sends
  `zed://marley-activation-token/<token>` ahead of the paths when one was kept;
  `activation_token_in(url)` reads one back.
- `crates/zed/src/zed/open_listener.rs` (Linux and FreeBSD listener): a token datagram goes to
  `gpui::set_next_activation_token` instead of opening, so it is set before the next datagram's
  request activates a window.
- `crates/gpui/src/platform.rs`: `set_next_activation_token` and `take_next_activation_token`,
  one process-wide slot.
- `crates/gpui_linux/src/linux/wayland/window.rs`: `activate` uses a token in the slot, once,
  instead of requesting one.
- `crates/zed/src/main.rs`: the one call.

### Out (explicitly deferred)
- X11's startup-notification id, and macOS.
- A compositor's own policy: sway's default `focus_on_window_activation urgent` marks the window
  urgent on a valid token; `smart` or `focus` brings it forward.

## Reference (§20)
Upstream Zed: `gpui_linux`'s Wayland client already takes `XDG_ACTIVATION_TOKEN` at startup and
activates its first window with it (`take_startup_activation_token_from_environment`,
`consume_startup_activation_token`); this extends the same token to a hand-off. The protocol is
xdg-activation-v1: a launcher passes the token it requested to the program it starts, which calls
`activate(token, surface)`.

### Prior art
- **Published material.** xdg-activation-v1 (wayland-protocols): tokens are single-use and tied to
  the requesting client's focus and input serial. sway(5): `focus_on_window_activation
  smart|urgent|focus|none`, default urgent.
- **Code we already ship.** `crates/gpui_linux/src/linux/wayland/client.rs` 115, 195 to 204, 512 to
  519 (the startup token), `window.rs` 1789 (`activate` requests its own token with the last
  pointer press); `crates/zed/src/zed/open_listener.rs` 409 to 430 (the datagram listener);
  `marley_workbench::single_instance` (#513's hand-off). GTK4's launch context requests a token with
  its latest serial (`Gdk.AppLaunchContext.get_startup_notify_id`), which the check uses as its
  launcher.

## UI proof
`script/e2e/545-activation-token-hand-off.sh` (sway, `focus_on_window_activation smart`): a GTK4
stand-in launcher takes the focus and requests a token. A hand-off without a token leaves the
focus on the launcher (`545-01-no-token`); a hand-off with the launcher's token brings Marley
forward (`545-02-token`), read from sway's tree as well as the shot.

## Locked-In Decisions
- D1 — The token travels as a datagram of its own, ahead of the paths, on the socket #513 already
  uses; the listener sets it before the next request is read, so no open request changes shape.
- D2 — The slot is used once, by the next Wayland activation, and a hand-off without a token
  leaves gpui asking for its own, as before.
- D3 — The token is read at the top of `main`, before the platform exists, and only read: the
  launch's own first window still takes it when it is the one that runs.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a second launch with `XDG_ACTIVATION_TOKEN` hands off to a running Marley, the running Marley shall activate its window with that token, and a compositor that checks tokens shall bring it forward. | `545-02-token`; sway's focused window |
| REQ-002 | WHEN a second launch has no token, the hand-off shall behave as before. | `545-01-no-token`: the focus stays on the launcher |
| REQ-003 | The token datagram shall not open anything or reach Zed's URL parsing. | The run's Marley log: no unhandled URL |

## Phase Plan
- **P1 Plan** — this spec, the design in the notes.
- **P2 Code** — the ledger rows first; the five hunks; clippy; `script/gates.sh --diff`.
- **P3 Test** — the scenario, both shots read, sway's tree.
- **P4 Complete** — CHANGELOG, docs, ledger, close, archive, commit.
