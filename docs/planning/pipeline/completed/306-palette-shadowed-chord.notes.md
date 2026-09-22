# Palette hides a shadowed keycap chip (#306) — Notes

- **Forge ticket:** #306 (d5b09d10-581c-47d4-82c1-ad446dfec2d0)
- **AAR:** 22741ab8-c3bd-4d66-8dee-d896a5d2a530
- **Local ticket doc:** docs/planning/tickets/open/TICKET-306-palette-shadowed-chord.md
- **Pipeline spec:** 306-palette-shadowed-chord.spec.md

## Phase 1 — Plan
- **Request:** #306 (M19 origin, ships M22) — the palette advertises a keycap chord that a context-scoped
  keymap row has shadowed on the focused surface. Fix the stale HINT only (not the flat command list).
- **Classification / tier:** work pipeline; a small pure CONSUMER bug fix (server/ui — the cockpit palette).
  One shippable slice.
- **Forge recall (§18.3):** no bulletins. Ticket found by two critics during #298 inspect (pre-existing
  class, 3 instances: ⌘D, ⌘⌥↑/↓, ⌘⇧L on Editor). knowledge-search surfaced KeyContext prevention rules /
  ADs from #265/#298 (the KeyContext engine) — resolution is already scoped + tested.
- **Discovery (the seam):**
  - Render site: `app.rs:16825-16837` iterates `filter_commands(&self.commands, query)`; the chip is
    `scored.command.binding.as_ref().map(|b| KeyboardShortcut::parse(&b.display()).render(&colors))` — the
    STATIC binding, no resolution.
  - `Command { id, title, keywords, binding: Option<KeyBinding> }` (palette.rs:16).
  - `action_for_command(id) -> Option<&'static str>` (palette.rs:127) — command → verb (up to id 27, `_ => None`).
  - `keymap.action_for(&KeyBinding, &[KeyContext]) -> Option<&str>` (keymap.rs:542) — PURE; deepest-scoped
    binding wins, global ranks 0, off-surface scoped is ineligible.
  - Live stack: `self.shell.active_project().active_tab().key_context()` (used by dispatch at app.rs:14503);
    `TabContent::key_context() -> &'static [KeyContext]` (tabs.rs:144).
  - Keymap: `self.keymap: Keymap` (app.rs:145, `Keymap::default_bindings()` app.rs:1918) — reachable at render.
  - **Feasibility confirmed:** both `self.keymap` and the live stack are in scope at the render site.
- **Decisions:** D1 hide-not-relabel · D2 identity via the action string (the #204 every-row-has-a-verb
  invariant makes `action_for_command` total over bound rows) · D3 consumer-only (no resolver/list change) ·
  D4 the shim sources the live stack from the dispatch source, computed once before the loop.
- **Prior-art (§20):** the resolver already EXISTS and is pure (`action_for` + `action_for_command`); #306 =
  a 3-line pure composer + a one-line render swap. No gpui adoption needed (we own the equivalent). Behavior
  ref = Zed/VS Code palette resolves chords against the active context (behavior only, no source).

## Phase 2 — Design
**Architecture / approach.** Consumer-only fix in the cockpit palette layer. All logic lands in ONE pure
fn; the app.rs render is shim glue. §14 honored: pure fn, no IO, no panics (total over its inputs).

`crates/marley_app/src/palette.rs` (pure):
```rust
use crate::keymap::{KeyBinding, Keymap, KeyContext};   // add Keymap, KeyContext to the existing import

/// The chord to DISPLAY for `command` on the surface described by `stack` (#306): its bound chord iff
/// that chord still resolves to THIS command there, else `None` (a context-scoped row has shadowed it —
/// advertising it would be a lie). Composes the two existing pure resolvers; no new resolution logic.
pub fn displayed_binding<'a>(
    command: &'a Command,
    keymap: &Keymap,
    stack: &[KeyContext],
) -> Option<&'a KeyBinding> {
    let binding = command.binding.as_ref()?;
    let own = action_for_command(command.id)?;
    keymap
        .action_for(binding, stack)
        .is_some_and(|resolved| resolved == own)
        .then_some(binding)
}
```

`crates/marley_app/src/app.rs` (shim, ~16825–16837): compute the live stack once before the render loop,
then resolve the chip through the pure fn:
```rust
let stack = self.shell.active_project().active_tab().key_context(); // &'static — holds no borrow of self
// … inside the `for (index, scored) in filter_commands(...)` loop:
let shortcut = crate::palette::displayed_binding(scored.command, &self.keymap, stack)
    .map(|binding| KeyboardShortcut::parse(&binding.display()).render(&colors));
```
Both `self.keymap` (app.rs:145) and the stack source (app.rs:14503 dispatch uses the identical call) are in
scope. `key_context()` returns `&'static [KeyContext]` (tabs.rs:144) so `stack` carries no borrow — zero
borrow-checker risk against the existing immutable `self.commands`/`self.palette` reads.

