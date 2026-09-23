---
pipeline_id: 8cbb63e6-f011-4624-9c29-560572687153
ticket: docs/planning/tickets/open/TICKET-478-terminal-notifications.md
status: QUEUED — Phase 1 Plan drafted; ready to promote
title: Desktop notifications from a terminal, and from Claude Code
type: feature
slice: prong 1 T7b
references: [docs/marley/three-prong-plan.md, docs/planning/pipeline/queued/477-agent-bar.spec.md]
---

## Title
The terminal reads the desktop-notification escapes, Marley shows them when you are not looking
at that terminal, and a chip in the agent bar makes Claude Code send them.

## Scope
### In
- **The escapes:** `marley_dcs` gains a scanner for OSC 9 (`ESC ] 9 ; body`, iTerm2's, but not
  the `9;4;…` progress form) and OSC 777 (`ESC ] 777 ; notify ; title ; body`, rxvt's and
  Ghostty's), ended by BEL or ST, found across reads, with a length cap. The vendored event
  loop runs it beside the DCS tap and reports `Event::Notification { title, body }`; Zed's
  `Terminal` passes it on as an event of its own. vte ignores both escapes, so the bytes stay in
  the stream.
- **The notification:** `marley_workbench` shows it with gpui's `show_system_notification`
  (freedesktop on Linux, mako on this box) unless that terminal is in front of the user (the
  window active and the terminal the visible item). Activating it focuses the terminal. The
  rail's row takes the waiting state it shows for a bell. `set_app_identity` names Marley.
- **Claude Code:** while Claude Code runs and Marley's plugin is not installed, the agent bar
  (#477) shows "Enable Claude Code notifications". The chip writes a plugin and a local
  marketplace under Marley's data directory, then runs `claude plugin marketplace add <dir>`
  and `claude plugin install marley@marley`. The plugin's `Notification` and `Stop` hooks answer
  with a `terminalSequence` holding OSC 777, which Claude Code writes to its terminal, and only
  inside Marley's terminals.

### Out (explicitly deferred)
- OSC 99, kitty's richer protocol; sounds; in-app toasts.
- The other CLI agents' hooks (Codex, Gemini); the escapes from any program already work.

## Reference (§20)
- **Warp:** agent notifications (https://docs.warp.dev/agent-platform/capabilities/agent-notifications/):
  "Complete", "Request" (command approval, permission requests, idle prompts) and "Error", shown
  in the app and as desktop alerts while Warp is in the background; installed with a chip that
  adds Warp's plugin for Claude Code. No Warp code, and not the plugin's.
- **Upstream Zed:** the Agent Panel's notifications for its own threads (`agent_ui`), which are
  pop-up windows, not OS notifications.

### Prior art
- **Published material:** iTerm2's OSC 9; rxvt-unicode's and Ghostty's OSC 777 `notify`.
  Claude Code 2.1.281: the `Notification` hook (matchers such as `permission_prompt`,
  `idle_prompt`, `agent_needs_input`, `agent_completed`) and `Stop`; a hook's JSON answer may
  carry `terminalSequence`, which Claude Code emits, allowing only OSC 0, 1, 2, 9, 99 and 777 and
  BEL; `preferredNotifChannel` is `auto`, `iterm2`, `terminal_bell`, `iterm2_with_bell`, `kitty`,
  `ghostty` or `notifications_disabled`, and `auto` picks from `TERM_PROGRAM` (read from its
  settings schema and bundle; the hooks from its docs).
- **Code we already ship:** gpui's `App::show_system_notification`,
  `on_system_notification_response` and `set_app_identity`, which nothing in Zed calls, with a
  test platform that records what was shown and can answer; the DCS tap
  (`vendor/alacritty_terminal/src/marley_hooks.rs`) as the template; the rail's waiting state.

## UI proof
UI-AFFECTING: desktop notifications, a chip in the agent bar.
- **Driven tests:** a real PTY prints OSC 777 while another item is in front: the test
  platform shows one notification with its title and body; in front, none; answering it focuses
  the terminal. With a fake `claude` in the foreground and no plugin, the chip shows; clicking it
  runs a fake `claude plugin …` with the right arguments and writes the plugin.
- **Unit tests:** the scanner (both escapes, both ends, split reads, the progress form, the cap);
  the hook script's answer inside and outside Marley.
- **Live drive:** a terminal prints OSC 777 on the hidden workspace; `makoctl history` shows it.

## Locked-In Decisions
- D1 — Read the escapes other terminals use, so any program can notify, Claude Code included.
- D2 — A notification is display only; activating it focuses a terminal and sends nothing
  (PR-claude-474-a-hook-frame-is-output-until-its-nonce-says-otherwise-001).
- D3 — Claude Code's side is a plugin, not a change to `preferredNotifChannel`, which is global
  and would reach other terminals.
- D4 — Only when the terminal is not in front of the user, as Warp shows desktop alerts only in
  the background.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a program prints OSC 9 or OSC 777 notify in a terminal that is not in front of the user, Marley shall show a desktop notification with its title and body | driven |
| REQ-002 | WHILE the terminal is in front of the user, Marley shall show no desktop notification for it | driven |
| REQ-003 | WHEN the user activates the notification, Marley shall focus that terminal | driven |
| REQ-004 | The scanner shall find an escape split across reads and ignore the OSC 9;4 progress form and escapes longer than its cap | unit |
| REQ-005 | WHILE Claude Code runs and Marley's plugin is not installed, the agent bar shall offer the chip; WHEN it is clicked, Marley shall install the plugin through `claude plugin` | driven |
| REQ-006 | The plugin's hooks shall answer with an OSC 777 terminal sequence inside Marley's terminals and with nothing elsewhere | unit |
| REQ-007 | The diff gate shall be green | `just gate-diff` |

## Phase Plan
- **P1 Plan** — this spec; promotion re-verifies the seams, the plugin layout and the hook
  schema, and asks the brain. Split in two if the plugin half outgrows one ticket.
- **P2 Code** — the scanner, the event, the notification, the plugin and the chip; ledger rows
  first.
- **P3 Test** — unit and driven tests, negative checks, the live drive, the gate.
- **P4 Complete** — docs, ledger, close, archive, commit.
