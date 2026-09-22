# Marley EARS Specs — Round-2 Re-Verification (pre-first-ticket gate)

**Reviewer pass:** re-verification of the round-1 fix pass against the canonical
`standards/seam-contracts.md` and `standards/quality-bar.spec.md`.
**Scope:** 18 `SPEC-*.spec.md` + `docs/pipeline/visual-testing.spec.md` + `SPEC-gate.spec.md`
harness + `README.md` manifest.
**Prior findings:** `review-r1.md` (11 blockers).

---

## TOP-LINE VERDICT: NOT_READY

**Rationale.** The fix pass did excellent work: 9 of the 11 round-1 blockers are
genuinely closed, every blocker-class seam in `seam-contracts.md` now has exactly one
owner with consumers importing rather than re-declaring, the three new specs
(`SPEC-marley-util`, `SPEC-asset-foundation`, `visual-testing.spec.md`) are high-craft and
each substantially closes its assigned blocker, and the spec **bodies** survived the rewrite
with zero EARS-keyword breakage, zero dropped AC/test rows, and no new in-spec
contradictions.

But the two **clean-room blockers (10 and 11)** are *not* fully closed, and the way they
fail is self-contradictory against the very enforcement the fix pass added. Gate 16
(`SPEC-gate`) mandates a **zero-violation self-test over the current `docs/specs/` tree** as
the green light for the first ticket — and that self-test provably **cannot pass today**
because two specs carry denylisted identifiers inside their own `## Public surface` section,
one `spec_source` cites fork architecture file paths, and six `spec_source` values point at
behavior docs that **do not exist on disk**. Gate 16 must be green before any ticket opens;
on the current tree it exits non-zero. That is a contradiction in the contract set at the
gate-before-first-ticket, which is exactly what NOT_READY is for.

Separately, the new `SPEC-asset-foundation` changed the central asset return type to
`AssetState<T::Output>` while its binding contract (`seam-contracts §10.1`) and its sole
consumer (`SPEC-assets`) still say `AssetState<T>` — a cross-spec type contradiction that
will not compile together. This re-introduces precisely the boundary mismatch the round-2
reconciliation existed to eliminate.

**4 surviving blockers.** None individually huge — all are well-scoped, mechanical fixes —
but each is a today-contradiction in the contract set, and the standard for this gate is the
strictest. Close the four, re-run the gate-16 self-test to green, and this flips to GO /
GO_WITH_FIXES quickly.

---

## Blocker-closure table (the original 11)

