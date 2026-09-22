---
pipeline_id: 1a4f775f-a789-465e-995e-d1d9a9f16a3d
ticket: forge#383 (4ddc8936-2e8f-4b0e-a3ce-5f7ec0926770) · local docs/planning/tickets/open/TICKET-383-selftest-focus-verb.md
aar_id: 241d89e1-3b32-482b-9e16-c4950469c9f6
status: Phase 5 — Complete PASS
title: selftest — layout-independent `focus`; the fixed click point misfires on restored layouts
type: chore
milestone: M25
references:
  - scripts/selftest/drive.swift
  - scripts/selftest/README.md
  - scripts/selftest/bundle-app.sh
---

## Title
Fix the self-test driver's `focus` verb, which today left-clicks a FIXED window fraction —
`click(w.x + w.w / 2, w.y + w.h * 0.12)`, i.e. **(0.5, 0.12)**, then a 300 ms settle
(drive.swift:212-214) — a point chosen when the boot-default layout put the terminal there. On a
RESTORED layout (Files panel open + an editor split) that point is a file-tree row: the 2026-07-21
live QA run's first `focus` silently clicked the `.cargo/audit.toml` row, opened it in the Editor
tab, and the follow-on `type:` keystrokes landed unhelpfully — several diagnostic rounds were
spent on the app before the harness was identified. Two deliverables, both inside
`scripts/selftest/`: **(1)** make `focus` activation-ONLY and layout-independent (mechanism =
D-OPEN-RAISE: a title-bar-region click at a geometry-argued safe point vs a programmatic
raise-without-click; pane targeting becomes an explicit caller-supplied-fraction idiom);
**(2)** promote the two re-learned operating rules from buried gotcha prose to a prominent README
pre-drive CHECKLIST — activation in the SAME shell command as the drive verbs, and
fresh-capture-`clickat:` (never `focus`) for pane targeting on restored workspaces. Harness-only
chore: no app code, no gates.

## Scope
### In
- **`scripts/selftest/drive.swift` — the `focus` arm only (:212-214) + its usage-header line
  (:11, "left-click the Marley window's top area … + focus it" — the stale contract).** New
  contract: `focus` brings the Marley window frontmost WITHOUT interacting with content on ANY
  layout. Window discovery unchanged (`marleyWindow()`, :26-38 — largest on-screen CGWindow whose
  owner name contains "marley"). Any click-variant mechanism reuses `click()` unchanged (:40-52 —
  its explicit `flags = []` guards the latched-⌘ trap, :42-44). Failure contracts preserved
  (D5): no-window → stderr + exit 2 (:213); `focus` stays classified event-posting for the AX
  preflight (`readOnly` stays `{find, check}`, :195-198; exit 3 fail-loud :199-204) regardless of
  the mechanism chosen.
- **A caller-supplied-fraction pane-targeting idiom** (D-OPEN-PANE-VERB): the lean candidate is
  the ZERO-code composition `focus clickat:<fx>,<fy>` — `clickat:` already exists (:215-221),
  actions run in order, and both verbs end with a 300 ms settle — vs a dedicated `focusat:<fx>,<fy>`
  verb if Phase 2's probe shows the composition insufficient. Either way the idiom is documented
  in the README as THE way to target a pane.
- **`scripts/selftest/README.md` — a pre-drive CHECKLIST + three specific corrections.**
  (a) NEW checklist section (placement Phase 2 — natural slot is between `## Usage` and
  `## Gotchas`): activate (`osascript … frontmost`) IN THE SAME shell command as the drive verbs
  — keys silently vanish when the host terminal regains frontmost between tool-call shells
  (re-confirmed live 2026-07-21; today this rule exists only as a buried sub-clause of the
  "type: WORKS" gotcha, README:78-80); and NEVER trust `focus` for pane targeting on a restored
  workspace — read coordinates from a FRESH capture of the current layout and use `clickat:`.
  (b) Fix the Usage example that TEACHES the hazard — README:41 issues `osascript … frontmost` as
  a SEPARATE command from the drive invocation (:43-46); the canonical example becomes one
  compound activation+drive command. (c) Correct the two stale pane-focus claims: ":49 `focus` —
  click the window to bring it frontmost + focus the pane" and ":80 `(`focus` first)`" (advice to
  use `focus` to make keys land in the terminal).
- **Verification fixture (Phase 2 designs it, Phase 4 runs it):** a deterministic restored layout
  — Files panel open + an editor split — either driven into place via proven affordances or
  seeded through the shipped session-persistence path, so REQ-001's no-interaction proof and its
  negative baseline are reproducible.

