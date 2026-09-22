---
pipeline_id: 9500762e-bc55-488d-bda5-4306d1e22395
ticket: forge#344 (0c023fd1-3df7-4a80-a19a-0383421e3899) · local docs/planning/tickets/open/TICKET-344-font-family-resolve-warning.md
aar_id: ca25106b-de10-4a34-a5ba-e0a8b2d1b621
status: Phase 5 — Complete PASS
title: Tell the user when appearance.font_family doesn't resolve (probe at boot, fall back to the built-in mono, flash)
type: feature
milestone: M22
references: [app.rs:2102 mono_family (the resolve point), app.rs:1074 new_in (the boot ctor — the probe home), app.rs:1982 the Self{} literal, app.rs:550 TERMINAL_FONT, app.rs:523 status_flash + Flash, gpui text_system.rs:90 all_font_names / :134 get_font_for_id / :150 resolve_font (PUB) / :109 font_id (PRIVATE), PR 5ee9347c (#337 landmine — assert a non-default observable), the #203 dirty-on-state-change lesson]
---

## Title
When `appearance.font_family` names a font that does not resolve, fall back to the built-in mono `TERMINAL_FONT`
EXPLICITLY (not gpui's silent, possibly-proportional system substitute) and tell the user via `status_flash`.
The #337 REQ-006 cut, now with its own probe + fallback policy + tests.

## Scope
### In
- A pure fallback POLICY (cov/MSI 100): given the requested family + whether it resolves + the built-in →
  the applied family + an optional flash message.
- A boot-time probe (in `new_in`) that asks gpui whether the requested family resolves, feeds the policy, and
  sets `font_family` + `status_flash` in the `Self{}` literal.

### Out
- A LIVE re-probe on a mid-session font (un)install — out of scope (font_family applies only at boot; restart
  re-probes; the ticket's cheap option). No `set_font_family` chord (none exists).
- The vertical/rows render, the ~6 downstream `.font_family(self.mono_family())` sites — unchanged.
- Any change to `resolve_font` usage at app.rs:4875/15483 (the render still resolves the STORED family, which
  is now guaranteed resolvable-or-built-in).

## Reference (§20)
N/A — the flash + explicit-mono-fallback UX is Marley's own (no Warp/Zed analog for a settings font-resolution
warning). Reading gpui's `TextSystem` to find a resolution probe is ADOPTION (outside the wall).

### Prior art
1. **★ gpui (Apache-2.0, adoption) — and a CORRECTION of the ticket's premise.** The ticket (echoing #337's
   amended ledger) claims gpui exposes `TextSystem::font_id(&Font) -> Result<FontId>` as the probe. **It does
   NOT: `font_id` at text_system.rs:109 is PRIVATE (`fn`, not `pub fn`)** — Marley cannot call it. The PUB
   surface on the App-level `TextSystem` (`impl TextSystem`, text_system.rs:64) is: `all_font_names() ->
   Vec<String>` (:90), `resolve_font(&Font) -> FontId` (:150, the SILENT fallback #337 used at app.rs:4875),
   `get_font_for_id(FontId) -> Option<Font>` (:134). So the PUBLIC probe is one of: (a) membership —
   `all_font_names()` contains the requested family; (b) round-trip — `resolve_font(font(requested))` →
   `get_font_for_id(id)` → the resolved font's family ≠ requested ⟹ it fell back. Design picks (see D2). The
   handle is `cx.text_system()` (App-level `TextSystem`, `App::text_system` app.rs:222/1446), reachable from
   `new_in`'s `cx: &mut Context<Self>` (Context derefs to App). `Window::text_system()` returns a DIFFERENT
   `WindowTextSystem`; `all_font_names` is on the App-level one.
2. **OUR OWN CODE:** `mono_family` (app.rs:2102) is the resolve point (empty→built-in); `status_flash`/`Flash`
   (:523) is the surface; `TERMINAL_FONT` (:550) is the mono built-in. The #337 `set_font_size` (:2062) +
   `zoom_persists_through_the_verb_headless` show the TempDir-isolated settings-boot test idiom.
3. Checked ropey/regex/tree-sitter — N/A (a font-resolution + UX concern).

## Locked-In Decisions
- **D1-PROBE-AT-BOOT** — probe once in `new_in` (app.rs:1074, the boot ctor, which has `cx`), where
  `applied.font_family` is first applied. font_family has no live setter, so boot is the only apply point and
  probing there is correct + complete. (A mid-session uninstall is out of scope — restart re-probes.)
- **D2-PUBLIC-PROBE (NOT font_id)** — because `font_id` is private, the probe uses a PUBLIC gpui API. Design
  chooses (a) `all_font_names()` membership or (b) the `resolve_font`→`get_font_for_id` round-trip, and
  isolates it behind a tiny `fn font_resolves(cx, family) -> bool` shim so the pure policy takes a plain `bool`.
  RECOMMEND (b) the round-trip — it detects EXACTLY gpui's silent-fallback behavior (the harm), using gpui's
  own matching, where (a) may diverge from gpui's case/style matching. Design confirms against `resolve_font`'s
  behavior on a garbage family.
- **D3-EXPLICIT-BUILTIN-FALLBACK** — on non-resolve, store `""` (empty), which `mono_family` already maps to
  `TERMINAL_FONT` — preserving `mono_family`'s "empty = built-in" invariant and not freezing "Menlo" into the
  field. `mono_family()` then returns the mono built-in, NOT gpui's proportional system substitute.
- **D4-FLASH-AT-BOOT** — surface via `status_flash: Some(Flash::new(msg))` in the `Self{}` literal. The first
  paint is guaranteed at boot, so the #203 idle-frame `dirty` dance is not needed here (confirm the Flash has
  no tick that a boot-set misses). Message names the unresolvable family + the fallback.
- **D5-PURE-POLICY-SEAM** — the decision is a pure fn (cov/MSI 100) in a settings/font module; the probe +
  Flash construction are the `new_in` shim (`mutants::skip`). Settings/probe are best-effort — a probe or
  settings-file problem must NEVER crash boot (fall back to the built-in, no panic).

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-UNRESOLVABLE-BUILTIN | WHEN `appearance.font_family` is non-empty and does NOT resolve, the applied mono family shall be the built-in `TERMINAL_FONT`. | headless: seed settings with a garbage family, boot, assert `mono_family()` == "Menlo" (a NON-DEFAULT observable — a do-nothing impl returns the garbage → the assert distinguishes; the #337 landmine). |
| REQ-UNRESOLVABLE-FLASH | WHEN it does not resolve, the system shall set `status_flash` with a message naming the family + the fallback. | headless: after the garbage boot, assert `status_flash` is `Some` + the message contains the garbage family + the built-in. |
| REQ-RESOLVABLE-SILENT | WHEN `appearance.font_family` DOES resolve, the family shall apply with NO flash. | headless/unit: a resolvable family (or the pure policy with probe_ok=true) → applied == requested, flash None. |
| REQ-EMPTY-BUILTIN | WHEN `appearance.font_family` is empty, the applied family shall be the built-in with NO flash. | unit: the pure policy `("", …)` → built-in, None. |
| REQ-POLICY-TOTAL | The pure policy shall be total (any requested/builtin string, any probe bool) with no panic. | unit: the policy over a small hostile matrix. |

## Phase Plan
- **P2 Design** — settle D2 (the exact pub probe: round-trip vs all_font_names; confirm gpui's garbage
  behavior); the pure policy signature/struct (`FontResolution { applied, flash }`); the `font_resolves` shim +
  the `new_in` wiring (rename `_window`, or use `cx.text_system()`); the headless test plan (seed garbage
  settings → boot → assert mono_family + flash; the #337 TempDir idiom); confirm the flash shows at boot;
  settings.rs/mono_family doc-truth restoration.
- **P3 Implement** — the policy fn + the probe shim in `new_in` + the doc restoration. `cargo check`.
- **P3.5 Inspect** — the probe truly detects a garbage family (D2); the #337 landmine (non-default observable);
  the `mono_family` "empty=built-in" invariant preserved; boot never crashes on a probe/settings problem.
- **P4 Validate** — the policy unit tests (cov/MSI 100) + a headless boot-with-garbage-settings test + the
  `--diff` gate. **NO live drive (a boot font-flash isn't eyeball-worthy + chad at the machine; the headless
  boot assert is the exact proof — stated, not masked).**
- **P5 Complete** — CHANGELOG + editor.md + restore the `mono_family`/settings doc (REQ-006 reads as written,
  and the doc's "font_id probe exists" line is corrected to the pub `all_font_names`/round-trip); AAR (the
  font_id-is-private correction is the headline lesson); close + archive.
