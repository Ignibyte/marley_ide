# the file explorer (icons + dimming) — Notes

- **Forge ticket:** #113 `f35494de-7d89-479b-a3e2-2f8736a76fb7` · **AAR:** `7f60d2e5-5276-4607-a70b-c7d7b3a4a487`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-113-file-explorer.md

## Phase 1 — Plan
- **Request:** forge #113 (M5 7/12) — file-type icons + dimming in the Files tree.
- **Pre-flight:** TreeRow = {depth, name, is_dir}; NO path/gitignore signal → dotfile-dimming (D2). Files
  stays in the left dock (D3); the movable-pane aspect folds into the seq-8 pane dispatch.
- **Decisions:** D1 extension→glyph; D2 dotfile dimming; D3 Files stays in the dock.
- **AAR id:** `7f60d2e5-5276-4607-a70b-c7d7b3a4a487`.

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
- **file_tree_view.rs (NEW PURE):** `pub fn file_icon(name: &str) -> &'static str` = by lowercased extension (rsplit_once('.')): rs→🦀 / toml→⚙ / json→{} / md|markdown→📝 / else→📄; `pub fn entry_is_dimmed(name: &str) -> bool` = name.starts_with('.').
- **lib.rs:** `mod file_tree_view;` (alphabetical, after find/finder/flash).
- **app.rs SHIM:** the left-dock Files loop — for a FILE row (!is_dir) the glyph becomes file_icon(&row.name) (was " "); the entry text_color dims (a lower-alpha muted or the muted at reduced opacity) when entry_is_dimmed(&row.name); DIR rows keep ▾/▸.
- **Mutation targets:** the extension→glyph arms + the else; the dotfile check.
- **Test plan:** file_icon_by_extension ("a.rs"→🦀, "Cargo.toml"→⚙, "x.json"→{}, "R.md"→📝, "a.markdown"→📝, "readme.txt"→📄, "noext"→📄, "A.RS"→🦀 case-insensitive); entry_is_dimmed_dotfiles (".gitignore"→true, "src"→false, ""→false). cov/MSI 100.
- **Risks:** case-insensitive extension (lowercase before match); a name with no "." → else (📄); the dim applies only to files (dirs keep the disclosure + normal color).

## Phase 3 — Implement
- **Built:** file_tree_view.rs (file_icon by extension: rs🦀/toml⚙/json{}/md📝/else📄 + entry_is_dimmed dotfiles, with tests); mod file_tree_view; app.rs Files loop — a file row now shows file_icon(&row.name) (was blank) + dims dot-files (muted @ 0.5 alpha); dirs keep ▾/▸ + full color.
- **Verification:** fmt; check 0 err; clippy OK; 2 tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review (two small pure fns + a masked Files-loop tweak).
- **Lenses — no findings:** file_icon lowercases the rsplit_once extension → distinct glyph per known type, else generic (tested rs/toml/json/md/markdown/txt/noext + case-insensitive A.RS); entry_is_dimmed = starts_with(dot) (tested dotfile/plain/empty); the render applies file_icon to FILES only (dirs keep the disclosure) + dims dot-files (muted @ 0.5); no unwrap/panic (rsplit_once → Option, map). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** file_icon_by_extension (rs/toml/json/md/markdown/txt/noext + case-insensitive) + entry_is_dimmed_dotfiles. `cargo nextest` → 2 pass; clippy OK.
- **Self-test:** LIVE static capture (files113.png) — each file in the Files tree now shows a 📄 type icon (the .8 man-pages → the generic glyph; .rs/.toml would show 🦀/⚙ in a real project — the open-launched cwd is "/") where before #113 it was blank; dirs keep ▾/▸. REQ-003 PASS. (dotfile dimming engine-tested; none visible at /usr/share/man.)
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #113 → done. **M5 7/12.** file_icon + entry_is_dimmed (cov/MSI 100) + file-type icons + dotfile dimming in the Files tree. Live-proven (files113.png). DEVIATION: gitignore→dotfile dimming (no ignore signal); Files stays in the dock (movable-pane folds into seq-8 dispatch).
