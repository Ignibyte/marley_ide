# Overlay card recipe (#318) — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-318-overlay-card-recipe.md
- **Pipeline spec:** 318-overlay-card-recipe.spec.md

## Phase 1 — Plan

- **Request:** `/work next` → top Queue row after #413 archived: TICKET-318,
  extract the shared overlay-card recipe. Born from #312's inspect (F17),
  correctly deferred then because extracting for one caller is a net loss.
- **Classification / tier:** chore, standard work pipeline, one shippable
  slice. UI-affecting but **zero intended delta** — a pure refactor. M20.

### Recall (§18.3)

- **`BF-lsp-hover-extracted-helper-new-mutation-surface-001`** (failures,
  medium) — the ticket's own "watch out", confirmed live in the ledger and now
  binding as D6. Extracting an inline expression from a coverage-excluded
  render closure into a NAMED fn creates a standalone cargo-mutants target
  that no existing test kills → MSI floor red despite unchanged behavior. It
  went red twice on #311 (`token_color`, plus a `markup_to_string` arm). Fix =
  pair each extraction with a direct unit test; enumerate the true target set
  with `cargo mutants --list -f <file>`.
- Prior work on this seam: #311 (hover), #312 (def_picker + the F17 that filed
  this), #313 (completion popup), #323 (code actions), #327 (problems), #393
  (section ＋ menu), context-menu (`menu_origin` itself), #355 (split-pane parity).
- Adjacent prevention rules touching overlays (not blocking, but the family
  this work sits in): `PR-claude-modal-open-dismisses-lower-overlay-keys-001`,
  `PR-claude-mouse-opened-modal-must-close-all-transient-overlays-001`,
  `PR-claude-input-handler-overlay-arms-must-stop-propagation-001`,
  `PR-claude-block-mouse-except-scroll-for-nonmodal-scroll-overlays-001`.
- **#405's standing constraint** (`app.rs:20855-20857`): any overlay drawn from
  that point down MUST be an `OverlayStates` field or it paints behind the
  browser webview. Checked: `agent_launcher` and `fleet_open` both ARE fields
  (`browser.rs:63-80`), which is what makes D2's scope expansion safe.

### Discovery (Explore agent — the ticket is STALE in four ways)

1. **The quoted chain no longer exists.** `TERMINAL_FONT_SIZE` was deleted
   (`app.rs:383` says so outright); live calls are
   `.font_family(self.mono_family()).text_size(px(self.font_size))`. The
   ticket's `fn overlay_card(colors: &ThemeColors)` signature is therefore
   impossible → D1.
2. **The 8-call core is verbatim at 17 sites, not 5** (full inventory in the
   Explore report: hover 15003, completion 15080, def_picker 15331, references
   15396, code_action 15510, file_symbols 15575, symbols 15671, search 15779,
   problems 15910, palette 20676, naming_workflow 20736, naming_pane 20785,
   fleet_dispatch_draft 20814, agent_launcher 20864, finder 20911, history
   20955, fleet 20990).
3. **The 13-call centered-quarter recipe has 5 verbatim users — and they are
   NOT the ticket's five**: palette, **agent_launcher**, finder, history,
   **fleet**. This is what forced D2's two-recipe factoring.
4. **Only 2 of the 5 are named fns.** `hover_card_overlay` (app.rs:14965) and
   `def_picker_overlay` (:15307) are functions; palette/finder/history are
   **inline blocks inside `impl Render for RootView::render`** (:17135).
5. Similarity claims re-measured: def_picker is 15 calls and hover 18 — they
   differ by def_picker omitting `.text_color(colors.foreground)` and hover
   adding `.p_2().gap_1()`. "15-of-16 identical" was close but the deltas are
   named now.
6. Positioning today: hover + def_picker call `menu_origin`; palette, finder,
   history (+ agent_launcher, fleet) are hand-rolled with **no clamping at
   all**. `completion_popup_overlay` deliberately uses `popup_origin` instead
   (documented at :15035) — untouched.
