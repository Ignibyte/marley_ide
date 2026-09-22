# cwd + git branch in the title bar [M8] — Notes

- **Forge ticket:** #142 `1e361d9f-4f16-4f03-b824-1d509091528d` · **AAR:** `b9ba6060-ab35-4a89-9b43-9a00b5486518`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-142-titlebar-context.md

## Phase 1 — Plan
- **Request:** forge #142 (M8 run 6/8) — cwd + branch in the unified titlebar.
- **Pre-flight:** app.project_root (144) = cwd; branch best read from {project_root}/.git/HEAD (block.prompt.git_branch
  is per-block, sidebar hardcodes "main" @2221 — latent bug, out of scope). #138 titlebar has title:None + centered search.
- **Decisions:** D1 branch from .git/HEAD; D2 abbreviate >3 segs → first/…/last; D3 render left of the search.
- **AAR id:** `b9ba6060-ab35-4a89-9b43-9a00b5486518`.

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
- **titlebar.rs (NEW PURE):** abbreviate_path(cwd,home)->String (home→~; >3 segs→first/…/last); branch_from_git_head(head)->Option<String> (strip "ref: refs/heads/", trim, non-empty); titlebar_label(cwd,home,branch)->String (path + " · b" when branch).
- **lib.rs:** mod titlebar.
- **app.rs (SHIM masked):** titlebar_context_label(&self)->String = read HOME env + fs::read_to_string(project_root/.git/HEAD).ok() → branch_from_git_head → titlebar_label(project_root, home, branch); render it in the top bar after the icons (left of the centered search), muted, centered vertically.
- **Test plan:** abbreviate_path (deep→~/…/Marley, shallow stays, home→~, non-home stays); branch_from_git_head (ref→Some, sha→None, empty→None); titlebar_label (with/without branch).
- **Risks:** the >3-seg threshold; the ref prefix exactness; best-effort HEAD read (None on error).

## Phase 3 — Implement
- **Built (titlebar.rs NEW PURE):** abbreviate_path, branch_from_git_head, titlebar_label (+ tests written, run in P4). **(lib.rs):** mod titlebar (alpha-sorted by fmt). **(app.rs SHIM):** titlebar_context_label() = $HOME + fs::read {project_root}/.git/HEAD → branch_from_git_head → titlebar_label; rendered in the top bar right(130) — between the centered search and the cockpit tabs (no collision), muted 12px.
- **Deviation:** placed the label RIGHT-of-search (right:130) not left-of-search — the left gap between the icons (x152) and the centered search (x~302) is too tight for the label; the search↔cockpit gap is roomy. Warp-context intent preserved.
- **Verification:** fmt; check 0 err; clippy OK.

## Phase 3.5 — Inspect
- **Method:** self-review (3 pure string fns + a best-effort read + render).
- **Lenses — no findings:** abbreviate_path handles home==cwd (→~), home-prefix (→~/rest), non-home (as-is), and collapses >3 segs to first/…/last; the segment filter drops empties so a trailing/leading / does not miscount. branch_from_git_head strips exactly "ref: refs/heads/", trims, rejects empty (→ detached/junk None). titlebar_label omits an empty branch. The shim read is best-effort (ok().and_then) → no panic on a missing/detached .git/HEAD; HOME missing → unwrap_or_default (empty home → path unchanged, still valid). Render right:130 avoids the search + cockpit tabs. No unwrap on IO. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** abbreviate_path_cases, branch_from_git_head_cases, titlebar_label_cases. Pass.
- **Self-test:** LIVE capture via the BARE binary launched from the project dir (a .app launch gets cwd / so project_root=/ — no repo; the bare binary inherits the Marley cwd → real project_root + branch).
- **Gate:** first run RED (1 missed: `> `→`>=` in abbreviate_path — no exactly-3-segment test); FIXED by adding the 3-seg boundary case (~/Projects/Marley stays). Re-run below. Capture (titlebar142.png, bare binary from the repo): titlebar shows "~/…/Marley · main". REQ-004 PASS.

## Phase 5 — Complete
- CHANGELOG; forge #142 → done. **M8 6/8.** Titlebar shows ~/…/Marley · main: pure titlebar.rs (abbreviate_path/branch_from_git_head/titlebar_label, cov/MSI 100) + a masked shim reading $HOME + .git/HEAD, rendered right of the search. LESSON: mutation boundary — a `>N` threshold needs an exactly-N test to kill the `>=` mutant.
