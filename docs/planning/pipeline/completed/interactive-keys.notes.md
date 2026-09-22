# full interactive key coverage — Notes

- **Forge ticket:** #41 `acf70690-53e8-45ab-a9ad-3e86598d51a4`
- **AAR:** `a5c302fd-1892-41ed-9006-4efced4cb324`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-041-interactive-keys.md
- **Pipeline spec:** interactive-keys.spec.md

## Phase 1 — Plan
- **Request:** forge #41 (M1.F "Real Interactivity" seq-2, auto-approved) — the full key vocabulary
  for interactive programs.
- **Classification / tier:** work pipeline, `feature`, a PURE encoder extension (keys.rs, cov/MSI
  100) + a SHIM key-name mapping. terminal_blocks + marley_app.
- **Discovery (§18):**
  - `encode_key` (keys.rs:72) matches `KeyCode` → bytes; `KeyCode` (11 variants) + `KeyInput{code,
    ctrl, alt}` (44). Tested: `encode_named_keys` (goldens, the #33 pattern).
  - `key_input_from_keystroke` (app.rs:278, `mutants::skip`) maps `keystroke.key` strings ("up",
    "tab"…) → `KeyCode` + builds `KeyInput{code, ctrl: modifiers.control, alt: modifiers.alt}` — add
    `shift` + the new key names here.
  - `KeyInput{...}` literals (the `shift` ripple): keys.rs:114 (the `key()` helper), :128/:138 (encode
    tests), session.rs:964 (the ctrl_c test) → each gains `shift: false`; app.rs:314 (the shim) gains
    `shift: keystroke.modifiers.shift`.
  - Deps #40 (routing, done) + #33 (encode_key/KeyCode/KeyInput).
- **Decisions:** D1–D4 in the spec (xterm modifier_param; F1-4 SS3 / F5-12 CSI; shift is a new field
  only for named keys; pure encoder + shim mapping).
- **Open questions for Design:** `modifier_param`/`csi_cursor` as free fns vs inline; where param is
  computed (top of encode_key vs inside csi_cursor — lean top, passed in); the exact F5-12 codes
  (15/17/18/19/20/21/23/24 — the standard non-contiguous set); the F(n) `_`-arm value (empty vec).
- **AAR id:** `a5c302fd-1892-41ed-9006-4efced4cb324`.

## Phase 2 — Design

### PURE — `crates/terminal_blocks/src/keys.rs`
```rust
pub enum KeyCode { /* …existing… */, BackTab, F(u8), Insert }
pub struct KeyInput { pub code: KeyCode, pub ctrl: bool, pub alt: bool, pub shift: bool }

/// The xterm modifier code for a CSI `1;<n>` parameter: 1 = none, then +Shift(1) +Alt(2) +Ctrl(4)
/// (2 = Shift, 5 = Ctrl, 8 = all). Private impl detail of `encode_key`; tested in-crate.
fn modifier_param(shift: bool, alt: bool, ctrl: bool) -> u8 {
    1 + shift as u8 + 2 * alt as u8 + 4 * ctrl as u8
}

/// A cursor key's bytes: `ESC[1;<param><final>` when a modifier is held (param > 1), else the plain
/// legacy `ESC[<final>` (unchanged for existing programs + the #33 goldens).
fn csi_cursor(param: u8, final_byte: u8) -> Vec<u8> {
    if param > 1 {
        let mut v = vec![0x1b, b'[', b'1', b';'];
        v.extend(param.to_string().into_bytes());
        v.push(final_byte);
        v
    } else {
        vec![0x1b, b'[', final_byte]
    }
}

/// The bytes for a function key (F1–4 → SS3 `ESC O …`, F5–12 → CSI `ESC[<code>~` with the standard
/// non-contiguous codes); an out-of-range `n` → empty (a safe no-op).
fn encode_fkey(n: u8) -> Vec<u8> {
    match n {
        1 => vec![0x1b, b'O', b'P'], 2 => vec![0x1b, b'O', b'Q'],
        3 => vec![0x1b, b'O', b'R'], 4 => vec![0x1b, b'O', b'S'],
        5 => vec![0x1b, b'[', b'1', b'5', b'~'], 6 => vec![0x1b, b'[', b'1', b'7', b'~'],
        7 => vec![0x1b, b'[', b'1', b'8', b'~'], 8 => vec![0x1b, b'[', b'1', b'9', b'~'],
        9 => vec![0x1b, b'[', b'2', b'0', b'~'], 10 => vec![0x1b, b'[', b'2', b'1', b'~'],
        11 => vec![0x1b, b'[', b'2', b'3', b'~'], 12 => vec![0x1b, b'[', b'2', b'4', b'~'],
        _ => vec![],
    }
}
```
`encode_key` adds — `BackTab => vec![0x1b, b'[', b'Z']`, `Insert => vec![0x1b, b'[', b'2', b'~']`,
`F(n) => encode_fkey(n)`, and routes the six cursor keys through
`csi_cursor(modifier_param(input.shift, input.alt, input.ctrl), b'A'/'B'/'C'/'D'/'H'/'F')` (replacing
their fixed `vec![0x1b, b'[', b'X']`). The `Char`/`Enter`/`Backspace`/`Tab`/`Escape`/`PageUp`/
`PageDown`/`Delete` arms are unchanged (shift on a `Char` is already resolved into the character).

### SHIM — `crates/marley_app/src/app.rs` (`key_input_from_keystroke`, mutants::skip)
- `KeyInput { …, shift: keystroke.modifiers.shift }`.
- match arms: `"tab"` → `if keystroke.modifiers.shift { KeyCode::BackTab } else { KeyCode::Tab }`;
  `"insert"` → `KeyCode::Insert`; a leading-`f` numeric key (`"f1"…"f12"`) → `KeyCode::F(n)` (parse
  `key[1..]`).

### File manifest
- M `crates/terminal_blocks/src/keys.rs` — KeyCode/KeyInput + the 3 private fns + encode_key arms +
  the `shift: false` ripple in the `key()` helper + the 2 explicit test `KeyInput` literals + new tests.
- M `crates/terminal_blocks/src/session.rs` — the `ctrl_c` test `KeyInput` literal + `shift: false`.
- M `crates/marley_app/src/app.rs` — the shim mapping + `shift`.
- M `docs/specs/SPEC-terminal-blocks.spec.md` — R25 (the key set) + mutation targets. CHANGELOG; arch.

### Mutation Targets
- `modifier_param` — the `1 +`, the `shift`/`2*alt`/`4*ctrl` coefficients (the powers of two keep the
  modifiers independent — no colliding combo). Killed by the 8-case truth table.
- `csi_cursor` — the `param > 1` branch (`>=`/`<`/`==`) + the byte assembly. Killed by plain (param 1
  → `ESC[A`) + modified (param 2 → `ESC[1;2A`) via `encode_key`.
- `encode_fkey` — each of the 12 arms + the `_` arm. Killed by the table-driven per-F golden + `F(0)`/
  `F(13)`→empty.
- `BackTab`/`Insert` arms — goldens.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `encode_backtab_and_insert` (`BackTab→ESC[Z`, `Insert→ESC[2~`); `encode_function_keys` — a table `[(1,ESC O P)…(12,ESC[24~)]` asserts each; `F(0)`+`F(13)`→`vec![]` (the `_` arm) | unit (keys.rs) |
| REQ-002 | `modifier_param_cases` — the 8: none→1, S→2, A→3, S+A→4, C→5, S+C→6, A+C→7, all→8 | unit |
| REQ-003 | `encode_modified_cursor` — `Shift-Up→ESC[1;2A`, `Ctrl-Right→ESC[1;5C`, `Alt-Left→ESC[1;3D`, `Shift-Home→ESC[1;2H`; plain `Up→ESC[A` (param 1 branch) | unit |
| REQ-004 | `scripts/gates.sh --diff` GREEN + receipt | gate |

Uncoverable: the app.rs `key_input_from_keystroke` gpui mapping (mutants::skip + cov-excluded; needs
live keystrokes).

### Risks / decisions
- D-2.1 `modifier_param`/`csi_cursor`/`encode_fkey` PRIVATE (impl details) — tested directly in-crate
  (modifier_param) or via `encode_key` (the others); no API surface added, `encode_key` stays the
  entry point.
- D-2.2 Normal-cursor-mode CSI only (no DECCKM app-cursor-mode `ESC O A` toggle) — the plain `ESC[A`
  form is what most programs accept; app-mode is deferred (spec Out).
- D-2.3 `KeyInput.shift` is additive; for `Char` it's ignored (the char is pre-shifted) — no change to
  the Char/Ctrl/Alt arms, so #33's `encode_char_and_control` stays green. The `shift: false` ripple to
  the 4 test literals keeps them compiling; the plain-cursor goldens in `encode_named_keys` still hold
  (param 1 → plain form).
- No equivalent-mutant risk on `modifier_param` (powers of two → the 8-case table pins every output).

## Phase 3 — Implement
- **Built (per manifest):** keys.rs — `KeyCode` +`BackTab`/`F(u8)`/`Insert`; `KeyInput` +`shift`;
  private `modifier_param` (1+shift+2·alt+4·ctrl), `csi_cursor` (param>1 → parameterized), `encode_fkey`
  (F1-4 SS3 / F5-12 CSI / _-empty); `encode_key` adds the 3 arms + routes the 6 cursor keys through
  `csi_cursor(modifier_param(...))`. The `shift: false` ripple: the keys.rs `key()` helper + 2 explicit
  test literals + session.rs `ctrl_c` literal. app.rs shim (`key_input_from_keystroke`) — +`shift`,
  `tab`+shift→BackTab, `insert`→Insert, `f1..f12`→`F(n)`. SPEC-terminal-blocks R25 + mutation targets;
  CHANGELOG (### Added).
- **Deviations from design:** (1) removed a premature `key_mod` test helper — it was unused until the
  Phase-4 tests → dead_code/clippy fail; Phase 4 re-adds it with its callers. (2) the shim F-key range
  used `(1..=12).contains(&n)` (clippy `manual_range_contains`), not `n>=1 && n<=12`.
- **Verification at this phase:** `cargo check --workspace` 0 errors; fmt; clippy `-D warnings` 0
  (terminal + marley); docs gate 0; 88 terminal tests pass (existing goldens green — plain cursors
  unchanged). The new-arm + modifier goldens are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (43 independent xterm/VT220 byte goldens via a verbatim probe + a real scoped
  cargo-mutants [isolated target dir] + the real crate's existing tests). Verdict: **PASS/SHIP — no
  HIGH/MED; correctness verified against the spec, not just the code.**
- **Byte goldens ALL CORRECT vs xterm:** BackTab `1b5b5a` (CSI Z), Insert `1b5b327e` (CSI 2~), F1–4
  SS3 `1b4f50..53`, F5–12 codes 15/17/18/19/20/21/23/24 (gaps at 16/22 correct), modified cursors
  `CSI 1;<mod><final>` (Shift-Up `1b[1;2A`, Ctrl-Right `1b[1;5C`). No wrong sequence.
- **modifier_param NO equivalent mutant:** the `1 + shift + 2·alt + 4·ctrl` coefficients are a binary
  place-value BIJECTION {0,1}³→{0..7}, so `+1`→{1..8} are all distinct — the 8-case truth table pins
  every output, killing all 12 mutants (incl. `2*alt`→`2+alt`, `*`→`/`). `csi_cursor` `param>1`
  boundary killed by plain(1)+modified(2). `encode_fkey` — each of the 12 arms needs its own golden.
- **NO #33 regression:** plain cursors (param=1 → `csi_cursor(1,x)` → `ESC[x`) are BYTE-IDENTICAL to
  the old fixed bytes; the real crate's `encode_named_keys` + `encode_char_and_control` pass unmodified
  with the diff. Char arms ignore `shift` (pre-shifted). No panic.
- **Findings table:**
  | # | Sev | Finding | Verdict | Carry-forward |
  |---|---|---|---|---|
  | F1 | LOW (Phase-4) | The new goldens aren't in the diff yet (scoped mutants: 33/33 KILLABLE with the planned fixtures, but 0 present now). | REAL (the Phase-4 deliverable) | P4 lands: the 8-case `modifier_param` table; all 6 cursor finals plain + modified; the 12 per-F goldens; BackTab + Insert; and an out-of-range `F(0)`/`F(13)`→empty (needed for the `_`-arm LINE COVERAGE, though the catch-all isn't mutation-load-bearing). |
  | F2 | LOW | The shim `k[1..].parse::<u8>()` accepts a leading `+` (`"f+2"`→F(2)) — UNREACHABLE (gpui f-key names never contain `+`); `k[1..]` is panic-safe (`'f'` is ASCII). | ACCEPTED (unreachable) | none. |
- **No code fix** — correctness is spec-verified. F1 is the Phase-4 test set.

## Phase 4 — Validate
- **Tests added (keys.rs):** `encode_backtab_and_insert` (goldens); `encode_function_keys` (a
  table asserting each F1–12 + `F(0)`/`F(13)`→empty for the `_`-arm coverage); `modifier_param_cases`
  (the 8-case binary-bijection table); `encode_modified_cursor` (Shift-Up/Ctrl-Right/Alt-Left/
  Shift-Home parameterized + plain Up → legacy, the #33-regression guard) + the `key_mod` helper
  re-added with its callers.
- **Runs (actual):** `cargo nextest run -p marley_terminal` → 92 passed (all 4 new PASS);
  `cargo nextest run --workspace` → 512 passed, 5 skipped.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, mutation **36 caught /
  0 missed → MSI 100.0%** (modifier_param 12 + csi_cursor + encode_fkey 12 arms + the new encode_key
  arms). Receipt written. No PTY hang.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added` (at implement); `terminal_blocks.md` — the encode_key
  key-set extension bullet. SPEC-terminal-blocks R25 at implement.
- **Knowledge captured:** no new prevention rule — the critic verified all byte sequences against
  xterm + confirmed `modifier_param`'s binary-bijection coefficients have no equivalent mutant (the
  same "independent operands → truth table pins all" property as #40's input_route OR). aar-submit
  `completed` (score 5). Win: the critic checked goldens against the SPEC (terminfo/xterm), not just
  the code — catching a wrong F-key code would've been a real user-felt bug; all correct first try.
- **Ticket:** forge #41 → done; local doc → closed/; pipeline pair archived. 2 of 3 in M1.F.
