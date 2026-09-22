# Honor DECCKM (application cursor keys) — Notes

- **Forge ticket:** #286 `bf9c4440-69b5-4249-9cbb-09bbae161d4a`
- **AAR:** `7daf1868-02b1-4827-8b7e-6ff41f3f83c6`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-286-decckm-app-cursor-keys.md
- **Pipeline spec:** 286-decckm-app-cursor-keys.spec.md

## Phase 1 — Plan
- **Request:** honor DECCKM — emit SS3 (`ESC O X`) instead of CSI (`ESC [ X`) for
  unmodified arrows/Home/End AND the #280 alt-scroll wheel fallback when
  `TermMode::APP_CURSOR` is active. Bug from #280 inspect F2.
- **Classification / tier:** work pipeline, one shippable slice, `bug`. Crates:
  `terminal_blocks` (keys.rs, mouse.rs, session.rs) + a one-line `marley_app`
  (app.rs) shim feed. No new UI surface (a wire-protocol correctness fix), but it
  IS input-path so Validate drives the live app (arrow into vim → SS3).
- **Forge recall (§18.3):** the ticket carries the grounding (verified zero
  APP_CURSOR consumers outside alacritty). #280 (mouse.rs) is the sibling — its
  header already cites the xterm ctlseqs spec as the §20 reference. The
  `paste_bytes(text, bracketed)` precedent (terminal-mode-as-param) settles D2.
- **Discovery (Design surface), grounded this phase:**
  - `keys.rs:81` `encode_key(input)` → the `cursor` closure (83-88) → `csi_cursor(param, final)` (125): `param > 1` → CSI-modified; else → plain CSI `ESC[<final>`. The six cursor keys (Up/Down/Right/Left/Home/End, lines 104-109) all route here.
  - `mouse.rs:9` `MouseModes` (click/drag/motion/sgr/alt_scroll/alt_screen); the fallback `mouse_report` (85-90) emits `ESC[A`/`ESC[B` for WheelUp/Down under `alt_screen && alt_scroll`.
  - `session.rs:191` `mouse_modes()` builds `MouseModes` from `TermMode` flags; `is_alt_screen`/`is_bracketed_paste` (171/185) are the single-bool accessor idiom to mirror for `is_app_cursor`.
  - `app.rs:6379` `write_bytes(&encode_key(input))` — the sole raw-key call site; `input` built at `app.rs:1836` (`KeyInput{code,ctrl,alt,shift}`). The 4 `mouse_report` sites (7333/7447/7493/7539) all read `modes = session.mouse_modes()` — so `MouseModes.app_cursor` flows for free.
  - `TermMode::APP_CURSOR` = `1 << 1` (alacritty 0.26.0), toggled by `NamedPrivateMode::CursorKeys` (DECSET/DECRST 1). Confirmed present.
- **Decisions:** D1–D4 in the spec. Crux: SS3 ONLY for unmodified (param==1) — modified cursor keys stay CSI even under DECCKM (xterm rule); the shared `cursor_key_bytes` puts the SS3-vs-CSI rule in one tested place used by both consumers.
- **Risk:** low. The change is additive (a new param defaulting behavior to the existing CSI when `app_cursor=false`); every existing golden stays green with `app_cursor=false`. Blast radius = ~20 mechanical `encode_key(x)` → `encode_key(x, false)` test edits.

## Phase 2 — Design

### Approach
Thread `TermMode::APP_CURSOR` (a single bool) from the session snapshot into the
two pure encoders that emit cursor-key bytes. The SS3-vs-CSI decision for ONE
unmodified cursor key lives in a single shared pure helper both consumers call, so
the xterm rule is expressed once. Fully additive: with `app_cursor = false` every
existing byte sequence is unchanged (all current goldens stay green).

**xterm rule (D1):** under DECCKM, an UNMODIFIED cursor key switches CSI→SS3
(`ESC[A` → `ESC O A`); a MODIFIED cursor key stays CSI (`ESC[1;<param>A`). So only
`csi_cursor`'s `param == 1` branch consults `app_cursor`; the `param > 1` branch
is byte-for-byte unchanged.

