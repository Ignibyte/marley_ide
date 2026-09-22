# TabContent::Browser + Content::Browser residency + the bare `B` shell tag — Notes

- **Forge ticket:** #403 `f7bf9657-b4c1-4fd6-b346-584660daf256` (feature, M29; sprint #40
  `6dff19c6-fb61-472b-89c3-98d3215a489b`)
- **AAR:** `f394490d-c4fe-4e29-9f30-cc8e7250d848` — submitted `completed`, effectiveness 4, 5 novel findings
- **Local ticket doc:** docs/planning/tickets/open/TICKET-403-browser-tab-residency.md
- **Pipeline spec:** 403-browser-tab-residency.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. Excluded
     from gate:14 doc-todos (docs/planning/ is working scratch), so in-progress
     TODO notes here are fine. -->

## Phase 1 — Plan
- **Request:** the /spec M29 batch — the embedded-browser train's first CODE slice (#389 train step
  2) fused with the Browser half of #388 slice-8 that #400 shipped cockpit-only: `TabContent::Browser`
  id-bearing, `Content::Browser` cashed into the #394 registry lifecycle, the bare `B` shell tag, a
  placeholder pane behind the Browser section. Pure model + codec; no webview (#405), no URL (#404).
- **Classification / tier:** feature, M29, marley_app only. Pure seams (tabs.rs / status_bar.rs /
  grid_layout.rs / content.rs) at cov/MSI 100 + masked app.rs wiring. REACT-FIRST: UI-AFFECTING
  (Zone A rail rows + Zone B placeholder pane) — the enforce-react-parity.sh contract applies.
- **Forge recall (§18.3), actual results:** `knowledge-search` ("browser tab residency TabContent
  codec bare marker registry") → 6 hits, ALL `prevention_rule`, id-only payloads (top
  `7f19c7c9-e40e-4d05-b452-2825ce593498`); the named rules this spec binds
  (`PR-claude-registry-release-returning-a-resource-must-be-must-use-001`,
  `PR-claude-shared-registry-needs-refcount-for-drop-on-last-close-001`,
  `PR-claude-closed-enum-gate-exhaustive-match-not-predicate-chain-001`,
  `BF-claude-skip-detach-pump-fleet-live-001`,
  `PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`) carry over from the #400
  exemplar's lineage, which this ticket templates on. `docs-search` ("embedded browser
  TabContent::Browser bare marker B tag …") → cross-project noise only (oathstar-studio /PCB
  pipeline chunks; no Marley embedded-browser hits) — the local design docs read directly are the
  authority. No blocking bulletins surfaced in this session.
