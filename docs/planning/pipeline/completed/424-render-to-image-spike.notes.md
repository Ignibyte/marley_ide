# 424 render_to_image spike — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-424-render-to-image-backport-spike.md
- **Pipeline spec:** 424-render-to-image-spike.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** `/goal` (Chad 2026-08-14): 407 then **424 auto approved** —
  autonomous through `/commit`. Queue-top promotion (Queue empties again).
- **Classification / tier:** spike, one slice = the decision memo (fork vs
  early-upgrade vs wait for `render_to_image`). Timebox ~half a day. NOTHING
  ships to the workspace.
- **Recall (§18.3):** the ticket was minted THIS session from the #271
  discussion — the recall IS fresh session context: 0.2.2's test platform
  has no renderer (`draw(&Scene)` no-op; zero `render_to_image` hits,
  re-verified in the registry source); the harness doc's Future-upgrade
  section documents the Zed-tree API (`Window::render_to_image() ->
  Result<image::RgbaImage>`, `capture_screenshot` on the test contexts);
  `deny.toml` sources = crates.io-only (`unknown-git = "deny"`);
  `docs/zed_architecture/crates/gpui.md` §9–10 are the maps. §20 posture:
  Zed-repo `crates/gpui` is Apache-2.0 (adoption); the GPL editor crates
  stay closed.
- **Decisions:** spec D1–D3 (measure-then-decide; the wall holds; the
  timebox is real — "too deep to size cheaply" is a valid answer). EARS
  REQ-001..004 (memo, diff numbers, the WindowServer probe receipt, the
  follow-up/stay-Deliberate consequence).

## Phase 2 — Design

### Design-time fact (the cheapest probe first, the 423 pattern)
**crates.io gpui: 0.2.2 (2025-10-22) is STILL the latest** — the 0.2.x burst
(0.2.0→0.2.2, Oct 2025) then ~10 months of silence (receipt:
`scratchpad/gpui-versions.json`). Consequence: "wait" has NO known release
cadence — it is indefinite, not scheduled. The decision weighs a real cost
against that.

### Probe designs (Reference §20 re-confirmed N/A; the wall is enforced
### MECHANICALLY at fetch — only `crates/gpui` ever lands on disk)
1. **Fetch (pinned):** shallow clone `zed-industries/zed` `--depth 1
   --filter=blob:none --sparse` into the SCRATCHPAD (never the repo), then
   `git sparse-checkout set crates/gpui` — the GPL crates are never
   materialized. Record the resolved HEAD sha in the notes (the memo's
   reproducibility ref). Verify `crates/gpui/Cargo.toml` license =
   Apache-2.0 before reading anything else.
2. **Locate + diff the offscreen path:** find `render_to_image` /
   `capture_screenshot` definitions + every file the path touches (expected:
   `window.rs`, the mac Metal renderer, `platform/test/*`, possibly
   scene/atlas plumbing). For that file SET: `diff -u` vs the registry
   `gpui-0.2.2`, record per-file changed-LOC + a same/moved/rewritten
   verdict — the "churn drag" number the memo needs (REQ-002). Timebox rule:
   if the path drags in a rewritten renderer core, record THAT as the
   answer, not a full accounting (D3).
