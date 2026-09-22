---
pipeline_id: f04aac53-fe51-4867-8a5b-0adecce0763c
ticket: docs/planning/tickets/open/TICKET-365-headed-font-policy-verification.md
status: Phase 5 — Complete PASS
title: Headed font-policy verification — the #344/#361 resolvable paths run end-to-end under real CoreText
type: chore
milestone: M22
references:
  - docs/planning/pipeline/completed/344-font-family-resolve-warning.notes.md
  - docs/planning/pipeline/completed/361-font-family-monospace-warn.notes.md
  - docs/planning/pipeline/completed/264-headless-drive.notes.md
  - docs/planning/tickets/open/TICKET-271-headless-pixel-capture-gpui.md
---

## Title
Close the app-wiring verification gap #344/#361 left open: the RESOLVABLE font
boot paths (a real family that applies, a real proportional family that falls
back with a "not monospace" flash) are unprovable under `#[gpui::test]` —
gpui's NoopTextSystem resolves no family and fakes equal-width advances — so
today they ride on pure-unit proof of the decision plus trust in two
`mutants::skip` shims (`font_resolves`, `font_is_monospace`). This pipeline
adds HEADED drives that boot the real app under real CoreText on a live screen
and assert the end-to-end behavior via the existing `*_for_test` observables.

## Scope
### In
- Headed end-to-end drives for the three ticket directions (design may collapse
  the two mono-silent directions into parameterized drives):
  1. resolvable MONO family applies silently (`mono_family` == the family, no
     flash) — the #344 RESOLVABLE-APPLY gap AND #361's no-false-warn direction;
  2. resolvable PROPORTIONAL family falls back to the built-in mono + a
     "not monospace" flash naming the family — the #361 warn direction.
- Wiring the drives into the existing headed lane (gate:15 / selftest
  conventions: conditional on a live GUI session, fail-closed there, skipped
  cleanly elsewhere) + the lane doc updated so the drives are runnable by name.
- Fonts are macOS-system-guaranteed only (e.g. Monaco / Helvetica — present in
  `all_font_names()` on any Mac; never a user-installed family like Fira Code).

### Out (explicitly deferred)
- #271 `render_to_image` pixel capture — still waits on a gpui upgrade; these
  drives assert state observables, not pixels.
- #417 activation/after-capture battery — activation-dependent AX work stays
  its own ticket.
- CI automation of the headed lane — it remains the conditional local lane
  gate:15 defines; this pipeline makes the drives EXIST and RUN here, not
  auto-run everywhere.
- Production behavior changes — #344/#361 policy is shipped and correct; this
  is verification. (A missing minimal test hook, if design finds one, is the
  only permitted app-code delta, following the existing cov-excluded shim
  conventions.)

## Reference (§20)
N/A — Marley's own settings/font policy verification; no Warp/Zed behavior is
being matched (the policy behavior itself was set + referenced in #344/#361).
No Warp/Zed source read.

### Prior art
- **Our permissive deps (gpui 0.2.2, read in registry — ADOPTION):**
  `NoopTextSystem` is `pub(crate)` (gpui `src/platform.rs:594`) and hardwired
  into the test platform (`src/platform/test/platform.rs:105`) — a
  `#[gpui::test]` can NEVER see real fonts, confirming headed-or-nothing for
  these paths. The mac platform binds the real CoreText-backed text system at
  construction (`src/platform/mac/platform.rs:196`). gpui 0.2.2 ships no
  `render_to_image`/`draw_to_image` (checked: zero hits in `src/`) — #271's
  deferral stands; state-observable asserts are the available idiom.
- **Our own code:** the #337/#344 TempDir persist-settings→boot→assert drive
  idiom + the `mono_family_for_test()` / `flash_message_for_test()` observables
  (added for exactly these asserts); the existing headed lane exemplars
  (headed_panes / headed_shell) and `scripts/selftest/` runner conventions;
  `docs/marley_architecture/marley_visual_harness.md` (lane doc).
- **Behavior maps / published:** N/A — test-lane chore; no reference-app
  behavior. The "probe advance widths to classify monospace" heuristic and its
  gpui seam were already swept + adopted at #361 (its spec's Prior art).

## React-first (parity)
N/A — no UI delta: the drives verify already-shipped #344/#361 boot behavior
(font application + existing flash surface); nothing the user sees changes.

## Locked-In Decisions
- D1 — **System-guaranteed fonts only.** Monaco (mono, ≠ the built-in Menlo, so
  applied-vs-fallen-back is distinguishable by family alone) and Helvetica
  (proportional) — both ship with macOS and appear in `all_font_names()` on
  any Mac. Never a user-installed family.
- D2 *(amended at Design — plan assumed the `*_for_test` accessors reach the
  headed child; they are `#[cfg(test)]` and don't exist in the spawned binary,
  and the Metal-painted flash is invisible to AX)* — **the app self-reports**:
  a runtime env-gated hook (`MARLEY_FONT_POLICY_SELFTEST=1`, the
  `MARLEY_WEBVIEW_PROBE` precedent) prints one post-boot line
  (`family=… flash=…`) and quits; the DRIVE asserts on that line. Still state,
  not pixels — the contract is family + flash content, which pixels only proxy.
- D3 — **No production code changes** beyond the minimal hook above, following
  the existing cov-excluded + `mutants::skip` shim conventions; the shipped
  policy is not touched and the hook is inert without the env var.
- D4 *(confirmed at Design)* — **Lane mechanics = the existing convention**:
  a new `#[ignore]`-gated target `crates/marley_app/tests/headed_fonts.rs`
  (standard headed reason string), invoked
  `cargo test -p marley --test headed_fonts -- --ignored`; settings seeded via
  a tempdir `$HOME` (config resolves `home_dir()/.marley/config`); spawn is
  deadline-guarded (kill + clear message on ghost-boot). Needs only a live
  WindowServer session — no Accessibility, no Screen Recording, no activation.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the app boots HEADED (real CoreText) with `appearance.font_family = "Monaco"`, the system shall apply it — `mono_family` reads `"Monaco"` and no status flash is set. | Headed drive RUN at P4 on this machine (live GUI session); assert failure reddens the lane. |
| REQ-002 | WHEN the app boots HEADED with `appearance.font_family = "Helvetica"` (resolvable, proportional), the system shall fall back to the built-in mono family AND set a flash containing "Helvetica" and "not monospace". | Headed drive RUN at P4; the flash-content assert pins the #361 warn arm. |
| REQ-003 | WHEN a `font_is_monospace`-always-false regression is injected (negative smoke), the REQ-001 drive shall FAIL (the false-warn is observable end-to-end). | One-off sabotage smoke at P4: flip the probe to `false`, run the REQ-001 drive, observe RED, revert, observe GREEN. |
| REQ-004 | WHEN the headed lane cannot run (no live GUI session / lane conditions unmet), the drives shall skip with a stated reason rather than false-green or hang. | Run the lane's skip path (condition unmet simulation or documented conditional mechanism) + review. |

## Phase Plan
- **P2 Design** — confirm lane entry point + drive location/naming, the settings
  seed mechanism under the headed boot (config-dir override), the skip
  condition, and whether REQ-001's two mono directions need distinct fonts.
- **P3 Implement** — the drives + lane wiring (test code; no app-code delta
  expected per D3).
- **P3.5 Inspect** — independent critics vs the diff; fix the real findings.
- **P4 Validate** — RUN the headed drives on this machine + the REQ-003
  sabotage smoke + the full static gate; document lane receipts in notes.
- **P5 Complete** — archive, ledger capture (§19), close the ticket (its
  BACKLOG row was removed at promotion).
