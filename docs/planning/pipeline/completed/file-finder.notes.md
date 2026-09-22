# fuzzy file-open overlay (cmd-P) — Notes

- **Forge ticket:** #57 `2d7d3df4-0ff3-4fe7-8e44-07a5c568496e`
- **AAR:** `5edda1b8-fc27-4af7-a856-ea28ec7c5f91`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-057-file-finder.md

## Phase 1 — Plan
- **Request:** forge #57 (M2.A seq-5, auto-approved) — cmd-P fuzzy file-open. First real consumer of
  `marley_search_core` (#54); ranks the project files (#56's list).
- **Classification:** work pipeline, `feature`, PURE `FinderState` (new marley_app module) + app.rs SHIM.
  UI — validate MUST self-test-capture.
- **Pattern:** `FinderState` mirrors `PaletteState` (palette.rs: push/backspace reset selected;
  move_up saturating_sub; move_down `(sel+1).min(len-1)`; activate/chosen via `.get(selected)`). The
  overlay + key routing mirror the palette's (handle_palette_key + the cmd-shift-p overlay).
- **Reuse (D4):** #56 already calls `list_files_in` in `new()` and discards the flat list (keeps only
  the FileTree). Store `project_files: Vec<PathBuf>` from that SAME call — one walk feeds both.
- **Keymap (D2):** cmd-p (bare) is FREE — the palette is cmd-shift-p; confirm no other cmd-p binding in
  keymap.rs at Design.
