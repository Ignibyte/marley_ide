---
pipeline_id: 9c847336-2bd7-4201-8975-7c4ce2a34a94
ticket: forge#16 (c17a08e5-9f30-4c0a-a0d6-1a95fd2b71e6) · local docs/planning/tickets/open/TICKET-012-app-shell.md
aar_id: 1090af94-6457-4151-80a3-1accaa2f2916
sprint: M1.A — The Usable Terminal (aa46e22f) seq 5/5 (FINALE)
status: Phase 5 — Complete PASS
title: marley_app (app-shell, minimal) — the first runnable Marley window (terminal + input)
type: feature
milestone: M1
references:
  - ../../../specs/SPEC-app-shell.spec.md
  - ../../../marley_architecture/crate-map.md
---

## Title

TICKET-012 — `marley_app` (crate **`crates/marley_app`**), sprint M1.A seq 5/5 — **THE FINALE**: the
first runnable Marley. `run()` boots ONE gpui window titled "Marley" mounting a live `marley_terminal`
pane + the `marley_editor` input; type a command, Enter runs it, a Block appears. The M1.A-minimal cut of
[`SPEC-app-shell.spec.md`](../../../specs/SPEC-app-shell.spec.md) — the full workspace shell (docks, pane
algebra, palette, keymap, multi-theme) grows in **M1.B**. **End state: `cargo run -p marley` opens a
usable bare terminal.**

## Scope

### In (M1.A — boot + mount + wire + Dark theme)
- **Boot (R1/R2/R3):** `run() -> ExitCode` boots one gpui `Application` + one window titled "Marley"
  (native macOS titlebar), fully offline (no network/auth/cloud/onboarding), `ExitCode::SUCCESS` on quit.
- **Mount the terminal (W1):** `RootView` spawns a `marley_terminal::TerminalSession` (`/bin/sh`, cwd =
  `std::env::current_dir()`), pumps it, and renders `session.blocks()` as text rows.
- **Mount the input (W2):** `RootView` holds a `marley_editor::Buffer`; renders `buffer.text()` as the
  input line + caret.
- **Key → edit (W3):** a printable key → `Buffer::edit(caret..caret, ch, EditOrigin::Human)`; Backspace
  deletes one char; the input re-renders.
- **Enter → run → block (W4 — THE finale AC):** Enter takes `buffer.text()` (trimmed) →
  `session.write_command(line)` → clears the Buffer → pumps; the new Block appears in the terminal view.
- **Live pump (W5):** the PTY is pumped on the gpui event loop so async shell output appears.
- **Dark theme (R21/R22/R23/R24):** a minimal `Theme`/`ThemeRegistry` wrapping
  `marley_ui_components::{Appearance,ThemeColors}` (NO local redeclaration, seam-contracts §6); a fresh
  `RootView` renders the **Dark default**; `set_theme` swaps it.
- §21 — CHANGELOG + `docs/marley_architecture/app_shell.md`. ONE screenshot baseline (`shell_dark`).

### Out / deferred to M1.B (per the spec's full surface)
- The 3-region **docks** (R4-R6); the **`PaneGroup` split/close/neighbor algebra** (R8-R13) AND
  `PaneGroup` entirely incl. `single` (R7) — RootView owns the terminal + input directly; no layout tree
  until split lands.
- The **command palette** + `filter_commands`/`ScoredCommand` (R14-R18) → **NO `nucleo` dep** in M1.A.
- The **`Keymap`** (R19-R20) — M1.A uses raw gpui `.on_key_down`, no binding registry.
- The **multi-theme registry** beyond the Dark/Light minimum; the other 3 screenshot baselines
  (both-docks-open, split 2x1, palette-open).

## Acceptance Criteria (EARS — M1.A subset + the wiring ACs)
- **AC1 (R1/R2/R3)** — WHEN `run()` is invoked, it shall boot exactly one gpui window titled "Marley"
  (native titlebar), perform zero network/auth/onboarding, and return `ExitCode::SUCCESS` on clean quit.
  *(boot/window/quit = ACCEPTED-UNTESTABLE, headed; the no-network boot path is asserted.)*
- **AC2 (W1)** — the terminal-view adapter shall turn a `marley_terminal::BlockList` into paintable rows
  (each Block → its `command` + `output_text()` lines). *(PURE — `rows_from_blocks`.)*
- **AC3 (W3)** — `apply_key(&mut Buffer, key)` shall insert a printable char at the caret (EditOrigin::
  Human), delete one char on Backspace, and otherwise leave the Buffer unchanged. *(PURE.)*
- **AC4 (W4)** — Enter shall yield `KeyOutcome::Submit(line)` where `line` is `buffer.text()` trimmed
  (or no submit when empty); the consumer writes it to the session + clears the Buffer. *(PURE decision:
  `apply_key`/`submit_line`; the write+clear+pump plumb = shim.)*
- **AC5 (R21/R22/R24)** — `ThemeRegistry::builtin()` shall contain ≥1 Light + ≥1 Dark theme; `default_for(
  a).appearance == a`; `by_name` exact-match `Some` else `None`; `dark_default().appearance == Dark` is
  the theme a fresh `RootView` reports. *(PURE.)*
- **AC6 (R23/R24)** — a fresh `RootView::active_theme()` shall be the Dark default; `set_theme(t)` shall
  make `active_theme()` return `t` and mark the view dirty. *(field-assign PURE; repaint = shim.)*
