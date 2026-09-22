# open a remote (ssh) pane — Notes

- **Forge ticket:** #84 `791c60d2-dbea-46c8-8e46-25f796a5eb9c` (BACKLOG — M3.A sprint pending)
- **AAR:** `a76fd453-2079-436c-a910-8dfa31ca3073`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-084-remote-pane.md

## Phase 1 — Plan
- **Request:** forge #84 (M3.A seq-2, auto-approved) — open a remote ssh pane. Reuses #83 + the pane spawn.
- **Classification:** work pipeline, `feature`, terminal_blocks (SessionOptions.args) + a marley_app SHIM.
  UI.
- **Pre-flight facts:** `pty_os::spawn` already does `Shell::new(shell, Vec::new())` — args are supported,
  just hardcoded empty; SessionOptions{shell,cwd,env,cols,rows} (session.rs:80) needs an `args` field.
  Only 2 SessionOptions constructions (spawn_session app.rs:178 + a terminal_blocks test) → add
  `args: Vec::new()`. cmd-shift-o FREE (0 "o"). spawn_session (app.rs:176) is the mirror for
  spawn_remote_session. #83 marley_remote (parse/command) + #72 compose-at-prompt + #77 flash in place.
- **Decisions:** D1 thread the argv through the existing PTY spawn (SessionOptions.args); D2 compose at the
  prompt + cmd-shift-o; D3 reuse #83's guarded argv; D4 blank target → no pane/clear.
- **Self-test:** reset_mods → type `localhost` → cmd-shift-o → an ssh pane opens; `ps` confirms the child
  is ssh.
- **AAR id:** `a76fd453-2079-436c-a910-8dfa31ca3073`.

## Phase 2 — Design

### terminal_blocks
- `session.rs` SessionOptions: add `/// Args for the shell program (empty for a plain shell). pub args: Vec<String>`.
- `pty_os.rs`: `Shell::new(options.shell.to_string_lossy().into_owned(), options.args.clone())` (was `Vec::new()`).
- Fixture updates: the 2 `SessionOptions { … }` sites (app.rs spawn_session + a terminal_blocks test) add
  `args: Vec::new()`.

### marley_app
- `Cargo.toml`: add `marley_remote = { path = "../marley_remote" }`.
- `app.rs` import: `use marley_remote::{parse_ssh_target, ssh_command};`.
- `keymap.rs`: `(chord(true, false, false, true, "o"), "open-remote")` + a keymap test.
- `spawn_remote_session` (mutants::skip, mirrors spawn_session):
```rust
#[cfg_attr(test, mutants::skip)]
fn spawn_remote_session(argv: &[String], zdotdir: &Path, cols: u16, rows: u16)
    -> Result<TerminalSession, SessionError> {
    TerminalSession::spawn(SessionOptions {
        shell: PathBuf::from(argv.first().map(String::as_str).unwrap_or("ssh")),
        args: argv.get(1..).unwrap_or(&[]).to_vec(),
        cwd: std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
        env: session_env(zdotdir),
        cols,
        rows,
    })
}
```
- `open-remote` dispatch (mutants::skip):
```rust
"open-remote" => {
    let compose = self.workspace.focused();
    let target = self
        .workspace
        .state(compose)
        .map(|state| state.buffer.text())
        .and_then(|line| parse_ssh_target(&line));
    if let Some(target) = target {
        let argv = ssh_command(&target);
        let host = target.host.clone();
        let opened = self
            .workspace
            .split_focused(PaneAxis::Horizontal, SplitDirection::After, || {
                spawn_remote_session(&argv, &self.zdotdir, self.term_cols, self.term_rows)
            })
            .is_ok();
        if opened {
            if let Some(state) = self.workspace.state_mut(compose) {
                state.buffer = Buffer::new();
                state.caret = CharOffset::zero();
            }
            self.status_flash = Some(Flash::new(format!("ssh {host}")));
        }
    }
}
```
The compose pane's id is captured BEFORE the split (the split FOCUSES the new remote pane), so the clear
hits the compose prompt, not the fresh ssh pane. `parse_ssh_target` None (blank/invalid) → no pane, no clear.

### File manifest
- MODIFY `crates/terminal_blocks/src/session.rs` — SessionOptions.args.
- MODIFY `crates/terminal_blocks/src/pty_os.rs` — pass args to Shell::new.
- MODIFY `crates/marley_app/Cargo.toml` — marley_remote dep.
- MODIFY `crates/marley_app/src/keymap.rs` — cmd-shift-o + test.
- MODIFY `crates/marley_app/src/app.rs` — import, spawn_remote_session, the open-remote dispatch, the 2
  (well, 1 here) SessionOptions fixture `args: Vec::new()`.
- (fixture) the terminal_blocks SessionOptions test construction gets `args: Vec::new()`.

### Mutation Targets (pure)
- keymap: the cmd-shift-o arm (a keymap test). (SessionOptions.args is a data field — no logic; pty_os +
  the spawn + dispatch are masked live PTY/gpui.)

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | keymap `default_keymap_maps_named_chords` extended — cmd-shift-o → "open-remote" | unit |
| REQ-002 | the workspace + terminal_blocks build with the new args field (fixtures updated); the zsh spawn is argless | build + self-test |
| REQ-003 | compose `localhost` → cmd-shift-o → an ssh pane opens | self-test (`ps` the child is ssh) |
| REQ-004 | gate GREEN, cov/MSI 100 keymap; the spawn/dispatch masked | gate |

