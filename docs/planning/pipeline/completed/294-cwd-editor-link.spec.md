---
pipeline_id: cd6f2818-d0a2-46c5-80f4-570a2133b944
ticket: forge#294 (bb15ec7a-a875-45d8-8452-1c0523605b37) · local docs/planning/tickets/open/TICKET-294-cwd-editor-link.md
aar_id: 3391a3fb-5bc6-4982-856a-977eb1b098b5
status: Phase 5 — Complete PASS
title: cwd↔editor link — open a terminal at the editor file's dir + cd the terminal to it
type: feature
milestone: M18
references: [docs/planning/intake/editor-as-peer-and-terminal-fusion.md]
---

## Title
A bidirectional cwd link: "Open Terminal Here" spawns a new terminal rooted at the
active editor file's directory; "cd Terminal to Editor Dir" writes `cd <dir>` to the
focused terminal. Pure `dir_of` + `cd_command`; the shim reuses the #281
`valid_dir_or`/`spawn_session_in` + the shipped `write_command`.

## Scope
### In
- **Pure** (`cwd_link.rs`): `dir_of(path: &Path) -> Option<PathBuf>` — the file's parent directory
  (`path.parent()` filtered to non-empty; `None` for a bare/empty path); `cd_command(dir: &Path) ->
  String` — a shell-safe `cd '<dir>'` (single-quote wrap, embedded `'` → `'\''`).
- **Shim** (`app.rs`), two cockpit palette commands:
  - **"Open Terminal Here"** (`CommandId(12)` → `open-terminal-here`): `cwd = valid_dir_or(dir_of(active
    editor file), root)` → `spawn_session_in(&cwd, …)` + `add_tab` (mirroring `new_terminal_pane`'s
    spawn+add-tab tail, but rooted at the editor dir instead of the focused pwd).
  - **"cd Terminal to Editor Dir"** (`CommandId(13)` → `cd-terminal-here`): `dir_of(active editor file)`
    → `write_command(cd_command(dir))` to the FOCUSED terminal (the #40 idle guard, like `rerun-last`).

### Out (explicitly deferred)
- A per-editor-tab "Open terminal here" BUTTON / a file-tree-dir right-click "Open terminal here" — the
  palette commands are the slice; affordances are follow-ups.
- (b) targeting a non-focused terminal — per the ticket, "cd" writes to the FOCUSED terminal; invoked
  from the editor (no focused terminal) it is a no-op. A workspace-terminal fallback (the #292/#295
  pattern) is a follow-up.
- A keybinding — the palette commands are the slice.

## Reference (§20)
The IDE "open terminal here" / "cd to file dir" convention — VS Code "Open in Integrated Terminal" (on
a file/folder), JetBrains "Open in Terminal". Marley matches with two cockpit commands over its own
terminal-spawn (#281) + PTY-write path. Clean-room §20: Marley's own `spawn_session_in`/`valid_dir_or`
(#281) + `write_command`; no Zed/Warp source read.

### Prior art
Not recorded: this gpui-era spec predates the required prior-art sweep. The subsection
was added on 2026-09-22 when the archive entered the fork's history.

## Locked-In Decisions
- **D1 — `dir_of` is the pure "which dir" seam; `valid_dir_or` (#281) is the existing "does it exist"
  guard.** `dir_of` extracts the parent; `valid_dir_or` validates absolute+dir and falls back to root.
  Composed: a live editor file's parent always exists → the terminal opens there; a degenerate path →
  root (never a spawn failure).
- **D2 — `cd_command` single-quotes the dir** (POSIX `'…'` with `'` → `'\''`) so a space/special char in
  the path is shell-safe. Pure string construction; `write_command` executes it (the `rerun_block` idiom).
- **D3 — two palette commands** (`CommandId(12/13)` → verbs), mirroring #292's `rerun-last-failed` wiring
  (append to `cockpit_commands` + `action_for_command` + `dispatch_action`); no new keybinding.
- **D4 — (a) spawns rooted at the editor dir; (b) writes to the FOCUSED terminal** (the ticket's split —
  (a) is the editor-focused case, (b) the terminal-focused case).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `path` has a non-empty parent, `dir_of(path)` shall return that parent; WHEN `path` is bare/empty (no usable parent), it shall return `None`. | cwd_link.rs unit: `/a/b.rs`→`Some(/a)`; `/a.rs`→`Some(/)`; `a.rs`→`None`; ``→`None`. |
| REQ-002 | `cd_command(dir)` shall be `cd '<dir>'` with embedded `'` escaped `'\''`. | cwd_link.rs unit: `/a/b`→``cd '/a/b'``; `/a'b`→``cd '/a'\''b'``. |
| REQ-003 | WHEN "Open Terminal Here" is invoked with a file open, the shim shall spawn a new terminal rooted at the file's directory. | Live drive: open a file → the command → a new terminal whose `pwd` is the file's dir. |
| REQ-004 | `dir_of` + `cd_command` shall be pure at 100% line coverage + MSI 100. | `scripts/gates.sh --diff`. |

## Phase Plan
- **P2 Design** — `dir_of`/`cd_command` (the pure module) + the 2 palette commands (CommandIds + verbs + dispatch, reusing `valid_dir_or`/`spawn_session_in`/`write_command`); the test plan.
- **P3 Implement** — the pure module + the shim.
- **P3.5 Inspect** — `dir_of` edges (bare/root/empty), `cd_command` quoting (embedded quote), the spawn-in-dir wiring (valid_dir_or fallback), the focused-terminal guard.
- **P4 Validate** — pure units (REQ-001/002) + a LIVE DRIVE (open a file → "Open Terminal Here" → `pwd` == the file's dir); gate green [diff].
- **P5 Complete** — CHANGELOG + app_shell.md, AAR, close #294 → the M18 goal train COMPLETE.
