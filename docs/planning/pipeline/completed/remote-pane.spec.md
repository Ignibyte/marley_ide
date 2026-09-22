---
pipeline_id: e24299ed-3cf8-42d8-bd75-cafa7fcbc93b
ticket: forge#84 (791c60d2-dbea-46c8-8e46-25f796a5eb9c) · local docs/planning/tickets/open/TICKET-084-remote-pane.md
aar_id: a76fd453-2079-436c-a910-8dfa31ca3073
status: Phase 5 — Complete PASS
title: open a remote (ssh) pane
type: feature
milestone: M3.A
references:
  - crates/terminal_blocks/src/session.rs (SessionOptions.args)
  - crates/terminal_blocks/src/pty_os.rs (pass args to Shell::new)
  - crates/marley_app/src/app.rs (SHIM: cmd-shift-o + spawn_remote_session + open-remote dispatch)
  - crates/marley_app/src/keymap.rs (cmd-shift-o → open-remote)
---

## Title
Compose an ssh target at the prompt, press cmd-shift-o → open a pane running `ssh <target>` — a normal
terminal pane, connected remotely. The first REMOTE pane.

## Scope
### In
- terminal_blocks: `SessionOptions.args: Vec<String>` + `pty_os::spawn` passes it to `Shell::new` (was a
  hardcoded empty vec). The existing zsh spawn passes `args: Vec::new()` (unchanged behavior).
- marley_app: `marley_remote` dep; keymap cmd-shift-o → `open-remote` + a test; a masked
  `spawn_remote_session(argv, cols, rows)` (shell=argv[0], args=argv[1..]); the `open-remote` dispatch —
  focused prompt line → `parse_ssh_target` (#83) → `ssh_command` → split a pane running it → clear the
  prompt → flash "ssh {host}".

### Out
- The remote-pane badge/tag (#85), connection status (#86), the hosts config (#87). ssh options beyond
  what #83 emits. Reconnect. (The pane tag may be stubbed here if trivial, else it's #85.)

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — thread the ssh argv through the EXISTING PTY spawn (SessionOptions.args) rather than a separate
  spawn path — a remote pane IS a normal session, just running ssh.
- D2 — the target is composed at the prompt (the #72 drivable pattern); cmd-shift-o (free) opens it.
- D3 — reuse #83's guarded `ssh_command` (argv, no shell, leading-dash-rejected + `--`) — no new injection
  surface; Marley spawns the user's ssh, which owns auth/known_hosts.
- D4 — a blank/invalid target (parse_ssh_target None) → no pane, no clear (the #72 F1 spirit).

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the keymap is queried, cmd-shift-o shall map to `open-remote`. | unit |
| REQ-002 | WHEN a session is spawned with `args`, `pty_os` shall pass them to the child (the zsh spawn stays argless). | unit/build (+ self-test) |
| REQ-003 (visual) | WHEN a valid ssh target is composed and cmd-shift-o is pressed, a new pane shall open running `ssh <target>`. | self-test (type → chord → ssh pane; `ps`) |
| REQ-004 | `scripts/gates.sh` GREEN, cov/MSI 100 on the pure surface (keymap); the spawn/dispatch masked. | gate |

## Phase Plan
- **P2** — SessionOptions.args + pty_os pass-through + the 2 fixture updates; keymap cmd-shift-o;
  spawn_remote_session + the open-remote dispatch; test plan.
- **P3** — implement (terminal_blocks + marley_app + the dep).
- **P3.5** — 1 critic (correctness + light security): the args thread-through doesn't break the zsh spawn;
  the remote spawn wiring (argv[0]/args split, empty-argv can't happen); reuses #83's safe argv.
- **P4** — the keymap test (cov/MSI 100) + the SELF-TEST (type `localhost` → cmd-shift-o → an ssh pane;
  `ps` the child) + gate GREEN.
- **P5** — docs, AAR, archive, close #84.
