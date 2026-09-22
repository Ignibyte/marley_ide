# 394-content-registry — Notes

## Phase 1 — Plan (drafted 2026-07-22, /spec batch, Fable)
- **Request:** start the #388 train's spine — the ContentId registry + terminals-first migration.
  Sprint #38 M27; forge #394 `adf0b9d6-08e1-43c6-8c7c-26c48670c2a5`.
- **The source of truth is the #388 doc** (pane-composition-model.md, decision-complete to the D5
  standard) — P2 begins by re-reading it, especially: the central-fork rationale (own registry, NOT
  gpui Entity), the refcount catch (PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-
  001 — recorded at #388 inspect), the migration-ripple table (close-teardown row; the EditorSurface
  birth sites are LATER slices), and the 8-slice ordering.
- **Fusion call (D5):** the doc's slices 1 (registry) + 2 (terminals) are fused here conservatively —
  a registry with zero residents would be dead-code-adjacent; terminals are the natural first
  resident (already semi-handle-based via the PaneId side-tables). If P2 sizes it past one shippable
  slice, SPLIT sequentially and surface it.
- **Hard invariants:** codec untouched (ids never persist — restore rebuilds); reaper semantics
  byte-identical (every view is 1-view until add-to-pane exists); zero visible change (the suite is
  the proof).
