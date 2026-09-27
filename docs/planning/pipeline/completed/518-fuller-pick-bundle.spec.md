---
pipeline_id: 51a0c1e9-93e5-4f5d-9887-4f9d385745a9
ticket: docs/planning/tickets/closed/TICKET-518-fuller-pick-bundle.md
status: Phase 4 — Complete PASS
title: "A fuller pick bundle: bounded HTML, computed styles, nearby text, and the React component with its source"
type: feature
slice: prong 3 (after wave 2), the Orca survey's item 4; before #505
references: [docs/orca_architecture/README.md, docs/orca_architecture/03-browser-and-design-mode.md, docs/planning/pipeline/completed/496-element-picker.spec.md, docs/planning/pipeline/completed/497-pick-source.spec.md, docs/planning/tickets/open/TICKET-516-secret-redaction-for-agents.md]
---

## Title
The pick bundle carries what an agent needs to change an element and to find its code: its HTML,
its computed styles, the text around it, and on a React dev build the components around it and
the file and line it was written at, with nothing secret in any of it.

## Scope
### In
- **The page's read (`crates/marley_browser/src/pick.rs`, `DESCRIBE`).** In the same call that
  reads the locators, box and blockers:
  - `html`: the element cloned; `<script>` elements removed; field values removed (the `value`
    attribute of every `input` but button, submit, reset and image types, and every `textarea`'s
    text); each attribute whose name looks secret, or whose value holds a secret-looking pattern,
    set to `[redacted]`; `href`, `src`, `action`, `formaction`, `poster`, `cite`, `data`, `ping`
    and each `srcset` candidate without query or fragment; serialized, and capped for the trip
    at 65,536 characters (`browser_pick` cuts it to its 4,096 after redaction, D5).
  - `styles`: sixteen computed properties, Orca's list: `display`, `position`, `width`,
    `height`, `margin`, `padding`, `color`, `background-color`, `border`, `border-radius`,
    `font-family`, `font-size`, `font-weight`, `line-height`, `text-align`, `z-index`.
  - `nearby_text`: up to ten texts of the element's siblings, before and after in turn;
    `selected_text`: the page's selection when it has one. Each is capped for the trip at 4,096
    characters and cut to its budget (200, 500) by `browser_pick` after redaction.
  - `react`: the element's `__reactFiber$…` (or `__reactInternalInstance$…`) fiber walked up
    `.return` for at most 35 levels: up to six component names (the `displayName` or `name` of
    `type`, `type.render` or `type.type`) with Orca's skip list, the first `_debugSource` on a
    fiber or its `_debugOwner`, and the host fiber's `_debugStack.stack` (4,000 characters) when
    there is no `_debugSource`.
- **The bundle (Rust).** `PickBundle` gains `html`, `styles`, `nearby_text`, `selected_text` and
  `component: Option<Component { chain, source }>`. Every string is clamped to its trip cap again
  on arrival. A React 19 stack is parsed into V8 frames (`at <name> (<url>:<line>:<column>)`); after
  the pick is staged, as the listeners' sources are (#497), each frame is mapped through its
  script's source map in turn, and the first whose original source is outside `node_modules/` and
  whose function is not React's own (`jsxDEV`, `jsx`, `jsxs`, `createElement`,
  `react_stack_bottom_frame`) is the source. The source is looked up in the project with #497's
  `find_source`.
- **For agents.** `browser_pick` returns the new fields, with its output schema and description in
  `crates/marley_mcp/src/registry.rs` extended. `browser_pick` and `browser_picks` hand agents a
  copy of the pick made for them. Every field the page's text reaches (the page's `title`, the
  `caption`, and the bundle's `name`, `text`, `html`, `nearby_text`, `selected_text` and each
  locator's value) passes through #516's `Redactor` (`crates/marley_mcp/src/redact.rs`), as
  #516's D2 asks of every tool that hands agents page or terminal text: each whole, and only then
  cut to its budget (PR-claude-redact-the-whole-text-before-cutting-it-001). The `summary` is
  made again from the redacted bundle, so its 60-character cut also comes after the redaction.
- **The fixture.** `mcp_agent pick` in `script/e2e/browser-fixture.sh` prints the new fields, and
  `mcp_agent pick-json <id> <file>` saves the whole answer for the scenario's greps.

### Out (explicitly deferred)
- Vue (`__vueParentComponent.type.__file`) and Svelte (`__svelte_meta.loc`), the survey's cheap
  extras: no project on the box has a Vue or Svelte build to check them against.
- Showing the component or its file in the tray, or opening it from there: the tray keeps
  #497's listener place until agents have used the field.
- Orca's separate allowlisted `attributes` map: the HTML carries the element's attributes.
- Redacting visible text. Orca's main process redacts any metadata string that contains
  "password", so a "Reset password" label disappears; visible text is on the screen and in the
  crop already.
