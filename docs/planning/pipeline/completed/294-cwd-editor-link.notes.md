# cwd↔editor link — Notes

- **Forge ticket:** #294 `bb15ec7a-a875-45d8-8452-1c0523605b37` — THE LAST GOAL TICKET
- **AAR:** `3391a3fb-5bc6-4982-856a-977eb1b098b5`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-294-cwd-editor-link.md
- **Pipeline spec:** 294-cwd-editor-link.spec.md

## Phase 1 — Plan
- **Request:** (a) "Open a terminal here" (a new terminal at the active editor file's
  dir); (b) "cd terminal to the editor's dir" (write `cd <dir>` to the focused terminal).
- **Classification / tier:** work pipeline slice, `feature`. Crates: `marley_app`
  (a new pure `cwd_link.rs` + the app.rs shim). The closing ticket of the M18 goal train.
- **KEY discoveries (bound the work):**
  - `new_terminal_pane` (app.rs:3521): `cwd = valid_dir_or(focused_pwd, root)` →
    `spawn_session_in(&cwd, zdotdir, cols, rows)` → `Tab::terminal(...)` + `add_tab` +
    `persist_grid`. #294 (a) mirrors this but roots at the EDITOR dir.
  - `valid_dir_or(cand: Option<&str>, root) -> PathBuf` (app.rs:628, #281): absolute + `is_dir`
    → keep; else `root`. The existing "does it exist" guard — reuse it.
  - `spawn_session_in(cwd, zdotdir, cols, rows)` (599): roots a session at `cwd`.
  - `active_file().path` (editor_surface.rs:185): the editor's active file (absolute, resolved).
  - `write_command(&cmd)` (the `rerun_block` idiom): writes + executes a command string.
  - #292 lesson: `focused_terminal` is None when the EDITOR is focused → (b) is a no-op from
    the editor (invoke from a terminal); (a) is the editor-focused command.
- **Pure seams:** `dir_of(path) -> Option<PathBuf>` (`path.parent()` non-empty) +
  `cd_command(dir) -> String` (`cd '<single-quote-escaped>'`). Small, clean (cov/MSI 100).
- **Palette scheme:** free static `CommandId(12/13)` (0/1/2/4/5/6/7/8/10/11 used; 3/9 special) →
  `action_for_command` verbs `open-terminal-here`/`cd-terminal-here` → `dispatch_action` arms
  (the #292 `rerun-last-failed` pattern). The roster test `action_for_command_maps_every_row`
  + `every_cockpit_command_resolves_to_a_verb` cover them.
- **Risk:** low. Two tiny pure fns + two palette commands reusing shipped spawn/write. The only
  care: `dir_of` edges (bare/root path) + `cd_command` quoting + the valid_dir_or fallback.

## Phase 2 — Design

### Architecture / approach
Two tiny pure fns + two cockpit palette commands reusing the shipped #281 spawn +
the PTY-write path. Fits the cockpit layer (palette → verb → `dispatch_action` →
the terminal-spawn / PTY).

- **Pure `cwd_link.rs`:**
  - `dir_of(path: &Path) -> Option<PathBuf>` = `path.parent().filter(|p| !p.as_os_str().is_empty()).map(Path::to_path_buf)`.
    `/a/b.rs`→`Some(/a)`, `/a.rs`→`Some(/)`, `a.rs`→`None` (parent `""`), ``→`None`.
  - `cd_command(dir: &Path) -> String` = `format!("cd '{}'", dir.to_string_lossy().replace('\'', "'\\''"))`
    — POSIX single-quote wrap, embedded `'`→`'\''`, so a spaced/special path is shell-safe.
  - Registered `mod cwd_link;` in `lib.rs`.
- **Shim (`app.rs`):**
  - Extract `spawn_terminal_tab_in(&mut self, cwd: &Path)` from `new_terminal_pane`'s tail
    (`spawn_session_in` → `Tab::terminal(format!("terminal {n}"), PaneGrid::new_with_base)` →
    `add_tab` → `persist_grid`); `new_terminal_pane` now calls it with its `valid_dir_or` cwd.
  - `cockpit_commands` += `CommandId(12)` "Open Terminal Here" (kw terminal/here/cwd/directory) +
    `CommandId(13)` "cd Terminal to Editor Dir" (kw cd/terminal/editor/directory).
  - `action_for_command` += `12 => "open-terminal-here"`, `13 => "cd-terminal-here"`.
  - `dispatch_action`:
    - `"open-terminal-here"`: `root = active_project().root`; `dir = active_editor().and_then(|s|
      cwd_link::dir_of(&s.active_file().path))`; `cwd = valid_dir_or(dir.as_ref().and_then(|p|
      p.to_str()), &root)`; `self.spawn_terminal_tab_in(&cwd)`. (A degenerate path → root, never a fail.)
    - `"cd-terminal-here"`: `if let Some(dir) = active_editor().and_then(|s| dir_of(&s.active_file().path))
      { let cmd = cwd_link::cd_command(&dir); if let Some(st) = self.workspace_mut().focused_terminal_mut()
      { if !st.session.is_command_running() { let _ = st.session.write_command(&cmd); } } }`.

**§14:** the pure fns return `Option`/`String` (no panic — `to_string_lossy` is total);
process-spawn stays in `spawn_session_in`; the PTY write reuses `write_command`. No new IO.

**§20 (clean-room) — CONFIRMED.** Reference = the IDE "open terminal here" / "cd to file
dir" (VS Code "Open in Integrated Terminal", JetBrains "Open in Terminal"). Marley matches
with two cockpit commands over its own `spawn_session_in`/`valid_dir_or` (#281) + `write_command`.
No Zed/Warp source read — observed convention only.

### File manifest
| File | Change |
|------|--------|
| `crates/marley_app/src/cwd_link.rs` | NEW pure module: `dir_of` + `cd_command` (+ tests in P4) |
| `crates/marley_app/src/lib.rs` | + `mod cwd_link;` |
| `crates/marley_app/src/palette.rs` | `action_for_command`: + `12 => "open-terminal-here"`, `13 => "cd-terminal-here"` |
| `crates/marley_app/src/app.rs` | extract `spawn_terminal_tab_in(cwd)`; `cockpit_commands` += the 2 entries; `dispatch_action` += the 2 arms (shim; excluded) |

### Regression Test Plan
| # | Test | AC | Asserts |
|---|---|---|---|
| T1 | `cwd_link::dir_of_parent_or_none` | REQ-001 | `/a/b.rs`→`Some(/a)`; `/a.rs`→`Some(/)`; `a.rs`→`None`; ``→`None`. Kills body→None + the `!is_empty` filter mutants. |
| T2 | `cwd_link::cd_command_shell_quotes` | REQ-002 | `/a/b`→``cd '/a/b'``; `/a'b`→``cd '/a'\''b'`` (embedded quote escaped); a spaced `/a b`→``cd '/a b'``. |
| T3 | `palette::action_for_command_maps_every_row` (extend) | REQ-003 | + `action_for_command(CommandId(12)) == Some("open-terminal-here")` + `CommandId(13) == Some("cd-terminal-here")`. `every_cockpit_command_resolves_to_a_verb` auto-covers. |
| T4 | LIVE DRIVE | REQ-003 | Open a file (in a real project) → the palette → "Open Terminal Here" → a NEW terminal tab whose `pwd` (run `pwd`) is the file's directory. |
| T5 | gate `scripts/gates.sh --diff` | REQ-004 | 100% line cov + MSI 100 on `dir_of` + `cd_command`; brand-scrub; clippy -D. |

**Uncoverable:** the app.rs shim (the 2 dispatch arms + `spawn_terminal_tab_in`) is
`mutants::skip`/excluded — proven by the live drive + reuse of the tested `spawn_session_in`/
`valid_dir_or`/`write_command`. The pure fns + the roster are unit-tested.

### Risks / decisions
- **D5 — extract `spawn_terminal_tab_in` (DRY)** rather than duplicating `new_terminal_pane`'s
  spawn+add-tab tail; `new_terminal_pane` delegates to it. Both are excluded shims.
- **D6 — `cd_command` replacement is `'\''`** (close-quote, escaped-quote, reopen-quote) — the
  canonical POSIX single-quote escape. `to_string_lossy` handles non-UTF-8 paths (best-effort).
- **D7 — (b) writes to `focused_terminal_mut`, idle-guarded** (the `rerun_block`/#40 idiom).
  **Critic-corrected:** this is NOT a no-op from the editor — `workspace_mut()` →
  `terminal_grid_index()` falls back to the FIRST terminal tab when the active tab is the editor
  (the never-empties guarantee → always ≥1 terminal), so it cd's that terminal. The real no-ops are
  "no editor active" (`active_editor()` None → no dir) or "target terminal busy" (`is_command_running`).
- **D8 — the drive needs a REAL project** (root = a git dir) so `dir_of(open file)` is a real
  directory `pwd` can echo — a `New empty workspace` (root `~`) with an absolute /tmp file works too
  (the new terminal spawns in `/tmp/...`).

## Phase 3 — Implement
- **`cwd_link.rs` (NEW)** — `dir_of(path)` (`parent().filter(non-empty).map(to_path_buf)`) +
  `cd_command(dir)` (`format!("cd '{}'", to_string_lossy().replace('\'', "'\\''"))`).
- **`lib.rs`** — `mod cwd_link;` (after `context_menu`).
- **`palette.rs`** — `action_for_command`: `12 => "open-terminal-here"`, `13 => "cd-terminal-here"`.
- **`app.rs`** — extracted `spawn_terminal_tab_in(&mut self, cwd: &Path)` from `new_terminal_pane`
  (which now delegates); `cockpit_commands` += the `CommandId(12/13)` entries; `dispatch_action`
  += the 2 arms (`open-terminal-here` = `valid_dir_or(dir_of(active file), root)` →
  `spawn_terminal_tab_in`; `cd-terminal-here` = `dir_of` → `write_command(cd_command)` to the
  idle focused terminal).
- **Deviations:** none. `cargo check -p marley` clean (no errors/unused). The `open-terminal-here`
  borrow is sound — `dir` is owned (`dir_of` returns `PathBuf`), so the `active_editor()` immutable
  borrow releases before `spawn_terminal_tab_in(&mut self)`; same for `cd-terminal-here` (owned `cmd`
  before `workspace_mut`).
- **Tests deferred to P4.**

## Inspect (Phase 3.5)
Rigorous self-review + the real mutant-list confirmation. (One critic spawned; it keeps
running and will notify — any finding folds via a re-entry, per the #292 flow. NOT killed
on a heartbeat this time — the #293 lesson.)

**Self-review — no confirmed defects.** Lenses:
- **`dir_of` / `cd_command` correctness + mutation:** the real set (via `--list`) is 5 —
  `dir_of` {`None`, `Some(Default)`, `delete !`}, `cd_command` {`String::new()`, `"xyzzy"`}.
  ALL 5 die to T1 (`/a/b.rs`→`Some(/a)` kills `None`/`Some("")`/`delete-!`; `a.rs`→`None`
  kills `Some("")`) + T2 (exact `cd '/a/b'` kills both `cd_command` body mutants). NO mutant
  on `.map`/`.to_string_lossy`/the `.replace` literals. Panic-free (`parent`/`filter`/`map`
  + `to_string_lossy` are total).
- **`cd_command` shell-safety:** POSIX `'\''` escape (close/escaped/reopen) — single-quoting
  neutralizes `;`/`$`/backtick/space; only `'` is special inside single quotes and it's escaped.
- **`spawn_terminal_tab_in` extraction — behavior-preserving** (diff-verified): the exact
  `spawn_session_in`/`next_pane_block`/`Tab::terminal(format!("terminal {n}"))`/`add_tab`/
  `persist_grid` tail; `new_terminal_pane` still computes its `valid_dir_or(live_pwd, root)` cwd
  and delegates → the ⌘D new-terminal is unchanged.
- **Wiring:** `CommandId(12/13)` appear ONLY in `cockpit_commands` + `action_for_command`
  (grep-unique); `dispatch_action` has an `other =>` wildcard so the 2 new verb arms don't break
  exhaustiveness; `every_cockpit_command_resolves_to_a_verb` iterates dynamically (auto-covers).
- **Borrow:** `open-terminal-here` — `dir` is owned (`dir_of`→`PathBuf`), so the `active_editor()`
  immutable borrow releases before `spawn_terminal_tab_in(&mut self)`; `cd-terminal-here` — owned
  `cmd` before `workspace_mut()`. Sound (compiles).
- **(b) `cd-terminal-here` reach:** writes to `focused_terminal_mut()`. Corrected below (the critic) —
  it is FUNCTIONAL from the editor via the first-terminal fallback, not a no-op.

### Background critic report (folded in — completed during P4; I WAITED for its notification this time)
**Verdict: SHIP — no CRITICAL/HIGH/MED.** It empirically verified (real `zsh -f` harnesses + traces):
- **`cd_command` quoting injection-PROOF** — `;` / `$(…)` / backtick / `$dir` / `/tmp/x; echo PWNED` /
  a NEWLINE in the path all parse to exactly ONE literal `cd` arg, zero substitution/PWNED. `'\''` is
  the canonical POSIX escape; only `'` is special inside single quotes and it's escaped.
- **`dir_of` total** (no panic) across all edges; **`spawn_terminal_tab_in` byte-identical** to the old
  `new_terminal_pane` tail (`persist_grid` once; ⌘D unchanged); borrows sound; exhaustiveness green
  (wildcard `other =>` + both roster tests pass); §20 clean (zero brand words).
- **Correction to my ledger:** `cd-terminal-here` is NOT a no-op from the editor — `terminal_grid_index`
  falls back to the FIRST terminal tab, so it cd's that terminal (always ≥1). Folded into D7 above.

| LOW finding | Verdict | Action |
|---|---|---|
| cd arm trusts "editor path is absolute" (not `valid_dir_or`-guarded like the open arm) | invariant VERIFIED holds (all `CodeViewState` paths go through `resolve_under_root(absolute root, …)`) | none — documented; optional `debug_assert` not added (no new panic path) |
| no unit tests on `cwd_link` | **STALE** — snapshot predated P4; T1/T2 now pin the quoting + edges; gate MSI 100 | none (covered) |
| idle guard checks only the foreground cmd, not un-submitted prompt text | pre-existing #40 idiom (identical to #292 rerun) | none — consistent |
| doc nit ("no-op when no terminal focused") | slightly imprecise (there's always a terminal via fallback) | fixed in D7 (the record); the in-code comment is defensively-worded (handles the None case), acceptable |

**Ledger:** no confirmed defects (critic SHIP verdict; all LOW/advisory, invariants verified — no code
change warranted). The shim (spawn-in-dir) is proven by the P4 live drive. Lenses: correctness+mutation,
shell-safety (injection), extraction-equivalence, wiring/exhaustiveness, borrow, provenance.

## Phase 4 — Validate
### Tests added
- **T1** `cwd_link::dir_of_parent_or_none` (REQ-001) — `/a/b.rs`→`Some(/a)`, `/a.rs`→`Some(/)`,
  `a.rs`→`None`, ``→`None`.
- **T2** `cwd_link::cd_command_shell_quotes` (REQ-002) — `/a/b`→``cd '/a/b'``, `/a b`→``cd '/a b'``,
  `/a'b`→``cd '/a'\''b'`` (embedded quote escaped).
- **T3** `palette::action_for_command_maps_every_row` — + `CommandId(12)`==`Some("open-terminal-here")`
  + `CommandId(13)`==`Some("cd-terminal-here")`.
- Together T1/T2 kill the real 5-mutant set (`dir_of` None/Some(Default)/delete-!; `cd_command`
  String::new()/"xyzzy").

### Real results
- `cargo nextest run -p marley` → **455 passed, 2 skipped** (+3 new).
- `scripts/gates.sh --diff` → **GATE GREEN [diff]** (a first rustfmt RED on the new test bodies →
  `cargo fmt` → green; coverage ≥100% lines + MSI ≥100% on `dir_of` + `cd_command`; brand-scrub,
  miri, visual all green).

### LIVE DRIVE — REQ-003 proven end-to-end (the final goal ticket)
`New empty workspace` → `echo /tmp/marley_294/nested/target.rs:1` → clicked the link → the editor
opened `target.rs` (`294-01-editor.png`). Then **⌘⇧P → "open terminal here" → Enter** → a NEW terminal
tab **"nested"** spawned; running `pwd` printed **`/private/tmp/marley_294/nested`** (`294-02-pwd.png`)
— exactly `dir_of(/tmp/marley_294/nested/target.rs)` (`/tmp`→`/private/tmp` is the macOS symlink). The
command worked with the EDITOR focused (reads `active_editor`, spawns — no focused terminal needed).
REQ-003 ✓.
- Cleanup: app quit, `~/.marley` settings restored, `/tmp/marley_294` removed, tree clean.

No pre-existing failures in scope.

## Phase 5 — Complete
- **CHANGELOG.md** `### Added` (M18): the #294 cwd↔editor-link entry (marks the M18 train close).
- **docs/marley_architecture/app_shell.md**: the `M18 (#294)` note after #293.
- **Knowledge (forge wired):** AAR `3391a3fb` submitted (completed, effectiveness 5);
  `prevention-rule` **PR-claude-single-quote-shell-command-args-001** (HIGH — POSIX single-quote
  wrap any path/string interpolated into an executed shell command; `'`→`'\''`; verify injection-proof
  in a real shell). No `failure-record` — the critic gave a SHIP verdict; all findings LOW/advisory
  with invariants verified.
- **Lessons:** (1) The shell-quoting `'\''` escape is the injection-proof idiom (critic verified in
  `zsh -f` against `;`/`$()`/backtick/newline). (2) I WAITED for the critic's real completion
  notification this time (the #293 lesson) — it took ~8min but returned a SHIP verdict + a useful
  ledger correction (`cd-terminal-here` is functional via the first-terminal fallback). (3) Reusing
  `spawn_session_in`/`valid_dir_or` (#281) + extracting `spawn_terminal_tab_in` made this a thin,
  DRY slice.

**🏁 M18 Terminal↔Editor Fusion train `/work 289-294` COMPLETE** — all 6 goal tickets + the 3
foundation deps (#212/#213/#270) shipped LOCAL through the full pipeline, each live-driven.
