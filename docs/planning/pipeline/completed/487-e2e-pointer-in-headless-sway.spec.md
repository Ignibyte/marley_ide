---
pipeline_id: 4ee747f0-2997-4561-b561-b2743e271744
ticket: docs/planning/tickets/open/TICKET-487-e2e-pointer-in-headless-sway.md
status: Phase 4 — Complete PASS
title: "Scenarios that click run Marley in a headless sway"
type: chore
slice: prong 3 tooling (three-prong-plan.md D18)
references: [docs/marley/three-prong-plan.md, CONSTITUTION.md §7, script/e2e.sh, docs/planning/pipeline/completed/483-e2e-visualization-tests.spec.md]
---

## Title
A second backend for the e2e harness: a scenario that sets `COMPOSITOR=sway` runs the debug
Marley inside a headless sway of its own, with a virtual pointer and keyboard, so it can
click, drag, scroll and type without anything reaching Chad's desktop. The browser tab
(#488 on) is driven by the mouse; its scenarios need this.

## Scope
### In
- `script/e2e.sh` chooses its backend from the scenario's `COMPOSITOR` (`hyprland`, the
  default, or `sway`). The sway backend:
  - starts `sway` with `WLR_BACKENDS=headless` and `WLR_LIBINPUT_NO_DEVICES=1` on a config the
    harness writes: one `HEADLESS-1` output at `SIZE` (default `1600x1000`), no borders, no
    bar, no key bindings, and an `exec` that reports the compositor's Wayland socket;
  - gives the seat a pointer and a keyboard before Marley starts: the pointer helper (below)
    and a keyboard `wtype -s` holds open;
  - launches Marley inside it (`WAYLAND_DISPLAY` of the sway), waits for its window through
    `swaymsg -t get_tree`, runs `steps`;
  - adds no rule to Hyprland and never runs `hyprctl reload`.
- `script/e2e/seat-pointer.c`, a small Wayland client over the published
  `wlr-virtual-pointer-unstable-v1` protocol (the XML vendored beside it, MIT, from
  wlroots/wlr-protocols): it creates a virtual pointer and reads commands on stdin, `move X Y`,
  `down BUTTON`, `up BUTTON`, `scroll STEPS`, answering `ok` after each. The harness builds it
  with `wayland-scanner` and `cc` into the shots directory the first time (and again when its
  source changes); no binary enters the repository.
- Helpers under sway, with the Hyprland helpers' signatures where both exist:
  - `click X Y [BUTTON]`, `pointer_to X Y`, `pointer_down [BUTTON]`, `pointer_up [BUTTON]` and
    `scroll STEPS` (positive is down; at the pointer), in window pixels (the window fills the
    output);
  - `press MODS KEY` and `type_text TEXT` through `wtype`;
  - `shot NAME` through `grim` on the headless output.
- A scenario may define `teardown`; the harness runs it on every exit, before Marley stops,
  for anything the scenario started outside Marley (the browser's Chromium unit from #488).
- Cleanup under sway: `teardown`, Marley, the pointer helper, the key holder, then
  `swaymsg exit`. The closing report says the user's session was not used.
- Before launching, each backend checks the tools it needs and names any that are missing
  (sway: `sway`, `swaymsg`, `wtype`, `grim`, `cc`, `wayland-scanner`, `pkg-config`).
- CONSTITUTION §7 amended in its own commit: a scenario that clicks runs in the headless
  sway; the Hyprland backend stays keys-only and remains the default.
- The proof scenario `script/e2e/487-sway-pointer.sh`.

### Out (explicitly deferred)
- Moving the existing scenarios to sway: they keep the Hyprland backend they were proven on.
- Touch, tablet, more than one output, pointer targets found by element instead of pixel.
- Installing sway: the dev box has it since 2026-09-24 (`pacman -S sway`); the harness only
  reports it missing.

## Reference (§20)
N/A — Marley-specific test tooling. Warp and Zed have no counterpart: Zed tests its UI in
process on gpui's test platform, which #483 retired for Marley in favor of scripted runs of the
real app.

### Prior art
- **Published material.** sway-input(5) documents `seat <seat> cursor move|set|press|release`
  as deprecated in favor of the virtual-pointer protocol; buttons 4 to 7 map to wheel axes.
  The wlr-virtual-pointer-unstable-v1 protocol (wlroots/wlr-protocols, MIT) creates a pointer
  device on a seat and sends absolute motion, buttons, and wheel axis events with a source and
  discrete steps. `wtype` types text and chords through the virtual-keyboard protocol and
  sleeps with `-s`; `grim` captures a wlroots output through screencopy.
- **Observed (the feasibility check, 2026-09-24).** The debug Marley ran in a headless sway at
  1600×1000 with no window on Hyprland. A headless seat started with
  `WLR_LIBINPUT_NO_DEVICES=1` has `capabilities: 0`, so neither `swaymsg seat cursor` nor a
  one-shot `wtype` reached Marley. Once a helper held a virtual pointer and `wtype -s` held a
  keyboard, a click switched the onboarding theme to Dark, three wheel detents scrolled the
  page and Ctrl+Shift+P opened the command palette; gpui took the devices although they came
  after it started.
- **Code we already ship.** `script/e2e.sh`'s Hyprland backend (`press`, `type_text`, `shot`,
  the focus report) is the model for the helpers. Nothing in the tree injects pointer events:
  gpui's `Window::dispatch_event` is in-process and would skip the platform layer a scenario
  exists to exercise. `wlrctl` (AUR) sends one-shot pointer events but cannot hold the device
  between commands, the same race a one-shot `wtype` loses on a seat with no devices.