| # | Round-1 blocker (subject) | Status | Evidence |
|---|---|---|---|
| 1 | editor must import shared offsets, own char/byte seam (no local newtype) | **CLOSED** | editor consumes `text-offsets` `CharOffset`/`ByteOffset`, declares none; `Buffer` owns `char_to_byte`/`byte_to_char`. |
| 2 | terminal `SessionId` ownership vs `marley_core` | **CLOSED** | terminal imports `marley_core::SessionId`; distinct `ShellSessionId` correlation-only; `Block.session_id: Option<marley_core::SessionId>`. |
| 3 | hook decode split (stateless `decode_hook` / stateful `apply_hook`) | **CLOSED** | §4.1 split present with `DecodeError`/`ApplyHookError` + `encoding_for_dcs_terminator`. |
| 4 | syntax binds editor/languages/text-offsets seam types | **CLOSED** | syntax consumes `BufferDelta`/`Rope`/`Point` from editor, `IndentUnit` from `marley_languages`. |
| 5 | `marley_command` non-PTY only; terminal owns PTY | **CLOSED** | process-command dropped its PTY claim + `pty_shell_spawn_seam` test; terminal owns PTY via `alacritty_terminal::tty`. |
| 6 | input-classifier R12/R13 disjoint regimes | **CLOSED** | classifier R12/R13 are disjoint threshold regimes per §9. |
| 7 | asset foundation crate unspecified (M1 gated on undefined base types) | **CLOSED** (spec authored) | `SPEC-asset-foundation` exists, owns `Asset`/`AssetCache`/`AssetSource`/`AssetState`/`AsyncAssetType`. *(Manifest + type-param defects below.)* |
| 8 | `marley_util` §8 value vocabulary unspecified | **CLOSED** (spec authored) | `SPEC-marley-util` pins `FileId`/`ContentVersion`/`HostId`/`StandardizedPath`/`LocalOrRemotePath`/`standardize_path`. *(R11 testability defect below.)* |
| 9 | gate-15 visual harness contract missing | **CLOSED** (spec authored) | `visual-testing.spec.md` exists: AX assertion API + screenshot diff + baseline/approval + headed driver; quality-bar gate 15 link resolves. *(dssim license defect below.)* |
| 10 | clean-room `spec_source` must repoint to a behavior-level wall, not a fork transcription | **STILL OPEN** | foundation-spike `spec_source` cites `docs/warp_architecture/subsystems/01-…md` + `03-…md` (fork paths); six specs cite `*.behavior.md` files that do not exist on disk. See B2, B3. |
| 11 | clean-room gate (gate 16) must mechanically wall fork identifiers out of public surfaces | **STILL OPEN** | gate-16's own mandated zero-violation self-test is unsatisfiable on the current tree (denylist tokens inside `## Public surface` of `SPEC-completions` and `SPEC-gate`). See B1. |

**Closed: 9/11. Still open: 10, 11.**

---

## Surviving BLOCKERS (must close before the first ticket)

### B1 — Gate-16 self-test is unsatisfiable against the tree it must bless *(closes 11)*
`SPEC-gate` R4 flags any whole-word denylist token anywhere in the `## Public surface (the
contract)` slice (R7 explicitly scans fenced code and quoted literals); R8 only exempts whole
**other** sections, never sub-blocks within Public surface. R6 suppresses only the R5
fork-name path, **not** R4 denylist hits.
- `SPEC-completions.spec.md` lines 137–142 — the "Removed for clean-room posture A"
  blockquote (`LiteCommand`/`ParsedToken`/`ParsedExpression`/`ClassifiedCommand`/`classify_command`)
  sits **inside** the Public-surface section (42→143; next H2 at 144). → R4 fires.
- `SPEC-gate.spec.md` line 29 — its own Public-surface section (25→49) enumerates the full
  denylist as string literals (`"LiteCommand"`…`"enum_iterator::cardinality"`, ~14). The prose
  at line 26 *claims* these "appear only as denylist string data," but **no EARS clause
  implements that exemption**. → R4 fires on the gate itself.

So `SPEC-gate` Test Plan ("report ZERO violations / exit 0 on the current clean tree", lines
~106/108) cannot be green — the gate fails its own contract. **Fix:** (a) move the
`SPEC-completions` blockquote out of Public surface into its Clean-room provenance section
(where R8 already exempts it); (b) add a real EARS clause + fixture exempting double-quoted
string-literal denylist *data* (or relocate `SPEC-gate`'s literal list to a non-surface
section); (c) reconcile the self-test fixtures to the corrected behavior. **Then run the gate
and confirm exit 0.**

### B2 — `foundation-spike` `spec_source` cites fork architecture file paths *(part of 10)*
`SPEC-foundation-spike.spec.md` line 10: `spec_source: net-new (INVENT) — behavior framed …
from the behavior-level subsystem notes docs/warp_architecture/subsystems/01-ui-framework-rendering.md
and subsystems/03-terminal-session-core.md`. quality-bar gate-16 §2 (line 48) mandates **no
fork file paths**, and `SPEC-gate` R9 flags a fork file path. Either the gate flags this
(self-test fails, compounding B1) or `transcription_source_patterns` only matches
`warp_architecture/crates/*.md` and a `warp_architecture/subsystems/*.md` source slips through
the very gate built to stop it. **Fix:** rewrite to an inline behavior-only statement
(observable I/O, no `warp_architecture` path) as the other INVENT specs do; broaden
`transcription_source_patterns` from `warp_architecture/crates/*.md` to all
`docs/warp_architecture/**`.

