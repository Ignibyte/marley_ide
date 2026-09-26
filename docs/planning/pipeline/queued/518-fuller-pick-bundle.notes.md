# A fuller pick bundle: HTML, styles and the React component — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-518-fuller-pick-bundle.md
- **Pipeline spec:** 518-fuller-pick-bundle.spec.md

## Phase 1 — Plan (drafted overnight, 2026-09-25)
- **Request:** Chad, 2026-09-25: the Orca survey's items are taken. Its item 4, "a fuller pick",
  lands here, before #505, which compares against these fields: bounded outerHTML with scripts
  stripped, the computed-style subset, the React component chain and its source (`_debugSource`,
  or React 19's `_debugStack` mapped through `source_map.rs`), and redaction of secret-looking
  attribute values and URL query strings; `browser_pick` returns the new fields (report 03 §2.4
  and item 1; Orca's `grab-guest-react-script.ts` and `browser-grab-payload.ts`).
- **Classification / tier:** feature, prong 3, the pick (B3a's bundle). Marley crates only
  (`marley_browser`, `marley_workbench`, `marley_mcp`); no Zed path.
- **Recall (§18.3):**
  - AD-claude-496-picks-are-staged-and-sent-to-the-last-terminal-001: the line is the reference,
    `browser_pick` the content. D6 keeps it so.
  - AD-claude-497-marley-reads-source-maps-and-finds-sources-in-the-worktrees-001 and
    L-claude-497-chromium-loads-a-source-map-for-its-client-001: maps come through the page or a
    `data:` URL, sources are looked for only in the worktrees, positions count from 1; the React
    frames reuse all of it.
  - L-claude-496-chromium-picks-an-element-over-cdp-001: `Debugger.enable` replays
    `scriptParsed` for the scripts already loaded, so the page's `scripts` map knows the bundle's
    source map by the time a pick needs it.
  - L-claude-492-chromium-shows-a-password-by-length-001: the accessibility tree leaks a
    password's length; the new HTML must not leak the value itself, hence D2's field rule.
  - #496's promise, "A pick never records what was typed into a field" (CHANGELOG): D2 extends it
    to the HTML.
  - #516 (active, in Code tonight): `Redactor::redact` at the tool boundary; D3 calls it for
    `browser_pick`.
  - Brain: not consulted in this drafting run (read-only brief); promotion runs `brain_ask`.
- **Discovery** (line numbers as of this draft; #516's Code phase is editing `browser_tools.rs`
  and `registry.rs` tonight, so theirs will move):
  - `crates/marley_browser/src/pick.rs`: `INTERACTIVE_ANCESTOR` (lines 25 to 37); `DESCRIBE` (43
    to 100: test id, id, text and CSS-path locators, the box through same-origin frames, the
    blockers; `text` is a button's label only, since a field's value is what the user typed);
    `PickBundle` (171 to 188); `Described` (200 to 208); `capture_pick` (285 to 332: resolve the
    node, walk to the interactive ancestor, `DESCRIBE` by value, the accessibility node, the
    listeners); `crop` (339); `call_on` (364, `Runtime.callFunctionOn` with no world named, so on
    the object's own, the main world `DOM.resolveNode` resolved it in).
  - `crates/marley_workbench/src/browser.rs`: `script_parsed` (1137) keeps the page's scripts by
    id with their source maps; `pick_requested` (1172) reads the pick, crops it, stages it, then
    maps the listeners through `original_positions` (5467) and `load_map` (5512) and stores them in
    `pick_sources` (1265), which finds each file with `find_source` (5540); `listener_place`
    (5589) is the tray's use of the listeners.
  - `crates/marley_browser/src/source_map.rs`: `map_location` (117), `source_path` (142, URL
    paths and `/@fs/` read as components), `SourceMap::original` (205, 0-based in and out).
  - `crates/marley_browser/src/observe.rs`: `SECRET_NAMES` (21), `redact_url` (287).
  - `crates/marley_workbench/src/browser_tools.rs` `pick` (285): serializes the whole `Pick`, so
    new `Serialize` fields reach the answer. `crates/marley_mcp/src/registry.rs`: the `pick`
    description (156) and `pick_schemas` (638).
  - `crates/marley_mcp/src/redact.rs` (#516, uncommitted): `Redactor::new(patterns)` and
    `Redactor::redact(text) -> Redacted`.
  - `script/e2e/browser-fixture.sh`: `mcp_agent pick` prints the bundle's fields; `serve_site`,
    `offline_chromium`.
  - React on the box: 18.3.1 UMD at `/srv/stacks/ignibyte/Ignibyte-Marketing-Site/node_modules/
    react/umd/react.development.js` (`config.__source` read at line 800) and `react-dom/umd/
    react-dom.development.js` (`fiber._debugSource = element._source` at 28517, the key
    `'__reactFiber$' + randomKey` at 11491); 19.2.8 CJS at `/srv/stacks/scorchkit_home/
    node_modules/react/cjs/react-jsx-dev-runtime.development.js` (`jsxDEV` and its
    `Error("react-stack-top-frame")` at 330 to 338) and `react-dom/cjs/
    react-dom-client.development.js` (`_debugStack` copied at 5044, the counter reset at 17306 to
    17308, the key at 24279); esbuild 0.28.2 at `/srv/stacks/scorchkit_home/node_modules/.bin/
    esbuild`. No Vue or Svelte package in any project's `node_modules` on the box.
- **Decisions:** D1 to D6 in the spec.

### Design
- **Approach.**
  1. *`DESCRIBE` grows.* After the locators and blockers, the function builds the clone (D2),
     reads the sixteen styles with `getComputedStyle(element).getPropertyValue`, the sibling
     texts (previous and next alternately, `innerText` trimmed and collapsed, ten at most, 200
     characters each), the selection (`getSelection().toString()`, 500), and the React fields: the
     first own key starting `__reactFiber$` or `__reactInternalInstance$`, the walk up `.return`
     (35 levels, six names, Orca's skip rules: `Fragment`, `Root`, `Routes`, `Route`, `Outlet`,
     `Provider`, `Consumer`, `Profiler`, `Suspense`; names ending `Boundary`, `BoundaryHandler`,
     `Router`, `Provider`, `Consumer`, `Context`, `Wrapper`; names starting `Inner`, `Outer`,
     `Client`, `Server`, `RSC`, `Dev`, `React`, `Hot`; two characters or fewer), the first
     `_debugSource` of a fiber or its `_debugOwner` (`fileName`, `lineNumber`, `columnNumber`),
     and without one the host fiber's `_debugStack.stack`, 4,000 characters. Each property read
     sits in a `try`, since a page's getters can throw. The secret rules are constants in the
     function, so one list serves the page's pass.
  2. *The clone's pass.* `element.cloneNode(true)`; `clone.querySelectorAll('script')` removed;
     for the clone and each element in it: an `input` whose type is not button, submit, reset or
     image loses its `value` attribute, a `textarea` its text; every attribute is checked, name
     against `SECRET_NAMES`, value against Orca's patterns (lower-cased `includes`), and set to
     `[redacted]` on a match; the URL attributes are parsed with `new URL(value,
     document.baseURI)` and written back without `search` and `hash` (a value that does not parse
     is written as `[redacted]`), `srcset` candidate by candidate. Then `outerHTML`, cut at
     4,096 characters plus ` (truncated)`.
  3. *The bundle.* `PickBundle` gains `html: String`, `styles: BTreeMap<String, String>` (CSS
     names as keys), `nearby_text: Vec<String>`, `selected_text: Option<String>`, and `component:
     Option<Component>` with `chain: Vec<String>` (outermost first) and `source:
     Option<ComponentSource { from: "debug source" | "debug stack", source: String, file:
     Option<String>, line: u32, column: u32 }>` (`source` as React or the map names it, `file`
     relative to its worktree). `Described` deserializes the new fields with defaults, so a page
     whose script fails to give one still gives a pick. A `clamp` step cuts each string to its
     budget with the same mark.
  4. *React 19's frames.* `pick::stack_frames(stack) -> Vec<StackFrame { function, url, line,
     column }>` parses V8 lines (`at f (u:l:c)` and `at u:l:c`), skipping the first line (the
     error's message). In the workbench, `pick_requested` hands the frames to the same task that
     maps the listeners: frame by frame, the script with that URL in the page's `scripts` map
     gives its source map, `load_map` loads it (cached per script, as for listeners), and
     `SourceMap::original(line - 1, column - 1)` gives the source; the first frame whose function
     is not React's own and whose source has no `node_modules/` component wins. A frame whose
     script has no map is judged by its URL. `find_source` gives the project file, as in
     `pick_sources`, which stores the component source with the listeners' and flips
     `sources_read`.
  5. *`browser_pick`.* The registry's output schema gains `html`, `styles`, `nearby_text`,
     `selected_text` and `component`; the description says what each is. The tool runs `html`,
     `text`, `nearby_text` and `selected_text` through the `Redactor` #516 builds from the
     settings, before the answer is serialized.
  6. *The fixture.* `mcp_agent pick` prints the HTML's length and first 300 characters, the
     styles, the nearby texts' count, the chain and the source; `mcp_agent pick-json <id> <file>`
     writes the structured answer to a file beside the shots.
- **File manifest.** Marley crates only: `crates/marley_browser/src/pick.rs`;
  `crates/marley_workbench/src/browser.rs` (the frames' mapping, `pick_sources`),
  `crates/marley_workbench/src/browser_tools.rs` (the `Redactor` call);
  `crates/marley_mcp/src/registry.rs` (schema and description). `script/e2e/browser-fixture.sh`
  (the fixture's printing and `pick-json`), `script/e2e/518-fuller-pick-bundle.sh` (Test). Docs at
  Complete: `CHANGELOG.md`, `docs/marley/three-prong-plan.md`,
  `docs/marley_architecture/marley_browser.md`.
- **Ledger rows.** None in `docs/marley/zed-touchpoints.md`: no Zed path. Knowledge at Complete
  (expected): an AD for the fuller bundle (what it holds, the in-page redaction, React 19 through
  the maps) and a lesson on reading React's fiber over CDP.

### E2E plan
Setup: `offline_chromium`; the scratch repository with `src/SaveButton.jsx` and
`src/SaveButton19.jsx` (the files the sources should resolve to); React 18.3.1's two UMD files
copied into the site from the path above (or `E2E_REACT18_UMD`); `src/app19.jsx` bundled into the
site with esbuild (`--bundle --jsx=automatic --jsx-dev --sourcemap=inline
--define:process.env.NODE_ENV='"development"'`) against React 19.2.8, the scratch repository's
`node_modules` a link to the React 19 project's (or `E2E_REACT19_MODULES`). Setup stops with a
message naming what is missing when either build is absent. The secrets page's tokens are built
at run time from pieces, so the scenario file holds none for gitleaks.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-004 | `r18.html`: a `Card` holding a `SaveButton`, created with `React.createElement` and the `__source` Babel's JSX dev transform writes; pick the button | `518-01-react18-pick`; the log: chain `<Card> <SaveButton>` and `src/SaveButton.jsx` at the line and column given |
| REQ-005 | `r19.html` loads the esbuild bundle; pick its button | `518-02-react19-pick`; the log: the chain, and the source `src/SaveButton19.jsx` at the JSX's line, from the debug stack through the inline map |
| REQ-001, REQ-006, REQ-007 | `secrets.html`: type an e-mail into the email field (whose script copies the value into its `value` attribute), then pick the form by a click on its padding, which no control covers | `518-03-secrets-pick`; the log: no `<script>`, `action` and `href` without query, `data-api-key` and the hidden `csrf_token`'s value `[redacted]`, no e-mail; no component |
| REQ-002 | pick the Go button, whose style sets `padding: 8px 16px` and `color: rgb(10, 20, 30)` | the log: those two values among the sixteen |
| REQ-006 | `mcp_agent pick-json` for each pick, then a grep of the files for the fake tokens and the typed e-mail | the log: nothing found |
| REQ-001 | pick the long list (over 4,096 characters of HTML) | the log: 4,096 characters and ` (truncated)` |
| REQ-003, REQ-008 | select a paragraph's text by a drag, then pick a list item beside it | the log: the selection and the siblings' texts |
| REQ-009 | rerun `496-element-picker.sh` and `497-pick-source.sh` | their logs and shots as before |

Not reachable here: Next.js's and Vite's own dev servers (the esbuild bundle stands in for the
hardest case, React and the app in one script), and React 17 and older.

### Risks
- The main world is where the page's own scripts run: a page that has replaced `cloneNode`,
  `getComputedStyle` or `Object.keys` changes what Marley reads. D3's clamps bound the damage;
  the probe at promotion confirms that `DOM.resolveNode` without a context resolves in the main
  world.
- React 19 makes debug stacks for the first 10,000 elements after each reset; an element created
  past that (a page building a huge list within one second) carries React's placeholder stack, so
  its source is empty, never wrong.
- A `_debugSource` column's base differs between Babel versions (older ones counted from 0); it
  is passed on as React gives it and labelled so.
- The HTML budget counts UTF-16 code units in the page and characters in Rust; the Rust clamp is
  the one that holds.
- #516 must land first for D3's `Redactor` call; if it slips, that call waits and the in-page
  pass stands alone.
