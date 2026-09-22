# 393-section-plus-menu — Notes

## Phase 1 — Plan (drafted 2026-07-22, /spec batch, Fable)
- **Request:** chad — "We will expand that + button to include various things (i think)". The "(i
  think)" uncertainty is about FUTURE items; the v1 shape (a small reuse-only menu) is concrete
  enough to spec — the menu IS the extension point, the future items stay documented-not-built.
  Sprint #38 M27; forge #393 `dd98405c-ec53-4aba-8376-8f4e434520c5`.
- **Ground truth:** #387 shipped the ＋ (hover-reveal, stop_propagation, `section_action` routing,
  `dispatch_section_action`, the persist-the-switch F1 fix). The #166/#175 context-menu machinery
  (`MenuKind` + `run_context_menu_action` + one-modal + esc/click-away) is the shipped menu system —
  reuse it, never a second popover.
- **Verb inventory confirmed shipped:** `new_terminal_pane`, `spawn_terminal_tab_in` (cwd, #294/#281),
  `start_file_finder`, the #246 `finder_split` path, `open_or_switch_cockpit` (3 sections),
  `split_focused_pane` (2 axes). All reuse-only.
- **Sequencing note:** independent of #390-392; if #390 lands first the menu gains the Panes arm,
  else the arm gates on the 3-section reality (design handles both).
- **Known follow-up context:** the #387 noted "Editor＋ finder defaults to editor-open" idea remains
  OUT (new finder logic); a menu item could later carry it.
- **Prior-art sweep:** in-repo #166 machinery decisive; IDE toolbar-menu convention. Recorded in spec.
- **AAR:** opened at promotion.

## Phase 1 — Promote to active (Opus, 2026-07-22, /goal /work 390-395)
Promoted queued → `active/` (FOURTH of the sprint; deps **#387 shipped**, **#390 shipped `6450c61`** →
the Panes arm is live, not gated; **#392 shipped `3f18f01`** → active/ was clean). §3 re-confirmed —
393 is the sole active spec. **AAR opened:** `c91eb602-fa59-4414-9bb0-82ac683758ff`.

- **Recall for the designer (knowledge-search "context menu MenuKind anchored dismiss modal
  exclusivity"):** top prevention rules to pull via `knowledge-explain` in Phase 2 —
  `3df9fa6e-b9e4-4551-9b1a-8803b5558e87`, `739601c4-643b-4a03-b50a-bca0b8e1b199`,
  `31c07755-98ad-46e1-a9ca-9c8b6c555a36` (+ failure `d89b8d9e-c5b0-43a8-a515-afce962afa05`). These
  cluster on context-menu/dispatch/modal-exclusivity — feed them to the modal-exclusivity + anchor-drift
  critics at inspect.
- **The #390 landing changes the Panes arm from conditional to live:** `section_menu(RailSection::Panes)`
  returns the 2 axis items unconditionally (Panes is a rendered section now). The design's "gates on the
  3-section reality" branch is moot — keep the code total (a `section_menu` arm per RailSection variant,
  the compiler enforces exhaustiveness), but the Panes arm is always populated.
- **UI-affecting** (the ＋ opens a menu) → the design MUST plan a driven capture. Synthetic INPUT is
  env-blocked this session (bare binary, no Accessibility perm) → fall back to the #390 pre-seed technique
  (`HOME=$ISO` `.marley/config/settings.toml [workspace] shell=<blob>` → launch → `screencapture -l`) for
  a menu-OPEN capture, and units + the #166 mechanism for the item-dispatch (the menu render/dispatch is
  the masked shim; the pure `section_menu` table carries cov/MSI 100). State this explicitly in the design.

## Phase 2 — Design (Opus, 2026-07-22, /goal /work 390-395)

### D4 sizing — ONE coherent slice (no split)
Bounded: extend the ONE existing pure menu module (`context_menu.rs`) by one `MenuKind`, widen a Copy
enum (`SectionAction`, tabs.rs) by 2 variants + 1 payload, rewire ONE ＋ handler, add ONE dispatch
helper. Reuse-only (every verb ships). NOT a totality audit. Confirmed single slice.

### Architecture — the DECISIVE finding: fold into the #166 machinery via `MenuAction::Section`
The context menu (`context_menu.rs`, PURE/gpui-free) is entirely driven by three seams: `MenuKind` (which
menu) → `items_for(kind) -> &'static [(MenuAction, &str)]` (const row tables) → `ContextMenuState{x,y,kind,
selected}` (the shell) → `run_context_menu_action(MenuAction)`. The **render loop** (app.rs:19280 `for
(i,(action,label)) in menu.items()`), the **keyboard nav** (15334-15345 up/down/enter→`run_context_menu_action(
menu.action())`), **esc/click-away dismiss** (15332/19256/19263), and **one-modal** ALL key off exactly
those seams. So the section menu becomes a true PEER of Split/Block/FileRef by adding ONE `MenuKind` +
teaching `items_for` its rows — and inherits render + keyboard + dismiss + one-modal FOR FREE. This is the
literal realization of spec D3 ("the #166 machinery, not a new popover; one menu system").

**D-SECTION-VIA-MENUACTION (the key decision — a REFINEMENT of the spec's placement, recorded loudly):**
the spec drafted `section_menu(section) -> Vec<(label, SectionAction)>` in **tabs.rs** as a parallel table.
The better realization (honoring D3 harder + the items_for precedent + zero allocation): the section rows
live in **context_menu.rs** as `&'static` const tables of `(MenuAction::Section(SectionAction), &str)`, and
`items_for` gains a `MenuKind::Section{p,section} => section_items(section)` arm. `SectionAction` stays the
verb currency (tabs.rs, widened), wrapped by a new `MenuAction::Section(SectionAction)` (both `Copy` →
derives clean). A parallel `section_menu`+parallel-render would DUPLICATE the render/keyboard/dismiss — the
D3 anti-goal. This is the "existing machinery already owns it" win the prior-art sweep is for. (SectionAction
stays in tabs.rs next to RailSection — the rail vocabulary; the menu tables are the menu presentation.)

**The target rides the KIND (the load-bearing #175 pattern, app.rs:6806-6811):** `MenuKind::Section{p:usize,
section:RailSection}` carries the PROJECT — so a #173-pump project/focus shift while the menu is open can
NEVER retarget the dispatch (exactly the reason Block carries pane/block, not live focus). `run_context_menu_action`
captures `p` from the kind BEFORE clearing `context_menu` (mirrors the `target`/`file_ref` capture).

### The verbs + the row tables (all reuse-only; item 0 = the #387 default per section)
| Section | Item 0 (#387 default) | Item 1 | Item 2 |
|---|---|---|---|
| Editor | "Open File…" → `OpenFile` (`start_file_finder`) | "Open in Split" → `OpenFileSplit` (finder + `finder_split=true`, the #246 `split-right-file` path) | — |
| Terminal | "New Terminal" → `NewTerminal` (`new_terminal_pane`) | "New Terminal at Root" → `NewTerminalAtRoot` (`spawn_terminal_tab_in(&root)`) | — |
| Browser | "Forge" → `OpenCockpit(Forge)` | "Agents" → `OpenCockpit(Agents)` | "Details" → `OpenCockpit(Details)` |
| Panes | "Split Right" → `SplitFocused(Horizontal)` | "Split Down" → `SplitFocused(Vertical)` | — |

**D-TERMINAL-ITEMS (a spec reconciliation):** the drafted "New Terminal / New Terminal **Here**" pair
COLLAPSES against the real code — `new_terminal_pane` ALREADY inherits the focused pane's live cwd (#281),
so a "Here" item would be byte-identical to the default. The distinct, useful, reuse-only pair is **"New
Terminal"** (item 0 = the #387 default, inherits the focused cwd, unchanged) + **"New Terminal at Root"**
(`spawn_terminal_tab_in(&project_root)` — a fresh terminal at the project root regardless of the focused
cwd). Item 0 preserves muscle memory (the ＋'s primary); item 1 exposes the root variant. Verified: the
drafted spec's mental model ("new_terminal_pane opens at root") was a misread — the design corrects it.

**SectionAction (tabs.rs) widening:** `NewTerminal`, `OpenFile`, `OpenCockpit(RightSection)` stay;
`SplitFocused` → **`SplitFocused(PaneAxis)`** (PaneAxis @ layout.rs:14 is pure + `Copy` — import it); add
`NewTerminalAtRoot`, `OpenFileSplit`. The #390 `SplitFocused` unit → `SplitFocused(PaneAxis::Horizontal)`
at its one dispatch site + the #390 test. `section_action(section)` (the #387 single-verb fn) is now DEAD
(its only caller, `dispatch_section_action`, is replaced) → **removed** (§0 no dead code); its
`section_action_routing` test → replaced by the `section_items` table tests in context_menu.rs.

### The dispatch (app.rs, masked shim)
- **Rewire the ＋** (app.rs:16616): the `on_mouse_down` was `view.dispatch_section_action(p, section)` →
  now `view.open_section_menu(p, section, e.position.x, e.position.y)` (uses the event position — currently
  ignored as `_e`) + `cx.stop_propagation()` (unchanged — still beats the #386 collapse). `open_section_menu`
  closes the other overlays first (palette/finder/history/rename — the #181 one-modal discipline) then sets
  `context_menu = Some(ContextMenuState::with_kind(x, y, MenuKind::Section{p, section}))`.
- **`run_context_menu_action`**: capture `section_p = match kind { Some(MenuKind::Section{p,..}) => Some(p),
  _ => None }` (alongside the existing `target`/`file_ref` captures, before the `context_menu=None` clear);
  add `MenuAction::Section(sa) => { if let Some(p) = section_p { self.dispatch_section_verb(p, sa); } }`.
- **`dispatch_section_verb(&mut self, p: usize, sa: SectionAction)`** (widened from `dispatch_section_action`'s
  body — per-ITEM, not per-section): activate-first (`switch_project(p)` + `sync_active_project` on change)
  then match `sa` → the verb, with the #387-F1 persist-the-switch rule (`if project_changed { persist_grid }`)
  on EVERY arm uniformly (OpenFile/OpenFileSplit persist on switch; the others persist internally). Remove
  the old `dispatch_section_action`.

### Reference (§20) — confirmed
Convention (IDE section-toolbar menus); the in-repo #166/#175 machinery is decisive prior art (adoption, not
a copyleft read). N/A for Warp/Zed source. Holds.

### File manifest
- `crates/marley_app/src/context_menu.rs` — `MenuAction::Section(SectionAction)` variant; `MenuKind::Section
  {p:usize, section:RailSection}` variant; 4 `SECTION_*_ITEMS` consts + `section_items(RailSection)` fn +
  the `items_for` arm; imports `crate::tabs::{RailSection, SectionAction}`, `crate::right_dock::RightSection`,
  `crate::layout::PaneAxis`; the pure tests.
- `crates/marley_app/src/tabs.rs` — `SectionAction`: add `NewTerminalAtRoot`, `OpenFileSplit`; `SplitFocused`
  → `SplitFocused(PaneAxis)` (import PaneAxis); REMOVE `section_action` fn + its `section_action_routing`
  test (moved to context_menu.rs as `section_items` tests).
- `crates/marley_app/src/app.rs` — rewire the ＋ `on_mouse_down` → `open_section_menu`; add `open_section_menu`
  + `dispatch_section_verb`; `run_context_menu_action` gains the `section_p` capture + the `MenuAction::Section`
  arm; the `NewTerminalAtRoot` (`spawn_terminal_tab_in(&root)`) + `OpenFileSplit` (finder + `finder_split=true`)
  wiring; the #390 `SplitFocused` site → `SplitFocused(PaneAxis::Horizontal)`; REMOVE `dispatch_section_action`.

### Regression Test Plan
| REQ | Test (in context_menu.rs unless noted) | Kind |
|---|---|---|
| REQ-001 | `section_items(section)` returns the right rows per section (Editor 2, Terminal 2, Browser 3, Panes 2) with item 0 = the #387 default (`OpenFile`/`NewTerminal`/`OpenCockpit(Forge)`/`SplitFocused(Horizontal)`); `items_for(MenuKind::Section{..})` routes to it | pure unit (cov/MSI 100) |
| REQ-001 | each `SECTION_*_ITEMS` label is non-empty + the action wraps the intended `SectionAction` | pure unit |
| REQ-002 | `dispatch_section_verb` routing — the pure decision is `section_items`; the dispatch is the masked shim (mechanism: mirrors the shipped `dispatch_section_action` body, per-item; activate-first + persist-on-switch identical) | unit + mechanism |
| REQ-003 | esc / click-away dismiss the Section menu without dispatching | the #166 machinery test (unchanged; a `MenuKind::Section` state dismisses like any) |
| REQ-004 | the ＋ click opens the menu and never fires the #386 collapse | the #387 `stop_propagation` mechanism (unchanged) + drive |
| REQ-005 | the deferred items (SSH/#87, workflows/#204, URL/Phase-E, New File) are documented as the extension point | doc review (spec Out + a code comment on `section_items`) |

**Mutation:** trace the REAL list via `cargo mutants --list -f context_menu.rs` for `section_items` + the
widened `SectionAction`/`MenuAction` — an enum-returning fn's body mutant viability depends on a `Default`
derive (the #203/#204 lesson); `section_items -> &'static [...]` returns a slice (no Default) → likely only
the match-arm/return mutants; kill each with the per-section asserts.

**Driven capture (UI-affecting, honest plan):** the menu is TRANSIENT state (`context_menu=Some`), NOT
persisted — so the #390 pre-seed technique CANNOT stage it, and synthetic INPUT (the ＋ click) is env-blocked
(bare binary, no Accessibility). Plan: (a) at validate, ATTEMPT a headless render — if `marley_visual_harness`
/ a `#[gpui::test]` can construct a RootView, set `context_menu = Some(…MenuKind::Section…)`, and
`render_to_image`, capture the open Section menu + READ it; (b) ELSE document env-blocked + verify via the
pure `section_items` units (row content) + the #166 render MECHANISM (byte-identity: Section flows through
the IDENTICAL `menu.items()` render loop already visually proven for Split/Block — only the row-source
differs, which the units cover). State which path was taken, never silently skip.

### Risks / decisions
- **D-SECTION-VIA-MENUACTION** (above) — the placement refinement; reversible (could split to a parallel
  table) but load-bearing for D3. Recorded.
- **D-TERMINAL-ITEMS** (above) — the "Here"→"at Root" reconciliation; the drafted label was a code misread.
- **Inspect critic lenses** (from the flagged PRs `3df9fa6e`/`739601c4`/`31c07755` + failure `d89b8d9e` +
  the code): modal-exclusivity (Section-menu open closes palette/finder/rename; one at a time); the
  target-rides-kind (p captured from kind, not live focus); persist-on-every-arm (the #387-F1 rule uniform);
  anchor geometry (menu_origin clamps the left-rail ＋'s menu inside the window); the #390 `SplitFocused`
  ripple (no missed dispatch site); `section_action` fully removed (no dead code, no stale caller).
- **Menu-on-collapsed-header:** opening the ＋ menu on a COLLAPSED section still works — the ＋ is on the
  header row (always rendered), independent of the section's expanded rows; the menu anchors at the header.

## Phase 3 — Implement (Opus, 2026-07-22, /goal)
Built per the manifest; `cargo check --workspace` + `--tests` green, `cargo fmt` clean, the marley suite
**831 pass** (the 830 baseline − the removed `section_action_routing`; the new `section_items` tests are
Phase-4 per discipline). Implemented EXACTLY the D-SECTION-VIA-MENUACTION design — no deviations.

- **context_menu.rs (the pure core):** `MenuAction::Section(SectionAction)` + `MenuKind::Section{p:usize,
  section:RailSection}`; four `SECTION_*_ITEMS` consts (Editor 2 / Terminal 2 / Browser 3 / Panes 2, item 0
  = the #387 default); `section_items(section) -> &'static [(MenuAction, &'static str)]` (the items_for
  idiom) + the `items_for` arm; imports `PaneAxis`/`RightSection`/`RailSection`/`SectionAction` (all pure —
  no cycle). A doc comment lists the DEFERRED extension-point items (SSH/#87, workflows/#204, URL/Phase-E,
  New File) per REQ-005. **The render (app.rs:19280 `menu.items()`), keyboard nav, esc/click-away, and
  one-modal all pick up the Section kind for FREE** — zero render/keyboard changes needed.
- **tabs.rs:** `SectionAction` gained `NewTerminalAtRoot` + `OpenFileSplit`; `SplitFocused` → `SplitFocused
  (PaneAxis)` (imported `crate::layout::PaneAxis`); `section_action` fn + its `section_action_routing` test
  REMOVED (dead — grep-confirmed the only callers were the removed dispatch + test). A comment marks the
  supersession.
- **app.rs:** the import dropped `section_action`; the ＋ `on_mouse_down` rewired
  (`_e`→`e`) to `open_section_menu(p, section, f32::from(e.position.x/y))` + `stop_propagation` (unchanged);
  `open_section_menu` (closes palette/finder/history/find first — the #181 one-modal — then sets
  `context_menu = Some(with_kind(x,y,MenuKind::Section{p,section}))`); `run_context_menu_action` gained the
  `section_p` capture (rides the KIND, alongside `target`/`file_ref`, before dispatch) + the
  `MenuAction::Section(sa) => dispatch_section_verb(p, sa)` arm; `dispatch_section_verb(p, sa)` (widened
  per-item from the old body — activate-first + the #387-F1 persist rule on every arm; `NewTerminalAtRoot`
  = `spawn_terminal_tab_in(&root)`, `OpenFileSplit` = the #246 `finder_split=true` inline); the old
  `dispatch_section_action` REMOVED. No stale `SplitFocused` unit-form remained (grep-verified).
- **Reuse-only (D1) held:** every verb is pre-existing; the only new logic is routing + menu-open glue.

## Inspect (Phase 3.5) — 2026-07-22 (Opus, /goal)
Three independent critics over the diff (context_menu.rs +89 / tabs.rs / app.rs). Lenses: (1) verb-dispatch
correctness + the #387-F1 persist rule + item-0 muscle-memory, (2) modal-exclusivity + the "it's just another
MenuKind" machinery-reuse bet, (3) dead-code / exhaustiveness / simplification. **2 real MED findings fixed;
1 LOW handed to validate.** cargo check + fmt clean, 831 tests still pass.

**F1 [MED, CONFIRMED, FIXED] — the `SplitFocused` arm loses a cross-project switch.** (Critic 1, + my own
pre-check.) `SectionAction::SplitFocused(axis) => self.split_focused_pane(axis)` delegated all persistence
to `split_focused_pane`, which persists internally EXCEPT on its #392 no-terminal early-return
(`if try_workspace_mut().is_none() { return; }` — skips the `persist_grid()`). So a cross-project Panes＋
Split on a no-terminal project would run `switch_project(p)` + `sync_active_project` (in-memory) then hit the
guard and return WITHOUT persisting → the active-project switch is lost on relaunch. It's the #387-F1 class
(`PR-claude-cross-project-affordance-must-persist-the-switch-001`) and it broke `dispatch_section_verb`'s own
docstring's "uniformly" claim. Latent today (a no-terminal project is guard-forbidden until #395) but the code
already carries the #392 guard and #395 is the very next ticket. **Fix:** the arm now does
`self.split_focused_pane(axis); if project_changed { self.persist_grid(); }` (mirrors OpenFile/OpenFileSplit;
a rare harmless double-persist in the success case). → `BF-claude-section-menu-splitfocused-skips-persist-on-noterminal-earlyreturn-001`.

**F2 [MED, CONFIRMED, FIXED] — `open_section_menu` opened keyboard-dead under an editor overlay.** (Critic 2,
verified concretely.) It closed only palette/finder/history/find, but the key router is one linear function
where ~12 editor-overlay arms (naming_workflow, def_picker, open_references, code_action_menu, signature_card,
open_symbols, open_search, open_problems, …) run BEFORE the `context_menu` arm and `return`. Critic 2 proved
realism: those pickers are single CENTERED cards (`.absolute()`, no full-window `inset_0` scrim — only the
context_menu itself has the backdrop), so the LEFT-rail ＋ is NOT occluded → focus an editor, trigger
multi-result Go-to-Definition / ⌘T symbols / a save-as-workflow draft, then click a rail ＋ → the section menu
renders but the keyboard belongs to the picker (the `naming_workflow` case even leaks typed chars into the
hidden draft). The #181 agent launcher — the sibling MOUSE-opened modal — already closes exactly these 12 for
this class (findings #229/#312/#317/#323/#324/#325/#326/#327). **Fix (the DRY one Critic 2 recommended):**
extracted `close_transient_overlays(&mut self)` (the launcher's 13 assignments, one source of truth), rewired
BOTH the #181 launcher-open (behavior-identical — same assignments) AND `open_section_menu` to call it. →
`PR-claude-mouse-opened-modal-must-close-all-transient-overlays-001` (a modal opened by a mouse click bypasses
the key router's overlay guards → it must close every overlay whose key-arm precedes it, or it opens
keyboard-dead; use the shared helper).

**F3 [LOW, handed to validate] — no `section_items` test yet.** (Critic 3.) The removed `section_action_routing`
test covered the old pure routing; the tombstone comment claims `section_items` is "tested there" but no test
constructs `MenuKind::Section` / calls `section_items` yet. Expected at inspect (validate writes tests) — but
validate MUST add the `section_items`/`items_for(MenuKind::Section{..})` 4-arm test + item-0-preserves-#387-default,
or the pure routing is a net coverage regression. Already in the Phase-4 plan (REQ-001); reconfirmed.

**Clean on:** item-0 = the #387 default per section (byte-identical muscle memory — Critic 1); OpenFileSplit =
the existing `split-right-file` behavior + persist (superset — Critic 1); NewTerminalAtRoot reads project p's
root AFTER the switch (Critic 1); Panes axis mapping Right=Horizontal/Down=Vertical matches the #166 convention
(Critic 1); the render/keyboard/dismiss/anchor-clamp all handle MenuKind::Section for free via items()/action()/
menu_origin (Critic 2); section_p rides the SAVED kind, no #175 retarget (Critic 2); stop_propagation still
beats the #386 collapse (Critic 2); section_action + dispatch_section_action FULLY removed, SplitFocused
widening total (4 sites), the match exhaustive with NO catch-all, one menu system (no parallel path), no
unwrap/expect, no orphaned imports, clippy -D green (Critic 3).

`status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate`.

## Phase 4 — Validate (Opus, 2026-07-22, /goal) — GATE GREEN [diff]

**Tests added (3, context_menu.rs pure core — REQ-001 + the F3 coverage the removed `section_action_routing`
held):**
- `section_items_tables` — full-slice equality per RailSection (Editor 2 / Terminal 2 / Browser 3 / Panes 2
  rows, exact labels + wrapped `SectionAction`). Kills the sole viable `section_items` mutant
  (`Vec::leak(Vec::new())` = empty; the two `Default::default()` candidates are UNVIABLE — `MenuAction` has
  no `Default` derive, confirmed via `cargo mutants --list`).
- `items_for_routes_section_to_section_items` — `items_for(MenuKind::Section{p:0,section:s}) == section_items(s)`
  over `RailSection::ALL` (the items_for Section-arm).
- `section_item_0_preserves_the_387_default` — item 0 per section = the old `section_action` default
  (OpenFile / NewTerminal / OpenCockpit(Forge) / SplitFocused(Horizontal)) — the muscle-memory pin.

**Runs (actual):**
- `cargo nextest run -p marley` → **834 passed, 2 skipped** (831 + the 3 new).
- `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15. gate:4 coverage **100% lines** (context_menu.rs
  pure seam), gate:5 mutation **MSI 100%** (the section_items viable mutant killed; the widened
  `SectionAction`/`MenuAction` enums added no surviving mutant), gate:15 visual/AX green. Receipt written.
- **Skip-binding verified (the detach trap):** `cargo mutants --list -f app.rs | grep -E
  'close_transient_overlays|open_section_menu|dispatch_section_verb'` → EMPTY = all three are correctly
  `mutants::skip`-bound (absent from the list), no detachment.

**Driven capture — ENV-BLOCKED (documented explicitly, not skipped; verified by units + mechanism + critics).**
The ＋ opens a TRANSIENT context menu (`context_menu = Some(MenuKind::Section)`) that is (a) not persisted → the
#390 pre-seed technique cannot stage it; (b) MOUSE-only → no keystroke/action path a headless
`simulate_keystrokes` could trigger; (c) unreachable by synthetic mouse input this session (bare binary, no
Accessibility perm — established #385-392); and (d) **not pixel-capturable headlessly — `render_to_image` is
NOT yet in this gpui version** (`headless_drive.rs:7` marks it pending the #264 follow-up). So no path can
capture the open menu this session. **Why the change is nonetheless proven:** the diff added **ZERO new render
code** — the section menu flows through the byte-identical `menu.items()` render loop (app.rs:19335), keyboard
nav (15334-15345), esc/click-away, and `menu_origin` clamp already VISUALLY proven for the Split/Block/FileRef
kinds (#166/#175 captures); the ONLY new inputs to that proven loop are `section_items` (the 3 unit tests above,
cov/MSI 100) and the `items_for` Section arm (unit-tested). The app.rs wiring (`open_section_menu` /
`dispatch_section_verb` / the `run_context_menu_action` Section arm) is a masked shim in the ACCEPTED-UNTESTABLE
exclude, and every dispatch verb it calls (`new_terminal_pane`, `spawn_terminal_tab_in`, `start_file_finder`,
`open_or_switch_cockpit`, `split_focused_pane`) is already exercised by existing `headless_drive` tests.
Inspect's 3 critics traced every dispatch arm + the machinery reuse concretely. This is the #392-style
honest N/A-with-mechanism, forced by a real gpui limitation (#264), not a shortcut. **Follow-up:** when #264
lands `render_to_image`, a headless capture of the open Section menu (Terminal rows) is a 30-min add — noted,
no ticket.

**Pre-existing exclusions:** none new. app.rs menu shims stay in the coverage/mutation exclude (masked,
mechanism-verified); the pure context_menu.rs `section_items` seam carries cov/MSI 100 here.

`status: Phase 4 — Validate PASS; ready for Phase 5 — Complete`.
