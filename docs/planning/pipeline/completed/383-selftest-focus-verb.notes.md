# selftest — layout-independent `focus`; fixed click point misfires on restored layouts — Notes

- **Forge ticket:** #383 (4ddc8936-2e8f-4b0e-a3ce-5f7ec0926770)
- **AAR:** 241d89e1-3b32-482b-9e16-c4950469c9f6
- **Local ticket doc:** docs/planning/tickets/open/TICKET-383-selftest-focus-verb.md
- **Pipeline spec:** 383-selftest-focus-verb.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** Forge #383, chore, sprint #36 "M25 — App-Grade QA Hardening" (milestone M25).
  **Provenance — the 2026-07-21 live QA misfire:** the QA run's first `drive.swift focus` was
  issued against a RESTORED workspace (Files panel open + an editor split). `focus` clicked its
  fixed point — (0.5, 0.12) of the window — which on that layout was a file-tree row: it silently
  clicked `.cargo/audit.toml`, opened it in the Editor tab, and the subsequent `type:` keystrokes
  went somewhere unhelpful. Several diagnostic rounds were spent suspecting the APP before the
  HARNESS was identified as the culprit. Same run re-confirmed the older rule live: activation
  (`osascript … frontmost`) issued in a SEPARATE shell command from the drive verbs → the host
  terminal regains frontmost between tool-call shells → keys silently vanish. Two deliverables:
  a layout-independent `focus` + a README checklist promoting both rules. Harness-only; no app
  code (the app behaved correctly — opening a clicked tree row IS the product behavior).
- **Classification / tier:** chore, M25, full pipeline — a **GATE-IS-TEST ticket** (CONSTITUTION
  §7: no `.rs`; harness Swift + README only → verification = driven evidence + captures +
  negative baselines + README diff; no cov/MSI surface; drive.swift is Swift, outside gate:11's
  `scripts/*.sh` shellcheck glob). Edit surface: `scripts/selftest/drive.swift` (the `focus` arm
  :212-214 + the usage-header line :11), `scripts/selftest/README.md`. `bundle-app.sh` cited for
  flow only, untouched. **Machine-safety constraint is binding on Phase 4:** synthetic input hits
  the frontmost window at screen points — driven runs only when the screen is unlocked AND chad
  is not at the keyboard; WAIT if occupied (mechanism-only fallback would gut this ticket, whose
  subject IS live driving).
- **Forge recall (§18.3):** drafted DOCS-ONLY (no MCP calls in this lane); recall drawn from the
  on-disk/session record — the two selftest prevention rules
  (`PR-claude-selftest-focus-marley-before-driving-input`: synthetic scroll/click hits the
  FRONTMOST window at the screen point, so activation ordering is load-bearing — the SAME
  mechanism behind both of this ticket's rules; and
  `PR-claude-selftest-screencapture-shadow-offsets-small-target-clicks`: fresh-capture coordinate
  reading must account for window-shadow offsets on small targets — relevant to the
  fresh-capture-`clickat:` rule the README will teach); README:75-80's own record that "type:
  works" hinges on staying frontmost. Live knowledge-context/bulletins recall re-runs at /work
  promotion.
