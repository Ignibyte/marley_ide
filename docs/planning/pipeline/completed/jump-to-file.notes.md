# jump-to-file from a diff — Notes

- **Forge ticket:** #105 `41f545f7-e2b3-4f0f-b736-fc2089cf7ffc` · **AAR:** `d8f8b283-ea9f-4b3f-86a9-0fe14bb6bff1`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-105-jump-to-file.md

## Phase 1 — Plan
- **Request:** forge #105 (M4 9/10) — parse `path:line` refs + click a diff file header to open the file.
- **Pre-flight:** #98 open_file_in_viewer + #101 jump_to + #103 diff FileHeader rows all exist; project_root
  (#102) is the join base.
- **Decisions:** D1 `/`-or-`.` path + numeric 2nd field; D2 FileHeader opens at top; D3 1-based→0-based.
- **AAR id:** `d8f8b283-ea9f-4b3f-86a9-0fe14bb6bff1`.

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
- **code_view.rs (PURE):** `struct FileRef{path:String,line:Option<usize>}`; `parse_file_ref(text)`: trim; splitn(3,":"); path=first (?); if empty || (!contains "/" && !contains ".") → None; line = 2nd field parsed as usize (None if absent/non-numeric); Some(FileRef).
- **app.rs SHIM:** the diff overlay render loop: for a FileHeader-role row, attach on_mouse_down(Left) → `let full=view.project_root.join(&fr.path); view.open_file_in_viewer(full);` then set cv.scroll=jump_to(fr.line.unwrap_or(1).saturating_sub(1),VIEWER_ROWS,len); view.diff=None; notify. (Compute full BEFORE the &mut self call to avoid a borrow clash.) Non-FileHeader rows stay plain.
- **Mutation targets:** parse_file_ref trim, the empty/`/`-or-`.` guard, the splitn(3) line parse, the None.
- **Test plan:** parse_file_ref_cases ("src/foo.rs:42"→Some(path,42); "a/b.rs"→Some(path,None); "no ref"→None; "src/a.rs:foo"→Some(path,None); "src/a.rs:12:5"→Some(path,12); "  x.rs:3  "→Some(x.rs,3)). cov/MSI 100. The FileHeader click masked.
- **Risks:** borrow order (full path computed first); FileHeader text is the bare path (line None → top); the terminal-token click is a follow-up.

## Phase 3 — Implement
- **Built:** FileRef + parse_file_ref (code_view.rs — trim/splitn(3)/path-guard(/ or .)/numeric-line); the diff overlay FileHeader rows are now clickable → open project_root.join(path) in the viewer, scroll = jump_to(line-1), close the diff.
- **Verification:** fmt; check 0 err; clippy OK (parse_file_ref consumed by the FileHeader click); parse_file_ref test passes.

## Phase 3.5 — Inspect
- **Method:** self-review (one small pure parser + a masked click reusing #98 open + #101 jump); gate cargo-mutants is the MSI authority.
- **Lenses — no findings:** parse_file_ref (trim; path=first colon-field, rejected unless it has / or . or is non-empty; line=2nd field if numeric else None; 3rd (col) ignored — tested path+line/path-only/non-path/non-numeric-line/col/trim); the FileHeader click computes the full path BEFORE the &mut self open (no borrow clash), 1-based→0-based via saturating_sub(1), reuses the read-only open + jump. No unwrap/panic (unwrap_or(1)). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** parse_file_ref_cases (6 cases). `cargo nextest` → pass.
- **Self-test:** clicking a diff file header needs a synthetic click (ENV-BLOCKED); parse_file_ref engine-tested cov/MSI 100; the click masked (+ #102 live git).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #105 → done. **M4 9/10.** parse_file_ref + FileRef (cov/MSI 100); the diff file headers click → open the file at the line. Terminal-token click = follow-up.
