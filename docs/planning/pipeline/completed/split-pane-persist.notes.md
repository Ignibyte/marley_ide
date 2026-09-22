# Persist split-file panes across restart (c=<path> grid codec) — Notes

- **Forge ticket:** #258 (a293b7ac-9714-4fb7-8805-c9f9c2c10ccd)
- **AAR:** 2fdf7580-0f75-45d7-9989-bebb6d09eea8
- **Local ticket doc:** docs/planning/tickets/open/TICKET-258-split-pane-persist.md
- **Pipeline spec:** split-pane-persist.spec.md

## Phase 1 — Plan
- **Request:** forge#258 "Make the split-file pane editable + persist open editor files (#246 follow-on)" — two
  #246 follow-ons: (a) make the read-only split-file `CodeView` pane editable; (b) persist it via a `c=<path>`
  grid codec. The LAST ticket of the M15 `/goal /work 249-258` (autonomous).
- **Classification / tier:** work pipeline, feature, LARGE → **SPLIT** (§3). This pipeline = (b) the codec
  persistence (tight pure seam, low risk). (a) the editable pane → fast-follow **#259** (a model change).
- **Forge recall (§18.3):** no bulletins. Surfaced + fed forward:
  - `PR-claude-persist-verify-trigger-not-just-codec-001` (the #243 persist-TRIGGER trap) — DIRECTLY governs (b):
    verify `persist_grid` is CALLED on `split_file_pane` + driven quit→relaunch, not just the codec round-trip → D4.
  - `AD 902ea928` (the #250 offset↔column model) — relevant to (a)/#259, not this codec pipeline.
  - the #234 relax-invariant / audit-ALL-sites → D3 (both restore paths).
  - the #246/#252 mutants::skip-detach → re-run `cargo mutants --list` + grep shim names if a shim is refactored
    (load_code_view_state may get an extracted load-from-path core).
  - the #253 gate-wedge watchdog (`ps -o pid,pcpu,etime` + kill + re-run if a real-PTY test sits at 0% CPU) + the
    #254/#255 one-drive-call harness note (relaunch on the launch-focus flake).
  - the #205 grid-framing D2 stance (drop a value that breaks the framing) → REQ-001.
- **Discovery (real anchors, confirmed this session):**
  - **(b) codec** — grid_layout.rs: `serialize_leaf(&(PaneKind,Option<String>))` L76 [`(Terminal,Some(cwd))→
    "t={cwd}"`, else bare `kind_char` → CodeView emits `'c'`, path LOST]; `parse_leaf` L173 [`t=<cwd>→(Terminal,
    Some)`, else `one_char_kind→(kind,None)`]; `flatten` L50 [sets the leaf's 2nd tuple elem `Some` ONLY for a
    Terminal leaf, from `cwds:&HashMap<PaneId,String>`]; `GridLayout.cwds:Vec<Option<String>>` L23; `serialize_grid`
    L85 (calls flatten + serialize_leaf); `breaks_grid_framing(s)->bool` L298 (the framing guard, reuse for the
    path).
  - **(b) restore arm** — app.rs `restore_panes` CLOSURE L862-882: `for (i,kind) in gl.kinds.iter().enumerate()
    .skip(1)` → today `PaneKind::FileTree | PaneKind::CodeView => {}` (L875-876, the #154 drop). ⚠️ the closure has
    NO `self` → `load_code_view_state` L2284 is `&mut self` → design must extract a load-from-path core or inline
    the fs ladder. SECOND boot path: L1053 `restore_grid(&applied.grid)` (legacy single-grid) — both must route
    CodeView identically (D3 / #234).
  - **(a) editable pane [DEFERRED #259]** — `split_file_pane(path)` L2343 → `load_code_view_state` L2284 →
    `open_pane(Horizontal,After,PaneContent::CodeView(cv))` L2349. `PaneContent::CodeView(CodeViewState)`
    (workspace.rs L295). The 16+ `active_tab().editor()` couplings (app.rs 2696/2734/3199/3525/4439/4470/4566/4576/
    5268/5313/5355/…) are ALL keyed to the TAB → the model-change evidence for the split.
- **Decisions:** D1 SCOPE SPLIT [(b) here, (a)→#259, evidence: 16 editor()-tab couplings] · D2 `c=<path>` mirrors
  `t=<cwd>` exactly · D3 restore re-reads via the shared load ladder, both paths · D4 split_file_pane triggers
  persist_grid · D5 flatten sources the CodeView path from the registry. (Detail in the spec.)
- **PURE seam (cov/MSI 100 target):** serialize_leaf + parse_leaf + the flatten path-capture. `cargo mutants
  --list -f crates/marley_app/src/grid_layout.rs` at implement. The restore arm + persist trigger are shim
  (render/IO, mutants::skip).

## Phase 2 — Design — PASS

### Architecture / approach (D1-D5 resolved by reading the code)
The change rides the SHIPPED grid-codec lineage (#122 kind chars → #205 `t=<cwd>` → #243 editor-tab paths). A
CodeView pane already flows through `serialize_grid`→`flatten`→`serialize_leaf` (as a bare `c`) and through
`restore_panes` (dropped). We (1) make the codec CARRY the path (`c=<path>`) and (2) make the restore arm RE-READ
it. §20 N/A confirmed — Marley's own blob format, no reference-app source.

**D5 — flatten/serialize_grid path source → DECISION (A): add a parallel `paths: &HashMap<PaneId,String>` param.**
`flatten` (grid_layout.rs L50) sets the leaf's 2nd tuple elem `Some` only for a Terminal (from `cwds`). Add a
`paths` map (symmetric with `cwds`); flatten's leaf-detail pick becomes a match: `Terminal → cwds.get(id)`,
`CodeView → paths.get(id)`, `_ → None`, each `.filter(|s| !breaks_grid_framing(s)).cloned()`. The leaf tuple
`(PaneKind, Option<String>)` + `GridLayout.cwds` are UNCHANGED — on parse, a `c=<path>` leaf's path lands in the
existing `cwds` vec (functionally "the leaf's detail"; the field name is a slight post-#258 misnomer, noted). The
TWO serialize call sites (`persist_grid` L2194, `build_shell_layout` L2234) build a `paths` map by mirroring the
`cwds` build: `pane_ids().filter_map(|id| state(id).code_view().map(|cv| (id, cv.path.display().to_string())))`
(`state().code_view() -> Option<&CodeViewState>` workspace.rs L352; `cv.path: PathBuf` code_view.rs L248; the
double `workspace_mut()` borrow in the filter_map already compiles for `cwds`). Rejected (B) unified `details`
map [renames `GridLayout.cwds` → ripples into the restore readers L870/911/917].

**serialize_leaf / parse_leaf — STRICT siblings of `t=` (not a generalization).** Mirror `t=` EXACTLY + add a `c=`
sibling (preserves the exact malformed-input behavior — `f=x`/`g=x` still → None → default, as today):
```rust
// serialize_leaf
match leaf {
    (PaneKind::Terminal, Some(cwd)) => format!("t={cwd}"),
    (PaneKind::CodeView, Some(path)) => format!("c={path}"),
    (kind, _) => kind_char(*kind).to_string(),
}
// parse_leaf
if let Some(cwd) = tok.strip_prefix("t=") { Some((Terminal, Some(cwd.into()))) }
else if let Some(path) = tok.strip_prefix("c=") { Some((CodeView, Some(path.into()))) }
else { one_char_kind(tok).map(|k| (k, None)) }
```
Back-compat: an old bare `c` → `one_char_kind` → `(CodeView, None)` → restore drops (no path).

**D3 — restore-arm load → DECISION: MIRROR the #243 editor-tab restore-load ladder INLINE (no extraction).**
`load_code_view_state` (L2284) is NOT self-independent (needs `self.project_root`/`self.code_tab_width`/
`self.status_flash`) AND is `mutants::skip`'d → extracting from it risks the #246/#252 skip-detach. INSTEAD the
#243 editor-tab restore (L948-968) already has a self-INDEPENDENT load ladder in the SAME boot scope
(`resolve_under_root(root, path)` + `fs::read().ok()` + `viewer_size_ok(len, VIEWER_MAX_BYTES)` +
`!is_probably_binary` + `CodeViewState::new(pb, &text, applied.code_tab_width, CODE_MAX_COLS)`). Mirror it in the
CodeView arm. **ONE arm covers BOTH restore paths** — `restore_panes` is a single closure called at L928
(whole-shell) AND L1054 (legacy single-grid), both in scope of `applied`/`root`/`resolve_under_root`/`viewer_size_ok`
/`is_probably_binary`/`CodeViewState::new` (the #234 audit-all-sites is satisfied by the shared closure). The arm
(replacing `FileTree | CodeView => {}` L875-876 with a `FileTree => {}` + a CodeView arm):
```rust
PaneKind::CodeView => {
    if let Some(path) = gl.cwds.get(i).and_then(|o| o.as_ref()) {
        let pb = resolve_under_root(root, &PathBuf::from(path));
        if let Ok(bytes) = std::fs::read(&pb) {
            if viewer_size_ok(bytes.len(), VIEWER_MAX_BYTES) && !is_probably_binary(&bytes) {
                let text = String::from_utf8_lossy(&bytes).into_owned();
                let cv = CodeViewState::new(pb, &text, applied.code_tab_width, CODE_MAX_COLS);
                grid_target.open_pane(gl.axis, SplitDirection::After, PaneContent::CodeView(cv));
            }
        }
    }
}
```
Unreadable / oversize / binary / None → the if-lets skip → dropped, no crash (matches #243).

**D4 — persist trigger → ALREADY SATISFIED.** `split_file_pane` (L2343) ALREADY calls `self.persist_grid()`
(L2351, wired at #246 anticipating #258). No change needed — the #243 persist-TRIGGER trap is already avoided
here. Once `serialize_leaf` emits `c=<path>`, the existing persist saves it. (The persist path's serialize call
DOES change — it must build + pass the `paths` map, the D5 edit.)

**COVERAGE/MUTATION boundary.** app.rs is the ACCEPTED-UNTESTABLE shim exclude (app.rs L1 `//! SHIM`; gates.sh
L181 documents app.rs coverage-excluded; the boot/restore region generates 0 mutants — app.rs stays at 6). So the
restore arm + the paths-map builds (app.rs) are DRIVEN-validated, NOT unit-tested. The PURE seam needing cov/MSI
100 is ONLY grid_layout.rs: serialize_leaf (c= arm) + parse_leaf (c= arm) + flatten (paths param + CodeView
path-pick), all covered THROUGH the public `serialize_grid`/`restore_grid` (flatten/serialize_leaf are private).

### File manifest
- **crates/marley_app/src/grid_layout.rs** (PURE) — `serialize_leaf`: add the `(CodeView, Some(path)) → "c={path}"`
  arm. `parse_leaf`: add the `c=` `strip_prefix` arm. `flatten`: add a `paths: &HashMap<PaneId,String>` param + the
  CodeView path-pick (framing-filtered). `serialize_grid`: add the `paths` param + thread to `flatten`. Update the
  2 existing serialize_grid tests (L441/L460) to pass the extra `&HashMap::new()` arg + ADD the new c= tests.
- **crates/marley_app/src/app.rs** (SHIM, coverage-excluded) — `persist_grid` (L2194) + `build_shell_layout`
  (L2234): build a `paths` map (mirror the `cwds` build) + pass `&paths` to `serialize_grid`. `restore_panes`
  (L875-876): split the `FileTree | CodeView => {}` arm into `FileTree => {}` + the CodeView re-read arm (covers
  BOTH restore paths via the shared closure). NO change to `split_file_pane` (D4 already persists).

### Regression Test Plan
| # | Test (grid_layout.rs `#[cfg(test)]`, via the public serialize_grid/restore_grid) | Proves |
|---|---|---|
| T1 | `serialize_grid(H:t,c` with a CodeView leaf + paths{id:"/x.rs"}) contains `c=/x.rs`; round-trips via restore_grid → kinds[.., CodeView], cwds[.., Some("/x.rs")] | REQ-001 (serialize) + REQ-002 (parse round-trip) |
| T2 | a CodeView path with a framing delimiter (`/a,b.rs`, `/x=y.rs`, `\x1f`) → the leaf serializes as a BARE `c` (no `c=`) | REQ-001 (framing-drop, `breaks_grid_framing`) |
| T3 | `restore_grid("H:t,c=/x.rs")` → GridLayout kinds[Terminal, CodeView], cwds[None, Some("/x.rs")]; `restore_grid("H:t,c")` (bare, pre-#258) → cwds[.., None] | REQ-002 (parse both forms + back-compat) |
| T4 | a single-leaf `restore_grid("c=/x.rs")` → one CodeView pane, cwds[Some("/x.rs")]; `single_char_grid("c")` → cwds[None] | REQ-002 (single-leaf + back-compat) |
| T5 | full serialize→restore identity on a `V:t=/tmp,c=/a.rs,t` grid (Terminal cwd + CodeView path coexist) | REQ-001/002 (t= and c= coexist, no cross-talk) |
| REQ-003 | split_file_pane already calls persist_grid (verified L2351) — DRIVEN: the `c=<path>` lands in settings.toml before relaunch (the #243 discipline — verify the SAVE fires, not just the codec) | REQ-003 |
| REQ-004 | `cargo mutants --list -f grid_layout.rs` → the new c= mutants (format strings, strip_prefix, the CodeView flatten arm + filter) all killed by T1-T5; gate:4 cov 100 | REQ-004 |
| DRIVEN | split a file pane (⌘P → Split Right → File) → quit → relaunch → the split file pane RESTORES with its file (data-safe: read-only pane, no ⌘S) | REQ-002 end-to-end |

Uncoverable-by-unit (driven/excluded): the restore arm + paths-map builds (app.rs shim exclude — driven); the
DRIVEN quit→relaunch (live GUI). NOTE the #253 gate-wedge watchdog + the #254/#255 one-drive-call harness note.

### Risks / decisions
1. **serialize_grid signature ripple** — 2 non-test callers (persist_grid, build_shell_layout) + 2 test callers
   (L441/L460) gain the `paths` arg; compile-checked, small.
2. **The `cwds` field carries a CodeView PATH on restore** (a post-#258 misnomer, not a bug) — the leaf tuple's
   2nd elem is "the leaf's detail" (cwd for Terminal, path for CodeView). Noted in a code comment; a rename is out
   of scope (ripples into 3 restore readers).
3. **No fn extraction** → NO mutants::skip-detach risk (D3 mirrors #243 inline). Re-confirm app.rs mutants stay at
   6 at implement anyway.
4. **Back-compat** — old bare `c` → (CodeView, None) → dropped, EXACTLY today's behavior (no regression).
5. **The paths-map double-borrow** (`workspace_mut()` twice in the filter_map) already compiles for `cwds` — the
   same shape.

## Phase 3 — Implement
- What was built; deviations from design (with reason).

## Phase 3.5 — Inspect
- Critics run; findings table (severity / finding / verdict); fixes.

## Phase 4 — Validate
- Tests RUN (with counts) + gate result; negative smokes; pre-existing notes.

## Phase 5 — Complete
- Docs updated; AAR capture (lessons / failures / prevention rules / ADs); archive.

## Phase 3 — Implement — PASS

Built to the manifest with ZERO deviations. `cargo check -p marley --all-targets` clean; `cargo fmt` applied.

**grid_layout.rs (PURE):**
- `serialize_leaf` — added `(PaneKind::CodeView, Some(path)) => format!("c={path}")` between the Terminal arm and
  the bare-kind fallback.
- `parse_leaf` — added an `else if let Some(path) = tok.strip_prefix("c=")` arm → `(CodeView, Some(path))`,
  mirroring the `t=` arm. A pre-#258 bare `c` still falls to `one_char_kind` → `(CodeView, None)`.
- `flatten` — added a `paths: &HashMap<PaneId, String>` param; the leaf-detail pick is now a `match kind`
  (Terminal→cwds, CodeView→paths, _→None), each `.filter(|s| !breaks_grid_framing(s)).cloned()`.
- `serialize_grid` — added the `paths` param, threaded to `flatten`. Doc comments updated (#258).
- Updated the 6 existing `serialize_grid(...)` test call sites with the 4th `&HashMap::new()` arg (the paths map).

**app.rs (SHIM, coverage-excluded):**
- `persist_grid` (~L2194) — added a `paths` map build (mirror the cwds build via `state().code_view()` →
  `cv.path.display()`), passed `&paths` to serialize_grid.
- `build_shell_layout` (~L2245) — added the same per-grid `paths` build inside the `tab.grid()` arm, passed to
  serialize_grid.
- `restore_panes` (~L875) — split `FileTree | CodeView => {}` into `FileTree => {}` + a CodeView re-read arm
  mirroring the #243 editor-tab restore ladder (`resolve_under_root` + `fs::read().ok()` + `viewer_size_ok` +
  `!is_probably_binary` + `CodeViewState::new(pb, &text, applied.code_tab_width, CODE_MAX_COLS)` →
  `open_pane(gl.axis, After, CodeView(cv))`); unreadable/oversize/binary/None → dropped, no panic. The ONE closure
  serves BOTH restore paths (L928 whole-shell + L1054 legacy) — both restore CodeView panes.
- `split_file_pane` — UNCHANGED (D4: it already calls `self.persist_grid()`).

**Mutation surface (for validate):** `cargo mutants --list -f grid_layout.rs` — the NEW pure mutants:
- flatten: `delete match arm PaneKind::CodeView` (L65:17), `delete !` on the path framing-filter (L65:64) [+ the
  pre-existing Terminal-arm mutants L64:17/L64:63, killed by the shipped #205 cwd tests], `replace flatten with ()`.
- serialize_leaf: `String::new()` / `"xyzzy".into()` (L81:5) — the c= arm's behavior is pinned by a T1 exact-value
  assertion (no arm-specific mutant is emitted, but the whole-fn return mutants + coverage force it).
- parse_leaf: `None` (L182:5) [+ 3 `Some((Default::default(), …))` — likely UNVIABLE, PaneKind has no Default
  derive, the #204/#249 rule; confirm at validate].
The NEW-behavior mutants my c= tests must kill: flatten L65:17 + L65:64, and the serialize_leaf/parse_leaf return
mutants via exact `c=/x.rs` assertions.

**Skip re-confirm:** app.rs `restore_panes|persist_grid|build_shell` mutant count = 0; app.rs total = 6 (UNCHANGED
from #257 — the boot region generates no mutants, so NO mutants::skip-detach; no fn was extracted, per D3). The 20
grid_layout tests (incl the 6 updated serialize_grid calls) stay GREEN.

**Phase 3 status: Implement PASS — ready for Phase 3.5 — Inspect.**

## Inspect (Phase 3.5) — PASS (2 doc fixes; 0 logic defects)

2 critics (pure codec · shim). **ZERO logic defects — the codec + restore arm are correct.** 2 real doc-rot
fixes folded; the coverage/mutant gap is the expected validate-phase test deliverable (not a code bug).

### Critic 1 — pure codec (all logic CONFIRMED; 1 HIGH coverage-req + 1 MED doc)
- (a) serialize_leaf: `(CodeView,None)`→bare `c`, Terminal unchanged. (b) parse_leaf: `c=/x.rs`→(CodeView,Some);
  bare `c`→(CodeView,None); `f=x`→None [STRICT sibling CONFIRMED — not a generalization]. (c) flatten: NO
  cross-talk (Terminal reads cwds, CodeView reads paths); framing filter drops a path with `,:=\t\n\r\x1f` → bare
  `c`; an `=` in a path is dropped by the framing guard AND strip_prefix("c=") only takes the leading `c=` (safe
  either way). (d) round-trip: `H:t=/x,c=/a.rs`→kinds[Terminal,CodeView],cwds[Some("/x"),Some("/a.rs")] — t=/c=
  coexist. (e) back-compat: `H:t,c`→cwds[None,None], no crash.
- **CORRECTION (viability):** `PaneKind` DOES derive `Default` (#[default] Terminal, workspace.rs:187) → the 3
  parse_leaf `Default::default()` mutants are VIABLE and ALREADY CAUGHT by the shipped `grid_round_trip_all_kinds`
  test (my implement-note guess that they'd be unviable was WRONG). All 6 serialize_leaf/parse_leaf mutants are
  killed by existing tests.
- **[HIGH → VALIDATE] the c=<path> path has 2 SURVIVING flatten mutants + no direct test** (empirically proven:
  `cargo mutants -F 'in flatten'` → flatten L65:17 delete-CodeView-arm + L65:64 delete-`!` MISSED, because every
  existing serialize_grid test passes `&HashMap::new()` for paths → the CodeView arm is never exercised). This is
  NOT a code defect — it is the expected mid-pipeline state (tests are the VALIDATE deliverable). The Phase-2 test
  plan already specifies the exact killers: T1 (`serialize_grid(CodeView, paths{0:"/a.rs"})=="c=/a.rs"` +
  round-trip) kills L65:17; T2 (a framing-breaking path → bare `c`) kills L65:64; T3 mixed `H:t=/x,c=/a.rs`; T4
  back-compat bare `c`. CARRIED to validate.
- **[MED → FIXED] GridLayout.cwds field doc was stale** (said "per-pane cwd; None for a non-terminal") — since
  #258 a CodeView leaf stores its file PATH there. FIXED: the field doc now states it is the leaf's generic detail
  slot (a Terminal's cwd OR a CodeView's path; the `cwds` name is the original #205 use). A rename cwds→details
  was REJECTED (ripples into 3 restore readers L870/911/917 — out of scope; doc fix is the minimum-correct).

### Critic 2 — shim (all PASS; 1 LOW doc)
- (f) paths-map builds: the double-borrow compiles (mirrors the cwds build; pane_ids() owns the Vec);
  state().code_view()→Option<&CodeViewState>; no unwrap; both sites consistent with #243's cv.path capture.
- (g) restore arm: PASS, panic-free — None/Some(None)/fs-Err/oversize/binary all skip→dropped; no unwrap/index;
  mirrors the #243 ladder EXACTLY (resolve_under_root + fs::read + viewer_size_ok(len,VIEWER_MAX_BYTES) +
  !is_probably_binary + CodeViewState::new(pb,text,applied.code_tab_width,CODE_MAX_COLS)); the intended read-only
  divergence (PaneContent::CodeView, not EditorSurface; open_pane, not split_focused) is correct.
- (h) both restore paths: the ONE restore_panes closure serves L955 (whole-shell) + L1081 (legacy); a crate-wide
  audit found NO third GridLayout/restore_grid pane-rebuild path that would drop a CodeView — the #234 audit is
  satisfied by construction.
- (i) D4 + skip: split_file_pane already persists (UNCHANGED); Terminal/Git/FileTree arms unchanged; app.rs
  mutants 6/0 UNCHANGED (the boot region generates none — no skip-detach, no fn extracted); clean-room OK.
- **[LOW → FIXED] split_file_pane doc said "drops on restart (v1 non-persisted)"** — #258 IS that follow-on; the
  pane now persists. FIXED the doc + softened the "(#258 makes it editable)" aside (editability is #259; #258 is
  persistence, read-only).

**Phase 3.5 status: Inspect PASS — 2 doc-rot fixes folded, 0 logic defects. The HIGH is the validate test
requirement (exact killers named). Ready for Phase 4 — Validate.** NO failure/prevention record — the coverage gap
is a pending test deliverable, not a code bug; the doc-rot is minor + covered by existing doc-freshness rules.

## Phase 4 — Validate — PASS

### Tests added (pure seam — grid_layout.rs, cov 100% lines / MSI 100%)
ONE test `codeview_path_round_trips_and_drops_framing_breakers` (mirrors the shipped #205 `grid_cwd_round_trips…`
sibling), 5 clauses:
- **T1** (REQ-001 serialize + REQ-002 parse): a single CodeView + paths{0:"/a.rs"} → `serialize_grid == "c=/a.rs"`;
  `restore_grid("c=/a.rs")` → kinds[CodeView], cwds[Some("/a.rs")]. KILLS flatten L65:17 (delete-CodeView-arm) +
  pins serialize_leaf/parse_leaf's c= arms.
- **T2** (REQ-001 framing): a path with `,`/`:`/`=`/`\t` → serialize drops to bare `c`. KILLS flatten L65:64
  (delete-`!` path-filter).
- **T3** (headline coexist): `H:t=/x,c=/a.rs` round-trips — Terminal cwd + CodeView path with no cross-talk.
- **T4** (REQ-002 back-compat): a pre-#258 bare `c` → kinds[CodeView], cwds[None] (dropped on restore, no crash).
- **T5** (no cross-talk): a CodeView with only a `cwds` entry (no `paths`) → bare `c` (reads ONLY paths).

### Test run (actual)
- `codeview_path_round_trips_and_drops_framing_breakers`: `1 test run: 1 passed`.
- shipped codec tests (grid/serialize/cwd_round): `21 tests run: 21 passed` (was 20 → +1).
- Full workspace under coverage: `873 tests run: 873 passed, 5 skipped`.
- MUTATION: `cargo mutants -f grid_layout.rs -F 'in flatten'` → **4 mutants tested: 4 caught** (both new CodeView
  arms L65:17/L65:64 + the 2 Terminal ones). app.rs `restore_panes|persist_grid|build_shell` = 0 mutants, total = 6
  (UNCHANGED — the boot region is the shim-excluded zone).

### Driven live proof (REQ-001/002/003 end-to-end — the #243 persist-TRIGGER discipline)
Bundled + launched the fresh binary (machine UNLOCKED). REAL UI path, all captured:
1. **Split a file pane:** ⌘⇧P palette → filtered "split" → selected **"Split Right → File"** (CommandId(10),
   #246) → the ⌘P finder opened in split mode → typed "README.md" → selected `docs/README.md` → Enter split it.
2. **REQ-001 + REQ-003 (the persist TRIGGER fired):** after the split, `~/.marley/config/settings.toml` read
   `grid = "H:t=<root>,c=<root>/docs/README.md"` — the `c=<path>` codec emitted the path (REQ-001), and it landed
   in settings BEFORE relaunch (count = 2: both the legacy `grid` key AND the shell blob's first tab), proving the
   SAVE fired, not just the codec round-trips (the #243 discipline).
3. **REQ-002 (restore re-reads, no crash):** pkill → relaunch → **process ALIVE** (a boot panic on the restore arm
   would exit it). The DECISIVE proof: after relaunch the live restored workspace RE-SERIALIZED `grid =
   "H:t=<root>,c=README.md"` (c= count still 2) — if the restore arm had DROPPED the CodeView (the old #154
   behavior), the re-persist would be bare `t=<root>`. Instead the restore arm re-created the CodeView pane from
   the `c=<path>` → a STABLE split→persist→quit→restore→persist round-trip. (The side-by-side pane pixels weren't
   captured — the split lives in the terminal tab, which restored collapsed in the rail (#236/#245) and the
   chevron-expand drive didn't land; the settings round-trip is a STRONGER proof than a pixel — it proves the pane
   is a live workspace object, not just a render.)
4. **DATA-SAFETY:** the split pane is READ-ONLY (no editing, no ⌘S) → README.md untouched. My test artifact (the
   README.md split pane) was REVERTED in chad's settings.toml (2 occurrences → back to `t=<root>`, c= count 0), so
   his session is pristine. `git status` confirms ONLY the 2 intended #258 `.rs` files changed (the repo tree is
   untouched — settings.toml is ~/.marley, not the repo).

### Gate
`scripts/gates.sh --diff` → **GATE GREEN [diff]** — 15 passed, 0 failed (gate:4 coverage 100% lines on
grid_layout.rs [the 99.90% region figure is 1 partial region, NOT a line miss — lines 579/0], gate:5 mutation MSI
100%, gate:15 visual/AX). No real-PTY wedge. No pre-existing failures.

**Phase 4 status: Validate PASS — gate green. Ready for Phase 5 — Complete.**

## Phase 5 — Complete — PASS

- **Docs (§21):** CHANGELOG.md — added the #258 `### Added` entry above #257. docs/marley_architecture/app_shell.md
  — added the M15 #258 bullet after #257 (the c=<path> codec + the paths param + the restore arm + the scope
  split to #259 + the driven round-trip proof).
- **Knowledge (forge wired):** aar-submit (aar_id 2fdf7580, outcome completed, effectiveness 5). LESSONS
  reinforced: (i) the #243 persist-TRIGGER discipline held — split_file_pane already called persist_grid; the
  driven proof verified the c=<path> LANDED in settings.toml BEFORE relaunch (not just the codec round-trip).
  (ii) a NEW test-map param leaves the new arm uncovered until a test EXERCISES it — the shipped serialize_grid
  tests passed `&HashMap::new()` for paths, so the flatten CodeView arm had 2 surviving mutants until T1/T2 fed a
  real paths entry (mirrors the #257 apply_key-arm coverage trap: a new arm needs a test that DRIVES it, not just
  compiles). (iii) mirroring a SHIPPED restore ladder (the #243 sibling) INLINE beat extracting from the
  mutants::skip'd load_code_view_state — no fn extraction → no skip-detach risk (the #246/#252 trap avoided by
  construction). NO new failure/prevention record — inspect was clean (0 logic defects); the 2 doc-rot fixes + the
  coverage-until-exercised note are covered by existing rules. surfaced_used:
  PR-claude-persist-verify-trigger-not-just-codec-001 (applied + driven-verified), the #234 audit-all-restore-sites
  (confirmed one closure serves both paths + no third path), the #254/#255 one-drive-call harness note.
- **Key decision — the SCOPE SPLIT:** #258 shipped (b) the persistence codec; (a) the editable split pane → the
  fast-follow #259 (a8bcfc50, filed at plan). Evidence: 16+ active_tab().editor() couplings prove (a) is a
  PaneContent model change, not a codec change (§3 one-shippable-slice).
- **Ticket:** TICKET-258 open→closed; forge ticket-close a293b7ac (→ done).
- **Archive:** split-pane-persist.{spec,notes}.md active/→completed/.

**Phase 5 status: Complete PASS. Ready for /commit. This is the LAST ticket of the M15 /goal /work 249-258 — the
whole train (#249-258, 10 tickets) is COMPLETE.**