### Out (explicitly deferred)
- **AX-based pane discovery** (walking the accessibility tree to find the terminal pane and click
  it semantically) — the durable end-state for layout-independent TARGETING, but a different
  ticket's size; noted as future work.
- Any app code — `crates/*/src` untouched; the app's focus model, layout restore, and file-tree
  behavior are all correct (the QA run proved the HARNESS was the culprit).
- `scripts/gates.sh`, hooks, gate semantics — nothing verdict-adjacent is edited.
- The other documented harness hazards (shadow-offset clicking, stale instances, bare-binary cwd
  lane, README:83-93) — already documented; not restructured beyond the checklist's cross-refs.
- Headless-lane changes (`headless_drive`) — the README's headless-first guidance (:7-18) stands
  unmodified; this ticket is the pixel/driven lane's tooling.

## Reference (§20)
**N/A — internal test tooling; no user-facing behavior.** `drive.swift` is the harness that
drives Marley from outside; there is no Warp/Zed BEHAVIOR to match — neither
docs/warp_architecture/ nor docs/zed_architecture/ maps a self-test-driver surface, and no
observed capture is meaningful for a tool whose job is to produce captures. Clean-room §20
untouched: no Warp (AGPL) / Zed (GPL) source is relevant or consulted.

### Prior art
1. **Behavior maps — N/A.** The Warp/Zed maps cover product behavior (terminal, editor, cockpit);
   no reference app ships this harness seam. Stated per the sweep rule.
2. **Published material — Apple platform documentation (the binding substrate):**
   - Quartz Event Services (CGEvent): synthetic events posted to `.cghidEventTap` are delivered by
     the window server via hit-testing at SCREEN coordinates to the frontmost/hit window — which
     is exactly why activation ordering is load-bearing and why a content-area click is
     layout-dependent. Documented platform behavior; the repo has re-proven it repeatedly
     (README:78-80; the prevention rule that `focus` must precede scroll/click in the SAME call).
   - Programmatic raise APIs exist WITHOUT a click: `NSRunningApplication.activate(options:)`
     (AppKit) and the AX `kAXRaiseAction` (ApplicationServices — already imported by
     drive.swift:22). These are the documented candidates for D-OPEN-RAISE arm (b).
   - macOS standard window chrome places the traffic-light controls at the LEFT edge of the title
     strip — the geometry premise of D2.
3. **Permissive deps — none: checked, no owner.** gpui, ropey, regex, alacritty_terminal,
   tree-sitter — none ships a synthetic-input driver seam; the harness is repo-local Swift. The
   real prior art is OUR OWN repo: `drive.swift` itself (the verb inventory + the `flags = []`
   and settle-wait idioms), `bundle-app.sh` (the stable-identity `.app` that makes activation
   reliable — bundle id `com.ignibyte.marley`, bundle-app.sh:36; ad-hoc signing keys the AX grant,
   :48-49), and the two recorded selftest prevention rules (focus-before-driving-input;
   screencapture-shadow-offsets-small-target-clicks).

## Locked-In Decisions
- **D1 — `focus` becomes activation-ONLY.** It shall never interact with content on any layout.
  The verb's old second job (putting keyboard focus in the terminal pane) moves to the CALLER via
  the explicit pane-targeting idiom with coordinates read from a fresh capture. Rationale: the
  misfire class is unfixable by picking a "better" fixed content point — every fixed content
  fraction is wrong on SOME layout; only chrome or no-click is layout-independent.
