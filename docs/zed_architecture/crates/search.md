# search

> Per-crate reference (Marley round 2) — crate dir `crates/search` (package `search`, lib root `src/search.rs`).
> Zed is the EDITOR reference for Marley's editing surface. Source cloned into session scratch only; this is
> Marley's own description, never Zed code.

| | |
|---|---|
| **Subsystem** | [06 — Project, FS, Worktree & Search](../subsystems/06-project-fs-search.md) |
| **License** | **GPL-3.0-or-later** (per-crate `LICENSE-GPL`) |
| **Provenance** | `[Zed-derived]` — the UI + the **editable-multibuffer results** pattern (the flagship); all matching algorithms live in [`project`](./project.md), not here |
| **Key external deps** | `bitflags` (MIT/Apache — `SearchOptions`), `any_vec`, `itertools` |
| **Zed-internal deps** | `project` (the engine), `editor` + `multi_buffer` (the results surface), `workspace`, `picker`/`picker_preview`, `ui`, `menu`, `file_icons`, `db`, `zed_actions` |
| **Zed dependents** | ~8 crates (`workspace`/app wiring, panels) |
| **Marley target** | phased results UI — **read-only `file:line` dock first**, editable `MultiBuffer` later |

## Purpose

`crates/search` is the **UI half** of find/replace. The matching lives in [`project`](./project.md) (the query
model + fan-out engine); this crate renders it in two surfaces that share one options model:

1. **`BufferSearchBar`** — the in-file find/replace bar (the `⌘F` strip) over the active editor/terminal.
2. **`ProjectSearchView`** — project-wide find/replace whose results are an **editable multibuffer.**

The single highest-leverage design decision is that project-search results are **not a custom list widget** —
they are a normal `Editor::for_multibuffer(...)`. Because a `MultiBuffer` stitches ranges from many real
`Buffer`s into one coordinate space, **typing in a result edits the underlying file and saving persists it — with
no propagation code in the search crate.** Edit-in-results and replace-all come "for free."

## Key types, modules & public API

**`src/search.rs`** (248 lines) — the shared options model + actions.
- **`bitflags! SearchOptions: u8`** (`:68`) — `WHOLE_WORD | CASE_SENSITIVE | INCLUDE_IGNORED | REGEX |
  ONE_MATCH_PER_LINE | BACKWARDS` (+ `NONE`), backed by `enum SearchOption` (`:82`). Each variant carries its
  `label()` / `icon()` / `to_toggle_action()` and an `as_button(active, source, focus)` renderer, so the *same*
  toggle chip drives both the buffer bar and the project view (`enum SearchSource{ Buffer, Project(cx) }`,
  `:95`). `actions!` (`:32`) declares the toggles (`ToggleWholeWord`, `ToggleCaseSensitive`, `ToggleRegex`, …).
- **`src/buffer_search.rs`** (3.9k lines) — **`struct BufferSearchBar`** (`:67`). Holds
  `active_searchable_item: Option<Box<dyn SearchableItemHandle>>` and a per-item map of dyn search state +
  `SearchToken`. Works against **any `SearchableItem`** (editor *or* terminal — the trait, not a concrete type):
  `deploy`/`toggle`/`show`/`dismiss` (`:861`/`:913`/`:921`/`:821`), `search`/`search_suggested`/`query_suggestion`,
  `activate_current_match`, `select_query`, `replacement`/`set_replacement`/`focus_replace`, and the
  `replace_all` action (`:488`). Builds a `SearchQuery` from the shared `SearchOptions` and drives the engine's
  in-buffer path; keeps a query-history cursor.
