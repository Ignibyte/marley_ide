# The shell's Enter goes with its paste — Notes

- **Local ticket doc:** ../../tickets/closed/TICKET-816-keys-typed-after-enter-reach-the-shell-before-it.md
- **Pipeline spec:** 816-the-shells-enter-goes-with-its-paste.spec.md

## Phase 1 — Plan
- **Request:** Chad, 2026-10-19: "lets fox 816". Filed by #806's triage of 561.
- **Classification / tier:** bug; one Marley crate (`marley_workbench`), one function.
- **Recall (§18.3):**
  - F-claude-594 / PR-claude-keys-after-a-paste-go-in-a-later-write-001: why `paste_then` waits
    200 ms (Python's REPL); the rule names every path after `Terminal::paste`.
  - #806's triage of 561 (`20261019-101458`, shot `561-06`): `cd …/repoBROWSER=/mnt` ran as one
    line; the 10-17 run had `cd …/elsewherepython3`. The race runs on Marley's event time: keys
    queued behind the Return are handled within the 200 ms even when typed a second later.
  - The brain (consultation 65ef190095cb430fbb85cfa8345cf80e): nothing on this seam.
- **Discovery:** `crates/marley_workbench/src/rich_input.rs`, `send`: for every target,
  `paste_then(&terminal, &text, |terminal| terminal.input(b"\r".to_vec()), cx).detach()`, then
  `close` (focus to the terminal). `Target::{Agent(AgentKind), Shell}`.
- **Decisions:** D1, D2 in the spec.

### Design
- Approach: in `send`, match the target: `Target::Shell` does `terminal.update(cx, |terminal,
  _| { terminal.paste(&text); terminal.input(b"\r".to_vec()); })`, with a comment saying why a
  shell needs no wait; `Target::Agent` keeps `paste_then`. The doc comment of `send` says so.
- File manifest: `crates/marley_workbench/src/rich_input.rs` (Marley crate);
  `script/e2e/816-the-shells-enter-goes-with-its-paste.sh` (new). Phase 4: #817's ticket.

### Visual check plan
- REQ-001: under sway, the repository has `sub/` and `sub2/`; the scenario's HOME has a
  `.bashrc` (`PS1='$ '`, `here() { pwd >>"$E2E_WORK/where.log"; }`) and a `.zshrc` with the same.
  After the trust prompt, one `wtype -d 20` burst types `cd sub`, Return, `here`, Return.
  `816-01-bash` shows the two blocks; `expect` reads the log's last line ending `/repo/sub`.
- REQ-002: `profile_setting terminal.shell '{"program": "zsh"}'`, then `workspace: new terminal`,
  the burst with `cd sub2`; `816-02-zsh`; the log's last line ends `/repo/sub2`.
- Before and after: the same scenario on the installed build (no fix) should fail REQ-001.
- REQ-003: `script/regress script/e2e/561-browser-env-opener.sh`.

### Risks
- A program other than bash or zsh at a prompt Marley reports (fish has no Marley hooks) would
  get the `\r` with the paste; the editor never opens there.

## Phase 2 — Code
- Built: `send` matches the target; `Target::Shell` pastes and inputs `\r` in one
  `terminal.update`, `Target::Agent` keeps `paste_then`; the doc comment says so.
- Found by the first check run (debug build): bash passed; in zsh `cd sub2` ran alone but `here`
  stayed on zsh's line unrun (`816-02-zsh`: `$ here`, no editor). `follow_prompt` reopens the
  editor at the next prompt and takes the line the shell shows (`open_for`, `typed_text`); zsh
  had not yet echoed the typed-ahead `here`, so the editor opened empty, took the focus, and the
  following Return met an empty editor, which only closes. Deviation from the plan, inside the
  ticket's acceptance (each command runs, in order): `Prompt::took_line` records whether
  `open_for` took the shell's line; when it did not, `send` puts the line the shell shows
  (`autosuggest::typed_text`) ahead of the editor's text, so the Ctrl-U, paste and `\r` run it.
  The spec's scope and D3 say so.
- Review: `took_line` is set at every `open_for`, so it follows each prompt; with the editor
  holding the line it is ignored (D3); the agent target never reads the shell's line; the
  `terminal` binding moved above the empty check, used the same way after it.
- Check runs: three in a row pass both shells on the debug build; the installed build (no fix)
  fails bash: `cd subhere`, "No such file or directory".
- The gate: `just gate-diff` green, 17 of 17.

## Phase 3 — Test
- `just e2e script/e2e/816-the-shells-enter-goes-with-its-paste.sh` under sway on the debug build
  after the green gate; focus report "1 Marley windows before the run, 1 after". Both `expect`s
  pass. Shots read:
  - `816-01-bash`: two green blocks, `cd sub` in `…/repo`, then `here` in `…/repo/sub`; the rail's
    row says `here · done` (REQ-001).
  - `816-02-zsh`: a `sub2 — zsh` tab with `cd sub2`, then `here` in `…/repo/sub2` (REQ-002).
- `script/regress script/e2e/561-browser-env-opener.sh`: PASS (89 s). `561-06-user-browser`: the
  `cd …/elsewhere`, `python3 -c …`, `cd …/repo` and `BROWSER=… python3 …` lines each in a block of
  its own, all green (REQ-003); before, `cd …/repoBROWSER=/mnt` ran as one.
- Before and after (Phase 2's check runs): the installed build failed bash with `cd subhere`.

## Phase 4 — Complete
- Docs: `CHANGELOG.md` (Fixed, #816); `docs/marley_architecture/marley_workbench.md`, the prompt
  editor's section. No Zed path; no slice status moves.
- Ledger appends: `F-claude-816-keys-typed-after-enter-came-before-it-001`,
  `PR-claude-the-shells-enter-goes-with-its-paste-001` (beside PR-claude-keys-after-a-paste-…, which
  still holds for agents).
- Filed #817 (an agent's editor and the other `paste_then` paths share the 200 ms window), its row
  in the Queue.
- Brain: consultation `65ef190095cb430fbb85cfa8345cf80e` closed with `brain decide` (follow-up
  2026-11-19).
- Ticket closed; its `BACKLOG.md` row left at promotion.
