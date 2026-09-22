# LSP doc-sync race-proof materialization — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-320-lsp-doc-sync-race-proof.md
- **Pipeline spec:** 320-lsp-docsync-race-proof.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** `/work` auto-pick, top of Queue: TICKET-320 — make the #309
  didOpen-empty-text bug class impossible by construction (one Ready read that
  decides AND materializes; delete `needs_text`) + add the app-level Ready
  (`fake_ls`) test lane that would have caught it + document the tee-wrapper
  wire-capture diagnostic.
- **Classification / tier:** chore, M20, single work pipeline (one shippable
  slice: structural fix + test lane + docs note ride together — the test lane is
  the point of the ticket, not an add-on).
- **Recall (§18.3):** the ledger carries the complete story:
  - `BF-lsp-didopen-carries-empty-text-ready-race-001` (critical) — the shipped
    bug: collect-before-drain saw pre-Ready (`text = ""`), reconcile saw Ready,
    sent the empty doc AND recorded it synced; permanent zero-length documents;
    every M20 request answered null. Wire-proven before/after in #312.
  - `PR-claude-two-gated-calls-must-read-the-phase-once-001` (critical) — the
    structural rule this pipeline enacts: collapse two same-state-gated reads
    into one call that decides and materializes.
  - `PR-claude-synthetic-response-tests-never-prove-the-live-wire-001` (high) —
    why no test caught it: every M20 drive injects via `push_response_for_test`
    into a process-less host that never reaches Ready; assert payloads, not
    method names.
  - `PR-claude-pump-materialize-hot-data-only-when-consumed-001` (medium) — the
    #309-S2 hot-path optimization (no per-tick full-buffer clones) must survive
    the refactor — D3 makes it provable with a counting text source.
  - Completed-notes recall (#309/#312): the host is APP-SIDE (`lsp_host` in
    `marley_app`; pump in `app.rs`); `marley_lsp` holds the pure seams
    (`docsync.rs` builders, `rpc` envelope, `position.rs`);
    `EditorSurface::open_docs()` (the #309-S1 fix) yields `(path, &Buffer)` for
    every open file; #312's fix was ordering-only (drain loop first, then
    collect, then reconcile) — explicitly recorded there as re-breakable by a
    refactor, with this ticket as the follow-up.
