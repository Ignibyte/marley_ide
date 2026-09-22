# 388-spike-pane-handle-model — Notes

## Phase 1 — Plan (drafted 2026-07-22, /spec batch, Fable)
- **Request:** chad locked model A (sections navigate, panes compose, nameable arrangements);
  the code map shows the blocker — inline-owned pane content — so the refactor gets a
  decision-complete spike before any code train.
- **Sprint:** #37 M26; forge #388 `4c8cbe75-f7c6-4d98-9893-130e22d922e1`.
- **The blocker, precisely (2026-07-22 Explore map):**
  - `PaneContent<S>` workspace.rs:317-332 — closed 4-kind enum, owns `Box<TerminalPane<S>>` /
    `EditorSurface` BY VALUE in `PaneGrid.panes: HashMap<PaneId, PaneState<S>>`.
  - `TabContent<S>` tabs.rs:23-31 — independently owns its `PaneGrid`/`EditorSurface`/section.
  - Same file in tab + split pane = TWO `EditorSurface` values (construction sites tabs.rs:64 vs
    app.rs:1962/:5255) = two Buffers, #275 conflict machinery as the net (#259's stance).
  - The asymmetry to exploit: terminals are semi-handle-based already (`S = TerminalSession` +
    PaneId-keyed agents/remotes maps) — the generalization seed.
  - Ripple list (REQ-002's floor): PaneContent, TabContent, kind() derivation, accessors, grid
    leaf codec (`t/t=<cwd>/c=<path>/f/g` + reserved `,:=\x1f`), rail_rows, focus_label, the two
    EditorSurface sites.
- **Central fork for P2/P3:** gpui `Entity<T>` adoption (render layer) vs own ContentId registry
  end-to-end, against the gpui-free pure-model discipline (D2). gpui source read = adoption
  (Apache-2.0); Zed maps = research only.
- **Prior-art sweep:** recorded in spec (the Zed 07/03 maps are the existence proof; gpui is the
  highest-yield leg; in-repo seams listed).
- **Open items:** none blocking promotion — the spike IS the open item, structured.
- **AAR:** `0181d2c0-8bf8-4e9e-bf7a-41d4cdbeeb8d`. **chad's 2026-07-22 4-section correction folded in**
  (spec question 0 + D5 + REQ-006): rail = `Editor · Terminals · Panes · Browser`; Panes dynamically
  collects split views (original stays home); global cross-workspace Panes.

## Phase 2 — Design (2026-07-22)

### Approach (this is a SPIKE — the deliverable is a design DOC, no shipped code)
The design phase plans the doc + runs the study; implement WRITES it. **Doc location:**
`docs/marley_architecture/pane-composition-model.md` (a new architecture doc, cross-linked from
app_shell.md). Structure = the 6 spec questions + the central fork + the M27 train.

**The study (in flight / done):**
- **gpui `Entity<T>` model (agent a2f5c8ca, running)** — the central-fork input: the gpui 0.2.2
  entity/handle API + whether Marley uses gpui entities at all (I expect Marley = ONE monolithic
  `AppView` owning everything inline + `impl Render`, no child entities — which shapes the fork).
- **Zed 07 map (READ):** the existence proof of model A — Zed's `Item: Focusable+EventEmitter+Render`
  → `Box<dyn ItemHandle>` (blanket-impl'd for `Entity<T>`); `Pane.items: Vec<Box<dyn ItemHandle>>`;
  items are gpui ENTITIES, so N views share one instance for free. The map's own ROI verdict already
  names the target: **"a `PaneItem` trait to generalize `PaneContent`, impls THIN over the existing
  pure fns so cov/MSI 100 holds"** — the strategic re-architecture pillar (its trigger = editor-as-peer
  + browser, i.e. the peer count 4→8+, which #389's Browser adds). Also: Zed's `MultiWorkspace`
  (many workspaces/window + a sidebar switcher) is the reference for the GLOBAL cross-workspace Panes
  section (needs the multi-workspace model). `workspace`/`item`/`pane` are `[Zed-derived]` GPL →
  reimplement CLEAN-ROOM in Marley's own code (fine for the GPL editor layer, off the brain boundary).
- **Zed 03 map (multibuffer)** — question 2's frame (per-view vs per-buffer state); READ in implement.

### The central fork — my LEAN (to be confirmed by the gpui-entity facts + argued in the doc)
**Keep Marley's own `ContentId`-keyed registry; do NOT push gpui `Entity<T>` into the model layer.**
Rationale: tabs.rs/workspace.rs/grid_layout.rs are gpui-FREE and that purity is what buys cov/MSI 100
on every decision fn. gpui `Entity` in the model would break it. Instead generalize Marley's EXISTING
"semi-handle" pattern (the PaneId-keyed agents/remotes app-side maps): the gpui-free model references
content by a pure `ContentId`; an app-side `ContentRegistry` (in `AppView`, which IS the gpui layer)
owns the one instance per id. A Pane (Panes section) or a Tab (Editor/Terminals section) both hold a
`ContentId` → one instance, many views, by construction. gpui `Entity` MAY still be adopted at the
RENDER layer only (if Marley moves to child views) — but the MODEL stays id-based. (This is the
Marley-original inverse of Zed's entity-handle: same one-instance-many-views destination, our pure-fn
discipline preserved.)

### The Panes-section shape (question 0 — the centerpiece)
- Rail = 4 fixed per-workspace sections. `RailSection` gains a `Panes` variant (Editor/Terminals/
  Panes/Browser). #385/#386's pane-nesting-under-tab is REMOVED — split panes no longer nest under
  their tab; they list under Panes.
- A split view = a grid cell referencing a `ContentId`. `rail_rows` emits, under Panes, one row per
  live split cell (across the workspace's grids), each a view of its `ContentId`; the referenced
  content STAYS listed under its home section (Editor/Terminals). Clicking a Pane row focuses that
  cell.
- "make a Pane" = persist a named multi-cell arrangement (grid layout + the ContentIds it references)
  — question 4's codec.
- GLOBAL cross-workspace Panes = a top-level section above the workspaces, holding panes whose cells
  reference ContentIds from >1 workspace. Requires the multi-workspace model (Zed `MultiWorkspace`
  reference) → DESIGN the shape, GATE the build on multi-workspace landing.

### File manifest
Docs only (the spike ships ZERO `crates/*` code — D3): `docs/marley_architecture/pane-composition-model.md`
(new) + a one-line cross-ref added to `app_shell.md`. Plus forge ADs (handle model; one-instance-many-
views) + the emitted M27 ticket-slice list (in the doc, handed to /spec).

### Regression Test Plan (a SPIKE — verify = doc review + gate)
| # | Check | Proves |
|---|---|---|
| V1 | The doc records the chosen handle/registry model (the gpui-Entity vs own-ContentId fork, decided with rationale) + the one-instance-many-views state-split table | REQ-001 |
| V2 | The doc carries the full migration-ripple inventory, per-file S/M/L + named test surface, ≥ the 8 map seams | REQ-002 |
| V3 | The doc records nameable-arrangement persistence + dangling-id restore | REQ-003 |
| V4 | forge ADs recorded + an ordered one-slice-each M27 ticket list emitted | REQ-004 |
| V5 | `git diff` under `crates/` is EMPTY at complete | REQ-005 |
| V6 | The doc specifies the 4-section rail + dynamic split-view→Panes + original-stays + Pane-row identity/focus + the global cross-workspace section (shape + multi-workspace gate) | REQ-006 |
| G | `scripts/gates.sh --fast` green (no-`.rs` static set) | §0 |

Uncoverable by unit test: N/A — the spike ships no code; verification is doc review + the empty-crates
diff + `--fast`.

### Risks / decisions
- **D-OWN-REGISTRY (lean, pending agent):** own `ContentId` registry over gpui `Entity` in the model —
  preserves the gpui-free purity. If the gpui-entity facts show Marley already leans on entities
  heavily, revisit.
- **D-PANES-UN-NEST:** the Panes section REPLACES #385/#386's pane-under-tab nesting (a real behavior
  change, shipped in the M27 train — NOT a #385/#386 amendment). The doc must sequence the un-nesting.
- **D-GLOBAL-GATED:** the global cross-workspace Panes section is DESIGNED now but its BUILD is gated
  on the multi-workspace model (still ahead) — the doc says so explicitly.
- **D-CLEAN-ROOM:** the `PaneItem`-trait shape is INSPIRED by Zed's `Item` (observed behavior), written
  clean-room in Marley's own code; never a translation of the GPL `workspace`/`item` source (§20).
- **D-THIN-IMPLS:** every content-kind's handle impl delegates to the EXISTING pure fns (the Zed-map's
  "impls thin → cov/MSI 100 holds" discipline) — the refactor must not regress the decision-fn purity.

## Phase 3 — Implement (2026-07-22)
The spike's "implementation" is the design DOC. Written:
- **`docs/marley_architecture/pane-composition-model.md`** (new) — decision-complete, all 6 questions +
  the central fork + the ripple + the M27 train. Central fork **DECIDED** (gpui-entity agent a2f5c8ca
  confirmed the facts): Marley keeps its OWN `ContentId` registry — the gpui-free model references
  content by id; gpui `Entity<T>` NOT adopted in the model (would break cov/MSI-100 purity; Marley is
  monolithic with no child-entity infra + already hand-rolls the id-keyed pattern). One-instance-many-
  views = a HashMap lookup; the per-view/per-instance state-split table records it. The 4-section Panes
  rail + dynamic split-view allocation + global cross-workspace (gated) + nameable arrangements (a
  `[[panes]]` settings table with stable-key/dangling-id restore) + the ripple (per-file S/M/L) + the
  ordered 8-slice M27 train are all specified.
- **`app_shell.md`** — a cross-ref note added (the rail lineage #152→#385→#386→#388).
- **ADs:** the two decisions (ContentId registry; one-instance-many-views) are recorded in the doc's
  "Architecture decisions" section. **The forge `architecture-decision-record` tool erred** (invalid
  args: missing field `decision`, on a well-formed payload — a tool-side quirk; the sibling forge
  tools worked) → **captured locally in the doc (§19 forge-soft).** Retry the forge AD write when the
  tool is healthy (codes: `AD-claude-pane-content-id-registry-001`, `AD-claude-one-instance-many-views-001`).
- **Zero `crates/*` code** — docs only (D3 / REQ-005 holds by construction). `cargo` not run (no code).

## Inspect (Phase 3.5) — 2026-07-22
2 critics attacked the design doc (critic 1 malfunctioned once — returned a skill-artifact, 0 tool uses;
re-spawned with a "verify these checks, don't load a skill" framing → real results). **Both landed
substantive findings; my own self-review had been too lenient.** Doc is decision-solid + genuinely
clean-room; the fixes below sharpen it. No code (docs-only spike). Verdicts:

- **[MED-HIGH → FIXED] a `HashMap` registry doesn't refcount** (critic 1 CHECK 2/3). The doc called
  one-instance-many-views "a HashMap lookup / comes for free" — but a bare map gives resolution, NOT the
  Drop-on-last-close gpui `Entity` gives. Today `PaneGrid::close` returns owned `PaneState<S>` whose
  `Drop` reaps the PTY off-thread (~600ms). **Fix:** Q1 now states the registry MUST carry a per-ContentId
  view-count + the close→reaper contract MOVES to the registry (last-close reaps); added a ripple row for
  the 5 close paths (app.rs:1570/7206/7344/8659/17707). A slice-1 requirement, not a footnote.
- **[MED → FIXED] the `Content` enum dropped `FileTree` + `Git`** (critic 1 CHECK 3) — live split-cell
  kinds (`PaneKind` = Terminal/FileTree/CodeView/Git; a Git-diff cell at app.rs:5292). **Fix:** enum is
  now `{Terminal, Editor, Cockpit, FileTree, Git, Browser}` (all 4 PaneKinds + Cockpit + Browser).
- **[MED → FIXED] Pane-row identity** (both critics) — a `ContentId` can sit in N cells, so it can't be
  the focus key. **Fix:** a Pane row carries BOTH the `(project,tab,pane)` coordinate (the focus key, via
  `jump_to_pane` app.rs:6819) AND the ContentId (label + cross-link).
- **[MED → FIXED] slice 4 over-coupled** (critic 1 CHECK 4) — the Panes-section display is registry-FREE
  (list cells by coordinate, focus by PaneId) and Git/FileTree cells never get a ContentId. **Fix:**
  slice 4 keys by `(tab,pane)` + ships after slice 1; the ContentId cross-link moves to slice 5.
- **[MED → FIXED] terminal dangling-key** (critic 1 CHECK 5) — cwd is neither stable nor unique; a
  restored terminal respawns FRESH (lost scrollback). **Fix:** Q4 now says files/cockpit keys are
  stable+unique, terminals respawn fresh-in-cwd + need a per-cell ordinal so same-cwd cells don't collapse.
- **[MED → FIXED] EditorSurface citations wrong/undercount** (critic 2) — `tabs.rs:64` is a doc comment;
  the real births are `Tab::code` (tabs.rs:119), split-restore (app.rs:1968), split-create (app.rs:5262),
  `from_files` (app.rs:2089) — FOUR, not two. **Fix:** ripple row re-anchored.
- **[LOW → FIXED] AD slugs were `…` placeholders** → real (`AD-claude-pane-content-id-registry-001`,
  `AD-claude-one-instance-many-views-001`). **[LOW → FIXED]** "disappears when closed" made explicit.
  **[NIT → FIXED]** the Terminal→"Terminals" label rename recorded as a micro-decision.
- **[clean, both critics] REQ-006** (4 sections / un-nest / original-stays / global-gated all present),
  **clean-room §20** (the closed-enum+own-registry is the INVERSE of Zed, genuinely Marley-original — a
  `PaneItem` trait explicitly deferred, not Zed's `Item` renamed), **REQ-005** (empty crates/ diff twice),
  **the bulk of the factual citations** (gpui 0.2.2, single `impl Render`, the 4 gpui-free modules, #155
  nesting, framing bytes, #259/#275 — all verified accurate).
- **[LOW — noted, not changed]** the MultiWorkspace field-names are a `[Zed-derived]`-flagged research
  pointer (derive Marley's own names when the gated global-panes slice is built); `~75 active_editor` +
  `~164 fields` are hedged with `~` and the sizing purpose survives.

**Forge:** `prevention-rule-record` PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001
(the load-bearing catch). No failure-record — the "defects" were incomplete design-doc sentences caught
+ fixed at inspect, not code bugs. (The two ADs remain doc-local — the forge `architecture-decision-record`
tool errs; retry when healthy.)

## Phase 4 — Validate (2026-07-22)
Docs-only spike → **no unit tests, no driven capture** (ships zero code; the doc describes future M27
work, no UI/render change). Verification = doc-review REQ checklist + the gate:
- **REQ-005 (zero code):** `git status --porcelain -- crates/` **empty** (re-confirmed). No TODO/FIXME
  in the doc (doc-todos gate).
- **V1–V6 (doc review):** the doc is decision-complete after the inspect fixes — V1 the fork (own
  ContentId registry, decided + the refcount caveat), V2 the ripple (per-file S/M/L + the close-teardown
  row), V3 nameable-arrangement persistence + dangling-id (files stable, terminals ordinal), V4 the M27
  8-slice train + the 2 ADs (doc-local; forge tool erred), V5 empty crates, V6 the 4-section rail +
  dynamic Panes + original-stays + global-gated + Pane-row dual identity.
- **Gate:** `scripts/gates.sh --fast` → **GATE GREEN [fast] 11/11** (fmt/clippy/tests/audit/deny/machete/
  gitleaks/shellcheck/no-suppressions/source-bans/**docs** — the docs gate incl. brand-scrub passed with
  the doc's Zed research citations intact). gate:4/5/6/15 SKIP (--fast, no `.rs` to cover). A `--fast`
  run writes no receipt; the docs-only commit is ungated by the commit-gate hook (no `.rs` in the change
  set), so `--fast` green is the validation.

## Phase 5 — Complete (2026-07-22)
- **Docs (§21):** the deliverable IS `docs/marley_architecture/pane-composition-model.md` (new); the
  `app_shell.md` cross-ref is added; a brief CHANGELOG entry (docs/spike) under `### Added`.
- **Knowledge (forge):** `aar-submit` (aar 0181d2c0, completed, effectiveness 5, 1 novel finding);
  PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001 recorded at inspect; ticket-comment
  + ticket-close #388. The 2 ADs are captured in the doc (forge `architecture-decision-record` tool
  erred; retry when healthy: `AD-claude-pane-content-id-registry-001`, `AD-claude-one-instance-many-views-001`).
- **Handoff:** the **M27 refactor train** (8 ordered slices in the doc's Q5) → `/spec` when chad
  greenlights the pane refactor.
- **Archive:** ticket → tickets/closed/; pipeline pair → completed/.
- **Ready for /commit** (3rd of /work 385-389, auto-approved → commit LOCAL, hold push; docs-only).
