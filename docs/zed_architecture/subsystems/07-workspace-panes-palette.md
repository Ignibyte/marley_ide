# Subsystem 07 — Workspace, Panes, Command Palette & the UI Shell (Zed)

> Part of the Marley architecture docs. Zed (github.com/zed-industries/zed) is the **editor / IDE-shell**
> reference for Marley — the counterpart to `../warp_architecture/` (the terminal / cockpit reference). This
> document deconstructs Zed's application-shell subsystem: the **workspace / pane / item** model, the **dock /
> panel** system, the **command palette**, the **picker** abstraction, the **file finder** + **outline panel**,
> and the **`ui` component library** — and, for each, compares it to **Marley's already-built shell** to mark
> where Marley matches Zed and where the gap is.

## Scope & why this matters for Marley

Marley is *already strong here*. Post-M15 it has a real workspace shell: a `Workspace → Project → Tab`
hierarchy, docks (#24), a recursive `PaneGroup` split algebra (#20/#23), a command palette + `filter_commands`
(#19/#25), a ⌘P file finder (#97), a ⌘R history overlay, a Workspace→Project→Tab side rail (#152), a unified
titlebar (#142), and a workspace switcher + launcher (M13/M14). So this is **not** a "Marley is missing a shell"
map. It is a **generality / extensibility** map: Zed's shell is built almost entirely from **open traits +
object-safe handles**, while Marley's is built almost entirely from **closed enums + a central action-string
router**. That single difference is the spine of every comparison below, and it is exactly the axis that starts to
bite as Marley grows peer surfaces (the M13 editor-as-peer and the embedded-browser intake).

The two questions the rest of this doc answers:

1. **Is Marley's pane model as general as Zed's?** (Zed makes editor + terminal + search + diagnostics all
   first-class pane *items* via one `Item` trait; Marley's `PaneContent::{Terminal, FileTree, CodeView, Git}` is
   the analog.) — **No, and deliberately so; the gap is real but is a simplicity trade, not an oversight.**
2. **What should Marley adopt, and in what order?** — Short answer: **the picker abstraction first** (cheap, high
   ROI), **a pane-item trait second** (strategic, tied to the editor/browser work), a **panel trait** and **item
   facet traits** only when the dock and persistence wiring actually sprawl.

## Provenance & licensing posture

- Everything in the `workspace`, `command_palette`, `command_palette_hooks`, `file_finder`, `outline_panel`,
  `outline`, `picker`, `title_bar`, `ui`, and `component` crates is **`[Zed-derived]` (GPL-3.0-or-later)** —
  Zed's own application model. `crates/ui/Cargo.toml` and `crates/component/Cargo.toml` declare
  `license = "GPL-3.0-or-later"` explicitly.
- The primitive layer these all build on is **`[gpui Apache-2.0]`**: `div()`, `Styled` (tailwind-ish method
  chains), `InteractiveElement`/`StatefulInteractiveElement`, `ParentElement`, `RenderOnce` + `#[derive(IntoElement)]`,
  `Render`/`Focusable`/`EventEmitter`, `Entity`/`WeakEntity`, `Action`, `FocusHandle`, `uniform_list`/`ListState`,
  `FluentBuilder` (`when`/`when_some`/`map`). This is the same layer Marley already depends on directly.
- **Boundary for Marley (open-core model):** Marley's editor/terminal layer is intended GPL, so reimplementing
  these Zed patterns *in Marley's own clean-room code* is licence-compatible for the editor layer. The rule from
  `../README.md` still holds — the proprietary **brain / agent layer** must stay clean of Zed-derived code. The
  pane/item/picker/panel patterns belong on the editor side of that line. `[Marley-original]` tags below mark
  where Marley's existing design already diverges (and is worth keeping).

## Crates surveyed

| Crate | Role |
|-------|------|
| `workspace` | The heart. `Workspace` (center `PaneGroup` + 3 docks + status bar + modal/toast layers + titlebar slot), the **`Item`/`ItemHandle`** pane-content model, `Pane`, `PaneGroup`/`Member`/`PaneAxis` split algebra, `Dock`/**`Panel`**/`PanelHandle`, `MultiWorkspace` (many workspaces per window + switcher sidebar), `ModalLayer`/`ModalView`, `StatusBar`/`StatusItemView`, `Toolbar`. |
| `picker` | The generic **`Picker<D: PickerDelegate>`** modal-list host: query editor, fuzzy list, selection, preview pane, footer/actions. Every fuzzy overlay is a delegate. |
| `command_palette` | `Picker<CommandPaletteDelegate>` over gpui's **action registry** (`available_actions`), humanized names, keybinding display, re-dispatch on the original focus. |
| `command_palette_hooks` | The two globals that make the palette extensible: `CommandPaletteFilter` (hide/show actions) + `CommandPaletteInterceptor` (rewrite queries, e.g. vim `:` commands). |
| `file_finder` | `Picker<FileFinderDelegate>`: fuzzy worktree path search unioned with recent-navigation history, `path:row:col` parsing, split-open. |
| `outline_panel` / `outline` | Two views over one `language::Outline`: a persistent docked **`Panel`** (tree of FS + symbols + search) and a transient go-to-symbol **`Picker`** modal. |
| `title_bar` / `platform_title_bar` | The app titlebar view (app menu, project host/branch, collaborators) built on a reusable platform window-control shell (traffic lights / drag region). |
| `ui` / `component` | The ~50-widget component library (`RenderOnce`-based widgets + a semantic design-token system) and a self-documenting component **registry/gallery**. |

---

## The one idiom that runs through everything: open trait + object-safe `…Handle`

Before the specifics: Zed's shell has a single recurring pattern, and recognizing it collapses the whole
subsystem into one idea. Every "slot" in the shell is an **open trait** (a concrete view implements it) paired
with an **object-safe erasure handle** (a `Box<dyn …Handle>` the shell actually stores), usually blanket-impl'd
for `Entity<T>`:

| Slot | Concrete trait (view implements) | Object-safe handle (shell stores) | Where stored |
|------|----------------------------------|-----------------------------------|--------------|
| Pane content | `Item: Focusable + EventEmitter + Render` | `Box<dyn ItemHandle>` | `Pane.items: Vec<Box<dyn ItemHandle>>` |
| Dock panel | `Panel: Focusable + EventEmitter<PanelEvent> + Render` | `Box<dyn PanelHandle>` | `Dock.panel_entries` |
| Status-bar cell | `StatusItemView: Render` | `Box<dyn StatusItemViewHandle>` | `StatusBar.{left,right}_items` |
| Fuzzy overlay | `PickerDelegate` | *(generic `Picker<D>`, not erased)* | `Entity<Picker<D>>` |
| Modal overlay | `ModalView: ManagedView` | `Entity<dyn …>` via `toggle_modal` | `ModalLayer` |
| Collab-follow | `FollowableItem: Item` | `Box<dyn FollowableItemHandle>` | registry |
| Session persist | `SerializableItem: Item` | `Box<dyn SerializableItemHandle>` | registry |
| Buffer search | `SearchableItem: Item` | `Box<dyn SearchableItemHandle>` | per-item |

The payoff: **new capabilities are added by implementing a trait for a new view, never by editing a central
enum.** A new editor kind, a new panel, a new palette, a new status cell — each is a new `impl`, discovered
through a handle or a registry. This is the exact inverse of Marley's shell, which adds capabilities by adding an
enum variant and updating every `match` over it. Hold this table in mind; the Marley comparisons all reduce to it.

---

## 1. The workspace / pane / item model `[Zed-derived]`

### 1.1 `Workspace` — the top-level container

`crates/workspace/src/workspace.rs` — `struct Workspace` is the root view of one project window. The load-bearing
fields:

- `center: PaneGroup` — the recursive split tree of editor panes (the "editor area").
- `left_dock`, `bottom_dock`, `right_dock: Entity<Dock>` — **three** docks, one per non-top edge.
- `panes: Vec<Entity<Pane>>`, `active_pane: Entity<Pane>`, `panes_by_item: HashMap<EntityId, WeakEntity<Pane>>`
  — the flat pane registry + the item→pane reverse index.
- `status_bar: Entity<StatusBar>`, `modal_layer: Entity<ModalLayer>`, `toast_layer`, `titlebar_item: Option<AnyView>`,
  `notifications` — the peripheral chrome.
- `project: Entity<Project>` — the filesystem/LSP/worktree model (a separate subsystem).
- `multi_workspace: Option<WeakEntity<MultiWorkspace>>` + `active_workspace_id: Option<Rc<Cell<EntityId>>>` — the
  hook that lets **many** `Workspace`s share one OS window (§1.7).
- Plus follower state (collab), serialization tasks, `terminal_provider`/`debugger_provider` seams,
  `centered_layout`, bounds.

The Workspace is assembled from parts it doesn't own the types of: docks host `dyn PanelHandle`, panes host
`dyn ItemHandle`, the status bar hosts `dyn StatusItemViewHandle`, the titlebar is an opaque `AnyView`. The
Workspace is a **composition root**, not a monolith.

### 1.2 The split algebra — `PaneGroup` / `Member` / `PaneAxis`

`crates/workspace/src/pane_group.rs`:

- `struct PaneGroup { root: Member, is_center: bool }`.
- `enum Member { Axis(PaneAxis), Pane(Entity<Pane>) }` — a recursive tree; a leaf is one pane, an internal node
  is a split.
- `struct PaneAxis { axis: gpui::Axis /* Horizontal|Vertical */, members: Vec<Member>, flexes: Arc<Mutex<Vec<f32>>>,
  bounding_boxes: Arc<Mutex<Vec<Option<Bounds>>>> }` — an **n-ary** split with per-child flex ratios and cached
  rects (for hit-testing during drag-resize).
- Operations mutate the tree and re-`mark_positions`: `split(old, new, SplitDirection)`, `remove(pane)`
  (collapses a one-child axis back to a plain pane), `resize(pane, axis, amount, bounds)`, `move_to_border`
  (vim `ctrl-w H/J/K/L`), `invert_axies`, `bounding_box_for_pane`, `pane_at_pixel_position`.

`SplitDirection::{Up, Down, Left, Right}` picks the axis and insertion side. A separate `pane_group::element`
module renders the tree with draggable handles (`HANDLE_HITBOX_SIZE`).

### 1.3 `Pane` — the tab host

`crates/workspace/src/pane.rs` — `struct Pane` is one tabbed region. Key state:

- `items: Vec<Box<dyn ItemHandle>>` — **the peerage**: a heterogeneous list of type-erased items (an editor, a
  terminal, a search view, a diagnostics view — all the same `Box<dyn ItemHandle>`).
- `active_item_index`, `activation_history: Vec<ActivationHistoryEntry>` (MRU for ⌃Tab), `preview_item_id`
  (the single italic "preview" tab), `pinned_tab_count`, `nav_history: NavHistory` (per-pane back/forward),
  `toolbar: Entity<Toolbar>`.
- Behavior injected as closures (not subtypes): `should_display_tab_bar`, `render_tab_bar_buttons`,
  `can_drop_predicate`, `can_split_predicate`, `double_click_dispatch_action`. This is how the same `Pane` type
  serves the center area *and* a dock's contents with different chrome.

The Pane never knows what an item *is*. It calls trait methods (`tab_content`, `is_dirty`, `save`, `navigate`) on
`dyn ItemHandle`. That indirection is the whole point.

### 1.4 The `Item` trait + `ItemHandle` erasure — the pane peerage (the crux)

`crates/workspace/src/item.rs`. This is the single most important abstraction for the Marley comparison.

**The concrete trait** — a view becomes pane-hostable by implementing:

```
trait Item: Focusable + EventEmitter<Self::Event> + Render + Sized {
    type Event;
    fn tab_content_text(&self, detail, cx) -> SharedString;   // required
    fn tab_content(&self, params, window, cx) -> AnyElement;   // default: a Label
    fn tab_icon(&self, …) -> Option<Icon>;
    fn tab_tooltip_text / tab_tooltip_content(…);
    fn is_dirty / has_conflict / has_deleted_file(&self, cx) -> bool;
    fn can_save / save / save_as / reload(…);
    fn can_split / clone_on_split(…);                          // split duplicates the item
    fn navigate(&mut self, data, …) -> bool;                  // back/forward restores this data
    fn breadcrumbs / breadcrumb_location(…);
    fn added_to_workspace / deactivated / on_removed(…);
    fn act_as_type(&self, TypeId, …) -> Option<AnyEntity>;    // "am I / do I contain an X?"
    fn as_searchable(…) -> Option<Box<dyn SearchableItemHandle>>;
    fn handle_drop / tab_extra_context_menu_actions(…);
    // ~40 methods, almost all defaulted
}
```

Nearly every method has a default, so a minimal item implements ~2 methods (`type Event`, `tab_content_text`) plus
the `Render`/`Focusable`/`EventEmitter` supertraits it already has as a gpui view. That low floor is why editor,
terminal, diagnostics, search results, the git panel's diff view, the settings editor, image/markdown/svg previews
— *dozens* of unrelated views — are all first-class pane items.

**The object-safe erasure** — `trait ItemHandle: 'static + Send` restates the whole surface in `&dyn`-friendly
form (~60 methods) and is **blanket-implemented once**: `impl<T: Item> ItemHandle for Entity<T>`. So any
`Entity<MyView>` where `MyView: Item` *is* a `Box<dyn ItemHandle>` for free. `WeakItemHandle` mirrors it for weak
refs. `ItemHandle::act_as_type` + `to_searchable_item_handle`/`to_followable_item_handle`/`to_serializable_item_handle`
are the runtime downcasts that recover typed behavior from the erased handle.

### 1.5 Item facet traits — one item, many cross-cutting capabilities

Rather than one giant `Item`, Zed layers **optional facet traits**, each `: Item` with its own `…Handle` erasure.
An item opts into a capability by implementing the facet; the shell discovers it via a downcast:

- `trait ProjectItem: Item { type Item: project::ProjectItem; fn for_project_item(project, pane, item, …) -> Self }`
  — the **file→view builder**. This is the registry that turns "open path X" into "construct the right pane item":
  the editor registers `Editor: ProjectItem<Item = Buffer>`, the image viewer registers for image files, etc.
  `workspace.open_path` walks these registrations.
- `trait FollowableItem: Item` — real-time collaborative "follow": `to_state_proto` / `from_state_proto` /
  `add_event_to_update_proto` sync the item's view state to a teammate over `proto`.
- `trait SearchableItem: Item` → `SearchableItemHandle` — project/buffer search integration (`searchable.rs`).
- `trait SerializableItem: Item` → `SerializableItemHandle` — session persistence: `serialized_item_kind`,
  `serialize`/`deserialize`, `cleanup`. A `SerializableItemRegistry` maps the kind string back to a deserializer at
  boot, so a restored session rebuilds the right item views.

The lesson for Marley: **capabilities are composable via traits, not baked into a variant.** "Can this pane
content be persisted / searched / followed?" is answered by "does it implement the facet?", uniformly, for any
item type — present and future.

### 1.6 `Dock` + `Panel` + `PanelHandle`

`crates/workspace/src/dock.rs`:

- `enum DockPosition { Left, Bottom, Right }`.
- `struct Dock { position, panel_entries: Vec<PanelEntry>, active_panel_index: Option<usize>, is_open, zoom_layer_open,
  modal_layer, … }` — a dock is a **stack of panels** sharing one edge, showing one at a time, with its own toggle
  buttons in the status bar.
- `trait Panel: Focusable + EventEmitter<PanelEvent> + Render + Sized` — the same open-trait shape as `Item`:
  `persistent_name` / `panel_key` (serialization keys), `position`/`set_position`/`position_is_valid` (a panel can
  live in any valid dock and be dragged between them), `default_size`/`min_size`, `icon`/`icon_tooltip`/`icon_label`
  (the status-bar button), `toggle_action`, `activation_priority`, `is_zoomed`/`set_zoomed`, `starts_open`, and an
  optional `pane()` (a panel may itself embed a `Pane`, e.g. the terminal panel). `PanelEvent::{ZoomIn, ZoomOut,
  Activate, Close}`.
- `trait PanelHandle: Send + Sync` — the object-safe erasure the `Dock` stores; blanket-impl'd for `Entity<T: Panel>`.

The project panel (file tree), terminal panel, git panel, agent panel, outline panel, debugger panel — all are
just `Panel` impls, and Zed's docks host them interchangeably.

### 1.7 `MultiWorkspace` — many workspaces per window + a switcher sidebar `[Zed-derived]` **(direct match to Marley's current direction)**

`crates/workspace/src/multi_workspace.rs`. This is the most Marley-relevant find in the whole subsystem — it is
*exactly* the IDE/PhpStorm-style model Marley is now moving toward (chad's 2026-07-10 workspace-centric vision;
Marley's M13 "Workspace Cockpit" and M14).

- `struct MultiWorkspace { window_id, retained_workspaces: Vec<Entity<Workspace>>, project_groups: Vec<ProjectGroupState>,
  active_workspace: Entity<Workspace>, active_workspace_id: Rc<Cell<EntityId>>, sidebar: Option<Box<dyn SidebarHandle>>,
  sidebar_open, sidebar_overlay: Option<AnyView>, … }`.
- One OS window hosts **multiple full `Workspace`s** (one per project group); only one is presented at a time.
  The shared `active_workspace_id` cell lets each member `Workspace` cheaply check whether it currently owns the OS
  window's title/edited indicator — without leasing its parent (a deliberate double-lease-avoidance note in the
  source).
- The `sidebar` (the `sidebar` crate, `struct Sidebar`, plus a `thread_switcher`) is the **workspace/project
  switcher rail** — and it is itself abstracted behind `dyn SidebarHandle`.
- Actions: `ToggleWorkspaceSidebar`, `FocusWorkspaceSidebar`, `NextProject`/`PreviousProject`,
  `NextThread`/`PreviousThread`, `NewThread`, and **`MoveProjectToNewWindow`** (drag a project out into its own
  OS window).

Marley's rail (`RailLevel::{Project, Tab, Pane}`), workspace switcher (`SwitcherRow`/`workspace_switcher_rows`),
launcher/empty state, and "highlight the active workspace" requirement map onto this almost one-to-one. **Zed's
`MultiWorkspace` + `sidebar` is the reference implementation for where Marley is already heading** — study it
before the multi-workspace re-architecture, not after.

### 1.8 The peripheral trait-erasure hosts — `ModalLayer`, `StatusBar`, `Toolbar`, `TitleBar`

- **`ModalLayer` / `ModalView`** (`modal_layer.rs`): `trait ModalView: ManagedView { fn on_before_dismiss(…) ->
  DismissDecision }`. `ModalLayer::toggle_modal(build_view)` mounts one modal at a time, centered, dismiss-on-blur.
  **This is the host for every picker** (§2): the command palette, file finder, and go-to-symbol are all
  `ModalView`s wrapping a `Picker<D>`.
- **`StatusBar` / `StatusItemView`** (`status_bar.rs`): `trait StatusItemView: Render`; `StatusBar { left_items,
  right_items: Vec<Box<dyn StatusItemViewHandle>> }` with `add_left_item`/`add_right_item`. Diagnostics summary,
  cursor position, language selector, panel toggle buttons — all status items.
- **`Toolbar`** (`toolbar.rs`): a per-pane toolbar hosting `ToolbarItemView`s (breadcrumbs, buffer search bar,
  quick-action bar) positioned by `ToolbarItemLocation`.
- **`TitleBar`** (`title_bar/src/title_bar.rs`): an app-level view showing the application menu (macOS collapses
  it; Linux/Windows render it inline), project host / project name / git branch, the collaborator facepile, and
  window controls — built on **`platform_title_bar::PlatformTitleBar`**, the reusable, `PlatformStyle`-aware shell
  that draws the traffic-lights / min-max-close controls and the draggable region.

### ► Marley comparison — pane / item / dock / workspace `[Marley-original]`

Marley's shell (subagent-verified against `crates/marley_app/src/`) is the **structural inverse**: closed enums +
a central router, essentially zero traits (the only `dyn` in the whole UI stack is three `Box<dyn Fn>` widget
callbacks in `marley_ui_components`).

| Concern | Zed | Marley (current) | Verdict |
|---------|-----|------------------|---------|
| **Pane content** | open `Item` trait; `Pane.items: Vec<Box<dyn ItemHandle>>` | **closed** `enum PaneContent<S> { Terminal(TerminalPane<S>), FileTree, CodeView(CodeViewState), Git }` (`workspace.rs:289`) + derived `PaneKind`; `PaneState<S>{content}` | **GAP** (the big one) |
| **Split algebra** | `PaneGroup{root: Member}`, `Member::{Pane, Axis(PaneAxis{axis, members: Vec, flexes})}` — n-ary + flex | `enum PaneGroup { Leaf(PaneId), Split{axis, children: Vec<PaneGroup>, ratios: Vec<f32>} }` (`layout.rs`) — **binary-in-a-Vec** + ratios; `PaneGrid<S>` wraps group + `HashMap<PaneId, PaneState>` | **MATCH** (Marley slightly less general: binary vs n-ary, but functionally equivalent and `resize_split` already takes n ratios) |
| **Docks** | 3 edges (`Left/Bottom/Right`), each a **stack of `Panel`s**, drag-between, zoom | `docks: [DockState; 2]`, `DockSide::{Left, Right}` (right **retired**), no bottom; left hosts the rail; no `Panel` trait, no multi-panel stacking | **PARTIAL** — concept matches, generality is thinner |
| **Dock contents** | any `dyn PanelHandle` (file tree, git, terminal, agent, outline…) | file tree = a `PaneContent` variant; cockpit sections = `enum RightSection { Details, Agents, Forge }` rendered as **top tabs** (`TabContent::Cockpit`), not dock panels | **GAP** — no uniform panel citizenship |
| **Workspace hierarchy** | `Workspace`(one project) + `MultiWorkspace`(many per window) + `sidebar` | `Workspace<S> → Project<S> → Tab<S> → TabContent<S>` (a *richer* built-in hierarchy) + switcher + launcher | **MATCH / AHEAD** in shape; Zed's `MultiWorkspace` is the reference for the multi-window step Marley is taking |
| **Item facets** (persist/search/follow) | composable traits (`SerializableItem`, `SearchableItem`, `FollowableItem`) | wired per-variant (grid codec for persistence #163/#205, search bespoke) | **GAP** — works, but not uniform/composable |

**Is Marley's pane model as general as Zed's? No.** Adding a pane kind in Zed is `impl Item for NewView` + one
registration; adding one in Marley means editing `PaneContent`, its `PaneKind` discriminant, and every `match` over
them (`pane_title`/`pane_icon`/`pane_display_name`/`terminal()`/`code_view()`/render/persist/…). With four
variants that is cheap and the closed enum buys real things Marley values: pure, exhaustively-matched decision
functions that hit cov/MSI 100, and no `dyn`-dispatch. **The enum is a good default for a small, terminal-first
variant set.** It stops being a good default the moment the variant count grows — which is precisely what the M13
editor-as-peer and the embedded-browser (CDP) intakes will do.

---

## 2. The `Picker<D>` abstraction `[Zed-derived]` — the highest-ROI thing for Marley to adopt

`crates/picker/src/picker.rs`. Zed has **one** generic modal-list host and **N** tiny delegates. Marley has **N**
hand-rolled overlays and **zero** shared host — this is the clearest, cheapest win in the whole comparison.

- `struct Picker<D: PickerDelegate>` owns everything generic: the query editor (`Head`, an `ui_input::ErasedEditor`),
  the results list (`ElementContainer::{List(ListState), UniformList(UniformListScrollHandle)}`), selection
  movement, scrollbar, an optional `Preview` pane (right/below/hidden, "telescope" sizing), a footer/actions menu,
  and its `Presentation::{Modal { resizable }, Popover, Embedded}` chrome. Size persists (`shape`/`persistence.rs`).
- `trait PickerDelegate: Sized + 'static` is the entire per-feature contract — everything else is free:

```
trait PickerDelegate {
    type ListItem: IntoElement;
    fn name() -> &'static str;                    // serialization key
    fn match_count(&self) -> usize;
    fn selected_index(&self) -> usize;
    fn set_selected_index(&mut self, ix, …);
    fn placeholder_text(&self, …) -> Arc<str>;
    fn update_matches(&mut self, query: String, …) -> Task<()>;   // the async fuzzy search
    fn confirm(&mut self, secondary: bool, …);                    // Enter / Cmd-Enter
    fn dismissed(&mut self, …);
    fn render_match(&self, ix, selected, …) -> Option<Self::ListItem>;
    // optional: preview, footer, header, actions_menu, select_history,
    //           confirm_input, separators_after_indices, editor_position, …
}
```

A delegate provides *data + behavior*; the `Picker` provides *all UI + interaction*. The command palette, file
finder, go-to-symbol, theme selector, recent-projects switcher, tab switcher, language selector, toolchain
selector, and more are each ~one delegate. New overlays are nearly free.

### ► Marley comparison — the overlay layer `[Marley-original]`

Marley has **two near-duplicate state structs** and **one boolean per overlay**, each with hand-written render +
key-routing in `app.rs`:

- `struct PaletteState { query, selected }` (`palette.rs`) — palette only.
- `struct FinderState { query, selected }` (`finder.rs`) — a near-copy of `PaletteState` (same
  push/backspace/move_up/move_down/reset-on-edit), shared by the ⌘P finder **and** the ⌘R history overlay.
- Booleans on `RootView`: `palette_open`, `finder_open`, `finder_split`, `history_open`, `forge_open`,
  `fleet_open`, `files_open`, `workspace_switcher_open`, `find_open` — each opened/rendered/routed by hand.

This works and is well-tested, but it is exactly the duplication `Picker<D>` exists to remove. **Verdict: GAP,
and the best first adoption.** A Marley `Picker<Delegate>` (host owning query/list/selection/scroll/chrome; a
`PickerDelegate`-style trait owning `update_matches`/`confirm`/`render_match`) would collapse
`PaletteState`+`FinderState`+history+switcher into one host + a handful of delegates, make every future overlay a
few lines, and keep Marley's discipline — the delegates can be thin wrappers over the *already-pure*
`marley_search_core` fuzzy layer (`fuzzy_score`/`fuzzy_rank`) and Marley's existing decision fns, so cov/MSI 100
survives. Low risk, immediate cleanup, no dependency on the bigger pane re-architecture.

---

## 3. The command palette `[Zed-derived]`

`crates/command_palette/src/command_palette.rs`. `CommandPalette` is a `ModalView` wrapping
`Picker<CommandPaletteDelegate>`. Its power comes from **not maintaining a command list at all** — it reads gpui's
action registry live.

1. **Enumerate**: `CommandPalette::new` calls `window.available_actions(cx)` — every `Action` registered in the
   gpui dispatch tree *at the current focus*. Each is filtered through `CommandPaletteFilter::is_hidden` (§3.1),
   then wrapped as `Command { name: humanize_action_name(action.name()), action: Box<dyn Action> }`.
2. **Humanize**: `humanize_action_name` turns `editor::GoToDefinition` → `"editor: go to definition"` (CamelCase
   split, acronym-aware: `OpenURL` → `open URL`). `normalize_action_query` maps `_`/`::` so keymap-style queries
   still match.
3. **Rank**: `update_matches` pre-sorts by `(Reverse(hit_counts), name)` — frequently-used commands and
   alphabetical order break ties — then fuzzy-matches on a background thread via
   `fuzzy_nucleo::match_strings_async(Case::Smart, …)`. `finalize_update_matches` can briefly block so ⌘⇧P paints
   with results instead of an empty flash.
4. **Display keybinding**: `render_match` renders a `HighlightedLabel` (name + match positions) plus
   `ui::KeyBinding::for_action_in(&*command.action, &self.previous_focus_handle, cx)` — resolved as the binding
   *would apply at the original focus*, so the palette shows the real shortcut.
5. **Dispatch**: `toggle` captured `window.focused(cx)` as `previous_focus_handle` before opening; `confirm`
   restores it (`window.focus(&self.previous_focus_handle)`) then `window.dispatch_action(action, cx)` — the
   action fires exactly as if invoked in the editor, not the palette. History/recency persist to a sqlite
   `CommandPaletteDB` (`hit_counts`, `QueryHistory` with prefix-aware up/down).

### 3.1 `command_palette_hooks` — the two globals that make it extensible

`crates/command_palette_hooks/src/command_palette_hooks.rs` (read in full):

- **`CommandPaletteFilter`** (a `GlobalCommandPaletteFilter`): three sets — `hidden_namespaces`,
  `hidden_action_types`, `shown_action_types` (an explicit show always wins). `is_hidden(action)` checks the
  `"namespace::"` prefix or the action `TypeId`. Features toggle their own actions in/out as they enable/disable via
  `CommandPaletteFilter::update_global` (vim, copilot, agent, project panel, extensions all do this). This is how
  the palette stays context-appropriate without the palette knowing about any feature.
- **`CommandPaletteInterceptor`** (a `GlobalCommandPaletteInterceptor(Rc<dyn Fn(&str, WeakEntity<Workspace>, &mut App)
  -> Task<CommandInterceptResult>>)`): lets a feature *rewrite* the query into synthetic results.
  `CommandInterceptResult { results: Vec<CommandInterceptItem>, exclusive }` — merged ahead of fuzzy matches, and
  if `exclusive`, replaces them. This is how vim turns `:w`/`:42` (Ex commands) into actions, and how `zed:` deep
  links resolve — all without touching the palette core.

### ► Marley comparison — commands `[Marley-original]`

| Aspect | Zed | Marley |
|--------|-----|--------|
| Source of truth | gpui **action registry** (`available_actions`), zero maintenance | hand-maintained `cockpit_commands() -> Vec<Command>` (`app.rs:3955`); `Command { id: CommandId(u32), title, keywords, binding }` |
| Dynamic entries | any feature's registered actions appear automatically | appended by hand: `CONNECT_BASE`/`THEME_BASE`/`WORKFLOW_BASE` ranges resolved by `dynamic_command_index` |
| Fuzzy | `fuzzy_nucleo::match_strings_async` | `filter_commands` over `marley_search_core::fuzzy_score` (also nucleo) — **UX matches** |
| Dispatch | re-dispatch the `Box<dyn Action>` on the saved focus | two-hop **string**: `action_for_command(id) -> &'static str` → the same `dispatch_action(&str)` ~360-line `match` the chords use |
| Keybinding shown | `KeyBinding::for_action_in` resolves live | `binding: Option<KeyBinding>` carried on the `Command` row |
| Extensibility | `CommandPaletteFilter` + `Interceptor` globals | edit `cockpit_commands` + the central `match` |

Marley's palette is a bespoke, statically-enumerated list with a string-router dispatch. It is simpler and works,
but every command must be hand-registered and hand-routed. **Verdict: fuzzy/UX MATCH; source-of-truth + dispatch
GAP.** The realistic Marley move is *not* to adopt gpui's full action registry (Marley deliberately uses a
closed action-string space with a `match` router — clean, testable, cov/MSI 100). It is to (a) fold the palette
into the shared `Picker<Delegate>` from §2, and (b) *if/when* the command set sprawls, add a filter/interceptor
seam (a `Fn(&str) -> Vec<Command>` hook) so features/plugins can contribute commands without editing
`cockpit_commands`. The keybinding-display and recency-bias ideas are cheap borrows worth taking now.

---

## 4. File finder + outline `[Zed-derived]`

### 4.1 File finder — `Picker<FileFinderDelegate>`

`crates/file_finder/src/file_finder.rs`. A `Picker` with an editor **preview** pane. The delegate's design worth
stealing:

- `enum Match { History { path, panel_match }, Search(ProjectPanelOrdMatch), Channel {…}, CreateNew(ProjectPath) }`
  — history, live search, collab-channel files, and a "create this file" affordance are **one unified result
  list**, sorted by a custom `ProjectPanelOrdMatch` `Ord` (score → worktree → ancestor distance → reversed path)
  so ordering matches the file tree and the currently-open file bubbles up.
- Search: `spawn_search` builds a `PathMatchCandidateSet` per visible worktree and runs
  `fuzzy_nucleo::match_path_sets(…, Case::Ignore, …)` on the background executor with an `AtomicBool` cancel flag +
  a `search_id` generation guard (each keystroke cancels the last). History seeds from
  `workspace.recent_navigation_history(Some(20))`; `Matches::push_new_matches` unions history (matched by
  *filename*) with search, dedups by `ProjectPath`, caps at 100, debounced 100 ms.
- Query grammar: `parse_file_search_query` understands `path:row:col` and `path:start-end` line ranges, strips
  `./`/`a/`/`b/` diff prefixes; after opening it jumps via `editor.go_to_singleton_buffer_range`.
- Open: `open_selected_file` → `workspace.open_path_preview` (normal) or `split_path_preview` (⌘-Enter /
  directional `pane::SplitLeft/Right/Up/Down`); preview-tab behavior gated by `PreviewTabsSettings`.

### 4.2 Outline — a `Panel` and a `Picker` over one `language::Outline`

Two surfaces, one data model — a clean illustration of the trait/handle payoff:

- **`language::Outline<T>`** (`language/src/outline.rs`): `{ items: Vec<OutlineItem<T>>, candidates:
  Vec<StringMatchCandidate>, leaf_offsets }`. Each candidate string is the **full ancestor path** (space-joined
  parent names) with `leaf_offsets` marking where the leaf begins — so fuzzy search scores against the qualified
  symbol path while callers still render just the leaf name. `Outline::search` → `fuzzy_nucleo`, returning real
  matches plus synthetic `Ancestor` rows for parent context.
- **`outline::OutlineView`** (`crates/outline/`): the ⌘⇧O **go-to-symbol modal** — a `ModalView` wrapping
  `Picker<OutlineViewDelegate>`. Empty query pre-selects the deepest symbol containing the cursor; selection
  live-previews via `highlight_rows` + autoscroll; `on_before_dismiss` restores the pre-open scroll. Sourced from
  `editor.buffer_outline_items` (tree-sitter or LSP `document_symbols`).
- **`outline_panel::OutlinePanel`** (`crates/outline_panel/`): the persistent docked **`Panel`** — a flattened,
  cached tree (`cached_entries: Vec<CachedEntry>`, `PanelEntry::{Fs, FoldedDirs, Outline, Search}`) that follows the
  active editor via a **workspace subscription** (`active_pane` change → `replace_active_editor` → wire
  editor/search subscriptions → `update_fs_entries`), fetches symbols per buffer with `buffer_outline_items`,
  supports expand/collapse (`collapsed_entries: HashSet<CollapsedEntry>`, auto-folding single-child dir chains), a
  `pinned` mode, and its own single-line `filter_editor` (fuzzy via the older `fuzzy` crate).

### ► Marley comparison — finder & outline `[Marley-original]`

- **File finder**: Marley has ⌘P (`FinderState` + `marley_search_core::fuzzy_rank`, `chosen()` returns the path)
  and split-to-file mode (`finder_split`, #246). It **matches** Zed on the core UX. Gaps vs Zed's delegate:
  the unified `Match` list (history + search + create-new in one ranked list), the `path:row:col` grammar, and the
  live preview pane. These are worthwhile *delegate features* once the finder is a `Picker` delegate (§2) — they
  come almost for free from the host's preview support.
- **Outline**: Marley has **no** outline panel or go-to-symbol yet (its `CodeView`/`EditorSurface` is young). When
  it arrives, Zed's "one `language::Outline` data model, two surfaces (a `Picker` modal + a `Panel` tree)" split is
  the pattern to copy — and it presupposes the two adoptions this doc recommends (a `Picker` for the modal, a
  `Panel` trait for the docked tree). **Verdict: finder MATCH; outline is a future GAP that the §2/§1.6 adoptions
  unlock.**

---

## 5. The `ui` component library `[Zed-derived]` (+ `[gpui Apache-2.0]` primitives)

`crates/ui/` + `crates/component/`. Marley already has this concept (`marley_ui_components`, the M1.B "5 cockpit
widgets"); Zed's is the mature end-state (~50 widgets + a design-token system + a self-documenting gallery).

**The component model.** Two flavors, both gpui-native:

- **Stateless widgets (the ~56 majority)**: a plain struct with `#[derive(IntoElement)]` + a hand-written
  `impl RenderOnce` (`render(self, window, cx) -> impl IntoElement`). Builder pattern: `Type::new(id, …)` then
  consuming `fn opt(mut self, …) -> Self`. **Composition, not inheritance**: `Button → ButtonLike → gpui div()`
  (each wraps the lower layer and forwards builder calls). `Tab`, `TabBar`, `List`, `Modal`, `KeyBinding`, `Label`
  work the same way.
- **Stateful widgets (only 2: `ContextMenu`, `Tooltip`)**: real gpui entities (`impl Render + Focusable +
  EventEmitter<DismissEvent>`) held as `Entity<…>` because they carry interaction state across frames.

**The styling trait mixins** — small single-purpose `self -> Self` traits a widget opts into, giving every widget a
uniform builder vocabulary: `Clickable` (`on_click`), `Disableable` (`disabled`), `Toggleable` (`toggle_state`, +
the tri-state `ToggleState`), `FixedWidth`, `VisibleOnHover`, `Transformable`, `SelectableButton`, `ButtonCommon`.
On top of gpui's `Styled` (the tailwind-ish `.bg()/.px()/.rounded_lg()/.flex_col()` chains) Zed blanket-adds
`StyledExt` (`h_flex`/`v_flex`, `elevation_1/2/3(cx)` surface helpers, `border_primary/muted`), `StyledTypography`
(`text_ui`/`text_ui_sm`/`font_buffer`), and `CommonAnimationExt` (spinner rotations).

**The semantic design-token system** (the real value-add over gpui) — components never hard-code color/size:

- `Color` — a semantic enum (`Default`, `Muted`, `Accent`, `Error`, `Success`, VCS `Created/Modified/Deleted`,
  `Player(u32)`, `Custom(Hsla)`) resolved through `cx.theme().colors()` / `.status()` (the `theme::ActiveTheme`
  extension on `App`).
- `TextSize`/`HeadlineSize` — a rems-based type scale that tracks the user's UI scale.
- `DynamicSpacing` — density-aware spacing (`Base04.rems(cx)`), three values per token for `UiDensity::{Compact,
  Default, Comfortable}`, generated by a `derive_dynamic_spacing!` macro.
- `ElevationIndex` (`Background < Surface < ElevatedSurface < ModalSurface`) — the surface/shadow layering model
  (`.shadow(cx)`/`.bg(cx)`), surfaced ergonomically via `StyledExt::elevation_*`.

**The `component` crate — a self-documenting gallery** (orthogonal to rendering): `trait Component` (associated
`id`/`scope`/`status`/`name`/`description`/`preview`, no `self`), a `#[derive(RegisterComponent)]` that
auto-registers each widget into a global registry via the `inventory` linker-collected slice (no central list), a
`ComponentScope`/`ComponentStatus` taxonomy, and an example DSL (`single_example`/`example_group`). "Open component
preview" renders the live gallery — docs and widgets never drift because `description()` reads the doc comment
(`documented` crate).

### ► Marley comparison — components `[Marley-original]`

Marley's `marley_ui_components` (button, switch, dialog, tooltip, shortcut per the M1.B seq) already uses the same
**gpui-native, builder-pattern, composition-over-inheritance** shape (the three `Box<dyn Fn>` callbacks are the
one bit of dynamism). So the *model* matches. Gaps are maturity, not architecture:

- **Semantic design tokens** — Marley themes via its settings/theme system (#199 live theme picker); adopting a
  `Color`-style semantic enum + a `DynamicSpacing`/`ElevationIndex`-style token set would make new widgets
  theme-correct by construction. Worth borrowing incrementally as the widget set grows.
- **The `component` gallery** — Marley has no self-documenting widget registry. A nice-to-have (dev tooling) once
  the widget count justifies it; the `inventory`-based auto-registration is the clean way to build it.
- **Widget breadth** — Marley has ~5; Zed ~50. Grow on demand; the trait-mixin vocabulary (`Clickable`/`Disableable`
  /`Toggleable`) is a cheap, high-leverage thing to standardize early so every Marley widget shares one builder
  surface.

**Verdict: model MATCH; adopt the semantic token system incrementally, the gallery eventually.**

---

## Data / control flow (one interaction: ⌘⇧P → run a command)

```
key chord ⌘⇧P ─► gpui keymap ─► action Toggle(command_palette)
   │                                   │
   │  Zed: Workspace.modal_layer.toggle_modal(CommandPalette::new)
   │        └─ CommandPalette wraps Picker<CommandPaletteDelegate>
   │             ├─ delegate.update_matches(query):
   │             │     available_actions(cx) ─► filter.is_hidden ─► humanize
   │             │     ─► fuzzy_nucleo::match_strings_async (bg thread)
   │             │     ─► (+ interceptor results, if any)
   │             ├─ Picker renders head(query editor) + uniform_list(render_match)
   │             │     each row = HighlightedLabel + KeyBinding::for_action_in
   │             └─ confirm(): window.focus(previous_focus_handle)
   │                            + window.dispatch_action(action)  ─► the real action fires
   ▼
(Marley today: ⌘⇧P ─► keymap.action_for ─► "open-command-palette"
   ─► RootView.dispatch_action(&str) match ─► self.palette_open = true; PaletteState::new()
   ─► hand-written overlay render + key routing; activate ─► action_for_command(id)
   ─► "new-terminal" ─► the SAME dispatch_action(&str) match)
```

The shapes rhyme; the difference is *who owns the list UI* (a reusable `Picker` vs a hand-written overlay) and
*where commands come from* (a live registry vs a static `Vec`).

---

## Marley relevance & adoption sequencing (the punchline)

Marley's shell is genuinely strong and its closed-enum + pure-decision-fn + thin-shim discipline is a real asset
(cov/MSI 100, no `dyn` sprawl, exhaustive `match`es). **Do not "trait-ify everything" reflexively.** Adopt in ROI
order, each gated on an actual trigger:

1. **`Picker<Delegate>` — do this first (low risk, immediate win).** Collapse `PaletteState` + `FinderState` +
   the ⌘R history + the workspace switcher into one host + N delegates. Delegates wrap the already-pure
   `marley_search_core` fuzzy layer, so the discipline survives. Unblocks cheap future overlays (outline modal,
   theme/host/workflow pickers that today are dynamic command ranges). No dependency on anything else here.
   *Trigger: already met — the overlay duplication exists today.*

2. **A `PaneItem` trait to generalize `PaneContent` — the strategic one.** Introduce
   `trait PaneItem { fn title(); fn icon(); fn render(); fn is_dirty(); … }` (Marley's `Item` analog) + a boxed
   handle, and migrate `Terminal/FileTree/CodeView/Git` to impls. Keep the impls thin — each delegates to the
   existing pure fns — so cov/MSI 100 holds. This is a **re-architecture pillar**, not a quick edit (it touches
   every current `match PaneContent`). *Trigger: the M13 editor-as-peer and the embedded-browser (CDP) intakes —
   the moment the peer count goes from 4 toward 8+, the enum's per-variant `match` tax exceeds the trait's
   dispatch cost.* Sequence it *with* the editor-as-peer work, not before.

3. **A `Panel` trait for docks — when the dock gains a second hosted type.** Today the left dock hosts the rail and
   the file tree is a `PaneContent` variant; cockpit sections are top tabs. When file-tree + git + agents + outline
   all want to be dock citizens, a `Panel` trait (à la Zed's `persistent_name`/`position`/`icon`/`toggle_action`)
   makes them uniform and drag-between-able. *Trigger: ≥2 real docked panel types.*

4. **Item facet traits (`Serializable`/`Searchable`) — when persistence/search wiring sprawls.** Today Marley
   persists panes via the grid codec (#163/#205) and searches bespoke. If that logic starts branching per
   `PaneContent` variant, lift it into facet traits so any future `PaneItem` opts in uniformly. *Trigger: the
   per-variant persist/search branches start to duplicate.*

5. **Study Zed's `MultiWorkspace` + `sidebar` now — it is the reference for Marley's current direction.** Marley's
   workspace-centric vision (multi-workspace per window, a launcher, rail highlight of the active workspace) is
   exactly `MultiWorkspace { retained_workspaces, active_workspace, sidebar }` + `MoveProjectToNewWindow`. Read it
   before designing the re-architecture. *No adoption of code — architecture reference for the M13/M14 work.*

6. **Incremental UI-token borrows.** A semantic `Color` enum + `DynamicSpacing`/`ElevationIndex` tokens + the
   `Clickable`/`Disableable`/`Toggleable` builder mixins standardize new `marley_ui_components` widgets; a
   component gallery is a later dev-tooling nicety.

**Licensing note:** all of the above are `[Zed-derived]` GPL patterns reimplemented clean-room in Marley's own
code — fine for Marley's intended-GPL editor/terminal layer, and they must stay on that side of the brain
boundary.

---

## Key files (cite these)

**workspace / pane / item**
- `crates/workspace/src/workspace.rs` — `struct Workspace` (center `PaneGroup` + 3 docks + status/modal/toast/titlebar + `multi_workspace`).
- `crates/workspace/src/item.rs` — `trait Item`, `trait ItemHandle` (+ `impl<T: Item> ItemHandle for Entity<T>`), `ProjectItem`, `FollowableItem`, `SerializableItem`, `WeakItemHandle`, `SaveOptions`, `ItemEvent`.
- `crates/workspace/src/pane.rs` — `struct Pane` (`items: Vec<Box<dyn ItemHandle>>`, activation history, preview/pinned tabs, nav history, closure-injected chrome).
- `crates/workspace/src/pane_group.rs` — `PaneGroup`, `enum Member`, `PaneAxis` (n-ary + flexes), `SplitDirection`.
- `crates/workspace/src/dock.rs` — `enum DockPosition`, `struct Dock`, `trait Panel`, `trait PanelHandle`, `PanelEvent`.
- `crates/workspace/src/multi_workspace.rs` — `struct MultiWorkspace`, the workspace-switcher actions (`NextProject`/`MoveProjectToNewWindow`/…). `crates/sidebar/src/` — the switcher rail.
- `crates/workspace/src/{modal_layer,status_bar,toolbar,searchable}.rs` — `ModalView`, `StatusItemView`, `ToolbarItemView`, `SearchableItem`.

**picker / palette / finder / outline**
- `crates/picker/src/picker.rs` — `struct Picker<D>`, `trait PickerDelegate`, `Presentation`, preview/footer.
- `crates/command_palette/src/command_palette.rs` — `CommandPaletteDelegate`, `humanize_action_name`, `available_actions` + re-dispatch; `.../persistence.rs` — `CommandPaletteDB`, `QueryHistory`.
- `crates/command_palette_hooks/src/command_palette_hooks.rs` — `CommandPaletteFilter`, `GlobalCommandPaletteInterceptor`, `CommandInterceptResult`.
- `crates/file_finder/src/file_finder.rs` — `FileFinderDelegate`, `enum Match`, `ProjectPanelOrdMatch`, `parse_file_search_query`, `open_selected_file`.
- `crates/outline_panel/src/outline_panel.rs` — `OutlinePanel` (`Panel`), `PanelEntry`, `cached_entries`, active-editor subscription.
- `crates/outline/src/outline.rs` — `OutlineView` (`Picker` modal); `crates/language/src/outline.rs` — `Outline<T>`, `OutlineItem`, `Outline::search`.

**titlebar / ui**
- `crates/title_bar/src/title_bar.rs` + `crates/platform_title_bar/src/platform_title_bar.rs` — `TitleBar`, window controls.
- `crates/ui/src/{ui,prelude,traits,styles}.rs` + `crates/ui/src/components/` — the widget library, mixin traits, design tokens.
- `crates/component/src/component.rs` — `trait Component`, `RegisterComponent`, the `inventory` gallery registry.

**Marley counterparts (for the comparison)**
- `crates/marley_app/src/workspace.rs` — `enum PaneContent<S>` (the closed peerage), `PaneKind`, `PaneGrid<S>`, `TerminalPane<S>`.
- `crates/marley_app/src/{layout,tabs,palette,finder,history,keymap,titlebar,right_dock}.rs` — the split tree, `Workspace→Project→Tab` model + rail, the bespoke palette/finder/history state, the keymap + `dispatch_action` router, the titlebar, the cockpit sections.
- `crates/marley_ui_components/` — the current widget set.

## Open questions (round 2)

1. **Pane-item trait shape.** When Marley introduces `PaneItem`, does it stay generic over the session handle `S`
   (like `PaneContent<S>` today) or erase it? Zed's `Item` has no `S` — session/PTY state lives in the concrete
   view. Resolving this decides whether the trait can be object-safe (`Box<dyn PaneItem>`) or must stay generic.
2. **Picker preview.** Zed's file-finder preview pane is a real embedded `Editor`. Marley's `Picker` preview would
   need `CodeView`/`EditorSurface` to be embeddable mid-overlay — is that ready, or does the finder preview wait on
   the editor-as-peer work?
3. **Command source-of-truth.** Should Marley keep the closed action-string space (and just add a
   filter/interceptor hook), or move toward a registry? The plugin/extensibility roadmap decides this.
4. **Dock generality.** Does Marley want Zed's 3-edge, multi-panel, drag-between docks, or is the current
   left-rail + retired-right model (plus cockpit-as-top-tabs) the intended end state? The `Panel`-trait adoption
   hinges on this.
5. **MultiWorkspace mapping.** Marley's `Workspace → Project → Tab` is richer than Zed's single `Workspace`. When
   Marley goes multi-workspace-per-window, does a Marley "workspace" map to a Zed `Workspace` (and a Marley
   `Project` to a Zed project group), or does the whole `MultiWorkspace` collapse into Marley's existing
   `Workspace<S>` container? This is the core re-architecture decision for M13/M14.
</content>