- **Prior-art sweep (§20):** recorded in the spec. Highlights: Zed behavior map
  §3.2 is the reference (didOpen/didChange keyed to agreed versions); LSP 3.17
  didOpen carries the FULL text (the contract the payload assertion enforces);
  deps leg — `lsp-types` 0.97 (MIT) already shipped but deliberately confined to
  the diagnostics parse seam (#310), owns types not client logic; no crate in
  the tree owns the reconcile seam. No adoption available; no locked decision
  dissolved.
- **Discovery (Explore agent, §18.2 — the seam map for Design):**
  - **Pump:** `crates/marley_app/src/app.rs:1906-1979` — current order is
    DRAIN (all hosts, `:1922`) → COLLECT `open_files: Vec<(PathBuf, String,
    BufferVersion)>` with the `needs_text` gate inside the map closure
    (`:1935-1954`) → `open_files.sort_by` (`:1956`, #397 determinism — MUST
    survive) → `ensure_lsp_host_for_open_docs(&root, paths)` (`:1970`, #321
    host CREATION consumes the same set — must keep receiving a path iterator)
    → `reconcile(&open_files)` (`:1975`). Exactly ONE call site each for
    `reconcile` and `needs_text` (workspace-grep-verified); the #312 ordering
    is guarded only by comments ("Do not introduce a `drain()` between…",
    `:1966-1969`) — precisely the fragility this ticket removes.
  - **Host:** `crates/marley_app/src/lsp_host.rs` (756 lines, `pub(crate)`,
    a shim by design — decisions live in `marley_lsp`'s pure modules).
    `reconcile(&mut self, open: &[(PathBuf, String, BufferVersion)]) -> bool`
    at `:176-177`; `needs_text(&self, path, version) -> bool` at `:241-242` —
    both early-return unless `Phase::Ready`; `needs_text` mirrors reconcile's
    decision table minus the uri step (the duplication being collapsed).
    `docs: HashMap<PathBuf, DocEntry>` (`:118`), `DocEntry { lsp: DocVersion,
    synced: BufferVersion }` (`:86-89`). Ready is set INSIDE `drain()`:
    `drain` `:277` → `on_message` `:419` → initialize response → `step` →
    `marley_lsp/src/lifecycle.rs:127`. `send_body` `:733` forwards to
    `ServerHandle::send` (an mpsc sender, `marley_lsp/src/process.rs:47-49`) —
    NO outbound capture exists anywhere today.
  - **Dependency direction:** `lsp_host.rs:13` already imports
    `marley_editor::BufferVersion` — passing `&Buffer` needs NO new Cargo
    edge. `Buffer` is `crates/editor/src/buffer.rs:36` (ropey-backed, pure);
    `text() -> String` `:95-98`, `version() -> BufferVersion` `:91`.
  - **fake_ls:** `crates/marley_lsp/src/bin/fake_ls.rs` (73 lines,
    auto-discovered bin, no `[[bin]]` section). Answers `initialize` (caps =
    bare `positionEncoding: utf-16` — reaches Ready), `shutdown`, `exit`;
    DROPS everything else and records nothing — so the payload assertion MUST
    come from the app-side send capture, not the fixture. Used today only by
    `marley_lsp/tests/integration.rs` (`env!("CARGO_BIN_EXE_fake_ls")`,
    `:14`); no `current_exe()` helper exists anywhere in `crates/`.
  - **Test lanes today:** all app drives live in
    `crates/marley_app/src/headless_drive.rs` (~13k lines). The five
    pump-driven LSP drives (#321/#359, `:10173-10366`) use a NONEXISTENT
    binary so nothing spawns (assertions are `docs == Some(0)`); the ~23
    injected-response drives never reach Ready. Comments at `:10254` and
    `:9761` explicitly park the live-child lane on "#320". Existing
    `#[cfg(test)]` shelf on the host: `open_doc_count` `:496`,
    `push_response_for_test` `:699`, `set_ready_for_test` `:724` (process-less
    Ready via the REAL Lifecycle — useful prior art, but wire-less). Pump
    ticking in tests = `tick_pump` (`advance_clock(40ms)` + park, `:116-121`).
  - **Gate posture (constrains verification):** `scripts/gates.sh:229`
    coverage-EXCLUDES `marley_app/src/lsp_host.rs`, `marley_app/src/app.rs`,
    `marley_lsp/src/process.rs`, `marley_lsp/src/bin/`; `lsp_host.rs` fns are
    `mutants::skip`'d. Changes here are DRIVE-PROVEN (gate:3), not
    coverage/mutation-proven — #321's notes park exactly this on #320.
  - **selftest docs:** `scripts/selftest/README.md` exists (AX driver
    harness); NO tee-wrapper documentation exists anywhere in the repo —
    Part 3 is fresh writing.
  - **Design constraints surfaced:** (a) a `text_of` closure capturing
    `&view.content` collides with `view.lsp_hosts.get_mut` at `:1974`
    (E0499/E0502) unless bound to a disjoint field first — cf.
    `PR-claude-two-mut-self-accessors-need-a-combined-accessor-001`; (b) the
    `send_body` capture must sit BEFORE the `if let Some(handle)` to record on
    process-less hosts too (`ServerHandle::send` silently no-ops childless —
    `set_ready_for_test` relies on that); (c) `ensure_lsp_host_for_open_docs`
    + the #397 sort must keep working over the reshaped open set.
- **Decisions:** D1–D6 locked in the spec (one phase read; delete `needs_text`;
  hot path provable; payload-not-method; real-Ready lane bans
  `push_response_for_test`; `#[cfg(test)]`-only capture). One fork deliberately
  left for Design: closure (`impl Fn(&Path) -> String`) vs `&Buffer` as the text
  source — both dependency-legal since the host is app-side.

## Phase 2 — Design

### Architecture / approach

**D-TEXT-SOURCE (settles the spec's open fork): a `DocText` trait, not a
closure.** `lsp_host.rs` gains

```rust
pub(crate) trait DocText {
    fn version(&self) -> BufferVersion;
    fn text(&self) -> String;
}
impl DocText for marley_editor::Buffer { … } // delegates to version()/text()
```

and `reconcile` becomes
`pub(crate) fn reconcile(&mut self, open: &[(PathBuf, &dyn DocText)]) -> bool`.
Why the trait beats both ticket suggestions:
- **Same-OBJECT atomicity, not just same-call:** the closure form fixes the
  gate/materialize split but still ships a separately pre-read `version`; the
  trait reads version AND text from ONE live object at decision time, so they
  cannot disagree either (the bug class was exactly "decision state and
  materialized data disagree").
- **Countable:** REQ-002's proof needs a counting text source — trivial with a
  test double implementing `DocText`, impossible with a bare `&Buffer`.
- `&dyn` (object-safe: two `&self` methods) keeps the shim non-generic — no
  monomorphization, callable with doubles.
- Trait + `impl for Buffer` live in `lsp_host.rs` (app-side; local trait on a
  foreign type is orphan-legal; `marley_editor::BufferVersion` import already
  exists at `lsp_host.rs:13`). `marley_lsp` stays editor-free.

Inside `reconcile`, the decision table (`lsp_host.rs:195-213`) changes only at
the leaves: `None` branch → `doc.text()` materialized AT the `open_notification`
call; `synced != doc.version()` branch → `doc.text()` at
`change_notification_full`; `Some(_)` idle branch touches NOTHING. `needs_text`
(`:238-251`, doc comment included) is DELETED — its one caller dies in the pump
reshape.

**Pump reshape (`app.rs:1906-1979`) — two passes, borrow-clean:**
1. drain all hosts (unchanged, `:1922`);
2. **paths pass:** `let mut paths: Vec<PathBuf>` from the registry
   (`content.iter() → as_editor() → root-filter → path().to_path_buf()`),
   sorted (#397 determinism) → `view.ensure_lsp_host_for_open_docs(&active_root,
   paths.iter().map(..as_path))` — a whole-`view` `&mut` call with NO live
   content borrows (this sidesteps the E0499/E0502 shape entirely; a
   single-pass `Vec<(PathBuf, &Buffer)>` would hold `view.content` borrows
   across the `&mut self` method call);
3. **docs pass:** `let mut open_docs: Vec<(PathBuf, &dyn DocText)>` from the
   same registry iteration (`(path.to_path_buf(), i.buffer() as &dyn DocText)`),
   sorted the same way → `view.lsp_hosts.get_mut(&active_root)` (direct
   field-place, disjoint from the `view.content` borrows — legal) →
   `host.reconcile(&open_docs)`.
   Two registry iterations over a handful of open tabs — negligible; the text
   clone (the actual cost) now happens at most once per send branch, same as
   the #309-S2 optimization, minus the race.

**Why this is "impossible by construction", stated for the comment rewrite:**
after the reshape NO phase-gated data exists outside `reconcile` — the pump
carries only paths and live object refs. Even a hostile refactor that moved the
doc-collect above the drain (or reconcile before drain) can no longer corrupt:
reconcile either sees not-Ready and no-ops (one-tick delay, next tick sends the
REAL text) or sees Ready and materializes fresh. The failure mode degrades from
"permanent empty document recorded as synced" to "≤16ms latency". The
`app.rs:1911-1920` race narrative + the `:1966-1969` "do not introduce a
drain()" warning are rewritten to say exactly this (drain-first is kept — it is
still the freshness-optimal order, just no longer load-bearing for
correctness).

**Capture hook:** `#[cfg(test)] sent_bodies: Vec<String>` field on `LspHost`
(cfg'd init in `new()`), pushed at the TOP of `send_body` — BEFORE the
`if let Some(handle)` — so process-less hosts (`set_ready_for_test`) record
too (Explore delta 6); `#[cfg(test)] pub(crate) fn sent_bodies_for_test(&self)
-> &[String]`. RootView-side drive accessor: `#[cfg(test)]
fn lsp_sent_bodies_for_test(&self, root: &Path) -> Vec<String>` (clones out,
joining the existing `*_for_test` shelf at `app.rs:5494-5519`).

**fake_ls bridge (settles the ticket's 3-way choice): config-seeded real
binary, path resolved from `current_exe()`.** The app's OWN `[[lsp.servers]]`
mechanism already accepts any command (`LanguageServerConfig { language,
command, args }` — `seed_stub_language_server` seeds a nonexistent one); the
lane seeds the REAL `fake_ls` instead — the exact mechanism the live tee
capture used, now in CI. Resolution: `fake_ls_path()` in `headless_drive.rs` —
`current_exe()` (`target/debug/deps/marley_app-…`) → pop to `target/debug` →
`fake_ls`; if absent (a `-p marley_app`-only run — under the workspace gate
`marley_lsp`'s integration tests force-build it), a `OnceLock`-guarded
`cargo build -p marley_lsp --bin fake_ls` builds it (nested cargo is safe
post-build-phase; cargo's own lock serializes; one-time ~seconds). No silent
skip either way. Rejected: moving the lane into `marley_lsp/tests/` (it needs
the app pump — dependency cycle); cargo artifact-deps (`bindeps` unstable).

**§20 confirm:** the spec's Reference stands — Zed behavior map §3.2
(didOpen/didChange keyed to agreed versions), behavior-level only; this design
matches it by sending the full text + version read atomically from the live
buffer at Ready. No copyleft source read or translated; `fake_ls`, `DocText`,
and the tee note are Marley-specific harness. **React-first N/A confirmed** —
no user-visible delta anywhere in this manifest.

### File manifest
1. `crates/marley_app/src/lsp_host.rs` — `DocText` trait + `impl for Buffer`;
   `reconcile(&mut self, &[(PathBuf, &dyn DocText)])` materializing on send
   branches; DELETE `needs_text` (+ its doc comment); `#[cfg(test)]
   sent_bodies` field + push at top of `send_body` + `sent_bodies_for_test()`;
   new `#[cfg(test)] mod tests` (counting double + capture + pre-Ready gate
   units — `LspHost::new(root, …)` arity checked at implement,
   `set_ready_for_test` promotes).
2. `crates/marley_app/src/app.rs` — pump reshape (two passes, above); race
   comment rewritten to the by-construction invariant; `#[cfg(test)]`
   `lsp_sent_bodies_for_test` RootView accessor.
3. `crates/marley_app/src/headless_drive.rs` — `fake_ls_path()` resolver (+
   OnceLock build fallback); `seed_fake_language_server(dir)` (real command,
   empty args); the lane test
   `didopen_payload_carries_real_text_against_fake_ls_headless`.
4. `scripts/selftest/README.md` — new `## LSP wire capture (tee wrapper)`
   section + the same-file-definition-empty tell (cites
   BF-lsp-didopen-carries-empty-text-ready-race-001 /
   PR-claude-synthetic-response-tests-never-prove-the-live-wire-001).
No `marley_lsp` source changes; no `marley-web` half (React-first N/A).

### Regression test plan

| REQ | Test | Kind / gate |
|---|---|---|
| REQ-001 | `didopen_payload_carries_real_text_against_fake_ls_headless` — tempdir Cargo fixture with DISTINCTIVE multi-line source (not the stock `fn main() {}`, so byte-equality is meaningful); seed fake_ls; boot; `poll_until` a didOpen appears in `lsp_sent_bodies_for_test`; parse JSON; assert `params.textDocument.text` byte-equal to the fixture string AND non-empty; exactly ONE didOpen for the uri; a few extra ticks then didChange-count == 0 (synced recorded right) | app drive, gate:3 |
| REQ-002 | host unit: `CountingDoc` (text() increments a counter) on a process-less `set_ready_for_test` host — reconcile → didOpen + count 1; reconcile again same version → no body, count STILL 1 (idle materializes nothing); bump version → didChange + count 2; drop the doc → didClose | unit, gate:3 |
| REQ-003 | compile surface: workspace builds with `needs_text` deleted (its one caller removed); validate greps `needs_text` absent from `crates/`; decision table drive-proven by the REQ-001 lane + REQ-002 unit (standing posture: `lsp_host.rs` coverage-excluded + `mutants::skip`) | compile + grep + drive |
| REQ-004 | same lane test: footer segment reaches Ready with the lane using NO injection API (`push_response_for_test`/`set_ready_for_test` absent from the lane — review) — Ready is reachable only via the child's initialize through `drain()` | app drive + review |
| REQ-005 | host unit: process-less host → send path → `sent_bodies_for_test` returns bodies in order (proves capture-before-handle placement); the REQ-001 lane consumes the same hook cross-tick | unit, gate:3 |
| REQ-006 | README section present + accurate; gate:14 (docs) green | review + gate |

- **trybuild:** none — no new public type contract (`DocText` is `pub(crate)`;
  the deleted API is an absence, not a misuse to assert).
- **Uncoverable/posture notes:** `lsp_host.rs`, `app.rs`, `marley_lsp/src/bin/`
  are standing coverage excludes; `lsp_host.rs` fns `mutants::skip` (shim) —
  the drive lane IS the proof surface (the #321 notes park exactly this on
  #320). The lane spawns a real child process — precedented (drives spawn real
  zsh PTYs; `marley_lsp` integration tests spawn fake_ls today).

### Risks / decisions
- **R1 borrow reshape** — the two-pass shape is pre-designed around
  E0499/E0502 (`PR-claude-two-mut-self-accessors-need-a-combined-accessor-001`);
  any residual conflict is compile-caught, not latent.
- **R2 fake_ls resolution** on `-p marley_app`-only runs — OnceLock cargo-build
  fallback; nested-cargo lock contention is serialized by cargo itself;
  one-time cost small (tiny bin).
- **R3 real-child flake** — `POLL_CEILING` 30s standing pattern; fake_ls
  answers initialize immediately (no rust-analyzer indexing heft).
- **R4 capture growth** — `Vec<String>` unbounded, but `#[cfg(test)]`-only and
  drives are short; accepted.
- **R5 existing drives** — `reconcile`/`needs_text` had exactly ONE call site
  each (pump); the #321/#359 nonexistent-binary drives assert `docs == Some(0)`
  and host existence — unaffected beyond recompilation; injected-response
  drives bypass the pump's sync path entirely.
- **R6 tick ordering preserved** — ensure-before-reconcile (same tick) and
  drain-first stay; only the collect's CONTENT changes (paths + object refs,
  no phase-gated data).

## Phase 3 — Implement
- **React-first: N/A** (per spec — LSP internals + test lane + docs; no user-visible delta).
- **Built (to the manifest):**
  1. `lsp_host.rs` — `DocText` trait (`version()`/`text()`) + `impl for Buffer`
     (plain delegations, deliberately NOT `mutants::skip`'d — the Phase 4 lane +
     an edited-buffer unit make both mutants killable); `reconcile` now takes
     `&[(PathBuf, &dyn DocText)]` and reads version + text inside the send
     branches (doc comment carries the #320 invariant); **`needs_text` DELETED**
     (fn + doc comment); `#[cfg(test)] sent_bodies: Vec<String>` field (cfg'd
     init in `new()`), pushed at the TOP of `send_body` before the handle check
     (process-less hosts record), + `sent_bodies_for_test()` beside the other
     test hooks.
  2. `app.rs` — pump reshape: drain (unchanged) → **paths pass**
     (`open_paths: Vec<PathBuf>`, sorted) → `ensure_lsp_host_for_open_docs`
     (whole-view `&mut`, no live content borrows) → **docs pass**
     (`open_docs: Vec<(PathBuf, &dyn DocText)>`, sorted — #397 preserved) →
     field-disjoint `lsp_hosts.get_mut` → `reconcile`. The #312 race narrative
     + the "do not introduce a drain()" warning are REPLACED by the
     by-construction invariant comment (reorder ⇒ worst case one-tick delay,
     never a wrong payload; drain-first kept as freshness-optimal). RootView
     `lsp_sent_bodies_for_test(root)` added to the `*_for_test` shelf.
  3. `scripts/selftest/README.md` — new "LSP wire capture (tee wrapper)"
     section: wrapper script + `[[lsp.servers]]` config, payload-grep recipes,
     the same-file-definition-empty tell, and the pointer to the #320 CI lane;
     cites the BF/PR codes.
- **Deferred to Phase 4 (per the implement rule "no tests here"):** manifest
  file 3 (`headless_drive.rs`: `fake_ls_path()` resolver + OnceLock build
  fallback, `seed_fake_language_server`, the lane test) and the `lsp_host.rs`
  unit tests (counting double, capture order, pre-Ready gate, edited-buffer
  didChange via the real `Buffer` impl).
- **Deviations from design:** none functional. Confirmations: `new(root, None)`
  gives the process-less unit-test host directly; per-instance open-set
  semantics kept exactly (no dedup added — #319's single canonical identity
  already guarantees one instance per (root, file)).
- **Compile evidence:** `cargo check --workspace` green FIRST compile (3.07s) —
  the two-pass borrow design needed no rework; `cargo fmt --all` +
  `cargo check --workspace --all-targets` green (4.47s) with exactly two
  `dead_code` warnings = the two new `#[cfg(test)]` hooks, consumed by the
  Phase 4 tests (gone before gate:2 runs at validate). Pre-existing:
  `block v0.1.6` future-incompat note (upstream, unrelated).

## Phase 3.5 — Inspect

Three parallel critics (correctness / data-state integrity / simplification+
provenance) over the diff. Headline: ZERO critical/high/med correctness
findings — the by-construction claim survived adversarial interleaving attempts
(crash mid-tick, writer death mid-reconcile, restart same-tick, hostile
reorders of drain/collect/ensure/reconcile: worst case one-tick delay, never a
wrong payload; the version/text atomicity is enforced by the borrow checker,
not call ordering). All five tick shapes traced wire-identical to the old code;
30/30 existing LSP drives pass.

| # | Sev | Finding | Verdict | Fix |
|---|-----|---------|---------|-----|
| I1 | LOW | two dead-code warnings: the new capture hooks have no callers yet | ACCEPTED-TRANSIENT — exactly `PR-claude-cfg-test-helpers-land-with-their-tests-001`; Phase 4's tests consume both | none now (gate runs at validate, after the consumers land) |
| I2 | LOW | didChange arm read `doc.version()` twice (guard + record) — safe for pure `Buffer`, a trap for interior-mutable test doubles | REAL (hardening) | ONE `version` binding per entry before the match; both arms record exactly what they compared |
| I3 | LOW | didClose emission is HashMap-ordered (pre-existing) while the rewritten comment claims deterministic order — and the new capture makes send order assertable (flake trap) | REAL | `closed.sort()` — the whole notification stream is now deterministic |
| I4 | INFO | `sent_bodies` survives crash-restart (host is REUSED; `on_connection_lost` clears docs/pending/responses but not the capture) and records bodies the dead wire never carried | ACCEPTED AS DESIGNED — the contract is cumulative "what the app SAID"; clearing would erase the pre-crash record that no-didOpen-before-Ready assertions need; Phase 4 drives are single-generation | contract documented on the hook |
| I5 | **MED** | the root-filtered editor chain (`content.iter().filter_map(as_editor).filter(root==)`) hand-copied 4× (pump ×2, #326 reference texts, #397 search overrides) — #321's ensure-set==sync-set invariant rested on copies staying byte-identical | REAL | `ContentRegistry::<Content>::editors_under(root)` in content.rs (borrows ONLY the registry field — pump's field-disjoint `get_mut` preserved); all 4 sites converted |
| I6 | **MED** | `impl DocText for Buffer` fns will be diff-mutated at the gate; a `version()`→constant mutant plausibly survives the then-planned tests (counting double doesn't use Buffer's impl; the lane fixture is never edited) | REAL | impls stay UNSKIPPED (mutation-proven beats skip-marked); Phase 4 test plan gains the killing row: a real-`Buffer` edit→didChange unit (constant-version mutant ⇒ no didChange ⇒ dies; empty-text mutant ⇒ byte-equality dies) |
| I7 | LOW | pass-2 `to_path_buf()` per tick avoidable (per-tick clones had DOUBLED vs pre-diff) — the exact PR-1095 hot-path surface | REAL | docs pass is `Vec<(&Path, &dyn DocText)>`; `reconcile(&[(&Path, &dyn DocText)])`; zero PathBuf allocation on the 16 ms tick (pass-1's owned paths are borrow-forced — confirmed load-bearing) |
| I8 | LOW | `open_paths.sort()` had no consumer-visible effect (ensure is set-semantics) and the determinism comment was attached to the wrong pass | REAL | sort dropped; determinism rationale moved onto the docs pass (the stream that reaches the wire) |
| I9 | LOW | capture's `body.clone()` avoidable | REAL | push moved AFTER the send (`encode_frame` only borrows) — the `String` moves in clone-free |
| I10 | LOW | README forward-references the Phase-4 lane test by exact name; hooks unwired until then | ACCEPTED-SEQUENCING — validate MUST land `didopen_payload_carries_real_text_against_fake_ls_headless` under that exact name (or fix the README in the same change) |
| I11 | LOW | README length-grep oversold (JSON escaping ⇒ lower bound) | REAL | labeled "approximate (a LOWER BOUND)" with the reason |
| I12 | LOW | `docs/marley_architecture/crate-map.md:110` still present-tenses `needs_text` + the old drain-ordering invariant | REAL — Phase 5 duty (§21 arch-doc update); tracked below |
| I13 | INFO | Phase-1 recall cited `EditorSurface::open_docs()` as the (path,&Buffer) source — the API does not exist (stale #309-notes memory; the #397 REGISTRY derivation superseded it) | CORRECTED here — Discovery's registry-based map was already right |

**Provenance (code-layer wall):** CLEAN — elementary Rust idioms with documented
in-repo lineage (#309 → BF → #320); nothing mirrors Zed's event-driven
LspStore/buffer-handle architecture; tee wrapper is universal folk practice;
README example matches `marley_lsp` config field-for-field.

**Post-fix verification:** `cargo fmt` + `cargo check --workspace --all-targets`
clean (only the I1 transient warnings) + `cargo nextest run -p marley -E
'test(lsp)'` → **30/30 pass** (1 leaky = the standing PTY-boot behavior).

**Validate obligations carried forward (binding on Phase 4):**
1. The lane lands under the EXACT name
   `didopen_payload_carries_real_text_against_fake_ls_headless` (I10).
2. A real-`Buffer` edit→didChange unit kills the `DocText`-impl mutants (I6).
3. Both capture hooks gain callers (I1); negatives tick the pump
   (`BF-claude-headless-drive-never-ticked-the-pump-negatives-passed-vacuously-001`).
4. Phase 5: fix `crate-map.md:110` (I12).

## Phase 4 — Validate

### Tests added (per the Phase 2 plan + inspect obligations)
1. `lsp_host::tests::idle_reconcile_materializes_nothing` — REQ-002 + REQ-005:
   `CountingDoc` double on a process-less `set_ready_for_test` host — didOpen
   materializes ONCE; the idle tick reads ZERO text; version bump → didChange
   (one more read); drop → didClose (no read); the capture holds exactly
   [didOpen, didChange, didClose] in send order.
2. `lsp_host::tests::pre_ready_reconcile_sends_and_reads_nothing` — the Ready
   gate lives inside the one call: not-Ready ⇒ no decision, no send, NO
   materialization.
3. `lsp_host::tests::didopen_payload_equals_buffer_and_edit_sends_didchange` —
   REQ-001 (unit half) + inspect-I6 mutant-killer through the REAL `Buffer`
   impl: didOpen byte-equal to the buffer; a real `edit()` → didChange carrying
   the post-edit text; an unedited follow-up tick is silent.
4. `headless_drive::didopen_payload_carries_real_text_against_fake_ls_headless`
   — REQ-001 + REQ-004, the ticket's core lane (exact I10 name): REAL spawned
   `fake_ls` via the app's own `[[lsp.servers]]` config (`fake_ls_path()`
   resolves `current_exe()` → `target/debug/fake_ls`, OnceLock `cargo build`
   fallback for `-p`-only runs), `Phase::Ready` via a real `drain()` of the
   child's initialize response — ZERO injection APIs in the lane; asserts the
   PAYLOAD: exactly ONE didOpen, text byte-equal to the distinctive fixture
   source, `TEXT_LEN > 0`, uri addresses the file, and after 3 further pump
   ticks still one didOpen + zero didChange (the synced-version bookkeeping
   half). Passed FIRST RUN (~0.16s — fake_ls answers initialize instantly).
5. `content::tests::editors_under_filters_by_root_and_kind` — inspect-I5: pins
   the ONE shared root-filter (kind + root halves) the ensure/reconcile passes
   ride on; also the mutation-killer for `editors_under`'s filter.
   (Unit-test canonicalization gotcha captured: virtual doc paths must be built
   from the CANONICAL root — macOS `/var` → `/private/var` — or the host's
   `starts_with` scope check filters them; the two host units hit this and were
   fixed by canonicalizing first. The lane never hits it: its file exists.)
- **trybuild:** none — no new public type contract (per plan; `DocText` is
  `pub(crate)`).

### Runs (real, transcript-visible)
- Targeted #320 set: **5/5 PASS** (after the canonicalization fix + the
  `Content::editor` boxing fix — both test-side).
- `cargo nextest run --workspace`: **2081/2081 PASS**, 5 skipped (the standing
  `#[ignore]` headed tests).
- `cargo test --workspace --doc`: green (no doctests in most crates — 0 run
  where absent).
- Both capture hooks now have callers (I1 closed: `sent_bodies_for_test` by the
  host units, `lsp_sent_bodies_for_test` by the lane) — the transient
  dead-code warnings are gone.

### Live-app drive (step 3)
N/A — no UI surface: this ticket changes the pump/host internals + test lanes
+ docs; no render, input, layout, or affordance path is touched (spec
React-first: N/A). The live-behavior proof for what DID change is the lane
itself: a real child process through the real config→resolve→spawn→drain→
reconcile path, asserting real wire payloads — precisely the proof surface the
ticket was minted to add.

### Gate
**`scripts/gates.sh --diff` → GATE GREEN [diff], 15/15 PASS** (receipt written).
Mutants on the touched lines: **4 caught / 0 missed / 2 unviable — MSI 100%**
(the unskipped `DocText` impls + `editors_under` are mutation-PROVEN, the I6
obligation). Two red iterations on the way, both fixed at source:
1. **Red #1 — gate:2 + gate:4:** the fake_ls build fallback used
   `std::process::Command` (disallowed type — spawns live in the
   `marley_command` adapter, seam-contracts §4.2) and its never-taken branch
   was uncovered. First fix (drop the fallback for a loud assert) healed
   clippy but broke gate:5 in a subtler way ↓
2. **Red #2 — gate:4 + gate:5:** (a) coverage missed exactly ONE line — the
   assert's failure-only `bin.display()` trailing-arg line (the drives'
   standing idiom is IMPLICIT capture, which has no failure-only arg line);
   (b) the cargo-mutants BASELINE builds `--package=marley` ALONE in a
   pristine tree copy — `fake_ls` never builds there, the loud assert failed
   the baseline, exit 4, fail-closed. Root insight: the first run's gate:5
   green had been CARRIED by the fallback.
3. **Final shape (all green):** the build call is UNCONDITIONAL (an
   up-to-date no-op when the workspace suite already built the bin) via
   `marley_command::blocking::Command` (the exact `git`-helper idiom in
   app.rs) — the mutants copy gets its fixture, every line executes for
   coverage, clippy is satisfied, and `CARGO_TARGET_DIR` inheritance keeps
   resolver and build agreeing under llvm-cov's redirect. Assert messages use
   implicit capture.

### Pre-existing (not in scope)
- `block v0.1.6` future-incompat note (upstream transitive, predates this
  pipeline; gate:7/8 green).
- nextest "1 leaky" on the LSP filter run — the standing PTY-boot behavior,
  predates this pipeline.

## Phase 5 — Complete
- **CHANGELOG:** `### Changed` entry under `[Unreleased]` (the full story: trait,
  deletion, two-pass pump, editors_under, capture, lane, gate iterations).
- **Architecture docs (§21b):** `docs/marley_architecture/crate-map.md` marley_lsp
  row — the stale "ordering is load-bearing / MUST drain before collect" claim
  replaced with the by-construction record (+ the lane + capture); the #312-era
  "do not introduce a drain()" warning explicitly RETIRED (inspect I12). Swept:
  no other live arch doc mentions `needs_text` (the one remaining mention is the
  deletion record itself).
- **Parity (§21c):** N/A — non-UI ticket (spec React-first: N/A).
- **Ledger appends this pipeline (§18.3/§19):**
  - `L-claude-same-object-atomicity-beats-same-call-001` (design phase)
  - `PR-claude-shared-set-derivations-must-be-one-function-001` (inspect)
  - `PR-claude-mutants-baseline-builds-only-the-tested-package-001` (validate's
    two-red-cycles class — cross-crate fixtures vs the pristine-copy baseline)
  - `L-claude-assert-messages-implicit-capture-for-coverage-001` (the gate:4
    one-line red)
  - `AD-claude-lsp-docsync-doctext-live-object-001` (the shipped seam decision)
- **Ticket:** TICKET-320 → `tickets/closed/`, status closed; BACKLOG row left at
  promotion (swept — none stale).
- **Archive:** doc pair → `docs/planning/pipeline/completed/`.
