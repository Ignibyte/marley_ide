---
pipeline_id: 653d4d33-fe1f-4c4e-8984-af6250c17987
ticket: forge#380 (f4d905aa-88dd-4339-aa8d-ef55273fb761) · local docs/planning/tickets/open/TICKET-380-quit-accelerator.md
aar_id: a2882e3b-b291-419e-a1a8-96ac5fa36b47
status: Phase 5 — Complete PASS
title: App ignores ⌘Q — register the standard app menu + Quit accelerator
type: bug
milestone: M25
references:
  - crates/marley_app/src/app.rs
  - crates/marley_app/src/mcp_host.rs
  - crates/marley_forge_client/src/adapter.rs
  - scripts/selftest/README.md
  - docs/warp_architecture/subsystems/07-app-entry-build-tooling.md
---

## Title
⌘Q does nothing in the focused app (2026-07-21 live QA). The process is healthy — the AppleEvent
quit (`osascript … to quit`) exits promptly and cleanly, sessions reaped, no orphans — so
shutdown is CORRECT and only the trigger is missing: `marley_app::run` (app.rs:18944-18992)
boots `Application::new().with_assets(Assets).run(…)` → `open_window` → `cx.activate(true)` and
never calls `cx.set_menus`, registers no quit action, and binds no cmd-q anywhere (Marley's own
pure keymap.rs has no quit verb — verified). With no menu there is no ⌘Q key equivalent, and
nothing else consumes the chord. Fix: define a Marley `Quit` gpui action, handle it globally
with `cx.quit()`, bind `cmd-q` in gpui's keymap, and register the standard app menu via
`cx.set_menus` — the menu item's ⌘Q equivalent is DERIVED from that gpui keymap binding
(gpui-0.2.2 mac/platform.rs:322-340), so menu + binding are BOTH required. `cx.quit()` converges
with the AppleEvent path at `[NSApp terminate:]`, so teardown parity (RootView drop → #375
discovery-file removal, #376 subscription join) holds by construction — then gets verified.

## Scope
### In
- **A `Quit` action** — gpui ships NO built-in quit action (its only action is `NoAction`,
  gpui-0.2.2 action.rs:427-434), so Marley declares its own via the `actions!` macro, with a
  global handler `cx.on_action(|_: &Quit, cx| cx.quit())` (`App::on_action` exists,
  gpui app.rs:1696; `App::quit` :749-751).
- **The gpui keymap binding** — `cx.bind_keys` (gpui app.rs:1677) with a gpui `KeyBinding` for
  `cmd-q` → `Quit`, no context. This is what paints ⌘Q onto the menu item (see D2) — Marley's
  own pure `keymap.rs` chord table is NOT touched (it's a different, window-level system).
- **The standard app menu** — `cx.set_menus(vec![Menu { name: "Marley", items: … }])`
  (gpui app.rs:1840-1841) with `MenuItem::action("Quit Marley", Quit)` (app_menu.rs:96-102);
  optionally the Hide/Hide Others/Show All one-liners (real gpui App methods exist:
  `hide` :984 / `hide_other_apps` :989 / `unhide_other_apps` :994; no cmd-h collision in
  Marley's keymap — verified) — P2 decides whether they ride along (D-OPEN-MENU-EXTENT).
- **Registration site** — inside `run()`'s boot closure (app.rs:18947-18989), app-level, one
  site, shim territory (masked, like the rest of app.rs's gpui wiring).
- **Verification both lanes** — live selftest drive `cmd:q` → process exits + teardown-parity
  sweep; AppleEvent quit smoke stays green; existing headless suite unchanged.

### Out (explicitly deferred)
- Full menu bar buildout (File/Edit/View/Window menus, OsAction cut/copy/paste rows) — this
  ticket ships the app menu with Quit only (+ optional Hide trio).
- About box content — a stub "About Marley" row is allowed only if free; any real About window
  is deferred.
- Window menu, dock menu, Services menu (`MenuItem::os_submenu`/`SystemMenuType::Services`
  exists in gpui but is not wired here).
- Confirm-before-quit / unsaved-editor-buffer prompting — quit stays immediate, matching the
  AppleEvent path's current semantics; a guard dialog is its own future ticket.
- Any change to Marley's pure `keymap.rs` chord table or the RootView `on_key_down` path.

## Reference (§20)
**macOS platform convention (Apple HIG), observed in Warp**: every mac app quits on ⌘Q via the
standard application menu's Quit item — Warp (the host terminal this session runs in) does
exactly that, behavior-observed. Marley's own clean-room Warp map records the analog at
architecture level: Warp's boot applies "platform menu wiring … macOS menu bar
(`app_builder.set_menu_bar_builder(app_menus::menu_bar)`), dock menu"
(docs/warp_architecture/subsystems/07-app-entry-build-tooling.md:194-196) and the same doc
already flags that "Marley's boot has **none** of that" (:36-38). Clean-room §20 intact: the
map is research-derived behavior documentation; no Warp (AGPL) / Zed (GPL) source read. The
implementation substrate is gpui (Apache-2.0) — reading its source is adoption, outside the
wall.

### Prior art
1. **Behavior maps** — docs/warp_architecture/subsystems/07-app-entry-build-tooling.md:194-196
   (Warp registers a macOS menu bar at boot; Marley's boot deliberately shipped without it —
   :36-38). That gap is exactly this bug.
2. **Published** — Apple HIG, "The menu bar": the application menu is required standard
   furniture (About / Hide / Quit with ⌘Q). macOS does NOT synthesize quit-on-⌘Q for a
   menu-less app; the key equivalent must be claimed by a menu item (or handled as an ordinary
   keystroke by the app — which nothing does today).
3. **OUR PERMISSIVE DEPS — gpui 0.2.2 (Apache-2.0), the leg that settles the design**
   (~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2/):
   - **Menu API**: `Menu { name, items }` (src/platform/app_menu.rs:5-11); `MenuItem` enum —
     `Separator` / `Submenu(Menu)` / `SystemMenu(OsMenu)` / `Action { name, action, os_action }`
     (:52-74); constructors `MenuItem::action(name, action)` (:96-102) and
     `MenuItem::os_action(…)` (:105-115). `OsAction` covers ONLY Cut/Copy/Paste/SelectAll/
     Undo/Redo (:210-228) — there is NO Quit os_action; Quit is an ordinary action item.
     `App::set_menus(&self, menus: Vec<Menu>)` → `platform.set_menus(menus, &keymap)`
     (src/app.rs:1840-1841). `init_app_menus` routes a menu pick through
     `cx.dispatch_action(action)` (app_menu.rs:246-251).
   - **The load-bearing trap**: the mac platform derives each menu item's key equivalent from
     gpui's OWN keymap — `keymap.bindings_for_action(action)` → keystroke → NSMenuItem
     key-equivalent + modifier mask (src/platform/mac/platform.rs:303, :322-340, :382-392);
     with NO binding the item is created with an EMPTY key equivalent (:402-409) and ⌘Q stays
     dead (menu click would still work). So `set_menus` alone is insufficient — the fix must
     also `bind_keys` (src/app.rs:1677).
   - **The quit chain**: `App::quit()` → `platform.quit()` (src/app.rs:749-751); mac
     `quit()` dispatches async → `[NSApp terminate: nil]` (mac/platform.rs:503-523) — the SAME
     `terminate:` the AppleEvent quit reaches → `will_terminate` delegate (:1421-1429) → the
     `on_quit` callback → `App::shutdown` (registered app.rs:686-689; shutdown :698-720 runs
     quit observers with a 100ms timeout then `self.windows.clear()` :705 — which drops the
     window → drops RootView). Both quit routes CONVERGE before any teardown runs.
   - **No default**: gpui ships no quit action (only `NoAction`, action.rs:427-434; the
     keymap.rs:224 `actions!` block is `test_only`) and no default menus — a bare gpui app
     ignores ⌘Q, which is precisely the observed bug.
   - **Marley's teardown chain (ours, verified)**: RootView has NO `impl Drop` of its own — the
     chain is its FIELDS' Drops: `mcp_host: Option<McpHost>` (app.rs:195) whose Drop removes
     the #375 discovery file (mcp_host.rs:91-101, `remove_discovery_file_in`);
     `fleet_subscription: Option<FleetSubscription>` (app.rs:395-398) whose Drop stops+joins
     the #376 pump (marley_forge_client/src/adapter.rs:268-275: `stop.store(true)` +
     `handle.join()`). `windows.clear()` in `App::shutdown` is what triggers it.

## Locked-In Decisions
- **D1 — Menu-based quit (the standard app menu), not a Marley-keymap chord.** The menu's key
  equivalent is claimed at the NSApp `performKeyEquivalent` level, BEFORE the window's key
  path — so it works regardless of what RootView's `on_key_down` swallows, works with any
  focus state, matches the HIG + the Warp-observed behavior, and buys the About/Hide slots.
  Marley's pure `keymap.rs` stays a window-level chord table (no quit verb added there).
  **Rejected:** routing cmd-q through Marley's own `on_key_down` chord dispatch (fights the
  shell's per-pane key handling, nonstandard, no menu bar shown).
- **D2 — BOTH `set_menus` AND `bind_keys`, atomically.** Evidence gpui
  mac/platform.rs:322-340 vs :402-409 (key equivalent derived from `bindings_for_action`;
  empty without a binding). A menu without the binding renders Quit with no ⌘Q; a binding
  without the menu leaves the chord to the window key path where nothing dispatches gpui
  actions today. One without the other is a silent partial fix.
- **D3 — The handler is `cx.quit()`; NEVER `std::process::exit`.** `cx.quit()` funnels into
  `[NSApp terminate:]` — the identical route the AppleEvent quit takes — so RootView drops and
  the #375/#376 field-Drop teardown runs on both paths by construction. `process::exit` would
  skip `App::shutdown` → leak the discovery file + orphan the pump thread. Structural inspect
  pins this.
- **D4 — Marley declares its own `Quit` action (`actions!` macro).** gpui ships none
  (action.rs:427-434 `NoAction` only). Namespace/name per P2 (e.g. `marley::Quit`); the action
  is app-level, dispatched globally via `App::on_action` (app.rs:1696).
- **D5 — Registration lands in `run()`'s boot closure (app.rs:18947-18989), shim-masked.**
  One site, before/around `open_window` (exact ordering P2) — the same
  accepted-untestable gpui wiring surface as the rest of `run()`; no pure seam is created
  unless P2 finds one worth extracting (none expected — this is pure registration).
- **D-OPEN-MENU-EXTENT — RESOLVED (P2): Quit-only.** The lean ("include the Hide trio") was
  dissolved by evidence: `hide`/`hide_other_apps`/`unhide_other_apps` are gpui App **methods**
  (:984/:989/:994), NOT actions, so each menu row needs its own `actions!` + `on_action` +
  `bind_keys` — tripled surface + a cmd-h binding for a focused quit fix. A one-item app menu
  satisfies REQ-004; the Hide trio + About are a trivial follow-up. (Notes: Phase 2 design.)
- **D6 — bind_keys BEFORE set_menus (P2, load-bearing).** `set_menus` reads the keymap
  synchronously to derive the ⌘Q equivalent (mac/platform.rs:914 → :322-323), so the binding
  must exist first; order is `on_action → bind_keys → set_menus`. Inspect pins it.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user presses ⌘Q with Marley focused, the app shall exit — the process terminates cleanly (no crash, no force-kill), the window closes. | LIVE selftest drive: bundle → `open target/Marley.app` → `swift scripts/selftest/drive.swift focus cmd:q` (the `cmd:<key>` verb, drive.swift:262-266) → poll `pgrep -x marley` empty within a bounded wait; no crash report |
| REQ-002 | WHEN quit is triggered via ⌘Q, the teardown shall take the SAME path as the AppleEvent quit — RootView dropped, the #375 discovery file removed (`McpHost::drop`, mcp_host.rs:91-101), the #376 fleet pump stopped+joined (`FleetSubscription::drop`, adapter.rs:268-275), shell sessions reaped, no orphan processes. | LIVE parity sweep after the REQ-001 drive: discovery file absent under the config dir; no orphaned marley-spawned zsh/pty procs (`pgrep` sweep); STRUCTURAL inspect: the handler calls `cx.quit()` and no `process::exit` appears in the diff |
| REQ-003 | WHEN a quit AppleEvent arrives (`osascript -e 'tell application "Marley" to quit'`), the app shall still exit promptly and cleanly, exactly as before the change. | LIVE smoke: launch → osascript quit → prompt clean exit (the 2026-07-21 QA command, re-run); same parity sweep |
| REQ-004 | WHILE the app runs, the menu bar shall show the application menu containing a Quit item displaying the ⌘Q key equivalent. | LIVE: System Events menu-bar read (`osascript` over process "marley" menu bar) and/or a screenshot of the opened app menu; a menu-CLICK on Quit also exits (proves the item dispatches, not just the chord) |
| REQ-005 | WHILE the app runs, existing in-app chords shall behave unchanged — ⌘W close-pane, ⌘D new-terminal, ⌘⇧P palette (Marley's pure keymap table untouched). | headless lane: `cargo nextest run -p marley headless_drive` green, zero expectation changes; LIVE spot-check `cmd:d` spawns a terminal before quitting; STRUCTURAL: `keymap.rs` has no diff |
| REQ-006 | WHEN the Quit action dispatches (menu click or ⌘Q), the app shall shut down gracefully through gpui — quit observers run, windows clear, RootView drops — never via `process::exit`/`abort`. | STRUCTURAL inspect on the diff (D3); the REQ-002 parity sweep is the behavioral witness; P2 additionally probes whether gpui's TestPlatform can pin the action→`quit()` dispatch headlessly (do not over-claim until read) |

## Floors (constitution)
This is a shim-only change: the registration lives in app.rs's masked, coverage-excluded gpui
boot closure (`mutants::skip` surface — lib.rs:15-18's accepted-untestable contract). No pure
seam is expected; if P2 extracts one (e.g. a menu-spec builder), it ships cov/MSI 100. No
`unwrap` on input paths; the existing headless suite and every live smoke stay green.

## Phase Plan
- **P2 Design** — settle D-OPEN-MENU-EXTENT (Quit-only vs + Hide trio); exact action
  namespace/name + `actions!` placement; exact registration ordering inside `run()`'s closure
  (menus/bindings before `open_window`); probe gpui TestPlatform for a headless
  action-dispatch pin (read test platform source first); write the exact live verify script
  (bundle → drive `focus cmd:q` → pgrep + discovery-file + orphan sweep; then the AppleEvent
  smoke) honoring the harness gotchas (README.md: `open`-launch, focus in the SAME drive call,
  quit stale instances first).
- **P3 Implement** — app.rs `run()` closure only (+ the `actions!` declaration); no keymap.rs,
  no RootView changes.
- **P3.5 Inspect** — adversarial: any `process::exit`? both halves of D2 present? does the
  menu claim any chord Marley already uses (must not)? teardown parity reasoning sound
  (cx.quit → terminate: convergence)? provenance (§20 — gpui-only reading).
- **P4 Validate** — run BOTH live lanes (⌘Q drive; AppleEvent smoke) + the parity sweeps +
  `headless_drive` suite; gate green (`--diff`); machine-locked fallback per
  `PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism` ONLY if the
  environment blocks driving — the ⌘Q live proof is the point of the ticket, so prefer
  rescheduling the drive over masking it.
- **P5 Complete** — CHANGELOG; ticket + AAR close; note the app-menu baseline in the app-shell
  doc (Marley now HAS platform menu wiring — update the 07-app-entry map's "none of that"
  contrast line if the map is touched this cycle, else file a docs follow-up).
