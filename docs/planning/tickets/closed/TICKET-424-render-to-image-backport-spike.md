# TICKET-424 — spike: measure the `render_to_image` backport surface (fork vs early-upgrade vs wait)

- **Ticket:** LOCAL #424 (spike, M-unset)
- **Tags:** testing, gpui, headless-pixels, 271-informer, timeboxed
- **Created:** 2026-08-14
- **Provenance:** minted from the #271 "can we patch it ourselves?" discussion (Chad decision,
  post-423 batch, 2026-08-14)
- **Status:** closed (2026-08-14 — spike complete; decision memo below)

## DECISION MEMO — 2026-08-14 (pipeline `424-render-to-image-spike`)

**Decision: WAIT — with a watch and pre-paid de-risking.** Neither
get-it-now path survives its measured cost against a payoff nothing currently
blocks on.

**The measured evidence** (full receipts in the archived pipeline notes;
Zed tree pinned at `a21007b7a948e46afbe719150f5e9968bfcd1078`):
1. **The backport is not a cherry-pick.** The Zed tree SPLIT gpui into a
   crate family (`gpui` core + `gpui_macos`/`gpui_linux`/`gpui_platform`/
   `gpui_wgpu`/…); crates.io 0.2.2 is the pre-split monolith. The offscreen
   mechanism spans: the `PlatformHeadlessRenderer` trait +
   `Window::render_to_image` + `capture_screenshot` on TWO new contexts
   (`HeadlessAppContext`, 284 lines; `VisualTestAppContext`, 484) +
   `VisualTestPlatform` (264) + TestWindow/TestPlatform factory plumbing +
   `MetalHeadlessRenderer` inside a renderer whose whole-file diff vs 0.2.2
   is 1,411 lines. Honest adaptation estimate: **~1,300–1,500 lines across
   ≥8 files spanning two crate layouts**, plus a `deny.toml` sources
   amendment and a fork rebased forever.
2. **Early-upgrade = adopting an unpublished re-architecture.** A git-rev
   dependency FAMILY (≥4 crates, none on crates.io), the same deny.toml
   amendment, and compile-drift risk (census: all load-bearing symbols
   survive at the same homes; signatures unverified — estimate band).
   Riding Zed HEAD means absorbing the family migration NOW and again at
   the eventual publish.
3. **Wait has no known cadence** — 0.2.2 (2025-10-22) is still newest;
   ~10 months of publish silence. Wait is indefinite, hence the WATCH.
4. **The payoff is real and BIGGER than #271 assumed — and now proven.**
   Three probes, each run in the GUI session AND over ssh (non-Aqua, no
   WindowServer), all byte-identical: offscreen Metal render+readback
   (`PROBE OK … [191,128,64,255]`), CoreText enumeration (`180 families;
   Menlo/Monaco/Helvetica=true`), glyph rasterization (`glyph=80
   litPixels=261`). With `HeadlessAppContext` + injected `MacTextSystem` +
   `current_headless_renderer`, a TRUE-headless pixel lane works — and the
   #344/#361 font-policy drives (#365's headed-only lane) could run
   headless under it. The environmental risk of the eventual adoption is
   ~zero; the cost is purely the migration.

**Why wait wins:** the headed lane works today (#365 shipped on it the same
day); there is no CI and no consumer blocked on headless pixels; both
get-it-now paths pay a four-figure-line, fork-or-family price for a
capability with no current demand — and the split strongly suggests the
NEXT release IS the family shape, so adopting now means migrating twice.

**The watch + the follow-up (mint when it fires):** check crates.io gpui
releases (`curl -s https://crates.io/api/v1/crates/gpui | jq
.crate.max_version` — a cheap idle-machine/periodic check). When a release
newer than 0.2.2 lands, mint the ADOPTION ticket: upgrade the family, wire
`HeadlessAppContext` via `with_platform` (NOT `::new` — it installs no
renderer) with `Arc::new(MacTextSystem::new())` (NOT the doc example's
CosmicTextSystem, NOT plain `TestPlatform::new`'s Noop) + `font-kit` +
`test-support` features, port gate:15's screenshot half off the
`screencapture` shim, and retire the frontmost/shadow-offset hazards
(#271's list). **The `VisualTestAppContext`/`VisualTestPlatform` twin is
in-session-only** (real MacPlatform: pasteboards at construction, real
`show: true` NSWindow, AppKit main-thread SIGABRTs) — never route the
headless lane through it.

**Consequence for #271:** stays Deliberate; its ticket now points here for
the adoption recipe + traps. #365's headless upside is recorded there too.

## Summary

TICKET-271 waits for a gpui release shipping `render_to_image` — the offscreen pixel capture
that would retire the headed `screencapture`/osascript shim's environmental hazards. The API
already exists in the NEWER Zed-tree gpui at zero license cost (gpui is Apache-2.0 there too —
adoption, §20-clean; the harness doc's "Future upgrade" section + `docs/zed_architecture/crates/
gpui.md` §9–10 are the maps). What we do NOT know is the cost of getting it early: 0.2.2's test
platform has NO renderer at all (`draw(&Scene)` is a no-op), so a backport means pulling the
offscreen-target path of the Metal renderer + `window.rs` plumbing across whatever churn
separates crates.io 0.2.2 from Zed HEAD. This spike MEASURES that surface instead of guessing.

**Timebox: ~half a day.** Deliverable is a DECISION, not an adoption.

## The work

1. Diff base: the on-disk crates.io source
   (`~/.cargo/registry/src/*/gpui-0.2.2/`) — copy aside for diffing, do not vendor into the
   workspace yet.
2. Obtain the Zed-tree gpui crate (Apache-2.0 — adoption is explicitly outside the §20 wall;
   stay INSIDE `crates/gpui` — the editor crates around it are GPL and off-limits).
3. Locate `render_to_image` / `capture_screenshot` + the offscreen render path (window.rs, the
   Metal renderer, the test-support plumbing). Diff those files against 0.2.2 and SIZE the
   entanglement: files touched, LOC, how much unrelated API churn the path drags in.
4. Enumerate the policy cost per option: the `deny.toml` sources amendment
   (`unknown-git = "deny"` today) a fork/git-rev needs; the fork-maintenance surface; for
   early-upgrade, a rough app-side API-migration census (grep the 0.2.2 APIs Marley uses
   against the newer tree).
5. Sanity-probe the assumption that offscreen Metal works WITHOUT a live WindowServer session
   (a Metal device without a GUI session on the mini) — the whole point is headless pixels; if
   that's false, "wait" wins outright.

## Acceptance

A decision memo appended to THIS ticket (and an `AD-…` if the decision is durable):
**fork-backport vs early-upgrade vs wait**, argued from the measured surface (the file/LOC/churn
numbers, the policy costs, the WindowServer probe result) — the 423 pattern: probe the
mechanism, then decide. If fork/upgrade wins, the memo names the follow-up pipeline ticket to
mint; if wait wins, #271 stays Deliberate and this closes with the evidence on record. No
production adoption, no workspace dep changes inside the spike.
