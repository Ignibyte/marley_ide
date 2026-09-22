---
spec_id: command-palette
component: marley_search_core
bucket: REIMPLEMENT
milestone: M1
status: draft
title: Search mixer / command palette — fan a query to data sources, return ranked deduped results
goal: Give every palette/finder surface one engine that fans a typed query out to many registered data sources, then returns a single tier-ordered, score-ranked, deduplicated result list with per-source loading and error state, plus a standalone fuzzy candidate-ranking entry.
reuses: [nucleo, tantivy]
spec_source: "behavior-only — observable I/O of a result-mixing search engine: fan a typed query (with an optional leading filter atom) to many registered sync/async data sources, collect their scored results, dedupe by identity, order by priority tier then descending score, truncate to a limit, and track per-source loading/error; expose a standalone fuzzy candidate-ranking entry (subsequence score, non-match omitted) and an embedded full-text index for document sources. No fork file paths, no private module/type/static names. Seam ownership per standards/seam-contracts.md §7."
clean_room: "behavior-derived from a fork-reference doc; IP-counsel sign-off pending"
browser_testable: no
visual_acceptance: N/A
mutation_testing: Enabled
references:
  - ./standards/quality-bar.spec.md
  - ./standards/seam-contracts.md
---

## Purpose
`marley_search_core` is the reusable engine behind every command-palette / fuzzy-finder surface in Marley (inline command menu, repos picker, slash-commands, sessions, models, and later the `marley:` / `forge:` launchers). It provides three layered capabilities: (1) a **result-mixing framework** that fans a typed `Query` out to many registered sync/async data sources, collects scored `QueryResult<T>`s, deduplicates them, orders them by priority tier then score, and tracks per-source loading/error state; (2) a **standalone fuzzy candidate-ranking entry** (`fuzzy_rank`) that scores short candidate string lists against a query via `nucleo` subsequence matching, so ranking is itself part of the public surface and not hidden inside a caller's source; and (3) a **full-text search backend** over `tantivy` exposed through a typed schema so a source can index and query documents. Heavy document indexing is delegated to `tantivy`; fuzzy ranking of short lists is delegated to `nucleo`. The component owns no rendering surface — it produces an ordered `Vec<QueryResult<T>>` that any front-end panel observes.

### M1/M2 consumer note (palette-twice resolution — search-core half, seam-contracts §7)
At **M1, `marley_search_core` is built and tested in complete isolation: it has no palette consumer.** The whole spec is asserted through its own API in unit/integration tests against deterministic fake sources; no `marley_app` surface depends on the mixer at M1. `marley_app`'s M1 command palette is, by an explicit and **intentionally-temporary** decision (mirrored in `SPEC-app-shell` and the M1 build order), a static in-memory command list with a local `filter_commands` wrapper that ranks via the same `nucleo` primitive this crate exposes as `fuzzy_rank`; its local `ScoredCommand { score: u32 }` is local-only and is **not** a second permanent ranking engine. In **M2** that static palette migrates onto `SearchMixer`: the command list is registered as a `SyncDataSource<Command>`, unifying the palette on this crate's `QueryResult<T>` rows and `f32` scores. Until that M2 migration lands, the two ranking paths coexist by design — this crate is the canonical engine, app-shell's static list is the declared temporary divergence. `SearchMixer` therefore intentionally has **no M1 consumer**, and that is asserted by this spec, not a coverage gap.

## Public surface (the contract)
All in `crates/marley_search_core/src/`. All identifiers are Marley-original; the `Action` payload trait, the `QueryResult`/`SearchMixer`/`fuzzy_rank` surface, and the filter-atom vocabulary are this crate's own names (seam-contracts §7, §11.1).

```rust
// ---- action payload vocabulary (data_source.rs) ----
/// The payload a result row carries — what accepting a row "does".
/// This is the central bound every result/source/mixer type is generic over.
/// Bounds are pinned here so no consumer re-declares them: `Clone` (rows are
/// cloned out via `accept_result`) + `'static` (rows outlive a single query frame).
pub trait Action: Clone + 'static {}

