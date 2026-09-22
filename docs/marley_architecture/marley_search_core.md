# `marley_search_core`

> Per-crate architecture note — **round 4 refresh · 2026-07-12 · current to M15.**
> Provenance: **`[Marley-original]`** (INVENT) thin wrapper over **`[permissive/public]` `nucleo`**
> (MIT/Apache). **Fuzzy-NAME match only — NOT content search.** The content-search gap (project-wide
> find-in-files + an editable results surface) maps to
> [`zed_architecture/crates/search.md`](../zed_architecture/crates/search.md); see **Scope & the gap** below.

The shared **fuzzy** search engine — the M2 search seam. A thin, tested wrapper over the [`nucleo`]
subsequence matcher so there is ONE matcher in the codebase.

## Scope & the gap — fuzzy-name match, not content search

This crate is a **fuzzy subsequence matcher over short strings** (command titles, file *names*/paths,
history entries). It never opens a file or scans file *contents*. That is the whole M2 search story so far,
and it is worth naming the gap precisely (mirrors [`zed_architecture/crates/search.md`](../zed_architecture/crates/search.md)'s
"Marley today · gap"):

- **No content search.** There is no project-wide find-in-files, no regex-over-files, no `file:line`
  results dock, and no editable `MultiBuffer` results surface. Confirmed: **no `tantivy`** (or any
  full-text index) anywhere in the tree — the M0 crate-map listed `tantivy` as an aspiration for the
  command palette's `SearchMixer`; it was never adopted, and this crate is nucleo-only.
- **In-file find is separate and literal.** The single-buffer in-file find lives in the editor
  (`find.rs`-style literal substring scan), NOT here — no regex, no replace, no cross-file.
- **The Zed reference is the phased plan to close it.** Zed splits find/replace into a headless matching
  *engine* + a *UI* whose project results are a generic `Editor::for_multibuffer` (so typing in a result
  edits the file for free). Marley's planned path: an in-file `SearchOptions` bar first, then a **Phase-1
  read-only `file:line` results dock** off a headless engine stream, then **Phase-2 an editable
  `MultiBuffer`** — gated on the editor-as-peer milestone (M14/M15). None of that is this crate; this crate
  stays the *fuzzy* seam.

## Surface

```rust
pub struct Scored { pub index: usize, pub score: u32 }   // Debug/Clone/Copy/Eq (deliberately NOT Default)
pub fn fuzzy_score(text: &str, query: &str) -> Option<u32>;
pub fn fuzzy_rank(candidates: &[&str], query: &str) -> Vec<Scored>;
```

- **`fuzzy_score`** — the case-insensitive nucleo subsequence score of `query` in `text`; `None` when the
  query's chars don't occur in order. Makes its own `Matcher`.
- **`fuzzy_rank`** — empty query → every candidate in input order (score 0); else the matching candidates
  only, sorted by score DESC then input index ASC (a stable, deterministic order), sharing one internal
  `Matcher`.
- A private `score_with(matcher, text, query)` is the shared primitive — `fuzzy_rank` reuses one matcher
  across many candidates (files can be numerous). Both sides are lower-cased for case-insensitive matching.

## Design notes
- **Wraps nucleo, doesn't reinvent it** — consecutive-run/prefix/boundary bonuses are nucleo's; this crate
  owns only case-folding, the match/no-match gate, and the stable ordering. Scores are version-local:
  consumers + tests assert ORDER and match/no-match, never absolute values.
- **`Scored` is `Copy`, not `Default`** — `Copy` for consumer ergonomics; omitting `Default` keeps
  cargo-mutants' `vec![Default::default()]` mutant of `fuzzy_rank` unviable (no free mutant to kill).
- **Mutation caveat** — the nucleo-wrapper fns are method-call-heavy; cargo-mutants emits only whole-fn
  value mutants, so behavioral assertions + 100% coverage are the real guard (see
  `PR-claude-method-call-code-mutation-hollow-001`).

## Consumers
- The **command palette** (`marley_app::palette::filter_commands`) ranks each command's title + keywords
  through `fuzzy_score`.
- **Fuzzy file-open** (#57, SHIPPED) — `marley_app::finder::FinderState` ranks the project's file *names*
  through `fuzzy_rank`; bare ⌘P opens the overlay, Enter inserts the chosen path at the prompt.
- **Command history search** (#60, SHIPPED) — ⌘R fuzzy-ranks `CommandHistory::recent()` through
  `fuzzy_rank` (reusing `FinderState`); Enter inserts the chosen command into the cooked prompt buffer.
  (⌘R = history; rerun-last moved to ⌘⇧R.)
- The overlays + key routing are the `marley_app` shim; the pure models are cov/MSI 100.

## M2.A workspace (docks, SHIPPED)
The workspace chrome is populated end to end: the **Left "Files" dock** shows the project file tree (#56,
[`marley_project::FileTree`](./marley_project.md)), the **center** is the terminal, and the **Right
"Details" dock** inspects the current command block (#58). ⌘P (#57) fuzzy-opens across the same project
files. Later: a multi-source `SearchMixer` (command + file + history; a forge source was on this list until the #409 scrap-forge pivot) — and, if/when content search
lands, the Zed-referenced engine/UI split above (which this fuzzy seam sits *alongside*, not inside).
