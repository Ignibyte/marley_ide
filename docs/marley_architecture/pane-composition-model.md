# The pane-composition model — sections navigate, panes compose (M26 #388 spike)

**Status:** DESIGN (spike #388, M26 — decision-complete; produces the **M27 refactor train** below). No
code ships from this ticket. **Reference (§20):** the behaviour is chad's own (model A + the 2026-07-22
four-section refinement); Zed's `workspace`/`item`/`pane` are the *studied* reference
([../zed_architecture/subsystems/07-workspace-panes-palette.md](../zed_architecture/subsystems/07-workspace-panes-palette.md))
— **`[Zed-derived]` GPL, reimplemented CLEAN-ROOM in Marley's own code** (observe behaviour, never
translate the source; stays on the GPL editor side of the brain boundary). gpui (Apache-2.0) is
adoptable.

## The target (chad, 2026-07-22)

The left rail is **four fixed-order per-workspace sections** — **`Editor · Terminals · Panes · Browser`**
— where the **sections NAVIGATE what's open** and the **Panes section dynamically COMPOSES split
views**:

```
Workspace
├─ Editor      → File 1, File 2          (open editor files)
├─ Terminals   → Terminal 1, Terminal 2  (open terminals)
├─ Panes       → Pane 1, Pane 2          ← every SPLIT VIEW lands here, dynamically
└─ Browser     → Agents / Details / open browser   (3 rows since #411 — Agents the row-0 default)
```

The load-bearing rule: **the original stays open.** Splitting a terminal does not move it — Terminal 1
stays under Terminals, and its split *view* appears under Panes as a reference to it. A Pane is a **view
of an already-open thing**, never an owner. "Make a Pane" = save + name a multi-cell arrangement.
**Cross-workspace panes** (cells referencing content from >1 workspace) get a **top-level GLOBAL Panes
section** above the workspaces (gated on the multi-workspace model).

This **supersedes #385/#386's pane-nesting-under-tab** (the M9 #155 nesting): split panes will no longer
nest under their tab row — they list under Panes. The un-nesting is a slice of the M27 train below, **not
a #385/#386 amendment.**

---

## Q1 — The registry/handle model: **DECIDED — Marley's own `ContentId` registry** (not gpui `Entity`)

