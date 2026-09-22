# input routing by running-command state — Notes

- **Forge ticket:** #40 `659e95ef-bd2d-4d6f-a464-bb2c3193d1cc` (BUG)
- **AAR:** `758a132f-9c4a-4be1-8212-3cc820547bcf`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-040-running-input-route.md
- **Pipeline spec:** running-input-route.spec.md

## Phase 1 — Plan
- **Request:** forge #40 (M1.F "Real Interactivity" seq-1, auto-approved, BUG) — the arrow-key
  routing fix chad hit live against Claude Code.
- **Classification / tier:** work pipeline, `bug`, a small PURE change (input_route +1 arg,
  is_command_running delegator) + a 1-line-ish SHIM (pass the new arg). terminal_blocks + marley_app.
- **Discovery (§18):**
  - `input_route(alt_screen, ctrl) -> Route` (keys.rs:100) = `Raw` if `alt_screen || ctrl`. Called at
    app.rs:576; tested `input_route_cases` (keys.rs:183, 4 cases). Exported lib.rs:47.
  - `BlockList::current() -> Option<&Block>` (block.rs:132) = the last block IFF `Running` → the exact
    "a command is running" signal (already `pub`; `blocks()` is pub on the session).
  - The app.rs routing (573-577) reads `alt_screen` from `workspace.state(focused())` → add
    `command_running` from the same state.
  - SPEC-terminal-blocks R26 (line 154) + the test-map (199) + mutation-targets (243) reference
    input_route → update.
  - Dep #33 (input_route/encode_key/is_alt_screen) all shipped.
- **Decisions:** D1–D3 (running = `current().is_some()`; the 3-way OR; local-edit-at-prompt unchanged).
- **Open questions for Design:** whether `is_command_running` delegates through the model or reads
  `self.blocks().current()` directly (lean direct — `blocks()` + `current()` are both pub); the app.rs
  single-vs-double state read (combine the alt_screen + command_running reads).
- **AAR id:** `758a132f-9c4a-4be1-8212-3cc820547bcf`.

## Phase 2 — Design

### PURE — `crates/terminal_blocks/src/keys.rs`
```rust
/// Route a keystroke: `Raw` (stream to the PTY) while a full-screen program holds the alternate
/// screen, OR a foreground command is running (an inline interactive program owns the terminal), OR
/// a control key is held (signals); else `Cooked` (the local line editor) — R26.
pub fn input_route(alt_screen: bool, ctrl: bool, command_running: bool) -> Route {
    if alt_screen || command_running || ctrl {
        Route::Raw
    } else {
        Route::Cooked
    }
}
```

### PURE — `crates/terminal_blocks/src/session.rs`
```rust
/// `true` while a foreground command is running — a `Running` block exists (opened by the shell's
/// Preexec, finished by the next Precmd). Drives R26 input routing so an interactive program on the
/// primary screen receives keys instead of Marley's local editor.
pub fn is_command_running(&self) -> bool {
    self.blocks().current().is_some()
}
```
(`self.blocks()` → `&BlockList`; `BlockList::current()` (block.rs:132) = last block IFF `Running`.)

### SHIM — `crates/marley_app/src/app.rs` (~573)
Combine the focused-state read + pass the new signal:
```rust
let (alt_screen, command_running) = view
    .workspace
    .state(view.workspace.focused())
    .map(|s| (s.session.is_alt_screen(), s.session.is_command_running()))
    .unwrap_or((false, false));
...
input_route(alt_screen, event.keystroke.modifiers.control, command_running)
```
The existing `Raw` branch (`encode_key` → `write_bytes`) streams every key unchanged.

### File manifest
- M `crates/terminal_blocks/src/keys.rs` — `input_route` +`command_running`; `input_route_cases` → 8.
- M `crates/terminal_blocks/src/session.rs` — `is_command_running` + a mock+DCS test.
- M `crates/marley_app/src/app.rs` — the combined routing read.
- M `docs/specs/SPEC-terminal-blocks.spec.md` (R26 + test-map + mutation-targets) + SPEC-app-shell
  (R40 routing note). CHANGELOG; arch docs.

### Mutation Targets
- `input_route` — the two `||` + the three operands (`alt_screen`/`ctrl`/`command_running`). The
  8-case truth table kills every operand-drop + `||`→`&&` (e.g. `(F,F,T)→Raw` kills dropping
  `command_running`; `(F,F,F)→Cooked` kills any make-it-always-Raw).
- `is_command_running` — `.is_some()`→`.is_none()`/`true`/`false`. Killed by the fresh/finished→false
  + running→true fixtures.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `input_route_cases` — all 8 of `alt×ctrl×running`: `(F,F,F)→Cooked`; the other 7 → `Raw` | unit (keys.rs) |
| REQ-002 | `is_command_running_tracks_the_foreground_block` — a `ran(chunk)` helper (mock+`dcs_plain`+pump): fresh (no pump)→false; `[init, preexec]`→true; `[init, preexec, precmd]`→false | unit (session.rs) |
| REQ-003 | the app.rs routing streams while running | shim + masked visual (chad drives Claude Code / `read`) |
| REQ-004 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs keystroke routing (shim exclude; needs a live window + keystrokes).

### Risks / decisions
- D-2.1 `is_command_running` reads `blocks().current()` (both pub) — no new model method. The
  session-level mock+DCS test needs SEPARATE pumped sessions to observe running (chunk ends after
  preexec) vs finished (chunk includes precmd) — a coalesced pump applies all queued reads at once, so
  one chunk can't show the intermediate state.
