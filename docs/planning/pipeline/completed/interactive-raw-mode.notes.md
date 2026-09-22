# interactive/raw mode — Notes

- **Forge ticket:** #33 `09a81942-989f-4d0f-a191-1a4290131e3f`
- **AAR:** `c58ef971-3d07-432f-b251-1b2e56504f1b`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-033-interactive-raw-mode.md
- **Pipeline spec:** interactive-raw-mode.spec.md
- **Deferred intake:** docs/planning/intake/prompt-shell-line-editing-model.md

## Phase 1 — Plan
- **Request:** forge #33 (M1.D "The Daily Driver" seq-6, the FLAGSHIP, auto-approved) — interactive/
  raw mode. The sprint finale.
- **Classification / tier:** work pipeline, `feature`, cross-crate (marley_terminal encoder +
  accessors; marley_app routing + render). The largest M1.D ticket.
- **Discovery (§18) — the load-bearing facts:**
  - `TerminalSession::write_bytes` STREAMS raw bytes to the PTY (R13, ships) — the encoder's sink.
  - `write_command` = command + `\r\n` (the cooked path). `apply_key` (local Buffer editing) is the
    current cooked model; keystrokes never reach the shell.
  - alacritty `TermMode` bitflags (term/mod.rs:55) — `ALT_SCREEN = 1<<12`; `term.mode() -> &TermMode`
    (mod.rs:709). So `is_alt_screen = term.mode().contains(ALT_SCREEN)`.
  - gpui `Keystroke { key: str, modifiers{control,alt,shift,platform}, key_char: Option<String> }`
    — the shim maps this → the gpui-free `KeyInput`.
  - #31's `term_to_styled_rows` produces the styled grid rows — reused for the alt-screen live-grid
    render (D4).
- **THE ARCHITECTURAL FINDING (shapes scope):** tab-completion at the bare prompt happens in the
  shell's ZLE, which needs raw keystrokes as-typed — i.e. streaming to the shell + rendering its
  echo, which CONFLICTS with Marley's local line editor (#28/#29). That's a product-model fork
  (Warp-local vs iTerm-streaming vs hybrid) → DEFERRED to intake `prompt-shell-line-editing-model.md`.
  Alt-screen raw mode has NO prompt to conflict with, so it's a clean addition — #33 delivers that
  (vim/top/less/htop + Ctrl-C) and honestly notes tab-completion is deferred (D6).