**The fork:** adopt gpui `Entity<T>` for shared content (Zed's way) vs. a Marley-original `ContentId`-keyed
registry.

**The facts (2026-07-22 gpui + code study):**
- gpui 0.2.2 already gives an app-owned, refcounted, id-keyed shared-state registry: `Entity<T>` (via
  `cx.new`, `Clone`, `.read`/`.update`, `WeakEntity::upgrade`); the `App`'s `EntityMap` *is* an
  `EntityId → Box<dyn Any>` map, and **N views share one state by cloning one handle**. It lives in the
  `gpui` crate.
- **Marley is ONE monolithic `RootView`** (~164 fields; a single `impl Render`; `cx.new` used exactly
  once, for the window root) with **no child entities** — and it **already hand-rolls the id-keyed
  side-table**: `agents: HashMap<PaneId, AgentRun>`, `remotes`, `notify_ticks`, `lsp_hosts:
  HashMap<PathBuf, LspHost>`, plus the model's own `panes: HashMap<PaneId, PaneState<S>>`.
- `tabs.rs` / `workspace.rs` / `grid_layout.rs` / `status_bar.rs` are **genuinely gpui-free** (zero
  `use gpui`) — the purity that gives cov/MSI 100 on every decision fn. Putting `Entity<T>` in any of
  them introduces the first `use gpui` there.

**Decision:** **keep Marley's own registry** — a new `ContentId` newtype; an app-side
`content: HashMap<ContentId, Content>` on `RootView`; the gpui-free model (tabs/workspace) references
content **by `ContentId`** instead of owning it inline. This is the direct generalisation of the
existing `agents`/`remotes`/`panes` id-keyed pattern.

**Why not gpui `Entity`:**
1. It would require the model layer to speak `gpui`, killing the pure-fn/cov-MSI-100 discipline (the
   thing that makes the eventual refactor safe).
2. Marley has **no child-entity infrastructure** to build on — adopting `Entity` means first fragmenting
   the monolithic `RootView` into child views (a separate, larger re-architecture), then threading
   `cx`/`Context` everywhere the model is unit-tested today.
3. Marley already owns this pattern; `ContentId` is a rename+generalise, not a new paradigm.

**One-instance-many-views** from the map: one `ContentId` ⇒ one `Content` entry; any number of tab rows
/ pane cells hold the same `ContentId` and all resolve to the one entry — the navigator semantics Zed
gets from a cloned `Entity<T>` handle. **The load-bearing catch (inspect):** a bare `HashMap` gives
*resolution*, NOT the refcounted drop-on-last-close gpui's `Entity` gives natively. So the registry MUST
carry a **per-`ContentId` view-count** (or a view-set of the referencing cells/tabs), and Marley's
existing close→teardown contract **moves to the registry**: today `PaneGrid::close` (workspace.rs:559)
returns the owned `PaneState<S>` whose `Drop` reaps the PTY off-thread (the child-reap blocks ~600ms —
workspace.rs:552); under the registry, closing ONE view *decrements the count* and only the **last**
close removes the `Content` entry and hands the owned session to the reaper thread. Every close path
(pump dead-pane, close-tab, close-project, `close_focused`, close-pane —
app.rs:1570/7206/7344/8659/17707) re-routes through this registry decrement. This refcount +
deferred-drop is the one thing the registry needs that a plain map lacks — a **slice-1 requirement, not
a footnote.** (If Marley later fragments `RootView` into child gpui views, `Entity<T>` — which refcounts
natively — may be adopted at the **render layer only**; the model stays id-based.)

**Content stays a closed enum, NOT a trait (for now).** `Content { Terminal, Editor, Cockpit, FileTree,
Git, Browser }` — **all four current `PaneKind`s (`Terminal/FileTree/CodeView/Git`, workspace.rs:210)
plus the tab-only Cockpit and the future Browser** (inspect: the first draft dropped `FileTree` + `Git`,
but a Git-diff cell is a live split-cell kind today — created at app.rs:5292 — so it needs a `Content`
variant and a `ContentId`). Owned by id in the registry; per-kind behaviour is the existing exhaustive
`match` (cov/MSI 100). Zed's open `Item` trait buys extensibility Marley doesn't need at ~6 kinds; the
map's own verdict is "the enum is a good default for a small variant set." **A `PaneItem` trait is a
deferred option** (revisit only if the kind count sprawls — e.g. many browser/preview kinds). The
registry indirection (`ContentId → Content`), not trait-ification, is what unlocks model A.

---

## Q2 — One instance, many views: the state-split table

The **instance** (owned once in the registry, keyed by `ContentId`) vs the **view** (owned per pane cell
/ per tab-mount, so N views of one instance each keep their own cursor):

| Per-INSTANCE (in the registry, one per `ContentId`) | Per-VIEW (one per pane cell / tab mount) |
|---|---|
| the `Buffer` (editor) / the `TerminalSession` PTY (terminal) | caret / `CharOffset` |
| undo history, dirty flag, `saved_version` | scroll offset / viewport anchor |
| language / LSP binding, syntax tree | text selection, find-bar state |
| the file path / cwd | the cell's focus flag |
| external-change (#275) watch state | column-anchor riders (#331/h-scroll) |

**Consequence for #259/#275:** #259 shipped "same file in tab + split = TWO Buffers" (guarded by the
#275 conflict machinery). Under this model that collapses to **ONE Buffer, two views** — the #275
tab-vs-split *self-conflict* disappears by construction (two views of one instance cannot disagree);
#275's **external**-change detection stays (the instance still watches disk). The per-view state above is
the minimal split that makes two carets in one buffer behave (Zed's multibuffer frame —
[../zed_architecture/subsystems/03-editor-multibuffer.md](../zed_architecture/subsystems/03-editor-multibuffer.md),
research only). **Open sub-question for the train's editor slice:** does undo stay per-instance (one
history, either view's ⌘Z rewinds the shared buffer — Zed's model) — **recommended yes**; a per-view
undo is a documented non-goal.

---

## Q0 / rail — the four-section Panes shape

- **`RailSection` gains a `Panes` variant** → `[Editor, Terminals, Panes, Browser]` (the #385 `ALL`
  grows to 4; `label()`/`from_label` grow a `Panes` arm; #386's `collapsed_sections` keys still work).
  Micro-decision (inspect): the Terminal section's `label()` becomes **"Terminals"** (plural, per chad's
  structure) — a one-line change with matching `from_label`/back-compat for the persisted key.
- **Split views un-nest.** `rail_rows` stops emitting `RailLevel::Pane` rows *under* a terminal tab
  (the #155/#385 nesting). Instead, after the per-section tab rows, it emits the **Panes section**: one
  row per LIVE split cell across the workspace's grids (a grid with >1 pane contributes one Pane row per
  cell; a single-pane tab contributes none). The list is derived fresh each render, so **a Pane row
  appears the moment a split is made and disappears when the cell closes.** A Pane row carries BOTH
  identities (inspect): the cell's existing `(project, tab, pane)` coordinate — the FOCUS key, since one
  `ContentId` may sit in N cells and only the coordinate says WHICH cell — and (once content is
  registry-backed) its `ContentId`, for the label + the Editor/Terminals cross-listing. Clicking a Pane
  row focuses that cell via the existing coordinate path (`jump_to_pane(PaneId)`, app.rs:6819).
- **Original stays.** The content a cell references is ALSO listed under its home section
  (Editor/Terminals) — because both the tab row and the Pane row hold the same `ContentId`. No
  duplication of the instance; two navigator entries into it.
- **A single-pane tab contributes NO Panes row** (it isn't a split view) — Panes lists only genuine
  splits, matching chad's "anything that opens a split view."
- **"Make a Pane"** (Q4) turns a live arrangement into a *named, persisted* Panes entry.
- **Global cross-workspace Panes** = a top-level `RailLevel::GlobalPanes` section emitted ABOVE the
  per-workspace rows, holding named arrangements whose cells span workspaces. **Designed, build-gated on
  the multi-workspace model** (Zed's `MultiWorkspace { retained_workspaces, active_workspace, sidebar }`
  is the reference — study before, not during). Until multi-workspace lands, the rail shows only the
  per-workspace Panes section.

---

## Q3 — The migration ripple (per-file, sized; the M27 surface)

Every seam that today owns content inline or keys off it. **S**=hours, **M**=a day, **L**=multi-day.

| Seam (file) | Change | Size | Test surface |
|---|---|---|---|
| **`ContentId` + `ContentRegistry`** (new, `workspace.rs` or a new `content.rs`) | the newtype + the `HashMap<ContentId, Content>` accessor API (insert/get/get_mut/remove, id allocation) | **M** | pure registry unit tests (insert/dedup/remove/id-monotonic) cov/MSI 100 |
| `PaneContent<S>` → `PaneState` holds a `ContentId` (`workspace.rs:317`) | panes stop owning `Box<TerminalPane>`/`EditorSurface`; hold an id; `kind()` reads the registry (or the id carries a `ContentKind` tag to keep `kind()` pure) | **L** | the id-tag keeps `kind()` a pure fn; grid tests move to ids |
| `TabContent<S>` → id-bearing (`tabs.rs:23`) | Terminal/Cockpit/CodeView variants carry a `ContentId` (+ a kind tag); `rail_section()`/accessors read the tag | **L** | `rail_rows`/`rail_section` tests re-keyed |
| accessors `grid()/editor()/cockpit_section()`, `PaneState::terminal()/code_view()` | resolve through the registry (or stay tag-based for the pure parts) | **M** | the #259 `active_editor()` focus-aware spine (~75 sites) moves once — sized by the #237/#259 precedent |
| `grid_layout.rs` codec (leaf `t/t=<cwd>/c=<path>/f/g`) | leaves persist a `ContentId`-resolvable key (path/cwd stays the identity; a Pane arrangement persists the id graph) — reserved framing bytes `,:=\x1f`/`\t\n\r` unchanged | **M** | codec round-trip + the #205/#258 leaf tests |
| `rail_rows` (`tabs.rs`) | +`Panes` section emission, − the #155 pane nesting; +GlobalPanes | **M** | the #385/#386 rail tests + new Panes-section tests |
| `focus_label`/`FocusTab` (`status_bar.rs`) | a Pane-cell focus reads its `ContentId`'s kind | **S** | #382's exhaustive match + a Panes arm |
| the FOUR `EditorSurface` births — `Tab::code` (tabs.rs:119), split-restore (app.rs:1968), split-create (app.rs:5262), `from_files` session-restore (app.rs:2089) | construct once → register → hand back a `ContentId` (kills the two-Buffers dup) | **M** | one construction path; #259 conflict test simplifies |
| the close→teardown contract (pump/close-tab/close-project/`close_focused`/close-pane — app.rs:1570/7206/7344/8659/17707) | `close` returns a `ContentId`, not the owned session → the registry decrements the view-count + hands the session to the reaper on LAST close (the ~600ms off-thread reap contract, workspace.rs:552) | **M** | registry refcount tests + the existing reap tests re-pointed |
| the app-side maps (`agents`/`remotes`/`notify_ticks`) | re-key from `PaneId` to `ContentId` where they track content (agents follow the instance, not the cell) | **M** | the pure map-consumer fns (agent_view/status_bar) re-keyed |

**Strangler order (validated instinct):** terminals FIRST — they are already semi-handle-based
(`S = TerminalSession` + the PaneId-keyed maps), so the registry generalises what exists with the least
new debt; editors SECOND (they carry the two-Buffers #259 debt the registry pays off); cockpit/browser
LAST (browser needs #389's substrate; cockpit is a thin `RightSection`). Every intermediate state
compiles + ships green (each kind converts behind the same accessor API).

---

## Q4 — Nameable pane arrangements (persistence)

A saved Pane = **a named grid layout + the `ContentId` graph its cells reference.** Persist via a **new
settings table** (the #204 `[[workflows]]` round-trip idiom), NOT the shell/grid codec — arrangements
are user-named, workspace-scoped (or global), and outlive any one session's grid:

```toml
[[panes]]
name = "review"
scope = "<project-root>"        # or "global" for cross-workspace
layout = "H:<cell>,<cell>"      # the split tree (grid_layout's algebra)
# each <cell> resolves a ContentId → a stable content KEY (path / cwd / cockpit-section),
# so a saved arrangement rebinds to the same file/terminal on restore.
```

**Dangling-id restore (the load-bearing edge):** a `ContentId` is session-local; the persisted form
stores the content's **stable key** (a full path for a file, a `RightSection` for cockpit — both stable
AND unique; the same keys the #205/#258 leaf codec uses). On restore, each cell re-resolves its key → an
existing-or-fresh `ContentId`. A key that no longer resolves (file deleted) → that cell is **dropped from
the arrangement** (the #205 `is_dir`/`is_file` guard stance), never a phantom or a panic. **Terminals are
the lossy case (inspect):** their key is cwd-based, a restored terminal RESPAWNS fresh-in-cwd (not
re-attached — scrollback/process gone, per #205's respawn), and cwd is NON-unique — so a persisted
arrangement carries a per-cell **ordinal** (cell 1/2 within the arrangement) so two same-cwd terminal
cells don't collapse onto one instance. (Marley editors always carry a path, so the untitled-buffer edge
doesn't bite today.) A whole arrangement whose cells all drop →
the arrangement is skipped (the #386 forgiving-restore posture).

---

## Q5 — The M27 refactor train (ordered, each shippable green)

Handed to `/spec` as the M27 sprint. Each slice keeps the gate green; the rail keeps working throughout.

1. **✅ SHIPPED (M27 #394) — `ContentId` + `ContentRegistry`** — the pure, gpui-free, **generic
   `ContentRegistry<C>`** (`content_registry.rs`): a `ContentId` newtype + `insert`(1 view)/`acquire_view`/
   `release_view`(returns the owned content on the last-view drop)/`get`/`get_mut`/`view_count`/`len`, the
   explicit view-count carrying the drop-on-last-close. Wired but UNUSED (additive; cov/MSI 100), gate-green
   via the #371 `orchestration_for` `pub use` template. **D5 SPLIT:** the queued spec fused slices 1+2; the
   design split them — this shipped ONLY the lifecycle (generic over `C` so it's proven purely with a test
   double, no live PTY), and **slice 2 became its own ticket #396** (`ContentRegistry<Content>` +
   terminals). No behaviour change. *(shipped M)*
2. **✅ SHIPPED (M28 #396) — Terminals onto the registry** — terminal panes/tabs hold a `ContentId`
   VIEW; the registry owns the `Box<TerminalPane<TerminalSession>>` (`Content::terminal` is the one
   birth idiom: spawn → insert → mount, leak-free by order); every accessor resolves at the
   RootView level (`focused_terminal(_mut)` / `workspace_terminal(_mut)` / `pane_of_content`); all
   close paths route through `release_view` — `#[must_use]`, one off-thread reap per gesture (the
   TICKET-348 contract preserved); persistence rebuilds from shapes (ids never serialize);
   `agents`/`remotes`/`notify_ticks`/`last_agent`/`pending_agent_send` re-key to `ContentId` (the
   instance) while `completion`/block-menu/`tab_flashes`/the MCP handle stay view-keyed. The
   S-genericity DISSOLVED (`PaneGrid<S>` → `PaneGrid`; `TerminalPane<S>` keeps its param as the
   payload type). Recorded deltas (inspect-sanctioned): Fleet/⌘K rows + broadcast order by LAUNCH
   order now; `mcp_surface_index` omits unmounted agents; non-PTY container payloads drop inline.
   *(shipped L — the load-bearing migration, its own reviewed ticket per the D5 split)*
3. **✅ SHIPPED (M28 #397) — Editors onto the registry** — the #259 two-Buffers dup is dead: one
   `EditorInstance` per open file (buffer + undo + dirty + the #275 snapshot) under
   `Content::Editor`; tab file rows and split cells are VIEW rows `(ContentId, view-half)` — the Q2
   split shipped as designed (per-view: lines cache re-keyed `(nonce, version)`, IME span, scroll
   parks by a per-row `view_key`, parked selection). **The first visible payoff landed:** the same
   file in a tab and a split shares edits live, one dirty ●, ONE ⌘Z history (open-Q3 resolved as
   recommended — undo is per-instance, either view invokes it; the observed Zed behavior). The
   selection home is a STICKY park/restore choke (`sync_editor_selection_home`, render top + the
   head of `active_editor_mut`; rows birth-seeded with a caret-0 park so a fresh twin never
   inherits the sibling's cursors); births route the ONE `resolve_open` seam (`(root, same_file)`
   scope — an open hit does zero disk IO); closes release per row, last release drops INLINE (no
   reaper analog); LSP/search/references sets derive from the registry (split-only files join; one
   uri per file structurally); persistence byte-identical (ids never serialize; tab+split restores
   one instance/two views from one read). App-side joins `EditorRef`/`EditorMut` mirror the old
   surface API so the ~75 accessor sites kept their shapes. #275 self-conflict removed
   structurally; `extchange` untouched. Recorded deltas: same-root projects union into the shared
   host's set; the rename applier goes through the registry (lex-first root on nested two-root
   opens); split files gain disk snapshots; twins share syntax/fold memos (one parse per file).
   *(shipped L)*
4. **✅ SHIPPED (M27 #390) — The Panes section (un-nest)** — `RailSection::Panes`; `rail_rows` lists
   split views under Panes **keyed by the `(project, tab, pane)` COORDINATE** (registry-free — so this
   shipped FIRST of the train, ahead of the registry, exactly as this slice predicted), removing the #155
   nesting. Delivered as one **`RailLevel::Arrangement`** "PANE n" row per multi-cell tab with the cells
   nested beneath (chad's locked unit = arrangement); click-to-focus the tab via the #174 idiom; Panes＋ =
   `split_focused_pane`. **chad's four-section rail landed here.** The `ContentId`-per-cell label +
   home-section cross-link is still deferred to slice 5 (needs terminals+editors registered). *(shipped M)*
5. **✅ SHIPPED (M28 #398) — Add-any-thing-to-a-pane + the `ContentId` cross-link** — the verb set
   that VIEWS open content into a split cell (`MenuKind::AddToPane` right-click rows + the
   rebuilt-at-open `ADD_TO_PANE_BASE` palette range → the one `add_content_to_split` dispatch:
   `acquire_view` + split the active tab's focused pane; activate-first per #174), making
   view_count > 1 REAL (one PTY in two cells across tabs; the live Buffer in an added editor
   cell). The cross-link half as shipped (inspect-corrected from the drafted per-cell rule): Pane
   CELL rows read content labels (the shim-built `ContentLabels` + pure `cell_label`; "PANE n"
   headers stay positional until #399), a terminal hosted by ≥2 DISTINCT tabs marks every hosting
   tab's row ⊞ (`terminal_hosts` — a tab row is the home entry for ALL its cells; per-cell
   terminal cross rows would have tripled the section on every plain split), and a pane-mounted
   FILE gets a per-file `RailLevel::CrossRef` row under Editor (its only home entry; the focused
   CodeView cell owns the active section so it can't be collapse-hidden). Lifecycle riders:
   content-keyed map scrubs moved to last-view release (#353 extends the same scrub to the
   editor side: `RootView::release_editor_views` clears `editor_folds` at the last same-path
   INSTANCE drop, census-guarded so an alias-root twin instance keeps the shared entry;
   #412 rides `git_marks` + its cache key on the same arm); one
   resize authority per content
   (focused-cell-wins; D-TERM-VIEW-STATE = shared display state v1); the pump ticks per content.
   Q5.4's deferred label+cross-link half closes here. *(shipped M)*
6. **✅ SHIPPED (M28 #399) — Nameable arrangements** — the `[[panes]]` settings table (the #204
   `Vec<T>` template: `name` + `scope` root + flat `axis` + per-cell `kind`/`key`/`ordinal` —
   ids NEVER serialize; the ordinal carries instance identity, so #398 twins share one and two
   same-cwd shells count up, and a future kind's cells load + drop at resolve + round-trip
   byte-preserved); the "Name Pane…" palette verb + Arrangement-row double-click open the
   #204-shaped inline card; `Tab::pane_name` binds the rail label (unnamed stays "PANE n"
   byte-identically; empty commit UN-names per #177 — the durable entry outlives the binding);
   the name restarts via a second `\x1f` rider on the shell `T=` entry (the #163 blob stays the
   ONE rebuild authority — `[[panes]]` never drives boot, so a closed named tab never
   resurrects); reopen-by-name "Pane: {name}" rows resolve with the dangling-DROP stance (a dead
   cwd drops its cell, not root-fallback — the arrangement promised THAT cwd; all-dropped skips
   with a flash) and rebuild through the registry so twins reunite (`insert` once,
   `acquire_view` the rest) in the persisted DFS order. *(shipped M)*
7. **(gated) Global cross-workspace Panes** — the top-level GlobalPanes section, once multi-workspace
   exists. *(L, gated)*
8. **✅ SHIPPED (M28 #400) — Cockpit onto the registry (the cockpit half; Browser stays gated on
   #389)** — Details / Agents become `Content::Cockpit(RightSection)` registry residents (a third,
   Forge, retired at #411 — the scrap-forge rip):
   app-wide SINGLETONS resolved through the ONE `resolve_or_register_cockpit` door with a
   slot-per-section `CockpitIndex` (3 slots at #400, 2 since #411; lazy — a section never opened
   has no entry), so two projects' tabs of
   one section are two VIEWS of one instance (the app's first legitimately >1-view content).
   `TabContent::Cockpit(ContentId, RightSection)` is the Q1 `(id, tag)` shape — the tag keeps
   `cockpit_section`/`rail_section`/the `C=` codec writer registry-free, and the codec is
   byte-identical (ids never serialize; live-verified `cmp`-identical renders + wire).
   Lifecycle is **PINNED**: the insert's first view is a standing anchor the index holds
   forever, so every user-reachable `release_view` returns `None` (asserted, not assumed —
   `release_cockpit_views` is the cockpit-ids-only close seam; a dead-slot desync self-heals by
   re-registering). Identity is the WHOLE payload — the surfaces stay pure projections over
   app-side service state, and `right_section` (the #95 persisted selection) stays dock-side.
   `addable(Cockpit)` stays `false`: residency landed, the pane-cell arm (`PaneContent`) did
   not — cockpit-in-a-pane is the exposure follow-up per D-OPEN-KIND-GATING. *(shipped M)*
8b. **✅ SHIPPED (M29 #403) — Browser onto the registry (the gated half, un-gated).** The Browser
   kind is now a TAB citizen: `TabContent::Browser(ContentId)` viewing a single app-wide
   `Content::Browser` resident, resolved through the ONE `resolve_browser` door and persisted as a
   bare `B` shell entry. Two things make it the *counter-example* to slice 8 rather than a copy of
   it. **(a) No tag beside the id.** Cockpit's `(id, tag)` bought registry-free purity because three
   sections share one variant and the codec writes a per-instance key; Browser has one kind and a
   payload-less marker, so the variant discriminant already discriminates — a tag field would be
   pure redundancy. The tag slot grows only if a multi-browser identity ever arrives, with the codec
   payload it would then need. **(b) The lifecycle is DROPPED, not pinned — and the wiring shape
   does not transfer.** `open_cockpit_tab` acquires a view only on append, correct *only* because a
   pinned anchor already holds the insert's first view. With no anchor, the insert's first view IS
   the first tab's, so the door must hand back exactly one acquired view in both branches (the #397
   `resolve_open` contract) and an unconsumed one must be released. Copying the cockpit shape here
   would have stranded a view forever. Dropped was chosen while the payload is still a unit — where
   either is safe — because the #405 webview is the reapable thing, and pinning would have baked in
   a leak-shaped contract needing reversal. `release_browser_views` is where that teardown will hook
   in (and where it must move OFF the UI thread, the `release_grid_terminals` hazard).
   `addable(Browser)` stays `false` on the same terms as Cockpit: residency landed, no `PaneContent`
   arm — the grid `b` leaf is the #389 Q4 deferred layer. *(shipped M)*

---

## Architecture decisions (recorded to the knowledge ledger, `docs/planning/knowledge/` — the forge home retired at #409)

- **`AD-claude-pane-content-id-registry-001`** — Marley keeps its OWN `ContentId`-keyed registry over content;
  gpui `Entity<T>` is NOT adopted in the gpui-free model layer (it would break cov/MSI-100 purity, and
  Marley already hand-rolls the id-keyed pattern; one-instance-many-views is a `HashMap` lookup).
- **`AD-claude-one-instance-many-views-001`** — content is owned ONCE per `ContentId` in the registry; tab rows
  and pane cells are VIEWS holding the same id; per-view state = caret/scroll/selection/focus, per-
  instance = buffer/PTY/undo/dirty/LSP. Supersedes the #259 two-Buffers stance.

## Open questions (for the train's slices, not blocking)

1. Does `PaneState`/`TabContent` carry a `ContentKind` TAG alongside the `ContentId` (to keep `kind()`
   /`rail_section()` pure, no registry read) — **recommended yes** (a `(ContentId, ContentKind)`).
2. The model's `S` genericity: with content in the app-side registry, the model holds ids + tags (no
   longer generic over the session handle for content) — a simplification to confirm in slice 1.
3. Undo scope across views — **recommended per-instance** (one history; either view's ⌘Z rewinds).

## Related
[app_shell.md](app_shell.md) (the rail lineage #152→#385→#386→this) ·
[../zed_architecture/subsystems/07-workspace-panes-palette.md](../zed_architecture/subsystems/07-workspace-panes-palette.md) ·
[../zed_architecture/subsystems/03-editor-multibuffer.md](../zed_architecture/subsystems/03-editor-multibuffer.md) ·
[roadmap.md](roadmap.md) (M26 → the M27 refactor train).