§20 confirmed N/A (xterm ctlseqs wire-protocol compliance, public spec, no
Warp/Zed UX analog) — reimplemented from the public spec exactly like #280's mouse
encoding; no copyleft source read.

### File manifest
| File | Change |
|------|--------|
| `crates/terminal_blocks/src/keys.rs` | ADD pure `pub(crate) fn cursor_key_bytes(final_byte: u8, app_cursor: bool) -> Vec<u8>` → `ESC O <final>` when `app_cursor` else `ESC [ <final>`. `csi_cursor(param, final)` → `csi_cursor(param, final, app_cursor)`: `param > 1` unchanged (CSI-modified); else `cursor_key_bytes(final, app_cursor)`. `encode_key(input)` → `encode_key(input, app_cursor: bool)`; the `cursor` closure passes `app_cursor`. |
| `crates/terminal_blocks/src/mouse.rs` | ADD `MouseModes.app_cursor: bool` (doc: DECSET 1 / DECCKM). The `mouse_report` alt-scroll fallback → `WheelUp => Some(crate::keys::cursor_key_bytes(b'A', modes.app_cursor))`, `WheelDown => …(b'B', …)`. |
| `crates/terminal_blocks/src/session.rs` | `mouse_modes()` gains `app_cursor: mode.contains(TermMode::APP_CURSOR)`. ADD `is_app_cursor()` (mirrors `is_alt_screen`). |
| `crates/marley_app/src/app.rs` | The sole raw-key call site (`~6379`): `encode_key(input)` → `encode_key(input, <session>.is_app_cursor())`. |

### Regression Test Plan
| # | Test (crate) | Proves |
|---|---|---|
| T1 | keys.rs `cursor_key_bytes` truth-table: `(b'A', false) == [1b,5b,41]`; `(b'A', true) == [1b,4f,41]`; same for a second final byte. | REQ-006 shared helper |
| T2 | keys.rs golden: `encode_key(key(K), true)` for K ∈ {Up,Down,Right,Left,Home,End} → SS3 `[1b,4f,{41,42,43,44,48,46}]`. | REQ-001 all six |
| T3 | keys.rs golden: `encode_key(key(Up), false) == [1b,5b,41]` (+ the existing goldens updated to `, false`). | REQ-002 legacy CSI |
| T4 | keys.rs golden: `encode_key(key_mod(Up, ctrl), true) == [1b,5b,31,3b,35,41]` (CSI modified, app_cursor IGNORED). | REQ-003 modified stays CSI |
| T5 | mouse.rs: alt-scroll fallback with `app_cursor:true` → `ESC O A`/`ESC O B`; `false` → `ESC [ A`/`ESC [ B`. | REQ-004 fallback |
| T6 | session.rs: a `MockPtyChannel` feeds `ESC[?1h` (DECSET 1) → `is_app_cursor()` && `mouse_modes().app_cursor` true; `ESC[?1l` → false. | REQ-005 round-trip |
| — | cov/MSI 100 on `cursor_key_bytes` + `csi_cursor` + the fallback via `--diff`. | REQ-006 |

Uncoverable: none. The app.rs one-line feed is a `#[cfg_attr(test, mutants::skip)]`-class shim (reads live `is_app_cursor()`), covered behaviorally by the Validate live-drive (arrow into vim → SS3 on the wire) — but the wire correctness itself is unit-proven in keys.rs/mouse.rs.

### Risks / decisions
- **R1:** the ~20 existing `encode_key(x)` test calls must gain `, false` (mechanical; a missed one fails to compile, not silently). No behavior risk — `app_cursor=false` reproduces today's bytes exactly.
- **D-reuse:** `cursor_key_bytes` is `pub(crate)` in keys.rs and imported by mouse.rs — the single home for the SS3-vs-CSI spec rule (avoids a 2-line duplicate that could drift).

