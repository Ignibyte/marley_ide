# App ignores ⌘Q — register the standard app menu + Quit accelerator — Notes

- **Forge ticket:** #380 f4d905aa-88dd-4339-aa8d-ef55273fb761
- **AAR:** a2882e3b-b291-419e-a1a8-96ac5fa36b47
- **Local ticket doc:** docs/planning/tickets/open/TICKET-380-quit-accelerator.md
- **Pipeline spec:** 380-quit-accelerator.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** Forge #380 (bug, sprint #36 "M25 — App-Grade QA Hardening", milestone M25),
  filed from the 2026-07-21 live QA run: ⌘Q in the focused app does nothing (window stays,
  event loop sampled healthy — NOT a hang); `osascript -e 'tell application "Marley" to quit'`
  exits promptly and cleanly (sessions reaped, no orphans). Diagnosis: shutdown is correct,
  the TRIGGER is missing — no menu, no quit action, no cmd-q binding. Drafted 2026-07-21 in
  the sprint #36 /spec batch (QUEUED — Phase 1 only; promote to active for Phase 2).
- **Classification / tier:** bug, small surface — one registration site in the app.rs boot
  shim + one `actions!` declaration; the risk is all in getting the two-halves wiring (D2)
  and teardown parity (D3) right, both settled by reading gpui.
- **Forge recall (§18.3):** `knowledge-search` ("quit teardown shutdown menu accelerator
  cmd-q RootView drop clean exit") returned no high-signal hit (6 low-score
  prevention-rule/AD ids, none on-point). Standing traps that DO bind here, from memory:
  the selftest screencapture/focus rules (focus in the SAME drive call; quit stale Marley
  instances first; locked-screen fallback PR) and app.rs = masked shim (no mutants surface).
  Deep recall re-runs at /work claim.
