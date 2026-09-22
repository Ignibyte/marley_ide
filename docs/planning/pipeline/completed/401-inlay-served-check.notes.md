# Inlay served-check — store the exclusive end (#401) — Notes

- **Forge ticket:** #401 4df45f2f-4975-4616-820d-055431588935
- **AAR:** be7a238a-465d-4333-9264-309f57a1b54c
- **Local ticket doc:** docs/planning/tickets/open/TICKET-401-inlay-served-check-exclusive-end.md
- **Pipeline spec:** 401-inlay-served-check.spec.md

## Phase 1 — Plan
- **Request:** `/work 401 auto approved` — every human gate pre-approved including the final
  commit (same contract as the #400/#384 runs). No sprint (39 closed); solo backlog bug.
- **Classification / tier:** bug, SMALL. One-line production fix + three `#[cfg(test)]` hooks +
  one pinning unit. No UI delta (React-first N/A), no wire change, no new types.
- **Forge recall (§18.3):** bulletins: none. `knowledge-context` (Plan) logged 13 surfacings into
  the AAR; top hits are the #352-adjacent memos (BF-claude-fold-projection-parse-on-pump-tick-001,
  the #352 distilled lessons) — all already embedded in the ticket's own description (it was
  written from #352's inspect ledger F5). No contradicting rule surfaced.
- **Discovery (the precise edit surface):**
  - The bug, confirmed at source: `refresh_inlay_hints` (app.rs:4743, `mutants::skip`) computes
    `end_row` EXCLUSIVE (`proj.buffer_row(geom.last - 1) + 1`, :4786-4790) and `want_last`
    INCLUSIVE (`.min(len_lines - 1)`, :4794-4796). `apply_inlay_response` (:4856, `mutants::skip`)
    stores `key.first_row..key.last_row` (:4921) — inclusive value in the `Range.end` position.
    Served-check :4798-4800: `rows.start <= first_row && end_row <= rows.end`. Full file visible ⇒
    `end_row == len_lines` vs `rows.end == len_lines - 1` ⇒ never served.
  - The cached range has exactly ONE reader (the served-check :4799) and ONE writer (:4921); the
    render read at :4139 destructures the range as `_`. Blast radius of the fix: those two lines'
    interaction only.
  - `inlay_request` clears on apply (:4862-4865) — so "a re-ask happened" is observable as
    `inlay_request` going Some again on the next tick; "served" leaves it None.
  - Wire params (marley_lsp/src/inlay.rs:106): end `{line: want_last, character: 0}` — #331's
    pinned shape (`inlay_hint_params_shape` pins `12, 80`). OUT of scope (spec ## Out).
  - Test enablers already present: `seed_editor_geom_for_test` (headless_drive.rs:8042 idiom —
    headless renders never fill `editor_geom`, probes seed it), `set_caps_for_test`
    (lsp_host.rs:706), `drive_inlay_for_test`/`inlay_hints_for_test`/`inlay_uri_for_test`,
    `push_response_for_test`. Missing: a way to make a process-less host **Ready** — `request()`
    is Ready-gated (:528) but `send_body` tolerates the missing child (:715-719, `handle: None` ⇒
    frame dropped), so Ready is the ONLY blocker to driving the real send path headless. The
    `Lifecycle` machine (marley_lsp, PURE, public API) reaches Ready via
    `on_event(SpawnOk) → on_event(InitializeResult)` — a `#[cfg(test)]` hook can replay that
    through the REAL machine, no bypass constructor.
  - `INLAY_PAGE = 50` (:802) — a short fixture (4 rope lines) clamps `want_last` to
    `len_lines - 1 = 3`; key `{0, 3}`; fixed cache `0..4`; `end_row = 4` ⇒ served.
  - `marley_lsp::lifecycle::Event` is reachable via the public `pub mod lifecycle` path (lib.rs:26)
    — the hook uses the full path, no import churn.
- **Decisions:** D1–D4 locked in the spec (store-side fix, inclusive wire values unchanged,
  real-request-path test via a Ready hook, test-only surface additions).

## Phase 2 — Design

### Architecture / approach
Unchanged — the #331 inlay pipeline stays as-is (pump tick → `refresh_inlay_hints` served-check →
capability gate → `request()`; response → `consume_lsp_responses` → `apply_inlay_response` →
cache). The fix lands at the cache's single write site: the stored `Range<usize>` becomes
true-exclusive (`key.first_row..key.last_row.saturating_add(1)`), which is what the compare side
already assumes. No new types, no module moves, no wire change. §14: no panics added; all new
surface is `#[cfg(test)]`; IO untouched. §20 Reference confirmed N/A (Marley-specific; the LSP
spec's own Range-end-exclusive convention agrees with the fix).

The test enabler is the one genuinely new piece: a process-less `LspHost` can now be driven to
`Phase::Ready` in tests by replaying `SpawnOk → InitializeResult` through the REAL
`marley_lsp::Lifecycle` machine (public `on_event`) — no bypass constructor, no marley_lsp change.
`request()` then genuinely mints ids, registers the pending/purpose entries, and `send_body`
no-ops on the missing child (existing tolerance, :715). This turns the previously-unreachable
"was a request sent?" fact into observable state (`inlay_request`), which REQ-002 pins.

### File manifest
| file | change |
|---|---|
| `crates/marley_app/src/app.rs` | (1) :4921 — store `key.first_row..key.last_row.saturating_add(1)` + `#401` comment. (2) Beside the #331 hook cluster: `#[cfg(test)] ready_inlay_host_for_test(&mut self, root)` (entry-or-create the host, `set_ready_for_test()` + `set_caps_for_test(inlayHintProvider: true)`), `#[cfg(test)] inlay_request_key_for_test(&self) -> Option<InlayKey>`, `#[cfg(test)] inlay_cached_rows_for_test(&self) -> Option<Range<usize>>`. |
| `crates/marley_app/src/lsp_host.rs` | `#[cfg(test)] set_ready_for_test(&mut self)` beside `set_caps_for_test` — `Lifecycle::new(RESTART_WINDOW_TICKS, MAX_RESTARTS)`, `on_event(SpawnOk)`, `on_event(InitializeResult)`, install. Uses the full `marley_lsp::lifecycle::Event` path (no import churn). |
| `crates/marley_app/src/headless_drive.rs` | New test `inlay_short_file_served_no_rerequest_headless` in the #331 section — the REQ-001/2/3 pin. |

### Regression test plan
| REQ | Test | Assert |
|---|---|---|
| REQ-003 (mint sanity) | `inlay_short_file_served_no_rerequest_headless` tick 1 | 4-rope-line fixture, geom seeded (0, 12) (viewport past EOF — the existing #352 seed idiom; `buffer_row` clamps ⇒ `end_row = 4 = len_lines`); after `refresh_inlay_hints_for_test`: `inlay_request_key_for_test() == Some(key)` with `key.first_row == 0 && key.last_row == 3` — the REAL path minted and SENT (Ready hook works). |
| REQ-001 (exclusive store) | same test, after response applied | Response injected for the CAPTURED key (`drive_inlay_for_test(Some(key), Some((key, json)))` — re-setting the identical key, noted in-test) + `consume_lsp_responses`; `inlay_cached_rows_for_test() == Some(0..4)` (old code: `0..3`); request key cleared to None by apply; hints present at row 1. |
| REQ-002 (no re-request) | same test, tick 2 | Geom re-seeded (headless renders reset it — :8026 idiom), `refresh_inlay_hints_for_test` again: `inlay_request_key_for_test()` STAYS None (bug: re-mints `Some(key)` — the in-flight guard cannot save it since `inlay_request` cleared on apply), cache intact (rows still `Some(0..4)`, hints still at row 1). |
| Existing #331/#352 suites | `cargo nextest run --workspace` | Stale-guard + toggle + geom tests unaffected (their asserts never read the cached range's end; `drive_inlay_for_test` keys `{0,40}` now cache `0..41` — no assert touches it). |
| Negative smoke (Phase 4, manual) | revert the `+1` momentarily | The new test goes RED (both fns are `mutants::skip`, so the unit is the pin — prove it pins). Restore. |

### Risks / decisions
- R1 — geom seed shape: seeding `last` past EOF is the established idiom (`:8045` "viewport past
  EOF") and exercises the same clamp the live render produces for a short file. Chosen over
  seeding exact slot count to make "file shorter than the viewport" literal.
- R2 — both touched fns are `mutants::skip` ⇒ the mutation gate will NOT police the `+1`; the new
  unit is the sole behavioral pin (hence the deliberate revert smoke in Phase 4).
- R3 — `set_ready_for_test` on a host WITH a real child would desync the lifecycle from the
  process; doc comment scopes it to process-less hosts, `#[cfg(test)]` keeps it out of prod.
- R4 — `ready_inlay_host_for_test` hardcodes `inlayHintProvider: true` caps — deliberately
  inlay-specific (name says so); other features keep their own seeding idioms.

## Phase 3 — Implement
- Built exactly to the manifest, zero deviations:
  - `app.rs` apply site — the exclusive-end store (`key.first_row..key.last_row.saturating_add(1)`)
    with the #401 constraint comment; the tuple wrapped multi-line (rustfmt).
  - `app.rs` — the three `#[cfg(test)]` hooks beside the #331 cluster
    (`ready_inlay_host_for_test`, `inlay_request_key_for_test`, `inlay_cached_rows_for_test`).
  - `lsp_host.rs` — `set_ready_for_test` beside `set_caps_for_test`, replaying
    `SpawnOk → InitializeResult` through the real `Lifecycle` (full-path `lifecycle::Event`, no
    import churn).
  - `headless_drive.rs` — `inlay_short_file_served_no_rerequest_headless` in the #331 section
    (written at implement so inspect sees the whole diff; RUN happens at validate per §7).
- React-first: **N/A** (spec: no UI delta — request cadence only).
- `cargo fmt --all` + `cargo check --workspace --all-targets` GREEN (11.4s; the `block v0.1.6`
  future-incompat note is pre-existing, transitive).

## Phase 3.5 — Inspect
Critics: 2 (scaled to the small diff) — correctness + reuse/test-idiom, parallel general-purpose
agents over the live diff; both instructed to verify concretely. Both did (critic 1 EXECUTED the
pre-fix variant in a throwaway scratchpad worktree).

| # | sev | finding | verdict | outcome |
|---|---|---|---|---|
| F1 | low | The +1 claims the wire's strict-semantics boundary row (`inlay_hint_params` end = `{line: want_last, char: 0}`): on a strictly-spec server, ONE viewport (`end_row == rows.end` exactly — bottom row = the fetch's boundary row) shows that row's hints absent, self-healing on any movement/edit. Invisible under rust-analyzer (accepts range-touching hints). | Real observation; the spec's EXPLICIT Out. Deferral confirmed SAFE — the pre-fix "recovery" at that boundary was the forever-refetch bug itself; re-asking with identical params can never fetch more. | Accepted-no-fix. Out bullet sharpened with the exact residual + the follow-up shape (wire end `{line: want_last + 1, char: 0}`). |
| F2 | low | **Reload/version-collision race loses its accidental self-heal.** `reload_active_from_disk` resets the version epoch (`BufferVersion::initial()`) and re-mints the nonce but cleared NO inlay state; `InlayKey` carries no nonce, so a raced pre-reload answer passes the numeric (uri, version) guards on a never-edited file, projects onto the NEW text, and — post-#401 — is served without the pre-fix refetch papering over it. | REAL — verified by reading `reload_active_from_disk` (:9014 area) + the apply guards; critic traced `reload_active` → version reset. Pre-existing acceptance-of-raced-answer (#331-era); this ticket only removed the accidental recovery, so it owns the hygiene. | FIXED — the reload now drops `inlay_hints` + `inlay_request` beside the #328 git-marks invalidation (same "content changed under us" family). New pin: `inlay_cache_dropped_on_disk_reload_headless` (cache AND parked in-flight key both cleared). |
| F3 | low | Stale #352 comment (":4779-4783"): "a direct `buffer_row(geom.last)` would clamp at EOF and flip the short-file served-check" defended bug-compatibility that no longer exists (post-fix, both forms pass the short-file case — nothing flips). | REAL — the projection stays load-bearing for a DIFFERENT reason (a direct clamp UNDERSTATES the exclusive end: it would serve a cache whose fetch stopped one row short of EOF). | FIXED — parenthetical reworded to the surviving reason, tagged #401. |
| F4 | low | `InlayCache` alias doc omitted the `Range` element entirely (pre-existing drift) and the field doc was silent on inclusivity — the exact ambiguity that bred this bug. | REAL. | FIXED — both docs now name `fetched_rows` + EXCLUSIVE end (#401). |
| F5 | nit | Fourth hook (`ready_inlay_host_for_test`) exceeded spec D4's three-item enumeration. | Real bookkeeping; the hook is NECESSARY (`RootView.lsp_hosts` is module-private to app.rs — the `set_signature_caps_for_test` precedent). | FIXED — D4 amended with the wrapper + rationale. |
| F6 | nit | `marley_lsp::lifecycle::Event::` full paths though `Event` is already in lsp_host.rs's import list (:18) and production writes bare `Event::SpawnOk` (:382). (The Phase-1 note claiming Event wasn't re-exported was wrong.) | REAL. | FIXED — bare `Event::`. |
| F7 | nit | The two mint-shape asserts were tagged REQ-003 but pin REQ-001's premise; REQ-003 (no cadence regression) lives in the tick-2 pass + the #331 suite per its own Verify column. | REAL (traceability). | FIXED — retagged "REQ-001 premise", comment names where REQ-003 actually lives. |

Verified clean (concretely, both critics): range algebra for every viewport position (`buffer_row`
clamps ⇒ `end_row ≤ len_lines` always, folds included — fold.rs:132-142); guard interplay
(edit/file-switch/fold/server-refresh all still refetch correctly); the cached range has no other
reader; NO hook duplication (no pre-existing headless path to Ready — every sibling feature
deliberately bypasses `request()` via the #331 injection lane, and this ticket's bug IS the send
decision, so the new lane is the minimal observable; locked by D3); house idiom exact (naming,
placement, `#[cfg(test)]` vs `mutants::skip`, doc style); no stale architecture docs (editor.md's
#331 blurb describes the error-reply loop, still true). **The test's discriminating power was
EXECUTED, not assumed:** pre-fix build fails REQ-001 (`Some(0..3)` vs `Some(0..4)`) and, with
range asserts weakened, independently fails REQ-002 with exactly the predicted re-minted key.
Post-fix: 895/895 lib tests green (critic run).

Disclosure: critic 1's pre-fix worktree build shared the repo `target/` and briefly overwrote the
marley test binary; repaired via `cargo clean -p marley` + rebuild before their green run. Phase 4
re-runs everything from the main tree regardless (first nextest run will rebuild the cleaned
package).

## Phase 4 — Validate
- **Tests added** (written at Phase 3 so inspect saw the whole diff; RUN here):
  - `inlay_short_file_served_no_rerequest_headless` — REQ-001 (exclusive store, `0..4`),
    REQ-002 (tick-2 no re-mint through the REAL Ready-host send path), REQ-001-premise mint pins;
    REQ-003 rides the tick-2 pass + the #331 suite per its Verify column.
  - `inlay_cache_dropped_on_disk_reload_headless` — the inspect-F2 pin (reload drops cache AND
    parked in-flight key).
- **Targeted run:** 6/6 inlay tests green (4 pre-existing + 2 new).
- **Negative smoke (design R2 — both touched fns are `mutants::skip`, so the unit is the sole
  behavioral pin):** reverted the `.saturating_add(1)` momentarily → the pinning test went RED
  exactly as predicted (`left: Some(0..3)` vs `right: Some(0..4)`); restored → green. The pin is
  real.
- **Full suite:** `cargo nextest run --workspace` — **2045/2045 passed** (5 skipped, the standing
  live-external set), up from 2043 pre-ticket. `cargo test --workspace --doc` — 0 doctests
  (none new), ok.
- **Live-app drive:** N/A — no UI delta (spec React-first N/A; cadence-only change, nothing
  renders differently). No `visual_acceptance` clause in the AC.
- **Pre-existing notes:** the `block v0.1.6` future-incompat warning (transitive, macOS
  objc stack) — pre-existing, untouched. No other pre-existing failures.
- **Gate:** `scripts/gates.sh --diff` — **GATE GREEN [diff], 15/15 on the FIRST run** (no red
  cycle): rustfmt, clippy -D warnings, tests (2045 + doctests; visual-harness 158/158), coverage
  ≥100% lines, mutation MSI ≥100%, miri, audit, deny, machete, gitleaks, shellcheck,
  no-suppressions, source-bans, docs, visual/AX. Receipt written (`.git/ignibyte-gate-receipt`,
  21:59). Exit code 0 owns the verdict; run bare in background per the standing rule.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG — new `### Fixed` section in Unreleased (house order: after Changed,
  the 0.0.0-block precedent) with the full #401 entry. `docs/marley_architecture/editor.md` — the
  #331 cadence section gains "The served-check's own forever-loop (#401)" (the success-path loop,
  the reload epoch-collision hygiene, the Ready-replay test lane). Parity sync: N/A — no UI delta;
  `marley-web` + MARLEY-PARITY.md untouched by design.
- **Knowledge capture (§19):** failure `BF-claude-inlay-reload-version-epoch-collision-001` +
  prevention rule `PR-claude-version-epoch-resets-invalidate-numeric-keyed-caches-001` recorded at
  inspect; AAR be7a238a submitted at close (completed; lessons in the submission: the
  Ready-replay hook pattern — replay real state-machine events over a bypass constructor; the
  negative-smoke-for-mutants::skip-territory idiom; fix-the-waste-loop ⇒ audit what the loop was
  accidentally healing).
- **Ticket:** forge #401 → done; local doc → `tickets/closed/`, Status closed, pipeline pointer
  → completed/.
- **Archive:** spec+notes pair → `docs/planning/pipeline/completed/`.
- Run summary: ONE red cycle count — zero (gate green first run). Two headless drives added;
  2045/2045; cov 100 / MSI 100.