- **D2 — geometry envelope binding ANY click variant.** A title-strip click sits at y ≈ 0.02 of
  window height (~15 pt on the 768-pt default — well above today's 0.12 ≈ 92 pt content point).
  The traffic-light controls occupy the LEFT edge of the strip — roughly the left 4-8 % of width
  on the 1024-pt default window (x ≲ 0.08) — so a CENTERED x = 0.5 clears them by a wide margin.
  The KNOWN interactive top-strip affordances extend only to x ≈ 0.157 (README:63-64: file icon
  ≈ 0.095, "+" ≈ 0.127, sparkle ≈ 0.157 at fy ≈ 0.018) — x = 0.5 clears every known affordance.
  HOWEVER the strip is CLIENT-DRAWN chrome (those icons live in it), so "top-center is inert" is
  an ASSUMPTION that Phase 2 must verify by capture on BOTH layouts — and it is fragile against
  future top-bar content at center; this fragility is a live argument for arm (b) of D-OPEN-RAISE.
- **D3 — README shape: ONE prominent checklist, not more buried bullets.** The two rules become a
  single pre-drive checklist section; the Usage example (:40-46) is fixed to the same-command
  shape it currently contradicts; the two stale pane-focus claims (:49, :80) are corrected in
  place. The existing Gotchas list is kept (history has value) with the checklist as the
  authoritative front door.
- **D4 — scope wall.** `scripts/selftest/drive.swift` + `scripts/selftest/README.md` only. No
  `crates/*/src`, no gates.sh, no hooks. AX-tree pane discovery is OUT (future ticket).
- **D5 — failure contracts preserved.** No-window → stderr + exit 2 (:213). `focus` remains an
  event-posting action for the AX preflight (exit 3, :195-204) EVEN IF the chosen mechanism posts
  no CGEvent — a `focus` that silently no-ops without the AX grant would resurrect the
  dropped-keys false-negative class the preflight exists to kill (callers immediately follow with
  `type:`).
- **D-OPEN-RAISE (the honest fork — Phase 2 settles with two probes):** does activation need a
  click at all, given `osascript … frontmost` is mandated in the same-command checklist anyway?
  - **(a) Title-bar-region click at (0.5, ~0.02)** — stays inside the harness's
    everything-is-a-CGEvent model; covers the bare-binary launch lane (a synthetic click raises
    any CGWindowList-visible window, while System Events cannot activate that lane —
    README:89-93); residual risk = the client-drawn-chrome inertness assumption (D2).
  - **(b) Programmatic raise, no click** — `NSRunningApplication(bundleIdentifier:
    "com.ignibyte.marley").activate(…)` or AX `kAXRaiseAction` from inside drive.swift — zero
    content-interaction BY CONSTRUCTION; open question: does it cover the bare-binary lane
    (README:92 proves only that System Events can't — pid-keyed NSRunningApplication may still
    work)? Note honestly: under (b), `focus` may reduce to raise + settle — still worth keeping
    as a verb (one word, fail-loud exit codes, works when a caller forgets the osascript step).
  - **(c) Hybrid** — (b) first, fall back to (a) if the process isn't activatable.
  - Probes: **P-A** — programmatic raise vs the bare-binary lane (launch the bare binary, attempt
    (b), observe). **P-B** — capture-verify top-center inertness on boot-default AND the restored
    layout (click (0.5, 0.02), diff captures).
- **D-OPEN-PANE-VERB (Phase 2):** dedicated `focusat:<fx>,<fy>` vs the zero-code composition
  `focus clickat:<fx>,<fy>`. Lean: composition + README documentation — both primitives exist and
  each already ends with a 300 ms settle (:214, :221); add a verb only if the probe shows the
  composition insufficient (e.g. the raise needs a longer settle before a reliable click).

## Acceptance Criteria (EARS)
GATE-IS-TEST ticket (harness Swift + README only, no `.rs`): per CONSTITUTION §7, verification =
executing the verb against the live app and reading the result (captures), negative baselines that
prove the smoke bites, and the README diff — not invented unit tests. Driven runs REQUIRE a safe
input window (screen unlocked, chad not at the keyboard — synthetic input hits the frontmost
window at screen points); if the machine is occupied, Phase 4 WAITS rather than substituting
mechanism-only verification, because live driving IS this ticket's subject.

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `focus` runs against a RESTORED workspace layout (Files panel open + an editor split), the harness shall bring the Marley window frontmost WITHOUT any content interaction — no file opened, no tab created or switched, no pane focus stolen. | Driven: stand up the restored-layout fixture, run `focus`, capture before/after — captures show NO tree file opened, NO new Editor tab. Negative baseline: the OLD (0.5, 0.12) click on the same fixture REPRODUCES the misfire (a tree row opens) — proving the check bites. |
| REQ-002 (REVISED at Inspect — the original premise was empirically false) | WHEN typing into a pane, the canonical flow (activate + `clickat:<pane-fx,fy> "type:echo hi" enter` in ONE shell command) shall execute the command in the terminal — the `clickat:` mouse-DOWN sets gpui keyboard focus. A raise-only `focus` does NOT set keyboard focus (gpui focuses on mouse-down only), so `focus type:…` types nowhere; the README/usage examples were corrected to `clickat:<pane> type:…`. | Driven (2026-07-22): `focus "type:echo REQ002_probe" enter` → capture shows NO echo (the negative — raise doesn't focus); `clickat:<terminal> "type:echo CLICKPROBE" enter` → capture shows the `✓ echo CLICKPROBE` block + `CLICKPROBE` output (the positive). Both captured this Inspect. |
| REQ-003 | The README shall carry a pre-drive CHECKLIST in which activation (`osascript … frontmost`) is issued in the SAME shell command as the drive verbs, and the Usage example shall demonstrate that shape. | README diff review: checklist section present with the rule + its rationale (keys silently vanish when the host terminal regains frontmost between tool-call shells — re-confirmed live 2026-07-21); the Usage block (:40-46 today) shows one compound activation+drive command, replacing the separate-command example at :41. |
| REQ-004 | The README shall state the restored-workspace rule — `focus` is activation-only; pane targeting uses `clickat:` with coordinates read from a FRESH capture of the CURRENT layout — and the stale pane-focus claims shall be corrected. | README diff review + grep: the rule appears in the checklist; no remaining text attributes pane-focusing to `focus` (today :49 "focus the pane", :80 "(`focus` first)"); drive.swift's usage header (:11) matches the new contract. |
| REQ-005 | The harness shall provide a caller-supplied-fraction pane-targeting idiom — `focus clickat:<fx>,<fy>` composition or a dedicated verb per D-OPEN-PANE-VERB — documented in the README. | Driven on the restored-layout fixture: raise + target a terminal-pane fraction → a typed probe renders in THAT pane (capture); README names the idiom in the checklist. |
| REQ-006 | WHEN no Marley window is on screen, `focus` shall fail with stderr + exit 2; WHILE Accessibility is not granted, any invocation containing `focus` shall still fail loud with exit 3 (the preflight counts `focus` as event-posting regardless of the chosen raise mechanism). | Negative smoke: run `focus` with no Marley instance → stderr + exit 2. AX arm: review pins `focus` outside the `readOnly` set in the new code (:195-198 shape) — revoking the live session's own AX grant to smoke it would break the session's driving, so this arm is review-verified, honestly stated. |
| REQ-007 | Every verb other than `focus` (`find`, `check`, `clickat:`, `rightclickat:`, `scrollat:`, `drag:`, `cmdclick:`, `cmdshiftclick:`, `cmd:`, `cmdshift:`, `alt:`, `ctrl:`, `cmdopt:`, `type:`, `wait:`, `dblclickat:`, `clearmods`, bare keys) shall be behavior-identical. | Diff review: the drive.swift delta confined to the `focus` arm + the usage header (+ any D-OPEN-PANE-VERB addition); one representative driven smoke — `clickat:0.127,0.018` on boot-default still spawns a second terminal (the proven affordance, README:63-66). |

## Floors (constitution)
No `.rs` → no cov/MSI surface. drive.swift is Swift, OUTSIDE gate:11's `scripts/*.sh` shellcheck
glob — the binding checks are: the interpreter run itself (`swift scripts/selftest/drive.swift
check` → `AX_TRUSTED` doubles as the parse/exec proof), the REQ-001..007 driven evidence +
captures, gate:10 gitleaks, gate:14 docs, and the FULL/DIFF gate green end-to-end. §0
anti-circumvention: nothing here touches a gate verdict; the harness change may not be used to
weaken any capture-validated claim (it STRENGTHENS them — a misfiring `focus` was silently
corrupting driven evidence).

## Phase Plan
- **P2 Design** — run probes P-A (programmatic raise vs the bare-binary lane) and P-B
  (capture-verify top-center inertness on both layouts); settle D-OPEN-RAISE + D-OPEN-PANE-VERB;
  design the restored-layout fixture recipe (driven-into-place vs seeded via the shipped
  session-persistence path — deterministic either way); exact README section layout + checklist
  wording; the REQ-001 negative-baseline procedure (how the old behavior is reproduced once the
  arm is rewritten — e.g. `clickat:0.5,0.12` IS the old behavior verbatim, no code stash needed).
- **P3 Implement** — drive.swift: the `focus` arm + usage-header line (+ the pane-targeting verb
  only if Phase 2 adopted it); README: checklist section, Usage example fix, the :49/:80
  corrections. Nothing else.
- **P3.5 Inspect** — adversarial: is there ANY layout/window state where the chosen mechanism
  still content-clicks (tiny window where 0.02·h rounds into chrome-adjacent content? stale
  multi-instance windows, README:85? off-main-display coordinates)? does raise-without-click leave
  keyboard focus somewhere that breaks REQ-002's typed flow? does the new checklist contradict
  any README passage left unedited? exit-code contract intact (D5)?
- **P4 Validate** — RUN the REQ-001..007 evidence for real in a safe input window (screen
  unlocked, machine unoccupied — WAIT if blocked, per the header note): the restored-layout
  no-interaction proof + its negative baseline, the boot-default canonical flow, the
  pane-targeting idiom, the exit-2 smoke, the representative `clickat:` smoke; archive the
  captures; gate `--diff` green.
- **P5 Complete** — CHANGELOG (harness); archive the pair to completed/; AAR (lesson candidates:
  a fixed-coordinate verb encodes a layout assumption — audit other fixed fractions;
  checklists beat buried gotchas — the same-command rule was ALREADY documented at README:78-80
  and still cost a QA session because it was buried); close forge #383.