// ---- identity & query (data_source.rs) ----
pub struct DataSourceId(u64);
impl DataSourceId { pub fn new() -> Self; } // process-unique, monotonic

pub struct Query {
    pub text: String,                 // the raw user input minus any leading filter atom
    pub filter: Option<QueryFilter>,  // parsed leading filter, if present
    pub limit: usize,                 // max results the mixer returns (post-merge)
}
impl Query {
    pub fn parse(raw: &str, limit: usize) -> Self; // splits a leading `atom:` off `raw`
}

// Lower tier number ranks first; within a tier, higher score ranks first.
pub struct PriorityTier(pub u8);

pub struct QueryResult<T: Action> {
    /* opaque */
}
impl<T: Action> QueryResult<T> {
    pub fn new(identity: String, tier: PriorityTier, score: f32, action: T) -> Self;
    pub fn identity(&self) -> &str;     // dedup key
    pub fn score(&self) -> f32;
    pub fn priority_tier(&self) -> PriorityTier;
    pub fn accept_result(&self) -> T;   // clones the row's action
    pub fn source_id(&self) -> DataSourceId;
}

// ---- filter taxonomy (data_source.rs) ----
pub enum QueryFilter { History, Workflows, Sessions, Commands, Marley }
pub struct FilterAtom { /* canonical + aliases */ }
impl QueryFilter {
    pub fn all() -> &'static [QueryFilter];
    pub fn from_atom(atom: &str) -> Option<QueryFilter>; // "h"/"history" -> History
    pub fn display_name(&self) -> &'static str;
    pub fn canonical_atom(&self) -> &'static str;
}

// ---- candidate ranking (ranking.rs) ----
// Standalone fuzzy ranking entry: ranks `candidates` against `query` by nucleo
// subsequence score. Returns one `(index, score)` per MATCHING candidate, where
// `index` is the candidate's position in `candidates` and `score` is its nucleo
// subsequence score; pairs are ordered by descending score then ascending index;
// a non-matching candidate (query chars are not an in-order subsequence) is OMITTED.
pub fn fuzzy_rank(query: &str, candidates: &[&str]) -> Vec<(usize, u32)>;

// ---- data sources (mixer.rs) ----
// `accepts` is the source's OWN gate: it returns whether this source should run for
// `query` (including any `query.filter` it opts into). The mixer trusts only this bool.
pub trait SyncDataSource<T: Action> {
    fn id(&self) -> DataSourceId;
    fn accepts(&self, query: &Query) -> bool;        // filter gating is the source's concern
    fn run(&self, query: &Query) -> Result<Vec<QueryResult<T>>, DataSourceRunError>;
}
pub trait AsyncDataSource<T: Action>: Send + Sync {
    fn id(&self) -> DataSourceId;
    fn accepts(&self, query: &Query) -> bool;
    fn run<'a>(&'a self, query: &'a Query)
        -> BoxFuture<'a, Result<Vec<QueryResult<T>>, DataSourceRunError>>;
}
pub struct DataSourceRunError { pub source_id: DataSourceId, pub message: String }

