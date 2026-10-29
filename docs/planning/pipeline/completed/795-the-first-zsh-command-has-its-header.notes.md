# The first command in a new zsh on a Mac is a block under its header — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-795-the-first-zsh-command-on-a-mac-has-no-header.md
- **Pipeline spec:** 795-the-first-zsh-command-has-its-header.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-29: "lets do 795, 794, 947 then clean up all your builds etc and
  reinstall"; asked whether to build on the Mac, he chose "Build on the Mac now".
- **Classification / tier:** bug, S to M; the fix, if any, is unknown until the Mac check.
- **Recall (§18.3):**
  - #792's notes: the first command (`terminal_run "echo probe-792b"`) returned block 1, and the
    shot showed it without a header; the terminal was restored, with scrollback from the `login`
    runs above it.
  - F-claude-792 and PR-claude-792: on macOS the shell must be the PTY's own child.
  - The Linux check of 2026-10-29 (the ticket's section): no case shows it.
  - The brain: consultation 4aa8a277ccc147ba9eaa796ecaabe14c; nothing on this seam.

### Design
- Install `94bb105044` on the Mac (`/Volumes/Ext/Projects/marley_ide`, fetched from this box,
  `just install --remote-server-from dev`, niced with capped jobs).
- Open Marley on the Mac (`caffeinate -u`, `open ~/Applications/Marley.app`), a new terminal (zsh
  -l), type `echo one` and `echo two` through AppleScript keystrokes, shoot the window through
  iTerm's `screencapture -l <id>`, and read `terminal_blocks` over the Mac Marley's MCP endpoint.
- If the header is missing: compare the frames the Mac's zsh sends (a PTY capture there, as on
  Linux) and the block model's state for the first command; fix where they differ.
- **File manifest:** `script/e2e/795-the-first-zsh-command-has-its-header.sh` (the Linux check),
  and whatever the Mac check finds.

### Visual check plan
- REQ-001: the Mac shot `795-mac-first`.
- REQ-002: the Linux scenario, `795-01-fresh` and `795-02-restored` (and its variants:
  `PLAIN_FIRST=1`, `RUNNING_LAST=1`, `AGENT_FIRST=1`).

### Risks
- The Mac's display may be asleep or locked: `screencapture` returns black then (#787's note).
- The build competes with the other agent's work on the Mac; it runs niced, jobs capped.

## Phase 2 — Code
- **No code change:** the Mac check (Phase 3) found the first command a block under its header on
  the current build, so nothing is left to fix. The only file added is the Linux scenario,
  `script/e2e/795-the-first-zsh-command-has-its-header.sh`, with its three variants.
- **Gate:** `script/gates.sh --fast` green (no `.rs` changed): shellcheck, gitleaks, typos and the
  rest passed.

## Phase 3 — Test
- **Mac, at `94bb105044`:** fetched from this box into `/Volumes/Ext/Projects/marley_ide`, `just
  install --remote-server-from dev` (5 jobs, LTO off; 15 min 26 s). The macOS install wrote its
  `install.json` (#945's Mac path, seen working). Marley opened; no key reaches it over SSH (no
  Accessibility grant), so `marley.agent_control.actions` was set to `allow` with
  `workspace::NewTerminal` in `actions_allowed` for the check, and the settings file was put back
  from its copy afterwards (`cmp` equal).
  - `action_run workspace::NewTerminal` over the Mac Marley's own endpoint: a new `zsh` (`-l`)
    terminal, 0 blocks.
  - `terminal_run "echo one"` → block 0, exit 0; `terminal_run "echo two"` → block 1.
    `terminal_blocks`: both `verified`, `restored False`. In #792 the first command came back as
    block 1.
  - `795-mac-first` (REQ-001), shot through iTerm's `screencapture -l` once Chad unlocked the
    screen (it locks when the display sleeps at 10 min): `echo one` and `echo two`, each a block
    under its `~` header with its exit pill; "2 blocks" in the status bar. (Home stands first in
    Default there too, as #794 makes it.)
  - Not checked on the Mac: a command typed by hand, since keys cannot reach the Mac from here;
    `terminal_run` is the path #792 saw fail.
- **Linux** (REQ-002): the scenario with `AGENT_FIRST=1` today: `795-02-restored` shows the restored
  `echo one` and `echo two` blocks and the new `echo three` under its header. The other variants
  ran on 2026-10-29 at `57c5d4e5d4` (the ticket's Linux section).
- **Conclusion:** fixed between the Mac's build of 2026-10-16 and today; the change that fixed it
  is not pinned down (#816 and later terminal work touched the first prompt's path).
- Mac and Marley put back: Marley quit (it was not running before), settings restored, the check's
  files removed from `/Volumes/Ext/Builds`; iTerm left running, as it was.

## Phase 4 — Complete
- Documented: no CHANGELOG entry, since nothing in Marley changed; the ticket says how it closed.
  The Mac facts learned (the display lock, no Accessibility over SSH, the check's settings) go to
  the ops handbook's Mac page.
- Knowledge: L-claude-795-check-a-mac-bug-on-the-macs-current-build-first-001.
- Brain: consultation 4aa8a277ccc147ba9eaa796ecaabe14c closed with no decision.
- Ticket closed; pipeline archived.
