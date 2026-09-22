# Agent-launcher popup should dismiss on click-outside — Notes

- **Forge ticket:** #229 (7062e0a2-6926-415c-8fd2-4ca03d6bdc5c)
- **AAR:** efe9cef1-9c72-480c-b7fd-5bbb98b33995
- **Local ticket doc:** docs/planning/tickets/open/TICKET-229-agent-launcher-dismiss.md
- **Pipeline spec:** agent-launcher-dismiss.spec.md

## Phase 1 — Plan
- **Request:** add click-outside dismiss to the #181 agent-launcher popup (chad live feedback #5). 2nd ticket
  of the /goal /work 228-237 auto-approved M13 train.
- **Classification / tier:** work pipeline · bug · M13 · ONE tiny shim slice (app.rs render only).
- **Forge recall (§18.3):** the #166 context-menu backdrop is the canonical dismiss pattern (memory: "modal
  backdrops MUST occlude in gpui, copy the nearest overlay"); `PR-claude-new-chord-shadowed-by-hardcoded-key`
  (unrelated). Clean-room §20.
- **Discovery (this turn):**
  - The launcher renders at app.rs ~:5394-5431: `if let Some(launcher) = &self.agent_launcher { let mut overlay
    = div().absolute().left(bounds.w*0.25).top(bounds.h/6.0).w(bounds.w*0.5).occlude()...; for row … ; root =
    root.child(overlay); }`. The box `.occlude()`s (:5400) but there is NO full-screen backdrop → an outside
    click does nothing. Closes only via Escape (:1500 `self.agent_launcher = None`) / Enter (:1502 `.take()`).
  - Rows (:5411-5417) are `div().child(launch_command)` with NO click handlers — selection is keyboard (↑↓/Enter
    at :1508-1524). So a backdrop doesn't break row selection.
  - The #166 context-menu backdrop (:6268-6287) is the pattern to mirror: `div().absolute().inset_0().occlude()
    .on_mouse_down(Left, |view,_,_,cx| { view.context_menu = None; cx.notify(); })` drawn BEFORE the menu box.
- **Decisions:** D1 mirror the #166 backdrop (full-screen occlude + on_mouse_down → agent_launcher=None + notify,
  before the box); D2 no pure seam (unconditional shim dismiss); D3 backdrop scoped inside the launcher block.
- **Env note:** the machine is UNLOCKED but chad is ACTIVELY on a SHARED desktop (live agents) → a driven
  capture (launch Marley + ⌘⇧A + outside-click) would hijack his screen. Validate: non-intrusive winid capture
  if a Marley is up, else env-considerate → mechanism (byte-identical to the proven #166 dismiss) + offer the
  20s driven check when his screen's free. gate-15 headless.

## Phase 2 — Design

### Architecture / approach
A masked render-shim addition in the app.rs render (the gpui `RootView::render`). Add a full-screen
occluding backdrop as the FIRST child of the `if let Some(launcher) = &self.agent_launcher {` block
(app.rs ~:5394), mirroring the proven #166 context-menu backdrop byte-for-byte:
```rust
        if let Some(launcher) = &self.agent_launcher {
            // #229: full-screen occluding backdrop → a click OUTSIDE the box dismisses the launcher (the
            // codebase's modal convention — mirror the #166 context menu). The box paints AFTER + `.occlude()`s,
            // so an inside click is swallowed (no dismiss); an outside click hits this backdrop → dismiss.
            root = root.child(
                div()
                    .absolute()
                    .inset_0()
                    .occlude()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|view, _e: &MouseDownEvent, _window, cx| {
                            view.agent_launcher = None; // click-away dismiss
                            cx.notify();
                        }),
                    ),
            );
            let mut overlay = div() … ;   // UNCHANGED
            … ; root = root.child(overlay);
        }
