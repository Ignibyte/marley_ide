---
pipeline_id: 99180a00-55d9-49ba-a84a-87873485b831
ticket: forge#252 (6fba8467-f1bf-4a9c-ad79-88938c0b6cba)
aar_id: c3576454-eafb-4236-b84d-2cc8599209d1
---

# Notes — Editor save (forge#252)

## Plan (Phase 1)

**Classification:** work pipeline, feature, medium. AUTONOMOUS (chad's `/goal /work 249 to 258`). #251 made the
editor editable (typing → dirty); #252 lets you SAVE + shows the unsaved ●.

**Discovery (anchors):**
- `OpenFile::is_dirty()` (editor_surface.rs:39, private) = `buffer.version() != saved_version`; `mark_saved`
  (:43); `EditorSurface::active_is_dirty()` (:167) + `active_mark_saved()` (:162) — the ACTIVE file only. The
  strip shows ALL files → need a per-file exposure (D2: `file_dirty_flags()`).
- The #237 tab strip renders at app.rs:4981 (`surface.files().enumerate()` → name · ×); the ● goes beside the
  name (shim).
- The keymap is PURE (`keymap.rs`): `Keymap { bindings: Vec<(KeyBinding, String)> }`, `default_bindings()`
  (:67) binds `(⌘⇧P,"open-command-palette")`, `(⌘D,"new-terminal")`, `(⌘W,"close-pane")`, etc.; `action_for`
  is the pure lookup. `dispatch_action` (app.rs:3379) matches the action string. → D3: add `(⌘S,"save")` +
  a `"save"` arm.
- NO fs-write helper exists → D1: `save_active` on the app (shim) does the `fs::write` + `active_mark_saved`.

**Decisions:** D1 save_active in the app.rs shim (editor_surface stays pure; mark_saved only on Ok) · D2 pure
`file_dirty_flags()` for the strip · D3 ⌘S via the keymap action ("save") + a guarded dispatch arm · D4 a save
error → a status flash, not a panic.

**Risks / load-bearing:**
- **Mark-clean-ONLY-on-success** (REQ-004) — a failed `fs::write` must leave `saved_version` unchanged (the ●
  stays). The ordering (`write()?` BEFORE `mark_saved()`) is the guard; the `?` short-circuits on Err.
- **Data-safety in the DRIVEN test** — ⌘S writes to disk. Use a THROWAWAY temp .txt (create, open, type, ⌘S,
  verify, delete). NEVER a real repo file (a stray save would dirty the tree).
- **Per-file dirty** (not just active) — the strip marks every dirty file; `file_dirty_flags` zips with
  `files()` in tab order.
- **The ⌘S guard** — ⌘S only saves when an editor tab is active; a terminal tab ⌘S is a no-op (the dispatch arm
  checks `active_tab().editor().is_some()`).

**Reference (§20):** Warp's edited/unsaved indicators (ticket-cited) + the universal editor ●/⌘S convention;
behavior/convention reference, no Warp/Zed source. Filled in the spec.

**Test plan (finalized at design):** REQ-001/005 pure units on `file_dirty_flags` (per-file dirty; independent;
in tab order); REQ-003/004 the mark-on-Ok logic (a testable helper: mark_saved clears dirty; NOT called on Err
— test by asserting `active_is_dirty()` after a simulated Ok vs an un-marked state); the `(⌘S,"save")` binding
via `action_for`. cov/MSI 100. REQ-002/003 DRIVEN (the real write to a temp file).

**AAR:** c3576454-eafb-4236-b84d-2cc8599209d1 (opened).

**Phase 1 status: Plan PASS — autonomous (M15 /goal). Ready for Phase 2 — Design.**

## Design (Phase 2)

**## Reference (§20) confirmed:** Warp's edited/unsaved indicators + the universal editor ●/⌘S convention. This
design MATCHES it — a ● on a file tab when `version != saved_version`, cleared by ⌘S which writes to disk. A
behavior/convention reference; the fs write is Rust std, the keymap/flash are in-repo; no Warp/Zed source.

**Architecture.** One PURE addition in `editor_surface.rs` (`file_dirty_flags`) + a PURE binding in `keymap.rs`
(`(⌘S,"save")`) + app.rs shim (`save_active` fs-write, the `"save"` dispatch arm, the ● in the tab strip). The
write stays in the app shim so `editor_surface` remains pure.

### THE PER-FILE DIRTY — RESOLVED (editor_surface.rs, PURE)
```
pub fn file_dirty_flags(&self) -> impl ExactSizeIterator<Item = bool> + '_ {
    self.files.iter().map(|f| f.is_dirty())      // parallel to files(), tab order
}
```
cov/MSI 100 — test: a fresh surface → all false; edit the active buffer → its flag true, others false; per-file
independent. `files()` UNCHANGED (no #250/#243 caller churn).

### THE SAVE — RESOLVED (app.rs shim, D1; mark ONLY on Ok)
```
fn save_active(&mut self) -> std::io::Result<()> {
    let Some((path, text)) = self.shell.active_project().active_tab().editor()
        .map(|s| (s.active_file().path.clone(), s.active_buffer().text())) else { return Ok(()); };  // no editor → no-op
    std::fs::write(&path, text)?;                 // IO; `?` returns BEFORE mark_saved on Err (REQ-004: stays dirty)
    if let Some(s) = self.shell.active_project_mut().active_tab_mut().editor_mut() { s.active_mark_saved(); }
    Ok(())
}
```
- Borrow-dance: the `.map` yields OWNED `(PathBuf, String)`, the immutable borrow ends → `fs::write` → the
  mutable `editor_mut().active_mark_saved()`. Sequential, NLL-fine.
- §14: typed `io::Result`, NO unwrap; a `None` editor → `Ok(())` no-op (⌘S on a terminal tab is safe — REQ-005).
- SHIM (fs IO) → `#[cfg_attr(test, mutants::skip)]` + coverage-excluded (driven + review carry it).

### ⌘S → "save" — RESOLVED (keymap.rs PURE + app.rs shim, D3)
- keymap.rs `default_bindings` (:69 vec): ADD `(KeyBinding::chord(true, false, false, false, "s"),
  "save".to_string())` (⌘S — `chord(cmd,ctrl,alt,shift,key)` confirmed :25). cov/MSI 100: `default_bindings()
  .action_for(&KeyBinding::chord(true,false,false,false,"s")) == Some("save")` (+ a negative: a bare "s" → None).
- app.rs `dispatch_action` (:3379): ADD `"save" => { if let Err(e) = self.save_active() { self.status_flash =
  Some(Flash::new(format!("Save failed: {e}"))); } }` — on Ok the ● clears (via mark_saved) + the caller's
  `cx.notify()` (dispatch site app.rs:~4267) re-renders; on Err a flash (REQ-004 visible). No cx needed in the
  arm (status_flash is a field). SHIM.

### THE ● RENDER — RESOLVED (app.rs strip ~4981, shim)
Zip the dirty flags with the files: `for (i, (file, is_dirty)) in surface.files().zip(surface.file_dirty_flags())
.enumerate()` (both immutable borrows). Build the tab as `[dirty_dot?, name, ×]`: when `is_dirty`, prepend
`div().text_color(colors.accent).child("●")` (an accent unsaved dot — "●" U+25CF is a MONO primitive, bucket-C
keep per the #232 icon-audit, NOT a colorful emoji). SHIM.

### File Manifest
| File | Change |
|---|---|
| crates/marley_app/src/editor_surface.rs | ADD pure `file_dirty_flags() -> impl ExactSizeIterator<Item = bool>`. Tests-phase. |
| crates/marley_app/src/keymap.rs | ADD `(⌘S,"save")` to `default_bindings`. Tests-phase (action_for). |
| crates/marley_app/src/app.rs | ADD `save_active` (shim, mark-on-Ok) + the `"save"` dispatch arm + the ● in the tab strip (zip file_dirty_flags). SHIM. |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | `file_dirty_flags`: fresh surface (1-2 files) → all false; after `active_buffer_mut().edit(..)` → the active flag true, others false; edit a 2nd file → both independent | REQ-001/005 |
| T2 | keymap: `default_bindings().action_for(&chord(true,false,false,false,"s")) == Some("save")`; a bare `"s"` (no cmd) → None (⌘S is the trigger, not plain s) | REQ-002 (binding) |
| T3 | mark-on-Ok logic (via the existing #249 `active_mark_saved`): edit → `active_is_dirty()` true; `active_mark_saved()` → false (proves save's success path clears; save_active calls it ONLY after `write()?`) | REQ-003 |
| T4 (review) | `save_active`: the `?` is BEFORE `active_mark_saved()` → an Err leaves the file dirty (REQ-004); a `None` editor → `Ok(())` no-op (REQ-005). Code-review invariant (fs Err hard to unit-force). | REQ-004/005 |
| T5 (driven) | a SCRATCH temp .txt: open → type → a ● appears on its tab; ⌘S → the ● CLEARS + the file on disk has the typed text; then DELETE the temp. | REQ-001/002/003 |
| — | `cargo mutants --list -f editor_surface.rs` + `-f keymap.rs` — kill file_dirty_flags + the binding mutants. | MSI 100 |

**Uncoverable / shim:** `save_active` (fs IO) + the ● render + the dispatch arm are shim (`mutants::skip`/
coverage-excluded) — driven (T5) + review (T4). The fs-write Err path is REVIEW (forcing a real write failure in
a unit is over-engineering; the `?`-before-mark ordering is the invariant).

**Risks:** (1) mark-clean-ONLY-on-Ok — the `?`-before-mark ordering (T4 review + T5 driven happy-path). (2)
data-safety in T5 — a THROWAWAY temp file only, deleted after. (3) the borrow-dance (owned path/text then mut) —
compiles. (4) per-file dirty in tab order — the zip.

**Phase 2 status: Design PASS — file_dirty_flags (pure) + save_active (shim, mark-on-Ok) + the (⌘S,"save")
binding (pure) + the ● strip render + T1-T5 locked. Ready for Phase 3 — Implement.**

## Implement (Phase 3)

**Built (to the manifest, exactly as designed):**
- **editor_surface.rs** — `pub fn file_dirty_flags(&self) -> impl ExactSizeIterator<Item = bool> + '_`
  (`self.files.iter().map(|f| f.is_dirty())`, tab order). Pure. `files()` UNCHANGED.
- **keymap.rs** — added `(KeyBinding::chord(true, false, false, false, "s"), "save".to_string())` to
  `default_bindings` (after the ⌘D entry). Pure.
- **app.rs** — (a) `save_active(&mut self) -> std::io::Result<()>` (`#[cfg_attr(test, mutants::skip)]` — fs IO):
  reads OWNED `(path, text)` from the active editor (None → `Ok(())` no-op), `std::fs::write(&path, text)?`
  (the `?` returns BEFORE mark), then `editor_mut().active_mark_saved()` — mark ONLY on Ok (REQ-004). (b) the
  `"save"` arm in `dispatch_action` → `save_active()`, on `Err` sets `status_flash = Some(Flash::new(format!
  ("Save failed: {e}")))` (`Flash::new` takes a `String` — matched app.rs:699). (c) the tab strip (~4981):
  `for (i, (file, is_dirty)) in surface.files().zip(surface.file_dirty_flags()).enumerate()`; when `is_dirty`,
  prepend `div().text_color(colors.accent).child("\u{25cf}")` (the ● unsaved dot) before the name — the ×
  handler + active-idx styling + activate handler UNCHANGED.

**Deviations from design:** none — the design's signatures + placements compiled as written (the save_active
borrow-dance [owned path/text → fs::write → mutable mark] is NLL-fine; the tab_el chain split for the conditional
● compiled).

**Compile:** `cargo check -p marley --all-targets` clean. Do NOT expand tests here (Phase 4 adds T1-T5).

**Phase 3 status: Implement PASS — file_dirty_flags + the (⌘S,"save") binding + save_active (mark-on-Ok) + the
"save" dispatch arm + the ● strip render all compile clean. Ready for Phase 3.5 — Inspect.**

## Inspect (Phase 3.5)

2 parallel general-purpose critics (C1 = data-integrity + save correctness; C2 = regression + reuse +
clean-room) + self-review. **The 5 save-correctness properties HOLD — no data-loss.** **2 HIGH folded (both
gate-blocking, both FIXED) + 1 LOW.** After the fixes: `cargo check` clean, keymap:: 10/10,
`dispatch_action` mutants back to 0, `save_active` skipped.

**Save correctness — CONFIRMED CLEAN (the highest-stakes checks):**
- **(a) RIGHT bytes → RIGHT path** — `save_active` reads `(active_file().path.clone(), active_buffer().text())`
  from the SAME active `OpenFile` (both index `files[active]`). C1 traced the path to be ABSOLUTE (set at open
  via `resolve_under_root(project_root, path)`, `project_root` from `current_dir()`) → `fs::write` hits the exact
  file that was opened, CWD-independently. NO data-loss / wrong-file risk. REJECTED as a defect.
- **(b) MARK-ONLY-ON-OK** (REQ-004) — `fs::write(&path, text)?;` is strictly BEFORE `active_mark_saved()`; the
  `?` returns on Err before the mark → a failed write leaves the file dirty (● stays) + a status flash. REJECTED.
- **(c) borrow + no panic** — owned (path, text) → immutable borrow ends → fs::write → mutable `editor_mut()`;
  None editor → `Ok(())` no-op (REQ-005); no unwrap. REJECTED.
- **(d) file_dirty_flags order** — both `files()` and `file_dirty_flags()` iterate `self.files` in order → the
  strip zip pairs file[i]'s view with file[i]'s dirty → the ● lands on the correct tab. REJECTED.
- **(e) ⌘S routing** — the cockpit-chord dispatch (`action_for` → `dispatch_action`) fires ⌘S FIRST (it's
  before the #251 editor branch, which skips ⌘S via `!platform`) → `"save"` → `save_active`. ⌘S is free (only
  ⌘⇧S = send-to-agent, a distinct chord). REJECTED.
- **C2 regression/reuse:** the non-dirty tab is byte-identical (the ● is added only under `if is_dirty`); the ×
  handler + active-idx styling + activate closure UNCHANGED; the ● glyph `"\u{25cf}"` is a MONO primitive
  (bucket-B/C keep per the #232 icon-audit; the exact `.child("\u{25cf}")` idiom already at app.rs:4731) — NOT a
  new emoji; save_active reuses active_file/buffer/mark_saved/status_flash — no dup; clean-room (std fs +
  in-repo keymap/Flash + the standard editor ●/⌘S convention).

**Findings folded (2 HIGH + 1 LOW):**
- **F1 [HIGH] the `mutants::skip` DETACH TRAP recurred** (C2) — inserting `save_active` (with its own doc+skip)
  directly above `fn dispatch_action` stranded `dispatch_action`'s doc+skip (a doc/attribute binds to the NEXT
  item) → both attached to `save_active`, leaving `fn dispatch_action` with NO skip → **11 live mutants → MSI
  RED**. THE EXACT KNOWN TRAP ([[mutants-skip-detach-trap]]; `PR-claude-recheck-mutants-skip-after-refactoring-
  shims-001`). **FIX:** moved `dispatch_action`'s doc+skip back directly above `fn dispatch_action`; kept
  `save_active`'s own doc+skip above it. Verified: `cargo mutants --list -f app.rs` → 0 `dispatch_action`
  mutants (was 11), 0 `save_active` mutants. `BF-claude-mutants-skip-detach-trap-recurred-252`.
- **F2 [HIGH] the keymap roster-count guard failed** (both critics) — adding `(⌘S,"save")` made
  `all_chords().len()` 38→39, but `all_chords_lists_every_binding` still asserted 38 → the test FAILED (the
  roster guard working AS DESIGNED — it catches an un-updated count). **FIX:** bumped the assert to 39 (comment
  30 vec-literal) + added `assert!(chords.contains(⌘S))`. keymap:: 10/10 now.
- **F3 [LOW] `save_active` had a duplicate doc+skip** (collateral of F1) — FIXED by the F1 regrouping.

**No forge prevention-rule (new)** — F1's rule already exists (`PR-claude-recheck-mutants-skip-after-refactoring-
shims-001`); the recurrence is recorded as a failure (a reminder to RUN `cargo mutants --list` after inserting a
fn near a masked shim — I should have on the #252 implement). F2 is a guard working correctly (no rule).
Lenses: data-integrity (path/bytes/mark), correctness, routing, regression, reuse, clean-room, mutation-hygiene.

**Phase 3.5 status: Inspect PASS — 0 data-loss (save writes right bytes to right absolute path, mark-on-Ok); 2
HIGH folded (F1 detach-trap → skip restored, F2 roster count → 39), 1 LOW. `cargo check` + keymap:: green;
dispatch_action re-skipped. Ready for Phase 4 — Validate.**

## Validate (Phase 4)

**Tests added (the pure seams):**
- **editor_surface.rs** — `file_dirty_flags_per_file_in_tab_order`: 2 files clean at open → `[false,false]`;
  edit the active (/b) → `[false,true]` (per-file, tab order); edit /a too → `[true,true]` (independent).
- **keymap.rs** — `cmd_s_maps_to_save`: `action_for(⌘S) == Some("save")`; `action_for(plain "s") == None` (⌘S is
  the trigger, plain s types). (The roster guard `all_chords_lists_every_binding` was fixed 38→39 +
  `contains(⌘S)` at inspect.)

**Runs (actual):** `cargo nextest run -p marley editor_surface:: keymap::` → all pass (incl the 2 new);
`cargo nextest run -p marley` → **356 passed, 2 skipped** (no regression).

**Mutation:** `file_dirty_flags` → **0 viable mutants** (the `|f| f.is_dirty()` closure is a method call —
unmutated, the #203 rule; T1 gives the required COVERAGE). keymap `default_bindings` → the `Default::default()`
mutant killed by the existing action_for tests + `cmd_s_maps_to_save`. **`dispatch_action` → 0 mutants**
(re-confirmed the detach-trap fix held — the skip is restored; the fs-shim `save_active` is skip). Gate ran the
real mutants → **MSI ≥ 100%**.

**DRIVEN — REAL CAPTURE (mac unlocked):** the dirty **●** is proven live. I dismissed the finder + typed "XY"
into the active editor tab (selection.rs) — a cyan **● appeared before "selection.rs"** on the tab strip
(`buffer.version() != saved_version`), while the OTHER three tabs (buffer.rs/lib.rs/movement.rs) stayed CLEAN —
proving **REQ-001** (the ● on the edited file's tab) + **REQ-005** (per-file independence). DATA-SAFE: the typing
edited only the in-memory buffer; I never ⌘S'd, and `git status crates/editor/src/selection.rs` is EMPTY (the
disk file is untouched). The literal ⌘S-writes-to-disk (REQ-002/003) could NOT be safely driven: the ⌘P finder
would not open a scratch temp file (a modal-overlay synthetic-click harness limitation, hit across #250/#251/
#252) and data-safety forbids ⌘S on a real repo file — so REQ-002/003/004 are carried by the 2 inspect critics
(who verified `save_active` writes the RIGHT bytes [`active_buffer().text()`] to the RIGHT ABSOLUTE path
[`active_file().path`, resolved under the project root], mark-ONLY-on-Ok via `write()?`-before-mark) + the units
(the ⌘S→"save" binding; `active_mark_saved` clears dirty, #249). Fixture + mutation leftovers cleaned; app quit;
tree = the three #252 `.rs` + docs.

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff]** 15/15 (cov ≥ 100% lines, MSI ≥ 100%, miri,
visual). The receipt for `/commit` is written. cargo-mutants at `--jobs 2` (the memory fix), no freeze.

**Pre-existing (not in scope):** the `block v0.1.6` future-incompat (transitive dep).

**Phase 4 status: Validate PASS — 2 new unit tests (cov/MSI 100 on file_dirty_flags + the ⌘S binding; the
detach-trap fix held → dispatch_action re-skipped), 356 marley tests green, the dirty ● driven-proven live (per-
file), the ⌘S-write mechanism-verified (finder + data-safety blocked the literal driven), GATE GREEN [diff].
Ready for Phase 5 — Complete.**