### B3 — Six `spec_source` values point at behavior docs that do not exist *(part of 10)*
Verified absent: `docs/specs/behavior/`, `docs/specs/fork-reference/`,
`docs/marley_architecture/behavior/` — no `*.behavior.md` anywhere under `docs/`. The six
danglers:
- `SPEC-app-shell` → `docs/specs/behavior/app-shell.behavior.md`
- `SPEC-editor` → `docs/marley_architecture/behavior/editor-behavior.md`
- `SPEC-input-classifier` → `docs/specs/behavior/input-classifier.behavior.md`
- `SPEC-markdown-render` → `docs/specs/behavior/markdown-render.behavior.md`
- `SPEC-marley_core` → `docs/specs/behavior/marley-core.behavior.md`
- `SPEC-syntax-highlight` → `docs/specs/fork-reference/syntax-highlight.behavior.md`

Blocker-10's fix was to author a real walled-off behavior doc and cite **that**; the citation
exists, the artifact is fictional, so the behavioral wall under every "behavior-derived from a
fork-reference doc" downgrade is unverifiable. Gate 16 (R9 = string-pattern only) cannot catch
this — a Marley-namespaced dangling path passes silently, so quality-bar §2 ("every
`spec_source` points at a behavior-level doc") is only partially mechanized.
(The `fork-reference/` dir name itself signals fork derivation.) **Fix:** author the six
walled-off behavior docs (observable-I/O only) **or** convert these six `spec_source` fields
to the inline behavior-only form the other ~11 specs already use; add an existence (+
best-effort behavior-level content) check to gate 16 so a missing behavior doc fails closed.

### B4 — Asset return-type contradiction across three binding docs *(regression in the 7-fix)*
- `SPEC-asset-foundation.spec.md` line 83: `pub fn get<T: Asset>(&self, source) -> AssetState<T::Output>`
  (line 94: "AssetState is generic over the decoded Output, not over the Asset type").
- `standards/seam-contracts.md` §10.1 line 381: `get<T: Asset>(&self, source) -> AssetState<T>`.
- `SPEC-assets.spec.md` line 26: `load_asset_from_url<T: Asset>(…) -> AssetState<T>`.

`marley_assets` delegating to `AssetCache::get::<T>()` receives `AssetState<T::Output>` but
must return `AssetState<T>`; these do not unify unless `T == T::Output`, so foundation and its
only consumer will not compile together. **Fix (adopt the better foundation design as
canonical):** update `seam-contracts §10.1` line 381 to `-> AssetState<T::Output>`, and change
`SPEC-assets` `AssetCacheExt::load_asset_from_url` return + AC row 15 to `AssetState<T::Output>`.
Grep every reference and confirm all read `AssetState<T::Output>`.

---

## Remaining must-fix (deduped, non-blocking HIGH/MEDIUM — fold before/at ticket)

**HIGH**
1. **dssim (AGPL) in a binding `reuses:` field** — `visual-testing.spec.md` line 9
   `reuses: [… dssim]`. The body (lines 406–410) flags it as AGPL and "a placeholder to be
   pinned," but R17/`Tolerance::gate15_default` make `perceptual_mean` a hard AND-term, so
   `compare()` cannot be built to spec until a permissive metric is chosen — and the binding
   `reuses` field (which feeds `deny.toml` allowlist generation, gate 8) still names a copyleft
   crate **in the spec that defines the license/clean-room gate harness**. Fix: drop `dssim`
   from frontmatter now, pin MIT/Apache (e.g. `deltae`+`lab` ΔE2000 mean, or a permissive
   `dssim-core` release), state it in frontmatter + Dependencies, confirm gate 8 passes.
2. **gate 6 (miri) vs harness objc2 FFI** — quality-bar gate 6 (line 16) is "mandatory for …
   objc2 interop"; the new `marley_visual_harness` carries first-party objc2/core-foundation
   AXUIElement unsafe FFI and self-declares miri N/A. Two binding docs disagree, unreconciled.
   Fix: amend gate 6 with an explicit system-framework-FFI carve-out (mirroring the spec), or
   put the unsafe AX calls behind a thin shim so miri runs the surrounding pure logic; make the
   two texts agree.
3. **R5 allowlist completeness makes the self-test fragile** — `SPEC-gate` R5 fires for any
   public-surface token in harvested `fork_doc_identifiers` AND absent from `allowlist`, but
   gives only ~8 allowlist examples and no completeness mechanism. Shared names (`SessionId`,
   `Command`, `Buffer`, `Range`, `FormattedText`…) must each be enumerated or R5 fires and the
   zero-violation self-test cannot be green. `FormattedText` (kept verbatim across
   `SPEC-markdown-render` Public surface, flagged in §11.1 line 426) is not enumerated. Fix:
   seed `allowlist` from the union of all Marley public-surface identifiers across the spec set
   (or invert R5 to denylist-only), and explicitly bless retained-but-fork-derived names
   (starting `FormattedText`) with a §11.1 "keep" row.
4. **README manifest omits required first-round tickets** — `README.md` line 64 "Total: 16
   specs," line 83 "These 15 specs are the first round of forge tickets," but there are 18
   `SPEC-*.spec.md` + `visual-testing`. `SPEC-asset-foundation` (`marley_asset_core`, the
   Blocker-7 fix that **must build before `marley_assets`**) and `SPEC-gate` are absent from the
   index and build-order. One-ticket-per-row generation would miss the asset foundation crate
   and its ordering, the gate-16 crate, and the gate-15 harness crate. Fix: add
   `SPEC-asset-foundation` (before `SPEC-assets`), add `SPEC-gate` + `visual-testing` as
   M0 infra rows, correct counts to 18 and the "15 specs" line.
5. **`marley-util` `standardize_path` R11 untestable on the only target platform** —
   `standardize_path(p: &std::path::Path)` canonicalizes to "the host flavor"; Marley is
   macOS-only at M0/M1, so host flavor is always Posix, `\` is a legal Unix filename byte (not a
   separator), and `StandardizedPath::Windows` is unconstructable (the only ctor + `#[non_exhaustive]`).
   R11's "foreign separator replaced" and the Test Plan's "host-flavor-explicit / any platform"
   claim contradict the host-only `&Path` signature. Fix: add
   `standardize_path_with_flavor(input: &str, flavor: PathFlavor)` (or accept a typed Unix/Windows
   path) so both branches are deterministic on a macOS runner and the Windows variant is
   constructable; re-scope R11; or drop the Windows variant if cross-host paths are out of M1.
6. **visual-testing leaves 2 EARS clauses with no test** — R5 (`LaunchTimeout`) and R25
   (`xtask visual review` lists pending, no auto-accept) have no named test and are not in the
   ACCEPTED-UNTESTABLE block, violating the bar's every-clause-maps-to-a-test rule *inside the
   spec that satisfies gate 15*. Fix: add a headed test for R5 (or move it to ACCEPTED-UNTESTABLE
   with justification), add a unit test for R25; re-audit all 27 clauses.

**MEDIUM**
7. **editor↔syntax `BufferVersion` second source of truth** — `SPEC-syntax-highlight`
   `apply_edit(&mut self, &Rope, &BufferDelta)` consumes only the delta then mints its own
   version via `BufferVersion::next` (R19, "shadow version"). The editor's authoritative
   `BufferVersion` (§3.2) is never threaded across the seam, so syntax's `current_version()` /
   `DecorationEvent.version` / `invalidate_cache_for_version` equal the editor's only by
   coincidence — version-keyed cache invalidation is unsound. Fix: thread the editor's
   `BufferVersion` through `apply_edit` (take `&BufferDelta` + `BufferVersion`, or the
   `BufferEvent::Edited{delta,version}`) and set `current_version()` to it.
8. **`asset-foundation` `ImmediateScheduler` test double contradicts R1's no-caller-I/O
   assertion** — the double "drives the scheduled future to completion synchronously inside
   `schedule()`," but R1 + `r1_fresh_get_returns_loading_sync_no_caller_io` assert neither
   embedder nor loader ran on the calling thread before `get` returned. Fix: make it a
   manual-pump queue (`schedule()` enqueues; test calls `drive()`/`run_pending()` between
   `get`s); keep the real two-thread scheduler for R12.
9. **`FormattedText` verbatim fork type name unresolved** — `SPEC-markdown-render` keeps
   `FormattedText` across its Public surface; not on the gate-16 denylist (line 47), so R4 never
   flags it; §11.1 line 426 leaves it ambiguous. For posture A the keep-decision must be
   explicit. Fix: either bless it (§11.1 "keep, Marley-original-by-adoption" row + allowlist
   entry + one-line rationale) or de-Warp the type name (e.g. `RenderedText`/`DocModel`).
10. **README EARS-count column stale for 7 specs** (text-offsets 20→22, foundation-spike
    15→18, editor 23→27, terminal-blocks 19→23, syntax-highlight 18→20, ui-components 16→17,
    assets 23→21) and the "306 EARS / 16 specs" headline wrong on both counts. Recompute from
    actual R-counts (incl. asset-foundation, gate).
11. **seam-contracts §2 `Block.session_id`→`marley_app` is forward-looking with no M1 adopter**
    — `SPEC-app-shell` defers all session/pane wiring to M2; mark §2's binding as M2 (mirror
    §7's M2-merge convention) so it is not read as an unmet M1 seam.

**LOW**
12. **`SPEC-marley-util` stray tool-call tags** — lines 138–139 contain literal `</content>`
    and `</invoke>` after the real EOF (Clean-room provenance paragraph). Delete them; they
    pollute a file the gate-16 scanner walks (extract-to-next-H2/EOF) and fail any "no junk in
    committed docs" check. **(Verified present.)**
13. **seam-contracts §10.1 `get` text is itself type-wrong** — it literally writes
    `-> AssetState<T>` (`T: Asset`); the owner spec correctly refines to `AssetState<T::Output>`.
    Per the contract's "this doc wins" rule, the divergence reads as a spec flag when the spec is
    actually correct. Folds into B4: update §10.1 to `AssetState<T::Output>`.
14. **`marley-util` `dirs` dependency vs R9 cwd** — R9 resolves against the process cwd
    (`std::env::current_dir()`); `dirs` resolves OS base dirs (home/config/data), not cwd. Drop
    `dirs` and state cwd uses `std::env::current_dir()`, or name the base-dir requirement
    explicitly.
15. **visual-testing `HarnessError::WindowNotUnique` misleading** — `window()`/`find()` return
    `AxQuery` (not `Result`), so they cannot return the `WindowNotUnique` variant listed in
    `HarnessError` (returned only by `mount`). Clarify fail-closed = panic, and move
    `WindowNotUnique` out of `HarnessError` (or note it is the panic message).
16. **gate triad false-positive risk** — `SPEC-gate` denylist bans bare `Component`/`Params`/
    `Options`; R7 correctly protects camelCase composites today, but a future standalone
    capitalized `Options` would false-positive. Tighten to trait-bound/impl context (or rely on
    R6 allowlisting as uses arise).
17. **README provenance line 9 stale** — still reads "spec'd from `docs/warp_architecture/…`,"
    contradicting the clean_room downgrade + the `spec_source` repointing. Rewrite to
    "behavior-derived fork-reference doc; IP-counsel sign-off pending; spec_source is
    behavior-level."

---

## Fold-into-ticket list (carry into the relevant forge ticket)

- **`marley_asset_core` / `marley_assets` tickets:** B4 type unification (`AssetState<T::Output>`
  everywhere) + §10.1 §13 contract edit; MEDIUM-8 manual-pump test double; README-row + build-order
  (HIGH-4); §10.1 line-381 edit (LOW-13).
- **`marley_spec_provenance` (gate) ticket:** B1 self-test reconciliation (string-literal-data
  exemption clause + fixture); B2 `transcription_source_patterns` broaden to
  `docs/warp_architecture/**`; B3 existence-check for `spec_source`; HIGH-3 allowlist completeness
  + `FormattedText` keep-row; LOW-16 triad tightening.
- **`marley_visual_harness` ticket:** HIGH-1 pin permissive perceptual crate + drop `dssim` from
  `reuses`; HIGH-2 gate-6/miri carve-out or shim; HIGH-6 R5/R25 tests; LOW-15 `HarnessError` scope.
- **`marley_util` ticket:** HIGH-5 `standardize_path_with_flavor` (or scope out Windows variant);
  LOW-12 strip stray tags; LOW-14 drop `dirs`.
- **`marley_syntax_highlight` ticket:** MEDIUM-7 thread editor `BufferVersion` through `apply_edit`.
- **`marley_markdown` ticket:** MEDIUM-9 resolve `FormattedText` keep-vs-rebrand.
- **README / manifest housekeeping (pre-ticket):** HIGH-4 + MEDIUM-10 counts/rows/build-order;
  LOW-17 provenance line; MEDIUM-11 §2 M2 annotation.
- **Six dangling behavior docs (B3) + foundation-spike (B2):** author the walled-off behavior docs
  or inline the prose — pre-ticket, since gate 16 must resolve every `spec_source`.

---

## Per-dimension summary

| Dimension | Verdict | Headline |
|---|---|---|
| **Blocker closure** | 9/11 closed | 1–9 genuinely closed; 10 & 11 (clean-room spec layer) not fully closed — gate-16 self-test would fail on the current tree. |
| **Seam conformance** (vs seam-contracts) | Strong, no blockers | Every blocker-class seam has one owner; consumers import. Residual: B4 asset type-param (cross-doc), MEDIUM-7 version threading, two LOW reconciliations. |
| **New-spec quality** (util / asset-foundation / visual) | High-craft, 3 residual defects | Each closes its blocker; but B4 type mismatch, HIGH-5 R11 untestability, HIGH-1 dssim license unpinned. |
| **Clean-room re-check (posture A)** | **3 blockers** | B1 gate self-test unsatisfiable; B2 fork-path `spec_source`; B3 six fictional behavior docs. Gate-16 mechanism contradicts the spec set it must pass; the behavioral wall does not physically exist. |
| **Regression check** | Bodies clean | Zero broken EARS, zero dropped AC/test rows, no lost visual/AX acceptance, no new in-spec contradictions. Damage is metadata-only: README counts/rows + dangling `spec_source` + stale provenance line. |
| **Standards compliance** (test/mutation/visual-AX/gates 15–16) | Needs-work | gate-15/16 now bind to real contracts; but HIGH-6 two unmapped clauses + HIGH-2 gate-6/miri contradiction + HIGH-1 dssim in binding `reuses`. |

**Bottom line:** the fix pass closed 9 blockers cleanly and the seam wall is now coherent —
this is close. But the clean-room layer (blockers 10/11) reopened as four today-contradictions
that block the gate-16 self-test the project itself made the green light for ticket 1. Close
B1–B4, re-run `scripts/spec-provenance.sh` to exit 0, and fold the HIGH items — then this is
GO_WITH_FIXES at worst.