- **`src/project_search.rs`** (5.7k lines) — the **editable-results surface.**
  - **`struct ProjectSearch`** (`:238`) — the model: `project: Entity<Project>`, **`excerpts: Entity<MultiBuffer>`**,
    `match_ranges: Vec<Range<Anchor>>`, `active_query: Option<SearchQuery>`, `search_id`, `pending_search:
    Option<Task<…>>`, three `SearchHistoryCursor`s, and a `SearchState{ Idle | Running(SearchActivity) |
    Completed(SearchCompletion) }` (`:255`) that tracks `Searching`/`WaitingForScan`/`NoResults`/`Results{
    limit_reached }`. `search()` (`:433`) clears prior ranges, kicks the engine, and drives
    **`consume_search_stream`** (`:504`).
  - **`consume_search_stream`** — drains the engine's `SearchResult` stream and, per file, calls
    `excerpts.set_anchored_excerpts_for_path(PathKey, buffer, ranges, multibuffer_context_lines(cx))` (`:555`),
    which builds+merges **`ExcerptRange{ context, primary }`** (the shown lines = match ± N; the exact
    highlighted span) off-thread and splices them into the `MultiBuffer`'s excerpt `SumTree`. Excerpts group/sort
    by `PathKey`. Accumulates `match_ranges` and flips `SearchState`.
  - **`struct ProjectSearchView`** (`:293`) — the view: `query_editor`, `replacement_editor`, `results_editor`,
    `included_files_editor`, `excluded_files_editor` (all `Entity<Editor>`), plus `search_options: SearchOptions`,
    `filters_enabled`, `replace_enabled`, `pending_replace_all`. **`results_editor` is a generic
    `Editor::for_multibuffer(excerpts)`** — that is the whole trick. Replace-all builds an edit list (regex:
    per-match `replacement_for` capture substitution; literal: one string) applied as a **single
    `ProjectTransaction`.** `struct ProjectSearchBar` (`:320`) is the toolbar host.
  - **`src/text_finder.rs`** (551 lines) — a newer **`Picker`-based** file/text finder (`struct TextFinder`,
    actions `ToProjectSearch`/`Fold`/`Unfold`/`ToggleFoldAll`) that previews matches and can **escalate into a
    full `ProjectSearchView`** (`matches_to_multibuffer`, `PopulateProjectSearch`) — the incremental "quick
    finder → full search" bridge.
- **`src/search_bar.rs`** (139) + **`src/search_status_button.rs`** (71) — shared toolbar chrome + a status-bar
  indicator.

## Depends on (internal)

- [`project`](./project.md) — the entire matching engine (`SearchQuery`, `SearchResult`, `SearchResultsHandle`,
  `SearchOptions` feed). This crate renders; it does **not** match.
- `editor` + `multi_buffer` — `Editor::for_multibuffer` + `MultiBuffer`/`ExcerptRange`/`PathKey`; the
  `SearchableItem`/`SearchableItemHandle` trait (`buffer_search` drives any conformer, editor or terminal).
- `workspace` — dock/pane placement + `ModalView`; `picker`/`picker_preview` — the `TextFinder` modal; `ui`/
  `menu`/`file_icons`/`theme` — chrome; `db` — persisted finder history.

## Used by (internal)

~8 crates — the `workspace`/app layer that mounts the search dock + buffer bar, and the panels that reuse
`SearchOptions`/`SearchableItem`.

## Marley today · gap

**Zero content search.** `command_bar.rs::search_everything` fuzzy-matches file *names* + session titles +
action labels via `nucleo` — never file contents. In-file find is `find.rs::find_matches(haystack, query) ->
Vec<Range<usize>>`, a literal substring scan over one buffer (no regex, no replace, no cross-file). There is
**no `MultiBuffer`** in Marley's editor (the `Buffer` is single-buffer; the "multi" in the code means
*multibyte*). Gap = the results surface *and* the richer in-file bar — the largest net-new user-visible build in
this subsystem.

## Reimplementation on our stack  `[Marley-original]`

Build in the same two layers as Zed, **phased on the multibuffer question** (Marley's editor has no `MultiBuffer`,
so results editability is the real cost gate):