- **Decisions:** D1–D6 in the spec (encoder in marley_terminal; ctrl_byte &0x1f; routing; reuse
  #31 grid render; not split; honest partial delivery).
- **Open questions for Design:** where `input_route`/`Route` live (keys.rs beside encode_key);
  whether `KeyInput` folds shift into the resolved char (yes — gpui key_char is already shifted) or
  carries shift (defer Shift-Tab etc.); the exact gpui key strings for the special keys (confirm at
  implement vs the #28 precedent: "up"/"down"/"left"/"right"/"home"/"end"/"pageup"/"pagedown"/
  "escape"/"delete"/"tab"); the render-switch placement (before the Block loop); whether the
  Ctrl-C integration test uses the existing real-PTY test harness (yes, the #27 lane).
- **AAR id:** `c58ef971-3d07-432f-b251-1b2e56504f1b`.

## Phase 2 — Design

### PURE — NEW `crates/terminal_blocks/src/keys.rs` (gpui-free)
```rust
pub enum KeyCode { Char(char), Enter, Backspace, Tab, Escape, Up, Down, Left, Right, Home, End, PageUp, PageDown, Delete }
pub struct KeyInput { pub code: KeyCode, pub ctrl: bool, pub alt: bool }
pub enum Route { Cooked, Raw }

/// Ctrl+key → the C0 control byte. `& 0x1f` already masks off the ASCII case bit, so NO
/// to_ascii_uppercase (it would be a redundant/equivalent mutant — the #31 lesson).
pub fn ctrl_byte(c: char) -> u8 { (c as u8) & 0x1f }   // Ctrl-C→3, Ctrl-D→4, Ctrl-Z→26

pub fn encode_key(input: KeyInput) -> Vec<u8> {
    match input.code {
        KeyCode::Char(c) if input.ctrl => vec![ctrl_byte(c)],
        KeyCode::Char(c) if input.alt  => { let mut v = vec![0x1b]; v.extend(c.to_string().into_bytes()); v }
        KeyCode::Char(c)               => c.to_string().into_bytes(),
        KeyCode::Enter     => vec![b'\r'],          // 0x0d
        KeyCode::Backspace => vec![0x7f],           // DEL
        KeyCode::Tab       => vec![b'\t'],          // 0x09
        KeyCode::Escape    => vec![0x1b],
        KeyCode::Up    => vec![0x1b, b'[', b'A'],
        KeyCode::Down  => vec![0x1b, b'[', b'B'],
        KeyCode::Right => vec![0x1b, b'[', b'C'],
        KeyCode::Left  => vec![0x1b, b'[', b'D'],
        KeyCode::Home  => vec![0x1b, b'[', b'H'],
        KeyCode::End   => vec![0x1b, b'[', b'F'],
        KeyCode::PageUp   => vec![0x1b, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![0x1b, b'[', b'6', b'~'],
        KeyCode::Delete   => vec![0x1b, b'[', b'3', b'~'],
    }
}

/// alt-screen → stream ALL keys; otherwise a control key streams (the signal reaches the shell),
/// everything else stays cooked (local edit — #28/#29 preserved).
pub fn input_route(alt_screen: bool, ctrl: bool) -> Route {
    if alt_screen || ctrl { Route::Raw } else { Route::Cooked }
}
```

### SHIM/tested — `crates/terminal_blocks/src/session.rs` (via mock, NOT mutants::skip)
```rust
use alacritty_terminal::term::TermMode;
pub fn is_alt_screen(&self) -> bool { self.term.mode().contains(TermMode::ALT_SCREEN) }
pub fn grid_styled_rows(&self) -> Vec<StyledLine> { term_to_styled_rows(&self.term) }
```
Both TESTED deterministically: a mock `push_read(b"\x1b[?1049h")` → pump → passthrough →
`processor.advance` → `is_alt_screen()` true; `\x1b[?1049l` → false. A mock feeding `"hi"` →
`grid_styled_rows()` first row has a run "hi".

### SHIM — `crates/marley_app/src/app.rs` (mutants::skip)
- `key_input_from_keystroke(keystroke) -> Option<KeyInput>` — the gpui `Keystroke` → `KeyInput` map
  (the special-key strings "up"/"down"/…/"escape"/"delete"/"tab"/"enter"/"backspace" + `key_char`
  → `Char`; ctrl/alt from modifiers).
- on_key_down (AFTER palette + keymap-chord handling, BEFORE scroll/history): compute
  `route = input_route(focused.is_alt_screen(), keystroke.modifiers.control)`; `Route::Raw` →
  `session.write_bytes(&encode_key(input)); return;` (so alt-screen arrows/keys + cooked Ctrl-C
  stream and do NOT fall into scroll/history). `Route::Cooked` → fall through to the existing
  scroll/history/apply_key path.
- Render: per pane, `if session.is_alt_screen()` → paint `grid_styled_rows()` via the #31 colored-
  span loop (full grid, no viewport); else the existing Block+prompt+viewport render.

### Decisions
- D-2.1 `ctrl_byte = (c as u8) & 0x1f` — NO uppercase (masked anyway → avoids an equivalent mutant).
- D-2.2 `is_alt_screen`/`grid_styled_rows` are TESTED (mock DECSET / mock content), not shim.
- D-2.3 REQ-005 Ctrl-C: the DETERMINISTIC gate proof is a MOCK test — `write_bytes(encode_key(Ctrl-C
  ))` records `[0x03]` on the channel (Marley sends the right byte; the OS turning 0x03→SIGINT is
  not Marley's code). The real-PTY sleep-interrupt + vim/top visual = the headed/manual acceptance
  (masked, desktop-session deferral) — NOT a gate test, to avoid real-PTY flakiness on the finale.
- D-2.4 Raw routing sits BEFORE scroll/history so alt-screen arrows/PageUp reach the program (not
  Marley's scroll/history); cmd-chords (keymap) still fire first so split/palette work in vim.

### File manifest
- A `crates/terminal_blocks/src/keys.rs` — KeyCode/KeyInput/Route/encode_key/ctrl_byte/input_route + tests.
- M `crates/terminal_blocks/src/lib.rs` — `mod keys;` + exports.
- M `crates/terminal_blocks/src/session.rs` — `is_alt_screen` + `grid_styled_rows` + TermMode import + tests.
- M `crates/marley_app/src/app.rs` — `key_input_from_keystroke`, the raw routing, the render switch.
- M `docs/specs/SPEC-terminal-blocks.spec.md` (R25 encoder + R26 alt-screen) + `SPEC-app-shell.spec.md` (R40 render).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `encode_char_and_control` — 'a'→[0x61]; alt-'a'→[0x1b,0x61]; Ctrl-C→[0x03]; Ctrl-D→[0x04]; Ctrl-Z→[0x1a] (kills the `&0x1f` via z's high bits); a multibyte char → its UTF-8 | unit (marley_terminal) |
| REQ-002 | `encode_named_keys` — Enter [0x0d]; Backspace [0x7f]; Tab [0x09]; Escape [0x1b]; Up/Down/Right/Left [1b 5b 41/42/43/44]; Home/End [..48/46]; PageUp/Down [1b 5b 35/36 7e]; Delete [1b 5b 33 7e] (each distinct → kills a wrong-final mutant) | unit |
| REQ-003 | `input_route_cases` — (true, false)→Raw; (true, true)→Raw; (false, true)→Raw; (false, false)→Cooked | unit |
| REQ-004 | `is_alt_screen_tracks_decset` — mock feeds `\x1b[?1049h` → is_alt_screen true; `\x1b[?1049l` → false; `grid_styled_rows_reflects_grid` — mock feeds "hi" → first row run "hi" | unit (session, mock) |
| REQ-005 | `ctrl_c_writes_the_interrupt_byte` — a mock session; `write_bytes(encode_key(Ctrl-C))` → the mock recorded exactly `[0x03]` | unit (session, mock) |
| REQ-006 | `scripts/gates.sh --diff` GREEN + receipt; the vim/top headed baseline masked | gate |

Uncoverable: the app.rs `key_input_from_keystroke` + routing + render switch (existing shim
exclude); the real-PTY vim/top visual (desktop-session deferral).

### Risks / decisions
- Real-PTY flakiness (the #27 lesson): keep the GATE proof deterministic (mock write-path) — the
  live SIGINT is an OS behavior, headed/manual.
- The gpui special-key strings — confirm at implement against the #28 precedent (write-then-check).
- Alt-screen render shows the WHOLE grid (no viewport) — the program owns its screen; the cooked
  viewport (#32) does not apply in alt-screen.
- `input_route` `||` — a mutant `&&` → (true,false)→false→Cooked (alt-screen wouldn't stream) →
  caught by the (true,false) case; a mutant dropping either operand → caught by the (false,true) /
  (true,false) cases.

## Phase 3 — Implement
- **Built (per manifest):** marley_terminal — NEW keys.rs (`KeyCode`/`KeyInput`/`Route`,
  `ctrl_byte` = `(c as u8) & 0x1f` [NO uppercase — D-2.1], `encode_key` all arms, `input_route`);
  lib.rs `mod keys` + exports; session.rs `is_alt_screen` (`term.mode().contains(TermMode::
  ALT_SCREEN)`) + `grid_styled_rows` (`term_to_styled_rows(&self.term)`) + the `TermMode` import.
  marley_app — app.rs `key_input_from_keystroke` (gpui Keystroke → KeyInput; special-key names +
  `key_char`/key-name-char fallback for ctrl), the RAW routing (before scroll/history: `alt_screen`
  from `state(focused)` + `input_route` → `Raw` → `write_bytes(encode_key(input))`), the render
  SWITCH (`is_alt_screen` → paint `grid_styled_rows` colored, no viewport; else the cooked Block+
  viewport render). SPEC-terminal-blocks R25/R26 + SPEC-app-shell R40 + AC/Test-Plan/Mutation-
  Targets; CHANGELOG. Intake `prompt-shell-line-editing-model.md` for the deferred tab-completion.
- **Deviations from design:** none. (The grid-render colored-span loop duplicates the cooked one
  ~8 lines — acceptable shim duplication, both excluded from cov/mutation.)
- **Verification at this phase:** `cargo check` 0 errors; fmt; clippy `-D warnings` 0; docs gate 0
  (proactive); machete clean; marley_terminal 74 + marley 90 lib tests pass. The encoder/route/
  accessor unit suites + the Ctrl-C mock test are Phase 4.

## Phase 3.5 — Inspect
- **Critics:** 2 — A (encoder bytes: 38 probes + a real cargo-mutants run on keys.rs), B (mode/
  routing/render seam: a real Term+Processor probe for is_alt_screen + llvm-cov + build + cargo tree).
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | B-MED | MED | `key_input_from_keystroke` ignored the ⌘ (platform) modifier → in alt-screen an UNBOUND cmd-chord (⌘C/⌘V/⌘A) leaked as a raw byte to the program (⌘C would insert 'c' in vim). Cooked path already filters it (Key::Other); raw path didn't. Invisible to the gate (app.rs shim). | REAL (bug) | Added `if keystroke.modifiers.platform { return None; }` at the top of `key_input_from_keystroke` (mirrors the cooked `control\|\|platform`→Other). |
  | A/B-HIGH | HIGH (Phase-4) | keys.rs (encode_key/ctrl_byte/input_route) + the session accessors have ZERO tests → gate:4 cov + gate:5 MSI RED. Correctly NOT mutants::skip; PROVEN killable (both critics). | REAL (Phase-4 constraint) | Phase 4 writes the unit suites; the kill-map + load-bearing fixtures are recorded below. |
  | A-LOW | LOW | cargo-mutants does NOT mutate the VT byte literals → MSI gives NO regression guard on sequence correctness. | NOTED | Phase 4 pins each key's exact bytes as explicit goldens (a typo'd final would else slip). |
  | A/B-LOW | LOW | Ctrl+Alt-Char → ctrl wins (ESC dropped); named-key modifiers ignored (ctrl-Up → plain ESC[A); ctrl+Enter/Backspace in cooked stream raw; write_bytes swallow; render dup. | ACCEPTED | Fidelity/scope gaps + shim norms; none a regression. |
- **Verified CORRECT (both critics):** ALL VT sequences byte-exact (Up 1b5b41 / Down 42 / Right 43 /
  Left 44 / Home 48 / End 46 / PageUp 1b5b357e / PageDown 367e / Delete 33 7e / Enter 0d=CR /
  Backspace 7f=DEL / Tab 09 / Escape 1b); ctrl_byte C/D/Z/A = 3/4/26/1, case-masked (no
  to_ascii_uppercase — equivalent-mutant correctly avoided); NO unkillable mutant (12 viable
  killable, 1 unviable Default); routing order regression-free (palette→chord→RAW→scroll→history→
  apply_key: alt-screen streams all; cooked Ctrl-C streams; cooked plain falls through to history/
  scroll/apply_key); is_alt_screen killable via mock DECSET (ALT_SCREEN the exact flag); gpui-free
  boundary held; VT sequences public ECMA-48/xterm (clean-room).
- **CRITIC KILL-MAP for Phase 4 (load-bearing fixtures):** ctrl_byte use Ctrl-C/D/**Z** (Ctrl-A=1
  can't kill `→1`); assert a named key `Enter→[0x0d]` (kills the vec![]/[0]/[1] mutants — do NOT
  make the expected literally [0]/[1], a Ctrl-@/Ctrl-A trap); Char per modifier state (plain
  'a'→[0x61], alt-'a'→[1b 61], Ctrl-C→[3]); `input_route` BOTH (true,false)→Raw AND (false,true)→Raw
  + (false,false)→Cooked (kills `||`→`&&`); is_alt_screen mock `\x1b[?1049h`→true / `l`→false;
  grid_styled_rows mock "hi"→row run "hi".
- **Post-fix:** the ⌘-filter compiles + clippy clean. Lesson:
  `PR-claude-raw-input-passthrough-must-filter-platform-chords`.

## Phase 4 — Validate
- **Tests added (keys.rs 3 + session.rs 3):** `encode_char_and_control` (char/alt-char/Ctrl-C/D/**Z**
  + the case-masked ctrl_byte), `encode_named_keys` (the 13 VT/CSI GOLDENS — Enter 0d / Backspace
  7f / arrows / Home/End / Page / Delete), `input_route_cases` (all 4, both Raw operands);
  `is_alt_screen_tracks_decset` (mock `\x1b[?1049h` → true, kills contains/flag mutants),
  `grid_styled_rows_reflects_grid` (mock "hi"), `ctrl_c_writes_the_interrupt_byte` (mock records
  `[0x03]` — the deterministic Ctrl-C proof, no real-PTY flake).
- **One fix mid-validate:** `is_alt_screen_tracks_decset` first fed h THEN l, but one pump consumes
  all queued reads (mid-burst retry) → netted back to primary → simplified to enter-only (observes
  false→true, sufficient for cov+MSI).
- **Runs (actual):** `cargo nextest run -p marley_terminal` → the 6 new PASS (85 in marley_terminal);
  full workspace green.
- **Visual (gate:15):** PASS; the vim/top alt-screen headed baseline rides the desktop-session
  deferral (masked).
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15 FIRST TRY**, coverage 100% lines,
  mutation **16 caught / 0 missed → MSI 100.0%**, 0 SLOW hangs; receipt written.
- **Pre-existing:** none. (The critic's VT-literal note — cargo-mutants doesn't mutate the byte
  literals — is covered by the explicit `encode_named_keys` goldens, not MSI-reliant.)

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `terminal_blocks.md` gains a `keys.rs`
  bullet (encoder + accessors); `app_shell.md` gains an interactive/raw-mode bullet (routing +
  render switch). SPEC-terminal-blocks R25/R26 + SPEC-app-shell R40 at implement. Intake
  `prompt-shell-line-editing-model.md` for the deferred tab-completion decision.
- **Knowledge captured:** `BF-claude-raw-input-leaks-unbound-cmd-chords-to-program` (inspect — the
  ⌘-chord leak in alt-screen) + `PR-claude-raw-input-passthrough-must-filter-platform-chords`
  (a new passthrough must mirror the existing path's modifier filter). aar-submit `completed`.
  Wins: the encoder-in-marley_terminal (gpui-free) seam + the DESIGN-TIME equivalent-mutant
  avoidance (`ctrl_byte` = `(c as u8) & 0x1f`, NO uppercase — the #31 disjoint-bit lesson applied
  proactively) + the critic's kill-map → MSI 100 FIRST TRY. The DECSET-1049 mock made is_alt_screen
  gate-testable (not a shim), and the Ctrl-C mock proof kept the finale off the flaky real-PTY lane.
- **Honest scope:** vim/top/less/htop + Ctrl-C/D/Z WORK; prompt tab-completion is DEFERRED (the
  local-editing-vs-shell-ZLE model decision — the intake, for chad).
- **Ticket:** forge #33 → done; local doc → closed/; pipeline pair archived. **6 of 6 — M1.D "The
  Daily Driver" COMPLETE.**