- **Discovery (all read this session; gpui = Apache-2.0, adoption outside the §20 wall):**
  - **Marley boot has no menus**: `run()` = app.rs:18944-18992 —
    `Application::new().with_assets(Assets).run(…)` → geometry → `open_window` →
    `cx.activate(true)`. No `set_menus` anywhere in marley_app (crate-wide grep: only the
    unrelated CodeActionMenu/CompletionMenu structs). `bin/marley.rs` is a 7-line
    `marley_app::run()` shim.
  - **No quit chord exists**: Marley's pure chord table (keymap.rs — own `KeyBinding::chord`
    model, :20-53; built-ins doc :86-89 = cmd-shift-p / cmd-d / cmd-w / cmd-b / cmd-shift-b)
    has NO quit verb (grep). No cmd-h binding either (Hide-trio collision check: clean).
  - **gpui menu API** (~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2/,
    version pinned by Cargo.lock = 0.2.2, crates.io):
    `Menu { name: SharedString, items: Vec<MenuItem> }` (src/platform/app_menu.rs:5-11);
    `MenuItem::{Separator, Submenu(Menu), SystemMenu(OsMenu), Action{name, action:
    Box<dyn Action>, os_action: Option<OsAction>}}` (:52-74); `MenuItem::action(name, action)`
    (:96-102); `MenuItem::os_action` (:105-115) with `OsAction` = Cut/Copy/Paste/SelectAll/
    Undo/Redo ONLY (:210-228) — no Quit OsAction; `App::set_menus(Vec<Menu>)` →
    `platform.set_menus(menus, &self.keymap.borrow())` (src/app.rs:1840-1841);
    `init_app_menus` → menu pick → `cx.dispatch_action` (app_menu.rs:230-252).
  - **THE finding (why ⌘Q needs BOTH menu and binding)**: mac `create_menu_item`
    (src/platform/mac/platform.rs:303) computes the NSMenuItem key equivalent from
    `keymap.bindings_for_action(action)` — gpui's OWN keymap (:322-340), single-keystroke
    bindings become keyEquivalent + modifier mask (:355-392); if NO binding exists the item
    is built with an EMPTY equivalent (:402-409) → menu click works, ⌘Q dead. Marley's pure
    keymap.rs is invisible to this. Hence D2: `cx.bind_keys` (gpui `KeyBinding`, App::bind_keys
    src/app.rs:1677) + `cx.set_menus`, atomically.
  - **No built-in Quit action, no default menus**: gpui's only shipped action is `NoAction`
    (src/action.rs:423-440); the keymap.rs:224 `actions!` block is namespace `test_only`.
    A bare gpui app does nothing on ⌘Q — the observed bug is gpui-default behavior.
  - **The quit chain, both routes converging**: `App::quit()` → `platform.quit()`
    (src/app.rs:749-751); mac quit → async dispatch → `[NSApp terminate: nil]`
    (mac/platform.rs:503-523; the async hop is deliberate — avoids double-borrow during
    window-close callbacks, comment :504-509). AppleEvent quit reaches the same `terminate:`.
    Then: `will_terminate` delegate (:1421-1429) → the registered on_quit
    (src/app.rs:686-689) → `App::shutdown` (:698-720): quit observers (100ms
    SHUTDOWN_TIMEOUT) → `windows.clear()` (:705) → RootView drops. Teardown parity is
    structural, not incidental.
  - **Marley's teardown chain = RootView FIELD Drops (no `impl Drop for RootView` exists —
    verified crate grep)**: `mcp_host: Option<McpHost>` (app.rs:195; #370 started on demand,
    :2302) → `McpHost::drop` removes the #375 discovery file (mcp_host.rs:91-101, D7 comment);
    `fleet_subscription: Option<FleetSubscription>` (app.rs:395-398 — "Dropping it stops+joins
    the pump") → `FleetSubscription::drop` = `stop.store(true)` + `handle.join()`
    (marley_forge_client/src/adapter.rs:268-275, the #376 join). `App::shutdown`'s
    `windows.clear()` is the trigger on BOTH quit routes.
  - **Harness verify shape**: `scripts/selftest/drive.swift` has the `cmd:<key>` verb
    (:262-266, single base key — `cmd:q`); README.md usage = bundle-app.sh → `open
    target/Marley.app` → `focus` + verbs in ONE drive call (:37-59); headless-first note
    (:7-18) — but pixels/menu-bar/process-exit are live-lane concerns; process name for
    pgrep/System Events is `marley` (:41).
  - **Reference doc**: docs/warp_architecture/subsystems/07-app-entry-build-tooling.md —
    Warp boot applies "macOS menu bar (`app_builder.set_menu_bar_builder(app_menus::menu_bar)`),
    dock menu" (:194-196); the same doc contrasts "Marley's boot has **none** of that"
    (:36-38). The gap was known and deliberate at boot-slimming time; #380 is the bill.
- **Decisions:** D1 menu-based (standard; NSApp-level performKeyEquivalent pre-empts the
  window key path → immune to RootView on_key_down swallowing; HIG + Warp-observed; About/Hide
  slots free). D2 set_menus + bind_keys BOTH (the mac/platform.rs derivation — the trap that
  would make a menu-only fix look done while ⌘Q stays dead). D3 handler = `cx.quit()`, never
  `process::exit` (parity by construction). D4 own `actions!`-declared Quit (gpui ships none).
  D5 registration in the `run()` closure, masked shim. D-OPEN-MENU-EXTENT for P2: Quit-only
  vs + Hide/Hide Others/Show All (`App::hide` :984 / `hide_other_apps` :989 /
  `unhide_other_apps` :994 all exist; lean = include the trio, skip About).
- **P2 pointers:** probe gpui's TestPlatform quit behavior before claiming any headless pin
  for REQ-006 (do not over-claim — the live drive is the required proof either way); exact
  `actions!` namespace + registration ordering; the live verify script must quit stale Marley
  instances first and focus in the same drive call (standing selftest traps).
- **Plan verification (Opus /work, 2026-07-22) — D2 + D3 confirmed against the real gpui-0.2.2
  source, the design-invalidating claims attacked first:**
  - **D2 CONFIRMED** (`gpui-0.2.2/src/platform/mac/platform.rs:303-410`, read directly):
    `create_menu_item` computes the key equivalent from `keymap.bindings_for_action(action)`
    (:322-323) — gpui's OWN keymap; a single-keystroke binding sets the real keyEquivalent +
    modifier mask (:382-392), but the **no-binding `else` (:402-410) builds the item with
    `ns_string("")`** — an EMPTY equivalent → ⌘Q dead (a menu CLICK still dispatches via the
    `handleGPUIMenuItem:` selector, :351). And `set_menus` (`app.rs:1840-1841`) passes
    `&self.keymap.borrow()` — the same keymap `bind_keys` (`app.rs:1677`) populates. So
    set_menus + bind_keys are genuinely BOTH required; a menu-only fix looks done while ⌘Q
    stays dead. A no-context binding is selected because the predicate check is `is_none_or`
    (:325) — predicate `None` ⇒ true. The drafter's central claim holds exactly.
  - **D3 CONFIRMED** (`app.rs:749-751`): `App::quit()` = `self.platform.quit()` → mac
    `[NSApp terminate:]` — the identical route the AppleEvent quit reaches, so RootView-drop
    teardown parity is structural. (Inspect still pins "no `process::exit` in the diff".)
  - Verdict: no decisions revised; the plan is solid. All cited line refs verified within ±2.

## Phase 2 — Design

**Architecture / approach.** A gpui app-menu registration, entirely inside `run()`'s boot closure
(the masked, coverage-excluded gpui-wiring shim). No new module, no pure seam, no state. §20
confirmed: the reference is macOS/Apple-HIG convention (Warp-observed at behavior level); the
substrate is gpui 0.2.2 (Apache-2.0) — every fact below was read from the gpui/gpui-macros source
(adoption, outside the §20 wall); no Warp/Zed source read.

**The wiring (exact shape for Implement):**
1. **Import** — extend the `use gpui::{…}` block (app.rs:14) with `actions, KeyBinding, Menu, MenuItem`.
2. **Action declaration** — module level, immediately above `pub fn run()`:
   `gpui::actions!(marley, [Quit]);` → a unit struct `Quit` implementing `gpui::Action`.
   Verified compilable without new deps: gpui-macros `derive_action.rs:119` special-cases
   `is_unit_struct` — the `build`/`json_schema` bodies (:124/:130) take the unit branch, so `Quit`
   needs NO `serde::Deserialize`/`schemars::JsonSchema` (marley_app already has serde + serde_json).
3. **Registration** — the FIRST statements inside the `.run(move |cx: &mut App| { … })` closure,
   before the geometry/`open_window` block, in THIS ORDER (the ordering is load-bearing — see D6):
   ```rust
   cx.on_action(|_: &Quit, cx: &mut App| cx.quit());
   cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
   cx.set_menus(vec![Menu {
       name: "Marley".into(),
       items: vec![MenuItem::action("Quit Marley", Quit)],
   }]);
   ```
   `App::on_action` (gpui app.rs:1696, `Fn(&A, &mut App)`), `App::bind_keys` (:1677),
   `App::set_menus` (:1840), `App::quit` (:749 → `platform.quit()` → `[NSApp terminate:]`).
   `KeyBinding::new<A: Action>(&str, A, Option<&str>)` (keymap/binding.rs:33) — `None` context skips
   the internal `parse(...).unwrap()`, so there is NO panic path (a non-None bad-context string would
   unwrap-panic; we pass None).

**D-OPEN-MENU-EXTENT — RESOLVED: Quit-only (revises the plan's "lean include the Hide trio").**
Evidence dissolved the lean: `hide`/`hide_other_apps`/`unhide_other_apps` are gpui App **methods**
(app.rs:984/:989/:994), NOT gpui actions — a menu item needs `impl Action`, so each would require its
OWN `actions!` declaration + `on_action` handler + `bind_keys` (cmd-h / cmd-alt-h), TRIPLING the
surface and adding a cmd-h binding for a focused quit-accelerator bug. A one-item "Marley" app menu
with "Quit Marley" is standard macOS furniture and fully satisfies REQ-004. The Hide trio + an About
row are recorded as a trivial follow-up (each is one action+handler+binding+menu row). This is a
locked-decision-dying-to-evidence WIN, per §20 discipline.

**D6 — bind_keys BEFORE set_menus (NEW, load-bearing, discovered this phase).** `set_menus`
(mac/platform.rs:914) synchronously calls `create_menu_bar(…, keymap)` → `create_menu_item` reads
`keymap.bindings_for_action(Quit)` AT CALL TIME (:322-323) to compute the NSMenuItem key equivalent.
If `set_menus` runs before `bind_keys`, the keymap has no cmd-q→Quit binding yet, so the Quit item is
built with `ns_string("")` (:402-410) → the menu shows Quit with NO ⌘Q. So the three calls are
order-sensitive: `on_action` (any time before dispatch) → `bind_keys` → `set_menus`. Inspect pins this.

**File manifest (crates/marley_app/src/app.rs ONLY):**
| File | Change |
|---|---|
| `crates/marley_app/src/app.rs` | (a) `use gpui::{…}` +`actions, KeyBinding, Menu, MenuItem`; (b) `gpui::actions!(marley, [Quit]);` above `run()`; (c) 3 registration stmts at the top of the `run()` closure (on_action→bind_keys→set_menus), before `open_window`. |

NO `keymap.rs` (Marley's pure chord table is a different, window-level system — untouched), NO
RootView, NO new files, NO pure seam.

**Regression Test Plan.** This is a shim-only change: the `actions!` unit-struct declaration has no fn
body (→ no cargo-mutants target), and both it and the `run()` registration live in app.rs, which is a
WHOLE-FILE coverage exclude (gates.sh:222 `--ignore-filename-regex …marley_app/src/app\.rs…`) and a
`mutants::skip` shim (run() is `#[cfg_attr(test, mutants::skip)]`, app.rs:18943). So there are **no new
unit tests** — the floors are structurally unaffected (nothing added to the 100% denominator). Proof is
LIVE + STRUCTURAL:

| AC | How proven | Lane |
|---|---|---|
| REQ-001 ⌘Q exits | drive `cmd:q` → `pgrep -x marley` empty within bounded wait | LIVE |
| REQ-002 teardown parity | after REQ-001: discovery file absent under config dir; no orphan marley pty/zsh procs; STRUCTURAL: handler is `cx.quit()`, no `process::exit` in diff | LIVE + STRUCTURAL |
| REQ-003 AppleEvent quit still clean | `osascript … to quit` → clean exit + same parity sweep | LIVE |
| REQ-004 menu shows Quit ⌘Q | System Events menu-bar read of process "marley"; a menu-CLICK on Quit also exits | LIVE |
| REQ-005 in-app chords unchanged | `cargo nextest run -p marley` headless suite green (unchanged); STRUCTURAL: `keymap.rs` no diff; spot-drive `cmd:d` spawns a terminal | headless + STRUCTURAL + LIVE |
| REQ-006 graceful gpui shutdown, never process::exit | STRUCTURAL (handler = `cx.quit()`; no `process::exit`/`abort` in diff). **Headless pin UNAVAILABLE and NOT claimed** — gpui `TestPlatform::quit` is a no-op (`platform/test/platform.rs:251 fn quit(&self) {}`), so there is no observable quit state to assert in `VisualTestContext`; the live drive is the behavioral witness. | STRUCTURAL + LIVE |

**The exact LIVE verify script (P4)** — honoring the harness gotchas (quit stale first; the ⌘Q drive
needs only app-FRONTMOST, not pane focus, so it SIDESTEPS the #383 `focus`-verb misfire entirely —
activate via `osascript frontmost` in the SAME command as the drive verb; `open`-launch):
```bash
# 0. kill any stale instance (a stale proc shadows the fresh binary + confuses pgrep)
osascript -e 'tell application "Marley" to quit' 2>/dev/null; sleep 1; pkill -x marley 2>/dev/null; sleep 1
# 1. bundle + launch the fresh binary
scripts/selftest/bundle-app.sh && open target/Marley.app && sleep 4
pgrep -x marley >/dev/null && echo "launched" || echo "FAIL launch"
# 2. drive ⌘Q — frontmost + keystroke in ONE shell command (keys vanish otherwise)
osascript -e 'tell application "System Events" to set frontmost of process "marley" to true' \
  && swift scripts/selftest/drive.swift cmd:q
# 3. assert exit within a bounded wait
for i in $(seq 1 12); do pgrep -x marley >/dev/null || break; sleep 0.5; done
pgrep -x marley >/dev/null && echo "REQ-001 FAIL: still running" || echo "REQ-001 PASS: exited"
# 4. teardown parity (REQ-002): discovery file gone + no orphans
ls "$HOME/Library/Application Support/marley"/mcp-endpoint* 2>/dev/null && echo "REQ-002 LEAK" || echo "REQ-002 clean"
pgrep -f 'marley' | grep -v grep || echo "no orphans"
# 5. REQ-003 AppleEvent smoke: re-launch, osascript quit, same sweep
open target/Marley.app && sleep 4 && osascript -e 'tell application "Marley" to quit' && sleep 2
pgrep -x marley >/dev/null && echo "REQ-003 FAIL" || echo "REQ-003 PASS"
# 6. REQ-004: read the app menu (Quit + ⌘Q) — relaunch, System Events menu read + click-Quit
```
If a synthetic CGEvent ⌘Q does not reach `performKeyEquivalent` (a known CGEvent subtlety), the
fallback proof for REQ-001/004 is a System-Events **menu CLICK** on Quit (proves the SAME action
dispatches) — both are live witnesses of the wired Quit; the machine-locked fallback PR applies only if
driving is fully blocked (prefer rescheduling — the ⌘Q proof is the ticket).

**Risks / load-bearing decisions:**
- **D6 ordering** (bind_keys→set_menus) — the one place a plausible reordering silently half-fixes the
  bug (menu with no ⌘Q). Inspect verifies both the presence of both calls AND their order.
- **D3 no `process::exit`** — the handler MUST be `cx.quit()`; any `process::exit`/`abort` would skip
  `App::shutdown` → `windows.clear()` → RootView field-Drops, leaking the #375 discovery file +
  orphaning the #376 pump. Structural inspect + the REQ-002 sweep are the paired witnesses.
- **No chord collision** — cmd-q is not in Marley's `keymap.rs` (verified); the gpui binding lives in
  gpui's separate keymap, and the NSApp menu key-equivalent pre-empts the window key path, so it cannot
  fight per-pane handling.

## Phase 2 — Design (status)
Design PASS — all decisions settled, both implementation risks (Action-derive deps, coverage floor)
retired against source; ready for Implement.

## Phase 3 — Implement
Built to the manifest — `crates/marley_app/src/app.rs` only, +22 lines (`git diff --stat`):
- `use gpui::{…}` (app.rs:14): added `Menu, MenuItem`.
- `gpui::actions!(marley, [Quit]);` at module level above `run()`.
- Inside the `run()` closure, first statements before the geometry block, in order:
  `cx.on_action(|_: &Quit, cx: &mut App| cx.quit())` → `cx.bind_keys([gpui::KeyBinding::new("cmd-q", Quit, None)])` → `cx.set_menus(vec![Menu { name: "Marley".into(), items: vec![MenuItem::action("Quit Marley", Quit)] }])`.

**Deviation from design (1, minor):** `KeyBinding` is NOT imported from gpui — marley_app already has
`use crate::keymap::{KeyBinding, Keymap}` (app.rs:84), its own window-level chord type used as
`KeyBinding::chord(...)` at ~17 call sites. Importing `gpui::KeyBinding` collided (E0252) and shadowed
the chord type (E0560/E0599 at every `::chord`). Fix: keep Marley's `KeyBinding` unshadowed, name the
gpui one fully-qualified at the single call site — `gpui::KeyBinding::new(...)`. `Menu`/`MenuItem` do
NOT collide (no existing defs) so they stay in the import. Net: the manifest's "add KeyBinding to the
use block" became "fully-qualify gpui::KeyBinding at the call site" — same wiring, no name clash.

`cargo check -p marley` clean (only a pre-existing `block v0.1.6` future-incompat dep note, unrelated);
`cargo fmt --check` CLEAN. `gpui::actions!(marley, [Quit])` compiled — the `marley` namespace is a bare
label (needs no real module), and the unit-struct Action derive needed no serde/schemars, both as the
design predicted. `Quit` is constructed at all three sites → no dead-code warning. No `process::exit`
introduced (D3 held). keymap.rs / RootView untouched.

## Phase 3 — Implement (status)
Implement PASS — compiles, fmt-clean, one minor name-collision deviation resolved; ready for Inspect.

## Phase 3.5 — Inspect
Two independent critics (general-purpose) over the diff, each tracing the ACTUAL gpui-0.2.2 source.
**Verdict: PASS — 0 HIGH, 0 MED; 2 LOW, both no-fix.** Lenses: (1) gpui-mechanism correctness,
(2) teardown-parity + provenance + collision safety.

**Confirmed correct (both critics, against source — matches my plan/design verification):**
- D6 ordering holds: `set_menus` reads `&self.keymap.borrow()` at call time (app.rs:1841) and
  `bind_keys` mutates it synchronously (app.rs:1678) → bind_keys-before-set_menus is satisfied;
  `on_action` position is irrelevant (separate `global_action_listeners` registry).
- Registration before `open_window`/`activate` is fine — the mac app menu is app-level
  (`sharedApplication.setMainMenu_`, platform.rs:914-922), valid with zero windows; the ⌘Q
  equivalent is baked onto the NSMenuItem at set_menus time.
- The action DISPATCHES: menu `handleGPUIMenuItem:` → `dispatch_action` → global-action bubble
  phase (window.rs:4066-4088) reaches our global listener → `cx.quit()`, exactly once (Bubble-gated).
- **Load-bearing bonus (critic 1):** `on_action` is REQUIRED for menu-item enablement —
  `is_action_available(Quit)` checks `global_action_listeners.contains_key` (app.rs:1834-1836);
  without the on_action call ⌘Q/the Quit item would grey out and no-op. My diff includes it.
- `KeyBinding::new("cmd-q", Quit, None)`: None-context skips the `parse().unwrap()` (binding.rs:34)
  — no panic; the no-context binding is SELECTED (`is_none_or` short-circuits true, platform.rs:325).
- D3/teardown: `cx.quit()` → `[NSApp terminate:]` (platform.rs:520) → `will_terminate` → `on_quit`
  → `App::shutdown` → `windows.clear()` (app.rs:705) → Window/RootView field-Drops. The #375
  discovery-file removal (mcp_host.rs:96-100) + #376 stop+join (adapter.rs:270-275) are UNTOUCHED
  by the diff and run synchronously during terminate. No `process::exit`/`abort` anywhere.
- REQ-005 structural witness (critic 2): the keybinding→action fires before `on_key_down`
  (window.rs:3834 vs :3848); the global listener does NOT stop propagation, so ⌘Q still reaches
  Marley's `on_key_down`, but it no-ops — unbound in keymap.rs AND `key_input_from_keystroke`
  returns None for any cmd chord (app.rs:3053-3054, "must NEVER stream to the program") → no PTY
  byte leak. Net effect of ⌘Q = exactly one `cx.quit()`. No chord collision, no double-dispatch.
- Provenance §20 CLEAN (gpui Apache-2.0 only, canonical idiom, nothing from Warp/Zed); no secrets,
  no `unsafe`, no untrusted input; `cargo check -p marley` warning-clean (only the pre-existing
  `block v0.1.6` dep note).

**LOW findings — reviewed, both REJECTED as no-fix (with reason):**
- **[LOW] duplicate-action-name panic possible** (app.rs:18944) — `actions!` registers `marley::Quit`
  globally; a SECOND same-name declaration would panic at `App::new` (action.rs:53 doc). REJECTED as
  a fix: only ONE `actions!`/gpui-action exists crate-wide (verified) and this is a don't-do-that
  caution, not a defect in the diff. Noted for future menu work.
- **[LOW/cosmetic] app-menu convention** (app.rs, the Menu) — a lone "Quit Marley" item under a
  "Marley"-titled app menu is functional but sparse; when launched NOT as a `.app` bundle the
  displayed app-menu title can differ from "Marley". REJECTED as a fix: cosmetic only, the ⌘Q
  mechanism is unaffected, and the Hide-trio/About expansion is already the recorded D-OPEN follow-up.
  **Validate awareness:** the live drive launches the `.app` bundle (`open target/Marley.app`), so the
  menu title reads "Marley" correctly in the test — the non-bundle caveat does not affect the proof.

**Fixes applied: none** (no confirmed defect). No `failure-record`/`prevention-rule-record` — the one
name-collision gotcha (gpui vs marley `KeyBinding`) was caught+resolved in Implement and is documented
there; too situational for a formal rule.

## Phase 3.5 — Inspect (status)
Inspect PASS — mechanism + teardown verified correct by two independent source-tracing critics;
no fixes; ready for Validate.

## Phase 4 — Validate
**Tests added: none** — shim-only change (app.rs is a whole-file coverage exclude + `run()` is
`mutants::skip`; the `actions!` unit struct has no fn body). Nothing enters the 100% denominator, so
the floors are structurally unaffected. Verified LIVE + STRUCTURAL per the design's test plan.

**REQ-005 (nothing broke):** `cargo nextest run -p marley` → **798 passed, 2 skipped, exit 0** (incl.
`headless_drive::*` and `fleet_livewire::*`). keymap.rs untouched (git diff confirms).

**LIVE validation (self-test harness, release bundle `target/Marley.app`):** the machine was in active
use — screen UNLOCKED (console user chadpeppers), **MSTeams frontmost** — so a raw GLOBAL synthetic ⌘Q
keystroke was DEFERRED for safety (a stray global ⌘Q could quit chad's frontmost app / a live call).
Instead the wiring was proven with TARGETED, non-destructive System-Events operations that dispatch to
Marley specifically:
- **REQ-004 ✓ (empirical D2/D6 proof):** the live menu bar reads `Apple, Marley`; the "Marley" app menu
  has exactly one item **"Quit Marley"** whose `AXMenuItemCmdChar = "Q"` and `AXMenuItemCmdModifiers = 0`
  (⌘ only). The ⌘Q equivalent is present on the item ONLY because `bind_keys` ran before `set_menus`
  (the key equivalent is derived from gpui's keymap at set_menus time) — so D2 (both calls required) and
  D6 (bind_keys-before-set_menus) are confirmed in the RUNNING app, not just in source.
- **REQ-001/006 ✓:** a System-Events **menu click** on "Quit Marley" → Marley exited (`pgrep -x marley`
  empty within the bounded wait). The Quit action dispatched → `cx.quit()` → graceful gpui shutdown.
- **REQ-002 ✓:** teardown parity — after quit: no `marley` binary, no `Marley.app/Contents/MacOS/marley`
  process, no launchd-reparented (ppid 1) orphan pty shells, and no stray discovery file (the config dir
  never existed — a bare launch with no forge config never starts the mcp host, so #375's file was never
  created; the Drop path is a no-op here and the exit is clean).
- **REQ-003 ✓:** relaunch → `osascript -e 'tell application "Marley" to quit'` → clean exit (the
  2026-07-21 QA command, re-run green).
- **DEFERRED (documented, not masked): the raw global ⌘Q keystroke live-fire.** Rationale: machine in
  active use (see above). Coverage without it: (a) the ⌘Q equivalent is EMPIRICALLY on the menu item
  (REQ-004), so macOS `performKeyEquivalent` will match ⌘Q; (b) both inspect critics source-traced that a
  ⌘Q keystroke routes through the IDENTICAL `handleGPUIMenuItem:` → `dispatch_action` → global-listener →
  `cx.quit()` path that the menu-click exercised live. So the keystroke half is proven by
  equivalent-on-item + shared-handler; only the final synthetic-keypress is unrun. Reschedulable on an
  idle machine — no ticket needed (the mechanism is proven end-to-end); per
  `PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism` (user-active variant).

**Full gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** (exit 0) — 15/15 pass: rustfmt,
clippy (-D warnings), tests (nextest + doctests), cargo-audit, cargo-deny, cargo-machete, gitleaks,
shellcheck, no-suppressions, source-bans, docs, **coverage ≥100%**, **mutation MSI ≥100%**, miri,
visual/AX. The shim-only change added nothing to the coverage/mutation denominators (app.rs excluded,
`run()` skip, unit-struct no body), so both floors held. Receipt written for `/commit`.

## Phase 4 — Validate (status)
Validate PASS — live wiring proven (⌘Q equivalent on the menu item + menu-click quit + AppleEvent
quit + clean teardown), suite green (798), gate GREEN [diff]. The global-keystroke live-fire is the
only deferred item (safety; mechanism proven). Ready for Complete.

## Phase 5 — Complete
- **CHANGELOG:** entry under `### Added` (⌘Q quit accelerator — the gpui app menu + Quit action,
  D2/D6 ordering, teardown parity via `[NSApp terminate:]`).
- **Architecture docs:** `docs/warp_architecture/subsystems/07-app-entry-build-tooling.md` — the
  "Marley's boot has **none** of that" contrast line changed to "**almost none**", naming #380 as the
  single platform-menu exception; `docs/marley_architecture/app_shell.md` — a Purpose note that `run()`
  now registers the app menu + Quit + cmd-q and quits through the RootView-drop teardown.
- **Forge knowledge:** `AD-claude-gpui-menu-accelerator-001` (d9e93dac — the set_menus+bind_keys-ordered
  fact + cx.quit()/terminate: parity); `PR-claude-destructive-keystroke-validate-via-targeted-menu-001`
  (eaf8c139 — targeted menu read/click instead of a global synthetic keystroke when the machine is in
  active use); `aar-submit` a2882e3b (completed, effectiveness 5, 2 novel findings, 13 verdicts). NO
  `failure-record` — inspect found no real bug (the KeyBinding name-collision was an implement-time
  compile fix, documented in Phase 3, too situational for a rule). `ticket-comment` + `ticket-close`
  #380 → done.
- **Ticket doc** → `docs/planning/tickets/closed/`, status closed. **Pipeline** → `completed/`.

Complete PASS.
