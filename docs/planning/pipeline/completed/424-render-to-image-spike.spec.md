---
pipeline_id: 7f2cb239-e9e3-4cf5-84f1-a5af7261eb0f
ticket: docs/planning/tickets/open/TICKET-424-render-to-image-backport-spike.md
status: Phase 5 — Complete PASS
title: Spike — measure the render_to_image backport surface; decide fork vs early-upgrade vs wait
type: spike
milestone: M-unset
references:
  - docs/marley_architecture/marley_visual_harness.md
  - docs/zed_architecture/crates/gpui.md
  - docs/planning/tickets/open/TICKET-271-headless-pixel-capture-gpui.md
---

## Title
TICKET-271 waits on a gpui release shipping `render_to_image` — the offscreen
pixel capture that would retire the headed shim's environmental hazards. The
API exists in the NEWER Zed-tree gpui (Apache-2.0 there too). This spike
MEASURES the cost of getting it early instead of guessing: diff the offscreen
render path against crates.io 0.2.2, size the entanglement, price each
option's policy cost, and empirically probe the load-bearing assumption
(offscreen Metal without a live WindowServer). Deliverable: a DECISION memo —
fork-backport vs early-upgrade vs wait — appended to the ticket. **Timebox
~half a day. No adoption, no workspace dep changes.**

## Scope
### In
- Obtain the Zed-tree `crates/gpui` source (Apache-2.0 — adoption; STAY inside
  that crate: the GPL editor crates around it are off-limits, §20).
- Locate `render_to_image`/`capture_screenshot` + the offscreen render path;
  diff those files against the on-disk crates.io `gpui-0.2.2`; record files
  touched / LOC / unrelated-churn drag.
- Enumerate per-option policy costs: the `deny.toml` sources amendment a
  fork/git-rev needs (`unknown-git = "deny"` today); fork rebase burden;
  early-upgrade's app-side API-migration census (grep our 0.2.2 API usage
  against the newer tree).
- The WindowServer probe: does offscreen Metal work WITHOUT a live GUI session
  on the mini (a Metal device from a non-GUI context)? If NO, "wait" wins
  outright.
- The decision memo appended to TICKET-424 (+ an `AD-…` if durable), naming
  the follow-up ticket if fork/upgrade wins.

### Out (explicitly deferred)
- ANY production adoption: no vendoring into the workspace, no dep bumps, no
  `deny.toml` edits — the spike only PRICES them.
- #271 itself (stays Deliberate; consumes this memo).
- Building the actual backport even if trivially small — that is the follow-up
  ticket's pipeline.

## Reference (§20)
N/A — Marley-specific test-infrastructure research; no reference-app BEHAVIOR
is matched. §20 posture for the reading itself: the Zed REPO is mixed-license —
`crates/gpui` is Apache-2.0 (adoption, explicitly outside the wall; the
harness doc's "Future upgrade" section already cites a prior legitimate read),
while the editor crates around it are GPL and stay unread. The spike touches
ONLY `crates/gpui`.

### Prior art
- **Our own record:** `docs/marley_architecture/marley_visual_harness.md`
  "Future upgrade" (the round-2 Zed finding: `Window::render_to_image() ->
  Result<image::RgbaImage>` in window.rs, exposed as `capture_screenshot`;
  zero license cost) + its M16 #264 correction (0.2.2 ships the INPUT+STATE
  half only; the test platform's `draw(&Scene)` is a no-op — re-verified this
  session: zero `render_to_image` hits in the 0.2.2 registry source).
  `docs/zed_architecture/crates/gpui.md` §9 (Platform/Scene/headless render)
  + §10 (test support) are the behavior maps.
- **Our permissive deps:** gpui-0.2.2 on disk in the cargo registry is the
  diff base; `image` (already shipped via the harness) is the pixel-buffer
  type the API returns.
- **Policy substrate:** `deny.toml` `[sources]` = crates.io-only allowlist
  (`unknown-git = "deny"`) — verified this session; a fork/git-rev needs a
  justified amendment (§0 gate:8).
- **Published:** the gpui crates.io releases page / Zed's gpui README state
  the crate is periodically published from the Zed tree — the "wait" option's
  cadence evidence to check during the spike.

## React-first (parity)
N/A — no UI delta: research spike; nothing ships.

## Locked-In Decisions
- D1 — **Measure, then decide** (the 423 pattern): no option is locked before
  the diff numbers + the WindowServer probe are in hand.
- D2 — **The wall holds during the read**: Zed-repo access is confined to
  `crates/gpui` (Apache-2.0); no GPL crate is opened.
- D3 — **The timebox is real**: ~half a day; if the diff proves unexpectedly
  deep, "the surface is too large to size cheaply" is itself a valid measured
  answer (and argues wait).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the spike completes, the ticket shall carry a decision memo naming ONE of fork-backport / early-upgrade / wait, argued from measured evidence (diff surface numbers, policy costs, probe result). | Memo present in TICKET-424; review. |
| REQ-002 | WHEN the offscreen-render path is diffed, the memo shall state files touched, approximate LOC, and the churn drag between 0.2.2 and the Zed tree for that path. | Numbers in the memo, reproducible from the recorded refs/paths. |
| REQ-003 | WHEN the WindowServer assumption is probed, the memo shall record the empirical result (offscreen Metal with vs without a live GUI session) with the probe's actual output. | Probe receipt in the notes; summarized in the memo. |
| REQ-004 | WHEN the decision is fork or early-upgrade, the memo shall name the follow-up ticket scope; WHEN it is wait, #271 shall stay Deliberate with the evidence attached. | Ticket state after close matches the memo. |

## Phase Plan
- **P2 Design** — the probe designs: where to fetch the Zed-tree gpui (pinned
  ref), the diff method, the Metal-without-WindowServer probe shape, the
  API-census grep set.
- **P3 Implement** — run the research (fetch, diff, census, probe) — no
  workspace code; artifacts in the scratchpad + notes.
- **P3.5 Inspect** — adversarial review of the memo's claims vs the evidence.
- **P4 Validate** — gate `--fast`/`--diff` (docs-only change); the probe
  receipts are the tests (§7 gate-is-test posture).
- **P5 Complete** — memo into the ticket, AD if durable, close + archive.