- A second redaction pass over the HTML in Rust (see D3).

## Reference (§20)
Orca's Grab payload (report 03 §2.4), whose budgets, style list, component-chain rules and
redaction Marley takes, with three differences: React 19's source comes from `_debugStack` and
the source maps, where Orca's source field is empty on React 19; field values are dropped from the
HTML, which Orca's clone keeps when React has synced a typed value into the `value` attribute;
and visible text is left alone. Chromium DevTools shows the same facts in its Elements panel
(Copy outerHTML, the Computed pane), and React DevTools shows the component tree and its source
link. Upstream Zed: N/A, no browser. Warp: N/A; the once-over
(`docs/planning/design-notes/warp-once-over-2026-09-25.md`) rules its browser features out as
covered by Marley's own, #488 to #499.

### Prior art
- **Behavior maps and reports.** `docs/orca_architecture/README.md` item 4 ("A fuller pick") and
  the "New tickets these imply" line that puts this before #505; report 03 §2.4 (the payload,
  its redaction and its weaknesses, among them `_debugSource` missing on React 19) and item 1 (the
  landing list for `pick.rs`). Orca files read at `1c2cf120e3`:
  `src/main/browser/grab-guest-react-script.ts` (`getFiberFromElement`, `shouldSkipReactName`,
  the 35-level walk collecting six names, `_debugSource` or `_debugOwner._debugSource`,
  `cleanSourcePath`); `grab-guest-foundation-script.ts` (`BUDGET`, `SECRET_PATTERNS`,
  `STYLE_PROPS`, `sanitizeUrl`); `grab-guest-content-script.ts` (`getHtmlSnippet`: clone, drop
  scripts, cap 4,096; `getSafeAttributes`); `grab-guest-element-context-script.ts`
  (`getComputedStyleSubset`, `getNearbyText`); `src/main/browser/browser-grab-payload.ts`
  (`clampGrabPayload`: the main process clamps every field and re-redacts attributes, and only
  clamps the HTML); `src/shared/browser-grab-types.ts`.
- **Published material.** React's own dev builds, read on the box: React 18.3.1's
  `react.development.js` reads `config.__source` into `element._source`, which `react-dom`
  copies to `fiber._debugSource`; React 19.2.8's `react-jsx-dev-runtime.development.js` `jsxDEV`
  makes `Error("react-stack-top-frame")` for each element while fewer than 10,000 have been made
  since the counter was last reset (`recentlyCreatedOwnerStacks`, which `react-dom` resets when
  it prepares a render at least a second after the last reset), and `createFiberFromElement`
  copies `_debugStack` to the fiber; `react-dom` keys the fiber on the
  node as `"__reactFiber$" + randomKey`. V8's stack format counts lines and columns from 1.
  Source Map v3 (ECMA-426).
- **Code we already ship.** `pick.rs`: `DESCRIBE`, `INTERACTIVE_ANCESTOR`, `capture_pick`,
  `call_on` (`Runtime.callFunctionOn` on the object `DOM.resolveNode` gave, in the page's main
  world, where React's expando properties live; an isolated world cannot see them).
  `browser.rs`: `pick_requested`, `pick_captured`, `pick_sources`, `original_positions`,
  `load_map`, `find_source`. `source_map.rs`: `SourceMap::original`, `map_location`,
  `source_path`, which already reads `webpack-internal:///./src/…` and Next.js layer prefixes as
  path components, so Orca's `cleanSourcePath` is not needed for the lookup. `observe.rs`:
  `SECRET_NAMES`, `redact_url`. `browser_tools.rs` `pick` (serializes the whole `Pick`),
  `registry.rs` `pick_schemas`. #516's `Redactor::redact`. Considered and not used: `html5ever`
  (a workspace dependency) and `lol_html` (in the lock through `merman-core`) for a Rust-side pass
  over the HTML (D3).

## UI proof
N/A — no UI delta: the tray, the pick line and the toolbar are unchanged, and the new fields reach
agents through `browser_pick`. The e2e run is a scenario rather than `just shot`, since the fields
come from real picks: `script/e2e/518-fuller-pick-bundle.sh` (`compositor sway`, offline Chromium)
picks on a React 18 page, a React 19 page and a page full of fake secrets, and the stand-in
agent's `browser_pick` answers, printed and saved, are the proof. Shots `518-01-react18-pick`,
`518-02-react19-pick` and `518-03-secrets-pick` record each staged pick for reading.

## Locked-In Decisions
- D1 — One call, in the page's main world: the HTML, styles, texts and the React read join
  `DESCRIBE`'s `Runtime.callFunctionOn` on the element, since React keeps its fiber as an
  expando of the page's own world.
- D2 — Redaction happens on a clone in the page, where the DOM gives each attribute exactly.
  Scripts go; field values go, because React writes a controlled input's value into its `value`
  attribute and #496 promised that a pick never records typing; an attribute whose name holds one
  of `observe.rs`'s `SECRET_NAMES` (token, key, secret, password, auth, code, sig, session), or
  whose value holds one of Orca's patterns (`access_token`, `auth_token`, `api_key`, `apikey`,
  `client_secret`, `oauth_state`, `x-amz-`, `session_id`, `sessionid`, `csrf`, `secret`,
  `password`, `passwd`), reads `[redacted]`; URL attributes lose query and fragment. Visible text
  is not redacted.
