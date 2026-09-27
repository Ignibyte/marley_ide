# Scenarios that click run Marley in a headless sway — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-487-e2e-pointer-in-headless-sway.md
- **Pipeline spec:** 487-e2e-pointer-in-headless-sway.spec.md

## Phase 1 — Plan (drafted by /spec, 2026-09-24)
- **Request:** wave 1 of prong 3 (Chad: "lets make this happen. plan, spec tickets, then
  execute"; "do your best for decisions"). The browser tab is mouse-driven; the harness could
  not click.
- **Classification:** chore, tooling. No Rust; the gate's shellcheck covers the scripts.
- **Recall (§18.3):**
  - L-claude-483-send-keys-to-one-hyprland-window-by-address-001: the Hyprland backend's key
    path; Hyprland has only `send_key_state`, `send_shortcut` and `pass`, none for a pointer.
  - L-claude-483-a-scenario-brings-its-own-shell-001 and L-claude-481-…: the stand-in agent
    this scenario reuses.
  - CONSTITUTION §7: "No mouse: a click would move the user's pointer" — the rule this
    ticket amends, for the sway backend only.
- **Discovery (this session):** the feasibility check in the scratchpad (headless sway,
  Marley at 1600×1000, a C helper over the virtual-pointer protocol, `wtype -s` holding the
  keyboard): click, wheel and chords all reached Marley. The helper prototype is 110 lines.
- **Checklist (no TaskCreate in this harness):** select ✓, mint ✓, prior art ✓, spec ✓.

## Phase 1 — Plan (promoted 2026-09-24)
- **Pre-flight:** cargo idle, gate and e2e present, hooks wired, no active pipeline, README
  marker present.
- **Brain:** consultation f2cc655642e44af5b384d1252427ae5e (nothing on the seam; only
  unrelated follow-ups due).
- **Seams re-verified:** `script/e2e.sh` as read at promotion: the Hyprland-only globals
  (`HYPRLAND_INSTANCE_SIGNATURE`, `marley_window` by class), the scenario sourced after the
  profile copy, `cleanup` defined after sourcing and trapped on EXIT, `write_terminal_env`
  after `setup`, the focus report. `lib-hook-helpers.sh` owns `script/e2e.sh|script/e2e/*`, so
  the new files need no ledger row; the gate's shellcheck covers `script/e2e.sh` and
  `script/e2e/*.sh`. CONSTITUTION's amending rule: a commit that touches only the constitution
  (and any hook or gate that enforces the rule).

### Design
- **Approach.** The harness keeps one entry point and grows a backend switch. The scenario is
  sourced first (it only defines functions and variables), so `COMPOSITOR` and `SIZE` are known
  before anything starts; each backend then checks its tools, prepares, launches and cleans
  up. The step helpers keep their names and signatures and branch on the backend; the pointer
  helpers exist only under sway and refuse under Hyprland with the reason.
  - **Sway.** A temp directory beside the profile holds the generated config (one
    `HEADLESS-1` output at `SIZE`, no borders, an `exec` that writes the compositor's
    `WAYLAND_DISPLAY` and `SWAYSOCK` to a file). Sway starts with `WLR_BACKENDS=headless`,
    `WLR_LIBINPUT_NO_DEVICES=1`, and without the user's `WAYLAND_DISPLAY`, `DISPLAY`,
    `HYPRLAND_INSTANCE_SIGNATURE` and `SWAYSOCK`, so neither it nor Marley's terminals can
    reach the user's session. The pointer helper runs as a bash coprocess (commands in, `ok`
    out); `wtype -s 86400000` holds the keyboard. Marley starts with the sway's display; its
    window is found in `swaymsg -t get_tree` by `app_id`.
  - **Keys under sway.** `press` maps Hyprland's modifier names (`CTRL`, `SHIFT`, `ALT`,
    `SUPER`) to wtype's (`ctrl`, `shift`, `alt`, `logo`) and sends the xkb key name with `-k`;
    `type_text` hands the text to `wtype` whole (any character, not only the US tables).
  - **The helper's build.** `seat-pointer.c` and the protocol XML are hashed; the binary lives
    in `$SHOT_DIR/.seat-pointer-<hash>/` and is built with `wayland-scanner` (client header and
    private code) and `cc` against `pkg-config wayland-client` when missing.
  - **Teardown and cleanup.** `cleanup` runs `teardown` (if the scenario defined one) for both
    backends, then the backend's stop: under sway Marley (TERM), the helper (its stdin
    closed), the key holder, `swaymsg exit`, and a closing check that no process of the run is
    left and the IPC socket is gone.