3. **API census (early-upgrade's price):** `grep -rhoE
   'gpui::[A-Za-z_]+' crates/ | sort | uniq -c | sort -rn` (+ the
   `use gpui::{...}` forms) → the symbol families Marley actually uses;
   spot-check the top ~20 against the Zed-tree gpui's pub surface
   (present/renamed/gone). Output: N symbols, M drifted — the migration-size
   estimate.
4. **The WindowServer probe (REQ-003, the decider):** ONE Swift file —
   `MTLCreateSystemDefaultDevice()` → command queue → 64×64 offscreen
   `MTLTexture` (renderTarget) → render pass clearing to a known color →
   `getBytes` readback → print `PROBE OK <pixel>` / the failure. Run twice:
   (a) inside the GUI session (baseline), (b) over `ssh localhost` — a
   non-Aqua login context, the same class the #318 forensics showed has no
   WindowServer access. Caveat recorded: ssh-localhost approximates
   "headless mini" (display asleep/no console user may differ); it IS the
   context a future headless-lane runner would actually use.
5. **deny.toml pricing:** no probe needed — the policy is on file
   (crates.io-only allowlist); the memo prices the amendment text + the
   fork's rebase surface (= the diff numbers from probe 2).

### File manifest
| Artifact | What |
|---|---|
| `scratchpad/zed-sparse/` | the pinned sparse clone (crates/gpui only) — spike-local, never committed |
| `scratchpad/metal-offscreen-probe.swift` | the REQ-003 probe |
| `scratchpad/424-*.txt/json` | diff stats, census output, probe receipts |
| `docs/planning/pipeline/active/424-*.notes.md` | phase entries + receipts |
| `docs/planning/tickets/open/TICKET-424-*.md` | the decision memo (at P5) |
- NO `crates/**` files, no Cargo/deny edits (spec Out).

### Test plan (gate-is-test posture: the probes ARE the evidence)
| REQ | Verification |
|---|---|
| REQ-001 | memo present, one decision, argued from the numbers below |
| REQ-002 | probe-2 diff table in notes (files/LOC/verdicts) + the pinned sha |
| REQ-003 | probe-4 receipts: both contexts' literal output in notes |
| REQ-004 | ticket/backlog state after P5 matches the memo's decision |
- Plus gate `--diff` green at commit (docs-only changeset expected).

### Risks
- R1 — ssh-localhost ≠ display-asleep exactly (recorded caveat; it matches
  the future runner's actual context, which is what matters).
- R2 — Zed HEAD churn makes the diff a moving target: pinned sha + D3's
  "too deep is an answer".
- R3 — census spot-check is a sample, not a compiler run — the memo prices
  migration as an ESTIMATE band, stated as such.

## Phase 3 — Implement
- **React-first: N/A** (research spike; nothing ships).
- **Fetch (probe 1):** `zed-industries/zed` shallow+sparse at
  **`a21007b7a948e46afbe719150f5e9968bfcd1078`**; materialized ONLY
  `crates/gpui` (7.3 MB), later + `crates/gpui_macos` + `crates/gpui_platform`
  — each license-verified `Apache-2.0` BEFORE reading. No GPL crate ever on
  disk (the wall held mechanically).
- **The offscreen path (probe 2) — the REQ-002 numbers:**
  - **Structural headline: gpui is no longer one crate.** The tree split it
    into a family — `gpui` core + `gpui_macos` / `gpui_linux` /
    `gpui_windows` / `gpui_platform` / `gpui_wgpu` / `gpui_web` /
    `gpui_util` / `gpui_shared_string` / `gpui_tokio` / `gpui_macros`.
    crates.io 0.2.2 is the pre-split MONOLITH — any backport spans TWO crate
    layouts, not two versions of one file set.
  - The mechanism (all Apache-side): `PlatformHeadlessRenderer` trait
    (`gpui/src/platform.rs:977`) + `Window::render_to_image`
    (`window.rs:2416`) + `capture_screenshot` on BOTH
    `VisualTestAppContext` (`app/visual_test_context.rs:384` — a NEW type,
    distinct from the classic `VisualTestContext` still in test_context.rs;
    corrected at inspect) and the NEW
    `HeadlessAppContext` (`app/headless_app_context.rs:168`) + TestWindow/
    TestPlatform renderer-factory plumbing + `VisualTestPlatform`
    (`platform/visual_test.rs` — REAL Metal render + TestDispatcher) + the
    Metal implementor `MetalHeadlessRenderer`
    (`gpui_macos/src/metal_renderer.rs:1610`, `new_headless` :184, reusable
    offscreen target :140, `render_scene_to_image` :569).
  - Size: the three new context files alone = **1,032 lines with NO 0.2.2
    counterpart** (headless_app_context 284 + visual_test 264 +
    visual_test_context 484); the Metal renderer is 1,626 lines vs 0.2.2's
    1,355 with a **1,411-line whole-file diff** (the base drifted too — a
    cherry-pick does NOT apply; it is a hand-adaptation). Honest backport
    estimate: **~1,300–1,500 adapted lines across ≥8 files spanning the
    layout split**, plus test-platform plumbing 0.2.2 lacks entirely.
- **The WindowServer probe (probe 4) — REQ-003, BOTH LEGS PASS:**
  - Context A (GUI session): `PROBE OK device=Apple M4 pixel(bgra)=[191,
    128, 64, 255]` — exact clear-color readback, no window, no CAMetalLayer.
  - Context B (`ssh localhost`, non-Aqua): **identical** `PROBE OK … [191,
    128, 64, 255]`. Offscreen Metal render + readback needs NO WindowServer
    access. Mechanics: a temporary loopback authorized_keys entry (the
    user's own existing key), REMOVED immediately after — file restored to
    its prior 2 lines. Caveat (R1) stands: ssh approximates
    display-asleep; it IS the context a headless runner would actually use.
  - Probe source: `scratchpad/metal-offscreen-probe.swift` (device → queue →
    64×64 renderTarget texture → clear pass → getBytes; the exact shape of
    gpui's `render_scene_to_image`).
- **API census (probe 3):** 46 unique `gpui::`-token families across
  crates/ (dominated by `test`/`hsla`/`WindowHandle`/`Div`); spot-check of
  the load-bearing set (VisualTestContext, TestAppContext, WindowHandle,
  Application, `all_font_names`/`advance`/`resolve_font` — the #344/#361
  probes) — **all present at the same homes** in the new tree. Early-upgrade
  cost is therefore dominated by the DEPENDENCY SHAPE (a git-rev family of
  ~4+ crates, none published) + compile-surfaced signature drift (an
  estimate band, R3 — no compiler run in the spike).
- **Deviations:** none from the design; census undercount acknowledged (grep
  form misses method-call-only usage — stated as an order-of-magnitude, not
  a count).

## Phase 3.5 — Inspect
- **Critic (sufficiency lens, Explore over the pinned checkout) — the
  memo-shaping findings:**
  | Sev | Finding | Verdict | Disposition |
  |---|---|---|---|
  | HIGH (for the memo) | The ssh probe is necessary-but-not-sufficient AS STATED — but the gap decomposes: for **`HeadlessAppContext`** (TestPlatform + TestDispatcher + synthetic displays + TestWindow + injected Metal headless renderer, `headless_app_context.rs:65-126`) the probe is ~the exact code path (NO NSApplication/NSWindow/pasteboard/display-link anywhere); the uncovered services were CoreText font ENUMERATION (SystemSource/CTFontManager → fontd/XPC) and glyph RASTERIZATION (CGBitmapContext — the atlas path) in an injected `MacTextSystem` | REAL | TWO follow-up probes, BOTH contexts each: enumeration `PROBE OK 180 families; Menlo/Monaco/Helvetica=true` and rasterization `PROBE OK glyph=80 litPixels=261` — byte-identical over ssh. Every named unknown is CLOSED; the headless-lane claim holds end-to-end for HeadlessAppContext with injected CoreText. |
  | HIGH (for the memo) | The claim is FALSE for the other new lane: `VisualTestAppContext`/`VisualTestPlatform` wrap the REAL `MacPlatform` (pasteboards at construction, real NSWindow `show: true` at (-10000,-10000), ScreenCaptureKit, AppKit-off-main-thread SIGABRTs — the crate's own tests are `#[ignore]`d for it) | REAL | The memo steers #271 to `HeadlessAppContext`, names the visual-test twin as in-session-only. |
  | MED | Trap set for the consumer: `HeadlessAppContext::new` installs NO renderer (`\|\| None` — `capture_screenshot` dead through it; use `with_platform` + `gpui_platform::current_headless_renderer`); the text system is CALLER-INJECTED (doc example passes `CosmicTextSystem` — copying it silently validates cosmic-text, not macOS font policy; plain `TestPlatform::new` = Noop); `MacTextSystem` needs `font-kit`, the renderer factory needs `test-support` — misconfigure and you silently lose the real stack | REAL | Trap list recorded verbatim in the memo for the follow-up ticket. |
  | — | Lead misattribution: Phase 3 credited `capture_screenshot` to `VisualTestContext`; it is on `VisualTestAppContext` (a distinct new type; the classic one is unchanged in test_context.rs:738) | REAL (self-found) | Phase-3 note corrected in place. |
- **Lead verification:** the critic's context-construction chain spot-checked
  against the source (constructor `\|\| None`, TestPlatform wiring, the
  synthetic display); the census-undercount framing already stated; the
  cadence receipt on file.
- **Transparency note (machine state):** both ssh-context probes used a
  TEMPORARY loopback `authorized_keys` entry (the user's own existing
  `id_ed25519.pub`), appended for the probe and removed immediately after —
  the file was verified back at its prior 2 lines both times. No other
  machine config was touched.
- **Positive side-finding for the ledger:** injected-CoreText
  `HeadlessAppContext` + the enumeration probe result means the #365
  headed-only font drives could someday run HEADLESS under an upgraded gpui
  — recorded in the memo as follow-up upside, not scope.

## Phase 4 — Validate
- **Gate-is-test posture (§7):** docs-only changeset; the probe receipts ARE
  the evidence — all literal outputs recorded at Phase 3/3.5 (REQ-002's diff
  numbers + pinned sha; REQ-003's THREE probes × two contexts each, all
  byte-identical over ssh).
- **Formal runs:** `cargo nextest run --workspace` → **2169/2169 passed, 7
  skipped**; `scripts/gates.sh --diff` → **GATE GREEN [diff], 15/15**
  (docs-only diff — gate:5 took the trivial-pass DIFF arm as designed).
- **Live-app capture: N/A** (research spike; nothing user-visible shipped).
- **Pre-existing, not in scope:** none surfaced.

## Phase 5 — Complete
- **The DECISION MEMO** (REQ-001/004) written into TICKET-424: **WAIT with a
  watch** — the follow-up adoption recipe + traps recorded there; #271
  updated in place (stays Deliberate, now pre-de-risked, pointer to the
  memo).
- **§21 docs:** CHANGELOG `[Unreleased] Added` entry; the harness doc's
  Future-upgrade section gained the spike-verdict callout. React parity: N/A.
- **Ledger appends:**
  `AD-claude-424-the-headless-pixel-stack-is-windowserver-free-and-we-wait-for-the-family-001`.
- **Ticket:** closed (memo IS the close); moved to `tickets/closed/`.
  BACKLOG: the Queue row left at promotion (placeholder row stands).
- **Archive:** pair → `pipeline/completed/`. Spike artifacts stay in the
  session scratchpad (zed-sparse pin, probes, receipts) — reproducible from
  the recorded sha + probe sources if ever needed again.