## Phase 3 — Implement
- **Built to the manifest, no design deviations.**
  - `keys.rs`: `encode_key(input, app_cursor)` (the `cursor` closure passes it); `csi_cursor(param, final, app_cursor)` — `param > 1` unchanged, else `cursor_key_bytes`; new `pub(crate) cursor_key_bytes(final, app_cursor)` (SS3 vs CSI).
  - `mouse.rs`: `MouseModes.app_cursor` field; the alt-scroll fallback → `cursor_key_bytes(b'A'/b'B', modes.app_cursor)`; the `mouse_report` doc updated.
  - `session.rs`: `mouse_modes().app_cursor = mode.contains(TermMode::APP_CURSOR)`; new `is_app_cursor()` (mirrors `is_alt_screen`).
  - `app.rs`: the raw-key call site computes `let app_cursor = state.session.is_app_cursor();` before the `&mut` write, then `encode_key(input, app_cursor)`.
- **Compile-fix (needed for the signature change, not new behavior tests):** the ~27 existing `encode_key(x)` goldens all test the legacy (app_cursor-off) form, so a `fn enc(input) = encode_key(input, false)` test helper was added and the existing calls bulk-renamed to `enc(` (scoped to the test module; the `pub fn` def untouched) — byte-identical results. `all()` (lists all `MouseModes` fields) + the session Ctrl-C test got `app_cursor: false` / `, false`; the other `MouseModes` test literals use `..Default::default()` so were unaffected.
- `cargo check --workspace --all-targets` 0 errors; `cargo fmt` clean; `cargo nextest run -p marley_terminal` → 117 passed (existing goldens byte-unchanged).
- **[out-of-band] While this pipeline was open, the #285 background equivalence critic completed and found a CONFIRMED critical regression in the shipped #285 (pure-deletion tail-trim drops a comment's highlight — `changed_ranges` is not a superset of stale spans for SHRINKING edits). Filed forge #286→ wait, forge #288 `ce37cfba` (sprint #30) with the repro + fix (grow the window over cached spans overlapping the edit). Slated as the TOP-priority next pipeline after this one.**

