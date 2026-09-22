---
pipeline_id: 333e220c-8fa8-442e-9a93-e14e18148520
aar_id: aba53b1c-44e6-4b14-af2c-11de0836b138
---

# app_shell layout — pipeline notes

## Phase 1 — Plan (2026-07-01)

**Intent:** the SPEC-app-shell workspace layout (R4-R13) in marley_app — the pure `PaneGroup` tree algebra
+ docks + the 3-region render. M1.B Cockpit seq 4/5; the biggest tested surface.

## Carry to Design (Phase 2) — the algebra

### layout.rs (PURE — cov 100/MSI 100)
Types (derive Debug/Clone/PartialEq/Eq; PaneAxis/SplitDirection/Direction/DockSide/DockState also Copy):
`PaneId(pub u64)`, `PaneAxis{Horizontal,Vertical}`, `SplitDirection{Before,After}`, `Direction{Up,Down,
Left,Right}`, `PaneError{PaneNotFound,LastPane}`, `DockSide{Left,Right}`, `DockState{Open,Closed}`,
`enum PaneGroup{ Leaf(PaneId), Split{axis:PaneAxis, children:Vec<PaneGroup>, ratios:Vec<f32>} }`.

- `DockState::toggled(self)->DockState` — `Open→Closed, Closed→Open` (R6).
- `fn equal_ratios(n: usize) -> Vec<f32>` — `vec![1.0/n as f32; n]` (R11).
- `single(pane)->Self` = `Leaf(pane)`.
- `panes(&self)->Vec<PaneId>` — `Leaf(p)=>vec![*p]`; `Split{children,..}=>children.iter().flat_map(|c|
  c.panes()).collect()` (depth-first, R7).
- `contains(&self, p)->bool` = `self.panes().contains(&p)` (helper for the error guards).
- `split(&mut self, target, new, axis, dir)->Result<(),PaneError>`:
  `if !self.contains(target) {return Err(PaneNotFound)}`; then recurse `split_at(target,new,axis,dir)`:
  at `Leaf(p)` where `*p==target` → `*self = Split{axis, children: match dir {Before=>vec![Leaf(new),
  Leaf(target)], After=>vec![Leaf(target),Leaf(new)]}, ratios: equal_ratios(2)}`; at a `Split` recurse each
  child. `Ok(())`. (R8/R11/R13.)