## UI proof
UI-AFFECTING (the harness drives Marley's UI). `script/e2e/487-sway-pointer.sh` with
`COMPOSITOR=sway`: #481's stand-in agent (a HOME whose `.bashrc` defines `stand_in`, `exec -a
claude` printing `claude got: …` for each line). The terminal prints 200 numbered lines, the
stand-in starts, and the scenario clicks the agent bar's Rich Input button, types into the
editor and sends, then scrolls the terminal back with the wheel. Shots: `487-01-marley`,
`487-02-rich-input-clicked`, `487-03-sent`, `487-04-scrolled-back`.

## Locked-In Decisions
- D1 — A headless sway, not a nested compositor in a window on Hyprland: nothing appears on
  Chad's desktop and no Hyprland rule is added or reloaded.
- D2 — A virtual pointer the harness holds, through the published protocol, rather than
  `swaymsg seat cursor`: the seat needs a pointer device either way, and the protocol is the
  supported path. The helper answers each command, so steps cannot race it.
- D3 — Keys under sway go through `wtype`, with one keyboard held open for the run so the seat
  has a keyboard before Marley starts.
- D4 — The helper is built at run time into the shots directory; only its C source and the
  protocol XML live in the repository.
- D5 — The backend is the scenario's choice, and Hyprland stays the default, so every
  scenario proven so far runs as it was proven.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a scenario sets `COMPOSITOR=sway`, the harness shall run Marley in a headless sway of its own, with no Marley window on Chad's Hyprland and no Hyprland rule added or reloaded. | Shot `487-01-marley`; the run's report (Hyprland's client list and config untouched) |
| REQ-002 | WHEN a scenario calls `click X Y`, the click shall land at that point of Marley's window. | Shot `487-02-rich-input-clicked`: the Rich Input editor is open, which only the button's click opens there |
| REQ-003 | WHEN a scenario types with `type_text` and `press` under sway, the keys shall reach the focused element. | Shot `487-03-sent`: `claude got: typed under sway` |
| REQ-004 | WHEN a scenario calls `scroll N` over an element, the wheel event shall reach it. | Shot `487-04-scrolled-back`: earlier numbered lines on screen |
| REQ-005 | WHEN a run ends, normally or on an error, the harness shall run the scenario's `teardown` and stop Marley, the pointer helper, the key holder and sway, leaving no sway socket behind. | The harness's closing check, printed in the run |
| REQ-006 | WHEN a tool the chosen backend needs is missing, the harness shall stop before starting anything and name the tool. | Negative smoke: a PATH without `wtype` |
| REQ-007 | The Hyprland backend shall behave as before. | `just shot` on Hyprland; gate:10 (shellcheck) green |

## Phase Plan
- **P1 Plan** — this spec, and the design and E2E plan in the notes.
- **P2 Code** — the harness's sway backend, the helper and its protocol file, the helpers, the
  `teardown` hook, the scenario; the §7 amendment and its record; shellcheck clean.
- **P3 Test** — run `487-sway-pointer.sh` and read every shot; `just shot` on Hyprland; the
  negative smoke; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, the harness's docs (CONSTITUTION §7, `docs/marley/README.md`),
  ledger capture (the lesson on seats without devices), close, archive, commit (the §7
  amendment in its own commit).