```

**Confirmed by reading:**
- The launcher box `.occlude()`s at the container (:5400) → an inside click is swallowed (no dismiss). ✓
- The launcher rows (:5411-5417) are `div().child(launch_command)` with NO `on_mouse_down` — selection is
  keyboard (↑↓/Enter, :1508-1524) → the backdrop shadows no row-click. ✓
- `MouseButton` + `MouseDownEvent` are imported at app.rs:16-17 (and used by the #166 menu in the same fn). ✓
- **MODAL-EXCLUSIVITY is already enforced:** the new-agent open site (:3153-3162) explicitly sets
  `palette_open = false; finder_open = false; history_open = false; find_open = false;` BEFORE
  `agent_launcher = Some(...)` ("Close any other overlay first... prevents a stacked palette/finder/history",
  an earlier inspect LOW). So while the launcher is open NO other overlay is → its backdrop is the only one →
  zero stacking/wrong-dismiss risk.
- Escape (:1500) + Enter-launch (:1502) paths are untouched.

**D — Left-click dismiss only** (the ask, chad #5). The #166 menu also added a right-click dismiss; for the
launcher that's optional parity, not the ask — Left-only is sufficient (a right-click while the launcher is open
is a non-goal). Noted; a trivial follow-up if ever wanted.

### File manifest
| File | Kind | Change |
|---|---|---|
| `crates/marley_app/src/app.rs` | SHIM (render) | Add the full-screen `.inset_0().occlude()` backdrop (Left-click → `agent_launcher = None` + notify) as the first child of the `if let Some(launcher)` block, before the box. |

No pure-crate/test changes.

### Regression Test Plan
| REQ | Test | Note |
|---|---|---|
| REQ-001 | Driven: ⌘⇧A opens the launcher → click outside the box → it closes | The headline; ELSE env-considerate (chad's active shared desktop) → the mechanism (byte-identical to the shipped #166 backdrop) + code review |
| REQ-002 | Code review: the box `.occlude()`s (inside-click swallowed) + rows are keyboard-driven → no dismiss/selection regression; a driven inside-click if unlocked | Mechanism + review |
| REQ-003 | Code review: the :1500 Escape + :1502 Enter-launch paths are untouched; driven Esc/Enter if unlocked | Regression |

**No new unit tests** — gate-is-shim render fix; `marley_app/src/app.rs` is in the gates.sh coverage
`--ignore-filename-regex` + is `mutants::skip`, so no cov/MSI delta; the only logic is `agent_launcher = None`
(trivial state). The gate stays green [diff] from the existing suite. **Uncoverable path:** the live overlay
dismiss — verified by pixels (driven) or the byte-identical-to-#166 mechanism (env-considerate), never a unit test.

### Risks / decisions
- **R1 — modal-exclusivity:** RESOLVED — the launcher-open site already closes the other overlays, so the
  backdrop can't stack. No change needed.
- **R2 — order:** the backdrop MUST be added before `root = root.child(overlay)` so the box hit-tests first
  (paints after). `cargo check` + the render structure confirm at implement.
- **R3 — validation:** env-considerate if chad's still on the shared desktop → mechanism (the #166 pattern is
  live-shipped + proven); offer the 20s driven check when free.
- Decisions D1-D3 per the spec; Left-click-only dismiss (right-click parity deferred as a non-goal).

## Phase 3 — Implement
**Built (manifest as designed) — `crates/marley_app/src/app.rs`, one insertion:** added the full-screen
`.inset_0().occlude()` backdrop as the first child of the `if let Some(launcher) = &self.agent_launcher {` block
(before the box), with `on_mouse_down(MouseButton::Left, |view, _e, _w, cx| { view.agent_launcher = None;
cx.notify(); })` — a byte-identical mirror of the #166 context-menu backdrop. The box + rows + prompt +
`root = root.child(overlay)` are unchanged.

**Deviations from design:** none. `MouseButton`/`MouseDownEvent` were already imported (:16-17); compiled first
try.

**Checks:** `cargo fmt` clean; `cargo check -p marley --all-targets` ✓ (the `block v0.1.6` note is the
pre-existing gpui-transitive warning); `cargo clippy -p marley --all-targets -- -D warnings` exit 0.

## Phase 3.5 — Inspect
1 focused general-purpose critic (proportionate to a ~15-line mirror of shipped code) + self-review. The critic
confirmed the fix is CORRECT — the Left-click dismiss is byte-equivalent to the proven #166 backdrop (dismiss
works, inside-click swallowed by the `.occlude()` box, no regression, borrow clean, `MouseButton`/`MouseDownEvent`
in scope). Two LOW findings, BOTH FIXED at source:

- **F1 [LOW — REAL, fixed] the launcher-open guard omitted `naming_workflow`.** The `agent_launcher = Some`
  opener (:3162) closed palette/finder/history/find but NOT the #204 save-as-workflow draft, which renders BEFORE
  the launcher. Reachable via the 🧠 top-bar icon (bypasses the key router): open a naming draft → click 🧠 → the
  launcher opens over it. Pre-#229 the naming box peeked around the launcher box; **#229's full-screen dismiss
  backdrop now FULLY HIDES it** while its keyboard guard (:3512) still owns input → a confusing invisible-focus
  state. **Verdict: REAL (LOW) — #229 worsened a latent co-open edge.** **Fix:** added `self.naming_workflow =
  None;` to the guard (:3158-3161), completing its stated "close any other overlay first" intent. Recorded as a
  failure + a class prevention-rule.
- **F2 [LOW — parity, fixed] the backdrop dismissed on Left-click only; #166 dismisses on Left AND Right.** A
  right-click outside the launcher was absorbed by `.occlude()` but had no handler → did nothing (a dead state).
  Since the ask is "click outside dismisses" and a right-click outside IS an outside click, **added the Right
  `on_mouse_down` handler** (mirror #166, :6280-6286) → any outside click, left or right, dismisses. Consistent +
  no dead absorb.

**Verify:** `cargo fmt` clean; `cargo check -p marley` ✓; `cargo clippy -p marley --all-targets -- -D warnings`
exit 0 after both fixes. Lenses covered: dismiss correctness, no-regression (box/rows/prompt/Esc/Enter unchanged),
modal-exclusivity (guard now complete), layering (backdrop scoped, all richer overlays render after), borrow/scope,
clean-room §20.

## Phase 4 — Validate
**Tests added:** NONE — gate-is-shim render fix. `marley_app/src/app.rs` is cov-excluded + `mutants::skip`; the
only logic is `Option` state clears (the backdrop dismiss + the `naming_workflow = None` guard line), no pure
surface changed → no unit test to add (per §7). Confirmed against the design test plan.

**RUN:** `cargo nextest run -p marley` → **312 passed, 2 skipped** (all existing green — the backdrop + the
guard line broke nothing).

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** (no cov/MSI delta — app.rs render
excluded/skip). Receipt written for `/commit`.

**Driven capture (REQ-001..003) — DEFERRED (env-considerate), verified by mechanism:** the desktop-state check
showed chad STILL actively on a SHARED desktop (live agent "Validating A1 ward-dbal slice… 30m" thinking;
"desktop shared with chadpeppers2@gmail.com" banner) with no Marley running — launching Marley + driving ⌘⇧A +
an outside-click would hijack his active screen. For a backdrop byte-identical to the shipped, live-proven #166
context-menu dismiss, that intrusion isn't warranted. Verified by the MECHANISM: the Left/Right `on_mouse_down →
agent_launcher = None + notify` on a full-screen `.inset_0().occlude()` backdrop drawn before the box is
character-for-character the #166 pattern (REQ-001); the box `.occlude()`s so an inside click is swallowed
(REQ-002 — no dismiss); Escape/Enter (:1500/:1502) are untouched (REQ-003); the inspect critic confirmed all of
this concretely + the F1 guard fix. **Offer a 20s driven check (⌘⇧A → click a corner → the launcher closes)
when chad's screen is free** — no separate ticket. gate-15 is headless → green regardless.

**Pre-existing failures:** none.

## Phase 5 — Complete
**Docs (§21):** CHANGELOG.md — #229 at the top of `[Unreleased] ### Fixed` (above #228). app_shell.md — a #229
note on the #181 launcher section (the click-away backdrop + the naming_workflow guard).

**Forge capture (§19):**
- `aar-submit` efe9cef1 — completed, effectiveness 5. Lessons: (a) the #166 context-menu backdrop is the
  codebase's reusable modal-dismiss idiom — a full-screen `.inset_0().occlude().on_mouse_down(Left[+Right] →
  state=None + notify)` drawn BEFORE the box (which `.occlude()`s so inside-clicks are swallowed); mirror it
  byte-for-byte for any new modal; (b) a full-screen dismiss backdrop HIDES (not just partially occludes) every
  overlay that renders before it → the modal's open path must close all co-open earlier overlays (F1); (c) mirror
  #166 FULLY — it dismisses on BOTH left AND right click-away (#175); #229 initially added only Left (F2).
- Recorded at INSPECT (referenced, not duplicated): `failure-record`
  **BF-claude-launcher-dismiss-backdrop-hides-coopen-naming-draft-001** (df2c21e0) + `prevention-rule`
  **PR-claude-fullscreen-dismiss-backdrop-must-close-earlier-overlays-001** (739601c4).

**Env note:** a 20-second driven check (⌘⇧A → click a corner → the launcher closes) when chad's screen is free —
no separate ticket (the byte-identical-to-#166 mechanism + the passing suite carry it).

**Close + archive:** forge #229 → done. Local TICKET-229 → closed/. Pipeline doc pair → completed/. Spec status
→ Phase 5 — Complete PASS.