- **In-file bar first (cheap, high value).** Upgrade `find.rs` into a `SearchOptions`-driven bar over the single
  `Buffer`: regex (`fancy-regex`) + whole-word + case + replace/replace-all, plus a query-history cursor. Adopt
  the **`SearchableItem` trait** so the *same* bar later works over a terminal pane, not just the editor. Reuse
  the `SearchOptions` bitflags model verbatim (it is pure `bitflags`, `[permissive/public]`).
- **Project-wide results — the decision point:**
  - **Phase 1 (no multibuffer): a read-only results dock.** A Left/Right-dock list of `file:line` hits (a `grep`
    pane); Enter opens the file at the match in the existing single editor. Delivers ~80% of the value
    (find-across-project) with none of the multibuffer cost, and fits Marley's current docks + finder idioms.
    Drive it straight off the engine's `SearchResult` stream ([`project.md`](./project.md)).
  - **Phase 2 (editable): a minimal `MultiBuffer`** in `crates/editor` — an excerpt = `(source Buffer, context
    range, primary range)` rendered by the existing editor over a stitched coordinate space; replace-all as one
    transaction. This is a real editor investment and should be **gated on the editor-as-peer intake (M14)**,
    since the same `MultiBuffer` also powers diagnostics/references later. Adopt the `ExcerptRange{context,
    primary}` + `PathKey` shape and the "results are a generic editor" decision when it lands.
- **A `TextFinder`-style bridge** (quick preview → escalate to full search) is a natural fit for Marley's
  existing `⌘P` picker idiom and can front the Phase-1 dock.

### Sequencing
1. In-file bar upgrade (regex/whole-word/replace) over the single `Buffer`, `SearchableItem` trait, `SearchOptions`.
2. Phase-1 read-only `file:line` results dock over the engine stream (Enter-to-open). **Ships project-wide find.**
3. Phase-2 `MultiBuffer` + editable results + replace-all — **only alongside the editor-as-peer milestone.**

## Provenance

- `[permissive/public]` — **`bitflags`** (`SearchOptions`), `fancy-regex`/`aho-corasick` (via the engine); the
  matching itself is all permissive (see [`project.md`](./project.md)).
- `[Zed-derived]` (patterns, clean-room) — the shared `SearchOptions`/`SearchOption`-carries-its-own-chip model;
  the `SearchableItem` trait so one bar drives editor *and* terminal; the **"results are a generic
  `Editor::for_multibuffer`"** decision; `ExcerptRange{context, primary}` + `PathKey` excerpt stitching via
  `set_anchored_excerpts_for_path`; replace-all-as-one-`ProjectTransaction`; the `TextFinder`→`ProjectSearch`
  escalation bridge.
- `[Marley-original]` — the phased read-only-dock-then-multibuffer plan and the down-scoped in-file bar.

## Notes / gotchas

- **This crate matches nothing** — every algorithm is in `project`. Keep that split: a Marley search UI should
  depend on the headless engine, not embed matching, so the engine stays pure/testable.
- **Editability is a property of the `MultiBuffer`, not of search** — Zed writes *zero* propagation code because
  excerpts are anchored into live `Buffer`s. Without a `MultiBuffer`, Marley's Phase-1 results are read-only *by
  construction* — which is fine, and honest about the editor investment Phase-2 requires.
- **Results anchored to buffers (not offsets)** is what makes Phase-1→Phase-2 a smooth upgrade: the engine
  already emits `Vec<Range<Anchor>>`, so a read-only dock and an editable multibuffer consume the *same* stream.
- **`SearchableItem` is the terminal-search unlock** — adopting the trait early means "find in this terminal
  pane" reuses the buffer bar, a natural terminal-first Marley feature.
- **`one_match_per_line` / `include_ignored`** are real toggles worth keeping — they map cleanly onto the
  engine's `SearchQuery` flags and the worktree's `is_ignored`.
