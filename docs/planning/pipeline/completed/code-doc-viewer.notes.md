# the code-doc model + viewer overlay — Notes

- **Forge ticket:** #97 `14d9c7b3-0025-4b0d-9919-678f6594fa6d` · **AAR:** `7e428549-7220-4cf4-9360-23753befef6c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-097-code-doc-viewer.md

## Phase 1 — Plan
- **Request:** forge #97 (M4 1/10, FOUNDATION) — the code-doc model + a viewer overlay.
- **Pre-flight:** FinderState (finder.rs) is the pure-state idiom to mirror; marley_project has NO read-file
  fn (only read_dir) → the shim reads via std::fs (seq-2); the finder overlay render is the pattern.
- **Decisions:** D1 split '\n' + 1-based; D2 tab-stop expand + char-truncate; D3 overlay not pane.
- **AAR id:** `7e428549-7220-4cf4-9360-23753befef6c`.

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
- **code_view.rs (NEW PURE):** `CodeLine{number:usize,text:String}`; private `expand_tabs(line,tw)` (per char: '\t'→`tw-(col%tw)` spaces, tracking col; else the char) + `truncate_cols(line,max)` (char-count>max → take(max)+'…'); `code_lines(text,tw,max)` = `text.split('\n').enumerate().map(|(i,raw)| CodeLine{number:i+1, text:truncate_cols(&expand_tabs(raw,tw),max)}).collect()`; `CodeViewState{path:PathBuf,lines:Vec<CodeLine>,scroll:usize}` + `new(path,text,tw,max)` (scroll 0, lines=code_lines).
- **lib.rs:** `mod code_view;` (after color, before complete).
- **app.rs SHIM:** `code_view: Option<CodeViewState>` on RootView (None in new()); a render block `if let Some(cv) = &self.code_view { a centered overlay (absolute, left/top bounds-fraction, occlude, bg surface, fg foreground) with a header (the file name) + each cv.lines row = `{muted number}  {text}`, capped ~40 }`; the key handler: when code_view is Some + Esc → `self.code_view = None` (+ notify). Opening = seq-2.
- **Mutation targets:** code_lines split/number(i+1)/tab-stop(tw-col%tw)/truncate(>max,take,…); CodeViewState::new scroll=0.
- **Test plan:** `code_lines_cases` ("a\\nb"→[1a,2b]; "a\\n"→[1a,2""]; ""→[1""]; "\\tx" tw4→"    x"; "a\\tb" tw4→"a   b"; ">max"→head+…) + `code_view_state_new` (scroll 0, lines match). cov/MSI 100. The overlay + Esc masked (live at seq-2).
- **Risks:** tab_width>0 assumed (a const 4 from the shim; tw=0 would div-by-zero but is never passed — not a real input, not tested). The viewer is an OVERLAY (no pane-kind change); seq-2 wires opening + the first live proof.

## Phase 3 — Implement
- **Built:** code_view.rs (CodeLine + code_lines[split/tab-stop-expand/char-truncate] + CodeViewState + new); `mod code_view` (lib.rs); app.rs `code_view: Option<CodeViewState>` (None boot) + the viewer overlay render (numbered lines, ≤40) + `handle_code_view_key` (Esc closes, modal capture) + `CODE_TAB_WIDTH=4`/`CODE_MAX_COLS=200`.
- **DEVIATION (necessary):** a foundation whose constructor is never called is DEAD CODE → clippy -D warnings FAILS the gate. Pulled a MINIMAL open into #97: the ⌘P finder now opens the chosen file in the viewer on ⌘↵ (a successful read only) — this makes CodeViewState::new/code_lines live + gives #97 a real (if synthetic-input-blocked) open path. seq-2 now scopes to the FILE-TREE-click open + non-file/dir guards + richer affordances.
- **Verification:** fmt; check --all-targets 0 err; clippy OK; the 2 code_view tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review (a small pure line-renderer + a masked overlay/open mirroring the proven finder pattern).
- **Lenses — no findings:** code_lines (split '\n' → 1-based numbers [i+1]; tab-stop expand `tw-col%tw`; char-truncate>max+…; trailing-\n → final empty line; "" → one empty line — all tested); CodeViewState::new (scroll 0, lines=code_lines — tested); the finder ⌘↵ open reads on Ok only (a read error → no open, no crash; path→PathBuf owned to avoid the borrow); Esc closes + the viewer is modal (captures keys so nothing leaks to the terminal); tab_width>0 assumed (const 4, never 0). No panics (no unwrap on the read path). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** code_lines_cases (split/1-based/tab-stop 4→"    x"+"a   b"/truncate+…/exactly-max/trailing-\n/empty) + code_view_state_new (scroll 0, lines==code_lines). `cargo nextest` → 2 passed.
- **Self-test:** the viewer only renders after a ⌘P + ⌘↵ open (synthetic keystrokes, ENV-BLOCKED all session) — and there is NO boot/config path to render it statically (unlike the M2.F dock). code_lines is engine-tested (cov/MSI 100); the overlay render + the finder ⌘↵ open are masked, mirroring the proven finder-overlay + finder-key patterns.
- **Gate finding + fix:** first run RED (MSI 94.7%) — the `col += 1` INSIDE the tab-space loop survived (my ≤1-tab-per-line tests couldnt distinguish it). Fixed at source: a two-tab case `code_lines("a\t\tx",4)` → "a"+7sp+"x" (the 2nd tab depends on the 1st advancing col). Re-gate → **GREEN [diff] 15/15, MSI 100** (19/0).

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(4); forge #97 → done. **M4 1/10 — FOUNDATION.** code_view.rs (CodeLine/code_lines/CodeViewState, cov/MSI 100) + the viewer overlay + finder ⌘↵ open. DEVIATION: pulled a minimal open in (dead-code foundation). Gate caught the tab-loop col+=1 mutant → two-tab test.
