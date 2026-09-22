---
pipeline_id: 15c82a56-4cb9-4d29-9994-aeff98db9c10
ticket: forge#401 (4df45f2f-4975-4616-820d-055431588935) · local docs/planning/tickets/open/TICKET-401-inlay-served-check-exclusive-end.md
aar_id: be7a238a-465d-4333-9264-309f57a1b54c
status: Phase 5 — Complete PASS
title: Inlay served-check never passes with the file bottom on screen — store the exclusive end (#352 F5)
type: bug
milestone: M28 follow-up
references: ["#331 (the inlay cache)", "#352 inspect ledger F5", "BF-claude-fold-projection-parse-on-pump-tick-001"]
---

## Title
The inlay served-check compares the genuinely-exclusive viewport end (`end_row`) against a cached
`Range` whose `.end` holds an INCLUSIVE row (`want_last`, clamped to `len_lines - 1`). With the
file's last line on screen, `end_row == len_lines` vs `rows.end == len_lines - 1` — the check can
never pass, so a short/fully-visible file re-sends the byte-identical inlay request once per LSP
round-trip, forever (the in-flight guard bounds cadence to one per timeout window; `inlay_request`
clears on apply and the pump re-asks). Waste only — hints render correctly. This ticket stores the
exclusive end (`want_last + 1`) at the apply site so the cached range is honest and the check
passes.

## Scope
### In
- `apply_inlay_response` (app.rs:4921): store `key.first_row..key.last_row.saturating_add(1)` —
  the one-line fix. The cached `Range` becomes true-exclusive; the served-check at :4799 is
  untouched and becomes correct.
- The pinning unit (Phase 4): file shorter than the viewport, one applied response, subsequent
  refresh ticks send NO second request — driven through the REAL `refresh_inlay_hints` request
  path headless. Enabling `#[cfg(test)]` hooks: `LspHost::set_ready_for_test` (replays
  SpawnOk → InitializeResult through the real `Lifecycle`) + two trivial app accessors
  (`inlay_request_key_for_test`, `inlay_cached_rows_for_test`).

### Out (explicitly deferred)
- The WIRE request shape (`inlay_hint_params` end = `{line: want_last, character: 0}` — #331's
  pinned inclusive-lines shape). Under a strictly-spec server that end position excludes hints on
  `want_last` past col 0; rust-analyzer accepts range-touching hints so it doesn't bite today.
  Re-asking with the SAME params can never yield more, so the cache honesty fix is independent.
  Inspect sharpened the residual: on a strict server the one affected viewport is `end_row ==
  rows.end` exactly (bottom row = the fetch's boundary row) — one row's hints absent at one scroll
  offset, self-healing on any movement/edit. Candidate follow-up: wire end `{line: want_last + 1,
  character: 0}` (servers clamp past-EOF positions).
- `refresh_inlay_hints`' mint math (`want_first`/`want_last` stay inclusive — the wire uses them).
- Any render change (the range's only reader is the served-check; render at :4139 ignores it).

## Reference (§20)
N/A — Marley-specific bug fix. The inlay request/cache/served-check architecture is #331's own
(Marley-designed); no Warp/Zed behavior is being matched. The only external contract touched is
the LSP `textDocument/inlayHint` request, whose wire shape is explicitly out of scope (unchanged,
pinned by #331's `inlay_hint_params_shape` unit).

### Prior art
1. Behavior maps: none apply — this is an internal cache-coherence bug (docs/warp_architecture has
   no inlay-cadence analog; observed captures are about visible behavior, and this bug is invisible).
2. Published material: LSP 3.17 `textDocument/inlayHint` — a `Range` end is an exclusive Position;
   the spec's own convention agrees with storing the exclusive end. No protocol change needed.
3. Our permissive deps: no crate owns this seam — the cache is app-local state
   (`RootView.inlay_hints`), the range is `std::ops::Range<usize>`. Checked: ropey (line counts —
   already used), gpui (no LSP layer), `lsp-types` is not a dep (Marley hand-rolls params in
   marley_lsp — #308's decision). The `Lifecycle` machine (marley_lsp, ours) already provides the
   public `on_event` path the new test hook replays — adoption of our own tested seam, no new
   machinery.

## React-first (parity)
N/A — no UI delta: the fix changes LSP request cadence (stops redundant identical re-requests);
hints already render correctly and continue to render identically. Nothing the user sees changes.

## Locked-In Decisions
- **D1 — Fix at the STORE, not the compare.** `apply_inlay_response` stores
  `key.first_row..key.last_row.saturating_add(1)`. The ticket allows either form; storing the
  exclusive end makes the `Range` mean what `Range` means everywhere in Rust — the compare side
  (`end_row <= rows.end`) is already exclusive-correct. `saturating_add` matches the local idiom
  (`:4789`); overflow is unreachable (`want_last ≤ len_lines - 1`) but the saturating form costs
  nothing and forecloses the debate.
- **D2 — `want_first`/`want_last` stay inclusive.** They feed the wire params (#331's pinned
  shape). Only the CACHED range changes meaning. No rename (the `last_row` field on `InlayKey` is
  wire-facing identity; renaming it would churn #331's tests for zero behavior).
- **D3 — The pinning test drives the REAL request path headless.** `set_caps_for_test` +
  new `set_ready_for_test` (SpawnOk → InitializeResult via the real `Lifecycle::on_event` — no
  bypass constructor) + `seed_editor_geom_for_test` (exists) make `refresh_inlay_hints` actually
  mint and send on a process-less host (`send_body` tolerates the missing child). Observable:
  `inlay_request` is re-set on a re-ask and stays `None` when served (apply clears it at :4865).
- **D4 — Test-only surface additions are `#[cfg(test)]`.** `set_ready_for_test` (lsp_host.rs);
  `ready_inlay_host_for_test` + `inlay_request_key_for_test` + `inlay_cached_rows_for_test`
  (app.rs — the first is the app-level wrapper the host-level hooks need because
  `RootView.lsp_hosts` is module-private to app.rs, the `set_signature_caps_for_test` precedent).
  No production API change.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN an inlay response for key `{first_row, last_row}` is applied, the system shall cache the row range with an EXCLUSIVE end — `first_row..(last_row + 1)`. | Unit: apply a response for key `{0, len-1}` on a short file; `inlay_cached_rows_for_test()` == `0..len`. |
| REQ-002 | WHEN a file shorter than the viewport is fully visible and its inlay response has been applied, subsequent refresh ticks shall send NO further request (the cache serves the viewport). | Unit (headless, real request path): Ready+caps host, seeded geom covering EOF; tick 1 mints+sends (request key set), apply clears it; tick 2 leaves `inlay_request_key_for_test()` == None and the cache intact. |
| REQ-003 | WHEN the viewport still fits inside the fetched range (same file+version), the served-check shall keep passing exactly as before the fix for non-EOF viewports (no cadence regression in the common case). | Same unit, algebra: the stored end grows by exactly 1; `end_row <= rows.end` passes for every `end_row` the old code passed, plus `end_row == len_lines`. Covered by REQ-002's tick-2 pass + existing #331 suite green. |

## Phase Plan
- **P2 Design** — file manifest (3 files) + regression test plan; confirm no other reader of the
  cached range (done in plan discovery: single reader :4799, single writer :4921, render ignores).
- **P3 Implement** — the one-line store fix + the three `#[cfg(test)]` hooks. React-first: N/A.
- **P3.5 Inspect** — 2 critics (small change): correctness (range algebra, guard interplay,
  overflow) + reuse/tests (hook idiom vs existing `_for_test` cluster, no logic forked).
- **P4 Validate** — write + RUN the REQ units; full workspace suite; `scripts/gates.sh` green.
- **P5 Complete** — CHANGELOG, architecture-doc touch, AAR capture, close #401, archive.