Uncoverable: `spawn_remote_session` + `pty_os::spawn` + the dispatch — live PTY + gpui, masked; proven by REQ-003.

### Risks / decisions
- D-2.1 the closure borrows argv (local) + self.zdotdir/term_cols (disjoint from self.workspace) — the
  #62 split pattern (two-phase/disjoint borrow) compiles. D-2.2 the compose pane id is captured pre-split
  so the clear targets the compose prompt (the split focuses the NEW pane). D-2.3 SessionOptions.args
  defaults empty for zsh → the local spawn is byte-identical. D-2.4 argv is #83's guarded output (non-empty
  by construction; `argv.first()` falls back to "ssh" defensively) → no injection, no panic.

## Phase 3 — Implement
- **Built (terminal_blocks):** `SessionOptions.args: Vec<String>`; `pty_os::spawn` passes
  `options.args.clone()` to `Shell::new` (was hardcoded `Vec::new()`).
- **Built (marley_app):** `marley_remote` dep; `use marley_remote::{parse_ssh_target, ssh_command}`; keymap
  cmd-shift-o → `open-remote` (+ the test assert); masked `spawn_remote_session(argv, zdotdir, cols, rows)`
  (shell=argv[0], args=argv[1..]); the `open-remote` dispatch (capture the compose pane pre-split → parse
  the line → ssh_command → split_focused spawning the remote session → clear the compose prompt → flash
  "ssh {host}").
- **Fixtures:** `args: Vec::new()` added to spawn_session + the 4 integration-test SessionOptions
  constructions (marley_app ×3 + terminal_blocks ×1).
- **Deviations:** the package name is `marley_terminal` (dir `terminal_blocks`); `// #77` sits on its own
  line after the broadcast flash (fmt).
- **Verification:** `cargo fmt`; `cargo check -p marley -p marley_terminal` 0 err; clippy `-D warnings` OK;
  `cargo nextest` 244 pass (no regression). The keymap test is the pure surface; the spawn/dispatch masked.

## Phase 3.5 — Inspect
- **Critic:** 1 (correctness + security; traced alacritty spawn + marley_remote guards; ran the crate tests).
  Verdict: **APPROVE / SHIP — no HIGH/MED/LOW defects.**
- **Confirmations:** (a) the args thread-through leaves the LOCAL zsh spawn byte-identical (`args:
  Vec::new()` → `Shell::new(shell, [])`); the 3 real-zsh marley_app + 5 real-PTY marley_terminal
  integration tests all pass. (b) the remote wiring is correct — argv (always `["ssh","--",dest]`, ≥3
  elems from #83) → shell="ssh" (PATH-resolved via posix_spawnp; the child inherits Marley's PATH,
  session_env only sets ZDOTDIR) + args=the rest; the `unwrap_or("ssh")`/`unwrap_or(&[])` are dead/
  defensive → no panic. (c) the compose-pane clear is correct — `compose` (a Copy PaneId) captured BEFORE
  split_focused (which mints a fresh id + focuses the new pane), so the clear hits the compose prompt, not
  the ssh pane; blank/invalid → parse None → no split/clear; clear+flash only on `opened`; the closure
  borrows disjoint self-fields (the #62 pattern). (d) NO new injection surface — #83's guarded argv passed
  straight as (program, args): no shell, no concat; a malicious line (`-oProxyCommand=x`→None; `; rm -rf /`
  / backticks → one inert literal host argv item); Marley spawns the user's ssh.
- **Findings + actions:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | N1 | NIT | `target.host.clone()` — target unused after ssh_command(&target). | **APPLIED** — moved (`let host = target.host`). |
  | O1 | OBS | Spawn failure → no clear/flash (silent). | Accept — identical to split-pane/new-agent; preserves the typed target for retry. |
  | O2 | OBS | session_env(zdotdir) passed to ssh (ZDOTDIR inert for ssh). | Accept — harmless; a remote pane is a normal session (D1). |
- **Fix applied (code):** N1 (clone→move). No bug (critic clean).

## Phase 4 — Validate
- **Test (in the keymap fixture):** `default_keymap_maps_named_chords` extended (REQ-001 — cmd-shift-o →
  "open-remote"). Also REQ-002 (the args field) is proven by the 244-pass build (all real-shell spawn
  tests updated with `args: Vec::new()`).
- **Runs (actual):** `cargo nextest -p marley -E 'test(default_keymap_maps_named_chords)'` → 1 passed;
  `cargo nextest -p marley -p marley_terminal` → 244 passed (no regression).
- **SELF-TEST (UI — REQ-003, drove the LIVE app + ps) — PASSED:** reset the stuck modifier → typed
  `localhost` at the prompt → ⌘⇧O. Result (`remote_pane.png`): a **new pane split** (right, focused) + the
  **compose prompt CLEARED** ("localhost" consumed) + the new pane shows a shell prompt.
  **`ps` confirms an `ssh` process is a child of Marley** (the pane runs `ssh localhost`, which connected).
  Proves cmd-shift-o opens a real remote ssh pane end-to-end.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. keymap tested; the spawn/dispatch masked.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_remote.md "## Open a remote pane (#84)".
- **Knowledge:** aar-submit (5). Critic APPROVE (no bug); 1 nit applied (clone→move). Key enabler: the PTY spawn already took args (hardcoded empty) — SessionOptions.args threaded it through.
- **Ticket:** forge #84 → done. **2/5 of M3.A** (tickets in the forge backlog tagged M3.A — sprint-create still erroring). cmd-shift-o opens a real ssh pane (verified via ps).
