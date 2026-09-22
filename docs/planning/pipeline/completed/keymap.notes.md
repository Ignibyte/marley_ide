---
pipeline_id: 247ca15e-e6dc-4ad0-b3f1-fa3551357cc3
aar_id: f9a6f321-56a2-402c-a78e-2fa85bd209bd
---

# app_shell keymap — pipeline notes

## Phase 1 — Plan (2026-07-01)

**Intent:** the SPEC-app-shell keymap layer (R19-R20) in marley_app — a pure `Keymap` (binding→action) +
the gpui chord interception. M1.B Cockpit seq 2/5; the enabler for the palette (#19) + pane (#20) chords.

**Carry to Design:**
- **keymap.rs (PURE):**
  - `KeyBinding { cmd: bool, ctrl: bool, alt: bool, shift: bool, key: String }` — derive
    Debug/Clone/PartialEq/Eq (PartialEq drives `action_for`'s lookup + is the mutation target). A helper
    ctor (e.g. `KeyBinding::chord(cmd, ctrl, alt, shift, key)`) or plain struct literals in tests.
  - `Keymap { bindings: Vec<(KeyBinding, String)> }` — private field; built via `default_bindings()`.
  - `default_bindings() -> Self`: the 3 mappings (cmd-shift-p→"open-command-palette", cmd-d→"split-pane",
    cmd-w→"close-pane"). key is the lowercase base ("p"/"d"/"w"); cmd=true; shift=true only for the palette.
  - `action_for(&self, binding: &KeyBinding) -> Option<&str>`:
    `self.bindings.iter().find(|(b, _)| b == binding).map(|(_, a)| a.as_str())`.
- **app.rs (SHIM, already mutants::skip + cov-excluded):** add `keymap: Keymap` to RootView (init
  `Keymap::default_bindings()` in `new`); `binding_from_keystroke(&Keystroke) -> KeyBinding` (mods from
  `keystroke.modifiers` {platform→cmd, control, alt, shift}, key from `keystroke.key`); in `on_key_down`,
  BEFORE `key_from_keystroke`/`apply_key`: `let b = binding_from_keystroke(&event.keystroke); if
  view.keymap.action_for(&b).is_some() { /* bound chord — intercept; #19/#20 dispatch */ } else { …the
  existing prompt path… }`.
- **lib.rs:** `mod keymap;` + `pub use keymap::{KeyBinding, Keymap};`.
- **Mutation map:** `action_for` — the `find` predicate `b == binding` (a wrong-eq or always-None mutant
  killed by R19 asserting each of the 3 Some(action) + R20 asserting None for an unbound chord);
  `default_bindings` — each of the 3 (binding, action) pairs (a perturbed action string / binding killed
  by R19's exact-mapping asserts). No .len()-style ops → the exact-value asserts are the guard.
- **Tests (validate):** `default_keymap_maps_named_chords` (the 3 Some asserts) + `action_for_unbound_is_none`
  (a chord absent from the map, e.g. cmd-x or a bare "a", → None). Fixtures: KeyBinding literals.

**Phase 1 status:** PASS (autonomous). → Phase 2 Design.

## Phase 2 — Design (2026-07-01)

**Confirmed the Carry-to-Design against SPEC-app-shell's Public surface — MATCHES.** The spec pins
`pub struct KeyBinding { /* mods + key */ }`, `pub struct Keymap { /* binding -> action name */ }`,
`Keymap::default_bindings() -> Self`, `action_for(&self, binding: &KeyBinding) -> Option<&str>`. The
`KeyBinding { cmd, ctrl, alt, shift: bool, key: String }` modeling realises `/* mods + key */` (a free
design choice — the spec left the fields open); it must derive `PartialEq`/`Eq` (drives `action_for`'s
`find` + is THE mutation target) + `Debug`/`Clone`. `Keymap { bindings: Vec<(KeyBinding, String)> }`
(private field) realises `/* binding -> action name */`.

- **Seam CONFIRMED:** keymap.rs (KeyBinding + Keymap + default_bindings + action_for) is gpui-free PURE →
  cov 100 / MSI 100. app.rs (the `keymap` field + `binding_from_keystroke` + the `on_key_down`
  interception) is the shim — app.rs is ALREADY `mutants::skip` + rust_cov-excluded (from #16), so NO
  gates.sh change is needed.
- **Mutation map CONFIRMED (covers R19 + R20):** `action_for`'s `b == binding` predicate + `.map(as_str)`
  — killed by R19 asserting each of the 3 `Some(action)` + R20 asserting `None` for an unbound chord.
  `default_bindings`'s 3 `(binding, action)` pairs — killed by R19's exact-mapping asserts (a perturbed
  action string or a swapped binding fails an assert). `default_bindings -> Self` whole-fn
  `Default::default()` mutant is UNVIABLE (Keymap derives no Default). No count/len ops → exact-value
  asserts are the sole guard (correct for this shape).
- **Test plan (validate):** `default_keymap_maps_named_chords` (R19 — the 3 Some asserts) +
  `action_for_unbound_is_none` (R20 — a chord absent from the map → None). Both plain `#[test]` with
  `KeyBinding` struct literals (zero gpui). No uncoverable path in keymap.rs; the shim wiring is proven by
  the existing `#[ignore]` headed_shell test (the window still renders + accepts keys).

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement (2026-07-01)

Built (in-context, no subagent):
- **crates/marley_app/src/keymap.rs (PURE, gpui-free):** `KeyBinding { cmd, ctrl, alt, shift: bool, key:
  String }` (+ `KeyBinding::chord(..)` ctor) + `Keymap { bindings: Vec<(KeyBinding, String)> }` +
  `default_bindings()` (the 3 R19 mappings) + `action_for(&KeyBinding) -> Option<&str>` (find-by-eq).
- **crates/marley_app/src/lib.rs:** `mod keymap;` + `pub use keymap::{KeyBinding, Keymap};`.
- **crates/marley_app/src/app.rs (SHIM — already mutants::skip + cov-excluded):** a `keymap: Keymap`
  field on RootView (init `Keymap::default_bindings()` in `new`); `binding_from_keystroke(&Keystroke) ->
  KeyBinding` (cmd=modifiers.platform, ctrl=.control, alt=.alt, shift=.shift, key=keystroke.key.clone());
  the `on_key_down` interception — `if view.keymap.action_for(&binding).is_some() { cx.notify(); return; }`
  BEFORE the prompt path (a bound chord is kept off the terminal prompt; #19/#20 dispatch the action).

**No deviations** from the Phase 2 design. `cargo check` + `clippy -D` + `fmt --check` + `rustdoc -D` +
no-`unsafe` all clean. keymap.rs is gpui-free (the only "gpui" is a doc-comment mention). The gpui
`Modifiers` fields (platform/control/alt/shift) matched (key_from_keystroke already uses .control/.platform).

**Carry to Validate:** the 2 unit tests (`default_keymap_maps_named_chords` R19 — the 3 Some asserts via
`KeyBinding::chord(true,false,false,true,"p")` etc.; `action_for_unbound_is_none` R20 — e.g.
`chord(true,false,false,false,"x")` → None). keymap.rs cov 100/MSI 100; app.rs shim excluded.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5) — 2026-07-01

Inspected in-context (70-line pure file; `cargo mutants --list -p marley` is the oracle). **No findings.**
Lenses: mutant surface, seam, clean-room (§20), §14 totality.

- **Seam CONFIRMED:** `cargo mutants --list` shows **0 app.rs mutants** (binding_from_keystroke + the
  field/interception are covered by app.rs's existing `mutants::skip` + cov-exclude) — the ONLY listed
  mutants are the 6 in keymap.rs.
- **Clean-room:** 0 `warp` refs in keymap.rs; `KeyBinding`/`Keymap`/`chord`/`default_bindings`/`action_for`
  Marley-original. **§14:** no unwrap/expect/panic (find/map are total).

**CARRY TO VALIDATE — the mutation kill map (6 mutants):**
- `chord -> Default::default()` (L26) + `default_bindings -> Default::default()` (L45) — **UNVIABLE**
  (KeyBinding/Keymap derive no `Default`).
- `action_for -> None` (L65) — killed by **R19** (a bound chord asserts `Some(action)` ≠ None).
- `action_for -> Some("")` (L65) — killed by **R19** asserting the EXACT action (≠ `Some("")`).
- `action_for -> Some("xyzzy")` (L65) — killed by **R19** exact (≠ `Some("xyzzy")`).
- `== -> !=` in the find predicate (L67) — killed by **R19** (with `!=`, `action_for(cmd-shift-p)` skips
  its own entry + returns the next → wrong action) AND **R20** (with `!=`, an unbound chord matches the
  first entry → `Some` instead of `None`).
- `default_bindings`' 3 mappings aren't individually mutated (no per-vec-element mutant) — verified
  through `action_for` by R19's three exact asserts (+ coverage runs `default_bindings`).

**Tests for validate:** `default_keymap_maps_named_chords` (R19 — assert `action_for` = `Some(
"open-command-palette")` / `Some("split-pane")` / `Some("close-pane")` for the 3 chords) +
`action_for_unbound_is_none` (R20 — e.g. `chord(true,false,false,false,"x")` → `None`). Fixtures:
`KeyBinding::chord(..)`.

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate (2026-07-01)

Wrote the 2 tests in-context (per the kill map). `crates/marley_app/src/keymap.rs` gained a `#[cfg(test)]
mod tests`: `default_keymap_maps_named_chords` (R19 — the 3 exact `Some(action)` asserts) +
`action_for_unbound_is_none` (R20 — cmd-x + a bare "p" both → `None`).

`cargo nextest run -p marley` = **20 passed** (the 2 new + the existing marley suite; 1 skipped = the
`#[ignore]` headed_shell; the `#[serial]` real-zsh integration ran green).

**FULL gate (scripts/gates.sh --diff): `GATE GREEN [diff]` — 15/15 on the FIRST run**: cov 100% (keymap.rs;
app.rs + bin excluded), mutation MSI 100% (the 4 viable action_for mutants killed, the 2 `Default`
whole-fn mutants unviable), gate-15 PASS (the existing shell_dark headed lane; marley_app dir → the
harness's headless component tests). No gates.sh change (app.rs was already excluded from #16). Receipt
written.

**Phase 4 status:** PASS. → Phase 5 Complete.