- `close(&mut self, pane)->Result<(),PaneError>`:
  `if !self.contains(pane){return Err(PaneNotFound)}`; `if self.panes()==[pane]{return Err(LastPane)}`
  (only pane); then `close_at(pane)` (recursive): at a `Split`, if a DIRECT child is `Leaf(pane)` →
  `children.remove(idx)`; if `children.len()==1` → `*self = children.remove(0)` (COLLAPSE, R9) else
  `ratios = equal_ratios(children.len())` (R11); else recurse into each child (a deeper close may collapse
  that child — this level's child count is unchanged, ratios untouched). `Ok(())`. (R9/R10/R11/R13.)
- `neighbor(&self, from, dir)->Option<PaneId>` — R12, PATH-BASED:
  1. `let path = self.path_to(from)?;` (Vec<usize> child-indices root→from; None if absent).
  2. `(target_axis, forward) = match dir { Left=>(Horizontal,false), Right=>(Horizontal,true),
     Up=>(Vertical,false), Down=>(Vertical,true) };` (forward = following sibling; !forward = preceding).
  3. `for depth in (0..path.len()).rev()`: `let anc = self.node_at(&path[..depth]);` if `anc` is
     `Split{axis,children,..}` and `*axis==target_axis`: `let idx = path[depth];` the sibling on the
     movement side = `forward ? (idx+1 if < children.len()) : (idx-1 if idx>=1)`; if it exists →
     `return Some(descend_boundary(&children[sib], forward))`.
  4. else `None` (edge).
  - `path_to(&self, target)->Option<Vec<usize>>` — `Leaf(p)`: `*p==target ? Some(vec![]) : None`; `Split`:
    for `(i,c)` if `c.path_to(target)` is `Some(sub)` → `Some([i]++sub)`.
  - `node_at(&self, path)->&PaneGroup` — follow the indices (`children[i]`; invariant-safe).
  - `descend_boundary(node, forward)->PaneId` — loop: `Leaf(p)=>return *p`; `Split{children,..}=>` next =
    `forward ? children[0] : children[children.len()-1]` (nearest boundary: FIRST for R/D, LAST for L/U).

### app.rs (SHIM — every new fn mutants::skip)
`RootView` gains `pane_group: PaneGroup` (init `single(PaneId(0))`) + `docks:[DockState;2]` (both Open;
index 0=Left,1=Right). `dock(side)` reads; `toggle_dock(side)` = `self.docks[i]=self.docks[i].toggled();
cx.notify()`; `pane_group()` reads. on_key_down (already routes palette/keymap): cmd-d → `pane_group.split(
<the sole/first pane>, PaneId(next_id), Horizontal, After)`; cmd-w → `pane_group.close(<first pane>)` (best-
effort; ignore LastPane/NotFound). The render: a flex-row of [left dock (width 0 if Closed) | center
(renders the pane_group leaves as columns/rows per axis) | right dock]. Minimal — shim.

### Mutation map (the spec's targets — all in layout.rs)
- `split` — Before/After swap (assert new-before vs new-after), axis mutate (assert the Split.axis),
  leaf-replacement drop (assert the leaf became a Split) — R8.
- `close` — collapse-skip (assert no 1-child Split; the survivor replaces the Split), wrong-child (assert
  the RIGHT pane removed), Ok-instead-of-LastPane (R10) / PaneNotFound (R13) — assert the Err + tree
  unchanged.
- `ratios` — equal_ratios drop / sum / len (assert len==child count + each==1.0/n) — R11.
- `neighbor` — dir→axis flip, preceding↔following (idx+1 vs idx-1), first↔last descend, edge-None — via
  FLAT 2x1 (Split(H)[A,B]: neighbor(A,Right)=B, neighbor(B,Left)=A, neighbor(A,Left)=None, neighbor(A,Up)=
  None) AND NESTED 2x2 (Split(H)[Split(V)[A,B], Split(V)[C,D]]: neighbor(A,Right)=C [descend the right
  H-sibling taking its FIRST V-child], neighbor(A,Down)=B, neighbor(D,Left)=B [descend left H-sibling
  taking its LAST V-child], neighbor(A,Up)=None) — R12.
- `DockState::toggled` — arm swap (assert toggled(Open)==Closed, toggled(Closed)==Open) — R6.
- Whole-fn `Default::default()` mutants: PaneGroup/PaneError/etc. derive no Default → UNVIABLE (confirm at
  inspect via cargo mutants --list).

### Fixtures (validate)
- `flat()` = `Split(H)[Leaf(A), Leaf(B)]` (build via `single(A).split(A,B,Horizontal,After)`).
- `nested()` = the 2x2 (single(A) → split A|B (H) → wait: build A, split→[A,B] H, then split A→[A,C] making
  it nested... design the exact build in validate to yield Split(H)[Split(V)[A,B],Split(V)[C,D]]).
- Ids: `PaneId(1)`.. distinct.

### Risks
The neighbor R12 is intricate (path-walk + boundary descend) — implement carefully + test BOTH directions,
BOTH movement sides, the edge-None, AND the nested-2x2 descend (first vs last child). The collapse (close)
+ the invariant (Split ≥2 children) — indexing is invariant-safe (not input). Write-then-check the gpui
render (the #16 lesson).

**Phase 1 status:** PASS (autonomous). → Phase 2 Design.

## Phase 2 — Design (2026-07-01)

**Confirmed the Carry-to-Design against SPEC-app-shell's Public surface — MATCHES.** The `PaneGroup` enum,
`PaneId(u64)` (→ `PaneId(pub u64)`), `PaneAxis`/`SplitDirection`/`Direction`/`PaneError`/`DockSide`/
`DockState`, + `single`/`split(target,new,axis,dir)->Result`/`close->Result`/`panes()->Vec`/`neighbor->
Option` all realised as specced. `RootView::{dock,toggle_dock,pane_group}` are the shim accessors.

- **Seam CONFIRMED:** the whole `PaneGroup` algebra + `DockState::toggled` + `equal_ratios` are gpui-free
  PURE → cov 100 / MSI 100. The 3-region render + zero-width-closed reflow + cmd-d/cmd-w wiring are SHIM
  in app.rs (already `mutants::skip` + rust_cov-excluded — no gates.sh change).

**NEIGHBOR R12 — TRACED CORRECT** on the nested 2×2 `Split(H)[ Split(V)[A,B], Split(V)[C,D] ]` (grid: A|C
top row, B|D bottom row — A=TL, C=TR, B=BL, D=BR):
- `neighbor(A,Right)` (axis=H, forward): path [0,0] → depth1 child0=Split(V)≠H skip → depth0 root=Split(H)==H,
  idx0, sibling idx1 → descend child1 taking FIRST V-child → **C** ✓.
- `neighbor(A,Down)` (axis=V, forward): path [0,0] → depth1 child0=Split(V)==V, idx0, sibling idx1 → Leaf **B** ✓.
- `neighbor(D,Left)` (axis=H, preceding): path [1,1] → depth1 child1=Split(V)≠H skip → depth0 root==H, idx1,
  sibling idx0 → descend child0 taking LAST V-child → **B** ✓.
- `neighbor(A,Up)` (axis=V, preceding): path [0,0] → depth1 child0=Split(V)==V but idx0 has NO preceding
  sibling (idx-1<0) skip → depth0 root=Split(H)≠V skip → **None** ✓ (top edge).
- Flat `Split(H)[A,B]`: `neighbor(A,Right)=B`, `neighbor(B,Left)=A`, `neighbor(A,Left)=None`,
  `neighbor(A,Up)=None` — all ✓. The loop `for depth in (0..path.len()).rev()` with `node_at(&path[..depth])`
  as the ancestor + `path[depth]` as from's subtree index is correct.

- **Mutation map CONFIRMED — covers the spec's targets:** split Before/After+axis+leaf-drop (R8); close
  collapse-skip + wrong-child + Ok-not-LastPane/PaneNotFound (R9/R10/R13); ratio drop/sum/len (R11);
  neighbor axis-flip + preceding↔following + first↔last-descend + edge-None (R12, via flat + nested);
  DockState arm-swap (R6). Whole-fn `Default::default()` mutants UNVIABLE (no Default derives). Confirm the
  exact viable set at inspect via `cargo mutants --list`.
- **Invariant:** every `Split` has ≥2 children (single→Leaf, split→2, close collapses at 1) → internal
  `children[i]`/`[0]`/`[len-1]` indexing is invariant-safe (the PaneGroup is internal state, not an
  input/response path — no §14 panic concern; and no uncoverable defensive branch).
- **Test plan:** the 8 unit tests (R6-R13) with the flat + nested fixtures; the app.rs render is
  headed-only (the #[ignore] docks/split assertion).

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement (2026-07-01)

Built (in-context):
- **crates/marley_app/src/layout.rs (PURE, gpui-free, 0 gpui refs):** all the types + `DockState::toggled`
  + `equal_ratios` + the `PaneGroup` algebra (single/panes/contains/split[+split_at]/close[+close_at]/
  neighbor[+path_to/node_at]/`descend_boundary`). Matches the Phase-2 design + the traced neighbor.
- **crates/marley_app/src/lib.rs:** `mod layout;` + `pub use layout::{Direction, DockSide, DockState,
  PaneAxis, PaneError, PaneGroup, PaneId, SplitDirection};`.
- **crates/marley_app/src/app.rs (SHIM — every new fn mutants::skip):** RootView gains `pane_group` (init
  `single(PaneId(0))`) + `docks:[DockState;2]` (Open) + `next_pane_id`; `dispatch_action` (open-palette /
  split-pane / close-pane the first pane); `dock`/`toggle_dock`/`pane_group` accessors; the on_key_down
  keymap branch → `dispatch_action`; a minimal layout status line in the render (references the pane count
  + dock state).

**Deviation (sound):** `close_at` ALWAYS collapses (no ratio-reset-on-surviving-multi-child branch). A
`Split` always has EXACTLY two children — R8 makes 2-child splits and `split` never adds to an existing
Split — so a `close` always leaves one survivor → collapse. The R11 "reset ratios of a surviving
multi-child node" case is UNREACHABLE via the public API (would be dead/uncoverable), so it's omitted.
R11 is satisfied by `split` (sets `equal_ratios(2)` = `[0.5, 0.5]`); `close` removes the split (its ratios
with it). The render's 3-region docks layout is a minimal status line for M1.B — the full masked
docks/split baseline is a headed-lane follow-up; `toggle_dock` is public API (no chord triggers it yet).
`cargo check`/`clippy -D`/`fmt`/`rustdoc -D`/no-`unsafe` all clean; layout.rs gpui-free.

**Carry to Validate:** the 8 tests (R6-R13). INSPECT: `cargo mutants --list -p marley` for the viable
layout.rs surface; the neighbor coverage needs BOTH directions/movement-sides + the edge-None + the nested
descend (first vs last); the split_at Leaf-no-match arm needs a split-in-a-2-leaf-tree; close covers the
collapse + the recurse (deep close). `panes()==[pane]` for LastPane.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5) — 2026-07-01

Inspected in-context (`cargo mutants --list -p marley` + an empirical `neighbor` probe). **One finding —
a TEST-FIXTURE correction, NOT a code bug.**

- **Seam CONFIRMED:** 0 app.rs mutants leaked (dispatch_action/dock/toggle_dock/pane_group all skipped);
  every listed mutant is in layout.rs. **Clean-room:** 0 `warp`; **§14:** no unwrap/expect/panic (the
  `children[i]` indexing is invariant-safe — a Split always has ≥2 children — not an input path).

**[FINDING — MEDIUM, fixture only] The design's nested-2×2 `Split(H)[Split(V)[A,B],Split(V)[C,D]]` is
UN-BUILDABLE:** `split` only ever replaces a LEAF with a 2-child Split (R8) — it never wraps an existing
Split — so you cannot nest a pre-made column. The BUILDABLE 2×2 (empirically confirmed) is:
`single(A) → split(A,B,H,After) → split(A,C,V,After) → split(B,D,V,After)` ⇒
`Split(H)[ Split(V)[A,C], Split(V)[B,D] ]`, `panes()==[A,C,B,D]` (grid A|B top, C|D bottom). The ALGORITHM
is correct per R12; only the notes' fixture + neighbor expectations were wrong. **R12-correct expectations
for this tree** (the descend takes LAST child for L/U, FIRST for R/D — a defined rule, not geometry):
`neighbor(A,Right)=B` (confirmed by probe), `neighbor(A,Down)=C`, `neighbor(B,Left)=C`,
`neighbor(D,Left)=C`, `neighbor(A,Up)=None`. Validate builds THIS tree + asserts THESE.

**CARRY TO VALIDATE — the mutation kill map (~30 viable; the ~7 `Default::default()` whole-fn mutants on
DockState/PaneGroup/PaneId/&PaneGroup are UNVIABLE — no `Default` derives):**
- **DockState::toggled** (only an unviable Default mutant → coverage-only): `dock_state_toggles` must call
  `toggled(Open)==Closed` AND `toggled(Closed)==Open` (both arms, for coverage).
- **equal_ratios** (→vec![]/[0.0]/[1.0]/[-1.0]): assert the Split's `ratios==[0.5,0.5]` after a split (R11).
- **single/panes** (panes→vec![]): `single(A).panes()==[A]` (R7).
- **contains→true/false** (used by split/close guards): killed by R13 (absent→PaneNotFound) + R8/R9
  (present→Ok, not PaneNotFound).
- **split/split_at** (→Ok/delete-!/→()/guard-true/guard-false/==→!=): R8 — split A in `[A,B]` After →
  assert `Split{H,[Leaf(A),Leaf(B)]}` (the RIGHT leaf, right order, right axis); Before → `[Leaf(B),Leaf(A)]`;
  + split in a 2-leaf tree covers the Leaf-no-match + recurse arms.
- **close/close_at** (→Ok/delete-!/==→!=/→()): R9 — close A in `Split(H)[A,B]` → `Leaf(B)` (collapse); R10
  — `single(A).close(A)==Err(LastPane)` + tree unchanged; R13 — close absent → PaneNotFound; + a DEEP close
  (close A in the 2×2 → collapses its Split(V)) covers close_at's recurse.
- **neighbor** (→None; ==→!= axis; the sibling `< / >= / +1 / -1`; descend `-1`): FLAT `Split(H)[A,B]` —
  `neighbor(A,Right)=B`, `neighbor(B,Right)=None` (forward edge — kills `<→<=`/`==` via OOB), `neighbor(B,
  Left)=A`, `neighbor(A,Left)=None` (preceding edge — kills `>=→<`), `neighbor(A,Up)=None` (axis mismatch);
  NESTED 2×2 — `A-Right=B`/`A-Down=C`/`D-Left=C`/`A-Up=None` (covers the multi-level descend: first-child
  for R/D, last-child for L/U; kills the descend `-1` via OOB when mutated).
- **path_to** (→None/Some(vec![])/Some(vec![0/1])/==→!=): killed by the neighbor tests (a wrong path →
  wrong/None neighbor).

**Tests:** `dock_state_toggles`, `single_pane_has_one_leaf`, `split_before_and_after_order_and_axis`,
`ratios_renormalize_equal_after_split`, `close_collapses_two_child_split_to_survivor`, `close_last_pane_errs`,
`absent_pane_errs`, `neighbor_boundary_rule_flat_and_nested` — VERIFY the neighbor results empirically
(the probe confirmed the tree + A-Right=B; assert the rest per R12, adjusting if the run differs).

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate (2026-07-01)

Wrote the 8 tests in-context (fixtures built via the API + verified empirically). `cargo nextest run -p
marley layout` = **8 passed** — the neighbor test earned its keep by catching **TWO real defects**:

1. **`.then_some(index - 1)` underflow panic (a real §14 bug).** `bool::then_some(x)` evaluates `x`
   EAGERLY, so for `index == 0` on the preceding-sibling path (e.g. `neighbor(A, Left)` at a matching-axis
   ancestor) `0 - 1` underflowed → panic on a reachable input path. Fix: `index.checked_sub(1)` (None at
   the edge, no eager subtraction). → BF/PR captured at complete.
2. **A dead `if let PaneGroup::Split = node_at(...)` else-branch → coverage gap (line 214, 99.58%).**
   `node_at` on a path PREFIX always returns a `Split` (intermediate nodes are always Splits), so the
   "not a Split" else was unreachable — a defensive branch on an invariant-guaranteed value. Fix
   (source, not a floor drop): rewrote `neighbor` as a single recursive `find_neighbor` (an `Adjacency{
   Found|Pending|Absent}` search) — every `Leaf`/`Split` + `Found`/`Pending`/`Absent` arm is reachable —
   and DROPPED `path_to`/`node_at`. Same behavior (all 8 neighbor cases still pass), now 100% coverable.

**FULL gate (scripts/gates.sh --diff): `GATE GREEN [diff]` — 15/15**: cov 100% (layout.rs — every algebra
branch incl the recursive find_neighbor's reachable arms; app.rs shim excluded), mutation MSI 100% (the
viable split/close/neighbor/ratios/toggled surface killed), gate-15 PASS. Receipt written.

**Phase 4 status:** PASS. → Phase 5 Complete.