- **Discovery (all verified by reading; cites are lines actually seen):**
  - **The `focus` arm (drive.swift:212-214):** `guard let w = marleyWindow() else { … exit(2) }`
    (:213) then `click(w.x + w.w / 2, w.y + w.h * 0.12); usleep(300_000)` (:214) — the fixed
    fraction is **exactly (0.5, 0.12)**, confirming the QA finding. On the 1024×768-pt default
    that is (512 pt, ~92 pt) — a CONTENT point (the boot-default terminal), not chrome. The
    usage header (:11) documents "left-click the Marley window's top area to bring it frontmost
    + focus it" — the "+ focus it" contract is precisely what a restored layout falsifies.
  - **Window discovery (drive.swift:26-38):** `CGWindowListCopyWindowInfo(.optionOnScreenOnly)`,
    owner name lowercased-contains "marley", LARGEST area wins. Unchanged by this ticket.
  - **Click plumbing (drive.swift:40-52):** `click()` posts down/up with explicit `flags = []`
    (:42-44 — the latched-⌘ trap) — any click-variant `focus` must reuse it unchanged.
  - **Verb inventory (what exists; the spec builds on it):** `find`/`check` read-only (:195),
    `focus` (:212), `clickat:` (:215-221 — fraction parse + click + 300 ms settle: THE existing
    pane-targeting primitive), `rightclickat:` (:222), `scrollat:` (:229), `drag:` (:238),
    `cmdclick:` (:248), `cmdshiftclick:` (:255), `cmd:` (:262), `cmdshift:` (:270), `alt:`
    (:279), `ctrl:` (:288), `cmdopt:` (:298), `type:` (:308), `wait:` (:310), `dblclickat:`
    (:312), `clearmods` (:317), bare keys via the keycodes map (:190 — enter/tab/esc/up/down/
    backspace/left/right/f12/f2).
  - **AX preflight (drive.swift:195-204):** `readOnly = {find, check}`; any other non-`wait:`
    action requires `AXIsProcessTrusted()` else stderr + **exit 3**. D5 pins `focus` in the
    event-posting set even if the chosen mechanism stops posting CGEvents.
  - **README — what exists vs what the checklist ADDS:** the Usage flow (:37-59) shows
    `open target/Marley.app` (:40) then `osascript … frontmost` as a SEPARATE command (:41) then
    the drive invocation (:43-46) — the example itself teaches the keys-vanish hazard. The
    same-command rule IS already present but BURIED as a sub-clause of the "type: WORKS
    (retested 2026-07-08)" gotcha (:78-80: "re-activate in the SAME command as the drive
    verbs"), which also carries the now-stale "(`focus` first)" advice (:80). The verb list
    describes focus as "click the window to bring it frontmost + focus the pane" (:49) — stale.
    Also already documented (kept, not restructured): largest-on-screen-window (:83-84), pkill
    stale instances (:85), rebuild-before-capture (:86-88), the bare-binary cwd lane (:89-93 —
    "you just can't `System Events`-activate it", :92-93 — load-bearing for D-OPEN-RAISE).
    NOT documented anywhere: the restored-layout hazard / focus-is-activation-only /
    fresh-capture-`clickat:` rule — that is the checklist's genuinely new half.
  - **Top-strip chrome is INTERACTIVE (README:61-68):** on the 1024×768-pt default the top-bar
    icons live at fy ≈ 0.018 — file ≈ 0.095, "+" ≈ 0.127 (proven M8 #140 to spawn a terminal),
    sparkle ≈ 0.157. Two consequences: (1) the KNOWN affordances stop at x ≈ 0.157, so a
    centered title-strip click at x = 0.5 clears them; (2) the strip is CLIENT-DRAWN chrome, so
    top-center inertness is an assumption needing a Phase-2 capture probe (P-B), not a given.
  - **bundle-app.sh (flow cite only):** always-rebuild (:16-23); `CFBundleIdentifier
    com.ignibyte.marley` (:36) — the id a programmatic raise would target; ad-hoc codesign keys
    the stable AX identity (:48-49). Untouched by this ticket.
- **Geometry / fork reasoning (recorded as D2 + D-OPEN-RAISE):**
  - Title-strip click geometry: y ≈ 0.02 of window height ≈ 15 pt on the 768-pt default —
    inside the title strip, far above today's 0.12 (~92 pt) content point. Traffic lights occupy
    the LEFT edge — roughly the left 4-8 % of width on the 1024-pt default (x ≲ 0.08) — so
    **x = 0.5 centered is safe from them by construction**; known icons extend to x ≈ 0.157,
    also cleared. Residual risk: the strip is client-drawn and the M-era cockpit work made the
    top bar workspace-scoped — future centered content would regress the bug class. That
    fragility argues for the no-click arm.
  - The honest fork: `osascript … frontmost` is MANDATORY in the same-command checklist anyway —
    so does `focus` need a click at all? Arm (b) (programmatic raise —
    `NSRunningApplication.activate` / AX `kAXRaiseAction`; ApplicationServices is already
    imported, drive.swift:22) is zero-content-risk by construction, but its coverage of the
    bare-binary launch lane is UNPROVEN (README:92 proves only that System Events can't activate
    that lane; a pid-keyed NSRunningApplication may still work → probe P-A). Arm (a)
    (title-strip click) covers every CGWindowList-visible window including the bare-binary lane,
    at the cost of the inertness assumption (probe P-B). Under (b), `focus` may honestly reduce
    to raise + settle — still worth keeping as a verb (fail-loud exit codes; one word; guards
    the caller who forgot the osascript step). Recorded as (a)/(b)/(c-hybrid) for Phase 2.
  - Pane-targeting verb: `clickat:` already takes a caller-supplied fraction and both verbs end
    with a 300 ms settle (:214, :221) — the composition `focus clickat:<fx>,<fy>` may be the
    whole answer (zero new code). D-OPEN-PANE-VERB records the lean; a dedicated
    `focusat:<fx>,<fy>` only if the probe shows the composition insufficient.
- **Verification class (§7 gate-is-test, no `.rs`):** REQ-001's proof = drive the NEW `focus`
  against the restored-layout fixture and show via before/after captures that NO content was
  touched; its negative baseline is cheap and exact — `clickat:0.5,0.12` IS the old behavior
  verbatim, so the misfire reproduces without any code stash. Boot-default flow, the targeting
  idiom, exit-2, and one representative other-verb smoke round it out; the AX exit-3 arm is
  review-verified (revoking the session's own AX grant would break the session's driving —
  stated honestly). Plus the README diff. Machine-safety window binding (above).
- **Decisions:** D1 activation-only; D2 geometry envelope (x = 0.5 clears traffic lights ~left
  4-8 % and known icons ≤ 0.157; y ≈ 0.02; inertness assumption named); D3 one prominent
  checklist + fix the Usage example + correct :49/:80; D4 scope wall (selftest only; AX pane
  discovery OUT); D5 exit-2/exit-3 contracts preserved, focus stays event-posting-classified.
  **D-OPEN-RAISE** (click vs raise vs hybrid; probes P-A/P-B) and **D-OPEN-PANE-VERB**
  (composition vs dedicated verb) go to Phase 2.
- **EARS (full table in the spec):** REQ-001 restored-layout activation with zero content
  interaction (+ negative baseline that bites); REQ-002 boot-default canonical flow still works;
  REQ-003 README same-command activation checklist + fixed Usage example; REQ-004 README
  fresh-capture-`clickat:` rule + stale-claim corrections (:49/:80/drive.swift:11); REQ-005
  documented caller-supplied-fraction targeting idiom, driven-proven; REQ-006 exit-2/exit-3
  failure contracts; REQ-007 every other verb behavior-identical (+ representative `clickat:`
  smoke). Seven rows.
- **Forge ids:** ticket #383 = 4ddc8936-2e8f-4b0e-a3ce-5f7ec0926770 (sprint #36 "M25 —
  App-Grade QA Hardening"); pipeline_id = 1a4f775f-a789-465e-995e-d1d9a9f16a3d; AAR
  241d89e1-3b32-482b-9e16-c4950469c9f6.
- **Plan verification (Opus /work, 2026-07-22) — bug confirmed + D-OPEN-RAISE settled by THIS
  session's own captures:**
  - The bug is exactly as specced: `drive.swift:214` `focus` arm = `click(w.x + w.w / 2,
    w.y + w.h * 0.12); usleep(300_000)` — a FIXED (0.5, 0.12) content click (read directly).
  - The AX preflight (:195-204): `readOnly = ["find", "check"]`; `focus` is NOT in it → it counts as
    event-posting → the exit-3 fail-loud fires for any invocation containing `focus` (D5 preserved as
    long as `focus` stays out of `readOnly`, regardless of the raise mechanism). No-window → exit 2
    (:213). `click()` (:40-52) has the `flags = []` latched-⌘ guard. `clickat:` (:215-221) exists.
    Imports: ApplicationServices, CoreGraphics, Foundation (NO AppKit yet — arm (b) adds it).
  - **D-OPEN-RAISE → arm (b) programmatic raise, DECIDED BY EVIDENCE from this session's captures:**
    the unified titlebar (#138) is CLIENT-DRAWN and its center-top holds the "Search sessions, agents,
    files…" input (~x 0.34-0.66, y≈0.02 — visible in `381-boot.png`/`382-after-*.png`). So arm (a)'s
    (0.5, 0.02) title-strip click would hit the SEARCH BAR, not inert chrome — arm (a) is unsafe and
    fragile (the D2 inertness assumption is FALSE at x=0.5). Arm (b) — a programmatic raise via
    `NSRunningApplication(processIdentifier: pid).activate()` (pid-keyed → covers BOTH the bundle and
    the bare-binary lanes, answering P-A) — is layout-independent BY CONSTRUCTION, no geometry
    assumption. `focus` reduces to raise + settle (belt-and-suspenders with the mandated same-command
    `osascript frontmost`). Needs `import AppKit` + the marley pid (add to `marleyWindow()`'s return
    via `kCGWindowOwnerPID`).
  - **D-OPEN-PANE-VERB → the zero-code composition `focus clickat:<fx>,<fy>`** (both primitives exist,
    each ends with a 300 ms settle) — documented in the README; NO new verb.
  - Verdict: plan solid; the fix is arm (b) + the composition + the README checklist. Design formalizes
    the fixture recipe + README layout + verifies REQ-002 (raise-only leaves the terminal focused for
    the typed flow).

## Phase 2 — Design

**Architecture / approach.** A harness-only chore in `scripts/selftest/` — no Marley crate touched, no
gate semantics. §20 N/A confirmed (internal test tooling; no reference-app behavior to match). The
`focus` verb stops CLICKING content and becomes a programmatic app raise, layout-independent by
construction; pane targeting moves to the caller via the existing `clickat:` (documented). The README
gains a pre-drive checklist promoting the two rules that cost a QA session.

**D-OPEN-RAISE → arm (b) programmatic raise (DECIDED, no probe needed).** Evidence from this session's
own captures: the unified titlebar's center-top holds the "Search sessions…" input, so arm (a)'s
(0.5, 0.02) click hits it — arm (a) is unsafe AND fragile. Arm (b) `NSRunningApplication(processIdentifier:
pid).activate()` is layout-independent by construction and pid-keyed → covers BOTH the bundle and the
bare-binary launch lanes (P-A answered by the design; P-B/inertness moot with no click).
**D-OPEN-PANE-VERB → the zero-code composition `focus clickat:<fx>,<fy>`** (no new verb).

**drive.swift changes (exact):**
```swift
import AppKit                                   // NEW (NSRunningApplication) — after the 3 existing imports
// marleyWindow(): return tuple gains `pid: Int` from `w[kCGWindowOwnerPID as String] as? Int ?? -1`
//   (other verbs destructure by name — w.x/w.w/… — so adding w.pid is behavior-identical, REQ-007)
} else if action == "focus" {
    guard let w = marleyWindow() else { FileHandle.standardError.write("no marley window\n"…); exit(2) }
    // #383: activation-ONLY, layout-independent — a programmatic raise, NEVER a content click (the old
    // fixed (0.5,0.12) click hit a file-tree row on a restored layout). pid-keyed → both the bundle and
    // bare-binary lanes. Pane targeting is the caller's job now: `focus clickat:<fx>,<fy>`.
    _ = NSRunningApplication(processIdentifier: pid_t(w.pid))?.activate()
    usleep(300_000)
}
```
- Usage header `:11` rewritten: `focus  bring the Marley window frontmost (programmatic raise;
  layout-independent, no content click). Pane targeting: focus clickat:<fx>,<fy>`.
- `focus` stays OUT of `readOnly` (:195) → the exit-3 AX preflight still fires for any `focus`
  invocation (D5 — a caller's following `type:` needs AX; a silent-no-op focus would resurrect the
  dropped-keys false-negative). No-window → exit 2 unchanged (:213).
- `_ = …?.activate()` binds the `Bool` result (no unused-result warning); the `?` handles a
  no-such-pid nil (e.g. the window vanished between find and activate) as a silent no-op + settle.

**README.md changes:**
| Edit | What |
|---|---|
| NEW `## Pre-drive checklist` (between `## Usage` and `## Gotchas`) | Rule 1: activate (`osascript … frontmost`) in the SAME shell command as the drive verbs — keys silently vanish when the host terminal regains frontmost between tool-call shells (re-confirmed live 2026-07-21). Rule 2: `focus` is activation-only; target a pane with `clickat:<fx>,<fy>` using coordinates read from a FRESH capture of the CURRENT layout — never `focus`. |
| Usage example (`:41`, the separate `osascript` line) | → one compound `osascript … frontmost && swift drive.swift focus type:… enter` command (the shape the checklist mandates). |
| Stale claims `:49`, `:80` | `:49` "focus … + focus the pane" → "bring the window frontmost (no content click)"; `:80` "(`focus` first)" → the same-command activation rule / `clickat:` for targeting. |

**File manifest:** `scripts/selftest/drive.swift` (import + marleyWindow pid + the focus arm + usage
header) and `scripts/selftest/README.md` (checklist + usage fix + :49/:80). NOTHING else — no
`crates/*/src`, no `gates.sh`, no hooks. Neither file is in `gate_state_hash` (.rs/.sh/.toml/.lock),
so the #382 receipt stays valid and the change is gate-neutral (verified via the driven evidence + the
static gate).

**Regression Test Plan (GATE-IS-TEST, §7 — harness Swift + README, no `.rs`; verification = driven
evidence + captures + the README diff, per the spec header):**
| # | Proof | AC |
|---|---|---|
| `swift drive.swift check` runs → `AX_TRUSTED` | the script parses + executes (the interpreter is the compile proof) | — |
| Restored layout (the persisted Files+editor-split workspace, already in hand): `focus` → capture shows NO tree file opened / NO new Editor tab; negative baseline `clickat:0.5,0.12` on the same layout REPRODUCES the misfire (a tree row opens) | REQ-001 |
| Boot-default: `osascript frontmost && swift drive.swift focus type:echo\ hi enter` → capture shows the `echo hi` block ran (proves raise-only leaves the terminal focused for typing) | REQ-002 |
| Restored layout: `focus clickat:<terminal-fraction>` → a typed probe renders in THAT pane | REQ-005 |
| No Marley running: `swift drive.swift focus` → stderr + exit 2 (negative smoke); the AX exit-3 arm is review-verified (revoking the session's own grant would break driving — stated honestly) | REQ-006 |
| Diff review: the delta is confined to the `focus` arm + marleyWindow pid + usage header; a representative `clickat:0.127,0.018` boot-default smoke still spawns a 2nd terminal (the proven affordance) | REQ-007 |

**Risks / decisions:** REQ-002 hinges on the boot-default terminal being ALREADY focused (so raise-only
suffices) — verified at Validate by the driven echo. Fixture = the persisted restored workspace (no
special setup). The driven runs need a safe input window (screen unlocked, chad away) — WAIT if blocked
(the spec header rule; live driving IS this ticket's subject, so no mechanism-only substitute).

## Phase 2 — Design (status)
Design PASS — arm (b) settled by evidence, drive.swift + README manifest exact, GATE-IS-TEST plan per
REQ. Ready for Implement.

## Phase 3 — Implement (done)
Built to the manifest — `scripts/selftest/` only:
- **`drive.swift`**: `import AppKit`; `marleyWindow()` return tuple gained `pid: Int` (from
  `kCGWindowOwnerPID`); the `focus` arm is now `_ = NSRunningApplication(processIdentifier: pid_t(w.pid))?.activate(); usleep(300_000)` — a programmatic raise, NO content click (guard→exit 2 unchanged);
  the usage-header `focus` line rewritten to the new contract. `focus` stays OUT of `readOnly` → the
  exit-3 AX preflight still fires.
- **`README.md`**: NEW `## Pre-drive checklist` section (rule 1 same-command activation, rule 2
  focus-is-activation-only + `clickat:` with fresh-capture coords); the Usage example folded to ONE
  compound `osascript … frontmost && swift … focus …` command (the standalone osascript line removed);
  the `focus` verb line + the stale ":49"/":80" pane-focus claims corrected.

**Deviations: none.** `swift scripts/selftest/drive.swift check` → **AX_TRUSTED** (the script parses +
executes — the compile proof for a Swift script; the `AppKit`/`NSRunningApplication` references are
valid, no deprecation error on this macOS); `find` parses with the new pid field. Diff confined to
drive.swift + README (no `crates/*/src`, no `.sh`, no gates) — neither file is in `gate_state_hash`, so
the change is gate-neutral. REQ-004 grep: no stale focus-the-pane text remains.

## Phase 3 — Implement (status)
Implement PASS — drive.swift + README edits parse/verify; harness-only; ready for Inspect.

## Phase 3.5 — Inspect
Two critics + an EMPIRICAL driven test (the deciding evidence). **Verdict: PASS — 1 HIGH fixed
(empirically confirmed), 1 LOW folded into the same fix, rest CLEAN.**

**[HIGH — critic 1, EMPIRICALLY CONFIRMED then fixed] Raise-only `focus` does not set gpui keyboard
focus → `focus type:…` types nowhere (REQ-002 broke).** Critic 1 traced (gpui-0.2.2 source + the app's
own `headless_drive.rs:158-160` "a test window starts unfocused… injected keystrokes go nowhere"): the
OLD `click(0.5,0.12)` was LOAD-BEARING — a mouse-DOWN on the root `track_focus` div is what set
`window.focus` (gpui div.rs auto-focuses on mouse-down); a programmatic `activate()` posts no mouse-down,
and nothing else sets `self.focus` in production (`workspace.focus(PaneId(0))` at app.rs:2165 is the
workspace-internal pane model, NOT gpui focus). I did NOT dismiss it despite contradicting #381 evidence
— I DROVE it (machine free): `osascript frontmost && drive.swift focus "type:echo REQ002_probe" enter` →
capture (`383-req002.png`) shows the terminal at a bare `> Marley` prompt, **the echo never ran**. Then
`clickat:0.6,0.5 "type:echo CLICKPROBE" enter` → capture (`383-clickprobe.png`) shows **`✓ echo
CLICKPROBE` + `CLICKPROBE` output**. So: a `clickat:` mouse-down focuses a pane; a raise does not.
- **FIX (harness-only, honors D1):** `focus` STAYS raise-only (correct — it kills the misfire, the
  ticket's point). The TYPING flow moves to `clickat:<pane> type:…` (the mouse-down sets focus). Corrected
  every `focus type:` example → `clickat:<pane> type:`: drive.swift usage `:21` + verb desc `:11`, README
  Usage `:48`, checklist rule-1 example `:83`, checklist rule-2 (now states the gpui mouse-down fact +
  the empirical result), the historical gotcha `:100`. Spec **REQ-002 REVISED** to the `clickat:` flow
  with the two captures as evidence. **Rejected** the app-side `window.focus(&self.focus_handle)` boot
  fix — it is `.rs` (scope creep for a harness chore) and D1 already chose "focus is activation-only;
  the caller clicks a pane." No app code touched.

**[LOW — critic 2] Historical/ordering examples modeled bare `focus`→`type:`** (README:97, drive.swift:21)
— FOLDED into the HIGH fix (the same `clickat:` correction covers them). Confirmed no bare `focus type:`
example remains (grep: the only residual `focus…type:` is the D5 preflight COMMENT at drive.swift:227).

**Confirmed CLEAN (both critics, against source):**
- Raise API correct — `NSRunningApplication(processIdentifier: pid_t(w.pid))?.activate()` is the macOS
  14+ replacement (not the deprecated `activate(options:)`); compiles + runs (`swift … check` →
  AX_TRUSTED); `_ =` discards the `Bool`. Cooperative-activation-may-no-op is a LOW, mitigated by the
  mandated same-command `osascript frontmost`.
- REQ-007 — all 9 `marleyWindow()` accesses are BY NAME (`w.x`/`w.w`/`w.num`/`w.h`/`w.pid`), zero
  positional; the unnamed `best` tuple coerces to the named return; other verbs behavior-identical.
- D5 — `focus` no-window → exit 2; stays OUT of `readOnly` → exit-3 AX preflight still fires (guards a
  following `type:` even though the raise posts no CGEvent). No content-click remains in the `focus` arm.
- Edge cases — off-screen → not found → exit 2; multi-instance → activates the largest window's pid
  (more precise than a screen-point click); missing-pid → `-1` → nil → silent no-op (harmless).
- README consistency — no contradiction with unedited passages; usage matches the same-command rule;
  drive.swift header agrees; §20 N/A (the only Warp mention is the host-terminal Accessibility note, not
  source); no secret; markdown well-formed (fences balance, `##` level correct).

**Fixes applied: 1** (the HIGH — `focus`-doesn't-focus correction across examples + the gpui-focus fact +
REQ-002 revision). `failure-record` + `prevention-rule-record` filed. Re-verified live (the two captures).

## Phase 3.5 — Inspect (status)
Inspect PASS — the HIGH (raise-only drops keyboard focus) confirmed by driving + fixed; the typing flow
is now the layout-safe `clickat:<pane> type:`; ready for Validate.

## Phase 4 — Validate
GATE-IS-TEST (harness Swift + README, no `.rs` → no unit tests; verification = driven REQ evidence +
captures + the README diff, per §7 + the spec header). Machine was free; all REQs driven for real.

- **Parse/exec proof:** `swift scripts/selftest/drive.swift check` → **AX_TRUSTED** (the script type-checks
  + runs; the `import AppKit` + `NSRunningApplication.activate()` are valid).
- **Regression (enforce-tests-ran):** `cargo nextest run -p marley` → **808 passed, 2 skipped** —
  unchanged from #382 (harness-only added no tests; nothing broke).
- **REQ-006:** `swift drive.swift focus` with NO Marley → stderr `no marley window` + **exit 2**. ✓
- **REQ-001 (the core fix):** launch restored layout (Files + terminal + editor split) → NEW raise-only
  `focus` → before/after captures **PIXEL-IDENTICAL** (`cmp -s`; `383-req001-before.png` ==
  `383-req001-after-focus.png`) — the raise touches NO content, can't misfire. Negative baseline: a fixed
  content click at the `audit.toml` tree row (`clickat:0.21,0.135`) → **audit.toml OPENED in the Editor**
  (`383-req001-treeclick.png`) — the EXACT 2026-07-21 QA misfire reproduced (the old `focus`'s content
  click). So the new `focus` can't open a file; a content click does. ✓ (Bonus: that capture's footer
  read "focus: editor" — #382 re-confirmed live.)
- **REQ-002 (revised) + REQ-005 (the Inspect HIGH's proof):** `focus "type:echo REQ002_probe" enter` →
  the terminal stayed at a bare `> Marley` prompt, **the echo did NOT run** (`383-req002.png`) — a raise
  sets no gpui keyboard focus. `clickat:0.6,0.5 "type:echo CLICKPROBE" enter` → **`✓ echo CLICKPROBE`
  block + `CLICKPROBE` output** (`383-clickprobe.png`) — the `clickat:` mouse-down focuses the pane, then
  typing lands. The corrected `clickat:<pane> type:` flow works; the README/usage now teach it. ✓
- **REQ-007:** representative clickat smoke — `clickat:0.127,0.018` (top-bar "+") → spawned a terminal
  (`383-req007-plus.png` differs from the prior frame); the tree-click + clickprobe already showed
  `clickat:` behaves unchanged; the diff is confined to the `focus` arm + `marleyWindow` pid + the usage
  header (no other verb touched). ✓
- **Full gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** (drive.swift `.swift`/README
  `.md` are gate-neutral — not in `gate_state_hash`, outside shellcheck's `*.sh` glob; brand-scrub did
  not flag the pre-existing host-terminal "Warp" mention; the #382 receipt stays valid for /commit). ✓

No pre-existing failures. Captures archived in the scratchpad.

## Phase 4 — Validate (status)
Validate PASS — every REQ driven-proven (incl. the Inspect HIGH's before/after), suite green, gate GREEN
[diff]. Ready for Complete.

## Phase 5 — Complete
- **CHANGELOG:** entry under `### Fixed` (the selftest `focus` verb → layout-independent app raise; the
  README pre-drive checklist; the gpui `clickat`-sets-focus fact).
- **Architecture docs:** `docs/marley_architecture/marley_visual_harness.md` — a cross-ref on the
  headless "FOCUS the root handle (a test window starts unfocused)" rule noting the driven/pixel lane's
  equivalent (#383: a `clickat:<pane>` mouse-down sets gpui keyboard focus; a raise/`focus` does not).
  The `scripts/selftest/README.md` is the harness's own doc (updated in Phase 3).
- **Forge knowledge:** `aar-submit` 241d89e1 (completed, effectiveness 4 — a real defect [raise-only
  drops focus] + a wrong spec premise [REQ-002], both caught at Inspect by a critic + a driven test and
  fixed). `BF-claude-raise-only-focus-drops-gpui-keyboard-focus-001` (failure) +
  `PR-claude-gpui-keyboard-focus-needs-a-mouse-down-not-a-raise-001` (rule) were filed at Inspect.
  `ticket-comment` + `ticket-close` #383 → done.
- **Ticket doc** → `docs/planning/tickets/closed/`, status closed. **Pipeline** → `completed/`.

Complete PASS.
