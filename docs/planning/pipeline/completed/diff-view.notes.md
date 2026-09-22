# a diff view (diff_rows projection) — Notes

- **Forge ticket:** #103 `b1eaffcb-0290-4c13-8c20-9869d0a34bc1` · **AAR:** `56535934-d25f-4c8a-aeca-7820268bec61`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-103-diff-view.md

## Phase 1 — Plan
- **Request:** forge #103 (M4 7/10) — diff_rows projection + refactor #102's overlay.
- **Pre-flight:** #102's ⌘⇧D overlay renders files→hunks→lines inline (app.rs); FileDiff/Hunk/DiffLine/
  DiffKind in git_diff.rs. Refactor the render onto a tested diff_rows.
- **Decisions:** D1 flat rows (FileHeader/HunkHeader/Add/Remove/Context) + signs + empty row; D2 role→color.
- **AAR id:** `56535934-d25f-4c8a-aeca-7820268bec61`.

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
- **git_diff.rs (PURE):** `enum DiffRole{FileHeader,HunkHeader,Add,Remove,Context}` (Copy); `struct DiffRow{text:String,role:DiffRole}`; `diff_rows(files)`: if empty → `vec![row("no working-tree changes",Context)]`; else for file: push row(file.path, FileHeader); for hunk: push row(hunk.header, HunkHeader); for line: match kind → (Add,"+")/(Remove,"-")/(Context," ") → push row(format!("{sign}{}",line.text), role). A private `row(text,role)` helper.
- **app.rs SHIM:** the #102 ⌘⇧D overlay: import diff_rows+DiffRole; replace the header child + the nested file/hunk/line loop with `for row in diff_rows(files) { overlay.child(div().px_3().text_color(match row.role {FileHeader=>foreground, HunkHeader=>accent, Add=>success, Remove=>danger, Context=>muted}).child(row.text)) }` (a header line "🔀 working diff (N files)" can stay above, or diff_rows's empty-row covers the no-changes case). Drop the now-unused DiffKind import if the manual match is gone.
- **Mutation targets:** diff_rows empty-case, the row order (FileHeader→HunkHeader→lines), the per-kind role+sign.
- **Test plan:** diff_rows_empty (→[Context "no working-tree changes"]); diff_rows_projects_in_order (a file{path,[hunk{header,[Context "z",Add "x",Remove "y"]}]} → [FileHeader path, HunkHeader header, Context " z", Add "+x", Remove "-y"]). cov/MSI 100. The overlay refactor masked.
- **Risks:** the overlay refactor must be output-equivalent (diff_rows produces the same lines #102 did); the File↔Diff-in-viewer toggle is folded into ⌘⇧D (deviation).

## Phase 3 — Implement
- **Built:** git_diff.rs DiffRole + DiffRow + diff_rows (flatten FileDiff tree → FileHeader/HunkHeader/signed line rows; empty → one "no working-tree changes" row); the #102 ⌘⇧D overlay refactored to render `diff_rows(files)` colored by role (dropped the inline nested loop + the DiffKind import).
- **Verification:** fmt; check 0 err; clippy OK; both diff_rows tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review (a pure flatten-projection + a masked overlay refactor); gate cargo-mutants is the MSI authority.
- **Lenses — no findings:** diff_rows (empty→one Context row; else FileHeader→HunkHeader→lines in order; each line a +/-/space-signed row with the Add/Remove/Context role — both cases tested exactly); the overlay renders diff_rows colored by role (output-equivalent to #102 modulo the header line). No panics. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** diff_rows_empty + diff_rows_projects_in_order. `cargo nextest` → pass.
- **Self-test:** the ⌘⇧D diff overlay needs a synthetic chord (ENV-BLOCKED); diff_rows engine-tested cov/MSI 100; the overlay masked (+ #102 live git).
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #103 → done. **M4 7/10.** diff_rows + DiffRow/DiffRole (cov/MSI 100); the ⌘⇧D overlay refactored onto diff_rows. File↔Diff-in-viewer folded into ⌘⇧D (deviation).
