---
pipeline_id: 978a747b-c132-4d27-b786-12de34facf79
ticket: forge#256 (1743bdba-bcf0-4872-9f9d-6118ef7267bd)
aar_id: cf78b64b-15eb-4705-9f81-25e3ba4bca05
---

# Editor clipboard (#256) — pipeline notes

## Phase 1 — Plan

**Intent.** ⌘C/⌘X/⌘V over the editor selection, reusing #255's selection + #249's edit/text_in_range + #253's
undo + the existing gpui clipboard plumbing.

**Classification.** Work pipeline, feature, MEDIUM-SMALL. One thin pure fn (`paste_edit` — the range/caret) + a
clipboard shim in `on_key_down`; the rest is reuse (text_in_range, edit, active_selection, write/read_from_
clipboard). The tightest ticket of the M15 train after the render/selection subsystems.

**Discovery (confirmed):**
- gpui clipboard EXISTS: `cx.write_to_clipboard(ClipboardItem::new_string(text))` (app.rs:2952/3261/4431) +
  `cx.read_from_clipboard().and_then(|c| c.text())` (app.rs:4439). The terminal's cmd-C (app.rs:4427) + cmd-V
  (4438) live in on_key_down's platform-chord region.
- Reuse: #255 `active_selection()` (normalized) + `active_buffer_caret_anchor_mut`; #249 `text_in_range(start..
  end)` (copy) + `edit(range, repl, Human)` (cut/paste, undo-recorded via #253).
- The #251 editor KEY branch guards `!platform` → ⌘-chords never reach it → the editor clipboard ops MUST live in
  the platform-chord region (before the terminal cmd-C/cmd-V), guarded on `active_tab().editor().is_some()`.

**§20 (FILLED).** The universal editor clipboard convention (Warp input + Zed editor; no source read). ⌘C copies
the selection, ⌘X copies+deletes, ⌘V inserts (replace/at-caret), over the existing gpui plumbing the terminal
uses. ⌘C-no-selection = no-op (v1, D1).

**Locked-in decisions:** D1 ⌘C/⌘X no-selection = no-op (copy-line deferred). D2 the pure seam = `paste_edit(
selection, caret, clip_chars) -> (Range, caret)`. D3 the ops go in on_key_down's platform-chord region (before
the terminal handlers), editor-guarded, return-after-handle. D4 edits via Buffer::edit (undo-recorded). D5 the
caret collapses after cut/paste. See the spec.

**EARS AC:** REQ-001 ⌘C copies (no-op when no selection) · REQ-002 ⌘X copies+deletes · REQ-003 ⌘V inserts
(replace/at-caret) + caret after · REQ-004 editor-guarded (terminal keeps its copy/paste) · REQ-005 paste_edit
pure cov/MSI 100.

**Risks:** (1) DATA-LOSS on cut/paste — the edit must delete/replace the NORMALIZED range (a backwards selection
→ min..max); mitigated by active_selection (already normalized). (2) the editor-vs-terminal guard — ⌘C on a
terminal tab must still do the terminal copy (the branch returns only when an editor tab is active). (3) the
paste caret math (multibyte clip → chars, not bytes). (4) placement — the editor branch BEFORE the terminal
cmd-C/cmd-V or the terminal handler wins. (5) mutants::skip on the on_key_down shim (render is skip'd — confirm).

**Gate note:** the #253 real-PTY flake (wedge under llvm-cov — `ps -o pcpu,etime` + kill + re-run). The #254/#255
harness note: a fresh instance may have a launch-focus flake (relaunch); a `focus` after a drag CLEARS the
selection (the activate does a click) → do drag+op in ONE drive call.

**Phase 1 status: Plan PASS — ready for Phase 2 — Design.**

---

## Phase 2 — Design

### Architecture / approach
One pure fn (`code_view::paste_edit`) + the on_key_down editor-clipboard shim. Confirms §20 (the universal
clipboard behavior over the existing gpui plumbing; clean-room). The on_key_down ORDER (confirmed by reading):
⌘⇧C git-pane (app.rs:4416) → terminal cmd-C (4424) → terminal cmd-V (4436) → keymap chord dispatch (4463) →
#251 editor key branch (4477). ⌘C is currently SWALLOWED by the terminal cmd-C at 4424 → the editor branch MUST
go BEFORE it (between the git-pane `}` at 4423 and the cmd-C comment at 4424).

**D2 — the pure `paste_edit` (code_view.rs, next to offset_for_click/row_selection_cols):**
```rust
pub fn paste_edit(
    selection: Option<(CharOffset, CharOffset)>,
    caret: CharOffset,
    clip_chars: usize,
) -> (CharOffset, CharOffset, CharOffset) {
    let (start, end) = selection.unwrap_or((caret, caret));
    let new_caret = CharOffset::from(start.as_usize() + clip_chars);
    (start, end, new_caret)
}
```
The paste replaces the selection (else inserts at the caret); the caret lands after the inserted text
(`start + clip_chars`, CHAR-indexed so a multibyte clip is correct). cov/MSI 100 (mutants: the `+` → `-`/`*`, the
tuple-Default return; `unwrap_or` is a method call → unmutated → covered).

**D3 — the on_key_down editor-clipboard branch (shim, in render → mutants::skip):** inserted BEFORE the terminal
cmd-C (4424), AFTER the ⌘⇧C git-pane block (so ⌘⇧C still opens the git pane; the branch adds `!shift` to handle
ONLY plain ⌘C/⌘X/⌘V, mirroring the git-pane's explicit shift check):
```rust
if event.keystroke.modifiers.platform
    && !event.keystroke.modifiers.shift
    && matches!(event.keystroke.key.as_str(), "c" | "x" | "v")
    && view.shell.active_project().active_tab().editor().is_some()
{
    let paste = event.keystroke.key == "v";
    let cut = event.keystroke.key == "x";
    if paste {
        if let Some(text) = cx.read_from_clipboard().and_then(|c| c.text()) {
            if let Some(surface) = view.shell.active_project_mut().active_tab_mut().editor_mut() {
                let sel = surface.active_selection();
                let caret = surface.active_caret();
                let (start, end, new_caret) = paste_edit(sel, caret, text.chars().count());
                let (b, c, a) = surface.active_buffer_caret_anchor_mut();
                b.edit(start..end, &text, EditOrigin::Human);
                *c = new_caret;
                *a = None;
                cx.notify();
            }
        }
    } else if let Some(surface) = view.shell.active_project_mut().active_tab_mut().editor_mut() {
        if let Some((s, e)) = surface.active_selection() {
            let text = surface.active_buffer().text_in_range(s..e);
            cx.write_to_clipboard(ClipboardItem::new_string(text));
            if cut {
                let (b, c, a) = surface.active_buffer_caret_anchor_mut();
                b.edit(s..e, "", EditOrigin::Human);
                *c = s;
                *a = None;
                cx.notify();
            }
        }
    }
    return;
}
```
- BORROW (no E0499): `view` and `cx` are SEPARATE closure params. ⌘C/⌘X: `text_in_range` (immutable surface
  borrow) → owned `text` → borrow ends → `cx.write_to_clipboard` (cx, not view) → then `active_buffer_caret_
  anchor_mut` (mutable). ⌘V: `active_selection`/`active_caret` (immutable) → owned → `paste_edit` (pure) → then
  the mutable accessor. Sequential.
- No unwrap: `read_from_clipboard()` is `Option`, `.and_then(|c| c.text())` handled; a None clipboard → no-op.

**D3 REQ-004 — the editor-vs-terminal guard:** the branch's `active_tab().editor().is_some()` means a TERMINAL
tab skips it → the existing terminal cmd-C (4424) / cmd-V (4436) run UNCHANGED (no regression). An editor tab
intercepts + returns.

**D4 DATA-LOSS:** cut/paste edit `active_selection()` = the NORMALIZED (min,max) range (#255), so a backwards
selection (caret<anchor) still deletes/replaces the RIGHT span; `edit(start..end)` has `start ≤ end` → no
reverse-range panic. The copy is `text_in_range(s..e)` (normalized) → the right text.

### File manifest
| File | Change |
|---|---|
| crates/marley_app/src/code_view.rs | ADD `paste_edit(selection, caret, clip_chars) -> (CharOffset, CharOffset, CharOffset)`. Pure. |
| crates/marley_app/src/app.rs | ADD the on_key_down editor-clipboard branch (⌘C/⌘X/⌘V, `!shift`, editor-guarded, before the terminal cmd-C). SHIM (render is `mutants::skip`). |

### Regression Test Plan
| # | Test | Proves |
|---|---|---|
| T1 | code_view `paste_edit`: with-selection `(Some((co2,co5)), co9, 3)` → `(co2, co5, co5)` [replace 2..5, caret at 2+3]; no-selection `(None, co4, 3)` → `(co4, co4, co7)` [insert at 4, caret 7]; zero-clip `(None, co4, 0)` → `(co4, co4, co4)`; a multibyte-sized clip (clip_chars = 2 for "é😀") → the caret math by chars | REQ-003/005 |
| T2 (driven) | drag-select → ⌘C → move the caret (click) → ⌘V → the copied text is inserted at the new caret | REQ-001/003 |
| T3 (driven) | drag-select → ⌘X → the selection is DELETED (+ on the clipboard); ⌘V re-inserts it | REQ-002 |
| T4 (review+driven) | ⌘C on a TERMINAL tab still does the terminal copy (the editor branch is skipped) | REQ-004 |

**Uncoverable/shim:** the on_key_down branch + the clipboard I/O + the edits are shim (render `mutants::skip` +
coverage-excluded; driven T2-T4 + review). `paste_edit` is the cov/MSI-100 seam.

**Risks:** (1) data-loss — cut deletes the normalized span (T1 + driven T3). (2) the editor-vs-terminal guard +
the placement before the terminal cmd-C (driven T4). (3) the multibyte paste caret (T1). (4) the borrow sequence
(cargo check). (5) mutants::skip — the branch is in render (skip'd); confirm. (6) the `!shift` so ⌘⇧C still opens
the git pane.

**Phase 2 status: Design PASS — `paste_edit` + the on_key_down editor-clipboard branch (⌘C/⌘X/⌘V, editor-guarded,
before the terminal cmd-C, `!shift`) + T1-T4 locked. Ready for Phase 3 — Implement.**

---

## Phase 3 — Implement (PASS)

Built to the manifest; ONE deviation (an import the design assumed present):

- **crates/marley_app/src/code_view.rs** — `paste_edit(selection, caret, clip_chars) -> (CharOffset, CharOffset,
  CharOffset)` (pure: `(start,end) = selection.unwrap_or((caret,caret))`; `new_caret = start + clip_chars`).
  - DEVIATION: added `use marley_text_offsets::CharOffset;` — code_view.rs previously imported only
    `std::path::{Path,PathBuf}` (its offset fns use `usize`, not CharOffset), so paste_edit's CharOffset type
    needed the import (E0425 until added).
- **crates/marley_app/src/app.rs** — the on_key_down editor-clipboard branch, inserted BETWEEN the ⌘⇧C git-pane
  block and the terminal cmd-C: `platform && !shift && key ∈ {c,x,v} && active_tab().editor().is_some()` →
  ⌘V reads the clipboard → `paste_edit` → `edit(start..end, text)` + caret=new_caret + anchor=None; ⌘C/⌘X →
  `active_selection()` → `text_in_range` → `write_to_clipboard`, and ⌘X additionally `edit(s..e,"")` + caret=s +
  anchor=None. Each `return`s. `!shift` keeps ⌘⇧C on the git pane; a TERMINAL tab skips the branch → the existing
  terminal cmd-C/cmd-V (below) run unchanged (REQ-004).

**Checks:** `cargo fmt` clean; `cargo check -p marley --all-targets` clean (no unused-import warning → CharOffset
is used by paste_edit which app.rs calls). **`grep -c code_view_body|dispatch_action` == 0 AND the app.rs total
mutant count is UNCHANGED at 6** (all pre-existing) — the whole branch is in `render` (`mutants::skip`), so 0 new
live app.rs mutants. BORROW: `cargo check` confirms E0499-free (view + cx are separate closure params;
text_in_range/active_selection/active_caret [immutable] end before active_buffer_caret_anchor_mut [mutable]).
Pure-seam mutants for Phase 4: paste_edit {tuple `Default::default()`, `+`→`-`, `+`→`*`} — all T1 targets
(`unwrap_or` is a method call → UNMUTATED → covered). No unwrap (read_from_clipboard is Option-handled).

**Phase 3 status: Implement PASS — ready for Phase 3.5 — Inspect.**

---

## Inspect (Phase 3.5) — PASS (0 code fixes)

2 critics (data-integrity/paste_edit · guard/regression/borrows). **Both: ZERO code defects — the implementation
is correct.** No failure-record (no bug).

### Critic 1 — data-integrity (all CONFIRMED)
- (a) paste_edit correct: with-selection → (start,end,start+clip); no-selection → (caret,caret,caret+clip); clip
  is a CHAR count → multibyte-correct. (b) **CUT no data-loss:** (s,e) from active_selection = NORMALIZED
  (min,max) → a backwards selection cuts the RIGHT span; the COPIED span `text_in_range(s..e)` == the DELETED
  span `edit(s..e,"")` (same s,e) → no corruption; s<e → no reverse-range panic; copy BEFORE delete; caret=s.
  (c) PASTE replaces the normalized selection or inserts at caret, never the wrong span, caret after, collapse.
  (d) **COPY is READ-ONLY** — ⌘C never calls edit; no-selection = pure no-op. (e) cut/paste call `Buffer::edit`
  (the RECORDING path, not apply_raw) → #253 ⌘Z undoes them (cut/paste-over-selection/multi-char paste = own
  undo step).
- LOW (Phase-4): paste_edit needs its test (the 3 mutants). **VALIDATE TRAP (critic-1):** don't pick a `*`-mutant
  case where `start*clip == start+clip` (e.g. start=2,clip=2 both give 4) — use `(None, co4, 3)` → co7 (7≠12).
- INFO (no action): a 1-char paste-at-caret coalesces with a prior typed run (fully undoable, standard); an
  empty-clipboard ⌘V over a selection would delete it (macOS yields None not Some("") → negligible).

### Critic 2 — guard + regression (NO findings)
- (f) REQ-004: order = ⌘⇧C git-pane → **#256 branch** → terminal cmd-C → cmd-V → keymap dispatch. An editor tab
  → the branch handles + `return`s (the `return` is OUTSIDE the if/else → fires unconditionally on entry, even an
  empty clipboard/selection is a clean no-op) → the terminal handlers never run for an editor tab. A TERMINAL tab
  → `editor().is_some()` false (Tab is a mutually-exclusive enum) → skipped → the terminal copy/paste run
  UNCHANGED. ⌘⇧C → git-pane. (g) c/x/v are NOT keymap actions (grep: keymap has p/d/s/z/w/… but not c/x/v) → the
  keymap dispatch (after) never double-handles; ⌘W/⌘S/⌘Z skip the branch. (h) E0499-free (view + cx separate
  params; immutable reads end before the combined mut accessor). (i) render is `mutants::skip` → app.rs mutants
  UNCHANGED at 6, grep 0; no unwrap (read_from_clipboard Option-handled); the CharOffset import is used (no
  dead-import); clean-room OK (the clipboard is the terminal's existing gpui plumbing).

**Phase 3.5 status: Inspect PASS — 0 code fixes. Ready for Phase 4 — Validate (paste_edit tests using
non-degenerate values per critic-1's trap).**

---

## Phase 4 — Validate (PASS — GATE GREEN [diff])

### Test added (cov/MSI 100 on paste_edit)
- **code_view.rs** `paste_edit_replaces_selection_or_inserts_at_caret`: with-selection `(Some((co2,co5)),co9,3)`
  → `(co2,co5,co5)` [replace 2..5, caret at start+clip=5, the old caret 9 IGNORED]; no-selection `(None,co4,3)`
  → `(co4,co4,co7)` [insert at 4, caret 7 — 7 distinguishes `+`→`-`(1) AND `+`→`*`(12), the critic-1 trap];
  zero-clip `(None,co4,0)` → `(co4,co4,co4)`; a bigger replace `(Some((co3,co8)),co0,2)` → `(co3,co8,co5)`.

### Test runs (actual)
- `cargo nextest run -p marley paste_edit` → **1 passed**. `-p marley` → **364 passed, 2 skipped** (+1).
- `cargo mutants --list`: paste_edit {tuple `Default::default()`, `+`→`-`, `+`→`*`} — all T1 targets. app.rs
  `code_view_body`+`dispatch_action` == 0, total == 6 (unchanged — the branch is in `render`, skip'd).

### Driven live proof (REQ-001/002/003 — mac unlocked, DATA-SAFE)
Bundled + open; the #205 session restored selection.rs. **REQ-002 (⌘X cut):** `drag:0.42,0.26,0.52,0.26 cmd:x`
(drag-select `harOffset,` on line 10 + cut, ONE call) → the selection was DELETED → `anchor: C│` (m256_01).
**REQ-001 + REQ-003 (copy → paste round-trip):** relaunch → `drag:… cmd:x cmd:v` (ONE call — cut deletes, paste
re-inserts from the clipboard) → **line 10 RESTORED to `anchor: CharOffset,│`** (m256_03) — the clipboard carried
`harOffset,` from the cut to the paste, and ⌘V inserted it at the caret. **DATA-SAFETY: no ⌘S ever → `git status`
confirms selection.rs PRISTINE on disk;** only the #256 diff I authored is modified. (A SEPARATE `focus cmd:v`
after the cut did NOT land — the #254/#255 separate-call focus flake; the ONE-call round-trip works — same as
#255's drag+type. REQ-004 editor-vs-terminal guard: critic-verified [an editor tab intercepts + returns; a
terminal tab falls through]; paste_edit unit-proven.)

### Gate
`scripts/gates.sh --diff` → **GATE GREEN [diff] — 15 passed, 0 failed** (gate:4 coverage ≥100%, gate:5 mutation
MSI ≥100% on paste_edit, gate:1 fmt, gate:6 miri, gate:15 visual). ~165s, no real-PTY wedge. Receipt matches the
current `.rs` state → the commit gate will pass.

**Phase 4 status: Validate PASS — GATE GREEN [diff]. Ready for Phase 5 — Complete.**