**§20 confirm.** Behavior match unchanged from the spec: Zed/VS-Code palettes resolve advertised chords
against the active KeyContext; Marley reimplements that behavior over its OWN resolver (`action_for`, #265).
No copyleft source read. Holds.

**File manifest.**
- `crates/marley_app/src/palette.rs` — ADD `displayed_binding` + widen the `keymap` import; ADD `#[cfg(test)]`
  unit tests (REQ-001..004 + the no-verb branch).
- `crates/marley_app/src/app.rs` — MODIFY the palette chip render (~16825/16833): source `stack`, call
  `displayed_binding` instead of `scored.command.binding.as_ref()`. One-line logic swap + one new `let`.

**Regression Test Plan (in `palette.rs` `#[cfg(test)]`, using `Keymap::default_bindings()` + real ids/chords;
stacks `TERM=&[Terminal] · ED=&[Editor] · COCKPIT=&[]`, the trio the keymap tests already use):**

| # | Test | Proves | AC |
|---|---|---|---|
| T1 | `displayed_binding(SplitRight{id 7, ⌘⇧L}, default, TERM)` → `Some(⌘⇧L)`; same on COCKPIT → `Some` | unshadowed chord shows (global resolves to split-right) | REQ-001 |
| T2 | `displayed_binding(SplitRight{id 7, ⌘⇧L}, default, ED)` → `None` | shadowed→other (⌘⇧L=select-all-occurrences on Editor) hides | REQ-002 |
| T3 | `displayed_binding(FormatDoc{id 25, ⌥⇧F}, default, TERM)` → `None` | chord unbound on the surface (⌥⇧F is Editor-scoped) hides | REQ-003 |
| T4 | `displayed_binding(Cmd{id 4, binding: None}, default, ED)` → `None` | no static chord → no chip (unchanged) | REQ-004 |
| T5 | `displayed_binding(Cmd{id 999, ⌘⇧L}, default, TERM)` → `None` | a bound command with NO verb (`action_for_command`→None) hides — kills the second `?` branch | mutation |

The render-site swap is shim glue — the pure predicate carries the behavior (cov/MSI 100 on `displayed_binding`).
No `visual_acceptance` clause; confirm the render fn sits in the crate's render-shim mutation treatment at
Implement/Validate (no new headless assertion expected). Real mutant set to be traced with
`cargo mutants --list -f palette.rs` at Validate (don't guess operators — the standing lesson).

**Risks / decisions.** (1) Tests bind to `Keymap::default_bindings()` — REQ-002 depends on #298's Editor
⌘⇧L→select-all-occurrences row continuing to exist; that's a real, intended invariant (acceptable coupling).
(2) `palette.rs` cannot construct `Keymap { bindings }` (private field, cross-module) — tests MUST use
`default_bindings()`; fine, it is the realistic input.

## Phase 3 — Implement
- **Built exactly to the design:**
  - `palette.rs` — added pure `displayed_binding<'a>(command, keymap, stack) -> Option<&'a KeyBinding>`
    (composes `action_for` + `action_for_command`; `is_some_and` + `then_some`); widened the import to
    `use crate::keymap::{KeyBinding, KeyContext, Keymap};`.
  - `app.rs` — added `displayed_binding` to the `use crate::palette::{…}` block; the palette chip render
    now computes `let stack = self.shell.active_project().active_tab().key_context();` once before the loop
    and resolves the chip via `displayed_binding(scored.command, &self.keymap, stack)` instead of
    `scored.command.binding.as_ref()`.
- **Deviations from design:** none. (`cargo fmt` rewrapped the app.rs palette import block — cosmetic.)
- **Compile:** `cargo fmt -p marley` + `cargo check -p marley` clean — Finished, no new warnings (only the
  pre-existing `block v0.1.6` future-incompat note, unrelated). NOTE: the crate dir `crates/marley_app` is
  package **`marley`** (use `-p marley`).
- Tests (T1–T5) deferred to Phase 4 — Validate per §3 (implement writes production code only).

## Phase 3.5 — Inspect
Two independent general-purpose critics over the diff (correctness/edge-cases; state-integrity/simplification).
**Verdict: CLEAN — no defects to fix.** Lenses covered: correctness/AC, over-hiding, panic/borrow, identity
uniqueness, dynamic-command regression, scope-creep, simplification, lifetimes.

| # | Finding | Verdict | Action |
|---|---|---|---|
| F1 | Identity uniqueness: could two commands share an action string → wrong-command chip? | REJECTED — all 26 `action_for_command` strings are distinct; a chord satisfies `resolved == own` only for its own command. | none |
| F2 | Dynamic-command regression: a pushed Command with `Some` binding + no verb would now hide a chip. | REJECTED — all 5 dynamic push sites (app.rs:1582/1598/1609/1618/13925) are `binding: None`; all 10 `Some`-binding commands live in `cockpit_commands()` with ids in `action_for_command`. Verified line-by-line (13919 by me). | none |
| F3 | Panic: `active_project()/active_tab()` index unchecked. | REJECTED — the palette block is unreachable when `project_count()==0` (render returns `render_launcher` at app.rs:13845 first); Project always holds ≥1 tab (close_tab refuses LastTab). No input/response unwrap. | none |
| F4 | Over-hiding beyond the 3 named shadows. | REJECTED (intended) — the only extra hidden chip is #22 ⌘⇧\ go-to-bracket (Editor-scoped) on non-editor surfaces = a dead chord there; hiding matches the ticket principle. Global chords (font trio, ⌘⇧J) still show everywhere. | none |
| F5 | Borrow/lifetime of the render swap. | REJECTED — `key_context()` returns `&'static`, `is_some_and` collapses the keymap borrow to a bool so it doesn't escape into the returned `&'a KeyBinding`; coexists with `filter_commands`'s immutable borrow. `cargo check` compiles. | none |
| F6 | `displayed_binding` has no unit test. | NOT a diff defect — tests are the Phase 4 artifact (planned T1–T5). Both critics confirmed the mutation targets: `==`→`!=`, both `?` branches (the `action_for_command(id)?` None branch is unreachable with production data → needs a synthetic `CommandId` not in the table, my T5). | Validate |

**Validate refinement (from F4/critic-1):** use **real command #22 ⌘⇧\ go-to-bracket** (a genuine Editor-scoped
chorded command) for REQ-003 on a `[Terminal]` stack → `None`, in place of the synthetic format-document
(whose real cockpit `Command` is `binding: None`). Confirm ⌘⇧\ is Editor-scoped in the keymap when writing T3.
No `failure-record` — no new bug introduced by the diff (the ticket bug is the one being fixed).

## Phase 4 — Validate
- **Tests added (5, `palette.rs` `#[cfg(test)]`):**
  `displayed_binding_shows_an_unshadowed_chord` (REQ-001, ⌘⇧L Split-Right on TERM+COCKPIT → Some),
  `_hides_a_chord_shadowed_to_another_action` (REQ-002, ⌘⇧L on ED → None),
  `_hides_a_chord_unbound_on_the_surface` (REQ-003, ⌘⇧\ go-to-bracket #22 on TERM → None, shows on ED),
  `_none_when_command_has_no_chord` (REQ-004, binding:None → None),
  `_none_when_command_has_no_verb` (T5, synthetic id 9999 + Some binding → None — the second `?` branch).
- **Run:** `cargo nextest run -p marley displayed_binding` → **5 passed** (0.014s).
- **Mutation (`cargo mutants --list -f palette.rs`):** exactly **3** mutants on `displayed_binding` —
  `169:5 → None` (killed by REQ-001), `169:5 → Some(Box::leak(default))` (killed by every None case),
  `173:42 == → !=` (killed by REQ-001+REQ-002). The app.rs render swap is `#[cfg_attr(test, mutants::skip)]`
  (RootView::render, app.rs:13823) → the render change yields NO mutants. **MSI 100 on the diff.**
- **Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** (8:27) — all 15 pass, incl. gate:4 coverage
  100% (palette.rs covered; app.rs coverage-excluded by the gate regex), gate:5 mutation MSI 100%, gate:15
  visual/AX. Commit receipt written.
- **Live-app verification — mechanism-verified, not driven (stated explicitly, not silently skipped).** The
  render swap introduces **no new visual path**: the `.map(|b| KeyboardShortcut::parse(&b.display()).render())
  … if let Some(sc) = shortcut { row.child(sc) }` code is byte-identical; only the Option's SOURCE changed
  (static `command.binding` → the MSI-100 pure `displayed_binding`). "None → no chip" is the SAME pre-existing
  branch that `binding: None` commands already exercise in production, so the pixels for a hidden chip are
  already validated. gate:15 (visual/AX) passed. Driving synthetic ⌘⇧P into the live app mid-autonomous-run
  (risking the user's foreground) would re-confirm an unchanged render branch — verifying nothing new. Per the
  established fallback lesson (verify via units + mechanism when a driven capture adds nothing / is env-unsafe).
  Genuinely new render paths in later tickets of this train WILL be driven.
- **Pre-existing:** none in scope. (The only build note is the unrelated `block v0.1.6` future-incompat.)

## Phase 5 — Complete
- **Docs (§21):** `CHANGELOG.md` → a `### Fixed` entry (#306); `app_shell.md` → an "M22 (#306)" note in the
  palette keycap passage (the chip now resolves per-row against the focused surface). No convention shift.
- **AAR (forge):** `aar-submit` outcome=completed, effectiveness 5. **Reuse-win lesson:** the resolution engine
  already existed (`action_for` #265 + `action_for_command`) — #306 was a pure 3-line CONSUMER composer + a
  one-line render swap, NOT new resolution logic; the swap reused the pre-existing None→no-chip render branch, so
  there was no new visual path (units + mechanism carried it, gate:15 green). No `failure-record` (no new bug —
  the ticket bug is the one fixed). No new prevention rule — the standing prior-art-sweep discipline already
  covers "check for the existing pure seam before building resolution logic".
- **Close:** forge #306 → done; ticket doc → `tickets/closed/`; pipeline pair → `pipeline/completed/`.
- Gate green — the `--diff` receipt stays valid through these doc-only edits (`gate_state_hash` fingerprints only
  crates/scripts/configs + HEAD, not `docs/**` or CHANGELOG). Ready for `/commit`.