- **File manifest** (all Marley-owned; no Zed path, no ledger row):
  - `script/e2e.sh` — the backend switch, the sway backend, the pointer helpers, `teardown`.
  - `script/e2e/seat-pointer.c` — new, the virtual-pointer helper.
  - `script/e2e/wlr-virtual-pointer-unstable-v1.xml` — new, vendored protocol (MIT, header kept).
  - `script/e2e/487-sway-pointer.sh` — new, the scenario.
  - `CONSTITUTION.md` §7 and the amending record — in their own commit.
  - `docs/marley/README.md` — the harness's line mentions the sway backend (Complete).
- **Ledger rows needed:** none.

### E2E plan
| REQ | Scenario step | Shot / evidence |
|---|---|---|
| REQ-001 | `COMPOSITOR=sway`; the run prints Hyprland's Marley-window count before and after (0) and never reloads Hyprland | `487-01-marley`; the run's report |
| REQ-002 | the stand-in agent runs; `click` on the agent bar's Rich Input button (coordinates read from `487-01`) | `487-02-rich-input-clicked` |
| REQ-003 | `type_text "typed under sway"`, `press "" Return` in the editor | `487-03-sent` |
| REQ-004 | `pointer_to` over the terminal; `scroll -10` | `487-04-scrolled-back` |
| REQ-005 | the end of the run | the closing check's line |
| REQ-006 | a second run with `PATH` stripped of `wtype` | the refusal message |
| REQ-007 | `just shot` (Hyprland) | its PNG and focus report |

Nothing here is out of a scenario's reach.

### Risks
- A key sent by a one-shot `wtype` switches the seat's keymap to its own; the held keyboard
  keeps the capability, and the feasibility check saw the chord arrive. If a key is lost, the
  fallback is a keyboard in the helper itself.
- The rich input button's position depends on the copied settings (font size, layout); the
  scenario reads it from its first shot, as the Hyprland scenarios read theirs.
- `swaymsg exit` ends sway's clients; a Marley that ignored TERM dies with its display.