- **Sequencing:** independent of #390-393 (the display half deliberately shipped on coordinates);
  the NEXT train slices (add-to-pane, one-instance-many-views, arrangements, Editor migration /
  the #259 two-Buffers collapse) consume this.
- **Prior-art sweep:** the #388 doc + the in-repo side-table pattern; gpui Entity evaluated-and-
  rejected there (adoption read). Recorded in spec.
- **AAR:** opened at promotion.

## Phase 1 — Promote to active (Opus, 2026-07-22, /goal /work 390-395)
Promoted queued → `active/` (FIFTH + LAST of the numbered sprint; #390-393 all shipped LOCAL; §3
re-confirmed — 394 is the sole active spec). **AAR opened:** `9dc2d03c-82d0-4c65-8a04-1d66055cf754`.

- **⚠️ THE SPRINT'S BIGGEST/RISKIEST TICKET — the D5 sizing call is the design's FIRST act.** This is a
  structural refactor of pane-CONTENT ownership (not display, like #390-393). The #388 train had
  "registry" (slice 1) and "terminals-migrate" (slice 2) as SEPARATE slices; the spec fuses them
  conservatively. P2 MUST re-read `pane-composition-model.md` and size the terminal-migration ripple —
  it touches `PaneContent`/`PaneGrid` ownership, `focused_terminal`/`states`/`split_focused`/`close_pane`,
  the PTY reaper contract, the #163/#205 persistence rebuild, and every `workspace()`/`focused_terminal`
  read. **If it's past one SAFE slice, SPLIT (registry-only first, terminals-migrate second as a follow-up
  ticket, the #392→#395 pattern). A split leaving the reaper/persistence half-migrated is FORBIDDEN.**
- **Recall for the designer (knowledge-search "content registry refcount drop-on-last-close ownership
  migration reaper"):** pull via `knowledge-explain` in P2 — prevention rules
  `452ae5ee-c7ee-4117-bf77-4b208c279f8c`, `7f19c7c9-e40e-4d05-b452-2825ce593498`,
  `71b36786-36e4-45db-b39e-be3e07b28ad2`, `e331b13c-6b9e-452c-ae76-3fb11a4cc407` + AD
  `fde3f2a1-fadf-4563-877a-344d7d530d37` (refcount/ownership/lifecycle traps) — feed them to the
  leak-path + reaper-timing inspect critics. The BINDING one is already known:
  `PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001` (#388 inspect — a bare HashMap
  does NOT refcount).
- **Hard invariants (P2/P3 must not break):** the codec NEVER learns ids (restore rebuilds registry
  entries from the restored shapes — #163/#205/#177 round-trips byte-identical); the reaper fires
  byte-identically to today for the 1-view case (every view is 1-view until add-to-pane exists,
  a later slice); ZERO user-visible change (the full 834-test suite is the proof).
- **Behavior-neutral → NO driven capture** (state N/A explicitly at validate — a structural refactor
  renders identically; the suite + the pure registry unit matrix carry it).

## Phase 2 — Design (Opus, 2026-07-22, /goal /work 390-395)

### (0) THE D5 SIZING CALL — **SPLIT** (decided from the evidence)
Re-read `pane-composition-model.md` IN FULL + fanned an Explore over the pane-ownership code. **The
evidence is unambiguous — SPLIT:**
- **The #388 doc's own Q5 train separates them:** slice 1 = "`ContentId` + `ContentRegistry` — the pure
  registry + id allocation, **wired but UNUSED (additive; cov/MSI 100). No behaviour change. (M)**"; slice 2
  = "**Terminals onto the registry** … (L)". They were NEVER one slice; the queued spec fused them
  "conservatively" and flagged D5.
- **The Q3 ripple table + the Explore inventory size the terminal migration as L/multi-day/high-risk:**
  `PaneContent<S>::Terminal(Box<TerminalPane<TerminalSession>>)` ownership change (2× **L** rows) +
  **~46 app.rs accessor sites** (`focused_terminal`/`_mut` ×~40 + `states`/`_mut` ×6) + **7 spawn-drop
  close paths** (5 user + 2 boot) EACH of which must preserve the ~600ms off-thread `thread::spawn(drop)`
  reap (a main-thread drop freezes the UI; a leaked view leaks a PTY) + the #163/#205 persistence rebuild
  + the agents/remotes re-key. Every one of those sites is a chance to leak a view or double-reap.
- **Fusing = the forbidden multi-site refactor** (a half-migrated reaper/persistence must never ship).
  → **#394 = slice-1 (the pure lifecycle); #396 `cd706b38` created for slice-2 (the ownership migration),
  depends on #394.** The #392→#395 pattern.

**The one thing that could have FORCED a fuse — can a wired-but-unused pure registry ship gate-green? —
the Explore CONFIRMED YES**, via the git-proven **#371 `orchestration_for` template**: `mod
content_registry;` (private in `lib.rs`) + **`pub use content_registry::{ContentId, ContentRegistry};`
at the crate root**. rustc's `dead_code` lint treats items reachable from a LIB's public API as used, so
a re-exported item needs NO caller (production or test). `orchestration_for` (pure, gpui-free) was added
at #371 and got its first production caller only at #376 — wired-but-unused + gate-green in between, the
exact template. **TRAP:** a bare private `mod` WITHOUT the `pub use` → crate-unreachable → `dead_code`
fires under `-D warnings` → gate-red; the fix is the re-export, not a consumer. **HARD CONSTRAINT:**
`#![deny(missing_docs)]` (lib.rs:19) + `RUSTDOCFLAGS=-D warnings` → EVERY pub item (`ContentId`,
`ContentRegistry`, every pub method) MUST carry a `///` doc.

### (1) Architecture — the pure generic `ContentRegistry<C>`
A NEW module `crates/marley_app/src/content_registry.rs`, **PURE + gpui-free** (like tabs/workspace/
grid_layout — the cov/MSI-100 seam discipline). **Generic over `C`** (the #388 open-Q2 simplification):
the refcounted lifecycle is content-AGNOSTIC, so `ContentRegistry<C>` proves it with a trivial test
double (no live `TerminalSession`/PTY needed for the unit matrix); production later instantiates
`ContentRegistry<Content>` in #396. This cleanly separates the LIFECYCLE (slice-1, fully testable) from
the CONTENT payload (slice-2). Shape:

```rust
/// A stable, session-local id for content owned once in a [`ContentRegistry`]. Monotonic; never reused
/// within a session; never serialized (the codec stays coordinate/shape-based — #388 D4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ContentId(u64);

/// A refcounted content registry (M27 #394, the #388 spine): content is owned ONCE per `ContentId`; N
/// views share it; the owned content is returned for teardown EXACTLY when the last view releases — the
/// drop-on-last-close gpui's `Entity` gives natively, hand-rolled here (a bare `HashMap` gives resolution
/// but NOT refcount — PR-…-shared-registry-needs-refcount-…-001). Generic over the content `C`.
pub struct ContentRegistry<C> { next: u64, entries: HashMap<ContentId, Entry<C>> }
struct Entry<C> { content: C, views: usize }   // views = the explicit refcount (the binding catch)

impl<C> ContentRegistry<C> {
    pub fn new() -> Self
    pub fn insert(&mut self, content: C) -> ContentId   // views = 1 (the first mount)
    pub fn acquire_view(&mut self, id: ContentId) -> Option<()>  // views += 1; None if unknown
    pub fn release_view(&mut self, id: ContentId) -> Option<C>   // views -= 1; Some(content) on last drop, else None (also None on unknown/underflow-guard)
    pub fn get(&self, id: ContentId) -> Option<&C>
    pub fn get_mut(&mut self, id: ContentId) -> Option<&mut C>
    pub fn view_count(&self, id: ContentId) -> usize   // 0 if unknown
    pub fn len(&self) -> usize
    pub fn is_empty(&self) -> bool   // clippy pairs len with is_empty
}
```
Plus `impl<C> Default for ContentRegistry<C>` (clippy new-without-default). `insert` acquires 1 view (an
inserted entry with 0 views would be instantly orphaned). `release_view` is total + safe: unknown id →
`None`; the last view → remove + return `Some(content)`; the underflow case can't arise (removed at 0).

**§14 / clean-room:** no `unwrap`/`expect` (all `Option` returns are total); no panics; gpui-free; §20
N/A (Marley-specific, gpui `Entity` evaluated + rejected at #388). No process-spawn here (the reaper stays
in the caller — #396 preserves the off-thread `thread::spawn(drop)`).

### (2) File manifest
- **`crates/marley_app/src/content_registry.rs`** (NEW) — `ContentId` + `ContentRegistry<C>` + `Entry<C>`
  (private) + the full `#[cfg(test)]` unit matrix. All pub items `///`-documented.
- **`crates/marley_app/src/lib.rs`** — `mod content_registry;` + `pub use
  content_registry::{ContentId, ContentRegistry};` at the crate root (the #371 gate-safety idiom), with
  the intent comment (mirrors lib.rs:99-105's `orchestration_for` note: "crate-public so it isn't
  dead-code before its #396 consumer lands").
- **NO other file** — dormant. `RootView` does NOT yet hold a `content:` field (that's #396, where it's
  read; an unread field would itself dead-code-warn).

### (3) Regression Test Plan (the pure matrix IS the deliverable — cov/MSI 100)
| REQ | Test (`content_registry::tests`, using a drop-tracking double for `C`) | Kind |
|---|---|---|
| REQ-001 | `insert` → a `ContentId`, `view_count == 1`, `get` returns the content; two inserts → DISTINCT monotonic ids | pure unit |
| REQ-002 | `acquire_view(live)` → `view_count == 2`; `acquire_view(unknown)` → `None`, no mutation | pure unit |
| REQ-003 | insert (1) → acquire (2) → `release_view` once → `None` + still `get`-able (`view_count 1`); `release_view` again → `Some(original_content)` (assert the RETURNED value) + `get` now `None` + `len 0` | pure unit (the keeps-alive + drop-on-EXACTLY-last + returns-the-content core) |
| REQ-003 | `release_view(unknown)` → `None`; `release_view` past the last (double-release) → `None`, no panic/underflow | pure unit (safety) |
| REQ-004 | `get`/`get_mut`/`view_count`/`len`/`is_empty` on live vs unknown | pure unit |
| REQ-005 | the FULL existing suite (834) passes byte-identical; the gate stays `-D warnings` green (the `pub use` proves non-dead-code) | `cargo nextest` + `scripts/gates.sh --diff` |

**Mutation:** trace `cargo mutants --list -f content_registry.rs` — expect the refcount arithmetic
(`+= 1`/`-= 1`, `next` bump), the `== 0`/`>`-boundary on the last-drop, and the `Option`/`usize` returns.
Kill each with the matrix (esp. the 2-views-release-one asserts the boundary is EXACTLY-last, not
off-by-one). **NO driven capture** — a dormant pure module renders nothing (N/A, stated).

### (4) Risks / decisions
- **D5-SPLIT** (above) — the load-bearing call; #396 created + sprint-added + TICKET-396 written.
- **D-REGISTRY-GENERIC** — `ContentRegistry<C>` over a concrete `ContentRegistry<Content>`: the lifecycle
  is content-agnostic, so genericity gives a PURE, live-PTY-free unit matrix at cov/MSI 100 AND defers the
  `Content` enum to #396 (where the payloads become real — declaring a 6-variant enum with live payloads
  that nothing constructs, purely to satisfy the old REQ-005 in slice-1, is form-over-substance and risks
  the gpui-free purity if a payload type isn't pure). REQ-005 (Content enum) → #396.
- **D-PUBUSE-GATE-SAFETY** — the `pub use` re-export (the #371 template) is REQUIRED, not cosmetic: without
  it the module is dead-code → gate-red. The inspect must confirm its presence.
- **`insert`-acquires-1-view** vs insert-then-acquire — insert = the first mount, so 1 view is the natural
  invariant (0 would orphan). Recorded.
- **Recall pulled** (the flagged PRs 452ae5ee/7f19c7c9/71b36786/e331b13c + AD fde3f2a1 cluster on
  refcount/ownership/lifecycle) → the inspect critics: refcount-underflow/off-by-one, drop-returns-the-
  right-content, double-release safety, the `pub use` gate-safety, missing-docs completeness.

## Phase 3 — Implement (Opus, 2026-07-22, /goal)
Built per the manifest; `cargo check -p marley` + `cargo clippy --all-targets` **clean (NO dead_code, NO
missing_docs)**, `cargo fmt` clean, the full suite **834 pass byte-identical** (additive/dormant module,
no new tests — those are Phase 4). No deviations from design.

- **`content_registry.rs` (NEW, PURE, gpui-free):** `ContentId(u64)` newtype (Copy/Hash/Ord) + the generic
  `ContentRegistry<C>` (`next: u64` + `entries: HashMap<ContentId, Entry<C>>`, `Entry<C>` private with the
  explicit `views` refcount) + `new`/`insert`(views=1)/`acquire_view`(+1, None-if-unknown)/`release_view`
  (−1, `Some(content)` on last-drop via `get_mut(&id)? ; views -= 1; if 0 remove+return else None` — no
  underflow path)/`get`/`get_mut`/`view_count`/`len`/`is_empty` + `impl Default`. Every pub item `///`-doc'd
  (the `#![deny(missing_docs)]` constraint). No `unwrap`/`expect`/panic (all `Option`-total; the one `?` is
  on a `get_mut` Option, not an input path).
- **`lib.rs`:** `mod content_registry;` (alphabetical, after `mod complete;`) + `pub use
  content_registry::{ContentId, ContentRegistry};` at the crate-root re-export block (beside
  `orchestration_for`), with the intent comment. **This `pub use` is load-bearing — the #371 gate-safety
  idiom:** it makes the wired-but-unused registry crate-public-API → NOT dead-code under `-D warnings`
  (confirmed: clippy clean). A bare private `mod` would have been gate-red.
- **Verified the Explore's prediction held:** a pure, wired-but-production-unused registry ships gate-green
  via the re-export (no consumer needed) — exactly the `orchestration_for` (#371→#376) precedent.

## Inspect (Phase 3.5) — 2026-07-22 (Opus, /goal)
Two focused critics over the small diff (content_registry.rs new + lib.rs +2). Lenses: (1) refcount-lifecycle
correctness (the drop-on-last-close boundary — a wrong boundary = a leaked PTY or a premature reap), (2)
gate-safety + §14 purity + simplification. **1 LOW fixed; 1 LOW informational (not fixed, agreed); the
correctness boundary + gate-safety both CLEAN.** cargo clippy/doc/fmt clean, 834 tests byte-identical.

**F1 [LOW, CONFIRMED, FIXED] — `release_view`/`acquire_view` were not `#[must_use]`.** (Critic 1.) The whole
point of `release_view` returning `Some(content)` on the last drop is that the caller hands it to an
OFF-THREAD reap (the ~600ms PTY teardown — #396); but `reg.release_view(id);` (result ignored) compiles and
drops the returned content INLINE on the calling thread, silently defeating the off-thread intent. Likewise a
dropped `acquire_view` `Option<()>` hides a stale-id no-op (caller believes it holds a view it doesn't). No
callers exist yet (wired-but-unused), so nothing manifested — but this is a footgun the #396 wiring would trip.
**Fix:** `#[must_use = "…must be handed to teardown…dropping it here tears down on the calling thread"]` on
`release_view` + `#[must_use = "…hides a stale-id bug"]` on `acquire_view`. Cheap compile-time guard for the
consumer; clippy/tests still clean (no callers → no warning fires; the Phase-4 matrix uses the returns). →
`PR-claude-registry-release-returning-a-resource-must-be-must-use-001`.

**F2 [LOW, informational — NOT fixed, both critics agree].** `next: u64` / `views: usize` overflow (2^64
inserts/views) → a wrapped `next` could collide a live key (leaked content) / underflow. Physically
unreachable (2^64 ≈ 585,000 years at 1M ops/s); a guard would be DEAD CODE. Noted, not fixed.

**Clean (verified concretely):** the refcount boundary is AIRTIGHT — Critic 1's exhaustive trace confirmed the
invariant "an entry is in the map only with views ≥ 1" (insert=1; acquire=+1; release=−1-then-remove-at-0; the
only decrementer removes at 0; `get_mut` exposes `&mut C` not `&mut Entry`, and `Entry`/`views`/`entries`/`next`
are private → external code CANNOT zero `views`), so `views -= 1` never underflows; double-release → `get_mut`
None → `?` → None (no panic); drop-on-EXACTLY-last (returns `Some` only when pre-decrement was 1); the returned
content is the moved-out original (no `Clone` bound). Gate-safety: the `pub use` re-export is present + proven
non-dead-code under `-D warnings` (Critic 2, recompiled 24.4s — not stale); all 13 pub surfaces `///`-doc'd
(missing_docs green under `-D warnings`); panic-free (the one `?` is on a `get_mut` Option); gpui-free (only
`std::collections::HashMap`); both `impl<C>` UNBOUNDED (so `ContentRegistry<Content>` compiles in #396 even
though Content isn't Clone/Default); the lib.rs diff surgical (alphabetical `mod` + the #371-idiom `pub use`).

`status: Phase 3.5 — Inspect PASS; ready for Phase 4 — Validate`.

## Phase 4 — Validate (Opus, 2026-07-22, /goal) — GATE GREEN [diff]

**Tests added (6, `content_registry::tests` — the pure matrix IS the deliverable, cov/MSI 100):**
- `insert_one_view_and_a_fresh_id` (REQ-001) — insert → view_count 1 + get; two inserts → distinct
  monotonic ids (kills the `next` `+=`→`*=` id-reuse mutant).
- `acquire_view_live_and_unknown` (REQ-002) — increment on live (view_count 2, kills `+=`→`-=`/`*=`);
  unknown → None + no insert-on-miss (kills the `-> Some(())` always-some mutant).
- `release_drops_on_exactly_last_returning_the_content` (REQ-003, THE boundary) — 2-view content
  SURVIVES the first release (None + still get-able → kills the `== 0`→`!=` boundary mutant that would
  drop while a view remains); the LAST release returns `Some(42)` the ORIGINAL (kills the
  `Some(Default::default())`=`Some(0)` body mutant) + removes the entry (len 0 kills `-=`→`+=`/`/=`).
- `release_unknown_and_double_release_are_none_safe` (REQ-003 safety) — unknown → None; double-release
  past the last → None, no panic/underflow.
- `get_and_get_mut_resolve_and_mutate` (REQ-004) — get live/unknown; get_mut mutates the REAL entry
  (kills the leaked-default get_mut mutant).
- `default_is_the_empty_registry` — covers the clippy-required `Default` impl (the coverage gap below).

**Runs (actual):**
- `cargo nextest run -p marley` → **840 passed, 2 skipped** (834 + the 6 new).
- `scripts/gates.sh --diff` → **GATE GREEN [diff]**, 15/15. gate:4 coverage **100% lines** + **100%
  functions** on content_registry.rs (the pure seam — NOT in any exclude), gate:5 mutation **MSI 100%**
  (all 22 listed mutants killed / unviable). Receipt written.

**One coverage RED fixed at source (§0, no suppression):** the first `--diff` run failed gate:4 —
content_registry.rs at 97.09% lines (1 function + 3 lines missed) — because `Default::default()` (the
clippy-`new_without_default`-required impl) was never called (all 5 tests used `::new()`). Mutation was
already 100 (cargo-mutants doesn't mutate the trivial `Self::new()` delegation). Fix: added
`default_is_the_empty_registry` calling `ContentRegistry::default()` → 100% lines. (Lesson: an
`impl Default` added purely to satisfy `new_without_default` still needs a test that exercises
`::default()`, or it is an uncovered line that mutation won't catch.)

**Driven capture — N/A (explicitly).** A dormant, production-UNUSED pure module (its only consumer is
#396) renders nothing and changes no pixel — there is no interaction to drive. The full 834-test suite is
byte-identical + the pure matrix carries the proof. (This is the #371 `orchestration_for` wired-but-unused
shape — no capture then either.)

**Pre-existing exclusions:** none. content_registry.rs is a fully-covered pure seam (100/100); no app.rs
shim involved (dormant, no production wiring this slice).

`status: Phase 4 — Validate PASS; ready for Phase 5 — Complete`.