- **Discovery (verified against the tree, 2026-08-06 — design-doc line numbers HAVE drifted; these
  are today's):**
  - **`Content::Browser` is ALREADY DECLARED** — content.rs:49, a payload-less unit variant ("DECLARED
    — browser residency is gated on the #389 substrate (Phase E)"), with `ContentKind::Browser` :67,
    `kind()` arm :109, `addable() == false` :83 (comment names the #400 D-OPEN-KIND-GATING posture),
    tested at :316-323 (`addable_kinds_are_terminal_and_editor`) and :331-335
    (`payloadless_kinds_classify`). #396's "all 6 kinds declared" DID ship. So #403 adds NO enum
    variant — it cashes the declaration (D5), exactly the #400 pattern (whose cockpit cash CHANGED
    the unit to `Cockpit(RightSection)`; Browser v1 keeps the unit — nothing to own).
  - **The #400 precedent shape (the template):** `TabContent::Cockpit(ContentId, RightSection)`
    tabs.rs:32; `Tab::cockpit` :166-173; tag-pure accessors `is_cockpit`/`cockpit_section`/
    `cockpit_view`/`cockpit_content_id` :210-236; `open_or_switch_cockpit` :445-459. The pinned
    lifecycle machinery: `CockpitIndex` content.rs:234-258 (a filled slot IS the anchor),
    `resolve_or_register_cockpit` :267-280, `release_cockpit_views` :289-302 (debug-asserts the
    anchor holds + cockpit-ids-only). App wiring: boot restore arm app.rs:2241-2256
    (resolve + acquire per restored tab), `open_cockpit_tab` :3965-3979 (acquire only on append),
    serialize collect :5303-5304, close-tab release :8497-8501, close-project release :8668-8670,
    render dispatch :18455-18474 (registry resolve with the tag as total fallback), `cockpit_body`
    :5854 (masked).
  - **Codec, today:** shell entries `T=`/`C=`/`V=` — `serialize_shell` grid_layout.rs:334 (doc
    :330), `C=` writer arm :382-386, `restore_shell` :405, reader chain :427-455 with the
    unknown-tag skip at :455, proven by `shell_codec_malformed_pieces_skipped` :996-1009 (`X=?`
    skipped). Shell framing reserves `\t\n\r` (`breaks_framing` :255); the GRID leaf layer reserves
    `\t\n\r,:=\x1f` (`breaks_grid_framing` :314-316) — the #389 Q4 two-layer analysis re-verified.
    `TabLayout` :207 (Cockpit variant :222). Exact-wire fixture :716; mixed round-trip
    `shell_codec_round_trips` :680. A bare `B` slots into the writer match + the strip_prefix chain
    additively.
  - **Status bar:** `FocusTab` status_bar.rs:64-75 (Terminal/Editor/Cockpit — NO Browser),
    `focus_label` :81-105, wired at app.rs:20561-20593 — whose comment (:20557-20558) explicitly
    anticipates "a future tab kind (e.g. an embedded browser)" failing the build into the label.
    Test `focus_label_non_terminal_tabs` :158.
  - **Rail:** `RailSection::Browser` tabs.rs:51 (ALL :56, label :69, from_label :80);
    `rail_section()` :127-133 (Cockpit → Browser :131 — the transitional tenancy). `rail_rows`
    groups tabs by `rail_section()` generically (:1090-1091) so a Browser tab files with NO section
    work; the exhaustive `(row_content, pane_marked)` match :1103-1118 forces a Browser arm
    (Cockpit's shape `(None, false)` :1118).
  - **Cell layer stays out:** `PaneContent` workspace.rs:341-355 = Terminal(ContentId)/FileTree/
    CodeView/Git — no Browser arm today, none added (Out).
  - **Registry:** `ContentRegistry` content_registry.rs:49; `insert` :65 (first view),
    `acquire_view` :74-79 (`#[must_use]`), `release_view` :85-95 (`#[must_use]`, returns owned on
    exactly-last), `iter` :125; ids never serialize (:20-22).
  - **Open exposure today:** Browser＋ menu = `SECTION_BROWSER_ITEMS` context_menu.rs:158-171
    (Forge/Agents/Details, all `SectionAction::OpenCockpit`; item 0 = Forge is the pinned default,
    tests :537-538), routed `section_items` :195-202 → `dispatch_section_verb` app.rs:7269-7316
    (OpenCockpit arm :7301-7304). No palette verb opens cockpit sections (palette.rs checked). A
    Browser open needs a NEW verb — the D-OPEN-OPEN-VERB fork, recommended as a 4th ＋ row.
  - **The one-config lineage (D-OPEN-IDENTITY evidence):** `mcp_json_path` mcp_config.rs:28-40
    (active-root-first, cwd fallback, misconfigured-surfaces-downstream doc :22-24); consumed ONCE
    at boot, app.rs:2400-2410, feeding the forge client AND the brain endpoint (:2428) — the #389
    Q3 "one config, three consumers" seam the future browser URL joins. One resolution ⇒ recommend
    one app-wide instance.
  - **marley-web (the parity side):** port map MARLEY-PARITY.md:585 maps `views/BrowserView.tsx` ↔
    `right_dock.rs` (it hosts the cockpit tab strip + a POC-only 'web' tab). LeftRail.tsx Browser
    section :374-407 with an ALWAYS-present "Web" row :388-393 (`activeBrowserTab` defaults 'web' —
    "a tab Marley has no equivalent for", parity doc :187-188); state `App.tsx:117-119`
    (`activeBrowserTab`/`openBrowserTabs`/`browserUrl`). BrowserView.tsx:242-396 renders a FULL
    simulated browser (URL bar, mock marley.dev, Manager chat) — future-slice content designed
    ahead; the #403 prototype adds the pre-webview PLACEHOLDER state (recommend a new
    `views/BrowserPlaceholder.tsx`; Phase 2 confirms) and makes the rail row a real open/close tab
    row. Dev loop: `pnpm --filter @workspace/marley-ide run dev` (package `@workspace/marley-ide`,
    vite) → localhost:5173. Shared vocabulary ContentId↔PaneItem :592-599; the #400 note :624-632
    already records the cockpit-residency tie.
  - **Prior-art sweep:** zed 07 re-verified at today's lines (§1.3 :126, §1.4 :143/:170-171, §1.6
    :199, GAP rows :266/:269; Zed ships NO browser pane item; the map cites Marley's
    embedded-browser intake at :19/:279/:554). Warp swept (subsystems 00-07 + crates notes):
    browser hits are all Warp-on-Web (WASM builds — warp_web_event_bus/serve-wasm/warp_logging),
    no browser-pane behavior — no owner. Deps: gpui/ropey/regex/alacritty_terminal/tree-sitter own
    nothing here; the in-house #394/#396/#397/#400 seams + the #163/#205 codec discipline dominate.
  - **Design-doc drift recorded honestly:** embedded-browser-model.md Q4 sketches
    `TabContent::Browser(BrowserState { url, … })` — superseded by its OWN bare-marker decision +
    the #400 id-bearing precedent: v1 is `TabContent::Browser(ContentId)`, no BrowserState, no URL
    field anywhere (the URL is #404's, app-side/derived). Q4's cited lines drifted
    (serialize_shell :355→:334, entries :325→:330, breaks_grid_framing :310→:314, restore_shell
    :410→:405, skip :410→:455 area; tabs.rs:82→:51; app.rs 2207/2220→2400-2428, cockpit_body
    5451→5854); focus_label :81 is still exact. pane-composition-model.md slice-8 marks the Browser
    half "stays gated on #389" — this ticket un-gates it.
- **Decisions:** D1 variant-discriminant-is-the-tag (`TabContent::Browser(ContentId)` — no
  redundant tag field; grounded in what cockpit's tag actually bought and Browser not needing it);
  D2 bare `B` + rebuild-on-restore + ids/URLs-never-serialize; D3 additive back-compat pinned at
  the shipped unknown-tag skip; D4 placeholder only, zero platform code, no mcp_config read; D5
  cash the existing declared unit variant, add no payload. D-OPENs for Phase 2 with
  recommendations: IDENTITY one-app-wide (the one-boot config lineage), LIFECYCLE dropped-on-last-
  close (the future webview is reapable; cockpit's pin rationale doesn't transfer), OPEN-VERB a 4th
  Browser＋ row keeping item-0 default. Full grounding in the spec.

## Phase 2 — Design

### The three D-OPENs, settled against the shipped tree (2026-08-06)

**D-OPEN-IDENTITY → ONE Browser instance app-wide. SETTLED (evidence confirmed).**
`mcp_json_path(active_root, cwd, |p| p.is_file())` runs exactly ONCE at boot (app.rs:2401-2406,
AFTER the shell restore so a Finder launch still finds the active project's file), producing ONE
`mcp_json` string that feeds BOTH the forge client (:2407-2410) and #376's brain-endpoint bearer
decision. One config resolution ⇒ one derived origin ⇒ one instance; a second project's Browser tab
is a second VIEW of it (the #400 cross-project shape). Revisit trigger recorded for #404: if the URL
seam re-derives on active-root SWITCH, identity follows the evidence then.

**D-OPEN-LIFECYCLE → DROPPED on last close. SETTLED — and the cockpit's WIRING SHAPE DOES NOT
TRANSFER.** This is the load-bearing finding of Phase 2. `open_cockpit_tab` (app.rs:3965-3979)
resolves the id and then acquires a view *only when a tab was appended*. That is correct **only under
a pin**: `resolve_or_register_cockpit`'s `insert` acquires the ANCHOR view (never released), so a
tab's view is always a separate, second acquire. Under a DROPPED lifecycle there is no anchor —
`insert`'s first view *is* the first tab's view. Copying the cockpit shape would then mean: door
inserts (count 1, held by nobody) → `open_or_switch_browser` must append (no tab can exist, since a
tab implies a live view implies no insert) → returns `true` → caller acquires → **count 2 with 1 tab,
a view that is never released and content that can never drop.** A leak that exactly defeats the
lifecycle being chosen.

The correct contract is the EDITOR one — `resolve_open` / [`ResolvedOpen`] (content.rs:194-213): the
door hands back **exactly one acquired view in both branches** (hit → `acquire_view`; miss →
`insert`'s first view), and the caller RELEASES it when it didn't consume it (the switch case).
The precedent is verbatim shipped at app.rs:5594-5598 (`open_or_switch_code` → `if !newly_row {
release_editor_views(…) }`). `ResolvedOpen` is reused as-is — no new result type.

Consequences, accepted: no index, no anchor (so no `BrowserIndex` analog — the door is
find-live-or-insert scanning `kind()`, the `find_open`/`resolve_open` shape); close-then-reopen
legitimately mints a FRESH id (identity is the KIND, not the id — ids are session-local and never
serialize, content_registry.rs:20-22). Rationale for choosing dropped over pinned while the v1
payload is a unit (both are safe TODAY): the future payload is precisely a reapable resource (the
#389 train's slice-4 names webview teardown "mirror the PTY reaper contract"), and no standing
reference outlives the tabs the way the cockpit's top-bar strip + persisted `right_section` do.
Pinning now would bake in a leak-shaped contract needing reversal at #405.

**D-OPEN-OPEN-VERB → a 4th Browser＋ row, APPENDED. SETTLED.** `SECTION_BROWSER_ITEMS`
(context_menu.rs:158-171) grows from 3 to 4 rows with `("Browser", SectionAction::OpenBrowser)` at
index **3**, dispatched through the existing `section_items` → `dispatch_section_verb` route
(app.rs:7275-7313). Item 0 stays Forge — `context_menu.rs:537-538` pins it as the #387/#393 default
and changing the default gesture is not this ticket's call. Appended rather than promoted because
what #403 ships behind that row is a PLACEHOLDER; the namesake row earns promotion when #405 puts a
page behind it. No palette verb this slice (none opens cockpit sections either — checked; adding one
would be a new exposure surface, not the minimum that makes the tab reachable).

### Architecture / approach
The change is the #400 cockpit slice run again for a second kind, with the lifecycle fork inverted —
so it lands entirely inside the shipped spine (gpui app → tab model → `ContentRegistry` → shell
codec) and introduces no new mechanism. Four pure layers plus a masked shim layer:

1. **Tab citizenship** (`tabs.rs`, pure) — `TabContent::Browser(ContentId)` joins the closed enum.
   Per **D1** the variant discriminant IS the tag: cockpit carries `RightSection` because three
   sections share one variant and the codec writes a per-instance key; Browser v1 has one kind and a
   payload-less codec marker, so `matches!` gives `is_browser` and the match arm gives
   `rail_section`/`focus_label`/the `B` writer — all pure, no registry read, zero redundancy.
2. **Residency** (`content.rs`, pure) — `find_browser` / `resolve_browser` / `release_browser_views`
   over the #394 registry, per D-OPEN-LIFECYCLE above. `Content::Browser` is CASHED, not
   re-declared (**D5**): the unit variant at content.rs:49 already exists with its `kind()` arm and
   `addable() == false`; #403 adds NO variant and NO payload. `addable()` stays `false` — no cell arm
   this slice.
3. **Codec** (`grid_layout.rs`, pure) — `TabLayout::Browser` + a bare `B` shell entry (**D2**), added
   to the writer match and the reader chain. Additive by the shipped unknown-tag skip (**D3**).
4. **Placeholder copy** (`browser.rs`, NEW, pure) — the empty-state strings as data, so the render
   arm in app.rs stays pure layout and the copy is unit-pinnable at cov/MSI 100. This module is also
   the declared landing site for #404's URL derivation (which needs a pure, `mcp_config`-fed home
   that is neither the tab model nor the footer) — seeded here with only what #403 uses.
5. **App wiring** (`app.rs`, `context_menu.rs`) — open verb, restore arm, serialize collect, both
   close-path releases, the FocusTab arm, and the placeholder render dispatch. All masked
   (`app.rs` is coverage-excluded; the shims carry `#[cfg_attr(test, mutants::skip)]` per their
   neighbours).

**§14 posture:** no `unwrap`/`expect` anywhere in the new code; every restore-derived path is total
(a `B` entry carries no payload that could be malformed). Every `acquire_view`/`release_view`
`#[must_use]` return is consumed at its site — `debug_assert!` on the just-resolved-live invariant
(the restore-arm idiom, app.rs:2249-2250) or an explicit `drop` (`release_editor_views`,
content.rs:224). No new dependency; no process spawn; no IO.

**§20 Reference — re-confirmed N/A (Marley-specific composition model).** The spec's finding stands
unchanged at design time: no reference app owns this seam. Zed ships NO embedded-browser pane item
(behavior map `docs/zed_architecture/subsystems/07-workspace-panes-palette.md`, item roster §1.4);
Warp's only browser material is Warp-on-Web (the inverse). The citizenship frame this slice extends
(§1.3/§1.4/§1.6 — one pane type, dozens of unrelated views as first-class items) was read from the
behavior MAP, never from Zed source. The design MATCHES that behavior by making a Browser tab an
ordinary tab citizen — same `Tab` struct, same rail grouping, same close paths, same codec — via
Marley's OWN route (closed enum + `ContentId` registry, `AD-claude-pane-content-id-registry-001`),
which is a different mechanism from Zed's trait-object `Item`/`Panel`. Nothing is translated.

### Exact signatures

```rust
// tabs.rs
pub enum TabContent { Terminal(PaneGrid), Cockpit(ContentId, RightSection), CodeView(EditorSurface),
                      Browser(ContentId) }                                   // + the D1 arm
pub(crate) const BROWSER_TAB_TITLE: &str = "Browser";                        // mirrors EDITOR_TAB_TITLE :308
impl Tab {
    pub fn browser(title: impl Into<String>, id: ContentId) -> Self;         // beside `cockpit` :166
    pub fn is_browser(&self) -> bool;                                        // matches!, like `is_cockpit` :210
    pub fn browser_content_id(&self) -> Option<ContentId>;                   // the close-path collect + codec writer
}
impl Project { pub fn open_or_switch_browser(&mut self, id: ContentId) -> bool; } // beside :445
// returns true = a tab was APPENDED (it consumed the caller's resolved view);
//         false = switched to the existing row — the caller owes a release.

// content.rs
pub fn find_browser(reg: &ContentRegistry<Content>) -> Option<ContentId>;    // scans kind() == Browser
pub fn resolve_browser(reg: &mut ContentRegistry<Content>) -> ResolvedOpen;  // ONE door; one acquired view
pub fn release_browser_views(reg: &mut ContentRegistry<Content>, ids: &[ContentId]);

// grid_layout.rs
pub enum TabLayout { Terminal{..}, Cockpit(RightSection), Code{..}, Browser } // unit — no payload

// status_bar.rs
pub enum FocusTab { Terminal{..}, Editor, Cockpit, Browser }
// focus_label(&FocusTab::Browser) == "browser"

// browser.rs (NEW, pure)
pub struct PlaceholderCopy { pub headline: &'static str, pub detail: &'static str }
pub fn placeholder_copy() -> PlaceholderCopy;   // exact words settled in the React stage

// tabs.rs (SectionAction)
pub enum SectionAction { …, OpenBrowser }
```

**The codec byte.** A bare `B`: the writer pushes `'\t'` then `'B'` (no `=`, no payload); the reader
gains `} else if entry == "B" {` **after** the `V=` arm, keeping the shipped chain byte-for-byte and
the writer/reader order aligned as T/C/V/B. Confirmed unambiguous against the `strip_prefix`
symmetry: `"B"` cannot carry a `T=`/`C=`/`V=` prefix, so arm order is immaterial; a hand-written
`B=…` does NOT equal `"B"` and falls to the existing unknown-tag skip, which is the correct
tolerant-boot answer. Framing-safe by construction — `B` is outside `breaks_framing`'s `\t\n\r`
(grid_layout.rs:255) and never reaches the grid-leaf alphabet.

### File manifest

**React half — built and visually approved FIRST** (`/Volumes/Offload/Projects/marley-web`,
`pnpm --filter @workspace/marley-ide run dev` → localhost:5173):
| File | Change |
|---|---|
| `artifacts/marley-ide/src/components/LeftRail.tsx` | Zone A. The always-present "Web" `SubItem` (:388-393) becomes a REAL tab row: absent until opened, born from the Browser＋ verb, closable ×, active-highlight, inside the rail tab filter — beside the existing `filteredBrowserTabs` cockpit rows (:134). Row label settled here (proposed **"Browser"** — the shipped `EDITOR_TAB_TITLE = "Editor"`-under-an-"Editor"-header precedent, tabs.rs:308). |
| `artifacts/marley-ide/src/App.tsx` | State (:117-119): `activeBrowserTab` drops its always-on `'web'` default; `openBrowserTabs` admits the browser row so open/close is real. `browserUrl` untouched (it belongs to #404/#405). |
| `artifacts/marley-ide/src/components/views/BrowserPlaceholder.tsx` | **NEW.** Zone B — the pre-webview placeholder: headline + muted detail, themed, no URL bar, no page. A separate file so `BrowserView.tsx`'s forward-looking web simulation (:242-396 — URL bar, mock marley.dev, Manager chat, all #404/#405/CDP-adjacent) survives untouched. **Confirmed against the POC tree: `views/` holds 9 components and has no `BrowserPlaceholder.tsx` — creating it is additive.** |
| `artifacts/marley-ide/src/components/ContextMenu*` (section rows) | The 4th Browser＋ row mirroring `SECTION_BROWSER_ITEMS`, so the prototype exercises the chosen open verb. |
| `docs/MARLEY-PARITY.md` | Retire the ":187-188 POC-only 'web' tab, no Marley equivalent" note — it becomes a real 1:1 row; record the #403 zone/port entries. (Phase 5 parity sync.) |

**Rust half — the 1:1 port:**
| File | Change |
|---|---|
| `crates/marley_app/src/tabs.rs` | `TabContent::Browser(ContentId)`; `BROWSER_TAB_TITLE`; `Tab::browser`/`is_browser`/`browser_content_id`; `Project::open_or_switch_browser`; `SectionAction::OpenBrowser`; **11 exhaustive matches grow an arm** — `rail_section` :127, `grid` :194, `grid_mut` :202, `cockpit_section` :216, `cockpit_view` :225, `code_view` :240, `code_view_mut` :248, `editor` :256, `editor_mut` :264, `key_context` :275 (`&[]` — the cockpit posture), `rail_rows` row-content :1103-1118 (`(None, false)` — the Cockpit shape; Browser is un-addable so it carries no add-target id). |
| `crates/marley_app/src/content.rs` | `find_browser`, `resolve_browser`, `release_browser_views`. No enum change (D5). |
| `crates/marley_app/src/content_registry.rs` | **UNCHANGED** — the #394 lifecycle is already exactly what this needs. |
| `crates/marley_app/src/grid_layout.rs` | `TabLayout::Browser`; writer arm; reader `else if entry == "B"`. |
| `crates/marley_app/src/status_bar.rs` | `FocusTab::Browser` + the `focus_label` arm → `"browser"`. |
| `crates/marley_app/src/browser.rs` | **NEW** — `PlaceholderCopy` + `placeholder_copy()`. |
| `crates/marley_app/src/lib.rs` | Declare `mod browser;`. |
| `crates/marley_app/src/context_menu.rs` | `SECTION_BROWSER_ITEMS` 3 → 4 rows (type `[(MenuAction, &str); 4]`), the new row appended. |
| `crates/marley_app/src/app.rs` (masked) | `open_browser_tab` beside `open_cockpit_tab` :3965; `SectionAction::OpenBrowser` arm in `dispatch_section_verb` :7275; `TabLayout::Browser` restore arm beside :2241; serialize collect `else if tab.is_browser()` beside :5304; close-tab release beside :8497; close-project `dead_browsers` beside :8668; `FocusTab` arm :20591; the placeholder render arm beside the cockpit dispatch :18455. |
| `crates/marley_app/src/workspace.rs` | **UNCHANGED** — no `PaneContent::Browser` (Out: the cell layer). |

**Match-site audit (the exhaustiveness proof, not a guess):** `grep -rn --include='*.rs' 'TabContent::'
crates/` returns 28 sites — 25 in tabs.rs (the 11 matches above plus constructors/`matches!`), 3 in
app.rs (:20574/:20591/:20592, the FocusTab wiring), 1 doc-comment in keymap.rs. `TabLayout::` returns
35 — 26 in grid_layout.rs (writer/reader/tests), 6 in app.rs (:2202/:2241/:2257 restore,
:5298/:5304/:5306 serialize), 2 CONSTRUCTIONS in headless_drive.rs:83/:88 (no match — no arm owed),
1 doc-comment in tabs.rs:307. **There are exactly two Tab-drop sites in the whole tree**:
`Project::close_tab` (tabs.rs:349, called only from app.rs:8456) and `Workspace::close_project`
(tabs.rs:530, called only from app.rs:8598, plus the `#[cfg(test)]` hook :5444 which delegates to
`close_project_at`). No other code path removes a `Tab` — `self.tabs.` in tabs.rs is otherwise
push/get/iter only.

### Close-path → release mapping (REQ-006)
| Path | Site | Mapping |
|---|---|---|
| Rail × / ⌘W close-tab | app.rs:8452-8501 (`close_tab_at`) | `let browser_id = removed.browser_content_id();` beside the cockpit collect :8476; after `drop(removed)`, `if let Some(bid) = browser_id { release_browser_views(&mut self.content, &[bid]); }`. |
| Close project (rail ×) | app.rs:8597-8674 (`close_project_at`) | `dead_browsers: Vec<ContentId>` collected from `removed.tabs()` beside `dead_cockpits` :8650-8654; `release_browser_views(&mut self.content, &dead_browsers);` beside :8669. A cross-project twin keeps the instance (its own view survives); the LAST one drops it. |
| Open-when-already-open (switch) | app.rs, new `open_browser_tab` | `if !project.open_or_switch_browser(resolved.id) { release_browser_views(&mut self.content, &[resolved.id]); }` — the speculative-view release, verbatim the shipped `open_or_switch_code` precedent :5594-5598. |
| Boot restore | app.rs:2241-area | No release: restore builds a FRESH `ContentRegistry` per boot; each `B` entry's `resolve_browser` view is consumed by the tab it creates. |
| App quit | — | No release: the registry drops wholesale with `RootView`. The v1 unit payload owns nothing (this is where #405's webview teardown will need a real answer). |

Not a close path, verified: `switch_tab`, `adjust_active`, `switch_project`, `add_tab` never drop a
`Tab`. The `#[must_use]` on both registry returns is consumed at every one of the sites above
(`PR-claude-registry-release-returning-a-resource-must-be-must-use-001`).

### Regression Test Plan
| # | REQ | Test | Where | Asserts |
|---|---|---|---|---|
| T1 | 001 | `browser_tab_classifies_purely` | tabs.rs | `Tab::browser(…)`: `rail_section()==Browser`, `is_browser()`, `browser_content_id()==Some(id)`, and `grid`/`grid_mut`/`cockpit_section`/`cockpit_view`/`cockpit_content_id`/`code_view`/`code_view_mut`/`editor`/`editor_mut` ALL `None`; `key_context()==&[]`. No registry constructed anywhere in the test — the purity claim. |
| T2 | 001 | `rail_section_maps_each_content_kind` (**extend** :1867) | tabs.rs | Browser → `RailSection::Browser`, per-variant, no catch-all. |
| T3 | 001 | `rail_rows_groups_tabs_by_section_in_fixed_order` (**extend** :1902) | tabs.rs | A Browser tab files under the Browser section header beside cockpit rows; its row carries `content: None`, `pane_marked: false`. |
| T4 | 001 | `focus_label_non_terminal_tabs` (**extend** :158) | status_bar.rs | `focus_label(&FocusTab::Browser) == "browser"`. |
| T5 | 001 | `browser_content_classifies` | content.rs | `Content::Browser.kind() == ContentKind::Browser`, `!ContentKind::Browser.addable()` (the un-addable pin holds — REQ pins the Out boundary). |
| T6 | 002 | `resolve_browser_is_one_door` | content.rs | miss → `existing:false` + census 1; second call → `existing:true`, **SAME id**, view_count 2; a third project's call → same id, count 3. Census (`iter().filter(kind==Browser).count()`) ≤ 1 throughout. |
| T7 | 002/006 | `browser_drops_on_last_close_and_reopen_mints_fresh` | content.rs | release down to 0 views → the entry is GONE (`get(id).is_none()`, census 0); a following `resolve_browser` returns a **different** id with `existing:false`. The dropped-lifecycle contract, and the one behavior a pinned design would fail. |
| T8 | 006 | `release_browser_views_pairs_and_is_safe` | content.rs | acquire/release counts pair exactly across open→open→close→close; double-release of a dead id is a safe no-op; an unknown id no-ops; the `#[must_use]` return is consumed (compiles). |
| T9 | 002/006 | `open_or_switch_browser_cases` | tabs.rs | absent → append + activate + `true` (the caller's view is consumed); present → switch, no new tab, `false` (the caller owes a release), and the existing tab KEEPS its own id even when a different id is passed — mirrors `open_or_switch_cockpit_cases` :1613. |
| T10 | 003 | `shell_codec_round_trips` (**extend** :680) | grid_layout.rs | A `B` entry mixed with `T=`/`C=`/`V=` in order round-trips `restore_shell(serialize_shell(x)) == x`. |
| T11 | 003 | `browser_tab_wire_is_a_bare_b` | grid_layout.rs | **Exact bytes**: the serialized entry is `\tB` — no `=`, no payload, no id digits anywhere in the blob (the ids-never-serialize pin). |
| T12 | 003 | existing exact-wire fixtures (:716 et al.) | grid_layout.rs | UNCHANGED and green — a shell without a Browser tab serializes byte-identical to today. |
| T13 | 004 | `shell_codec_malformed_pieces_skipped` (**extend** :996) | grid_layout.rs | `X=?` still skipped; **and** `B=x` / `b` / `BB` are skipped (only the exact `B` is the marker) — the tolerant-boot pin around the new byte. |
| T14 | 004 | `legacy_shell_restores_unchanged` | grid_layout.rs | A pre-#403 fixture string restores to exactly the same `ShellLayout` as today (the additive pin — no arm reorder observable). |
| T15 | 005 | `placeholder_copy_is_stable` | browser.rs | The headline/detail strings are non-empty and exact (the copy the React stage settled; mutation-proof). |
| T16 | 005/007 | **Parity pair** | drive + POC | Live Marley: bundle → open → drive the Browser＋ row → `screencapture` the rail row + placeholder pane; POC at localhost:5173 driven to the SAME state → screenshot; READ both PNGs and compare per `MARLEY-PARITY.md` (geometry, casing, type, color — **sample pixels via `magick … "%[pixel:p{x,y}]"`, don't eyeball**). Both paths + verdict recorded in the Phase 4 notes. |
| T17 | 005 | no-config totality | drive | Boot with no `.mcp.json` reachable and open the Browser tab: the placeholder renders identically (it reads nothing that could be unconfigured — D4). |

**Genuinely uncoverable / masked, stated not hidden:** every `app.rs` site (open verb, restore arm,
serialize collect, both close releases, FocusTab arm, placeholder render) — `app.rs` is
coverage-excluded and these are live-gpui shims; they are covered *behaviorally* by T16/T17's drive
plus the pure units beneath them. There is no `trybuild` row: this ticket adds no type-safety
contract expressible as a compile-fail case (the `#[must_use]` returns are already pinned by #394's
own suite).

### Risks / decisions
- **R1 — the acquire/release asymmetry is the whole ticket's hazard.** Documented above; T6/T7/T8/T9
  exist specifically to pin it, and T7 is the test a pinned-lifecycle mistake fails. Inspect gets
  this as its first lens.
- **R2 — `SECTION_BROWSER_ITEMS` changes its ARRAY LENGTH** (3 → 4). Any test asserting the row count
  or indexing past item 0 must be re-checked (context_menu.rs:530-542 asserts item 0 per section —
  unaffected, but verify at implement).
- **R3 — skip-detach on the touched app.rs regions** (5th-strike `BF-claude-skip-detach-pump-fleet-live-001`).
  The new shims land beside `#[cfg_attr(test, mutants::skip)]` neighbours; after placement, re-run
  `cargo mutants --list -f` on the ACTUAL touched files and re-verify the skip bindings
  (`PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`) — never infer them.
- **R4 — a new module (`browser.rs`) for two strings** is a deliberate, reversible call: it keeps the
  masked render arm free of copy and gives #404's URL derivation a pure home. If #404 lands the URL
  elsewhere, folding these consts back into `tabs.rs` is a two-line change.
- **R5 — `Content::Browser` stays a UNIT.** Tempting to add a payload now "for #405"; refused (D5) —
  the payload slot grows when something ownable exists, which is also when the dropped lifecycle
  starts paying.
- **R6 — the row LABEL is settled in React, not here.** "Browser" is the proposal (the
  `EDITOR_TAB_TITLE` precedent); the React stage is the design surface and whatever it settles ports
  1:1 into `BROWSER_TAB_TITLE`.

## Phase 3 — Implement

### React-first stage (DONE FIRST, per the enforce-react-parity contract)
Dev server `pnpm --filter @workspace/marley-ide run dev` → localhost:5173; `pnpm … typecheck` green
after every edit. Built and **visually verified by reading the PNGs**, four captures in
`…/scratchpad/poc-403-0{1,2,3,5}-*.png`:
1. **`poc-403-01-empty-workspace.png`** — the Browser section header now stands with **no rows under
   it**. The always-present "Web" fixture is gone; the section is empty until a tab is opened. This
   is the behavioural heart of the slice and it reads correctly at a glance.
2. **`poc-403-02-browser-plus-menu.png`** — the Browser＋ menu shows **four** rows: Forge, Agents,
   Details, **Browser**. Row 0 is still Forge (the pinned default), the namesake is appended.
3. **`poc-403-03-browser-tab-open.png`** — a **"Browser"** row born under the section header,
   active-highlighted with a closable ×, and the center rendering the placeholder: *"No page
   loaded"* / *"The embedded browser arrives with the Forge web view."* The footer reads
   **`focus: browser`** — an independent confirmation that `"browser"` is the right `focus_label`
   word, since the POC's StatusBar already derived that string from its own section state.
4. Tab semantics driven and asserted programmatically, not eyeballed: re-opening via ＋ **switches**
   (row count unchanged, no duplicate); the rail filter matches the row's **label** ("brow" keeps it,
   "zzz" hides it, header unaffected); the × **removes** the row (2 matches → 1, the header alone).

**Row label settled: "Browser"** — the design's proposal held up on screen. A row named for its own
section header looked wrong in the abstract but is the shipped convention: `EDITOR_TAB_TITLE =
"Editor"` sits under the "Editor" header today. Ported verbatim to `BROWSER_TAB_TITLE`.

React files touched: `components/LeftRail.tsx` (the `BROWSER_TAB_LABEL` map + one unified row list —
the "Web" special-case is deleted, the filter now matches labels, close mirrors `adjust_active`),
`App.tsx` (`openBrowserTabs` admits `'web'`), `components/views/BrowserPlaceholder.tsx` (NEW),
`components/ContextMenu.tsx` (`SectionOpenBrowser` + the 4th row), `pages/Workspace.tsx` (the verb
handler + the placeholder route).

### Rust port
Built to the manifest; `cargo check --workspace --all-targets` green, `cargo fmt --all` applied,
`cargo clippy --workspace --all-targets` clean (only the pre-existing `block v0.1.6`
future-incompat notice). Every planned file landed as designed:
`tabs.rs` (variant + `BROWSER_TAB_TITLE` + `Tab::browser`/`is_browser`/`browser_content_id` +
`Project::open_or_switch_browser` + `SectionAction::OpenBrowser` + all 11 exhaustive arms),
`content.rs` (`find_browser`/`resolve_browser`/`release_browser_views`), `grid_layout.rs`
(`TabLayout::Browser` + the bare-`B` writer/reader), `status_bar.rs` (`FocusTab::Browser` →
`"browser"`), `browser.rs` (NEW) + `lib.rs`, `context_menu.rs` (3→4 rows), `app.rs` (the eight
masked sites). `content_registry.rs` and `workspace.rs` untouched, as designed.

### Deviations from the design (2, both additive)
1. **Added `open_browser_tab_for_test`** (`#[cfg(test)]`, app.rs). Not in the manifest. Reason: the
   design assumed the door would be mutation-testable, but `open_browser_tab` is unskipped while
   its only production caller (`dispatch_section_verb`) is `mutants::skip`-ed and reachable solely
   from a mouse handler — so its mutants would have had no possible killer. This is the shipped
   idiom for exactly this situation (`close_project_at_for_test`, app.rs:5443, exists because
   `close_project_at` is a private rail-click handler). The #400 exemplar avoided the problem only
   because `toggle-right-dock` gives `open_cockpit_tab` a real dispatchable action; #403 chose NOT
   to add a palette verb (D-OPEN-OPEN-VERB), so it needs the test hook instead.
2. **Refreshed four stale `content.rs` doc comments** (§21): the module header plus
   `Content::Browser` / `ContentKind::Browser` / `addable()` all still read "DECLARED — gated on
   the #389 substrate", which this ticket makes false. Also recorded there, where it belongs, why
   the variant stays a unit and how the two lifecycles differ.

### Mutation surface — the REAL `cargo mutants --list -f` output on the touched files
Traced, not inferred (`PR-claude-trace-the-real-cargo-mutants-list-not-guessed-operators`; the
5th-strike skip-detach risk R3 is what forced running this at implement rather than validate):

| Mutant | Killed by (Phase 4) |
|---|---|
| `tabs.rs:245 is_browser -> true` / `-> false` | T1 |
| `tabs.rs:251 browser_content_id -> None` | T1 |
| `tabs.rs:528 open_or_switch_browser -> true` / `-> false` | T9 |
| `tabs.rs:533:43 replace - with +` / `with /` | T9 (asserts the active index after append) |
| `content.rs:321:51 replace == with != in find_browser` | T6 — **must include a NON-Browser entry** in the registry, else `!=` still yields `None` and the mutant survives |
| `content.rs:366 release_browser_views with ()` | T8 |
| `grid_layout.rs:469:29 replace == with != in restore_shell` | T13 (with `!=`, `X=?` would push a Browser tab and a bare `B` would be skipped) |
| `app.rs:3998 open_browser_tab with ()` | the headless drive |
| **`app.rs:4003:12 delete ! in open_browser_tab`** | the headless drive — **THE leak mutant.** Deleting `!` releases on APPEND (dangling id) and skips the release on SWITCH (the permanent leak). The drive must assert view counts after open AND after re-open; a test that only checks tab counts would let it live. |
| `browser.rs:27`, `tabs.rs:195`, `content.rs:341` (→ `Default::default()`) | UNVIABLE — no `Default` impl on `PlaceholderCopy`/`Tab`/`ResolvedOpen`; excluded from MSI, and covered by T15/T1/T6 regardless |

No mutant is listed for the `B` **writer** arm (`out.push('B')` — char pushes aren't a mutation
operator), so T11's exact-bytes assert is what pins it; line coverage still applies.

## Phase 3.5 — Inspect

**4 critics, 4 lenses, run in parallel over the working-tree diff + the marley-web prototype diff:**
(1) registry view-count leak pairing & lifecycle, (2) codec byte-identity & tolerant boot,
(3) correctness/totality/panic-freedom of the new wiring, (4) simplification/reuse/convention/§20
provenance + React↔Rust parity. 20 findings; **11 confirmed and fixed, 9 rejected with reasons.**

**The headline result: the design's central claim SURVIVED.** The leak-pairing critic — the lens
aimed squarely at D-OPEN-LIFECYCLE — returned **no findings**, having empirically verified the
dropped-lifecycle pairing with 7 throwaway `#[gpui::test]` drives (open/switch/close/reopen;
two-project restore; project-close; tab-close; persist round-trip; #395 empty workspace; vanished
root; browser-only project under ⌘W; double-`B`; cross-project switch), then reverted them. It
proved the enumeration too: `Project.tabs` is private and mutated at exactly five sites (4 pushes,
1 remove), `Tab`/`TabContent` derive no `Clone` so no path can mint a second holder of a
`ContentId`, and `close_tab`/`close_project` each have exactly one non-test caller. Critic 3
independently reached the same verdict. Two independent lenses could not construct a leak.

### Confirmed and fixed

| # | Sev | Finding | Verdict + fix |
|---|---|---|---|
| F1 | high | **`cargo clippy --all-targets -- -D warnings` was RED** — `open_browser_tab_for_test` never used (app.rs). gate:2 runs exactly this; `/commit` could not have passed. | **REAL, mine.** Found by two critics independently. The test hook's whole purpose is the Phase-4 drive, and it had no caller yet. Fixed by landing that drive early (below) rather than by an `#[allow]` — §15 forbids the suppression, and the caller *is* the fix. |
| F2 | high | **An existing test was RED** — `context_menu::tests::section_items_tables` pins the ＋ row tables; `SECTION_BROWSER_ITEMS` went 3→4 and the expectation wasn't updated. 895 passed / 1 failed. | **REAL, mine — a break I introduced and missed.** Appended the Browser row to the expected slice. |
| F3 | high | **React: two call sites activate the Browser tab without OPENING it** — `LeftRail.tsx` section-header `onClick` and `CommandPalette.tsx`'s `go-browser` both set `activeBrowserTab: 'web'` without pushing into `openBrowserTabs`, rendering the placeholder for a tab with **no rail row** — a state Rust cannot reach (`app.rs` gates on `active_tab().is_browser()`, which reads `Project::tabs`). | **REAL, mine** — the direct consequence of making `'web'` born-on-open without auditing its existing callers. `go-browser` now routes through `openBrowserTab('web')` like its siblings; the section header focuses the **first open row** and is a no-op when none is open (matching Marley, where an empty section has no tab to focus). |
| F4 | high | **React: duplicate "Web" row.** `BrowserView`'s tab strip hardcodes a Web button AND maps `openBrowserTabs`, which now contains `'web'` — so "Web" renders twice whenever the Browser tab is open and a cockpit row is active. | **REAL, mine.** Deleted the hardcoded button; the map renders `'web'` like any row and labels it **"Browser"**, matching the rail. One tab, one name. |
| F5 | medium | **React: two stale hardcoded-`'web'` close fallbacks** (`LeftRail` and `BrowserView`) can land the active tab on a row that was just closed. | **REAL, mine** — the same bug in both twins. Both now move to the predecessor (`adjust_active`); when the section empties, the rail leaves the section rather than pointing at a closed row. |
| F6 | medium | **Parity drift: the placeholder's detail line.** React dims it (`text-muted-foreground/60`); Rust rendered both lines at full `colors.muted`, collapsing the two-tier hierarchy `browser.rs`'s own doc promises to a size-only distinction. | **REAL, mine.** `.text_color(colors.muted.opacity(0.6))` — the idiom already shipped at app.rs:1145/1297. Caught by two critics. |
| F7 | medium | **`build_shell_layout`'s predicate chain is not exhaustive** and the fn is `mutants::skip`-ed, so a future tab kind would silently stop persisting — no build error, no mutant, no test. | **REAL and the most valuable structural catch.** This is literally the rule the spec binds (`PR-claude-closed-enum-gate-exhaustive-match-not-predicate-chain-001`) — and my Browser arm had been added *to the chain*, extending the anti-pattern. Converted to an exhaustive `match &tab.content` (and `filter_map` → `map`, since every arm yields a layout — the old `filter_map` falsely implied tabs could be dropped here). |
| F8 | medium | **Five `Content` accessors end in `_ => None`** (`as_terminal{,_mut}`, `as_editor{,_mut}`, `as_cockpit`) — `Content::Browser` became a live, id-bearing, resolvable resident this slice and passed through all five with no compile error. | **REAL.** Strictly pre-existing, but this diff is what made `Content::Browser` load-bearing, and `as_cockpit` feeds the render dispatch's `unwrap_or(tag)` fallback — a future kind absorbed there surfaces as a *wrong pane*, not a build failure. Expanded all five to per-variant. Cheap, and the file was already open. |
| F9 | medium | **Doc names a symbol that does not exist** — my new `context_menu.rs` comment cited `section_menu_defaults`; `grep` finds only that comment. | **REAL, mine.** A fabricated symbol name is a navigation trap in a codebase this doc-dense, and `rustdoc -D warnings` can't catch it. Now cites `section_items_tables`, the test that actually pins row 0. |
| F10 | low | **`browser_content_id`'s doc claims the render dispatch resolves it** — it does not; the arm uses `is_browser()` and the id has exactly two callers, both close paths. | **REAL, mine.** Corrected, and the doc now says *why* (a unit payload has nothing to resolve) and *when it changes* (#405). |
| F11 | low | **Convention: `PlaceholderCopy` is the crate's only struct-of-copy.** Every shipped empty-state hint is a bare `-> &'static str` (`agents_empty_hint`, `forge_empty_hint`, `fleet_rail::empty_hint`); the `Clone`/`Copy` derives had no consumer. | **REAL.** Split into `placeholder_headline()` / `placeholder_detail()`. Bonus: this *improves* the mutation surface — the struct's only mutant was an unviable `Default::default()`, whereas two `&'static str` fns yield viable `-> ""` mutants that the Phase-4 test kills. |

Two further doc-hazard comments were added at the leak critic's request, both marking seams #405
will have to change: the restore arm now states that its unconditional `tabs.push` is what pairs the
resolved view (**both sibling arms drop their tab conditionally**, and a future URL/webview early-out
must move the release with it — nothing in the compiler will say so), and `release_browser_views`
now names the fact that its drop runs on the **UI thread**, the exact hazard `release_grid_terminals`
exists to avoid once a webview payload lands. Three stale `tabs.rs` docs the diff invalidated were
refreshed (`RailSection::Browser`, `SectionAction::OpenCockpit`, and `rail_section`'s "total over the
*three* variants"), and the two new accessors were moved below `cockpit_content_id` so the cockpit
accessor cluster stays contiguous.

### Rejected, with reasons

| Finding | Why rejected |
|---|---|
| **Downgrade drift**: a new build writes `B`; an OLD build skips it and `active_tab` shifts, so the user returns focused on the wrong tab. | **Real mechanism, out of scope.** The drift *class* is pre-existing — the current build already drops `V=` tabs whose files are unreadable and never reclamps at tab level. The old build cannot be fixed retroactively, and the proposed forward-direction fix (applying `reclamp_active` to restore-time tab drops) changes behavior for every existing kind — a bigger, riskier change than #403 warrants. **Ticketed as follow-up #408**; the cost is recorded under D3. |
| **`serialize_shell` writes the grid `blob` verbatim** with no D2 guard, so a `\t`-bearing blob could forge a `B` entry. | **Production-unreachable and pre-existing.** The only writer is `serialize_grid`, whose alphabet excludes `\t` (`breaks_grid_framing`). #403 doesn't create the class — a pre-#403 blob could equally forge `C=agents`. Ticketed with the above rather than adding a `debug_assert` that would show up as an uncovered test-build branch. |
| **A hand-edited `B\tB` yields two Browser tabs in one project.** | **Not a defect, and deduping would be worse.** Both critics who raised it confirmed the accounting is correct (2 tabs, 2 views, both released, resident drops at 0), it is unreachable from our writer, and `open_or_switch_cockpit` has the identical shape with duplicate `C=details` — so it is not a #403 regression. Adding a restore-arm dedupe would *introduce* the very active-index drift rejected in the row above. Tolerant boot rebuilds what is on the wire. |
| **`release_*` seams now carry two parameter conventions** (`impl IntoIterator` vs `&[ContentId]`). | Noted, not fixed. #403 copied the closer sibling (cockpit), which the critic itself called defensible; unifying means touching `release_editor_views` and its call sites — unrelated churn in this diff. Worth settling at the fourth seam. |
| **`release_browser_views` predicates on `kind()` where the cockpit sibling uses `as_cockpit().is_some()`.** | Correct as written — a unit variant has no payload accessor to call, and `kind()` is the file's own payload-free classifier. |
| **Collapse the new resolve/release seams into shared helpers.** | The critic argued both sides and landed where I did: `resolve_open` is generic over a fallible `make`, and `release_cockpit_views`' extra assert *is* its lifecycle contract — collapsing would erase the very difference a reviewer needs to see. Duplication is load-bearing. |
| **`browser.rs` is speculative generality.** | Rejected on evidence: the crate ships several sub-40-line pure modules (`editor_inlay.rs` at 19 lines, `editor_format.rs` at 31), and no existing module owns render copy. The critic's real point — that the module's stated reason (unit-pinnable copy) was unrealized because it had no tests — is answered by F11 + the Phase-4 test. |
| **"Zero tests on every new surface."** | Correct but not a finding — tests are Phase 4 by design. Two drives landed early *only* because they are the fix for F1. |
| **`px(11.0)` hardcoded rather than a type-scale role.** | The critic checked and withdrew it: `app.rs:1095`/`:10589` and the #395 center panel this mirrors all use raw `px`. |

### Clean on every remaining lens
**§20 provenance:** no Warp/Zed derivation anywhere in the new Rust or TSX; the spec's Reference
section was independently judged honest and specific. **Secrets:** `git diff | grep -iE
"token|secret|bearer|api[_-]?key|password|credential|authorization|https?://"` returns nothing in
either repo's code. **Nothing id- or URL-shaped persists** — `TabLayout::Browser` is fieldless, so it
is not structurally representable. **Framing:** no user-controllable string (root, title, arrangement
name, file path) can produce a field equal to `"B"` — each is either field 0, `sanitize_title`-filtered,
or `breaks_framing`-filtered; verified against all 24 permutations of the four tab kinds and 8
replayed pre-#403 wires. **Panic-freedom:** the render arm's `active_project()`/`active_tab()` are
reachable only past the launcher guard and the zero-tab arm respectively. **Exhaustiveness:** every
match over `TabContent` (×12), `TabLayout` (×2), `FocusTab`, `Content`, `ContentKind`, `RailSection`,
`SectionAction` is per-variant with no catch-all. **`key_context() == &[]`** breaks nothing — both
consumers already `unwrap_or_default()`, so an empty stack is the pre-existing zero-tab path.
**The #382 background-grid leak stays closed by construction** — `FocusTab::Browser` carries no
agent/remote/pane fields, so there is structurally nothing to read a grid for.

### A process correction (recorded because it is the reusable lesson)
The Phase-3 notes claimed "`cargo clippy --workspace --all-targets` clean". **That claim was false**,
and a critic caught it against the very tree it described. I had run clippy without `-D warnings` and
filtered the output with a grep pattern (`warning: unused`) that does not match how rustc words
dead-code (`warning: method ... is never used`). The lesson is not "grep more carefully" — it is
**run the gate's exact command, never an approximation of it**; `scripts/gates.sh` is the only
authority on whether the gate is green. The Phase-3 entry above stands as written, uncorrected, so
the record shows what was believed at the time; this is the correction.

### Verification after fixes
`cargo fmt --all` · `cargo clippy -p marley --all-targets -- -D warnings` **clean** (the exact gate:2
command) · `cargo test -p marley --lib` → **898 passed, 0 failed** · the two new drives green ·
marley-web `typecheck` clean.

## Phase 4 — Validate

### Tests added (12 new, mapped to the Phase-2 plan)
| Test | REQ | File | What it pins |
|---|---|---|---|
| `browser_tab_classifies_purely` (T1) | 001 | tabs.rs | Both Browser accessors answer; **all ten** others refuse; `key_context()` empty; and the inverse — terminal/cockpit/code tabs all refuse `is_browser`/`browser_content_id`. No registry constructed anywhere in the test: that IS the purity claim. |
| `rail_section_maps_each_content_kind` (T2, extended) | 001 | tabs.rs | Browser → `RailSection::Browser`. |
| `rail_rows_groups_tabs_by_section_in_fixed_order` (T3, extended) | 001 | tabs.rs | A 4th tab added: the Browser row files under the SAME header as the cockpit tenants, in storage order behind them; its row carries `content: None` and `pane_marked: false` (the un-addable shape). Tab-row count 3 → 4. |
| `focus_label_non_terminal_tabs` (T4, extended) | 001 | status_bar.rs | `FocusTab::Browser` → `"browser"`. |
| `browser_content_classifies_and_is_unaddable` (T5) | 001 | content.rs | `kind()`, `!addable()`, and all three payload accessors refuse. |
| `resolve_browser_is_one_door` (T6) | 002 | content.rs | Miss constructs (1 view, no anchor); hit resolves the SAME id and adds a view; census ≤1 throughout. **Two decoy entries are load-bearing** — see the mutation note below. |
| `browser_drops_on_last_close_and_reopen_mints_fresh` (T7) | 002/006 | content.rs | The last view out DISPOSES the resident; census → 0; `find_browser` → `None`; a reopen mints a **fresh** id. **This is the test a pinned-lifecycle design fails.** |
| `release_browser_views_pairs_and_is_safe` (T8) | 006 | content.rs | Two-project sweep, double-release of a dead id, unknown id, empty sweep — all safe. |
| `open_or_switch_browser_cases` (T9) | 002/006 | tabs.rs | Both return values pinned (the contract that keeps the lifecycle honest); an existing row keeps its OWN id when a different one is passed. |
| `shell_codec_round_trips` (T10, extended) | 003 | grid_layout.rs | A `B` **mixed** with `T=`/`C=`/`V=`; the exact wire now ends `…\tB`. |
| `browser_tab_wire_is_a_bare_b` (T11) | 003 | grid_layout.rs | Exact bytes `0\n/tmp/p\t0\tB` — no `=`, no payload. The writer arm's ONLY guard (a char `push` has no mutation operator). |
| `only_an_exact_b_is_the_browser_marker` (T13) | 004 | grid_layout.rs | `B=x`, `b`, `BB`, `B `, ` B`, `Browser` all skipped; only the bare `B` restores. Kills an `entry.starts_with("B")` mutant that would otherwise survive the whole suite. |
| `legacy_pre_403_shell_restores_unchanged` (T14) | 004 | grid_layout.rs | The pre-#403 wire restores identically AND re-serializes byte-identically — the new writer arm is inert without a Browser tab. |
| `placeholder_copy_is_exact` (T15) | 005 | browser.rs | Both strings pinned — the module finally keeping the promise inspect flagged as unkept. |
| *(landed at inspect as the F1 fix)* `browser_door_drops_on_last_close_and_reopens_fresh_headless` | 002/006 | headless_drive.rs | The REAL app door: open → 1 view, switch → still 1, close → 0 + entry gone, reopen → fresh id. |
| *(same)* `browser_restores_from_a_bare_b_shared_across_projects_headless` | 002/003/004 | headless_drive.rs | Two projects' `B` entries restore as views of ONE resident (view count 2, census 1, title "Browser"); closing one project leaves the survivor holding it. |

**T12 (existing exact-wire fixtures unchanged):** verified — `shell_codec_titles`,
`shell_codec_pane_name_rider`, `serialize_zero_tab_project_round_trips` and the rest pass untouched.
The only fixture edited was `shell_codec_round_trips`, deliberately, to add the mixed `B`.

**The decoys in T6 are not scenery.** `find_browser` scans by kind; in a registry holding *only* a
Browser, both `==` and `!=` answer `None` on a miss, so the `content.rs:321 == → !=` mutant would
survive. The `FileTree`/`Git` decoys make the wrong spelling resolve a FileTree id as the browser,
which the assertions catch. (Both are payload-less, so no PTY is needed.)

### Test results — actual output
```
cargo test -p marley --lib      → 908 passed; 0 failed; 0 ignored
cargo nextest run --workspace   → 2062 tests run: 2062 passed, 5 skipped
cargo test --workspace --doc    → 0 tests, ok (no doctests in the touched crates)
```
No pre-existing failures anywhere in the workspace.

### The live-app drive (REQ-005/REQ-007) — driven, captured, and READ

**Operator protocol used (recorded, because #402's abort is the reason it exists).** Before sending
any synthetic input I sampled `HIDIdleTime` three times: **3s → 6s → 9s, climbing monotonically** —
no human input, with Teams merely left open frontmost. Then two hard containments: (a) the app was
launched with an **isolated `HOME`** pointing at a scratch dir, so `marley_config_dir()` resolved
there and **the user's real `~/.marley` was never opened** (confirmed after: `settings.toml` still
dated Aug 5, untouched); (b) **only mouse clicks inside Marley's own window** — no ⌘-chords at all,
which is what made #402's drive unsafe. Activation and the drive verbs shared one shell command per
the harness README, so focus could not drift between them.

The isolated home also bought **T17 for free**: the scratch project has no `.mcp.json` and neither
does the fake home, so every capture below IS the no-config case. The placeholder rendered
identically — as designed, it reads nothing that could be unconfigured.

| Capture | What I read in it |
|---|---|
| `marley-403-01-boot.png` | Boots into the seeded workspace; the rail shows Editor / Terminal / Panes / **Browser** with the **Browser section EMPTY** — no rows. The always-present fixture is gone in Rust exactly as in React. |
| `marley-403-03-plus-menu.png` | The Browser＋ menu open with **four rows: Forge, Agents, Details, Browser** — Forge still item 0, the namesake appended. |
| `marley-403-04-browser-tab-open.png` | A **"Browser"** row born under the section header, active-highlighted, closable ×; the placeholder centered — *"No page loaded"* / *"The embedded browser arrives with the Forge web view."*; footer reads **`focus: browser`**. |
| `marley-403-05-restored-from-B.png` | **After a real quit + relaunch**: the tab is back, active, placeholder rendering, `focus: browser` — rebuilt from a payload-less marker with no id and no URL on the wire. |
| `marley-403-06-after-close.png` | After clicking ×: the row is gone, the Browser section is empty again, the terminal tab is active, footer back to `focus: terminal`. |

**The wire, read off disk at each step** (`fakehome/.marley/config/settings.toml`):
```
after open : 0\n<root>\t1\tT=t=<cwd>\tB      ← the bare B, active_tab=1
after close: 0\n<root>\t0\tT=t=<cwd>         ← B gone, back to the pre-#403 shape
```
REQ-003 proven on the live app, not only in a unit: no `=`, no payload, no id, no URL.

### Parity pair (REQ-007) — pixels sampled, not eyeballed
`marley-403-04-browser-tab-open.png` ↔ `poc-403-parity-1024.png`, both driven to the same state at
**1024×768**. Per `MARLEY-PARITY.md`, sampled with `magick … "%[pixel:…]"`:

| Sample | Marley | POC | Verdict |
|---|---|---|---|
| Center-pane background | `srgb(11,12,15)` | `srgb(11,12,15)` | **exact** |
| Active rail-row fill | `srgb(36,63,70)` | `srgb(36,64,71)` | 1/255 on G+B — **pre-existing shell chrome**, the parity doc already records `--rail-active` as a live-capture approximation. Not a #403 delta. |
| Headline band tone | 9.292% | 8.469% | — |
| Detail band tone | 9.062% | 8.235% | — |
| **Headline → detail drop** | **0.230 pp** | **0.234 pp** | **matches** — the two-tier hierarchy holds on both sides by the same margin. This is inspect F6's `colors.muted.opacity(0.6)` fix confirmed empirically; before it, Marley's detail sat at the headline's tone. |

Absolute band levels differ (~0.8pp) because the two crops sit 2px apart vertically and include
different amounts of background — measurement offset, not a colour delta. Row copy, casing,
closability, section placement, the 4-row ＋ menu with Forge first, and the footer's `focus:` word
all match. **Verdict: parity holds on this ticket's new surface.**

**One footer difference in the capture pair, checked and NOT a delta** (recorded so a later reader
does not "fix" the POC on a misreading, which I nearly did): the POC footer shows `lsp: ready` and
Marley's does not. Marley's LSP segment is gated on `lsp_hosts.get(root)` — whether a host exists
for the project ROOT — **not** on the active tab's kind (`active_lsp_segment`, app.rs:5520). The
scratch project driven here is a two-file temp dir that never spawned rust-analyzer, so the segment
is legitimately absent; the POC's mock is always "ready". Fixture difference, not surface behaviour.
The POC's `lspVisible` gate (`activeSection === 'editor' || 'browser'`) is an unrelated
pre-existing approximation of "a host exists" and is out of this ticket's scope.

### Gate

**Run 1 — `scripts/gates.sh --diff` came back RED on gate:14 (docs).** Recorded rather than
quietly re-run, because two things in it matter.

First, the floors held on the very first attempt: **gate:4 coverage ≥100% PASS** and **gate:5
mutation MSI ≥100% PASS**, alongside miri and visual. Every mutant the Phase-3 trace predicted was
killed by the test mapped to it — including `app.rs:4003 delete !`, the leak mutant, and
`content.rs:321 == → !=`, which needed T6's decoy entries to be killable at all.

Second, the failure was mine and it was a real defect, not a formality: two of my new doc comments
used intra-doc LINKS to `resolve_browser`, which is a private item inside the private `content`
module — `RUSTDOCFLAGS="-D warnings"` rejects that (`rustdoc::private_intra_doc_links`).
```
error: public documentation for `Browser` links to private item `resolve_browser`
  --> crates/marley_app/src/content.rs:52:47
error: public documentation for `browser` links to private item `crate::content::resolve_browser`
  --> crates/marley_app/src/tabs.rs:195:58
```
Fixed at source by demoting both to plain code spans — which is the shipped convention here for
exactly this reason (`Content::Cockpit`'s own doc writes `` `resolve_or_register_cockpit` `` and
`` `CockpitIndex` `` in backticks, not links). rustdoc suggested `#[allow(rustdoc::private_intra_doc_links)]`;
that is precisely the suppression §15 forbids, and it was not used.

A note on how this was caught, since it nearly wasn't: the background run's exit code was **0**
despite `GATE RED`, because the command was piped through `tail` and the pipeline reported *tail's*
status. The verdict came from READING the output, not from the exit code — the same lesson as the
Phase-3 clippy miss, one layer down.

**Run 2 — the green one:**
```
══ gate summary (diff) ══
  PASS  gate:1  rustfmt              PASS  gate:11 shellcheck
  PASS  gate:2  clippy (-D warnings) PASS  gate:12 no-suppressions
  PASS  gate:3  tests (nextest+doc)  PASS  gate:13 source-bans (SAST)
  PASS  gate:7  cargo-audit          PASS  gate:14 docs (rustdoc/todos/brand)
  PASS  gate:8  cargo-deny           PASS  gate:4  rust coverage (>= 100% lines)
  PASS  gate:9  cargo-machete        PASS  gate:5  mutation (MSI >= 100%)
  PASS  gate:10 gitleaks (secrets)   PASS  gate:6  miri (unsafe crates)
                                     PASS  gate:15 visual / AX
  15 passed, 0 failed
GATE GREEN [diff]

mutation: 33 caught / 0 missed → MSI 100.0% (floor 100%)
```
Receipt written: `.git/ignibyte-gate-receipt` = `6c1151e4d00416bc09f36807180ae58bfcb5e129`.

**33 mutants, 33 killed, zero missed** — every one the Phase-3 `--list` trace predicted, killed by
the test the plan mapped to it. The two that would have been silent without deliberate design:
`app.rs:4003 delete !` (the leak mutant — killed by the drive's VIEW-count assertions, not its tab
counts) and `content.rs:321 == → !=` (killed only because T6 seeds non-Browser decoys).

**Pre-existing failures: none.** Unlike #402, no unrelated red surfaced — the FULL-audit debt
tracked as forge #407 is a whole-workspace mutation concern and is untouched by this diff's scope.
No baseline was added, no floor lowered, no suppression used (gate:12 confirms).

## Phase 5 — Complete

### Documentation (§21)
- **CHANGELOG.md** — an `### Added` entry naming the tab citizenship, the bare `B`, the ONE-resident
  identity, and — at length, because it is the slice's real content — why the lifecycle is the
  INVERSE of #400's and why copying its wiring would have leaked.
- **`embedded-browser-model.md`** — train step 2 marked ✅ SHIPPED with the as-built shape
  (`TabContent::Browser(ContentId)`, no tag beside the id, and the reason), the lifecycle inversion,
  the ONE-resident identity evidence, and the live open→persist→restart→restore→close verification.
- **`pane-composition-model.md`** — a new slice **8b** beside #400's slice 8, framed deliberately as
  the *counter-example* to it rather than a repetition: (a) no tag, because the variant discriminant
  already discriminates when there is one kind and a payload-less marker; (b) DROPPED, with the
  reason the cockpit's acquire-on-append shape does not transfer.
- **`roadmap.md`** — Phase E now tracks the train: slice 1 ✅ #402 (GO), slice 2 ✅ #403, with
  #404/#405/#406 named as remaining.
- **`marley-web/docs/MARLEY-PARITY.md`** — the ":187 POC-only 'web' tab, a tab Marley has no
  equivalent for" note is RETIRED (it has one now), and the port map gained three rows:
  `BrowserPlaceholder.tsx` ↔ `browser.rs`, the LeftRail Browser rows ↔ `tabs.rs`, and
  `SECTION_BROWSER_ITEMS` ↔ its Rust twin.

### Parity sync at close
The React and Rust sides agree on every #403 surface: row label, born-on-open, closability, filter
behaviour, the 4-row ＋ menu with Forge pinned at 0, the placeholder copy (byte-identical), and the
two-tier tone hierarchy. Both port-time deviations found at inspect were resolved in the direction
the contract dictates — the Rust detail-line opacity moved to match the approved React prototype
(new surface → React is the reference), and React's three orphan-state callers were fixed to match
Rust's impossibility (pre-existing shell behaviour → Marley is right). The one remaining visible
difference in the capture pair, the footer's `lsp:` segment, was investigated and is a FIXTURE
difference, not a delta — see the Phase-4 note.

### Knowledge captured (forge)
- **`AD-claude-registry-lifecycle-fork-pinned-vs-dropped-001`** — the durable decision: a registry
  resident's lifecycle dictates its resolve-door CONTRACT, and the two wirings are not
  interchangeable. Records the leak arithmetic in full, because the wrong version reads as correct.
- **`BF-claude-approximated-the-gate-command-and-reported-green-001`** → **`PR-claude-run-the-gates-exact-command-never-an-approximation-001`**
  — I wrote "clippy clean" into the Phase-3 record while the tree was red, having run clippy without
  `-D warnings` and filtered with a grep pattern that misses rustc's dead-code wording; and I read a
  clean `cargo check --all-targets` as "no test broke" when a pinned test was red the whole phase.
  The rule: run the literal command `gates.sh` runs, and never infer a pass from `check`.
- **`BF-claude-fixture-to-real-tab-orphaned-its-callers-001`** → **`PR-claude-fixture-to-instance-audit-every-activator-001`**
  — turning an always-present fixture into a born-on-open instance orphaned three existing callers
  that activated it without opening it. Type checking cannot catch this (the key stays valid) and a
  happy-path drive of the new verb cannot either.
- **AAR submitted** (`completed`, effectiveness 4, 5 novel findings) — distillation, confidence-drift
  and pattern-emergence jobs enqueued.

### Lessons — what worked, what bit
- **What worked: tracing the real mutant list at IMPLEMENT, not validate.** Running
  `cargo mutants --list -f` on the actually-touched files before writing any test is what surfaced
  `app.rs:4003 delete !` — and knowing that mutant existed is why the drive asserts view counts
  instead of tab counts. A test written from the design alone would have passed while the leak lived.
  The same trace showed `content.rs:321 == → !=` needs non-Browser decoys to be killable at all.
  33/33 mutants died on the first mutation run because the tests were written against that list.
- **What worked: four critics on distinct lenses.** The lens aimed at the design's central claim
  (leak pairing) cleared it empirically with 7 throwaway drives, which is worth more than my own
  reasoning; meanwhile the *other* three found everything real — a red gate, a red test, three
  orphaned React callers, and a non-exhaustive persistence writer. Independent lenses beat a deeper
  single pass.
- **What bit: approximating a verification command.** Twice, one layer apart — clippy without
  `-D warnings` at Phase 3, and a backgrounded gate whose exit code was `0` because it was piped
  through `tail` while printing `GATE RED`. Both times the honest signal was in the OUTPUT, not the
  status. Rule recorded.
- **What bit: the sibling-shaped trap.** #403 was written as "#400 again for another kind", and the
  closest structural sibling was the wrong template. The tell was cheap to check and easy to skip:
  ask who owns `insert()`'s first view.
- **Operator protocol that made the live drive safe** (the #402 abort is why it exists): sample
  `HIDIdleTime` repeatedly and require it CLIMBING; launch with an isolated `HOME` so the user's real
  `~/.marley` is unreachable by construction; and use only clicks inside the app's own window — no
  ⌘-chords, which is what made #402's attempt unsafe. The isolated home also delivered the
  no-`.mcp.json` totality case for free.