## Phase 1 — closeout
Phase 1 PASS (autonomous, Chad's goal). Checklist: pre-flight ✓, recall ✓, promote ✓, prior art ✓,
spec ✓, design ✓.

## Phase 2 — Code
- **Built:** `script/e2e.sh` sources the scenario first and chooses its backend from
  `COMPOSITOR` (hyprland by default, `sway`), checks the backend's tools and `SIZE`, and keeps
  one run path with backend branches: the sway start (generated config, the session file the
  config's `exec` writes, the pointer helper as a coprocess, `wtype -s` holding the keyboard),
  Marley launched without the user's `DISPLAY` and `HYPRLAND_INSTANCE_SIGNATURE`, the window
  found by `app_id`, `teardown` for both backends, the sway stop and its closing check, and the
  Hyprland report under sway. `press` maps Hyprland's modifier names to wtype's; `type_text`
  hands the text to wtype whole; `shot` uses `grim` on the headless output; `click`,
  `pointer_to`, `pointer_down`, `pointer_up` and `scroll` talk to the helper and refuse under
  Hyprland. `script/e2e/seat-pointer.c` (the helper, built on first use into
  `$SHOT_DIR/.seat-pointer-<hash>`) and `script/e2e/wlr-virtual-pointer-unstable-v1.xml`
  (vendored, MIT, header kept). CONSTITUTION §7 and the amending record in their own commit
  (20369a93c7).
- **Deviations:** `COMPOSITOR` and `SIZE` also come from the environment, so
  `COMPOSITOR=sway just shot <name>` works without a scenario of its own. The Hyprland
  signature lookup no longer exports `WAYLAND_DISPLAY` by itself, so the sway path's read-only
  Hyprland report does not change the run's environment.
- **Review of the diff:** found and fixed before the first run: a redundant self-assignment of
  the coprocess pid; `pointer` writing to a helper that had died (now a named error); a sway
  that starts but never writes its session file (now killed by its config path in the stop).
  shellcheck clean on `script/e2e.sh` and `script/e2e/*.sh`.

## Phase 3 — Test
- **Scenario:** `script/e2e/487-sway-pointer.sh` (`compositor sway`; #481's stand-in agent; 200
  numbered lines before it starts). The first run doubled as the calibration: its click at a
  guessed point landed on the agent bar's right side and moved the focus off the terminal (the
  cursor went hollow in 487-02), which already showed a click landing; the Rich Input button
  was then read from 487-01 at (402, 954).
- **Shots (second run, `just e2e script/e2e/487-sway-pointer.sh`, SHOT_DIR in the scratchpad):**
  - `487-01-marley` — Marley at 1600×1000 inside the headless sway: the rail with the
    "Claude Code · waiting" row, the terminal ending `line 200`, `$ stand_in`, `ready`, the agent
    bar. (REQ-001)
  - `487-02-rich-input-clicked` — after `click 402 954`: the Rich Input editor is open above
    the bar ("A prompt for Claude Code"), the pencil button is highlighted, and its tooltip
    "Rich Input Ctrl-G" shows under the pointer. Only a click opens it at this point. (REQ-002)
  - `487-03-sent` — after `type_text "typed under sway"` and Return: the terminal shows
    `typed under sway` and `claude got: typed under sway`; the rail reads "working". (REQ-003)
  - `487-04-scrolled-back` — after `pointer_to 900 400` and `scroll -10`: the terminal shows
    lines 132 to 175, with its scrollbar, instead of the bottom. (REQ-004)
- **Run report:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule
  and did not reload it" (REQ-001); "sway: stopped, with the run's Marley, pointer and
  keyboard" (REQ-005); afterwards no `sway`, `marley`, `wtype` or `seat-pointer` process and no
  sway socket (checked with `ps` and `ls /run/user/1000/sway-ipc.*`).
- **Negative smoke (REQ-006):** with a PATH of `/usr/bin` minus `wtype`, the run printed "the
  sway backend needs wtype" and exited 1 before creating a profile copy.
- **Hyprland unchanged (REQ-007):** `just shot 487-hyprland` drew Marley on hidden workspace 9
  with the user's profile; "focus: the user's window and workspace are as they were". The shot
  showed the user's own project and was deleted once read.
- **Gate:** `just gate-diff` → `16 passed, 0 failed`, `GATE GREEN [diff]`; the receipt
  matches the tree.
- **Pre-existing:** none.

## Phase 4 — Complete
- **Docs (§21):** CHANGELOG (Changed: e2e scenarios can click, drag and scroll); CONSTITUTION §7
  and its amending record (20369a93c7, its own commit); `.claude/commands/pipeline/test.md` and
  `plan.md`; the spec template's UI proof; `docs/marley/README.md`'s standards line; the plan's
  prong 3 slices table (#487 shipped). No Marley crate and no Zed path changed.
- **Ledger:** L-claude-487-a-headless-seat-has-no-devices-until-a-client-adds-them-001,
  AD-claude-487-scenarios-that-click-run-in-a-headless-sway-001. No F- block: the review's
  finds were fixed before the first run, and the calibration miss was the scenario reading its
  target, not a bug.
- **Brain:** consultation f2cc655642e44af5b384d1252427ae5e closed as
  `decisions/marleys-e2e-scenarios-that-click-run-in-a-headless-sway` (follow-up 2026-10-24).