- No regression: while running, Ctrl-C still streams (via `ctrl` OR `command_running`); PageUp now
  goes to the RUNNING program (correct — a pager wants it) instead of Marley's scrollback (which is
  the at-prompt behavior, #32) — matches a normal terminal.
- The bare-prompt path (Cooked → local editor + history #28/#29) is UNCHANGED — #40 is orthogonal to
  the tab-completion fork.

## Phase 3 — Implement
- **Built (per manifest):** keys.rs — `input_route` gains `command_running` (`Raw` if `alt_screen ||
  command_running || ctrl`) + the fn doc; `input_route_cases` rewritten to the 8-case truth table
  (maintenance — the 2-arg calls broke; it's also the REQ-001 test). session.rs —
  `is_command_running(&self) = self.blocks().current().is_some()`. app.rs — the routing now reads
  `(alt_screen, command_running)` in one focused-state lookup + passes `command_running` to
  `input_route`. SPEC-terminal-blocks R26 (+ is_command_running + the 3rd arg) + test-map +
  mutation-targets; SPEC-app-shell (the running-command routing clause); CHANGELOG (### Fixed).
- **Deviations from design:** none. (Rewrote the existing `input_route_cases` to 3-arg now, since the
  signature change broke its compile — the 8-case form is the design's REQ-001 test.)
- **Verification at this phase:** `cargo check --workspace` 0 errors; fmt; clippy `-D warnings` 0
  (terminal + marley); docs gate 0; `input_route_cases` PASS (8/8). The `is_command_running` mock+DCS
  test is Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (real scoped cargo-mutants on both pure fns [isolated target dir] + a full
  shell-lifecycle trace + regression grep). Verdict: **the fix is CORRECT; the only gap is the
  Phase-4 test.**
- **Findings table:**
  | # | Sev | Finding | Verdict | Carry-forward |
  |---|---|---|---|---|
  | F1 | HIGH (Phase-4) | `is_command_running` has NO test → cargo-mutants: 2/2 MISSED (`→true`, `→false`) + session.rs:176 uncovered (its only caller, app.rs, is coverage-excluded) → would fail gate:4 + gate:5. | REAL (the Phase-4 deliverable, not a code defect) | P4 adds the 3-fixture test: fresh→false (kills `→true`); `[init, preexec]`+pump→true (kills `→false`); +`precmd`→false. INIT before preexec (else MissingSession — the #37 precondition rule). |
- **Verified CORRECT by the critic (no code change):**
  - **input_route MSI 100** — 3 mutants: 2 `||`→`&&` CAUGHT (killed by `(T,F,F)` + `(F,F,T)`), 1
    `→Default` UNVIABLE (Route has no Default). Arg order matches the call site.
  - **is_command_running SEMANTICS sound** — `blocks().current().is_some()` (last block IFF Running)
    brackets exactly "a foreground command executes" (Preexec opens → Precmd finishes). All 5 edges
    traced: (a) Enter→Preexec startup window briefly false — negligible + no-worse-than-before; (b)
    `python` REPL stays Running → raw ✓; (c) `read` builtin → Running → raw ✓ (the bug fixed); (d)
    `sleep 1 &` → precmd finishes it → false at the prompt, NO misroute ✓; (e) multi-pane → the
    FOCUSED pane only ✓. History at the bare prompt preserved.
  - **SHIM reads the focused pane** (`.state(focused())`), safe `unwrap_or((false,false))` Cooked
    default; the Raw branch + the Cooked fall-through (history #29 / apply_key #28) are byte-for-byte
    UNCHANGED — only the routing READ + the `input_route` args changed. No #33 (alt/ctrl) regression.
    No panic/borrow conflict (baseline builds).
- **No code fix** — the fix is correct. F1 is the Phase-4 test; reuses
  `PR-…-state-machine-accessor-test-must-satisfy-preconditions` (#37 — init before preexec).

## Phase 4 — Validate
- **Tests:** `keys.rs::input_route_cases` (8-case truth table, at implement) + `session.rs::
  is_command_running_tracks_the_foreground_block` (a `ran(chunk)` helper: fresh→false;
  `[init, preexec]`→true; `[init, preexec, precmd]`→false — init-before-preexec per the #37
  precondition rule).
- **Runs (actual):** `cargo nextest run -p marley_terminal` → 88 passed (both #40 tests PASS);
  `cargo nextest run --workspace` → 508 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **4 caught /
  0 missed → MSI 100.0%** (2 `input_route` `||`→`&&` + 2 `is_command_running` `→true`/`→false`).
  Receipt written. No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Fixed` (at implement); `terminal_blocks.md` — the input-routing
  bullet. SPEC-terminal-blocks R26 + SPEC-app-shell at implement.
- **Knowledge captured:** `failure-record` `BF-…-input-routing-alt-screen-only-misses-primary-screen-
  interactive-programs` (category runtime, id d447a473) — the model conflated "wants raw keys" with
  "alt-screen active"; the real signal is "a command is running". Reused
  `PR-…-state-machine-accessor-test-must-satisfy-preconditions` (#37) for the is_command_running test
  (init before preexec). aar-submit `completed` (score 5). Win: the critic RAN cargo-mutants + traced
  all 5 shell-lifecycle edges, confirming `blocks().current().is_some()` is the exact signal (sleep &
  → finished, REPL → running) — the fix needed no logic change, only the Phase-4 test.
- **Ticket:** forge #40 → done; local doc → closed/; pipeline pair archived. 1 of 3 in M1.F.
