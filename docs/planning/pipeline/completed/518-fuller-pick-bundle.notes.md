# A fuller pick bundle: HTML, styles and the React component — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-518-fuller-pick-bundle.md
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
| REQ-003, REQ-008 | select a word of a paragraph by a double click, then pick a list item beside it (a drag was planned; its selection does not outlast the release, #580) | the log: the selection and the siblings' texts |
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

## Promotion (2026-09-26, at `2b49011426`)
- **Seams re-read:**
  - `pick.rs`: `INTERACTIVE_ANCESTOR` (25), `DESCRIBE` (43), `PickBundle` (171), `Described`
    (201), `capture_pick` (285), `crop` (339), `call_on` (364). As the draft read them.
  - `browser.rs`: `script_parsed` (1279), `pick_requested` (1314), `pick_captured` (1362),
    `pick_sources` (1408), `original_positions` (5762), `load_map` (5807), `find_source` (5835),
    `listener_place` (5884). Moved by #503 to #504, unchanged in shape.
  - `source_map.rs`: `map_location` (117), `source_path` (142), `SourceMap::original` (205),
    `load_resource` (356), now over `load_bytes` (368) since #504, the same for maps.
  - `observe.rs`: `SECRET_NAMES` (21), `redact_url` (287).
  - `browser_tools.rs` `pick` (516) serializes the whole `Pick` and redacts nothing;
    `registry.rs`: the `pick` description (160), `pick_schemas` (687).
  - `redact.rs`: `Redactor::new` (158) and `redact` (175); the workbench's
    `mcp::agent_redactor(cx)` (369) gives the redactor, or `None` when the user turned it off,
    and `browser_console` shows the call's shape (`browser_tools.rs` 225).
- **Landed since the draft:** #516 (the redactor, applied to `browser_console` and the terminal
  tools only, so `browser_pick` hands the page's text to agents unredacted today); #562's rules;
  #504's `load_bytes`.
- **The React builds** are still on the box: React 18.3.1's UMD files in
  `Ignibyte-Marketing-Site`, React 19.2.8's CJS files and esbuild 0.28.2 in `scorchkit_home`.
- **The main world, confirmed from the code:** `capture_pick` resolves the node with
  `DOM.resolveNode` and no `executionContextId`, which CDP resolves in the node's main world,
  and `call_on` names no context, so `DESCRIBE` runs where React keeps `__reactFiber$…`. The
  Test phase's React picks prove it on a page.
- **Recall added:**
  - PR-claude-redact-the-whole-text-before-cutting-it-001 and F-claude-516-a-cut-before-
    redaction-leaks-the-cut-secret-001. The draft cut the HTML at 4,096 in the page and redacted
    at the tool, so a token straddling the cut would have reached agents as a piece that no rule
    matches. Changed: the page and the bundle cap each text for the trip only (the HTML at
    65,536 characters, every other text at 4,096), and `browser_pick` redacts each whole field,
    then cuts it to its budget (D3, D5, REQ-010).
  - AD-claude-516 (#516's D2): every tool that hands agents page text redacts it, so the call
    covers each field the page's text reaches: `html`, `text`, `name`, `nearby_text`,
    `selected_text` and each locator's value, the older ones included.
  - F-claude-496-the-picks-text-read-a-fields-value-001: no field value in any field; D2's rule
    extends it to the HTML.
  - L-claude-516-fake-secrets-are-put-together-at-run-time-001: the secrets page's tokens and the
    long list's are built in the setup from pieces.
- **Brain consultation ef1154d6c93740a38d4fb5ade5e604ef:** nothing on this seam.

### Design changes at promotion
- **Approach 1 and 2:** the HTML is capped at 65,536 characters with ` (truncated)`; the sibling
  texts and the selection at 4,096. The page does no budget cut.
- **Approach 3:** `clamp` holds each field to its trip cap on arrival; the budgets move to
  `browser_pick`.
- **Approach 5:** `browser_pick` and `browser_picks` answer from one copy made for agents
  (`pick_for_agents(pick, redactor)` in `browser_tools.rs`). It takes `agent_redactor` and
  redacts, each whole: the page's `title`, the `caption`, and the bundle's `name`, `text`,
  `html`, each `nearby_text`, `selected_text` and each locator's `value`. Then it cuts the HTML
  to 4,096, each nearby text to 200 and the selection to 500, each with ` (truncated)` when cut,
  and makes `summary` again from the redacted bundle (`PickBundle::summary`, whose 60-character
  cut then follows the redaction). With redaction off it only cuts. The Code phase checks a
  listener's script URL, and passes it through `redact_url` when nothing does yet.
- **E2E plan, the long list's row:** the list's HTML holds a fake token built in the setup so
  that it spans the 4,096th character of the serialized list; the saved answer holds no piece of
  it (REQ-010), and the HTML is 4,096 characters and ` (truncated)` (REQ-001).
- **E2E plan, a new row (REQ-011):** the secrets page has a button whose label holds a fake token
  built in the setup; pick it, then `mcp_agent picks` and `pick-json`: its summary, name and HTML
  show `[redacted: …]` and no piece of the token.
- **Phase 1 checklist (no task tool in this session):** pre-flight clean; recall written; pair
  promoted, BACKLOG row removed, ticket in progress; seams re-read; prior art as drafted (Orca's
  grab scripts, React's dev builds, CDP, Source Map v3, the code we ship); spec updated (D3, D5,
  REQ-010, REQ-011, the tools' copy for agents); design changes written. Status: Plan PASS.

## Phase 2 — Code
- **Built** (Marley crates only):
  - `marley_browser/src/pick.rs`:
    - `DESCRIBE` (now `function (secretNames)`, called through the new `call_on_with`, which
      passes `observe::SECRET_NAMES`) reads the clone's HTML, the sixteen styles, the sibling
      texts, the selection and React's fiber, each read in a `try` so a failure empties one field
      and the pick still comes. Every cut in the page is a trip cap (HTML 65,536, texts 4,096, a
      style 500, a name 200, a debug source 500, a stack 4,000) marked ` (truncated)`.
    - `PickBundle` gains `html`, `styles` (`BTreeMap`), `nearby_text`, `selected_text` and
      `component: Option<Component { chain, source, frames }>`; `ComponentSource { from, source,
      file, line, column }` with `SourceKind::{DebugSource, DebugStack}` serialized as
      `debug source` and `debug stack`; `StackFrame` and `StackFrame::is_reacts`.
    - `Described`'s new fields default, and its React part is read by hand from JSON
      (`component_of`), since the page's own objects fill it; `trip` holds each field to its cap
      on arrival; `within(text, budget)` and the public budgets `HTML_BUDGET`, `TEXT_BUDGET` and
      `SELECTION_BUDGET` are what the tool cuts to; `stack_frames` parses V8's lines.
  - `marley_browser/src/observe.rs`: `SECRET_NAMES` is `pub(crate)`.
  - `marley_workbench/src/browser.rs`: `MapCache` (each map loaded once per pick, now shared by
    the listeners and the component), `original_in`, `original_positions` over the cache,
    `stack_source` (the first non-React frame whose original source is outside `node_modules`,
    or, without a map, whose URL is), and `pick_sources` takes the stack's source and sets the
    component's file with `find_source`, for a debug source too.
  - `marley_workbench/src/browser_tools.rs`: `pick_for_agents`, from which `browser_pick` and
    `browser_picks` answer.
  - `marley_mcp/src/registry.rs`: both descriptions, and `element_context_properties` for the
    bundle's new fields in `browser_pick`'s schema.
  - `script/e2e/browser-fixture.sh`: `mcp_agent pick` prints the new fields; `pick-json <id>
    <file>` saves the answer.
- **Deviations from the plan, and why:**
  - D2's value patterns pass over `type`, `autocomplete`, `inputmode`, `role`, `id`, `name`, `for`
    and `class`, which name things rather than hold data: otherwise `type="password"` reads
    `[redacted]` and the agent loses what the field is.
  - URL attributes: a relative URL keeps its form, cut at `?` or `#`; a URL with a user or a
    password loses them and its query; `data:` becomes `data:…`; any scheme but http, https,
    file, about, mailto and tel reads `[redacted]` (a `javascript:` URL holds code).
  - The copy for agents also redacts the blockers and each listener's `on` (both name the
    page's ids and classes), and passes each listener's script URL through `redact_url`: nothing
    did, and REQ-011 covers every text from the page.
  - The clone's pass stops after 20,000 elements: each element takes four characters at least,
    so none past them can start within the trip cap, and a huge subtree costs no more.
  - Each cut in the page steps back from a lone surrogate, and each text is made well formed: a
    lone surrogate in the answer fails serde_json's parse of the whole CDP message, so the pick.
  - `pick_schemas` passed clippy's 100 lines, so the new properties are a function of their own.
- **Review** (against each REQ and the security line):
  - Field values: `input` values but buttons', `textarea` text, the selection while a field has
    the focus, and the element's own text (a button's label only, as before) stay out.
  - Order: every text an agent gets is redacted whole before its cut
    (PR-claude-redact-the-whole-text-before-cutting-it-001), and the summary is made from the
    redacted bundle.
  - The two findings above (the work bound; the blockers and listener hosts) were fixed here.
  - No entity is updated while it is updated: the mapping runs in the pick's task, and
    `pick_sources` updates the hub alone.
  - REQ-009: locators, role, name, listeners, blockers, box and crop are read as before; they
    change for agents only where a redaction rule matches.
- **Checks:** `cargo check`; `cargo fmt`; `cargo clippy -p marley_browser -p marley_workbench -p
  marley_mcp --all-targets -- -D warnings` clean; `cargo dylint` on the three, nothing in a
  Marley crate; `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` on the three, clean
  (L-claude-504); `node --check` on the three page functions.

## Phase 3 — Test
- **The scenario**, `script/e2e/518-fuller-pick-bundle.sh` (`compositor sway`, offline Chromium):
  five pages in five Browser tabs, seven picks, each read by the stand-in agent through
  `browser_pick` and saved as JSON, and sixteen checks on the saved answers. The coordinates were
  measured on a first run with the checks made soft (a copy whose `expect` calls were renamed,
  since the runner sources the scenario before it defines `expect`). The React builds come from
  `Ignibyte-Marketing-Site` (React 18.3.1's UMD files) and `scorchkit_home` (React 19.2.8 and
  esbuild), or from `E2E_REACT18_MODULES` and `E2E_REACT19_MODULES`.
- **The selection (REQ-008): a double click, not a drag.** The plan selected the paragraph with a
  drag. The first runs and four probes showed that a drag's selection in a Browser tab does not
  outlast the release: a press and two long moves select nothing; steps of 60 px with pauses grow
  the selection in the page's own log while the button is down, and `browser_look` reads none
  right after the release. That is filed as TICKET-580, at the top of the Queue. The scenario
  double-clicks "sentence" (page x 138 to 217, measured from a shot) as #489 does, and the pick's
  `selected_text` is `sentence`.
- **The run** (`518-run2.log`, the last before the gate): 16 checks pass, none fail.
  - React 18 (REQ-004): chain `Card`, `SaveButton`; `debug source`, `src/SaveButton.jsx` line 4
    column 5.
  - React 19 (REQ-005): chain `Panel`, `SaveButton19`; `debug stack`, through the bundle's inline
    map, `src/SaveButton19.jsx` line 4 column 5.
  - The secrets form (REQ-001, REQ-006, REQ-007): no `<script>`; `data-api-key="[redacted]"`; no
    `value=` (the hidden CSRF field's and the typed e-mail's gone); `href` without its query, the
    access token's link `[redacted]`, `action="/submit"`; no component.
  - The Go button (REQ-002): `padding` `8px 16px`, `color` `rgb(10, 20, 30)`, sixteen styles.
  - The token button (REQ-011): the summary, the name and the HTML read `[redacted: github
    token]`, and so does `browser_picks`.
  - The long list (REQ-001, REQ-010): the HTML is 4,096 characters and ` (truncated)`; no piece
    of the token that straddled the 4,096th character is in any saved answer.
  - The list item (REQ-003, REQ-008): `selected_text` `sentence`; `nearby_text` `First item`,
    `Third item`.
  - No fake secret, and no 12-character piece of one, in the seven saved answers or the list;
    nothing reached the system browser (the leak log is empty).
- **The shots**, read:
  - `518-01-react18-pick`: the Card with its Save 18 button; the tray's row "Pick 1 button
    "Save 18"" with React's root listener (`abort react-dom.development.js:6461`); the rail's
    React 18 row counts one pick.
  - `518-02-react19-pick`: the same for Save 19, the listener in `react-dom-client.development.js`.
  - `518-03-secrets-pick`: the typed e-mail in its field, both links, the token button and Go;
    the tray's row names the form with the page's own text, the fake token in it. The tray is the
    user's and the page shows the token anyway; agents get the redacted copy (REQ-011).
  - `518-04-selection-pick`: "sentence" still selected after the pick of Second item; every tab's
    row in the rail with its pick count (Secrets 3).
- **Focus:** each run printed "hyprland: 0 Marley windows before the run, 0 after; the run added
  no rule and did not reload it"; the scenarios ran in their own headless sway.
- **Seen, not in scope** (in `marley_workbench.md`'s known limits):
  - The tray's Send types a pick's summary as the page shows it, secrets included. #516's
    redaction covers Marley's tools, and a Send lands at the user's prompt before Enter, as
    #549's selection does.
  - On a React page the tray shows React's root listener (`abort`), as #497 recorded; the
    component and its file reach agents only, as the spec's scope keeps them.
- **REQ-009:** `496-element-picker.sh` and `497-pick-source.sh` rerun. Their logs show the
  new fields next to the old ones, and their shots match what they showed before. #496: the hover
  overlay, the staged row, the typed line in the terminal, pick mode, and after Escape a click
  that reaches the page ("saved"). #497: `src/app.ts:2` in the tray, the file open at line 2,
  and `plain.js:2` opening nothing.
- **The golden set**, with `518-fuller-pick-bundle.sh` added: `just regress` ran all 26 against
  the debug build, and all 26 passed (#518's in 117 s).
- **The gate:** `script/gates.sh --diff` (`gate-518.log` in the session's scratchpad): 16 passed,
  0 failed, `GATE GREEN [diff]`, and the receipt written for the tree it ran on. cargo-audit
  printed its standing notice (proc-macro-error unmaintained) and passed.
- **Verdict:** Phase 3 PASS. Nothing pre-existing was excluded.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: "A fuller pick for agents"); the plan's row B3c in
  `docs/marley/three-prong-plan.md`; `docs/marley_architecture/marley_browser.md` (a section,
  "The fuller bundle"); `docs/marley_architecture/marley_workbench.md` (the component's source
  beside the listeners', `pick_for_agents` under the browser tools, and two known limits: the
  tray's Send, and a drag's selection, #580). No Zed path was touched, so no ledger row.
- **Knowledge appended:** F-claude-518-a-drag-in-a-browser-tab-leaves-no-selection-001 (open as
  TICKET-580), L-claude-518-a-lone-surrogate-fails-the-whole-cdp-message-001,
  L-claude-518-select-with-a-double-click-in-a-scenario-001,
  AD-claude-518-a-pick-carries-its-html-styles-texts-and-component-001. No new prevention rule:
  PR-claude-redact-the-whole-text-before-cutting-it-001 already covers the order the design keeps.
- **Brain:** consultation ef1154d6c93740a38d4fb5ade5e604ef closed with
  `decisions/a-pick-carries-its-html-styles-texts-and-react-component-518`, follow-up by
  2026-10-26 (whether agents use the component field, and whether the tray should show it).
- **Filed:** TICKET-580, at the top of the Queue.
- **Closed and archived:** the ticket in `docs/planning/tickets/closed/`, this pair in
  `docs/planning/pipeline/completed/`.
