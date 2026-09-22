# persist-editor-files — pipeline notes (forge #243, M14 sprint #27)

Pipeline: f3e34465-4adf-45dd-80ea-dadc7a77ad39 · AAR: a456ff66-1cb3-424e-a711-1759826ad737
Ticket: forge#243 (3c1e984e-e2f9-49c6-b890-c96f237f18d5). 5th of M14 (#238 config, #239/#240/#241 done; #242 held).

## Phase 1 — Plan (discovery inline)

**#237 limitation:** the editor surface persists only the ACTIVE file → the other file-tabs vanish on restart.

**Discovery (this session):**
- Save: app.rs:2155-2156 `tab.code_view().map(|cv| TabLayout::Code(cv.path.display()))` — code_view() = ACTIVE.
- Restore: app.rs:916-937 `TabLayout::Code(path)` → read (#154 size/binary guards + resolve_under_root) →
  `Tab::code(EDITOR_TAB_TITLE, cv)` (#240) — a 1-file editor tab.
- Codec: `TabLayout::Code(String)` (grid_layout.rs:208). Serialize (grid_layout.rs:293-300): guards
  `breaks_framing(path)` → `continue` (drop, D2), else `V=<path>` in a `\t`-separated tab-entry. Parse
  (grid_layout.rs:345-346): `V=<path>` → `Code(path)`. So there's an EXISTING `breaks_framing` helper to reuse.
- Source: `EditorSurface { files: Vec<CodeViewState>, active }` (editor_surface.rs); `surface.files()` +
  `active_index()`. `tab.editor()` returns the surface; `code_view()` the active file.

**Design (proposed, D1-D4):** `TabLayout::Code { paths: Vec<String>, active }`; wire `V=<active>\x1f<path…>`;
back-compat old `V=<path>` (no `\x1f`) → `{[path],0}`; a framing-or-`\x1f` path dropped + active re-clamped;
save maps `surface.files()`; restore reads each (skip failures, re-clamp) + rebuilds the surface + activates.

**Driven plan (control):** open 3 files → quit → relaunch → all 3 restore in the strip, the saved active shown.

**ENV:** control granted. Autonomous auto-approved (M14) → run through commit; do not stop; do not push. #242
(editable editor) HELD for chad's scope input (AFK); #243 is INDEPENDENT (persists paths, editable or not).
**AWAIT the inspect critic before Inspect-PASS** (the #240 missed-path + #241 set-PASS-early lessons).

## Phase 2 — Design

**Approach.** A pure codec (grid_layout.rs) + a pure surface constructor (editor_surface.rs) + shim save/restore
wiring (app.rs). Changing `TabLayout::Code`'s shape makes the compiler enumerate every match arm — a good
forcing function.

**File manifest:**
- **grid_layout.rs** — (1) `TabLayout::Code(String)` → `Code { paths: Vec<String>, active: usize }`. (2) two NEW
  pure fns:
  - `serialize_code_paths(paths: &[String], active: usize) -> Option<String>` — keep only paths that DON'T
    `breaks_framing(p) || p.contains('\x1f')`; if none survive → `None` (caller drops the tab, as today);
    re-clamp active = the position of the first survivor whose ORIG index `>= active`, else the last survivor;
    emit `"{new_active}\x1f{joined_by_\x1f}"`.
  - `parse_code_paths(payload: &str) -> (Vec<String>, usize)` — if `payload` contains `\x1f`: split; field 0 =
    active (parse usize, default 0 on error — #95 tolerant), rest = paths; clamp active into paths. ELSE
    back-compat: `(vec![payload.to_string()], 0)`.
  - (3) serialize block (~293): `TabLayout::Code { paths, active } =>` call `serialize_code_paths`; `None` →
    `continue` (drop); else `V=<payload>`. (4) parse block (~345): `V=<payload>` → `parse_code_paths` →
    `Code { paths, active }`. (5) update the round-trip test (~518): the `Code(...)` → `Code { paths: vec![…],
    active }` + the pinned wire string gains the `<active>\x1f` prefix.
- **editor_surface.rs** — NEW pure `EditorSurface::from_files(files: Vec<CodeViewState>, active: usize) -> Self`
  — `active.min(files.len().saturating_sub(1))` clamp (caller guarantees `files` non-empty — the ≥1 invariant).
- **tabs.rs** — NEW thin `Tab::code_surface(title, surface: EditorSurface) -> Self` (wraps a prebuilt surface;
  `Tab::code(title, state)` = `code_surface(title, EditorSurface::new(state))`).
- **app.rs** — save (~2156): `tab.editor().map(|s| TabLayout::Code { paths: s.files().iter().map(|cv|
  cv.path.display().to_string()).collect(), active: s.active_index() })`. restore (~916): for each path
  resolve_under_root + the #154 read/size/binary guards → `CodeViewState`; collect the readable; if empty drop
  the tab; else `Tab::code_surface(EDITOR_TAB_TITLE, EditorSurface::from_files(readable, active))`.

**Regression Test Plan:**
| AC | Test | Proves |
|----|------|--------|
| REQ-001 | `serialize_code_paths`/`parse_code_paths` round-trip: `["a","b"],1`→`"1\x1fa\x1fb"`→back; a NEW 1-file `["a"],0`→`"0\x1fa"` | N-path + active wire round-trips |
| REQ-001 | `parse_code_paths("a")` (no `\x1f`) → `(["a"],0)`; `parse_code_paths("9\x1fa")` → active clamped to 0; `parse_code_paths("x\x1fa")` → active default 0 | back-compat + tolerant active |
| REQ-002 | `serialize_code_paths(["a","BAD\t","c"], 2)` → `"1\x1fa\x1fc"` (BAD dropped, active 2→the survivor "c" at kept-idx 1); `["a\x1f"],0`→`None` | framing/`\x1f` drop + active re-clamp + empty→None |
| REQ-002 | `EditorSurface::from_files(files, 99)` → active clamped to `len-1` | surface active clamp |
| REQ-001 | extend the grid_layout `serialize_shell`/`restore_shell` round-trip with a 2-file `Code` tab (+ the pinned wire) | whole-shell codec round-trips a multi-file code tab |
| REQ-003/004 | driven capture | 3 files → quit → relaunch → 3 tabs restore, saved active shown |

**Mutation/coverage:** `serialize_code_paths` (the drop filter `||`, the `>= active` re-clamp position, the join),
`parse_code_paths` (the `contains('\x1f')` branch, the usize parse/clamp), `from_files` (the `min` clamp) — RUN
`cargo mutants --list -f grid_layout.rs` (+ editor_surface.rs) for the real set; cov/MSI 100. The app.rs save/
restore is shim (coverage-excluded).

**Risks:** (1) the re-clamp when the active path is dropped (load-bearing — a wrong index shows the wrong file or
panics; from_files' `min` clamp is the backstop). (2) back-compat (old `V=<path>` must still restore its 1 file).
(3) ALL `TabLayout::Code` arms updated — compiler-enforced by the shape change. (4) a file literally named "0"
or containing digits — the active field is always FIRST + `\x1f`-delimited, so unambiguous.

**Phase 2 status: Design PASS — codec + from_files + save/restore; manifest + test matrix set.**

## Phase 3 — Implement

Applied the manifest (4 files):
- **grid_layout.rs:** `TabLayout::Code(String)` → `Code { paths: Vec<String>, active: usize }`; pure
  `serialize_code_paths` (drop framing/`\x1f` paths, re-clamp active via "first survivor ≥ active, else last",
  `None` when empty) + `parse_code_paths` (`\x1f` → `<active>\x1f<path…>` w/ tolerant usize + clamp; else
  back-compat `[payload],0`); serialize/parse blocks rewired; the 2 existing tests updated (the round-trip Code
  tab + its pinned wire gained the `0\x1f` prefix; the framing-drop test's `\n`-path tab still drops).
- **editor_surface.rs:** pure `EditorSurface::from_files(files, active)` (clamps active via `min(len-1)`).
- **tabs.rs:** `Tab::code_surface(title, surface)` (wraps a prebuilt surface); `Tab::code` now delegates to it.
- **app.rs:** SAVE maps `tab.editor()` → all `files()` paths + `active_index()`; RESTORE re-reads EVERY path
  (the #154 guards per file via a `filter_map`), drops the tab only if NONE read, else
  `Tab::code_surface(EDITOR_TAB_TITLE, EditorSurface::from_files(readable, *active))` (fully-qualified
  `crate::editor_surface::EditorSurface`).
- **Deviations:** none. The enum shape change compiler-forced the one missed arm (a framing-drop test) — fixed.
- **Build:** `cargo fmt` clean; `cargo check --all-targets -p marley` clean; `clippy -D warnings` clean;
  `grid_layout` tests 12/12 pass. The new pure fns' matrix + `from_files` clamp test land at Phase 4.

**Phase 3 status: Implement PASS — compiles + clippy clean; existing codec tests green (updated for the shape).**

## Phase 3.5 — Inspect (IN PROGRESS — awaiting critic; spec HELD at Implement-PASS so fixes stay in-code)

1 critic spawned + self-review. Self-review so far:
- **[CLEAN, self] round-trip fidelity** — `serialize_code_paths(["a","b","c"],2)`="2\x1fa\x1fb\x1fc" →
  `parse_code_paths` = `(["a","b","c"],2)`. Survivors round-trip; a framing-dropped path is lossy-but-safe with
  the active re-clamped onto a survivor (serialize side).
- **[CLEAN, self] back-compat** — `parse_code_paths("/tmp/x.rs")` (no `\x1f`) → `(["/tmp/x.rs"],0)`; a new
  1-file save `V=0\x1f<path>` round-trips. (Old `cv.path.display()` can't contain `\x1f` in practice.)
- **[CLEAN, self] mutation** — serialize/parse mutants finite + killable by the Phase-4 matrix; `from_files`'s
  only mutant is `Default::default()` (UNVIABLE — no Default derive) → its clamp is coverage+regression.
- **[CLEAN, self] no other consumer** — grep: the only non-updated `TabLayout::Code` hit is a doc comment; the
  enum shape change compiler-forced every real arm.
- **[LOW → FIXED] restore re-clamp (save/restore asymmetry)** — the saved `active` indexes the ORIGINAL `paths`,
  but the restore's `readable` may be shorter if a file was skipped (unreadable/oversized/binary since save).
  `from_files`'s `min` clamp prevented a PANIC, but a file skipped BEFORE the active one shifted the active onto
  the WRONG file (e.g. `[a,b,c]` active=1(b), `a` deleted → shows `c`). SAVE (`serialize_code_paths`) already
  re-mapped active positionally onto survivors; RESTORE did not (naive `min`) — an asymmetry. **FIXED:** extracted
  a shared pure `pub(crate) reclamp_active(surviving_orig, active)` (grid_layout.rs) used by BOTH serialize AND the
  restore (which now `.enumerate()`s the filter_map to track orig indices). Symmetric now. `grid_layout` 12/12
  still green (serialize output byte-identical for surviving-only cases); `reclamp_active` = 3 mutants (Phase-4).

**The critic RETURNED and CONCURS — its ONLY finding is this exact LOW, and its suggested fix IS `reclamp_active`
(already applied). All other lenses CLEAN:** round-trip fidelity (no out-of-range reachable — traced all
drop shapes); back-compat; drop/re-clamp correctness; the restore skip/none-drop semantics; NO other
`TabLayout::Code` consumer assumes a single path (all go through editor()/code_view()/files()); 16 viable
serialize/parse mutants all killable by the 5-test matrix; `from_files`'s only mutant is `Default::default()`
(UNVIABLE — no Default); clean-room + no-panic (tolerant parse, no unwrap/expect). **2 INFO-only (not blockers):**
(a) a PRE-#243 path literally containing raw `\x1f` could mis-parse — astronomically unlikely + CAN'T recur
(serialize now drops `\x1f` paths); (b) `from_files` doesn't self-defend its non-empty precondition — a latent
footgun, but every caller gates on non-empty (app.rs restore + tests) — left as the documented precondition.

**Phase 3.5 status: Inspect PASS — critic CONCURRED; its 1 LOW (restore re-clamp) FIXED via the shared
`reclamp_active`; 2 INFO-only; all other lenses clean.**

## Phase 4 — Validate

**Tests:** added `serialize_code_paths_cases`, `parse_code_paths_cases`, `reclamp_active_cases` (grid_layout.rs)
+ `from_files_clamps` (editor_surface.rs) — the plan matrix. `cargo nextest run -p marley grid_layout
editor_surface` → **19/19 PASS** (the 4 new + existing). Covers REQ-001/002; kills the serialize/parse/reclamp
mutants (from_files' only mutant unviable).

**Driven capture (REQ-003/004, control) — surfaced an ESSENTIAL scope addition:** the first quit→relaunch showed
the editor tab GONE. Isolation (read settings.toml immediately after opening) proved the editor tab was NEVER
PERSISTED — because **`open_file_in_viewer` (app.rs:2215) never called `persist_grid`** (a pre-existing gap: it
mutated the tab in-memory only). #243's codec was correct but the SAVE was never TRIGGERED on file-open, so
REQ-004 ("open N → quit → relaunch → restore N") could not be met. **FIX (a scope addition the driven capture
forced): added `self.persist_grid()` to `open_file_in_viewer`'s success arm.** After the fix: opened 3 files →
settings.toml gained `V=2\x1f<buffer.rs>\x1f<lib.rs>\x1f<movement.rs>` (all 3 paths + active 2, the #243 wire) →
quit → relaunch → **`243-restored-final.png`: the strip restored ALL 3 files, movement.rs (saved active) shown,
rail "Editor", dark**. REQ-004 met LIVE. LESSON: a "persist X" ticket must verify the persist is TRIGGERED, not
just that the codec round-trips — the unit tests + the codec all passed while the feature was broken end-to-end;
only the driven quit→relaunch caught it.

**Docs fix:** gate:14 flagged the public `Tab::code_surface` doc's `[EditorSurface]` intra-doc link (a private
item — its module isn't pub-reexported) → dropped to plain backticks.

**Gate:** `git add -A && scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** — cov 100 + MSI 100 (the
grid_layout serialize_code_paths/parse_code_paths/reclamp_active killed; editor_surface from_files clamp covered);
the app.rs save/restore + the open_file_in_viewer persist are shim/coverage-excluded.

**Phase 4 status: Validate PASS — 19 tests, the persist-trigger fix made REQ-004 work LIVE (3 files restore),
gate green [diff].**
