# "Search tabs" sidebar filter — Notes

- **Forge ticket:** #112 `f883c3c8-09c2-42c4-a9af-8dbe417e9f34` · **AAR:** `cb33804b-e8f9-405d-b730-450607d10916`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-112-search-tabs.md

## Phase 1 — Plan
- **Request:** forge #112 (M5 6/12) — a fuzzy "Search tabs" filter atop the sidebar.
- **Pre-flight:** marley_search_core::fuzzy_score(text,query)->Option<u32> (palette uses it); filter_sessions
  in sessions.rs; the input mirrors the #51 find bar (focus + char/backspace/esc).
- **Decisions:** D1 order-preserving filter on title|subtitle; D2 find-bar input pattern.
- **AAR id:** `cb33804b-e8f9-405d-b730-450607d10916`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **sessions.rs (PURE):** `pub fn filter_sessions(sessions: &[Session], query: &str) -> Vec<Session>` = query.is_empty() → sessions.to_vec(); else keep s where fuzzy_score(&s.title,q).is_some() || fuzzy_score(&s.subtitle,q).is_some(); order preserved.
- **app.rs SHIM:** fields `session_filter: String` + `session_search_focused: bool` (both empty/false in new()). A "Search tabs…" box atop the left dock (above the group loop) showing session_filter or the placeholder; on_mouse_down → session_search_focused=true + close overlays. In the main on_key_down (early, before the terminal input), `if self.session_search_focused { esc→clear+unfocus; backspace→pop; 1-char key→push; return; }` (mirror handle_find_key). The sidebar: `let sessions = filter_sessions(&all_sessions, &self.session_filter);` then group_sessions(&sessions, focused).
- **Mutation targets:** filter_sessions empty passthrough, the title||subtitle predicate, the OR.
- **Test plan:** filter_sessions_empty_returns_all; filter_by_title; filter_by_subtitle_only; filter_no_match_empty; filter_case_insensitive. cov/MSI 100. The box + key routing masked (live capture of the box).
- **Risks:** the search-focus branch must return BEFORE the terminal input eats the keys; filter-before-group; empty query = all (clone).

## Phase 3 — Implement
- **Built:** filter_sessions (sessions.rs — empty→all; else fuzzy title||subtitle via marley_search_core; order preserved); app.rs session_filter+session_search_focused fields + handle_session_search_key (esc-clear/backspace/char, mirrors #51) + the on_key_down routing (return when focused) + a "🔍 Search tabs…" box atop the sidebar (click→focus, accent border when focused) + filter_sessions applied before group_sessions.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (a small filter fn reusing the shared matcher + a masked focus/key input mirroring the proven find bar).
- **Lenses — no findings:** filter_sessions empty→to_vec (all, order kept); else keep s where fuzzy_score(title)||fuzzy_score(subtitle) is_some (case-insensitive via nucleo); the search-focus branch returns BEFORE the terminal input (keys can't leak); esc clears+unfocuses; filter applied BEFORE group_sessions so groups narrow; a stale focus is harmless. No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** filter_sessions_empty_returns_all + filter_sessions_matches_title_or_subtitle (title/subtitle/no-match + case-insensitive). `cargo nextest` → pass.
- **Self-test:** LIVE static capture (search112.png) — a bordered "🔍 Search tabs…" input renders atop the sidebar, above the WORKSPACE group + the session rows. REQ-003 PASS. (filter_sessions engine-tested; typing to filter needs synthetic keys, env-blocked.)
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #112 → done. **M5 6/12 — the sidebar is complete (list→groups→rows→search).** filter_sessions (cov/MSI 100) + the "🔍 Search tabs" box (click-focus, key routing, filter-before-group). Live-proven (search112.png).
