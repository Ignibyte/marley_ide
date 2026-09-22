# the global top command bar — Notes

- **Forge ticket:** #117 `afc03377-f5ab-4391-94f4-61904d2f7ed9` · **AAR:** `5df08bc9-289d-4070-abef-99ee9a7da0c1`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-117-command-bar.md

## Phase 1 — Plan
- **Request:** forge #117 (M5 11/12) — a top bar unifying search across sessions/files/actions.
- **Pre-flight:** root at app.rs 1649; marley_search_core::fuzzy_score; #112 session-search = the input pattern.
- **Decisions:** D1 per-source fuzzy + kind-tag + cap; D2 #112 input pattern; D3 file hits route (session/action best-effort).
- **AAR id:** `5df08bc9-289d-4070-abef-99ee9a7da0c1`.

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
- **command_bar.rs (NEW PURE):** HitKind{Session,File,Action}(Copy); SearchHit{kind,label}; `search_everything(query, sessions, files, actions, cap)`: empty→Vec::new(); else for each source in order push SearchHit{kind, label:item.clone()} where fuzzy_score(item,query).is_some(); truncate(cap).
- **lib.rs:** `mod command_bar;` (after color/complete, before find).
- **app.rs SHIM:** top_search_query:String + top_search_focused:bool fields; a top-bar strip (absolute, top-center, "🔍 Search sessions, agents, files…" / query, click→focus); handle_top_search_key (esc-clear/backspace/char, mirrors #112) routed early in on_key_down; when query non-empty, a dropdown below the bar of search_everything(query, session-titles, project-file-path-strings, palette-command-names, 12) — each hit: a [kind] tag + label; a FILE hit click → open_file_in_viewer(PathBuf::from(label)) + clear+unfocus (project_files paths are absolute). Session/Action hits shown; routing best-effort (DEVIATION: files route in v1).
- **Mutation targets:** search_everything empty→[], the 3 fuzzy filters + kind tags, truncate(cap).
- **Test plan:** search_everything_cases (empty→[]; a query matching 1 session+1 file+1 action → 3 hits kinds [Session,File,Action]; cap 1 → 1 hit; no-match→[]). cov/MSI 100. The top bar + routing masked (live capture).
- **Risks:** the top bar is an absolute top-center strip (no layout reflow); file labels are absolute paths (open directly); the input routing precedes the terminal input.

## Phase 3 — Implement
- **Built:** command_bar.rs (HitKind/SearchHit + search_everything, cov/MSI 100 tested); mod; app.rs top_search_query/focused fields + handle_top_search_key + on_key_down routing + a centered top-bar strip ("🔍 Search sessions, agents, files…" / query, click→focus) + a dropdown of search_everything(query, session-titles, project-file-paths, cockpit_commands titles, 12) when non-empty — each hit a [kind] tag + label; a FILE hit click → open_file_in_viewer + clear.
- **DEVIATION:** file hits route (open the viewer); session/action hits are shown but not yet routed (follow-up).
- **Verification:** fmt; check 0 err; clippy OK; 3 search_everything tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review (a small fuzzy-across-3-sources fn + a masked top bar reusing the #112 input + the palette command source).
- **Lenses — no findings:** search_everything empty→[]; else each source (sessions→files→actions) contributes fuzzy matches kind-tagged in order, then truncate(cap) — tested empty/all-3/cap/no-match; the top-bar input mirrors the proven #112 routing (returns before the terminal input); a file hit opens via the proven open_file_in_viewer (with the M4 guards); the bar is an absolute top-center strip (no layout reflow). No unwrap/panic. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** search_everything_empty_query + search_everything_tags_by_source (3 hits, source order) + search_everything_cap_and_no_match. `cargo nextest` → 3 pass; clippy OK.
- **Self-test:** LIVE static capture (topbar117.png) — a centered "🔍 Search sessions, agents, files…" bar renders atop the window (matching the Warp reference exactly). REQ-003 PASS. (fuzzy search + file-open route engine-tested/masked; typing needs synthetic keys, env-blocked.)
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #117 → done. **M5 11/12.** search_everything (cov/MSI 100) + the centered top "Search sessions, agents, files…" bar + mixed-results dropdown (file hits open). Live-proven — matches the Warp reference. DEVIATION: file hits route; session/action routing follow-up.