- **AC7 (gate-15, headed)** — a headed launch shall show one AX window titled "Marley" painted in the
  Dark theme with the terminal + input visible, matching the `shell_dark` baseline (masked + text-
  tolerant). *(ACCEPTED-UNTESTABLE render, proven by the marley_visual_harness headed lane.)*
- **AC-gate** — the PURE surface (themes + input + terminal_view) cov 100 / MSI 100; the gpui/PTY shim
  (`app.rs` + `bin/marley.rs`) ACCEPTED-UNTESTABLE (mutants::skip + a documented rust_cov exclude); FULL
  `scripts/gates.sh` → `GATE GREEN [diff]` INCL gate-15 (the `crates/marley_app` dir exists → the
  component is asserted, NOT silently skipped; the harness headless tests stay green).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
1. **Crate layout (load-bearing — satisfies all 3 constraints, zero spec edits):** dir
   **`crates/marley_app`** (gate-15's `visual_g` reads `component: marley_app` then checks `[ -d
   crates/marley_app ]` — a different dir → the UI component is SILENTLY SKIPPED, the blank-shipped-green
   trap); `[package] name = "marley"` (so `cargo run -p marley` works literally); `[lib] name =
   "marley_app"` (the frozen `marley_app::{run,RootView,ThemeRegistry}` surface M2 binds to); `[[bin]]
   name = "marley"` (`src/bin/marley.rs` → `marley_app::run()`). Run: `cargo run -p marley`.
2. **The pure/shim seam** — PURE (cov 100/MSI 100): `themes.rs` (Theme/ThemeRegistry), `input.rs`
   (`Key`[gpui-free]/`KeyOutcome`/`apply_key`/`submit_line`), `terminal_view.rs` (`rows_from_blocks`).
   ACCEPTED-UNTESTABLE (mutants::skip + rust_cov exclude `marley_app/src/app\.rs|marley_app/src/bin/`):
   `app.rs` (the gpui `RootView` Render + `run()` + the pump-timer) + `bin/marley.rs`. Keep `app.rs` THIN
   — every decision lives in the 3 pure files.
3. **Reuse, don't redeclare** — `Theme`/`ThemeRegistry` wrap `marley_ui_components::{Appearance,
   ThemeColors}`; the input drives a real `marley_editor::Buffer`; the terminal is a real
   `marley_terminal::TerminalSession`. Promote the proven `marley_spike` gpui boot/window/Render.
4. **Deps:** `gpui="0.2.2"` + `marley_ui_components` + `marley_terminal` + `marley_editor`; dev
   `marley_visual_harness` + `mutants`. **NO `nucleo`** (palette deferred), **NO `marley_core`**
   (SessionId is internal to `marley_terminal::spawn`; cwd via `std::env::current_dir` — an unused
   marley_core would fail machete). gpui already allowlisted; no new external crates.
5. **gate-15 in CI** = (`crates/marley_app` dir exists) + (harness exists) + (`cargo nextest -p
   marley_visual_harness` green). The app-shell `shell_dark` baseline is asserted in the **headed lane**
   (`cargo test -p marley --test headed_shell -- --ignored`), the spike precedent (the per-commit gate
   stays display-free).

### Open design decisions (Phase 2)
- **The live pump-on-timer** (the one surface the spike never exercised — it pumped to completion BEFORE
  `run()`): design the gpui event-loop pump (likely `cx.spawn` + `background_executor().timer(~16ms)` →
  `session.pump()` → `cx.notify()` on non-empty). Confirm the `cx.spawn`/`timer` signatures at gpui
  0.2.2. ACCEPTED-UNTESTABLE; prototype FIRST.
- **The headed baseline determinism** — the terminal body is non-deterministic. Decide the mask: a
  composite (titlebar band + the terminal-output region) so `shell_dark` asserts only the deterministic
  Dark bg + input-row chrome; plus the spike's blank-detector backstop (light-text-pixels > threshold)
  + the hard AX title/size assert.
- **Key mapping** — the gpui `KeyDownEvent` → our gpui-free `Key` (in the shim); `apply_key` is pure.

## Phase Plan
- **P2 Design** — read SPEC-app-shell + the spike's app.rs/read.rs/bin + the harness HeadedSession +
  the 3 mount crates' surfaces; the module manifest (themes/input/terminal_view pure + app.rs/bin shim);
  the pump-timer design (prototype); the headed-test + mask plan; the mutation map (apply_key transitions,
  submit trim, rows_from_blocks, ThemeRegistry lookups — non-trivial fixtures). DELEGATE the read.
- **P3 Implement** — the 3 pure modules + the app.rs gpui shim + bin + the rust_cov exclude.
- **P3.5 Inspect** — critics: the wiring purity (apply_key/submit/rows), the seam (only app.rs+bin shim),
  the ThemeRegistry, clean-room, machete (no nucleo/marley_core), the gate-15 dir trap.
- **P4 Validate** — the pure unit suite (AC2-AC6) + the headed visual test (AC7, #[ignore]); FULL gate
  INCL gate-15 (confirm the component is asserted, not skipped).
- **P5 Complete** — §21; close #16; archive; **the /goal "/work 12 to 16" is COMPLETE**.
