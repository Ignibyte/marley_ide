# Appearance settings — font size + zoom (#337) — Notes

- **Forge ticket:** #337 169609bd-40e6-4e08-901d-2a86cfc15226
- **AAR:** d8d283b2-cfda-49e7-a41b-dd8fbfa8e340
- **Local ticket doc:** ../../tickets/closed/TICKET-337-appearance-font.md
- **Pipeline spec:** 337-appearance-font.spec.md

## Phase 1 — Plan

Promoted `queued/m22-appearance-font.spec.md` → `active/337-appearance-font.spec.md` (pipeline_id
`0f37229d…`, AAR `d8d283b2…`, local ticket doc written). Classification: work pipeline, feature, one slice.

### THE VERIFICATION LEDGER — **the spec's central premise is FALSE**

Run 2 of promote-don't-author, and it earned its keep harder than run 1. #336 landed at `df7784b` between
authoring and promotion, so line drift was expected — but the real finding is that the spec is built on a
claim that does not hold.

| # | Claim | Verdict |
|---|---|---|
| 1 | `TERMINAL_FONT_SIZE` is the ONE font-size const behind BOTH surfaces | **F1 — WRONG. There are TWO font-size systems, and the terminal uses the other one for its own text.** |
| 2 | `fallback_cell(TERMINAL_FONT_SIZE)` is an unlisted consumer | **VERIFIED** (workspace.rs:121; + an unlisted caller at app.rs:10835) |
| 3 | Two cell-metric sources (per #336's ADR) | **F2 — there are FOUR in-app, and the fourth is a pre-seeded bug this ticket would DETONATE** |
| 4 | The workspace.rs stale guard (`was a stale 14`) | **VERIFIED** (:123; hardcodes 13.0 today) — **F3: it will go stale a THIRD time unless #337 sources it from the setting** |
| 5 | ⌘= / ⌘− / ⌘0 are FREE | **VERIFIED** — no cmd-chord binds `=`, `-` or `0`; the only `-` is ⌃− → nav-back (Editor-scoped, not a collision) |
| 6 | An existing window/pane resize path re-grids PTYs | **DRIFTED — the path exists but the mechanism is not what the spec says.** Good news for us. |
| 7 | The #330 four-wiring + the #199 write-through precedent | **VERIFIED** — `set_theme` (app.rs:1915) is the line-for-line shape to copy |
| 8 | `appearance.font_size` does not exist in the app | **VERIFIED** — the key is free (the marley_settings fixture is a generic stand-in) |

**F1 [BLOCKING — the premise is false, and the spec's scope is wrong because of it].** There is a SECOND
font-size system: **`crates/marley_app/src/typography.rs`** — `type_scale(Role::Command) = 13.0`,
`Role::Output) = 13.0`, `Caption = 11.0`, `Nav = 12.0`, ~18 call sites. And the terminal draws **its own text
with `type_scale`, not with `TERMINAL_FONT_SIZE`**: output rows at app.rs:13735
(`type_scale(Role::Output).size`) and command headers at :13706/:13979 (`cmd.size`). The pane CONTAINER sets
`TERMINAL_FONT_SIZE` (:13248) and its comment claims the children "inherit the mono font + size" — **they do
not**; only the prompt and alt-grid actually inherit.
→ **Ship as specced and the PTY re-grids, the prompt scales, and the terminal's command headers and scrollback
stay pinned at 13pt.** A half-zoom, on the app's most visible surface. It is invisible today only because
**#195 set both constants to 13.0**, so the two systems agree by coincidence rather than by design.
Also: 12 `text_size(px(TERMINAL_FONT_SIZE))` sites exist; **the spec lists 6 and misses 6** (def_picker,
code_action, symbols, search, problems overlays + the terminal pane container). Plus raw `px(13.0)` literals
at app.rs:4137/4147/4164/4182/4241/14587/14754/15249/15343.
**→ DESIGN MUST ANSWER A QUESTION THE SPEC NEVER ASKS: does `type_scale` become a function of the font-size
setting, or stay a fixed chrome scale?** That gates this ticket harder than the metrics unification does, and
D-ONE-KEY ("one shared key for both surfaces") was premised on one shared const — the premise is gone, so the
decision needs re-taking, not re-stating.

**F2 [the ADR's assumption is already violated — by existing code].** #336's
`AD-claude-hscroll-shift-clip-split-and-two-probe-domains-001` says #337 must not add a THIRD source of cell
width. There are already **four** in-app: (1) `fallback_cell` (workspace.rs:121, `w = size*0.6, h = size*1.2`
— and the SOLE source of `h` everywhere; no probe ever measures height); (2) the editor probe
(app.rs:12969-12986, `em_advance` → `EditorDraw.cell`); (3) the PTY probe (app.rs:13179-13192, a
byte-identical `em_advance`, independently re-measured); (4) **the terminal hit-test at app.rs:13222-13223 —
`let cell_w = fallback.w`**, using the DERIVED fallback while the MEASURED advance from source 3 sits in scope
30 lines above. Plus `terminal_blocks/src/pty_os.rs:41-42` hardcodes `cell_width: 8, cell_height: 16` into the
spawn `WindowSize`.
**Source 4 is a pre-seeded instance of this ticket's own bug class, and #337 DETONATES it.** Menlo@13pt's
advance ≈ 7.83 vs `13 × 0.6 = 7.8` — a 0.4% gap, invisible. The gap scales with size: at 26pt it is ~0.06px
per column, ~5px across 80 columns → **the terminal's click→cell mapping drifts a full cell near the right
edge.** Today's font size hides it; zoom exposes it.
→ Sources 2+3 unify trivially (identical calls in the same `fn render`). Source 1 correctly STAYS as the
fallback + the height source. **Source 4 is not a refactor — it is a bug fix**, and folding it into a
font-size ticket makes the diff un-bisectable (if terminal clicking regresses, was it the zoom or the metric
swap?). Design decides: precursor ticket, or an independently-tested hunk with its own justification.

**F3 [the stale guard will go stale a third time].** workspace.rs:123 `let size = if font_size > 0.0 {
font_size } else { 13.0 };` — the comment records that it *was* 14, went stale when #195 moved to 13, and was
fixed by #230/#223. A literal default here is the same trap a third time; #337 must source it from the
setting's default, not re-type the number. (Amusing corroboration: the marley_settings fixture still says
`FontSize: u32 = 14` — the same stale 14, fossilized.)

**F6 [DRIFTED — in our favour].** There is **no window-resize handler and no SIGWINCH**. The re-grid is
**render-driven every frame**: `RootView::render` → app.rs:13178-13210 → `plan_resize(...)` (workspace.rs:133,
returns `Some` only when the grid actually changed) → `session.resize(cols, rows)` → `channel.set_winsize` →
`pty_os.rs:61` `tcsetwinsize`; the KERNEL raises SIGWINCH, Marley never sends one. → **D-REGRID-REUSE holds
and is cheaper than written: change the cell and the next frame re-grids every live PTY automatically.**
`plan_resize` IS the seam and already self-guards.

### Decisions (Phase 1)
D-CLAMP-AT-EVERY-DOOR, D-LIVE-PERSIST (copy `set_theme`, app.rs:1915 — best-effort persist, never crash the
terminal) and D-REGRID-REUSE all **stand**. **D-ONE-KEY and D-ONE-METRICS-SEAM are RE-OPENED** by F1 and F2
and become Design forks:
- **Fork A (F1):** is `type_scale` font-size-driven or fixed chrome? Without an answer the terminal half-zooms.
- **Fork B (F2):** does source 4 (the terminal hit-test's derived cell width) get fixed here, in a precursor,
  or explicitly deferred with the zoom-drift documented?

**LIVE drives OFF-LIMITS** (chad is at the machine; synthetic input hits his frontmost window) → units +
headless + mechanism.

**§20:** N/A confirmed — ⌘±/⌘0 zoom is a universal OS-level convention (browsers, terminals, editors), not a
copyleft behavior; no reference source read.

**Method verdict (run 2):** run 1 caught a REQ resting on a deleted seam; run 2 caught a **false premise** —
the spec's one-const story, which every one of its decisions was built on. Both were authorship errors, not
code drift, and both would have surfaced at Implement or later. The recurring lesson sharpens: **the
pre-authored spec's confident sentences are the dangerous ones.** "X is the ONE Y" is a claim to verify, not a
fact to build on.

status: Phase 1 — Plan PASS; ready for Phase 2 — Design

## Phase 2 — Design

Both Phase-1 forks resolved. F1 changed the ticket's shape; F2 changed its scope.

### FORK A (F1: does `type_scale` zoom?) — **RESOLVED: content zooms, chrome does not. The module already
says so; it just doesn't act on it.**
`typography.rs` is PURE (gpui-free, cov/MSI 100) and its `Role` doc comments **already encode the
distinction**: `Command` = "a command line (the Block header)", `Output` = "terminal output — the base body
text" — CONTENT; `Caption` = "a caption / secondary label (dock headers, metadata)", `Nav` = "left-sidebar /
nav **chrome**" — CHROME. The semantic split exists and is documented; the function simply returns a constant
for all four.
**Decision — D-CONTENT-ZOOMS:** `type_scale(role)` becomes `type_scale(role, font_size)`. `Command`/`Output`
return `size: font_size`; `Caption`/`Nav` keep their fixed 11.0/12.0. So ⌘= means **"the text I read gets
bigger"** — editor code and terminal output — not "the whole UI rescales". That matches the mental model of
`editor.fontSize` (a content size) rather than View→Zoom (a UI scale), and it is what makes D-ONE-KEY
survivable: one key drives both surfaces' CONTENT, which is exactly what the one shared const drove.
Without this the ticket half-zooms: PTY re-grids, prompt scales, terminal scrollback stays 13pt.
**D-ONE-KEY is re-confirmed on a NEW basis.** Its original premise (one const behind both surfaces) was false;
the surviving reason is better — both surfaces' content is the same MONO reading surface, and a user who zooms
one and not the other has to zoom twice. The two systems (`TERMINAL_FONT_SIZE`, `type_scale`) agreeing at 13.0
was #195 coincidence; this ticket makes the agreement structural: one setting, two readers, no const.

### FORK B (F2: the terminal hit-test's derived cell width) — **RESOLVED: fix it HERE, as its own hunk +
its own test + its own CHANGELOG line. Not deferred, not silently folded in.**
- **Deferring is not available.** app.rs:13222 uses `fallback.w` (13 × 0.6 = 7.8) while the MEASURED advance
  (Menlo@13pt ≈ 7.83) sits in scope 30 lines above. The 0.4% gap is invisible at 13pt and scales linearly:
  at 26pt it is ~0.06px/col → ~5px across 80 columns → **the terminal's click→cell mapping drifts a full cell
  near the right edge.** This ticket is what makes it reachable. Shipping zoom on top of a metric that only
  works at one size is shipping the bug.
- **A precursor ticket is the textbook answer and it is wrong here.** The fix is a 1-line swap
  (`fallback.w` → the measured `cell.w` already in scope); a whole pipeline to move one identifier costs more
  than it protects, and #337 cannot ship correctly without it — so the "precursor" would be a dependency, not
  an independent slice.
- **So: fix it here, but make it BISECTABLE BY CONSTRUCTION** — one isolated hunk, its own headless test
  (`terminal_hit_test_uses_the_measured_advance`), its own CHANGELOG bullet under Fixed, and a comment naming
  the drift-at-zoom it prevents. The reviewer's question "was it the zoom or the metric swap?" is answered by
  the test that fails without the swap and passes with it, independent of any font size.
- `pty_os.rs`'s hardcoded `cell_width: 8, cell_height: 16` stays OUT (it is the spawn-time WindowSize, not a
  render metric; the first frame's `plan_resize` corrects it). Recorded, not touched.

### D-ONE-METRICS-SEAM — narrowed by evidence
Four in-app sources; the seam unifies **2 and 3** (the editor and PTY probes — byte-identical `em_advance`
calls in the same `fn render`, trivially one memoized call) and **redirects 4** (Fork B). Source **1
(`fallback_cell`) deliberately SURVIVES underneath**: it is the honest fallback when no measurement exists yet
AND the sole source of cell HEIGHT (no probe measures height — a real, pre-existing gap, recorded not fixed).
So the seam is "one MEASUREMENT, read by every consumer", not "one function to rule out `fallback_cell`".
**F3:** `workspace.rs:123`'s `else { 13.0 }` must read the setting's default, not a literal — it has gone
stale twice already (14 → 13 via #195/#230/#223). The default lives in `define_setting!` and nowhere else.

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/typography.rs` | `type_scale(role, font_size)` — content roles return `font_size`, chrome roles stay fixed. Pure, cov/MSI 100. Its ~18 call sites thread the size. |
| `crates/marley_app/src/settings.rs` | `define_setting!(FontSize: f32 = 13.0, "appearance.font_size")` + `FontFamily: String = ""`; `AppliedSettings` fields; both resolvers (**the [8,32] clamp lives in `applied_from`, beside `code_tab_width`'s `.max(1)` — the established site**); `persist_font_size` / `persist_font_family`; the NON-DEFAULT round-trip leg. |
| `crates/marley_app/src/font_zoom.rs` | **NEW** pure seam: `zoom_step(cur, delta) -> f32` + `clamp_font_size(v) -> f32`, [8,32], total (a hand-edited 0/NaN/300 → the default/clamp). cov/MSI 100. `git add -N` before the gate. |
| `crates/marley_app/src/app.rs` | Delete `TERMINAL_FONT_SIZE` (**grep-gate: the identifier leaves the tree**); one memoized `font_metrics` read replacing all 12 `text_size` sites + the editor/PTY probes; the Fork-B hit-test swap (isolated hunk); ⌘=/⌘−/⌘0 chords + the palette trio; `set_font_size` write-through mirroring `set_theme` (app.rs:1915, best-effort persist — never crash the terminal). |
| `crates/marley_app/src/workspace.rs` | `fallback_cell`'s `else { 13.0 }` → the setting's default (F3). |
| `crates/marley_app/src/keymap.rs` | ⌘= / ⌘− / ⌘0, GLOBAL scope (zoom is app-wide, not editor-buffer). Keys are the raw literal chars `"="`, `"-"`, `"0"` (keymap.rs:775 is emphatic: `"/"` not `"slash"`, with a test pinning the mistake). |
| `crates/marley_app/src/headless_drive.rs` | The drives below. |

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| 003 | `zoom_step` / `clamp_font_size` truth table — [8,32] at both ends; a step from the max stays; 0/negative/NaN/300 → default-or-clamp; **a mid-range assert that is neither 0 nor 1** | pure (cov/MSI 100) |
| 001 | `type_scale` — Command/Output track `font_size`; **Caption/Nav do NOT** (the row that pins D-CONTENT-ZOOMS); weights unchanged | pure |
| 002 | `settings_round_trip_survives_reload` gains a NON-DEFAULT leg (persist 16 → reload → applied 16) | unit |
| 003 | the resolver clamps a hand-edited 300/0/-1 | unit (resolver table) |
| 004 | `zoom_applies_live_and_persists_headless` — ⌘= raises the size, the metrics recompute, the setting is written | headless |
| 005 | `zoom_regrids_every_live_pty_headless` — a size change → the next frame's `plan_resize` yields a NEW (cols, rows) (F6: render-driven, no new plumbing) | headless |
| 006 | `font_family` resolves; an unresolvable family falls back + flashes | unit + headless |
| 007 | **`grep -c TERMINAL_FONT_SIZE crates/` == 0** — the one-seam proof | gate step in validate |
| **F2** | `terminal_hit_test_uses_the_measured_advance` — the isolated Fork-B hunk, size-independent | headless |
| 008 | the palette trio resolves (the existing completeness tests extend) | existing |
| — | LIVE pixel drive **OFF-LIMITS** (chad at the machine) → units + headless + mechanism; deferred, not skipped | uncoverable-live |

### Risks
1. **The 12+6 consumer sweep is the whole ticket.** A missed `text_size` site or raw `px(13.0)` literal is a
   half-zoom that tests will not catch — REQ-007's grep gate catches the const, but not the literals
   (app.rs:4137/4147/4164/4182/4241/14587/14754/15249/15343 and px(22/12/11) at 6978/6984/7013). Inspect gets
   this as an explicit lens: **enumerate every size literal in the render and justify each as chrome or fix it.**
2. **Cell HEIGHT has no measurement at all** (`fallback_cell`'s `size * 1.2` is the only source). Zoom scales
   it linearly, which is probably fine — but it is an assumption, not a measurement, and it is now
   size-variable. Recorded; a real `line_height` probe is a follow-up.
3. `type_scale`'s signature change touches ~18 call sites — mechanical, but it is where a missed content/chrome
   classification hides.

status: Phase 2 — Design PASS; ready for Phase 3 — Implement

## Phase 3 — Implement (PASS 1 of 2 — the pure seams + settings; app.rs's const-death is pass 2)

`cargo check --workspace` clean, zero warnings, `cargo fmt --all`, `git add -N font_zoom.rs`.

### Built (pass 1)
- **`typography.rs`** — `type_scale(role, font_size)`. `Command`/`Output` return `size: font_size`;
  `Caption`/`Nav` keep 11.0/12.0. Stays pure/gpui-free. The doc records that the split was already in the
  enum's own comments and the fn simply wasn't acting on it.
- **`font_zoom.rs` (NEW, pure, no skips)** — `FONT_SIZE_MIN/MAX/DEFAULT`, `clamp_font_size`, `zoom_step`.
  Both total. **`clamp_font_size` answers a nonsense value (0/NaN/negative) with the DEFAULT, not the
  MINIMUM** — a hand-edited `font_size = 0` is a mistake rather than a request for 8pt, and the recoverable
  answer matters here because you cannot read the palette to fix the setting that made the palette
  unreadable. `zoom_step` clamps `cur` FIRST (it arrives from live state a bad file could have seeded).
- **`settings.rs`** — `FontSize: f32 = font_zoom::FONT_SIZE_DEFAULT` + `FontFamily: String`; the #330
  four-wiring; **the clamp lives in `applied_from`**, the established door (beside `code_tab_width`'s
  `.max(1)`); `persist_font_size` / `persist_font_family`; the 3 test ctors updated.
- **`workspace.rs` (F3)** — `fallback_cell`'s guard now reads `font_zoom::FONT_SIZE_DEFAULT`. Its history IS
  the argument: it said 14 until #195 moved the app to 13, went stale, and #230/#223 fixed it by hand. The
  number now lives in exactly one place.
- **`app.rs` (partial)** — the live `font_size` view field, seeded from `applied.font_size`; all 18
  `type_scale` sites threaded.

### Deviations (both forced, both recorded)
1. **`AppliedSettings` lost its `Eq` derive.** `font_size: f32` and floats are not `Eq`. Verified nothing
   needed it (a value snapshot, never a map key; only `PartialEq` is used). The alternative — an integer size
   to keep the derive — buys one trait for an `as f32` at every consumer, and the entire existing metric chain
   (`fallback_cell(f32)`, `TextStyle.size: f32`, `px(f32)`) is already f32. Reason recorded at the derive.
2. **`caption_header` takes the DEFAULT, not a threaded size.** It is a free fn drawing CHROME only, so
   threading a parameter through its callers purely to be ignored would be noise. The call is honest about
   what it is: a fixed role at a fixed scale.

### Confirmation of the Phase-1 finding, from the compiler
Threading the size produced exactly **four `E0503` "borrowed" errors — at app.rs 13585/13637/13741/13954**:
the terminal's command headers and output rows. **Those are precisely the sites F1 said were on the OTHER
font system** — the ones that would have stayed pinned at 13pt while the PTY re-gridded. The borrow checker
independently pointed at the half-zoom. Fixed by capturing `font_size` before the `workspace_mut()` borrow.

### Built (pass 2) — the const is DEAD
- **`TERMINAL_FONT_SIZE` deleted.** All 17 references → `self.font_size` (the live field). REQ-007's proof:
  `grep -rn TERMINAL_FONT_SIZE crates/ --include='*.rs'` returns **2 hits, both PROSE** in doc comments
  explaining why it is gone. The identifier no longer exists as code.
- **The editor row closure captures `font_size` by value** — it is `'static` and cannot borrow `self`, the
  same reason #336's `scroll_x` is captured there. The compiler found it (`E0521`).
- **Fork B's isolated hunk** — the terminal hit-test now uses the MEASURED `cell.w`/`cell.h` instead of
  `fallback.w`/`fallback.h`, with the reasoning at the site: the 0.4% gap at 13pt is PROPORTIONAL, so making
  the size adjustable is exactly what exposes it (~5px across 80 columns at 26pt = a full cell's drift).
  `cell_h` still comes from the fallback because **nothing measures line height anywhere** — recorded, not
  silently inherited.
- **⌘= / ⌘⇧= / ⌘− / ⌘0**, GLOBAL scope (zoom is app-wide; it must work with focus in a terminal). **⌘⇧= is
  bound to the same verb deliberately** — shift-equals IS ⌘+ on a US layout, and gpui reports the unshifted
  key plus the modifier, so a user pressing what they read as ⌘+ has to land somewhere.
- **`set_font_size`** — clamp + best-effort persist, mirroring `set_theme` (app.rs:1915) line-for-line. The
  doc records why the clamp is here as well as at the resolver: **this door is the one a CHORD comes
  through, and a chord repeats** — without it, holding ⌘− walks the size to zero and renders nothing you
  could read to undo it.
- Palette rows `CommandId(16/17/18)` with their real `KeyBinding`s (the field is a `KeyBinding`, not a
  display string — the chords render as keycaps).

### The tripwire that fired, and why that is the system working
`keymap::tests::all_chords_lists_every_binding` failed 66 vs 62 — a deliberate roster guard. Updated to 66
**with each of the four new chords asserted individually**, never by loosening the count. That test exists to
make a chord addition a conscious act, and it did its job.

### One semantic collision, resolved not papered over
`type_scale_terminal_text_clears_legibility_floor` asserted terminal text is ≥12pt "so a future density tune
can't silently shrink it into unreadability". #337 lets a USER pick 8pt — which is not that bug. The floor was
always a guard on the DESIGN's default, never on the user's choice; `font_zoom::FONT_SIZE_MIN` is the
guard-rail for the user's path. Reframed to assert the DEFAULT clears 12pt, with the distinction written down.

**Verification:** `cargo check --workspace` clean, ZERO warnings; `cargo nextest run -p marley --lib` →
**615 passed**; `cargo fmt --all`; §20 clean; `git add -N font_zoom.rs`.

### Still open for Validate (not implement)
`font_family` is wired through settings + `AppliedSettings` but **not yet resolved into the render** (the
fallback-flash path, REQ-006). The size — the ticket's actual payload — is complete end to end.

status: Phase 3 — Implement PASS; ready for Phase 3.5 — Inspect

## Phase 3.5 — Inspect

2 parallel critics (the consumer sweep; state/clamp/metrics + Fork B). Both verified empirically — a
196-cell `zoom_step` cross-product, a real `toml` round-trip probe, a baseline worktree for the detach trap,
and the actual Menlo advance ratio from gpui's text system. Both left the tree clean (615 tests, not the 618
that showed while a probe was live). **Both caught the tree mid-edit** — I was wiring `font_family` while they
read, which is why each opens with a caveat; findings are against the settled state.

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| F1 | **CRITICAL** | **The tree did not compile.** `let mono_font = gpui::font(self.mono_family().as_str())` — `mono_family()` returns an owned `String`, so `.as_str()` borrows a temporary that dies at the semicolon (E0716 ×2). The old `TERMINAL_FONT` const was `'static`, which is exactly why this shape never bit before. | **REAL** (both critics, independently) | Bind the family to a named local first: `let fam = self.mono_family(); gpui::font(&fam)`. |
| F2 | **HIGH** | **gate:2 fails — 2 dead-code errors** (`-D warnings`, and gates.sh:120 bans a blanket `allow`). `persist_font_family` had no caller; `font_size_for_test` was written ahead of its unwritten Phase-4 drive. | **REAL** | `persist_font_family` **deleted** — nothing in the app SETS a family, and that is correct rather than incomplete: the file IS the interface (there is no settings UI). The size has a chord so it needs a persist; the family does not. `font_size_for_test` deleted — Phase 4 re-adds what it actually uses. |
| F3 | **MEDIUM** | **My Fork B comment's physics was BACKWARDS.** It claimed "at 26pt ~5px across 80 columns → drifts a full cell", arguing zoom makes the error worse. Menlo's advance is 1233/2048 = 0.60205 em and gpui scales it linearly, so the cell grows with the pixels: **the error is SCALE-INVARIANT at ~0.27 cells per 80 columns**, and in a fixed pane zooming IN *reduces* it (8pt → 1.14 cells; 26pt → 0.35). My numbers were the 32pt ones and my reasoning inverted. | **REAL — the code is right, the justification was wrong** | Comment rewritten with the stronger true argument: **it is wrong RIGHT NOW at the default 13pt** — a center-of-cell click reads one column right from ~column 146, i.e. any pane wider than ~1143px. Zoom did not create it; it made it impossible to keep ignoring while touching the metric next door. |
| F4 | **MEDIUM** | **The terminal's completion popup was the one content-adjacent surface left behind** — hardcoded `px(13.0)`, no family, and worse: its anchor `top = r.y + r.h - 34.0 - rows_h - 2.0` calibrates against an input row assumed ~30px. That row's height is `font_size * 1.2`, so at 32pt it is 38.4px and the popup's bottom sits ~2px INSIDE it — **and it `.occlude()`s, so it eats clicks on the very line being completed.** `fit = (r.h - 56.0)/POPUP_ROW_H` shares the calibration. | **REAL — and the critic's framing landed: I argued this exact class eloquently at the hit-test, then left these two numbers alone 2000 lines later** | The popup now scales + takes `mono_family()`. The constants are corrected by the DELTA from the size they were calibrated at (`input_grow = (font_size − 13) * LINE_HEIGHT_RATIO`) — exact at 13pt, directionally right elsewhere. And **`1.2` is now `font_zoom::LINE_HEIGHT_RATIO`**, read by both `fallback_cell` and the popup: #337 gave the ratio a second reader, and two copies of a number is how the font default went stale twice. |
| F5 | **MEDIUM** | **`font_family`'s docs contradicted its code, in the same diff.** settings.rs claimed "falls back to the built-in WITH a flash — never a silent wrong font"; no flash exists, and gpui's fallback is a SYSTEM font, not the built-in. app.rs's newer doc said the truth. | **REAL** | The settings doc now describes what the code does: hand-edited only, no chord, no persist; an unresolvable name degrades to gpui's system fallback SILENTLY, and telling the user is a **named cut** needing a resolution probe this path lacks. A doc does not get to make a claim the code doesn't keep. |
| F6 | LOW | **The DEFAULT-vs-MIN line is "defensible but oversold."** The doc states a *mistake-vs-request* partition; the code implements `!is_finite() \|\| v <= 0.0` → default. The tell: **`inf → 13.0` but `1e30 → 32.0`**, though both mean "absurdly large" — and TOML v1.0 permits `inf`/`nan` literals, so it is hand-edit reachable. | **REAL, doc-only** | Deferred to Phase 4 with the tests: the code's behavior is safe either way (the guard's real job — never a zero-size layout — holds), but the prose should describe the rule the code implements. |
| F7 | MEDIUM (unasked) | **Scope creep: `font_family` is a second feature** — no chord, no palette, no test, a dead persist, and it produced 2 of the 4 gate errors. | **PARTLY ACCEPTED.** Not split out, because the setting ALREADY EXISTED in the spec's scope and I had shipped it DEAD (a setting nothing reads is a claim the code doesn't keep — the #331 rule). Cutting it would mean deleting a spec'd key; leaving it dead was the actual error. Now it has a live read path, an honest doc, and no dead write path. The flash is the named cut. | — |

**Rejected / verified clean (recorded — several cost real effort):** totality PASSES (14 clamp inputs + the full
196-cell `zoom_step` cross-product: no panic, all finite, all in [8,32]); `zoom_step` bounds all correct
(`(32,+1)→32`, `(8,−1)→8`, `(13,±inf)→13`, `(NaN,NaN)→13`); **no unclamped path to `self.font_size`** (exactly
two writes: the boot seed from `applied_from` which clamps, and `set_font_size` which clamps);
**the re-grid reaches the PTY with no cache** — chord → `set_font_size` → `cx.notify()` → render →
`fallback_cell` → `em_advance` → `cell` → `plan_resize` → `session.resize`, and gpui multiplies by font_size
at the END of `advance()` so it can never go stale (13→14pt in a 1600×900 pane moves cols 204→189, rows 57→53
→ the resize fires); **the editor chain is CLEAN** — the hypothesized glyph/cell desync does not exist,
because `geom.x0/y0` are MEASURED from a real canvas rather than computed and `gutter_width()` returns a char
count rendered as mono text, so both absorb the scale by construction; **all 18 `type_scale` sites pass the
right size** (Command/Output take the local captured before the `workspace_mut()` borrow); `caption_header`'s
`FONT_SIZE_DEFAULT` is provably safe (`type_scale(Role::Caption, X)` is X-independent for ALL X including
NaN); **no other font source exists** (ui_components + visual_harness have ZERO `text_size`/`font_family`);
**the TOML int→float round trip is sound** — probed against the real `toml` 1.1.2 because `FontSize: f32` is
the app's FIRST float setting (`14`→14.0 ✓, `13.5` ✓, `"14"`→default ✓, writes+round-trips ✓); dropping `Eq`
is safe (never a map key, no `Eq` bounds); app.rs font mutants = **0** (shims skipped) and the **detach trap is
CLEAR** (66 vs 66 at a baseline worktree, and the normalized SET compared, not just the count); §20 empty.

**Per-site classification (critic 1's deliverable):** 14 remaining size literals — 12 verified CHROME
(dock captions, the launcher, the status badges, the context menu), 1 was the completion popup (F4, fixed),
and 1 is the **agent diff overlay** (app.rs ~14873): it renders literal code in the PROPORTIONAL UI font with
no family at all — a pre-existing oddity #337 neither caused nor fixed. Follow-up filed.

**Verification after fixes:** `cargo clippy --workspace --all-targets` CLEAN (the only warning is the
pre-existing upstream `block v0.1.6` future-incompat); `cargo nextest run -p marley --lib` → **615 passed**;
`cargo fmt --all`; §20 empty; app.rs font mutants 0; the tested surface is font_zoom 12 + typography 4.

**Validate owes (critic-supplied, so P4 need not guess):**
- **font_zoom.rs has 12 mutants and ZERO tests (MSI 0%).** All killable, but there is a landmine:
  `clamp_font_size(13.0)==13.0` does NOT kill mutants 5 (`delete !`) and 6 (`<=`→`>`), because at v=13 the
  mutants' early return yields 13.0 — *the correct answer, coincidentally*. **A suite built on the default
  alone leaves both alive; it needs an in-range NON-DEFAULT** (`clamp_font_size(20.0)==20.0`).
  `clamp_font_size(0.0)==13.0` is the mutant-4 killer (`||`→`&&`), so the F6 decision test IS a mutation test.
  `zoom_step(13,1)==14` kills 7–12.
- **typography.rs: 4 listed, 3 viable** — `type_scale → Default::default()` is UNVIABLE (`TextStyle` derives
  no `Default`), the #203/#204 rule again. The signature change added ZERO new mutants.
- F6's doc correction; the D-CONTENT-ZOOMS row (Caption/Nav must NOT track the size).

### ⚠️ CORRECTION — F5's fix was NEVER APPLIED, and this ledger vouched for it (found at Phase 4)

**F5's row above is false as written.** It says the settings doc "now describes what the code does". It did
not: `settings.rs` still carried, verbatim, the exact sentence F5 quotes as the defect —

> "An unresolvable family falls back to the built-in **WITH a flash** — never a silent wrong font."

I wrote the finding, wrote the resolution, and never verified the edit landed. It shipped through Implement
and through Inspect's own "verification after fixes" re-run, because every check I ran afterwards
(`clippy`, 615 tests, `fmt`, §20) was blind to a doc comment's *content*. The gate cannot catch this class
either — a false doc is well-formed Rust.

**And the reason F5 recorded was itself false** (see Phase 4's REQ-006 section): both docs claimed the flash
needed "a resolution probe this path lacks", when gpui has `TextSystem::font_id -> Result`. So a single
finding produced a fix that was never applied AND a justification that was never true, and the ledger
asserted both were handled.

Fixed at Phase 4: `settings.rs`'s doc now states the silent degradation + names #344; `app.rs`'s
`mono_family` doc drops the false capability claim and records the real (scope) reason. **The lesson is not
"check your edits" — it is that a LEDGER is a doc, and it does not get to make a claim the code doesn't keep
either.** The rule I enforced on the settings doc at F5 applies to the row that enforced it.

Captured as `BF-claude-inspect-fix-recorded-but-never-applied-001` +
`PR-claude-doc-only-fix-has-no-verifier-reread-it-001` — **a doc-only fix has no verifier**, because every
tool in the gate (check, clippy, tests, fmt, coverage, mutation, even rustdoc `-D warnings`) passes a comment
that says the exact opposite of the code. A false doc is well-formed Rust. So: re-read the file, don't re-run
the tools. An audit of F1/F2/F3/F4/F6 against the source confirmed those five DID land — the miss was
isolated, but proving that took a grep each and was worth it.

**One refinement, found by using the rule once (worth carrying into the PR's next revision).** The rule says
"grep the defective sentence and prove ZERO hits". Run on the fixed tree it returns **one** hit — at
settings.rs:48, inside my own corrective doc, which QUOTES the old promise so the next reader learns why it
is gone. The rule as written would flag its own fix as a regression. **A presence-grep must prove the hit IS
the defect, not a citation of it** — the mirror image of the standing `PR-claude-grep-for-absence-must-prove-
the-command-ran`. Count is not evidence in either direction; read the hits.

status: Phase 4 — Validate PASS; ready for Phase 5 — Complete

## Phase 4 — Validate

Every obligation the inspect ledger carried forward was discharged, and the phase found one gap of its own
that neither critic nor the design saw.

### Tests added

| Where | Test | What it pins |
|---|---|---|
| font_zoom.rs | `t337_clamp_font_size_table` | REQ-003 — the clamp table, **including the load-bearing in-range NON-DEFAULT row** |
| font_zoom.rs | `t337_clamp_font_size_nonsense_resolves_to_the_default` | the ≤0/non-finite half → DEFAULT (and the `\|\|`→`&&` mutant); the F6 asymmetry pinned deliberately |
| font_zoom.rs | `t337_zoom_step_moves_one_point` | REQ-003/004 — `zoom_step(13,1)==14`, the arithmetic mutants |
| font_zoom.rs | `t337_zoom_step_at_the_bounds_stays` | a step from a bound stays (an out-of-range value would PERSIST and return every boot) |
| font_zoom.rs | `t337_zoom_step_non_finite_inputs` | a non-finite delta is not a step; `cur` clamps FIRST |
| font_zoom.rs | `t337_font_zoom_is_total_over_hostile_input` | totality — 14 inputs × the 196-cell cross-product, all finite + in range, no panic |
| typography.rs | `t337_type_scale_content_zooms_chrome_does_not` | **D-CONTENT-ZOOMS — this test IS the decision** (Caption/Nav must NOT track the size) |
| settings.rs | `settings_round_trip_survives_reload` (+leg) | REQ-002 — the NON-DEFAULT 16.0 leg; also the app's FIRST f32 TOML round trip |
| settings.rs | `t337_applied_from_clamps_a_hand_edited_font_size` | REQ-003 — a hand-edited 300.0 → 32.0, 0.0 → 13.0 |
| workspace.rs | `t337_a_font_size_change_replans_the_pty_grid` | REQ-005 — pure `plan_resize`: a size change yields a NEW (cols,rows) |
| workspace.rs | `fallback_cell` (extended) | F3 — the guard sources `FONT_SIZE_DEFAULT`, never a re-typed literal; `h = size × LINE_HEIGHT_RATIO` |
| headless_drive.rs | `zoom_verbs_move_the_live_font_size_headless` | REQ-001/004 — boot resolves the setting; each verb steps a point **through the real dispatch path** |
| headless_drive.rs | `zoom_verbs_cannot_escape_the_bounds_headless` | REQ-003 — 40× ⌘− floors, 80× ⌘= ceilings (a chord REPEATS; this is why `set_font_size` clamps too) |
| headless_drive.rs | **`zoom_persists_through_the_verb_headless`** | REQ-004's **persist** half — see below |

### The gap Validate found: REQ-004's persist half was asserted by nobody

Re-reading REQ-004 ("applies live **and persists**") against the tests I had just written, the second half was
resting on two facts that never met: the settings round-trip leg proves `persist_font_size` *writes*, and the
verb drive proves the chord moves the *live field* — but **nothing joined them**, and `set_font_size` is a
`#[mutants::skip]` shim, so no mutant could catch the join either. A verb that zoomed the view and silently
never wrote a byte would have shipped green.

**Proven, not assumed** — deleting the persist from `set_font_size` and re-running:
- `zoom_verbs_move_the_live_font_size_headless` → **still PASSED** (the pre-existing test is blind to it)
- `zoom_persists_through_the_verb_headless` → **FAILED: left 13.0, right 15.0**

That is exactly the symptom a user cannot diagnose: the zoom works all session and evaporates at the next
boot. Probe reverted; `grep NEGATIVE-SMOKE PROBE` → 0 files, tree clean.

The new test's first read is taken at a **NON-DEFAULT 15.0 on purpose**. A read after `font-size-reset` sees
13.0 — precisely what an app that never wrote anything also reports — so it is the *pair* (15.0 on disk, then
13.0 after reset) that proves both writes landed. **Same trap as the clamp table's in-range row: an assertion
on the default is an assertion that cannot fail.** Worth recording that this blind spot is INHERITED, not
new — #199's theme picker, the write-through precedent #337 was told to mirror, has the identical gap
(`persist_theme` is tested at the settings level only, and no drive proves the picker reaches it).

### Gate

First run: **14 of 15 PASS** — coverage 100%, **mutation MSI 100% (26 caught / 0 missed)**, miri, visual all
green. One red: **gate:14 rustdoc**, `unresolved link to set_theme` (app.rs:1922) — mine. `set_theme` exists
(app.rs:1977) but a bare `` [`set_theme`] `` resolves in **module** scope, where a method name is not; the
codebase idiom is the explicit path (buffer.rs: `` [`edit_at_selections`](Self::edit_at_selections) ``).
Fixed at the source; both are in `impl RootView` so `Self::` resolves. Swept the diff for every other
intra-doc link — `` [`crate::font_zoom`] ``, `` [`FONT_SIZE_DEFAULT`] ``, `` [`TERMINAL_FONT`] `` (still live
at app.rs:492 — #337 deleted the SIZE const, kept the FAMILY one) — all resolve.

I deliberately let the failing run finish rather than killing it: interrupted `cargo mutants` runs leak
multi-gig temp trees (**#335**, filed for exactly that), and letting it complete surfaced the MSI result so
the re-run fixes everything at once instead of serially.

**The mutation surface, checked rather than assumed.** The `--in-diff` set is 23 CaughtMutant + 3 Timeout +
5 unviable + **0 missed**. Two things worth writing down:

- **`font_zoom.rs`'s mutants really were included** — the `git add -N`-or-they-are-silently-skipped trap did
  not fire (9 of the caught are font_zoom's, and coverage reports the file at 111/111 lines, 100%).
  `typography.rs` contributes exactly ONE mutant and it is **unviable** (`type_scale -> Default::default()`;
  `TextStyle` derives no `Default` — the #203/#204 rule holding again). Note the critic's "4 listed, 3 viable"
  counted the WHOLE file; the gate mutates only CHANGED lines, so the numbers differ legitimately.
- **The 3 Timeouts are `clamp_font_size -> {0.0, 1.0, -1.0}` and gates.sh counts a Timeout as CAUGHT**
  (:258, the Infection/Stryker "a hang IS detection" convention). That deserved a look rather than a shrug,
  because a slow suite could launder a live mutant into MSI 100. **Verified honest**: the mutant log
  (`mutants.out/log/…font_zoom.rs_line_46_col_5_001.log`) shows SIX of my asserts FAILED — the mutants are
  killed decisively. The Timeout label is an artifact: `cargo test` does not fail-fast, so it ground through
  all 626 tests after the failures until the deadline blew. MSI 100 is real here.

  **But it was honest by luck of inspection, and that is a gate hole → #345.** The classifier cannot tell
  "hang = detection" from "slow suite + undetected mutant". The fix is `cargo mutants --test-tool=nextest`
  (the process-per-test model gate:3 already uses — it fail-fasts, so a killed mutant dies in seconds instead
  of timing out), which also relieves the load-flaky baseline and #334's class. Explicitly NOT a floor
  change (§0).

**It took THREE runs, and runs 2→3 are the interesting part.** Run 2 came back `GATE GREEN [diff]`, all 15
PASS — and I did not take the receipt, because while it was finishing I found the F5 doc still promising a
flash that does not exist (see the Inspect correction above). A green gate on a tree containing a false claim
is still a tree containing a false claim; the gate cannot see doc content, so its greenness is not an opinion
about that sentence. Run 3 is the same tree with the two docs corrected.

**Final: `GATE GREEN [diff]` — 15/15.**

```
PASS gate:1 rustfmt          PASS gate:11 shellcheck
PASS gate:2 clippy (-D warnings)  PASS gate:12 no-suppressions
PASS gate:3 tests (nextest + doctests)  PASS gate:13 source-bans (SAST)
PASS gate:7 cargo-audit      PASS gate:14 docs (rustdoc + doc-todos + brand-scrub)
PASS gate:8 cargo-deny       PASS gate:4  rust coverage (>= 100% lines)
PASS gate:9 cargo-machete    PASS gate:5  mutation (MSI >= 100%)  — 26 caught / 0 missed
PASS gate:10 gitleaks        PASS gate:6  miri     PASS gate:15 visual / AX
GATE GREEN [diff]
```

**1488 tests pass** (1487 + the new persist drive). Receipt `b2efb83a80b2c49fa193b2f7b469311e2c6b3c1f`
verified against a live `gate_state_hash` — MATCH, so it is commit-valid for this worktree.

### REQ → evidence

| REQ | Evidence | |
|---|---|---|
| REQ-001 | `zoom_verbs_move_the_live_font_size_headless` — boot resolves the setting into the live field | ✅ |
| REQ-002 | `settings_round_trip_survives_reload` — the NON-DEFAULT 16.0 leg (the app's first f32 round trip) | ✅ |
| REQ-003 | the clamp table + `t337_applied_from_clamps_a_hand_edited_font_size` + `zoom_verbs_cannot_escape_the_bounds_headless` | ✅ |
| REQ-004 | live: `zoom_verbs_move_…`; **persist: `zoom_persists_through_the_verb_headless`** (negative-smoke proven) | ✅ |
| REQ-005 | `t337_a_font_size_change_replans_the_pty_grid` — pure `plan_resize`, a size change yields a NEW (cols,rows) | ✅ |
| REQ-006 | **AMENDED** — the resolvable half ships; the flash is CUT → **#344** | ⚠️ |
| REQ-007 | `grep TERMINAL_FONT_SIZE` → 2 hits, both prose | ✅ |
| REQ-008 | the palette completeness tests — **and mutation proves it**: `delete match arm CommandId(16/17/18) in action_for_command` are all in `caught.txt`, so each row genuinely resolves to its verb | ✅ |

### REQ-007 — the const is dead

`grep -rn 'TERMINAL_FONT_SIZE' crates/ --include='*.rs'` → **exactly 2 hits, both PROSE** in doc comments
(typography.rs:53, app.rs:297). Zero code.

### Live pixel drive — DEFERRED, not skipped

chad is at the machine (verified: two captures 32 min apart showed the frontmost app change into a live
session). Synthetic input lands on the FRONTMOST window, so a drive would type into his session. REQ-001/003/
004/005 are unit- + headless-proven through the real dispatch path; REQ-006 (`font_family` → render) is
mechanism-verified. Re-verify the rendered type scale visually when the machine is free (~30s, no ticket).

### REQ-006 is AMENDED, not verified — and inspect's reason for the cut was FALSE

Checking my own tests against the EARS list rather than against my memory of it: **REQ-006's second half does
not ship.** It reads "fall back to the default family **WITH a status flash** when unresolvable". There is no
flash; an unresolvable family degrades to gpui's **system** fallback, silently. The first half (apply a
resolvable family) does ship.

Leaving that REQ standing while calling the phase PASS would break the exact rule I enforced on the settings
doc at inspect F5 — *a doc does not get to make a claim the code doesn't keep* — with the spec as the doc. So
it is amended in place, struck through, with the cut named.

**And the reason inspect recorded for the cut is wrong.** F5 said the flash needed "a resolution probe this
path lacks". gpui has one: `TextSystem::font_id(&Font) -> Result<FontId>` (gpui-0.2.2 text_system.rs:109),
and the mac backend `?`s `load_family` (platform/mac/text_system.rs:115-131), so an unloadable family really
does return `Err` rather than silently substituting. **The flash is a CHOICE, not a limitation.** Reading a
permissive dep to settle this is adoption, outside the §20 wall (the #336 precedent).

The honest reason to cut it: the flash is a distinct behavior — probe + flash + fallback policy — with its own
design questions (probe at boot, or per-render? what does it say?) and its own tests, and it would land in a
`mutants::skip` render shim. Bolting it on at **Validate** would skip Design and Inspect entirely, which is
the one thing the phase gates exist to prevent. It is not the ticket's payload; the size is.

**Follow-up: #344** — and it carries the corrected reason plus the specific harm the flash is really for: an
unresolvable family lands on a *system* fallback that **may not be mono**, and a proportional font in a
terminal grid breaks column alignment. So #344's fix is not only "tell the user" but "fall back to the
built-in `TERMINAL_FONT` explicitly instead of letting gpui choose".

**This is the third time in this batch that a correct decision carried a wrong justification** (#336 F5
cited `element_offset` when taffy was the carrier; #337 F3's zoom physics was backwards; now F5's "no probe
exists"). The decisions survived review; the reasons did not. The pattern is that a *plausible* reason stops
the review — mine and the critics' — so the reason never gets measured. Worth an AAR lesson at Phase 5.

### Pre-existing, not in scope
The only clippy warning in the workspace is the upstream `block v0.1.6` future-incompat notice. The agent
diff overlay (app.rs ~14873) renders literal code in the proportional UI font with no family — a
pre-existing oddity #337 neither caused nor fixed; follow-up filed.

## Phase 5 — Complete

**Docs (§21).** CHANGELOG under **Added** (the premise-was-false story, the content/chrome split, clamp-at-
every-door, the one-home constants, the hit-test fix, the #344 cut). Architecture record: `app_shell.md` ×2
(the type-scale bullet — which had itself claimed `typography` was "the ONE place" text size lives *while*
`TERMINAL_FONT_SIZE` also held 13, and **both were true, which was the bug** — plus the terminal-font bullet,
whose `>0 guard→14.0` was the stale literal *in prose*); `editor.md` (the #336 ADR's "no third source"
forward-reference resolved: the constraint HELD); `crate-map.md` (`font_zoom.rs` + the `type_scale` signature);
`roadmap.md` (M22 B-a → ✅ SHIPPED).

**Knowledge captured.**
- `AD-claude-content-zooms-chrome-does-not-001` — the durable decision. Its consequences are the point: a new
  role must be classified content-or-chrome at birth, and an overlay near content must derive from
  `font_size × LINE_HEIGHT_RATIO` rather than a calibrated constant (the popup bug, F4).
- `BF-claude-inspect-fix-recorded-but-never-applied-001` + `PR-claude-doc-only-fix-has-no-verifier-reread-it-001`
- `PR-claude-presence-grep-must-prove-the-hit-is-the-defect-001` — found by applying the rule above to its own
  ticket, one minute after writing it.
- AAR `d8d283b2…` submitted: outcome completed, effectiveness **4** (not 5 — a false doc shipped through my own
  inspect and was caught only at Phase 4 by luck of re-reading; the ticket succeeded, the process leaked).

**Follow-ups filed:** **#344** (font_family resolution flash — the REQ-006 cut, carrying the corrected reason
and the real harm: a non-mono system fallback breaks column alignment) · **#345** (gate:5 counts a Timeout as
CAUGHT — verified benign here, but the classifier cannot tell a hang from a slow suite; fix is
`--test-tool=nextest`, explicitly NOT a floor change).

### What this ticket actually taught

Three of the ticket's findings share one shape, and it is not "be more careful":

1. #336 F5 — a correct fix citing `element_offset` when **taffy** was the carrier.
2. #337 F3 — a correct fix whose comment had the physics **backwards** (I claimed zoom worsens the cell-width
   error; it is scale-INVARIANT, and the true argument is stronger: it is wrong *today* at 13pt past ~column 146).
3. #337 F5 — a fix that was **never applied**, whose ledger entry said it was, justified by a capability claim
   ("no resolution probe exists") that was **false**.

In each case the *decision* survived review and the *reason* did not. A plausible reason terminates review —
mine and the critics' — so the reason never gets measured. The mechanical corollary is #337's real lesson:
**a doc-only fix has no verifier.** Every gate — check, clippy, 1488 tests, fmt, coverage 100, MSI 100, even
rustdoc `-D warnings` — passes a comment that states the exact opposite of the code, because a false doc is
well-formed Rust. Re-read the file; don't re-run the tools.

The bright spot is the same one as #336: **the tests found what review missed.** The negative smoke (delete the
persist, watch the pre-existing drive still pass and the new one fail 13.0 vs 15.0) turned "REQ-004 is probably
fine" into proof, and the landmine row (`clamp_font_size(20.0)`) exists only because a suite built on the
default alone leaves two mutants alive *by coincidence*. Assertions on defaults are assertions that cannot fail.

status: Phase 5 — Complete PASS