- **Open (D3):** first cut writes the chosen path to the PTY (shell prompt insert); editor-pane open later.
- **Hollow-MSI watch:** `results` wraps fuzzy_rank (method-call → few mutants); push/backspace/move/
  chosen carry the viable mutants (reset, clamp, get-chain) — behavioral tests guard them (like the
  palette's proven pattern).
- **AAR id:** `5edda1b8-fc27-4af7-a856-ea28ec7c5f91`.

## Phase 2 — Design

### PURE — `crates/marley_app/src/finder.rs` (NEW)
```rust
use std::path::{Path, PathBuf};

/// The cmd-P file finder's PURE state — the live query + the selected result row.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FinderState { query: String, selected: usize }

impl FinderState {
    pub fn new() -> Self { Self::default() }
    pub fn query(&self) -> &str { &self.query }
    pub fn selected(&self) -> usize { self.selected }
    pub fn push(&mut self, text: &str) { self.query.push_str(text); self.selected = 0; }
    pub fn backspace(&mut self) { self.query.pop(); self.selected = 0; }
    pub fn move_up(&mut self) { self.selected = self.selected.saturating_sub(1); }
    pub fn move_down(&mut self, results_len: usize) {
        self.selected = (self.selected + 1).min(results_len.saturating_sub(1));
    }
    /// The fuzzy-ranked indices of `files` for the current query (empty query → all in input order).
    pub fn results(&self, files: &[&str]) -> Vec<usize> {
        marley_search_core::fuzzy_rank(files, &self.query).into_iter().map(|s| s.index).collect()
    }
    /// The chosen file: the `selected`-th of `results`, or `None` when `results` is empty.
    pub fn chosen<'a>(&self, files: &'a [PathBuf], results: &[usize]) -> Option<&'a Path> {
        results.get(self.selected).and_then(|&i| files.get(i)).map(PathBuf::as_path)
    }
}
```

### keymap.rs (PURE)
Add to `default_bindings`: `(KeyBinding::chord(true, false, false, false, "p"), "open-file-finder")` —
bare cmd-p (the palette is `chord(…, true, "p")` = cmd-shift-p). A keymap test asserts `action_for(cmd-p)
== "open-file-finder"` and that cmd-shift-p is still the palette (no regression).

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- `RootView { finder_open: bool, finder: FinderState, project_files: Vec<PathBuf> }`. `new()` keeps the
  `list_files_in(&project.root)` result (from #56) and stores BOTH `file_tree` (from `.files`) and
  `project_files` (the `.files` Vec) — one walk.
- Dispatch `"open-file-finder"` → `self.finder = FinderState::new(); self.finder_open = true; cx.notify()`.
- Key routing: the `on_key_down` handler checks `finder_open` FIRST (like `palette_open`) → `handle_finder_key`.
- `handle_finder_key` (mirror `handle_palette_key`): `escape`→close; `enter`→ resolve
  `finder.chosen(&project_files, &finder.results(&strs))`, write its path bytes to the active session's
  PTY, close; `up`→move_up; `down`→move_down(results_len); `backspace`→backspace; a printable key→push.
- Overlay render (mirror the palette overlay): a centered panel with the query line + the ranked file
  rows (`finder.results(&strs)` → the paths), the `selected` row highlighted; rendered when `finder_open`.
- `strs`: `project_files.iter().map(|p| p.to_string_lossy()).collect()` (built where needed).

### File manifest
- NEW `crates/marley_app/src/finder.rs` — FinderState + tests.
- MODIFY `crates/marley_app/src/keymap.rs` — cmd-p binding + a test.
- MODIFY `crates/marley_app/src/app.rs` — `mod finder;`/use, RootView fields, new() wiring, dispatch,
  handle_finder_key, key routing, overlay render.
- MODIFY `crates/marley_app/src/lib.rs` (or wherever mods are declared) — `mod finder;` if needed.

### Mutation Targets (pure)
- `push`/`backspace` the `selected = 0` reset; `move_up` `saturating_sub`; `move_down` `(sel+1).min(len-1)`
  incl. the empty `saturating_sub`; `results` the `fuzzy_rank(files, &self.query)` wiring + `.index` map;
  `chosen` the `get(selected).and_then(get).map` chain. keymap: `action_for(cmd-p)` arm.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `edit_resets_selection` — move down, then push/backspace → selected 0 | unit |
| REQ-002 | `move_clamps` — move_up at 0 → 0; move_down×N clamps at len-1; move_down(0) → 0 | unit |
| REQ-003 | `results_ranks_via_fuzzy_rank` — `["app.rs","main.rs","lib.rs"]` query "app" → `[0]`; empty query → `[0,1,2]` | unit |
| REQ-004 | `chosen_selected_or_none` — results `[2,0]` selected 0 → files[2]; empty results → None | unit |
| REQ-004b | keymap `cmd_p_opens_file_finder` — `action_for(cmd-p)=="open-file-finder"`, cmd-shift-p still palette | unit |
| REQ-005 | cmd-P overlay lists ranked matches, top highlighted | self-test (drive cmd-P + type → capture) |
| REQ-006 | gate GREEN, cov/MSI 100 finder.rs + keymap.rs; app shim excluded | gate |

Uncoverable: the app.rs overlay/key routing/PTY write — masked + cov-excluded, proven by REQ-005.

### Risks / decisions
- D-2.1 `results` takes `&[&str]` (fuzzy_rank's shape); the shim builds the lossy strings per render. The
  index returned by `results` indexes into the ORIGINAL `project_files` (same order as `strs`), so
  `chosen(&project_files, &results)` is consistent. D-2.2 finder key routing must gate BEFORE palette/
  terminal input (finder_open wins) — same precedence pattern as palette. D-2.3 Enter-writes-path-to-PTY
  is the first-cut open (D3); no editor pane yet.

## Phase 3 — Implement
- **Built (PURE):** `finder.rs` — `FinderState` (new/query/selected/push/backspace/move_up/move_down/
  results/chosen) verbatim from the design. `keymap.rs` — added the bare `cmd-p` → `open-file-finder`
  binding (below the cmd-shift-p palette one). `lib.rs` — `mod finder;`.
- **Built (SHIM, app.rs — mutants::skip/cov-excluded):** `RootView { finder_open, finder,
  project_files }`; `new()` now lists the project files ONCE into `project_files` and builds `file_tree`
  from it (one walk, feeds both); dispatch `"open-file-finder"`; `handle_finder_key` (esc/enter→write
  chosen path to the active PTY via `focused_state_mut().session.write_bytes`/close/up/down/backspace/
  push — mirrors `handle_palette_key`); key routing gates `finder_open` BEFORE palette; a cmd-P overlay
  (query line + up to 20 ranked file rows, selected highlighted) mirroring the palette overlay.
- **Deviations:** none of substance — the enter arm scopes the `project_files` borrow (results+chosen→
  owned bytes) before the `workspace` mut-borrow (borrow-checker); overlay caps at 20 rows (design's
  "no scroll for long lists" out-of-scope).
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; `cargo clippy -p marley -- -D warnings`
  OK; `cargo nextest -p marley` 115 pass (no regression). FinderState + keymap tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (faithful FinderState probe + real cargo-mutants + a RUN of the keymap/palette tests +
  a keymap `action_for` probe for both chords). Verdict: **PASS — code correct, no bugs.**
- **keymap NON-CONFLICT RESOLVED (the flagged risk):** probed — `action_for(cmd-p)=="open-file-finder"`,
  `action_for(cmd-shift-p)=="open-command-palette"`. `KeyBinding` derives `Eq` over ALL fields incl.
  `shift`, so cmd-p ≠ cmd-shift-p; the linear first-match scan is order-safe. All 12 existing keymap/
  palette tests still pass.
- **Findings (all Phase-4 test-completeness; NO code fix):**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | F1 | MED | finder.rs ships with no tests → MSI/cov 0 (14/14 viable mutants missed). | P4 writes the FinderState tests — critic's 15-test mirror reaches MSI 100 / cov 100. |
  | F2 | MED | The new cmd-p binding is asserted by NO keymap test → a wrong shift-bit/action would pass silently. | P4 adds `action_for(cmd-p)=="open-file-finder"` (+ keep the cmd-shift-p assert). |
  | F3 | LOW | `results` hollow-MSI: a single-match assert leaves the `vec![0]` mutant alive. | P4 asserts empty→`[0,1,2]` AND a non-zero-leading case (`"main"`→`[2,1]`, also proves `.index` = original file index). |
  | F4 | LOW | push/backspace reset not mutation-forced (whole-body mutant only). | P4 asserts `selected()==0` after editing from a MOVED selection. |
  | F5 | LOW | overlay renders 20 rows but move_down clamps to full len → arrowing past 20 hides the highlight (Enter still writes the right path, no panic). | Accept — deferred in-code (scroll is later). |
- **Verified (probe):** move_down len=3 (0→1→2→2), len=1→0, **len=0 no panic** (saturating_sub guards);
  move_up saturates; push/backspace reset + backspace-empty no panic; results empty→[0,1,2]/"app"→[0]/
  "main"→[2,1] (original index); chosen [2,0]→c/a, out-of-range/empty→None, double-get panic-free.
  Shim sound: enter-arm scopes the borrow → owned bytes before workspace mut; `write_bytes` has NO
  trailing `\n` (inserts, not executes); overlays mutually exclusive; render/handle_finder_key/
  dispatch_action carry mutants::skip + app.rs cov-excluded. §20 mirrors the palette pattern.
- **No code change** — F1–F4 are P4 tests, F5 accepted.

## Phase 4 — Validate
- **Tests added:** `finder.rs` — `edit_resets_selection` (REQ-001 + backspace-empty no-panic),
  `move_clamps` (REQ-002 + move_down(0) no underflow), `results_ranks_via_fuzzy_rank` (REQ-003 — empty→
  `[0,1,2]`, "app"→`[0]`, "main"→`[2,1]` proving `.index` = original index; F3), `chosen_selected_or_none`
  (REQ-004 + empty→None; F4). `keymap.rs` — added the `action_for(cmd-p)=="open-file-finder"` assertion
  (F2, pins the shift bit so cmd-p ≠ cmd-shift-p).
- **Runs (actual):** `cargo nextest -p marley -E 'test(finder) or test(keymap)'` → 7 passed; workspace
  158 passed.
- **SELF-TEST (UI — REQ-005, drove the LIVE app):** rebuilt + launched the bare binary from the project
  dir; added a `cmd:` chord action to `drive.swift`; drove `focus cmd:p` → the finder overlay OPENED
  (my 📄 icon confirms it's the finder, not the 🔍 palette) rendering the project files with the top row
  `.cargo/audit.toml` **highlighted in the accent color** (`scratchpad/finder.png`). This proves cmd-P
  opens the finder + the keymap NON-CONFLICT live (bare cmd-P ≠ cmd-shift-p) + the ranked-list-with-
  highlight render.
- **Honest harness limitation (stated per §rule):** the type-to-FILTER could not be driven — the finder
  (like the palette) reads `keystroke.key_char`, which macOS does NOT populate for the harness's
  synthetic keycode events (the terminal path reads `.key`, which is why `echo hello` typed fine
  earlier; same keycode-vs-character gap as the shell). Real keyboard input sets `key_char`, so users
  are unaffected. The filter itself is UNIT-verified at MSI 100 (`results`/`push`); only the live
  keystroke→query path is un-driveable.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0% (finder.rs
  14/14 viable killed). app.rs shim + drive.swift excluded/N-A as designed.
- **Pre-existing:** none.

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Added`; `marley_search_core.md` (fuzzy file-open now a shipped consumer).
- **Knowledge:** PR `PR-claude-selftest-cannot-drive-key-char-ui-input-001` (the harness can't drive
  key_char-based typing — it proves OPEN+RENDER, filter is unit-tested). aar-submit `completed` (5).
- **Ticket:** forge #57 → done; local doc → closed/; pair archived. **5/6 of M2.A.** Only #58 (Details
  dock) remains.