7. All five builders sit inside `mutants::skip` render shims; **no test asserts
   card style**, and the visual baselines dir holds only `harness_selftest.png`.
   The real existing coverage is the pure state tests (palette.rs:237/255/275,
   finder.rs:78/94, editor_nav.rs:224, context_menu.rs:403).

### Prior-art sweep (§20) — two hits, both recorded as decisions

- **`OverlayShell.tsx` in the React POC already IS this refactor** ("The one
  overlay shape"): shared chrome, `OVERLAY_TOP_FRACTION = 1/6`,
  `OVERLAY_ROW_H = 27`, `OverlayRow`, `useOverlaySelection`. Strongest prior
  art; the shape to mirror. It also surfaced the D4 parity drift.
- **gpui 0.2.2 ships `anchored()` + `.snap_to_window()`**
  (`anchored.rs:27/68/74`); its snap block (`:190-205`) is semantically
  identical to our `menu_origin`, plus corner-flip and margin modes.
  `menu_origin` is a reimplementation of a shipped gpui element. **Not adopted**
  → D5, because it would move a unit-tested pure clamp into an untestable
  render element under a 100%-MSI floor. Recorded as a named future option
  rather than silently ignored.
- `marley_ui_components` has no card/panel/popover helper; `Dialog::render`
  shares 3 of 8 calls and no positioning. `corner_radius` used 36× — all in
  app.rs. The abstraction genuinely does not exist.

### Decisions

D1 helper signature is stale → design settles it · D2 two recipes, seven
overlays (agent_launcher + fleet join Recipe B; both are `OverlayStates`
fields) · D3 pure refactor, any delta is a bug · D4 clamp-vs-wrap split STAYS
(structurally coupled to row windowing) and the POC is what gets corrected ·
D5 keep `menu_origin` over gpui `anchored()` for testability · D6 every
extraction ships a direct unit test. Full text in the spec.

**The #312 open question is now closed** — clamp vs wrap was left "decide
deliberately if the pickers are unified"; D4 decides it with a structural
reason (wrapping an unwindowed list selects an unrendered row) rather than
inheriting the split by accident.

## Phase 2 — Design

### The finding that reshapes REQ-003/004 (spec amended, reason here)

Recipe B has **NO `max_h`** — verified at all three ticket sites (palette
:20669-20673, finder :20906-20910, history :20950-20954 carry only
`.left(w*0.25).top(h/6).w(w*0.5)`). Height is content-driven, and the POC's
measured contract says so explicitly (`OverlayShell.tsx` doc: "height is
CONTENT-driven … allowed to overflow the bottom edge"). `menu_origin` clamps a
KNOWN `(menu_w, menu_h)` box; Recipe B has no height to hand it. Converting
onto `menu_origin` would therefore either (a) invent a max-height → a visual
delta at small windows → violates D3, or (b) pass a fabricated height → a
clamp that never fires horizontally anyway, since `left + w = 0.75·w < w` and
`top = h/6 < h` hold BY CONSTRUCTION at every window size. **The ticket's
"convert onto menu_origin's clamp" is dead on evidence** (the #339 pattern —
a locked intention dying because the geometry already guarantees the property).
Replacement: ONE shared, unit-tested PURE geometry helper whose tests PROVE the
containment property, which is what the requirement was actually for.

### Architecture / approach

Two helpers, two testing postures (D7 below):

1. **`overlay_card_chrome(colors: &ThemeColors) -> gpui::Div`** — the 8-call
   core (`occlude / flex / flex_col / bg(surface) / rounded(corner_radius) /
   overflow_hidden / border_1 / border_color(border)`). A FREE FN in `app.rs`
   next to `icon_label` (:1035), following its exact precedent:
   `#[cfg_attr(test, mutants::skip)] // render-shape only — asserted by the
   #318 driven captures`. NOT in `marley_ui_components` for this pure refactor
   (no card helper exists there; `corner_radius` is consumed 36× all in
   app.rs; a cross-crate move mid-refactor widens the blast radius for zero
   behavioral gain — recorded future option).
   **Deliberately excluded from the chrome**: `.text_color(foreground)`
   (def_picker's card doesn't set it — zero-delta forbids adding it),
   font family/size (Recipe B is sans/16px-default per the POC's measurement;
   hover/def_picker are mono/`self.font_size` — the five genuinely do not
   share typography; the ticket's original chain was wrong about this too),
   position/size, hover's `.p_2().gap_1()`, and all handlers.
2. **`overlay_quarter_geometry(bounds_w: f32, bounds_h: f32) -> (f32, f32, f32)`**
   — `(w*0.25, h/6, w*0.5)`, PURE, in `context_menu.rs` beside `menu_origin`
   (the placement home; that file's banner is "PURE, gpui-free"). Direct unit
   tests: exact fractions at two window sizes + the containment property
   (`0 ≤ left`, `left + w_card ≤ w`, `0 ≤ top < h`) at degenerate sizes
   (0×0, 1×1, huge). Exact-value asserts kill the fraction mutants (D6).

**Call-site shape** (7 sites): Recipe B (palette :20670, agent_launcher
:20860, finder :20907, history :20951, fleet :20986):
`let (ox, oy, ow) = overlay_quarter_geometry(bounds.w, bounds.h);` then
`overlay_card_chrome(&colors).absolute().left(px(ox)).top(px(oy)).w(px(ow))
.text_color(colors.foreground).child(icon_label(…))`. Recipe A (hover :14997,
def_picker :15325): keep `menu_origin` + their `W`/`MAX_H` consts + typography
+ hover's padding/mouse handlers; only the 8-call run is replaced by the
helper. Builder-call ORDER changes (chrome first, position after) — gpui style
setters are one-write-per-property here, so computed style is identical; the
captures are the proof regardless (D3).

**§20 reference:** N/A confirmed — internal refactor, no external behavior
matched; unchanged from plan.

### Spec amendments made at design (each with its reason)

- **Scope-In bullet 2 + REQ-003/REQ-004** — reworded off `menu_origin` onto
  the shared unit-tested geometry helper + by-construction containment (the
  finding above).
- **Scope-Out site count** corrected 12 → **10** (17 core-8 sites total; 7
  converted here: hover, def_picker, palette, finder, history, agent_launcher,
  fleet; remaining: completion 15080, references 15396, code_action 15510,
  file_symbols 15575, symbols 15671, search 15779, problems 15910,
  naming_workflow 20736, naming_pane 20785, fleet_dispatch_draft 20814).
- **New D7 (mutation-posture split, refining D6):** COMPUTATIONAL extractions
  (`overlay_quarter_geometry`) get direct unit tests; RENDER-SHAPE extractions
  (`overlay_card_chrome`) follow the `icon_label` precedent —
  `mutants::skip // render-shape only — asserted by driven captures`. The
  BF-001 rule's substance is "no NEW un-killed mutation surface"; a skipped
  render-shape fn creates none, and its correctness proof is REQ-002's pixel
  captures. Enumerate the real target set at implement with
  `cargo mutants --list -f crates/marley_app/src/context_menu.rs -f
  crates/marley_app/src/app.rs`.
- **New D8 (no `max_h` added):** Recipe B's content-driven height IS the
  settled contract (Marley-authoritative). The POC's `maxHeight: 83.3333%`
  (`OverlayShell.tsx:84`) is **POC drift #2** (alongside D4's wrap drift) —
  corrected POC-side in the named follow-up, never here.

### File manifest

- `crates/marley_app/src/context_menu.rs` — add `overlay_quarter_geometry` +
  its unit tests (same-file `#[cfg(test)]`, house style beside
  `menu_origin_clamps` :403).
- `crates/marley_app/src/app.rs` — add `overlay_card_chrome` beside
  `icon_label`; rewrite the 7 call sites. No other files.
- `marley-web`: NO changes (inverted React leg — capture-the-reference only;
  the two POC drifts are the scoped-out follow-up).
- Tests elsewhere: none modified — REQ-006/007 require the existing
  selection + overlay-flip tests to pass UNTOUCHED.

### Regression test plan

| REQ | Proof |
|---|---|
| REQ-001 | Inspect review: the 8-call run appears once (the helper); grep shows no converted site repeating it. |
| REQ-002 | Driven pixel-parity, before/after at identical window+state, compared by `magick … "%[pixel:p{x,y}]"` sampling (border px, bg px, row-text px, origin corner) — the capture matrix below. |
| REQ-003 | `overlay_quarter_geometry` exact-fraction units (two window sizes). |
| REQ-004 | Containment-property units at degenerate sizes (0×0, 1×1, huge). |
| REQ-005 | gate:5 MSI 100 with the D7 posture; `cargo mutants --list` enumeration at implement. |
| REQ-006 | `palette_selection_clamps_at_both_ends` / `move_clamps` / `def_picker_moves_and_wraps` pass unmodified. |
| REQ-007 | `overlay_flip_table_each_of_26_states` passes unmodified. |

**Capture matrix (live app, selftest harness, fixed window):** BEFORE captures
from a pre-refactor build; AFTER from the refactored build; same project,
no status flash. Recipe B: palette ⌘⇧P · finder ⌘P · history ⌘R ·
agent launcher · fleet ⌘⇧E — all keystroke-reachable. Recipe A: hover via ⌘K
on a symbol in the Marley repo itself (rust-analyzer live); def_picker needs a
MULTI-target F12 — attempt a trait method with several impls; **if genuinely
unreachable live, document the exception per §7** (its diff is the same
code-motion the other six prove + its chrome is the identical helper output)
rather than faking a state.

### Risks

- R1 builder-order change → style-field identity; captures prove (D3).
- R2 capture flake (flash/focus) → capture with no flash, fixed window,
  static overlays.
- R3 def_picker live-reachability → contingency recorded above.
- R4 a future 11th/18th site copy-pasting again → the follow-up ticket for
  the remaining 10 sites is filed at complete; the helper's doc names the
  inventory.

## Phase 3 — Implement

### React leg (inverted — captures, not design)

- POC references captured at localhost:5173 (Playwright, css-scale viewport):
  `.playwright-mcp/318-react-{palette,finder,history,launcher}.png` + a
  contact sheet (`318-react-contact.png`), all four visually verified (palette
  rows with keycap chips, finder file list, history mono commands, launcher
  card). One POC quirk hit and recorded: its keydown handlers compare
  `e.key === 'P'`/`'A'` (uppercase) for the shifted chords, so Playwright key
  descriptors must be `Meta+Shift+P`, not `+p`.
- Marley BEFORE captures from the pre-318 binary:
  `scratchpad/318/before/{baseline,palette,finder,history,launcher,fleet}.png`,
  card presence pixel-verified per capture (surface `srgba(25,26,28)` at
  (700,200) + (1192,192) vs baseline; close verified between overlays).
  Hover/def-picker: the D-contingency applies — dwell over the open buffer
  produced no card (doc-comment position; RA hover None) and def-picker needs
  a multi-target F12 real RA rarely emits; both are covered by the code-motion
  identity + the five captures proving the shared chrome's output (recorded
  per §7, not silently skipped).

### The live-drive incident (recorded honestly — it shaped the harness notes)

Driving the user's LIVE workspace bit twice before the captures landed:
1. **Chords silently no-op'd until a `clickat` preceded them** — programmatic
   activation does NOT hand gpui keyboard focus; the harness README says
   exactly this and the first capture loop skipped it (six byte-identical
   PNGs = nothing ever opened). All-identical hashes are now the tell.
2. **A blind typing test corrupted the user's open buffer** — `type:hello` +
   blind backspaces landed in the focused EDITOR (keymap.rs, open in the
   restored workspace), not a terminal. Repaired to pristine via undo-to-floor
   (⌘Z spam converges: the user had no edits of their own, disk stayed clean —
   verified `git status` clean + crop-inspection of the restored lines +
   0-error footer). Fleet also needed its TOGGLE (⌘⇧E) to close — Esc alone
   was unreliable under synthetic delivery.
   Consequence for validate: the AFTER capture run reuses the exact
   clickat-first protocol, editor-area focus click only (caret moves are the
   only safe blind gesture), and verifies overlay-closed between captures.

### Rust (built exactly to the design)

- `crates/marley_app/src/context_menu.rs` — `overlay_quarter_geometry(win_w,
  win_h) -> (left, top, width)` beside `menu_origin`, with
  `overlay_quarter_geometry_fractions_and_containment` (exact fractions at
  2384×1119 + 1000×600; containment property at 0×0 / 1×1 / 10k×10k).
- `crates/marley_app/src/app.rs` — `overlay_card_chrome(colors:
  &ThemeColors)` beside `icon_label` (its mutants posture + the occlude/#221
  rationale folded into the doc); 7 sites converted (hover, def-picker,
  palette, launcher, finder, history, fleet), each preserving its own
  position/size/typography/children byte-for-byte.
- **Deviation (mechanical):** the def-picker's replacement needed
  disambiguation by its centered `menu_origin` preamble — its bare card chain
  is verbatim at **6** sites (the fixed-width family: references, code-action,
  file-symbols, symbols, search, problems share it exactly), which the design
  had under-measured as merely "similar". Strengthens the follow-up ticket's
  case; no behavior impact.
- Verified: `cargo check --workspace --all-targets` clean; fmt clean; clippy
  clean; targeted run green (new geometry test + `menu_origin_clamps` +
  `palette_selection_*` + `move_clamps` + `def_picker_moves_and_wraps` +
  `overlay_flip_table_each_of_26_states` — 7/7, REQ-006/007's tests
  unmodified).

## Phase 3.5 — Inspect

Two critics (small diff): zero-delta correctness · simplification/state. Both
went deep — the correctness critic reconstructed every effective builder chain
against `git show HEAD` and enumerated the actual cargo-mutants target set;
the simplification critic ran an EXHAUSTIVE f32 sweep (all 2,139,095,039
positive finite values) proving the containment assert total.

### Findings ledger

| # | Sev | Finding | Verdict | Fix |
|---|---|---|---|---|
| F1 | MED | The five Recipe-B sites still repeated a token-identical positioned prefix; caller-side positioning buys nothing (none of the 10 follow-up sites use quarter geometry — naming trio is 0.3/h4/0.4, fixed-width six ride `menu_origin`), and single-site `text_color` drift stays possible | **REAL** — accepted with the critic's evidence | `quarter_overlay_card(colors, win_w, win_h)` wrapper beside the chrome (house `icon_label` idiom, same mutants posture); 5 sites converted; the pure geometry fn + its tests untouched underneath |
| F2 | MED | The chrome doc universalized the `.occlude()` rationale, but `PR-claude-block-mouse-except-scroll-for-nonmodal-scroll-overlays-001` brands occlude WRONG for non-modal members over scrollable content (hover — a shipped pre-#318 tension — and the follow-up's completion popup); the 10-site sweep would read the doc and cement the modal choice onto them | **REAL (doc hazard)** | Doc clause added: "this is MODAL chrome … a non-modal member needs a `block_mouse_except_scroll` variant, not this fn". Zero code change (D3: hover keeps its shipped occlude) |
| F3 | LOW | Doc + spec claimed `0 ≤ top < h`; strict inequality is false at h=0 (the test's own degenerate case) — test asserted the weak form correctly | **REAL** | Doc + spec REQ-004 now say `≤` (strict for h > 0) |
| F4 | LOW | context_menu.rs banner under-described the module (9 `menu_origin` consumers, only one a menu; the new fn is overlay-only); name nit: "origin" returns width too | **REAL** | Banner gained the overlay-placement-home line; fn renamed `overlay_quarter_geometry` while renaming was free (docs + lesson swept) |
| F5 | LOW | Spec P5 still said "12 remaining sites" after two corrections to 10; the #405 anchor "app.rs:20855" drifted into the launcher block | **REAL** | s/12/10; the #405 reference de-anchored to "the OVERLAY REGION comment" |
| F6 | LOW (corr) | launcher/history/fleet have ZERO automated drives — the driven captures are the entire evidence for 3 of 7 sites | **REAL — routed to P4** (the capture matrix already covers all five Recipe-B overlays; before-captures banked) |
| F7 | INFO | Working tree carries unrelated docs (the browser intake amendment etc.) | Known — commit stages selectively (user-flagged) |
| — | — | Correctness critic: style-call identity verified per site vs HEAD (no property set twice anywhere — reorder inert); geometry f32-identical; the 6 sibling fixed-width chains byte-unchanged; 33 new mutants all killed by the first exact assert; app.rs coverage-excluded (documented), context_menu.rs fully covered | **clean** | — |
| — | — | Simplification critic checked-and-clean: "17 verbatim sites" exact; def-picker-no-text_color true; 16px-default claim verified against gpui source; containment assert TOTAL (exhaustive sweep; sole equality at w=3·2⁻¹⁴⁹, still ≤); `PR-claude-verify-skip-attr-still-attached-after-inserting-fn-001` honored | **clean** | — |

Post-fix: one clippy doc-lint round (a wrapped `+` read as a markdown list item
— reflowed), then clippy/fmt clean; `overlay_quarter_geometry` + overlay-flip
tests green. Test-extremes bonus from the sweep: `(f32::MAX, f32::MAX)` and
`(3e-45, 3e-45)` joined the degenerate loop.

No F-/PR- ledger appends: zero-delta HELD (no bug shipped or nearly shipped);
all findings were structure/doc hardening. The pre-existing hover-occlude
tension with the block-mouse rule is already recorded by that rule itself.

## Phase 4 — Validate

### Runs (actual)

- Full workspace: `cargo nextest run --workspace` → **2124 → 2125 passed**
  (the draw-smoke joined; 5 pre-existing skips), doctests all ok. REQ-006/007's
  tests (`palette_selection_clamps_at_both_ends`, `move_clamps`,
  `def_picker_moves_and_wraps`, `overlay_flip_table_each_of_26_states`) pass
  UNMODIFIED.
- `overlay_quarter_geometry_fractions_and_containment` green (exact fractions
  ×2 sizes; containment at 0×0 / 1×1 / 10k×10k / `f32::MAX` / 3e-45).
- **REQ-001** (chain uniqueness): normalized count of the 8-call chain in
  app.rs = **11** = the helper + the 10 scoped-out follow-up sites; every
  converted site carries none.
- **NEW: `recipe_b_overlays_open_and_draw_headless`** (closes inspect F6's
  launcher/history/fleet zero-drive gap): each of the five open-verbs fires
  through `dispatch_for_test`, a frame DRAWS with the overlay up (the converted
  `quarter_overlay_card` block renders), and the #405 `OverlayStates` snapshot
  reports the driven overlay. Writing it with a strict five-flag equality
  surfaced a **pre-existing gap**: `close_transient_overlays` does NOT include
  `agent_launcher` (palette/finder/history are in the set; fleet is
  toggle-scoped by design) — the launcher survives a transient-close and
  stacked under fleet. Recorded here for a future ticket (the
  `PR-claude-mouse-opened-modal-must-close-all-transient-overlays-001` family
  suggests the launcher belongs in the set); the shipped test asserts
  per-index so iterations stay independent, per D3 (no behavior change here).

### Gate

First run RED on gate:2 alone (`clippy::type_complexity` on the smoke's
fn-pointer array — restructured to a verb list + per-index asserts, which is
also what surfaced the launcher gap); re-run:

```
══ gate summary (diff) ══
  PASS  gate:1 rustfmt · gate:2 clippy · gate:3 tests · gate:7 audit
  PASS  gate:8 deny · gate:9 machete · gate:10 gitleaks · gate:11 shellcheck
  PASS  gate:12 no-suppressions · gate:13 source-bans · gate:14 docs
  PASS  gate:4 coverage (>= 100%) · gate:5 mutation (MSI >= 100%)
  PASS  gate:6 miri · gate:15 visual / AX
  15 passed, 0 failed
GATE GREEN [diff]
```

Receipt written. The 33 `overlay_quarter_geometry` mutants (27 constant-tuple
+ 6 operator swaps) all die on the first exact assert (verified by the inspect
correctness critic's enumeration; gate:5 green confirms).

### REQ-002 driven AFTER-captures — environmentally blocked, §7 documented

The before-set is banked (5 overlays, card-presence pixel-verified,
`scratchpad/318/before/`). The AFTER run is **blocked by the machine's session
state**, exhaustively diagnosed:

- This mini is **headless by design** (the switch-monitors profile doc): a
  physical monitor is currently attached, so the BetterDisplay virtual screen
  self-disconnects (`display-or` agent enforces physical XOR virtual — the
  `switch-monitors full` attempt correctly refused). Nobody is at the desk:
  the physical display SLEEPS.
- With the display dark, new app instances get a ghost window (no first
  paint, no AX registration, `screencapture -l` fails). `caffeinate -u/-d`
  wakes the compositor (whole-screen capture then works), **but no process can
  become frontmost** in the no-active-user session: SE `set frontmost`
  silently no-ops, `open` activation fails, `NSRunningApplication.activate` →
  `isActive: false`, and synthetic titlebar clicks don't grant focus. Without
  frontmost, chords never reach the app — the capture protocol cannot run.
  (This also retro-explains yesterday's flaky first loop: the display was
  drifting in and out of sleep while Chad's session was live.)
- Substitute evidence, per §7's documented-skip rule (the path literally
  "needs a live GUI runtime" — an INTERACTIVE session): (1) the inspect
  correctness critic reconstructed every converted site's effective builder
  chain against `git show HEAD` — property sets and arguments identical, no
  property set twice anywhere (the reorder is inert under last-write-wins);
  (2) the geometry is bit-identical f32 (same ops, same order) with the
  containment property proven exhaustively over all positive finite f32;
  (3) the new draw-smoke renders all five overlays through the new code path
  headlessly; (4) the full suite + flip table green.
- **PENDING ITEM (carried to complete/commit):** run the scripted after-loop
  (`scratchpad/318/` protocol: activate → clickat-focus → chord → capture →
  probe, five overlays) in the next LIVE session (Chad at the desk or an
  active CRD session) and compare the ~25-point battery per overlay against
  the banked before-set. One command per overlay; the before-values are the
  expected pixels.

### Pre-existing (documented, not in scope)

- `block v0.1.6` future-incompat (upstream). The 5 workspace skips.
- The `close_transient_overlays`-launcher gap (above) — future ticket.
- Hover `.occlude()` vs the block-mouse rule (inspect F2) — recorded there.

## Phase 5 — Complete

- **CHANGELOG (§21a):** `[Unreleased] → Changed` entry — the extraction, the
  died-clamp story, zero-delta evidence, the launcher-gap find, the pending
  after-captures.
- **Architecture docs (§21b):** `docs/marley_architecture/app_shell.md` — new
  "The overlay-card recipe (M20 #318)" section: one modal chrome, two recipes,
  the containment proof, the deliberate selection split, the 10-site remainder,
  the POC drifts.
- **Parity (§21c):** the two POC drifts are TICKET-416 (Marley-authoritative;
  corrected POC-side there). React reference captures banked at implement.
- **Knowledge (§19) — codes:**
  - `L-claude-318-extraction-testing-posture-splits-computational-vs-render-shape-001`
    (design) · `L-claude-318-driving-the-live-app-needs-an-active-user-session-001`
    (here)
  - `PR-claude-never-type-blind-into-the-live-app-001` (here — the keymap.rs
    incident's rule)
  - `AD-claude-318-overlay-chrome-two-recipes-001` (here)
- **Follow-ups filed:** TICKET-414 (the 10-site sweep + the
  block_mouse_except_scroll variant), TICKET-415 (the launcher transient-close
  gap — Queue TOP as a bug), TICKET-416 (POC drifts), TICKET-417 (the #318
  after-capture battery — Deliberate: needs a live session). Queue rows added.
- **Ticket:** TICKET-318 → `tickets/closed/`, status closed (2026-08-12).
- **Archive:** this pair → `docs/planning/pipeline/completed/`.
