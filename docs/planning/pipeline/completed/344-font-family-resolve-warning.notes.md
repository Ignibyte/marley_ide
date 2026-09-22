# 344 — font_family resolve warning — notes

- pipeline_id 9500762e-bc55-488d-bda5-4306d1e22395 · forge #344 0c023fd1-3df7-4a80-a19a-0383421e3899
- aar_id ca25106b-de10-4a34-a5ba-e0a8b2d1b621 · on `b5e4995`

## Phase 1 — Plan

**Intent:** an unresolvable `appearance.font_family` degrades to gpui's SILENT (possibly-proportional) system
font — breaking the terminal grid with nothing to explain it. Probe at boot; on fail, use the mono built-in +
flash. The #337 REQ-006 cut, now with its own design.

**Recon (live on `b5e4995`) — confirmations + one decisive CORRECTION:**
1. **Resolve point** = `mono_family(&self)` (app.rs:2102, `mutants::skip`): empty→`TERMINAL_FONT`, else the
   stored family. `&self`, per-render, no cx → the probe can't live here.
2. **★ CORRECTION — the ticket's `font_id` claim is FALSE.** The ticket + #337's amended ledger say gpui
   exposes `TextSystem::font_id(&Font)->Result` as the probe. **It's PRIVATE** (text_system.rs:109, `fn` not
   `pub fn`) — uncallable from Marley. The PUB probe surface (`impl TextSystem`, :64): `all_font_names()->
   Vec<String>` (:90), `resolve_font(&Font)->FontId` (:150, the SILENT one #337 already uses at app.rs:4875),
   `get_font_for_id(FontId)->Option<Font>` (:134). So the probe is either (a) `all_font_names()` membership or
   (b) `resolve_font(font(x))`→`get_font_for_id(id)`→resolved-family≠requested (round-trip). RECOMMEND (b) — it
   detects EXACTLY gpui's silent fallback via gpui's own matching. Handle = `cx.text_system()` (App-level
   TextSystem; `Window::text_system()` is a DIFFERENT WindowTextSystem lacking all_font_names).
3. **Probe home** = `new_in(_window: &mut Window, cx: &mut Context<Self>) -> Self` (app.rs:1074), which builds
   the `Self{}` at :1982 (`font_family: applied.font_family.clone()`). Has `cx` (derefs to App → text_system).
   font_family is applied ONLY here (no `set_font_family` — unlike `set_font_size` :2062) → boot probe is
   complete.
4. **Pieces:** `TERMINAL_FONT="Menlo"` (:550); `status_flash: Option<Flash>` (:523) + `Flash::new`; the ~6
   downstream `.font_family(self.mono_family())` sites unchanged. First paint is guaranteed at boot → a
   boot-set flash shows without the #203 idle-frame `dirty` (confirm no Flash tick a boot-set misses).
5. **Seam (D5):** a pure `resolve_font_family(requested, probe_ok, builtin) -> FontResolution{applied, flash}`
   (cov/MSI 100) + a `font_resolves(cx, family) -> bool` probe shim + the `new_in` wiring (mutants::skip).
   Store `""` on fallback (D3 — preserves `mono_family`'s empty=built-in invariant).

**knowledge-context (Plan):** 13 nodes logged — incl. PR 5ee9347c (the #337 landmine: assert a NON-DEFAULT
observable, never one a do-nothing impl returns — the AC's REQ-UNRESOLVABLE-BUILTIN honors it by asserting
`mono_family()=="Menlo"` after a GARBAGE seed) + the #203 dirty lesson + the #337 font AD.

**Prior-art sweep:** (1) gpui — font_id PRIVATE (the correction); the pub probe = all_font_names/round-trip on
the App-level TextSystem; resolve_font (:150) is the silent one #337 used. (2) our own mono_family/status_flash/
TERMINAL_FONT + the #337 boot-settings test idiom. (3) ropey/regex — N/A. §20 = N/A/gpui-adoption.

**EARS:** REQ-UNRESOLVABLE-BUILTIN, REQ-UNRESOLVABLE-FLASH, REQ-RESOLVABLE-SILENT, REQ-EMPTY-BUILTIN,
REQ-POLICY-TOTAL.

**Risks:** (a) ★ D2 — the exact pub probe. The round-trip (b) needs `resolve_font` to actually report the
FALLBACK family for a garbage input (design confirms by reading resolve_font/the mac backend); if it instead
echoes the requested family, use (a) all_font_names membership. This is the design's real question. (b) the
headless probe — a `gpui::test` boot has a TextSystem (the #337 lane boots a real window), so the probe runs;
but confirm `all_font_names`/`resolve_font` return sanely under the test platform (they should — #337's metrics
tests use resolve_font). (c) the flash-at-boot visibility (D4) — confirm no tick.

**Status: Phase 1 — Plan PASS; ready for Phase 2 — Design.**

## Phase 2 — Design

**★ D2 SETTLED — the probe is `all_font_names()` membership, NOT the round-trip.** Read gpui:
- `TextSystem::all_font_names()` (text_system.rs:90) = `platform_text_system.all_font_names()` + the fallback
  stack + `.SystemUIFont`, sorted/deduped. PUB. Never panics.
- `resolve_font()` (:150) **PANICS** if the font + none of the fallbacks resolve → unsafe to call on a garbage
  family at boot (D5 "never crash boot"). The round-trip (b) ALSO needs `get_font_for_id`, which reads the
  cache.
- ★ **The test platform is `NoopTextSystem`** (platform.rs:594): `all_font_names()` → `Vec::new()` (empty, no
  panic); `font_id()` → **always `Ok(FontId(1))`** (echoes success for ANY family). So under `gpui::test`: the
  round-trip (b) is BROKEN (Noop echoes → a garbage family reads as "resolved"), and `all_font_names()` returns
  empty → a garbage family is correctly ABSENT → `font_resolves` = false. **(a) all_font_names wins on both
  correctness AND test-behavior AND no-panic.** Tradeoff recorded: membership may be STRICTER than gpui's fuzzy
  resolver (a face-name like "Menlo-Bold" that gpui might resolve isn't a family in the list → a false-positive
  warning + a harmless mono fallback) — acceptable vs a panic-prone, Noop-broken round-trip.

**The pure policy (settings.rs — coverage-included; the cov/MSI 100 home):**
```rust
pub struct FontResolution { pub applied: String, pub flash: Option<String> }
pub fn resolve_font_family(requested: &str, resolves: bool, builtin: &str) -> FontResolution {
    if requested.is_empty() {
        FontResolution { applied: String::new(), flash: None }           // empty = built-in (mono_family maps it)
    } else if resolves {
        FontResolution { applied: requested.to_string(), flash: None }   // resolvable → apply, silent
    } else {
        FontResolution {                                                 // unresolvable → built-in + flash
            applied: String::new(),
            flash: Some(format!("Font \"{requested}\" not found — using {builtin}")),
        }
    }
}
```
Total (any strings, any bool; no panic). `applied: String::new()` (D3) → `mono_family()` maps empty→built-in,
preserving the invariant + not freezing "Menlo" into the field.

**The probe shim (app.rs, `mutants::skip`):**
```rust
fn font_resolves(cx: &Context<Self>, family: &str) -> bool {
    !family.is_empty()
        && cx.text_system().all_font_names().iter().any(|n| n.eq_ignore_ascii_case(family))
}
```
`cx.text_system()` = the App-level `TextSystem` (Context derefs to App; app.rs:222/1446) which HAS
`all_font_names`. (`window.text_system()` is a different `WindowTextSystem`.) `eq_ignore_ascii_case` handles a
case typo.

**The `new_in` wiring (app.rs:1074, inside the `mutants::skip` ctor).** Before the `Self{}` literal:
```rust
let font_res = crate::settings::resolve_font_family(
    &applied.font_family,
    Self::font_resolves(cx, &applied.font_family),
    TERMINAL_FONT,
);
```
Then in the literal: `font_family: font_res.applied,` (was `applied.font_family.clone()`) and
`status_flash: font_res.flash.map(Flash::new),` (was `None` — FIND the exact `status_flash:` line in the
literal). `Flash::new` takes a String/&str (called with `format!` at :1225 + `"…"` at :2696). The probe uses
`cx` before the literal — no borrow clash (returns a bool + owned strings). `_window` stays unused (probe uses
`cx`).

**File manifest:**
| file | change |
|------|--------|
| `crates/marley_app/src/settings.rs` | +`pub struct FontResolution` + `pub fn resolve_font_family` (the cov/MSI 100 policy) + its `#[cfg(test)]` tests. |
| `crates/marley_app/src/app.rs` | +`fn font_resolves` (mutants::skip probe shim); `new_in` computes `font_res` + sets `font_family`/`status_flash` from it; restore the `mono_family` doc (font_id→all_font_names, #344 shipped). ALL inside `mutants::skip`. |
| `crates/marley_app/src/headless_drive.rs` | +1 `#[gpui::test]` — boot with a garbage font_family, assert `mono_family()`==built-in + flash set. Test code. |

**Mutation/coverage:** `resolve_font_family` → settings.rs cov/MSI 100 (the unit tests). `font_resolves` + the
`new_in` wiring → app.rs `mutants::skip` + coverage-excluded. gate:5 `--diff` mutates ONLY `resolve_font_family`.

### Regression Test Plan (headless — no live drive)
| # | REQ | test |
|---|-----|------|
| T1 | RESOLVABLE-SILENT + EMPTY + UNRESOLVABLE + TOTAL | `settings.rs` unit tests on `resolve_font_family`: `("", true/false, "Menlo")`→{"",None}; `("Fira", true, "Menlo")`→{"Fira",None}; `("Garbage", false, "Menlo")`→{"", Some(msg contains "Garbage" + "Menlo")}; a small hostile-string × bool matrix → no panic. cov/MSI 100. |
| T2 | UNRESOLVABLE-BUILTIN + UNRESOLVABLE-FLASH | `font_family_garbage_falls_back_to_builtin_headless` (the #337 `zoom_persists…` TempDir idiom): seed settings `appearance.font_family = "Menlo-Bold-Oblique-Nope"`, `boot`, assert `mono_family()` == TERMINAL_FONT (a NON-DEFAULT observable — a do-nothing impl returns the garbage; the #337 landmine) AND `flash_message_for_test()` (app.rs:8663) is `Some` containing the garbage family. Under Noop `all_font_names()`==[] → the garbage is absent → the real probe→policy→fallback path runs. |

Uncoverable headlessly: the RESOLVABLE-boot integration (Noop's `all_font_names` is empty → even "Menlo" reads
unresolvable in tests) — proven by the pure policy (`resolves=true`) instead; noted. The LIVE font-flash pixel —
DEFERRED (a boot flash isn't eyeball-worthy; the headless mono_family + flash-message asserts are the proof).

**Risks:** (a) D2 — SETTLED (all_font_names; the round-trip is Noop-broken + panic-prone). (b) `new_in` never
panics on a garbage family: `all_font_names` doesn't panic, and the fallback stores `""` so the render resolves
"Menlo" (never the garbage) — no `resolve_font` panic. (c) the exact `status_flash:` literal line + `Flash::new`
arg type — confirm at implement. (d) plain-backtick docs (the corrected mono_family doc).

**Status: Phase 2 — Design PASS; ready for Phase 3 — Implement.**

## Phase 3 — Implement

Built to the manifest — 2 src files, exactly as designed:
1. **settings.rs** — `pub struct FontResolution { applied, flash }` + `pub fn resolve_font_family(requested,
   resolves, builtin) -> FontResolution` (the 3-arm policy, after `clamp_files_width`). Doc plain backticks. No
   tests (Phase 4).
2. **app.rs** — (a) `fn font_resolves(cx: &Context<Self>, family) -> bool` (`mutants::skip`) after
   `mono_family`: `!family.is_empty() && cx.text_system().all_font_names().iter().any(|n|
   n.eq_ignore_ascii_case(family))`. (b) `new_in` (before the `Self {` at :1888): `let font_res =
   crate::settings::resolve_font_family(&applied.font_family, Self::font_resolves(cx, &applied.font_family),
   TERMINAL_FONT);`. (c) the literal: `font_family: font_res.applied` (was `applied.font_family.clone()`) +
   `status_flash: font_res.flash.map(Flash::new)` (was `None`, found at the literal). (d) corrected the
   `mono_family` doc — the "gpui has one — `TextSystem::font_id`" claim → the PUBLIC `all_font_names` probe (font_id
   is private; resolve_font panics) + "#344 SHIPPED".

**Deviation:** none. `Self::font_resolves(cx, …)` (a self-less associated fn) compiles; `cx.text_system()`
resolves (Context derefs to App); `font_res.flash.map(Flash::new)` → `Option<Flash>` = the `status_flash` type.

**Verification:** `cargo check -p marley` clean; `cargo clippy -p marley --all-targets -- -D warnings` rc=0 (the
`block v0.1.6` line is a pre-existing dep future-incompat). `cargo fmt --all` applied. Diff = settings.rs +
app.rs. No test expansion (Phase 4). No Zed/Warp.

**Status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect.**

## Phase 3.5 — Inspect

Scaled to **1 critic** (general-purpose, correctness+probe+boot-safety lens) + own review. Verdict: **SHIP-READY**
(all points OK; 2 LOW by-design scope boundaries, both fail-safe).

### Findings (verdicts)
| # | Sev | Finding | Verdict |
|---|-----|---------|---------|
| C1 | LOW | A font that RESOLVES but is PROPORTIONAL (e.g. `appearance.font_family = "Arial"`) passes `font_resolves` → applies silently → the grid breaks with no warning. | **By-design scope boundary — filed follow-up #361.** #344 is a *resolution* warning (asked for X, X didn't resolve → gpui's silent substitute); a monospace check is a different feature (glyph-advance metric introspection). Not a code defect. #361 tracks it. |
| C2 | LOW | `all_font_names()` returns FAMILY names, so a face/PostScript name ("Menlo-Bold") isn't a member → false-negative over-warn + a mono fallback. | **REJECT — no change.** Harmless (a spurious flash + a correct mono grid, never a crash), and the `font_resolves` doc already frames it as a *membership* probe. Whitespace-only settings ("   ") also fail SAFER than pre-#344 (which fed them to `resolve_font` → a proportional fallback). |

Critic confirmed **OK** (own-spot-checked the diff + gpui source):
- **(a) policy:** 3 arms correct; the else `msg` names both `requested` + `builtin`; total (no unwrap/panic/index); `applied:""` on fallback (D3 — the empty-first arm wins even on a contradictory `resolves` flag).
- **(b) probe (the D2 crux):** uses PUBLIC `all_font_names()` (`font_id` confirmed PRIVATE at gpui text_system.rs:109; `resolve_font` confirmed PANICS at :150 — correctly avoided); `cx.text_system()` = the App-level TextSystem (Context derefs to App, app.rs:1446) that HAS all_font_names; empty short-circuits; `eq_ignore_ascii_case`. Under `NoopTextSystem` all_font_names=`Vec::new()` (platform.rs:609) → garbage correctly absent (and the round-trip would have been WRONG — Noop.font_id always `Ok(FontId(1))` echoes).
- **(c) ★ boot NEVER panics:** the garbage family reaches ONLY the `font_res` binding; the two `resolve_font(` sites (app.rs:4898/:15506) resolve `mono_family()` = `""`→"Menlo", never the garbage. #344 strictly REDUCES panic exposure.
- **(d) wiring/types:** `&applied.font_family` borrowed twice (both `&`); `font_res.applied`/`font_res.flash` are distinct-field moves; `Flash::new(impl Into<String>)` under `Option::map` → `Option<Flash>` = the field type.
- **(e) doc + landmine:** the corrected `mono_family` doc names the pub probe + flags font_id PRIVATE + resolve_font PANICS; no new `[link]` to a private item; the AC's `mono_family()=="Menlo"` after a GARBAGE seed is genuinely non-default (a do-nothing impl → the garbage).
- **(f) regression:** the render sites + `set_font_size` + the settings round-trip untouched; happy path (resolvable/empty) unchanged.

**Result: 0 findings requiring a code fix.** C1 → follow-up #361 (a distinct monospace-check feature); C2 rejected (safe + documented). No forge failure-record (no bug in this diff). Lenses: policy correctness, probe semantics, boot-safety, wiring/types, doc/landmine, regression. `git status --porcelain` = settings.rs + app.rs + the #344 docs.

**Status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate.**

## Phase 4 — Validate

**Tests — 3 policy units (settings.rs, the cov/MSI 100 surface) + 1 headless garbage-boot (headless_drive.rs) +
2 test hooks:**
| test | REQ | what it proves |
|------|-----|----------------|
| `t344_resolve_font_family_empty_is_builtin_no_flash` | EMPTY-BUILTIN | empty→`{"", None}` for BOTH `resolves` values (empty-first arm wins). |
| `t344_resolve_font_family_resolvable_applies_silently` | RESOLVABLE-SILENT | `("Fira Code", true)`→`{"Fira Code", None}`. |
| `t344_resolve_font_family_unresolvable_falls_back_and_warns` | UNRESOLVABLE-BUILTIN+FLASH | `("Nope-Font", false)`→`{"", Some(msg)}`, msg contains BOTH the family + "Menlo". |
| `font_family_garbage_falls_back_to_builtin_headless` | UNRESOLVABLE-BUILTIN+FLASH (integration) | seed `appearance.font_family="Menlo-Bold-Oblique-Nope"` (via test-only `persist_font_family`) → `boot` → `mono_family_for_test()`=="Menlo" (NON-DEFAULT — a do-nothing impl keeps the garbage) + `flash_message_for_test()` names it. Proves the `new_in` probe→policy→field wiring. |

Added: test-only `settings::persist_font_family` (`#[cfg(test)] pub(crate)`, mirroring `persist_language_servers`
— Marley has no live font_family setter) + `RootView::mono_family_for_test` (`#[cfg(test)] pub(crate)` — the
private `mono_family` isn't callable cross-module).

**★ Mutation refinement:** the sole `resolve_font_family` mutant is the fn body→`Default::default()`. `FontResolution`
did NOT derive `Default` → the mutant would be UNVIABLE (like #203's Notify) → excluded → a hollow MSI 100. Added
`#[derive(Default)]` (a sensible `{"" , None}` default) → the mutant is now VIABLE and the resolvable/unresolvable
tests (which expect NON-default values) KILL it — real assurance (the #204 lesson: a Default-able return makes the
body mutant concrete+viable+killed).

**Runs (all `CARGO_BUILD_JOBS=4 CARGO_INCREMENTAL=0`):**
- `cargo nextest run -p marley t344_resolve_font_family font_family_garbage` → **4/4 PASS**.
- `cargo mutants --in-diff <settings.diff> -f settings.rs --test-tool=nextest` → **1 mutant tested: 1 caught**
  (the now-viable body→Default, killed).

**NO LIVE DRIVE — STATED (not masked).** A boot font-flash isn't eyeball-worthy + chad at the machine; the
headless garbage-boot (mono_family + flash) + the pure policy are the exact proof. The RESOLVABLE-boot case is
NOT headless-testable — `NoopTextSystem.all_font_names()` is empty so even "Menlo" reads unresolvable under the
test platform — covered by the pure policy's `resolves=true` row instead (stated).

**FULL `--diff` GATE → `GATE GREEN [diff]` 15/15** (gate:4 coverage ≥100 [resolve_font_family fully tested;
app.rs excluded; the hooks + headless test are test code]; gate:5 mutation MSI 100 [1/1]; gate:14 docs PASS [the
corrected mono_family/FontFamily docs, plain backticks]; gate:6 miri skip-clean [no unsafe]). Receipt
`75e300cd2c788fd23b4dbe9565ad68f201000296`, worktree-bound. #334 flake did not recur.

**Status: Phase 4 — Validate PASS; ready for Phase 5 — Complete.**

## Phase 5 — Complete