- D3 — Rust clamps every field to its trip cap again and never parses the HTML: a page that
  rewrites the functions Marley runs in it can misreport only its own content, and the clone's
  pass sees the live attributes. What agents receive also passes #516's `Redactor`, which catches
  token shapes in visible text, over each whole field before the field is cut to its budget, so
  a token that straddles a cut is whole when the rules read it.
- D4 — The component's source is `_debugSource` when React gives one (18 and older), as React
  gives it; else the first frame of `_debugStack` outside React, found by mapping frames through
  their source maps rather than by URL, because a bundle serves React and the app from one file.
  The mapping runs after the pick is staged, beside the listeners', so the tray shows the pick at
  once.
- D5 — The budgets are Orca's: HTML 4,096 characters, text 200, ten nearby texts of 200, the
  selection 500, six component names within 35 levels, a source of 500 characters. The text
  budgets apply in `browser_pick`, after the redaction; until then the page and the bundle hold
  each text up to a trip cap (the HTML 65,536 characters, every other text 4,096), which only
  bounds what crosses the socket.
- D6 — The pick line stays one short reference and the fields stay behind `browser_pick`, so an
  agent reads 4 KB of HTML only when it asks (report 03 §4).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user picks an element, `browser_pick` shall return its HTML without `<script>` elements, at most 4,096 characters, marked when cut. | The run log: the secrets pick (no script), the long list's pick (4,096 characters and the mark) |
| REQ-002 | WHEN the user picks an element, `browser_pick` shall return its sixteen computed styles as the page computed them. | The run log: the Go button's `padding` and `color` as its style sets them |
| REQ-003 | WHEN the user picks an element, `browser_pick` shall return up to ten of its siblings' texts. | The run log: the list item's pick |
| REQ-004 | WHERE the page runs a React 18 dev build, `browser_pick` shall return the component chain around the element, outermost first, and the file, line and column `_debugSource` gives, with the file in the project. | The run log: the React 18 pick |
| REQ-005 | WHERE the page runs a React 19 dev build, `browser_pick` shall return the chain and the file and line of the first `_debugStack` frame outside React, through its source map, with the file in the project. | The run log: the React 19 pick |
| REQ-006 | The bundle shall hold no field value, no URL query string or fragment, and `[redacted]` in place of each secret-looking attribute value. | The run log: a grep of the saved answers for the run's fake tokens and the typed e-mail finds none |
| REQ-010 | WHEN a token-shaped secret straddles a field's budget, `browser_pick` shall hand agents no part of it. | The run log: the long list's pick, whose fake token spans the 4,096th character, holds no piece of the token |
| REQ-011 | WHEN an agent lists or reads picks, the page's title, the caption, the summary and every text from the page shall reach it through #516's redaction. | The run log: `mcp_agent picks` and the saved answer for a button labelled with a fake token show the redaction's marker and no piece of the token |
| REQ-007 | WHERE the page runs no React, the bundle shall have no component. | The run log: the secrets pick |
| REQ-008 | WHEN the page has a selection at the pick, `browser_pick` shall return its text. | The run log: the list item's pick after a drag across the sentence that ends inside the text |
| REQ-009 | WHEN the user picks an element, its locators, role and name, listeners, blockers, box and crop shall read as they did before this change. | #496's and #497's scenarios rerun, their logs and shots as before |

## Phase Plan
- **P1 Plan** — promote this pair; check that #516 has landed (D3's `Redactor` call waits for it);
  confirm that the pick's read runs in the page's main world, which sees `__reactFiber$…`.
- **P2 Code** — `DESCRIBE`'s additions and the React read, the bundle's fields and clamps, the
  stack parser and its mapping beside the listeners', the registry, the fixture's printing; fmt and
  clippy clean; a review of the diff (every budget, every redaction rule, no field value).
- **P3 Test** — write and run the scenario and read the shots and the log; rerun #496's and #497's
  scenarios; `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, the prong's slice status and `docs/marley_architecture/
  marley_browser.md` (§21), ledger capture (§19), close, archive, commit.