## Phase 3.5 — Inspect
Inline adversarial trace (mechanical spec-compliance change — no subtle invariant like #285's equivalence; a long-running critic isn't warranted). **No defects.** Angles:

| Angle | Verdict | Evidence |
|---|---|---|
| **SS3-only-unmodified (D1)** | **SAFE** | `csi_cursor`'s `param > 1` branch is byte-for-byte UNCHANGED (never consults `app_cursor`) → a MODIFIED cursor key stays CSI (`ESC[1;<param>X`) under DECCKM, per xterm. Only `param == 1` routes to `cursor_key_bytes`. |
| **All six cursor keys** | **SAFE** | Up/Down/Right/Left/Home/End all go through the `cursor` closure → `csi_cursor` → SS3 when unmodified+app_cursor. |
| **Non-cursor keys unaffected** | **SAFE** | PageUp/PageDown/Delete/Insert use hardcoded `ESC[…~` (NOT cursor keys) — DECCKM does not touch them, and the code doesn't either. |
| **Mouse fallback** | **SAFE** | `cursor_key_bytes(b'A'/b'B', modes.app_cursor)`, still gated by `alt_screen && alt_scroll` (unchanged). |
| **Session mode source** | **SAFE** | `mouse_modes().app_cursor` and `is_app_cursor()` both read `TermMode::APP_CURSOR` — one source, consistent. |
| **app.rs borrow + correct child** | **SAFE** | `let app_cursor = state.session.is_app_cursor();` computed before the `&mut write_bytes` (no two-phase reliance); `state` is the FOCUSED terminal, so the mode reflects the running child. |
| **`enc`/sed rename** | **SAFE** | Only the `pub fn encode_key` def remains un-renamed (verified `grep`); the `enc` helper body (`encode_key(input, false)`) was added AFTER the sed → no self-recursion; 117 marley_terminal tests pass byte-identical. |
| **MSI** | **SAFE (Phase-4-gated)** | `cursor_key_bytes`'s `if app_cursor` killed by both-mode goldens (T1/T2/T3); `csi_cursor`'s `param > 1` killed by param==1 (unmodified) vs param>1 (modified) cases (T4). Byte literals unmutated. |

No `failure-record` (no bug). No new prevention rule (mechanical change). Fully additive: `app_cursor=false` reproduces every pre-#286 byte.

## Phase 4 — Validate
- **Tests added (5):** keys.rs `cursor_key_bytes_ss3_vs_csi` (T1), `app_cursor_unmodified_arrows_are_ss3` (T2+REQ-002, all six keys × both modes), `app_cursor_modified_arrows_stay_csi` (T4/REQ-003); mouse.rs `alt_scroll_fallback_honors_app_cursor` (T5, SS3 vs CSI); session.rs `app_cursor_tracks_decckm` (T6, `MockPtyChannel` DECSET 1 → `is_app_cursor()` + `mouse_modes().app_cursor`, before/after the pump covering the flag both ways). REQ-002 legacy-CSI is also carried by the pre-existing `enc()` goldens (unchanged bytes).
- **`cargo nextest run -p marley_terminal`: 122 passed, 0 skipped** — the 5 new tests + all 117 existing (byte-identical under `app_cursor=false`).
- **`scripts/gates.sh --diff` → `GATE GREEN [diff]`** — 15/15 incl. **coverage ≥100% lines** + **mutation MSI ≥100%** (the `cursor_key_bytes` `if app_cursor`, `csi_cursor` `param > 1` boundary, and `is_app_cursor()`/`mouse_modes().app_cursor` all killed by the both-mode goldens + the round-trip), clippy -D, brand-scrub, miri, visual/AX.
- **Live-drive: N/A for verification (documented, not silently skipped).** This is a WIRE-PROTOCOL change with zero distinguishable visual behavior — SS3 (`ESC O A`) and CSI (`ESC [ A`) move a vim/less cursor identically (the ticket's own premise: "most TUIs parse both forms"). A screen capture cannot observe the byte form; distinguishing it needs a byte-echoing program in DECCKM mode driven by synthetic keys (Accessibility-gated, env-blocked on this machine per the #204/#205 precedent). The wire bytes are exhaustively unit-proven and the app.rs feed is a 1-line accessor read reusing the already-proven `state.session` write path (every terminal interaction exercises it). gate:15 passed via the terminal-library asserts.
- No pre-existing failures.

## Phase 5 — Complete
- **Docs:** CHANGELOG.md ### Fixed (#286); docs/marley_architecture/terminal_blocks.md keys.rs section gained the M17 #286 DECCKM note (SS3 vs CSI, `cursor_key_bytes`, the `paste_bytes` precedent, `is_app_cursor`).
- **Knowledge (forge):** AAR `7daf1868` submitted — completed, effectiveness 5. No failure-record (clean first-try, no bug). No new prevention rule (mechanical spec-compliance with a clear existing precedent). Lesson: the `paste_bytes(text, bracketed)` precedent settled the terminal-state-as-param decision instantly; the `enc()` legacy helper + a test-module-scoped sed rename handled the ~27 signature-change test-call sites in one pass (byte-identical, 117 tests stayed green).
- **Ticket** TICKET-286 → closed/ (status closed) + forge ticket-close. **Pipeline** spec+notes → completed/.
- **Result:** DECCKM honored (SS3 for unmodified arrows/Home/End + the alt-scroll fallback; modified stays CSI); wire-only, byte-identical when off; cov/MSI 100; GATE GREEN [diff]. LOCAL commit only (push un-OK'd).
- **NOTE:** the #285 critical regression (#288, `ce37cfba`) surfaced mid-pipeline remains the TOP-priority next work — the shrinking-edit windowing fix, before #287.