// ---- the mixer (mixer.rs) ----
pub struct SearchMixer<T: Action> { /* opaque */ }
impl<T: Action> SearchMixer<T> {
    pub fn new() -> Self;
    pub fn add_sync_source(&mut self, src: Box<dyn SyncDataSource<T>>) -> DataSourceId;
    pub fn add_async_source(&mut self, src: Box<dyn AsyncDataSource<T> + 'static>) -> DataSourceId;
    pub fn reset(&mut self);                            // clears results + per-source state
    pub fn run_query(&mut self, query: Query);          // dispatches to accepting sources
    pub fn results(&self) -> &[QueryResult<T>];
    pub fn is_loading(&self) -> bool;                   // any async source still pending
    pub fn first_data_source_error(&self) -> Option<&DataSourceRunError>;
}

// ---- full-text backend (searcher.rs, cfg(not(target_family = "wasm"))) ----
pub trait SearchSchemaConfig { /* field set, weights, id field */ }
pub struct SimpleFullTextSearcher<C: SearchSchemaConfig> { /* opaque */ }
impl<C: SearchSchemaConfig> SimpleFullTextSearcher<C> {
    pub const DEFAULT_MEMORY_BUDGET: usize = 50 * 1024 * 1024;
    pub const MIN_MEMORY_BUDGET: usize     = 15 * 1024 * 1024;
    pub fn new(budget: usize) -> Result<Self, SearchError>; // clamps budget to >= MIN
    pub fn insert_document(&mut self, doc: C::Doc) -> Result<(), SearchError>;
    pub fn delete_document(&mut self, id: &C::Id) -> Result<(), SearchError>;
    pub fn search_id(&self, text: &str, limit: usize) -> Result<Vec<C::Id>, SearchError>;
    pub fn clear_search_index(&mut self) -> Result<(), SearchError>;
}
pub enum SearchError { IndexBuild, Write, Query }
```

## EARS Requirements

- **R1.** WHEN `Query::parse(raw, limit)` is called and `raw` begins with `"<atom>:"` where `<atom>` resolves via `QueryFilter::from_atom`, the system shall set `filter` to that `QueryFilter` and set `text` to `raw` with the `"<atom>:"` prefix and any single following space removed.
- **R2.** WHEN `Query::parse(raw, limit)` is called and `raw` has no leading token that resolves via `QueryFilter::from_atom`, the system shall set `filter` to `None` and set `text` to `raw` unchanged.
- **R3.** WHEN `QueryFilter::from_atom` is called with a registered canonical atom or one of its aliases (e.g. `"h"` or `"history"`), the system shall return the matching `QueryFilter`, and IF the atom is unregistered THEN it shall return `None`.
- **R4.** WHEN `add_sync_source` or `add_async_source` is called, the system shall register the source under a `DataSourceId` that is unequal to every previously registered source's id within the mixer and return that id.
- **R5.** WHEN `run_query(query)` is called, the system shall invoke `run` on exactly those registered sources whose `accepts(&query)` returns `true` and shall not invoke `run` on any source whose `accepts(&query)` returns `false`; the boolean returned by `accepts` is the mixer's sole dispatch gate, so a leading filter atom narrows dispatch only insofar as a source consults `query.filter` inside its own `accepts` (the mixer applies no separate filter check).
- **R6.** WHEN results are merged from all accepting sources, the system shall order them by ascending `priority_tier()` and, within an equal tier, by descending `score()`.
- **R7.** WHEN two merged results share an equal tier and equal score, the system shall break the tie by ascending `identity()` so that ordering is total and deterministic.
- **R8.** WHEN merged results contain more than one result with the same `identity()`, the system shall retain only the one with the best rank (lowest tier, then highest score) and drop the rest.
- **R9.** WHEN `run_query` completes its merge, the system shall truncate `results()` to at most `query.limit` entries, keeping the highest-ranked entries.
- **R10.** WHILE at least one accepting async source has dispatched but not yet completed, the system shall report `is_loading()` as `true`, and WHEN all dispatched async sources have completed it shall report `is_loading()` as `false`.
- **R11.** WHEN an async source completes after `run_query` returned, the system shall merge its results into `results()` re-applying R6–R9 without discarding results from already-completed sources.
- **R12.** IF a source's `run` returns `Err(DataSourceRunError)`, THEN the system shall exclude that source's results from the merge, record the error, and still return the results of all succeeding sources.
- **R13.** WHEN one or more sources have errored, the system shall return the earliest-recorded error from `first_data_source_error()`, and WHEN no source has errored it shall return `None`.
- **R14.** WHEN `reset()` is called, the system shall clear `results()` to empty, clear all recorded errors, and set `is_loading()` to `false`, while leaving the set of registered sources unchanged.
- **R15.** WHEN a fresh `run_query` is dispatched, the system shall discard results and errors from the prior query before merging the new query's results, so stale rows from a superseded query never appear in `results()`.
- **R16.** WHEN `fuzzy_rank(query, candidates)` is called, the system shall return exactly one `(index, score)` pair for each candidate whose characters contain `query`'s characters as an in-order subsequence — where `index` is that candidate's position in `candidates` and `score` is its `nucleo` subsequence score — and shall omit every candidate that is not such a subsequence match (a non-matching candidate yields no entry), so the returned length equals the count of subsequence-matching candidates.
- **R17.** WHEN `fuzzy_rank(query, candidates)` returns two or more pairs, the system shall order them by descending `score`, breaking score ties by ascending `index`, so a closer (higher-scoring) subsequence match ranks before a looser one and the ordering is total and deterministic.
- **R18.** WHEN `SimpleFullTextSearcher::new(budget)` is called with `budget < MIN_MEMORY_BUDGET`, the system shall clamp the writer's memory budget up to `MIN_MEMORY_BUDGET`.
- **R19.** WHEN a document is inserted via `insert_document` and then `search_id(text, limit)` is queried with text matching that document's indexed fields, the system shall return that document's id within the first `limit` results; and after `delete_document(id)` the system shall not return that id for the same query.
- **R20.** WHEN `search_id(text, limit)` matches more documents than `limit`, the system shall return at most `limit` ids ordered by descending tantivy relevance score.
- **R21.** IF `insert_document`, `delete_document`, or `search_id` fails at the index layer, THEN the system shall return the corresponding `SearchError` variant and leave the index contents unchanged.
- **R22.** WHERE the build target is `target_family = "wasm"`, the system shall compile the mixer framework and `fuzzy_rank` while excluding the `tantivy`-backed `searcher` module.

## Acceptance Criteria
| # | Criterion (maps to R#) | Status |
|---|---|---|
| 1 | `parse` strips a known `atom:` prefix into `filter` + trimmed `text` (R1) | planned |
| 2 | `parse` with no known atom leaves `filter=None`, `text` verbatim (R2) | planned |
| 3 | `from_atom` resolves canonical + alias, returns `None` for unknown (R3) | planned |
| 4 | Each registered source gets a unique `DataSourceId` (R4) | planned |
| 5 | `run_query` calls `run` on accepting sources only; mixer trusts only `accepts()`'s bool (incl. filter gating) (R5) | planned |
| 6 | Merge orders by ascending tier then descending score (R6) | planned |
| 7 | Equal tier+score ties break by ascending identity (R7) | planned |
| 8 | Duplicate identities collapse to the best-ranked one (R8) | planned |
| 9 | Results truncated to `query.limit`, top-ranked kept (R9) | planned |
| 10 | `is_loading` true while async pending, false when all done (R10) | planned |
| 11 | Late async completion merges in without dropping prior rows (R11) | planned |
| 12 | An erroring source is excluded; succeeding sources still returned (R12) | planned |
| 13 | `first_data_source_error` returns earliest error, else `None` (R13) | planned |
| 14 | `reset` clears results/errors/loading, keeps sources (R14) | planned |
| 15 | A new `run_query` discards the prior query's results/errors (R15) | planned |
| 16 | `fuzzy_rank` includes exactly subsequence-matching candidate indices; omits non-matches (R16) | planned |
| 17 | `fuzzy_rank` orders pairs by descending score, ties by ascending index (R17) | planned |
| 18 | `new(budget < MIN)` clamps budget up to `MIN_MEMORY_BUDGET` (R18) | planned |
| 19 | Insert→search returns id; delete→search omits it (R19) | planned |
| 20 | `search_id` caps at `limit`, ordered by relevance (R20) | planned |
| 21 | Index-layer failure yields the right `SearchError`, no mutation (R21) | planned |
| 22 | wasm target compiles the mixer + `fuzzy_rank`, excludes `searcher` (R22) | planned |

## Visual / Behavioral Acceptance
N/A — this component owns no rendering surface (`browser_testable: no`). All behavior is asserted through the `SearchMixer` / `Query` / `QueryFilter` / `fuzzy_rank` / `SimpleFullTextSearcher` API in unit and integration tests. The palette UI that renders `results()` is a later M2 panel spec; the M1 app-shell static palette (which ranks via `fuzzy_rank`) carries its own visual acceptance in `SPEC-app-shell`, not here.

## Test Plan
- **Unit:** one `#[test]` per EARS clause, named for the requirement:
  - `r1_parse_strips_known_atom`, `r2_parse_keeps_text_when_no_atom`, `r3_from_atom_canonical_alias_and_unknown`, `r4_source_ids_unique_within_mixer`, `r5_run_query_invokes_accepting_sources_only` (asserts both the accept→run and reject→no-run halves, and that a `Some(filter)` query reaches a source iff that source's own `accepts` returned `true`), `r6_merge_orders_tier_then_score`, `r7_equal_tier_score_tiebreak_by_identity`, `r8_duplicate_identity_collapses_to_best`, `r9_truncates_to_limit`, `r14_reset_clears_state_keeps_sources`, `r15_new_query_discards_prior_results`, `r16_fuzzy_rank_includes_subsequence_matches_omits_nonmatch`, `r17_fuzzy_rank_orders_by_descending_score`, `r18_budget_clamped_to_min`.
  - `r16_fuzzy_rank_includes_subsequence_matches_omits_nonmatch` drives a fixed candidate slice where some entries are subsequence matches and some are not, asserting the returned indices are exactly the matching positions and that non-matches contribute no pair.
  - `r17_fuzzy_rank_orders_by_descending_score` asserts a contiguous/closer match outscores a sparser one and that the returned pairs are sorted descending by score with ascending-index tie-break.
  - Sources in unit tests are deterministic fakes implementing `SyncDataSource`/`AsyncDataSource` with scripted tiers/scores/identities and an `accepts` flag; their payload is a fake unit `Action` impl.
  - R12/R13 use a fake source returning `Err` to assert exclusion + `first_data_source_error` ordering (`r12_erroring_source_excluded_others_returned`, `r13_first_error_is_earliest_else_none`).
  - 100% line coverage on every touched line of `data_source.rs`, `ranking.rs`, `mixer.rs`, and `searcher.rs`.
- **Integration** (async runtime + real `tantivy` index):
  - `r10_is_loading_transitions_with_async` and `r11_late_async_merges_without_drop` drive a controllable async source (a oneshot the test resolves) under a real executor.
  - `r19_insert_search_delete_roundtrip`, `r20_search_caps_and_orders_by_relevance`, `r21_index_failure_returns_search_error` exercise `SimpleFullTextSearcher` against a real on-disk/in-RAM tantivy index.
- **Visual:** none (`browser_testable: no`).
- **Regression:** the full `marley_search_core` suite stays green; merge ordering stays a total order after any ranking retune; `from_atom` alias table stays stable; `fuzzy_rank`'s "non-match → no pair" invariant holds after any `nucleo` bump; a `target_family = "wasm"` build job confirms R22 (the crate compiles with the mixer + `fuzzy_rank`, without `searcher`). A future M2 seam test (in `SPEC-app-shell`) confirms the static palette migrates to a `SyncDataSource<Command>` on `SearchMixer` — named here for traceability, out of scope at M1.

## Mutation Targets
`cargo-mutants` must kill every viable mutant on:
- `Query::parse` atom-split (prefix detection, space trim) — killed by R1/R2.
- `QueryFilter::from_atom` match arms and alias table — killed by R3.
- the dispatch gate: invoking `run` when `accepts()` is `false`, or skipping it when `true` (negating the `accepts` bool) — killed by R5.
- the merge comparator: tier ordering (`<`↔`>`), score ordering (ascending↔descending), and the identity tiebreak — killed by R6/R7.
- the dedup keep-best logic (keeping worst instead of best, or keeping all) — killed by R8.
- the `limit` truncation boundary (`<`↔`<=`, off-by-one) — killed by R9.
- `is_loading` pending-count transitions and the late-merge path — killed by R10/R11.
- the error-exclusion branch and `first_data_source_error` earliest-selection — killed by R12/R13.
- `reset` / new-query state-clear (forgetting to clear results, errors, or loading) — killed by R14/R15.
- the `fuzzy_rank` match/no-match gate (emitting a pair for a non-match, or dropping a real subsequence match) and the emitted index (off-by-one into `candidates`) — killed by R16.
- the `fuzzy_rank` ordering comparator (ascending↔descending score, dropping the index tie-break) — killed by R17.
- the `MIN_MEMORY_BUDGET` clamp comparison — killed by R18.
- `SearchError` variant selection on the insert/delete/search error arms — killed by R21.
- MSI target: **100%** on the testable surface. No ACCEPTED-UNTESTABLE lines anticipated; the tantivy I/O paths are covered by the integration tests (R19–R21), the `nucleo` subsequence math by `fuzzy_rank`'s R16/R17, and the mixer dispatch/merge by the deterministic-fake unit tests, so none is excluded.

## Dependencies
- REUSE (permissive, MIT/Apache): `nucleo` (MIT — fuzzy subsequence scoring/ranking for short in-memory candidate lists, behind `fuzzy_rank`), `tantivy` (MIT — embedded full-text index behind `SimpleFullTextSearcher`).
- Marley components: `marley_text_offsets` (`ByteOffset` for future highlight ranges — declared for traceability; highlight rendering itself is out of scope here). **No upstream UI dependency and no palette consumer at M1** — the crate is built and tested in isolation (see the M1/M2 consumer note); the `SyncDataSource<Command>` palette consumer is added by `SPEC-app-shell` in M2.

## Out of scope / deferred
- Row rendering / `QueryResultRenderer`, icons, highlight spans, and the palette panel UI — deferred to an M2 UI spec (this spec stops at producing the ordered `results()` list and the `fuzzy_rank` ranking primitive).
- **The app-shell palette consumer.** At M1 `marley_app` ships an intentionally-temporary static command list ranked via `fuzzy_rank` (local `ScoredCommand { score: u32 }`); its migration onto `SearchMixer` (registering commands as a `SyncDataSource<Command>`, unifying on `QueryResult`/`f32`) is a named **M2** item here, in `SPEC-app-shell`, and in the M1/M2 build order. This crate does not gain that consumer until M2.
- The concrete `marley:` / `forge:` launcher data sources — each is a thin `AsyncDataSource` added in its own later spec; this spec defines only the `Marley` filter atom and the source trait they implement.
- Custom tantivy tokenizers, phrase-prefix/fuzzy field matching tuning, and the schema-definition macro DSL — deferred; M1 ships `SimpleFullTextSearcher` with default tokenization and the typed `SearchSchemaConfig` trait only.
- Score-blending/normalization retune across heterogeneous sources, and any blend of `fuzzy_rank`'s `u32` subsequence scores into `QueryResult`'s `f32` tier scores — treated as tuned constants, revisited in a later ranking spec.
- Persisted/on-disk index lifecycle and incremental reindex scheduling — deferred.

## Clean-room provenance
Behavior-derived from a fork-reference doc; **IP-counsel sign-off pending** (open item in `clean-build-plan.md`). The `spec_source` above is a behavior-level statement (observable I/O only — no private module/type/static names, no fork file paths). The public surface uses Marley-original identifiers and seam-contracts-blessed names only: the central `Action` payload trait (bounds `Clone + 'static`), `Query`/`QueryFilter`/`QueryResult<T: Action>`, `SearchMixer<T: Action>`, the `fuzzy_rank` ranking entry, and `SimpleFullTextSearcher` (seam-contracts §7, §11.1). No Warp-internal type/module/static name appears in the contract. REUSE crates (`nucleo`, `tantivy`) are MIT/Apache. The package is named `marley_search_core`; no AGPL/fork source was read or transcribed for this spec.
