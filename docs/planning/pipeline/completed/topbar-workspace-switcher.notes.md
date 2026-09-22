---
pipeline_id: 69541642-66c0-42ee-a098-94349d1282e1
ticket: forge#244 (3f762a4d-e498-4375-82ba-dfc38cdf511b)
aar_id: c3aa9110-dc55-4647-8679-d53252dc0b44
---

# Notes — Make the top-bar focused-workspace indicator a click-to-switch popover (forge#244)

## Plan (Phase 1)

**Classification:** work pipeline, feature, small. 7th ticket of the M14 round (#238 config, #239/#240/#241/
#243/#245 done; #242 editable-editor HELD for chad). Independent follow-on — reuses only SHIPPED machinery
(#233 switch, #235 indicator, #166 overlay). Autonomous auto-approved (M14 /work 238-247).

**Intent:** the #235 focused-workspace indicator is display-only. Make it a click target that opens a popover of
the open workspaces; click a row to switch the focused workspace.

**Discovery (Explore agent — the anchors):**
- **Indicator:** `titlebar::focused_workspace_indicator(name,branch,max) -> String` (titlebar.rs:50); rendered
  as a NON-interactive `div().absolute().top(9).left(topbar_icon_x(3,…)).text_size(13).child(label)` at
  app.rs:6429-6445 (left cluster slot 3; cockpit tabs positioned after at +TOPBAR_INDICATOR_WIDTH=180). Click
  wiring GREENFIELD. Consts TOPBAR_INDICATOR_MAX_CHARS=22 (app.rs:309), TOPBAR_INDICATOR_WIDTH=180 (app.rs:310).
- **Switch idiom (REUSE):** rail-click at app.rs:4548-4551 — `let changed = i != shell.active_project_index();
  let _ = shell.switch_project(i); if changed { sync_active_project(); }` + `persist_grid()`.
  `Workspace::switch_project(idx)->Result<(),TabError>` (tabs.rs:331). `sync_active_project()` (app.rs:2723-2733)
  re-targets Files/file_tree/project_root/files_scroll (no-ops at 0 ws). `cycle_project` (tabs.rs:343) is
  key-only (⌘⇧]/[ dispatch app.rs:3289-3300) with NO RootView wrapper → do NOT reuse it for the popover.
- **List accessors:** `Workspace::projects()->&[Project]` (tabs.rs:373), `active_project_index()->usize` (368),
  `project_count()` (384). `Project { name: String (144), root: PathBuf (146) }` — NO branch field.
  `rail_rows()` (tabs.rs:547-563) is the precedent (iterates projects, `label: name.clone()`, `active: i==idx`).
- **Overlay template (#166):** `context_menu: Option<ContextMenuState>` (field app.rs:197, init None 1094);
  render app.rs:6689-6757 = `inset_0().occlude()` backdrop [Left+Right → None, notify] + a `bg(surface)
  .border_1.border_color(border).rounded(corner_radius)` box + rows `on_mouse_down(Left, listener{
  stop_propagation; act; notify})` painted AFTER the backdrop. `agent_launcher` card (field app.rs:140, render
  5778-5838) is the VISUAL reference (titled card + accent-highlighted selected row + same backdrop).
- **Click idiom:** `.on_mouse_down(MouseButton::Left, cx.listener(move |view,_e:&MouseDownEvent,_window,cx|{ …;
  cx.notify(); }))` — app.rs:6416 (new-agent), app.rs:6504-6511 (cockpit tab, captures a value → closest analog).

**Prior-lesson recall (forge knowledge-search):** top hit = the #229 rule
`PR-claude-fullscreen-dismiss-backdrop-must-close-earlier-overlays` — a full-screen dismiss backdrop silently
HIDES any co-open earlier overlay. → **D4**: opening the switcher closes co-open overlays; the toggle occludes.

**Decisions:** D1 popover (not click-to-cycle) · D2 reuse the rail-click switch idiom (not cycle_project) ·
D3 name-only rows (no branch stored) · D4 close co-open overlays on open + occlude the toggle (#229) · D5 pure
model in titlebar.rs.

**Risks / load-bearing:**
- The switch on a popover row MUST be byte-identical to the shipped rail-click switch (D2) — a divergent switch
  path risks the accessor-panic class (#234). Design pins the exact 3-line idiom.
- Overlay coexistence (D4) — the #229 class. The popover backdrop must not orphan a co-open context_menu/
  agent_launcher; opening resets them.
- The pure model may have 0 viable mutants (a thin enumerate+active-mark, like `type_scale`/`default_for`) →
  the per-case unit is then coverage+regression, not MSI-killing. `cargo mutants --list -f titlebar.rs` at design
  confirms the real set.
- Single-project today → the driven popover lists 1 row. That still proves toggle+render+highlight+dismiss; the
  actual SWITCH is mechanism-verified (byte-identical to the rail click) unless a 2nd project can be opened.

**Test plan (finalized at design):** REQ-001 pure `workspace_switcher_rows` (normal N-row + active-marking;
active out-of-range → none; empty → empty; single row active). REQ-002/003 driven (the click→popover→row→dismiss
sequence + the active-row highlight). `cargo mutants --list -f titlebar.rs` for the new fn.

**AAR:** c3aa9110-dc55-4647-8679-d53252dc0b44 (opened).

**Phase 1 status: Plan PASS — autonomous auto-approved (M14). Ready for Phase 2 — Design.**

## Design (Phase 2)

**Approach.** A pure display model in `titlebar.rs` + a bool state field + a popover overlay mirroring the #166
`context_menu` render EXACTLY (verified against app.rs:6689-6757). No new subsystems; no IO; the switch reuses
the shipped rail-click idiom verbatim. Fits the cockpit render layer (gpui `RootView::render`).

**Confirmed anchors (re-read):**
- Indicator render: app.rs:6429-6445 — a bare `div().absolute().top(9).left(topbar_icon_x(3,TRAFFIC_LIGHT_INSET,
  ICON_GAP)).text_size(13).text_color(foreground).child(label)`. A project is ALWAYS active here (the launcher
  branch returns earlier — comment at 6427) → `active_project()/projects()/active_project_index()` are panic-safe.
- Overlay template: context_menu render app.rs:6689-6757 (backdrop `inset_0().occlude()` Left+Right→None+notify;
  `menu_box` left/top/w + `bg(surface).border_1().border_color(border).rounded(corner_radius).overflow_hidden()
  .text_size(13)`; rows `h(MENU_ROW_H).px_2().flex().items_center().child(label).on_mouse_down(Left, listener{
  stop_propagation; act; notify})`; selected row `bg(background).text_color(foreground)` else `text_color(muted)`).
- Geometry consts: TOP_BAR_H=30 (app.rs:350), MENU_ROW_H=26 (368), MENU_W=150 (367), TOPBAR_INDICATOR_WIDTH=180
  (310), TOPBAR_INDICATOR_MAX_CHARS=22 (309). Field-init region ~app.rs:1094 (context_menu: None).
- Switch idiom (rail-click, app.rs:4548-4551): `let changed = i != shell.active_project_index(); let _ =
  shell.switch_project(i); if changed { sync_active_project(); persist_grid(); }`.

**Decisions confirmed + refined:**
- **D1** popover (not click-to-cycle). **D2** reuse the rail-click switch idiom verbatim (not `cycle_project`).
  **D3** name-only rows. **D5** the pure fn lives in titlebar.rs beside `focused_workspace_indicator`.
- **D4 (refined) — INLINE the transient-overlay resets in the toggle listener, no helper.** On opening the
  switcher, reset the 5 confirmed transient overlays: `context_menu=None; agent_launcher=None;
  palette_open=false; naming_workflow=None; renaming_tab=None;`. Rationale: (a) naming_workflow + renaming_tab
  are INLINE drafts with NO backdrop → a full-screen backdrop would orphan them (the exact #229 class); (b)
  context_menu/agent_launcher/palette are backdrop-overlays → closing them prevents two stacked backdrops. Docks
  (forge/fleet/files_open) are persistent — NOT reset. finder/find/history_open are pane/keyboard-scoped and not
  realistically co-open with a top-bar-indicator click — left alone (a documented v1 bound). Called ONCE (the
  toggle) → inline keeps it in the already-coverage-excluded render closure, avoiding a new masked `fn` +
  its untestable body mutant. **Known-minor (documented):** the REVERSE direction (switcher open, then a
  keyboard-opened overlay) isn't coordinated — the new overlay renders under the switcher backdrop until
  dismissed; low-impact (both clear on Escape/click), a follow-up if it bites. Strictly better than today.
- **D6 — toggle-close via z-order.** The backdrop paints AFTER (above) the indicator (render order: indicator at
  6437, switcher block appended near 6758), so a 2nd click at the indicator location hits the backdrop → closes.
  The toggle handler still sets `open = !open` for robustness (belt-and-suspenders; both paths close).
- **D7 — geometry.** Popover anchored under the indicator: `left(px(topbar_icon_x(3,TRAFFIC_LIGHT_INSET,ICON_GAP)))`,
  `top(px(TOP_BAR_H))` (=30, just below the bar), `w(px(TOPBAR_INDICATOR_WIDTH))` (=180, aligns under it). Rows
  `h(px(MENU_ROW_H))`. Box styled identically to `menu_box`.
- **D8 — active-row highlight.** The active workspace row uses the selected-row treatment (`bg(colors.selection)`
  if present, else the context_menu selected `bg(colors.background)`); non-active `text_color(muted)`. Visual
  intent = read as "this is the focused workspace," matching the rail's active-project highlight. (No check/dot —
  bg highlight only, KISS. Implement confirms the exact token against ThemeColors.)
- **D9 — the toggle div gets `.occlude()`** (match the cockpit-tab container at 6464) so the click doesn't start
  a text-selection / fall through, plus a subtle hover affordance if the sibling top-bar controls have one.

## File Manifest
| File | Change |
|---|---|
| crates/marley_app/src/titlebar.rs | ADD `#[derive(Debug,Clone,PartialEq)] pub struct SwitcherRow { pub index: usize, pub label: String, pub active: bool }` + `pub fn workspace_switcher_rows(names: &[String], active: usize) -> Vec<SwitcherRow>` (doc'd, #244) = `names.iter().enumerate().map(|(i,n)| SwitcherRow{index:i,label:n.clone(),active:i==active}).collect()`. Plus `#[cfg(test)]` tests (Phase 4). |
| crates/marley_app/src/app.rs | (1) `use crate::titlebar::{… , SwitcherRow, workspace_switcher_rows}`. (2) RootView field `workspace_switcher_open: bool` (near context_menu ~197) + init `workspace_switcher_open: false,` (~1094). (3) Indicator div (6436-6444): add `.occlude()` + hover + `.on_mouse_down(Left, cx.listener(|view,_e:&MouseDownEvent,_w,cx|{ let open = !view.workspace_switcher_open; view.context_menu=None; view.agent_launcher=None; view.palette_open=false; view.naming_workflow=None; view.renaming_tab=None; view.workspace_switcher_open=open; cx.notify(); }))`. (4) After the context_menu block (~6758): `if self.workspace_switcher_open { <backdrop inset_0().occlude() Left+Right→open=false+notify> + <popover box left(topbar_icon_x(3,…)) top(TOP_BAR_H) w(TOPBAR_INDICATOR_WIDTH) styled like menu_box> iterating workspace_switcher_rows(&self.shell.projects().iter().map(|p|p.name.clone()).collect::<Vec<_>>(), self.shell.active_project_index()); each row: capture `let i=row.index;` then on_mouse_down(Left, listener{ stop_propagation; let changed=i!=view.shell.active_project_index(); let _=view.shell.switch_project(i); if changed { view.sync_active_project(); view.persist_grid(); } view.workspace_switcher_open=false; notify }); active row bg-highlighted }`. |

Shim (app.rs render + the click closures) is coverage-excluded + `mutants::skip` via the app.rs gate exclusion —
no new masked `fn` (D4 inlined). The ONLY pure seam is `workspace_switcher_rows`.

## Regression Test Plan
| # | Test (`titlebar.rs #[cfg(test)]`) | Proves | Kills |
|---|---|---|---|
| T1 | `workspace_switcher_rows(&["a","b","c"].map(String::from), 1)` == `vec![Row{0,"a",false},Row{1,"b",true},Row{2,"c",false}]` (exact) | REQ-001 normal + active-marking | body→`vec![]`; `i==active`→`i!=active` |
| T2 | `workspace_switcher_rows(&["a","b"]…, 9)` → 2 rows, ALL `active==false` | REQ-001 out-of-range active | `==`→`!=` (would flag none correctly, but T1 already flips); guards no-panic on OOR |
| T3 | `workspace_switcher_rows(&[], 0)` == `Vec::<SwitcherRow>::new()` | REQ-001 empty | body variants |
| T4 | `workspace_switcher_rows(&["x"]…, 0)` == `vec![Row{0,"x",true}]` | REQ-001 single active | body→`vec![]` |
| — | `cargo mutants --list -f titlebar.rs` (post-impl) — confirm the new fn's viable set (predicted: `vec![]` + `==`→`!=`, both killed by T1/T4). If 0-viable, the units are coverage+regression (the type_scale precedent). | MSI 100 | — |
| REQ-002/003 | DRIVEN (shim, coverage-excluded): bundle→open→click indicator→popover (active row highlighted)→click row→dismissed+focused→open→click backdrop→dismissed. READ the PNGs. Single-project → 1-row popover proves toggle/render/highlight/dismiss; the SWITCH is mechanism-verified byte-identical to the rail click (or open a 2nd project via the launcher to prove an actual switch). | REQ-002/003 | — |

**Uncoverable by unit:** the render/click path (gpui `RootView`) — driven-only, per the harness (§7). Documented.

**Risks (load-bearing):** (1) D2 — the switch MUST be byte-identical to the rail click (no new panic path); the
render happens only when a project is active (launcher returns earlier), so `switch_project/active_project_index`
are safe. (2) D4 — overlay coexistence (#229); handled by the inline resets + the documented reverse known-minor.
(3) D7 — the popover must anchor UNDER the indicator on-screen (left=indicator left, top=bar height), not
off-screen. (4) The active-row highlight token (D8) — confirm against ThemeColors at implement.

**Phase 2 status: Design PASS — manifest + test plan locked. Ready for Phase 3 — Implement.**

## Implement (Phase 3)

Built to the manifest; `cargo fmt` + `cargo check -p marley --all-targets` clean (only the pre-existing
unrelated `block v0.1.6` future-incompat warning).

- **titlebar.rs** — added `#[derive(Debug,Clone,PartialEq)] pub struct SwitcherRow { index, label, active }` +
  `pub fn workspace_switcher_rows(names, active) -> Vec<SwitcherRow>` (the exact map/enumerate/collect), beside
  `focused_workspace_indicator`.
- **app.rs** — (1) imported `workspace_switcher_rows` (NOT `SwitcherRow` — never named, type is inferred; avoids
  an unused-import). (2) RootView field `workspace_switcher_open: bool` after `context_menu` + init `false` after
  `context_menu: None`. (3) The indicator div (the #235 block) gained `.occlude()` + a Left `on_mouse_down` that
  toggles `workspace_switcher_open` after inlining the D4 resets (`context_menu/agent_launcher/palette_open/
  naming_workflow/renaming_tab`). (4) A `if self.workspace_switcher_open { … }` block before
  `root.into_any_element()`: the `inset_0().occlude()` Left+Right dismiss backdrop, then a popover
  `left(topbar_icon_x(3,…)).top(TOP_BAR_H).w(TOPBAR_INDICATOR_WIDTH)` styled like `menu_box`, rows from
  `workspace_switcher_rows(project names, active_project_index())`; each row's Left click = `stop_propagation` +
  the rail-click switch (`switch_project` + guarded `sync_active_project` + `persist_grid`) + close + notify.

**Deviations from design (with reason):**
- **D8 → use `rail_highlight` (app.rs:436), not `colors.background`/`colors.selection`.** Discovery found
  ThemeColors has NO `selection` field, and a dedicated `rail_highlight(colors, active)` helper already exists —
  it's the accent-wash (`accent.opacity(0.22)` active / `0.10` hover) the #236 rail uses for the active-project
  row, with a test asserting it's a DISTINCT accent-wash (guards the #219 surface-on-surface-invisible trap).
  The switcher active row now uses `rail_highlight(&colors, true)` + non-active `text_color(muted).hover(bg
  rail_highlight(false))` — byte-identical to the rail row treatment (app.rs:4397-4406). Strictly better than
  the design's fallback: consistent with the rail + proven-visible + REQ-003 renders clearly.
- **No hover cue / cursor on the toggle** — kept minimal (design said "only if siblings have one"; the top-bar
  siblings don't set one). The popover-on-click is the affordance. A hover/cursor cue is a trivial follow-up if
  chad wants it.
- **`.occlude()` on the toggle** — added (matches the cockpit-tab cluster occlude at app.rs:6464); harmless here
  (nothing interactive sits under the indicator) and prevents a stray text-selection.

**Phase 3 status: Implement PASS. Ready for Phase 3.5 — Inspect.**

## Inspect (Phase 3.5)

Lenses: correctness · switch-idiom byte-identity · #229 overlay coexistence · z-order/dismiss · styling ·
borrows/panics/clean-room · mutation. One general-purpose critic (background) + my own skeptical review.

**F1 — [MEDIUM, self-found + FIXED] the switcher backdrop orphans co-open no-backdrop overlays (the #229 class).**
The D4 reset set (context_menu/agent_launcher/palette_open/naming_workflow/renaming_tab) MISSED four transient
overlays that are ALSO no-backdrop and DON'T cover the top bar → reachable by the indicator click while open:
`finder_open` (fuzzy file finder, a card at `left(25%).top(h/6)`, app.rs:5846 — `.occlude()` but NO `inset_0`
backdrop; dismissed by Escape only), `history_open` (cmd-R history, same shape, 5886), `find_open` (the find
bar, 6234), and `completion` (the tab-completion popup, prompt-anchored). With one of these open, clicking the
indicator opens the switcher whose `inset_0` backdrop then covers (orphans) the card until the switcher is
dismissed — the exact #229 orphan the D4 principle ("the switcher is the sole transient overlay when open")
exists to prevent. **Fix:** the toggle now also resets `finder_open=false; history_open=false; find_open=false;
completion=None`. Verified: docks (forge/fleet/files_open) are NOT touched (persistent); compiles clean.
Class = the existing `PR-claude-fullscreen-dismiss-backdrop-must-close-earlier-overlays` (#229) — this is that
rule applied; the miss was enumerating only the OBVIOUS overlays, not grepping EVERY no-backdrop floating one.

**Self-review — the other lenses (clean, verified concretely):**
- **Pure fn correctness** — `workspace_switcher_rows` maps enumerate→row in order, `active` only for `i==active`;
  out-of-range active flags none (a `usize` compare, NO slice index → no panic); empty→empty. ✓
- **Switch byte-identity (no #234 panic)** — the row handler is `stop_propagation` + `let changed = i !=
  active_project_index(); switch_project(i); if changed { sync_active_project(); persist_grid(); }` + close +
  notify — identical to the rail Project-row click (app.rs:4407-4411) and the ⌘⇧] persist behavior. And the
  popover renders only AFTER the launcher early-return (`render` returns `render_launcher` at app.rs:3715-3716
  when `project_count()==0`) → `active_project_index()/projects()` are always non-empty here. No panic path. ✓
- **Z-order / dismiss** — backdrop child appended before the popover child (rows hit-test first via
  `stop_propagation`); the backdrop paints above the indicator (drawn earlier in `render`), so a 2nd indicator-
  location click hits the backdrop → closes; a row click switches + closes without double-firing the backdrop.
  No stuck-open path found. ✓
- **Styling** — active row `rail_highlight(&colors, true)` = `accent.opacity(0.22)` on a `surface` box =
  accent-on-surface (distinct; NOT the #219 surface-on-surface trap; a test at app.rs:6842 asserts the wash is
  distinct). `active_bg`/`hover_bg` computed once, captured by Copy (`Hsla: Copy`) → the per-row `move` hover
  closure is sound. ✓
- **Borrows / clean-room** — `names`/`rows` immutable borrows of `self.shell` are released before the row `move`
  closures (which capture only `i: usize`, Copy). No `unwrap/expect` on a reachable path. Clean-room: reuses
  Marley's own #166 overlay + #233 switch + #219 `rail_highlight`; nothing external copied. ✓
- **Mutation** — `cargo mutants --list -f titlebar.rs`: `workspace_switcher_rows` → `vec![]` (viable),
  `i==active`→`!=` (viable), `vec![Default::default()]` (UNVIABLE — no `Default` derive → won't compile). 2
  viable, both killed by the Phase-4 T1/T4 exact-value cases. NOT 0-viable → cov/MSI 100 achievable. ✓

**Critic verdict (folded):** the general-purpose critic **independently CONFIRMED F1** and sharpened it — there
is a SHIPPED PRECEDENT the switcher should mirror: the `new-agent` top-bar arm (app.rs:3319-3332) closes exactly
`palette_open/finder_open/history_open/find_open/naming_workflow` with an inline comment noting "the 🧠 mouse-
click reaches this arm WITHOUT the key router's overlay guards, so this prevents a stacked palette/finder/history
underneath." The switcher (also a top-bar mouse click that bypasses the key-router overlay guards — routing still
sends keystrokes to finder/history/find/completion, app.rs:3884/3998/4003/4013) had applied HALF that precedent.
My F1 fix added `finder_open/history_open/find_open/completion` → now a SUPERSET of the precedent (also closes
context_menu/agent_launcher/renaming_tab). Critic-rated "defensibly LOW" (Escape-recoverable, uncommon path).
Every other lens: the critic returned CLEAN with concrete cites, concurring with my self-review point-for-point
(pure-fn correctness + no-panic on OOR; the switch byte-identity + the `render` launcher early-return at 3715
guarding zero-project; D4 (a)(b) right set + no dock touched; z-order/dismiss idempotent; `rail_highlight`
accent-on-surface distinct + Hsla Copy; borrows released + no unwrap + clean-room). Mutation: 3 listed, the
`vec![Default::default()]` UNVIABLE (no Default derive) → 2 viable (`vec![]`, `==`→`!=`), Phase-4 kills both;
do NOT add a Default derive to make the 3rd viable (unnecessary).

**Declined nit (critic did not file):** the popover box doesn't `.occlude()`, so a click landing exactly on its
1px border falls through to the backdrop and dismisses. The rows tile the full interior → sub-pixel edge, no
practical impact; the #166 `menu_box` precedent likewise doesn't occlude its box → left as-is for consistency.

**Findings:** 1 real (F1 — MEDIUM-rated by me, LOW by the critic; FIXED + verified compiling; captured as
`BF-claude-mouse-opened-backdrop-overlay-skips-keyrouter-overlay-guards`). No other real findings.

**Phase 3.5 status: Inspect PASS — F1 fixed (mirrors the `new-agent` precedent); critic concurred, all other
lenses clean. Ready for Phase 4 — Validate.**

## Validate (Phase 4)

**Tests added** — `titlebar.rs #[cfg(test)] workspace_switcher_rows_cases` (T1-T4): T1 exact 3-row vec with
active-marking (kills body→`vec![]` AND `i==active`→`i!=active`); T2 out-of-range active=9 → len 2, 0 active
(no panic); T3 empty→empty; T4 single active. **RAN:** `cargo nextest run -p marley workspace_switcher_rows
titlebar` → **9/9 pass** (incl `workspace_switcher_rows_cases`).

**Driven capture (REQ-002/003) — machine UNLOCKED, captured live** (scratchpad/244-*.png):
- **244-a-initial** — the app boots dark; the top bar shows the "Marley · main" focused-workspace indicator
  (left cluster, slot 3). (#243/#245 also re-confirmed live: the 3 editor files buffer.rs/lib.rs/movement.rs
  restored + the rail project row.)
- **244-b-popover** + **244-b-crop** — clicking the indicator OPENS a popover dropped just below it (at the
  indicator's left, `top(TOP_BAR_H)`), listing one row **"Marley"** (name-only, per D3 — the indicator keeps
  "Marley · main"). The crop shows the row carries the **teal `rail_highlight` accent-wash** (the active
  workspace), clearly distinct from the dark surround → **REQ-002 toggle+list + REQ-003 active-highlight**. ✓
- **244-c-rowclick-dismissed** — clicking the "Marley" row switches (single project → self) and CLOSES the
  popover; still "Marley · main" → **REQ-002 row-switch + dismiss**. ✓
- **244-d-backdrop-dismissed** — re-open, then click the editor (backdrop) → popover CLOSES → **REQ-002
  backdrop dismiss**. ✓
  (Single project today → the popover lists 1 row; the actual cross-workspace SWITCH is mechanism-verified —
  the row handler is byte-identical to the shipped #233 rail-click, critic-confirmed. A 2nd project would prove
  a visible switch; not needed for the AC.)

**Gate** — `git add -A && scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** — cov 100 + MSI 100 on
titlebar.rs (`workspace_switcher_rows`: the 2 viable mutants `vec![]` + `==`→`!=` killed by T1/T4; the
`vec![Default::default()]` is UNVIABLE — no Default derive — and doesn't count). The app.rs render/toggle/
popover shim is coverage+mutation excluded. No pre-existing failures in scope.

**Phase 4 status: Validate PASS — 1 unit (4 cases) + full driven REQ-002/003, gate green. Ready for Phase 5 —
Complete.**

## Complete (Phase 5)

- **Docs (§21):** CHANGELOG.md `### Added` (top) — "Click the top-bar workspace indicator to switch
  workspaces". app_shell.md M14 — a #244 bullet (the `workspace_switcher_rows` model, the popover mirroring
  #166, the #233 rail-click switch, `rail_highlight`, the D4/#229 close-all-transient-overlays incl the
  finder/history/find/completion additions, single-Workspace scope).
- **Knowledge:** `aar-submit c3aa9110` outcome=completed, effectiveness 5 (the pipeline WORKED — my self-review
  caught the real F1 #229-orphan, the independent critic CONFIRMED it and sharpened it by citing the shipped
  `new-agent` precedent to mirror; every other lens clean; driven-validated live incl the accent-wash). Failure
  + rule recorded at inspect: `BF-claude-mouse-opened-backdrop-overlay-skips-keyrouter-overlay-guards`
  (22da54ed) + `PR-claude-mouse-overlay-mirror-new-agent-close-set-001` (0759c916); the F1 fix embodies the
  existing #229 rule `PR-claude-fullscreen-dismiss-backdrop-must-close-earlier-overlays` (materialized).
- **Close:** forge #244 (`3f762a4d`) → done; TICKET-244 open→closed.
- **Archive:** spec status `Phase 5 — Complete PASS`; the spec+notes → `pipeline/completed/`.

**Phase 5 status: Complete PASS. Run `/commit` to deliver.**
