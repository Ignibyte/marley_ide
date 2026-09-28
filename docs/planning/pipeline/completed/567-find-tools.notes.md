# browser_find and terminal_find for agents — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-567-find-tools.md
- **Pipeline spec:** 567-find-tools.spec.md

## Phase 1 — Plan
- **Request:** the Jev note's use 2 (2026-09-25): "a choice over a page's refs or a block's
  tagged lines, with `none`, and a `present` noul", so "the agent gets the top three, or 'not
  sure: read the snapshot', instead of a 7,500-token snapshot"; with Jev off or down "the tools
  are not listed". Chad approved the use on 2026-09-26 with the rules quoted in #565's D1 to
  D4; "local first" gives the tools a match by words before any call.
- **Classification / tier:** feature, prong 2 (the server's tools) with prong 3 (the Browser
  tab). Marley crates: `marley_mcp`, `marley_system_one`, `marley_workbench`. One Zed path with a
  row: `marley_page.rs`. Depends on #565. Size S each, as the note sizes them.
- **Recall (§18.3):**
  - AD-claude-491-marleys-mcp-server-runs-in-the-app-behind-a-stdio-bridge-001: wire names are
    `family_verb`; a tool whose answer is the app's comes back deferred and is answered on the
    main thread. Both tools follow their families' routes.
  - AD-claude-492-agents-drive-the-browser-tab-through-the-mcp-server-001: the refs are the
    snapshot's, kept in the hub per tab, and a stale ref says "take a new snapshot"; `browser_find`
    keeps the same currency (D3).
  - AD-claude-516-redact-at-the-tool-boundary-on-by-default-001: a block's output is masked
    before anything leaves; `terminal_find` masks before tagging.
  - L-claude-516-fake-secrets-are-put-together-at-run-time-001 (REQ-009's fixture).
  - The 491 completed notes: the stand-in agent's shape and the bridge's 40-second wait; the
    508 queued notes record `APP_CALL_TIMEOUT_SECONDS` at 30, which bounds the deadline (D8).
  - Brain: not consulted in this drafting session (the brief is read-only); promotion asks.
- **Discovery** (2026-09-26, at the working tree of `ca70b6488d`):
  - `crates/marley_mcp/src/registry.rs`: `Family` (12-21), `is_served` (38: `Terminal` and
    `Browser`), `ToolSpec` (46-57: `family`, `verb`, `tier`, `grant_class`, `description`),
    `REGISTRY` (78-218), `browser_read` (221), `lookup` (251), `tools_list` (264-279: filters
    by `is_served`), `tool_schemas` (285-303, exhaustive over `Family`), `browser_schemas`
    (333-346, a `_` arm to the write schemas), `browser_arguments` (357), `snapshot_schemas`
    (421-440: `full`; the answer's `tab`, `snapshot`, `refs`, `cut`), `terminal_schemas`
    (811-817), `terminal_argument_schema` (820), `terminal_read_schemas` (916-948). The tests at
    963 pin the registry's names and count and will change with the rows.
  - `crates/marley_mcp/src/dispatch.rs`: `handle_message` (18: `tools/list` at 28 calls
    `registry::tools_list()` with no context), `tools_call` (133-174: `lookup`, the permission
    check, the deferred arm for the app's families). `crates/marley_mcp/src/marley_mcp.rs`:
    `RequestCtx { snapshot, grants, surface_index }`, `PendingCall`, `APP_CALL_TIMEOUT_SECONDS`.
  - `crates/marley_workbench/src/browser_tools.rs`: `answer` (53-61: spawns `run`), `run`
    (63-114: the verb match; `page_of` for the tab), `take_snapshot` (318-383: the main tree, the
    frames, the iframes, `snapshot::render(&trees, full)`, `hub.set_refs(tab, refs)`),
    `ref_target` (678), `target_point` (690: a `ref` argument resolves through the hub).
  - `crates/marley_browser/src/snapshot.rs`: `MAX_CHARS` (15), `RefTarget` (115-131), `describe`
    (133), `Snapshot` (144), `render` (155), the ref line (334: `[ref=eN]`).
  - `crates/marley_workbench/src/browser.rs`: `set_refs` (1593), `ref_target` (1601).
  - `crates/marley_workbench/src/mcp.rs`: `MAX_READ_LINES` (48), `MAX_READ_BYTES` (51), `answer`
    (317-329: a synchronous match; `terminal_find` needs a spawned path like the browser's),
    `terminal_with_id` (364), `terminal_argument` (373), `terminal_read` (491-526: the block,
    `block_output`, masking whole before `tail`), `tail` (530-545).
  - `crates/terminal/src/terminal.rs`: `blocks` (1845), `block_output` (1859),
    `block_output_kept` (1869).
  - `script/e2e/browser-fixture.sh`: `mcp_agent` (76), `write_mcp_agent` (205-290: the client,
    `tool`, `find_ref` by role and name over the snapshot's lines, the commands' `main`).
  - #565's spec: `UseSpec`, `StateBuilder`, `ask`, `Mode`, the replay rows with `match`.
  - TypeSafe's cookbook and API reference (the spec's prior art), read 2026-09-26.
- **Decisions:** D1 to D8 in the spec.

### Promotion (2026-09-27)
- **Checklist** (no task tool): pick ✓; pre-flight ✓ (gate, e2e, hooks, README marker; cargo busy
  with #566's release install, so no cargo ran and no crate or scenario was edited); recall ✓;
  promote ✓ (the pair to `active/`, the backlog row removed, the ticket in-progress); the seams
  re-verified (Explore, at `e47cbc7b9d`) ✓; spec and design updated ✓.
- **Recall, added:** AD-claude-524-outside-clients-reach-a-list-of-browser-tools-by-name-001 (a
  client calls only an explicit list, so a tool added later reaches no client until it is named
  there); PR-claude-derive-wire-list-from-single-registry-001 (the listing derives from the
  registry); AD-claude-565 and L-claude-565-jevs-answer-shape-as-recorded-001 (a choice answers
  `choice`, `confidence` and `probabilities`); L-claude-566-a-use-whose-questions-vary-needs-a-set-per-shape-001
  and AD-claude-566 (the parts' pattern: fixed questions, the varying text in the state, a set per
  shape, statics); F-claude-566-a-cut-that-kept-a-messages-end-could-part-a-secret-from-its-name-001
  (redact before any cut). The brain (consultation c25b649370f145dab192aa6c1d7aef14): nothing on
  this seam; a duplicate consultation asked in the same minute was closed with no decision.
- **What the code says now** (the Explore report, 2026-09-27; the drafted line numbers are stale):
  - `registry.rs`: `Family` has `Ports` (#521) and `is_served` serves `Terminal`, `Browser` and
    `Ports`; `ToolSpec` has no feature field; `REGISTRY` holds 26 rows; the listing is
    `tools_list_for(principal)` (#524), which filters on `is_served` and `clients::permits`;
    `tool_schemas` is exhaustive over `Family`, but `browser_schemas` ends in `_ =>
    browser_write_schemas(verb)` and `terminal_schemas` in `_ => terminal_list_schemas()`, so a new
    verb with no arm gets another tool's schema. The registry's unit tests are stale (they pin 5
    tools) and no gate runs them; they must still compile.
  - `dispatch.rs`: `tools/list` calls `tools_list_for(ctx.principal)`; `tools_call` runs `lookup`,
    `permits`, `permission::decide`, then defers `Terminal`, `Browser` and `Ports`.
    `initialize_result` advertises `tools.listChanged`, but the server never sends
    `notifications/tools/list_changed` on its own; the bridge sends one only when Marley comes up
    or goes down, and Zed's agent reloads its tools on it.
  - `RequestCtx` has four fields (`snapshot`, `grants`, `surface_index`, `principal`) and nine
    literals, tests included; `transport.rs` builds it under the data lock in `serve_post`.
    `ServerData` has no setter, and the main thread never takes the server's lock (`mcp.rs`'s
    comment at 161-163): the fleet snapshot goes through a channel to a background task
    (`publish`), which is the pattern for any state the app pushes.
  - `browser_tools.rs`: `run`'s third match holds the page tools, with `other => Err(…)`;
    `take_snapshot` (941-1005) walks the trees inline and ends in `snapshot::render` and
    `hub.set_refs`; `page_of` resolves `tab` (a named tab, else the caller's project's focused tab
    (#574), else the focused one). A tab's project is `hub.project_of(target)`, a `BrowserProject
    { key, name, paths, host }` whose `paths` are the group's main worktree paths.
  - `snapshot.rs`: a ref is `RefTarget { id, session, frame_id, backend_node_id, role, name }`, its
    name cleaned (`"` as `”`, cut at 100 characters) and not redacted; `full` numbers refs
    differently.
  - `mcp.rs`: `answer` sends `browser_*` to `browser_tools::answer` and `ports_list` to a spawned
    route (`cx.spawn`, then `call.answer`), the precedent for an async terminal-side tool;
    `terminal_argument` is gone, and `terminal_of(arguments, caller, cx)` defaults to the caller's
    terminal (#520); `terminal_read` picks the block by index, masks the whole output, then
    `tail` keeps 2,000 lines and 256 KiB.
  - #565's types are all `&'static`: `QuestionSet`, `Question` (a `Choice`'s options too), `UseSpec`,
    and `ask(spec: &'static UseSpec, …)`; there is no `question` module. A `Reading` keeps a
    choice's top option and confidence, not its per-option probabilities, which live only in the
    parsed `Answers` and as raw JSON in the row. The noul band is 0.35 to 0.65, not the cookbook's
    0.7 and 0.35. A state is `label: value` lines, each text value cut at 300 characters, and
    `Asking`'s labels are `&'static str`. `use_mode` is off while the layer is off.
  - The stand-in agent has a `tools` command, and `click-on` clicks by role and name after a fresh
    `browser_snapshot`, which renumbers the refs; there is no raw-ref click. The golden set pins
    outside clients' counts (524: 10 and 18; 584: 18), which a find tool outside the client lists
    leaves alone.
  - `clippy.toml` bans neither `LazyLock` nor `Box::leak`; the Marley manifests deny
    `declare_interior_mutable_const`, so a lazy table is a `static`, as `redact.rs`'s rules are.

### Design
- **Changed at promotion** (the seams re-read on 2026-09-27; each item overrides the drafted design
  after it):
  - **The sets are a static table, by the number of items.** #565's questions are compiled in and
    a caller never supplies its own (#566's lesson), so the query goes in the state as its `query`
    line and the questions stay fixed. `marley_system_one::find_set(items)` answers one of 254
    sets, `find_1/1` to `find_254/1`: a choice `which` ("Which item in the state matches its
    `query` line?") over the options `none` then `1` to `N`, and the noul `present` ("Does any
    item match the `query` line?"). The options are one table, `none` first, so every set's
    options are a prefix of it; the names and meanings are `String`s in a `static LazyLock`, which
    gives `&'static str` without a leak. The items are the state's lines `1: …` to `N: …`, masked
    and cut like any text, their labels from the same table.
  - **One request per window.** A set holds one choice over its window, so a page's refs or a
    block's lines go in windows of at most 254 items, a request each, asked together. The block's
    windows stop at 24,000 tokens (bytes over four), the newest lines kept. Found when a window's
    `present` reads yes, absent when every window's reads no, unsure otherwise; the candidates are
    the top three options by probability across windows, `none` aside.
  - **The layer's band, not the cookbook's cutoffs** (D5 changed): `present` reads found above
    0.65, absent below 0.35, and no signal between, as every use of the layer reads a noul; a
    `which` counts at confidence 0.5 or more and never for `none`. One reading drives the tool and
    the Decisions row.
  - **`Asked` gains the parsed `answers`**, which the candidates' probabilities need: `Draft` keeps
    them when a provider or the replay answers, and `finish` hands them on.
  - **`ask` and `record` take a `UseSpec` by value.** It is `Copy`, and its references are
    `&'static`; the find tools make theirs at call time (`browser_find` or `terminal_find`, the
    window's set, 3 s) rather than a static table of 508.
  - **The listing.** A tool is conditional by name: `registry::CONDITIONAL_TOOLS` holds
    `browser_find` and `terminal_find`, each on while its use's mode is not `off` (with the layer
    on, as `use_mode` reads it). `RequestCtx` gains `enabled: &BTreeSet<String>`, from a new
    `ServerData.enabled`; `tools_list_for(principal, enabled)` leaves an off tool out, and
    `tools_call` refuses one by name: "browser_find is off: turn System One on and set
    marley.system_one.uses.browser_find to shadow, suggest or act". The app pushes the set as it
    pushes the fleet snapshot: an observer of the `SystemOne` global sends it down a channel to a
    background task that takes the lock (`transport::set_enabled`). A running client keeps the
    list it read until it lists again; the guide says to reconnect the MCP server after turning a
    tool on (Out: `list_changed`).
  - **Schemas.** `browser_schemas` and `terminal_schemas` gain `find` arms (`find_schemas`,
    `terminal_find_schemas`) ahead of the arms that catch every other verb. Neither tool joins `CLIENT_READ_TOOLS`:
    outside clients do not spend the user's budget (Out).
  - **`browser_find`** runs in `browser_tools::run`'s page match: the snapshot through a helper
    both it and `browser_snapshot` call (`take_snapshot`'s walk moves out), the refs kept in the hub
    (D3), the items `role "name"` in ref order; the tab's project from `hub.project_of(target)`
    (`paths`, and `host` for a remote one).
  - **`terminal_find`** runs on `ports_list`'s spawned route in `mcp.rs`: `terminal_of` (the caller's
    terminal by default) and `block` as `terminal_read` takes them, the output masked whole
    (`model_redactor` for the state, `agent_redactor` for the answer) before `tail`; `line` is the
    line's number in the block's output. The project from the terminal view's workspace
    (`system_one::project_of`).
  - **The local match** (`marley_mcp::find`, pure) drops stop words (`the`, `a`, `an`, `of`, `to`,
    `for`, `on`, `at`, `where`, `which`, `what`, `is`, `are`, `was`, `did`, `does`, `do`, `with`)
    and matches whole words, case folded: an item matches when it holds every word left. A query
    with no word left matches nothing.
  - **A repeated find** of the same query over an unchanged page is refused by #565's gate as
    unchanged; the tool answers its local match with the note "asked already; nothing changed".
    The replay provider skips the gate.
  - **The stand-in agent** gains `find <query> [--tab id] [--full]`, `tfind <block> <query>` and
    `click-ref <ref>`, which clicks a ref as given with no new snapshot.
  - **The settings.** Browser Find and Terminal Find dropdowns after Stop Kind on the Marley page
    (the section at eleven items), `"browser_find": "off"` and `"terminal_find": "off"` in
    `default.json`'s `uses`, and the `uses` docstring; ledger rows 55, 58 and 61 widen.
- **`marley_mcp::find`** (pure): `words(query) -> Vec<String>` (lowercased, punctuation
  dropped, stop words left out); `Item { id, text }`; `matches(items, words) -> Vec<usize>` (an
  item holds every word as a whole word); `windows(count) -> Vec<Range<usize>>` of at most 254;
  `Local { Sure(usize), Several(Vec<usize>), Nothing }`; `CONDITIONAL_TOOLS` and the `enabled`
  filter live in `registry.rs` beside the listing. A ref's text is `role "name"`, a line's the
  line.
- **The sets** (`marley_system_one`'s root, beside the stop kind's): `FIND_WINDOW` (254),
  `find_set(items) -> &'static QuestionSet` over a `static LazyLock` table of the option names,
  their meanings, the questions and the sets (`find_N/1`), and `find_label(item) -> &'static str`
  for the state's labels.
- **The adapter** (`system_one.rs`): `ask` and `record` take a `UseSpec`; `Asked` gains
  `answers: Option<Answers>`.
- **`browser_find`** (`browser_tools.rs`, in `run`'s page match): the snapshot through the shared
  walk, `set_refs`, the items in ref order; the local match; the project from
  `hub.project_of(target)`; with the use on and the project sending text, one `ask` per window
  (a `UseSpec` named `browser_find` over the window's set, 3 s, the subject the tab and the
  query), awaited together; the answer in the spec's shape.
- **`terminal_find`** (`mcp.rs`, a spawned route like `ports_list`): `terminal_of`, the block by
  index, the output masked whole and cut by `tail`, then to 24,000 tokens from the newest line
  back; the items the lines; the project from the view's workspace; the same asks with the
  terminal and block in the subject.
- **Reading the windows** (`marley_workbench`, shared by both tools): each window's `present` and
  `which` readings, the per-option probabilities from `Asked.answers`, the candidates (the top
  three across windows, `none` aside), and the modes (D7).
- **The listing** (`marley_mcp`): `CONDITIONAL_TOOLS`, `ServerData.enabled`,
  `RequestCtx.enabled`, `tools_list_for(principal, enabled)`, the refusal in `tools_call`, and
  `transport::set_enabled(shared, enabled)`; the app's observer of the `SystemOne` global and its
  channel in `mcp.rs`.
- **The stand-in**: `find`, `tfind` and `click-ref` print the answer's JSON; `tools` already
  prints `tools/list`'s names.
- **File manifest.**
  - Marley: `crates/marley_mcp/src/find.rs` (new), `registry.rs` (the two rows, their schemas,
    `CONDITIONAL_TOOLS`, `tools_list_for`), `dispatch.rs` (the refusal, `RequestCtx.enabled` in
    its tests), `marley_mcp.rs` (`RequestCtx`), `transport.rs` (`ServerData.enabled`,
    `set_enabled`, `serve_post`'s context), `expose.rs` (its two `RequestCtx` literals);
    `crates/marley_system_one/src/marley_system_one.rs` (the find sets);
    `crates/marley_workbench/src/system_one.rs` (`UseSpec` by value, `Asked.answers`),
    `agent_events.rs` (its `ask` and `record` calls), `find.rs` (new: the windows' reading and
    the answer's shape, shared), `browser_tools.rs` (`browser_find`, the shared walk), `mcp.rs`
    (`terminal_find`, the enabled set's observer and channel).
  - Zed paths with rows: `crates/settings_ui/src/marley_page.rs` (two dropdowns),
    `crates/settings_content/src/marley.rs` (the docstring), `assets/settings/default.json`
    (`uses`).
  - Scenarios: `script/e2e/567-find-tools.sh` (new), `script/e2e/browser-fixture.sh` (`find`,
    `tfind`, `click-ref`), `script/e2e/golden` (the 567 entry).
- **Ledger rows.** `docs/marley/zed-touchpoints.md` rows 55, 58 and 61 name the two uses, before
  those files are written.

### E2E plan
Fixtures: the browser fixture's offline Chromium and `serve_site` with a page holding a "Sign in"
button that shows "Signed in" when clicked, two "Submit" buttons in two forms and a "Docs" link; a
scratch repository with `print-lines.sh` (300 numbered lines, line 212 `connect: connection
refused`, and, for REQ-009, a `ghp_` token assembled at run time on line 250); the layer on
`replay` with the repository listed and both uses in `act`; the replay file written in setup, its
rows keyed by `find_N/1` for the window's item count and a `match` on the state's `query` line.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001, REQ-004 | `mcp_agent navigate` to the page; `mcp_agent find "sign in"`; `mcp_agent click-ref <ref>` | `567-01-clicked`: the page says "Signed in"; the answer `source: rules`, `sure: true`; the day's file has no find row |
| REQ-002 | `mcp_agent find "submit"` (two local matches; `find_2/1`: `which` `2` at 0.8, `present` 0.9) | the run log: `source: model`, the second submit's ref, 0.8 |
| REQ-003 | `mcp_agent find "coupon code"` (no match; `find_N/1` over the page's refs: `present` 0.05) | the run log: `sure: false`, `next: browser_snapshot`, absent |
| REQ-005 | `./print-lines.sh` in the terminal; `mcp_agent tfind <block> "connection refused"`; then `"where did the server fail to connect"` (two windows, 254 and 46 lines; `find_254/1`: `which` `212` at 0.85, `present` 0.92) | the run log: line 212 by rules, then by the replay |
| REQ-009 | the second `tfind`'s row | the day's file: the state holds the token masked and not as printed |
| REQ-006 | `browser_find` in `shadow`; `mcp_agent find "submit"` | the run log: the two local matches, `sure: false`; the day's file: the model's row, mode `shadow` |
| REQ-008 | `projects []`; `mcp_agent find "submit"` | the run log: the local matches and the note; no new row |
| REQ-007 | both uses `off`; `mcp_agent tools`; `mcp_agent find "submit"` | `567-02-off-not-listed`; the run log: the list without either tool, the refusal naming the setting |
| (the settings) | the Marley settings page, scrolled to System One | `567-03-settings`: Browser Find and Terminal Find after Stop Kind |
| REQ-010 | Test's gate and regression runs, 491, 492, 524, 565, 566 and 584 again | the exit codes |

What no scenario reaches: a real model's ranking (the replay rows stand in), and #565's gate
refusing a repeated find, which the replay provider skips.

### Risks
- The time budget: `settled` can wait 20 s and `ATTACH_WAIT` 5 s before the tree reads, and the
  model's 3 s come after, inside the app's 30 s. A slow page leaves less; the deadline stands.
- An item is its role and name alone, so elements that share a name (two "Submit" buttons) look
  alike to the model. Context for same-named elements is Out.
- A repeated find of an unchanged page is refused as unchanged; the local match and the note
  answer it.
- A running client keeps the tool list it read: turning a tool on reaches a Claude Code session
  when it lists again, which the guide says to do by reconnecting the MCP server.
- The windows multiply calls for a long block: at most five at the 24,000-token cut.
- The refs move when the page changes between the find and the click; the same rule as today
  ("take a new snapshot") applies.
- Token estimates by bytes over four: a block of wide characters underestimates; the cut keeps a
  margin under the 32k the note records for the state.

## Phase 2 — Code
- **Checklist** (no task tool): the ledger rows 55, 58 and 61 first ✓; `marley_mcp` (`find.rs`,
  the rows, the schemas, `CONDITIONAL_TOOLS`, the enabled set through `ServerData` and
  `RequestCtx`, the refusal) ✓; the find sets ✓; the adapter (`UseSpec` by value,
  `Asked.answers`) ✓; `marley_workbench::find` ✓; `browser_find` and the shared walk ✓;
  `terminal_find` and the enabled set's observer ✓; the settings page, `default.json` and the
  docstring ✓; check, fmt and clippy ✓; the review ✓.
- **Built.**
  - `marley_mcp::find` (new, pure): `words` (lowercased, split at everything but letters and
    digits, twenty-two stop words left out), `matches` (every word a whole word of the item),
    `local` (`Local::Sure`, `Several`, `Nothing`) and `windows(count, size)`.
  - `registry.rs`: `browser_find` (a read tool after `browser_snapshot`) and `terminal_find`
    (after `terminal_read`), their schemas in `find` arms ahead of the arms that catch every
    other verb (`find_schemas`, `terminal_find_schemas`, over a shared `query_schema` and
    `found_properties`); `CONDITIONAL_TOOLS` and `is_off`; `tools_list_for(principal, enabled)`.
    `dispatch.rs` lists with `ctx.enabled` and refuses an off tool by name after `permits`;
    `RequestCtx.enabled`; `ServerData.enabled` and `transport::set_enabled`, the context built
    from it in `serve_post`; the nine `RequestCtx` literals, tests included.
  - `marley_system_one`: `FIND_WINDOW` (254), the `FindText`, `FIND_OPTIONS` (`none`, then `1` to
    `254`), `FIND_QUESTIONS` and `FIND_SETS` statics (`find_1/1` to `find_254/1`), made once, the
    option strings held by a `static` so no string is leaked; `find_set(items)` and
    `find_label(item)`.
  - `system_one.rs`: `ask`, `record` and `send` take a `UseSpec` by value; `Draft` keeps the
    parsed answers and `Asked.answers` hands them on; the check and the stop kind pass theirs.
  - `marley_workbench::find` (new): `Place`, `Found`, `find_items` (the words; the use's mode;
    `detail` before any ask, so an unlisted or metadata-only project makes no call and no row;
    a window's `Asking` of the `query` line and the labeled items; one `ask` per window with a
    `UseSpec` made at call time, 3 s; shadow detaches the asks and answers by the words) and
    `read_windows` (found when a window's `present` reads yes, absent when every window's reads
    no, the candidates by the parsed probabilities, the top in `act` when found and chosen, a
    `rules` provider's reading taken as no answer).
  - `browser_tools.rs`: `read_snapshot`, the walk `browser_snapshot` did inline; `find_element`
    (the snapshot, the refs kept, the items `role “name”`, the place from `hub.project_of`, the
    answer's JSON and text); `find_query` and `found_text`, shared with the terminal.
  - `mcp.rs`: `terminal_find` on `ports_list`'s spawned route; `FindLines::of` (the block by
    index, the output masked whole with the model's rules, `tail`, then the newest lines up to
    96,000 bytes, their first line's number, the workspace's place); `found_lines`; the enabled
    set: `enabled_tools` from `MarleySettings`, `push_enabled` at start and on each settings
    change, and `enabler`, a background task that takes the server's lock.
  - Settings: Browser Find and Terminal Find after Stop Kind (`marley_page.rs`, eleven items),
    `browser_find` and `terminal_find` `"off"` in `default.json`, the docstring's default.
- **Deviations from the plan, and why.**
  - `terminal_find`'s candidates carry the lines as the model's rules masked them, whatever
    agents' redaction says: the model and the answer read one text, so the lines' numbers agree,
    and a multi-line secret collapses the same way `terminal_read` collapses it with redaction on.
  - A `rules` provider's reading counts as no answer (a find has no rules of its own), and the
    tool answers by the words with a note.
  - `find_items` checks the use's mode as well as the server's listing: a use turned off between
    the list and the call answers by the words with a note.
  - `read_snapshot` and `find_items` take `&AsyncApp` (clippy's `needless_pass_by_ref_mut`), and
    `found_text` builds its lines and joins them (`format_push_string`).
- **Check, fmt and clippy.** #566's release install ended first (regress: all 39 passed, installed
  at `e47cbc7b9d`; its binary was built before any of this ticket's source changed). `cargo check
  -p marley_mcp -p marley_system_one`, then `-p marley_workbench -p settings_ui`, clean at once;
  `rustfmt` on the touched files; `just clippy marley_mcp marley_system_one marley_workbench
  settings_ui settings_content` green after two first doc paragraphs were split, `found_text`
  rebuilt, `found_lines` given a slice and two `&mut AsyncApp` made `&`. Logs in the scratchpad's
  `567/`.
- **Review against each REQ and D6.** REQ-001: `Local::Sure` answers before the mode is read, with
  no ask and no row. REQ-002, REQ-003: `Several` asks the model over the matches alone, `Nothing`
  over every item; `read_windows` gives `found`, `absent` or `unsure` and the top in `act` only
  when found and chosen. REQ-004: `browser_find` keeps its refs in the hub as `browser_snapshot`
  does. REQ-005: `terminal_find` numbers the block's lines from its output's first. REQ-006:
  shadow detaches the asks and answers by the words. REQ-007: `tools_list_for` and `tools_call`
  both read `ctx.enabled`, which `push_enabled` sets at start and at each settings change.
  REQ-008 and D6: `detail` before any ask, so an unlisted or metadata-only project makes no call
  and no row. REQ-009: the output masked whole before any cut, then each item again in the state.
  Re-entrancy: `FindLines::of` runs in the incoming call's own update of the app, and
  `find_element` in the browser call's task; neither reads an entity another update holds.
  Provenance: nothing from Warp; no Zed function body carried over. Zed hunks: the two settings
  items, the defaults and the docstring, each additive, the rows written first.
- **Found and changed at review.** `enabled_tools` passed an unsized `str` to the map's `get`
  (found reading the diff before the first build), and `FindLines::of` shadowed the kept tail's
  string with the lines that borrow it; both fixed before the first build.


## Phase 3 — Test
- **Checklist** (no task tool): the scenario ✓; the stand-in's `find`, `tfind` and `click-ref` ✓;
  `just build` ✓; the runs ✓; every shot read ✓; 491, 492, 524, 565, 566 and 584 again ✓; the
  golden entry ✓; the gate ✓; the golden set ✓.
- **The scenario**, `script/e2e/567-find-tools.sh` (`compositor sway`, a headless sway of its own:
  no key or click reached the user's session, and no `claude` ran). The browser fixture's offline
  Chromium shows a local page with a Sign in button that writes "Signed in.", two Submit buttons
  in two forms and a Docs link; the scratch repository's `print-lines.sh` prints 300 numbered
  lines, line 212 `connect: connection refused` and line 250 a token put together at run time.
  The profile turns the layer on with the `replay` provider and the repository listed, both uses
  in `act`; `replay.jsonl`, written in setup, holds two `find_2/1` rows for `query: submit`
  (`which` `2` at 0.8, `present` 0.9), a `find_254/1` row for the terminal's second query (`212`
  at 0.85, `present` 0.92) and `find_1/1` to `find_20/1` rows for `query: coupon code` (`present`
  0.05), since that query asks over every ref of the page. The stand-in agent gains `find`,
  `tfind` (by the block's command, as `terminal-read` finds it) and `click-ref`, which clicks a
  ref as given. `script/e2e/golden` gains the 567 entry (40).
- **Runs.** Run 1: every check passed; the settings shot stopped at Browser Find's title, so the
  scroll went from 20 to 28. Run 2: all 18 checks passed (exit 0).
- **The run log** (run 2):
  - With both uses on, `tools` lists 26, `terminal_find` and `browser_find` among them.
  - `find "sign in"`: `source: rules`, `sure: true`, `ref` `e1`, the one candidate `button "Sign
    in"`, and no call in the day's file (REQ-001).
  - `find "submit"`: `source: model`, `sure: true`, `present: found`, `ref` `e5`, the second
    Submit the snapshot lists, at 0.8, then `e3` at 0.15 (REQ-002).
  - `find "coupon code"`: `sure: false`, `present: absent`, `next: browser_snapshot`, the one
    candidate at 0.1 (REQ-003).
  - `tfind print-lines "connection refused"`: `source: rules`, `line` 212 (REQ-005).
  - `tfind print-lines "where did the server fail to connect"`: two windows (254 and 46 lines);
    `source: model`, `present: found`, `line` 212 with its text at 0.85, then line 211 at 0.05
    (REQ-005). The `find_254/1` row's state holds the token as `[redacted: …]` and nowhere as
    printed (REQ-009).
  - `browser_find` in `shadow`: `find "submit"` answers `e3` and `e5` by the words, `sure: false`,
    and the day's file gains the one window's row, mode `shadow` (REQ-006).
  - With no project listed: the words' two candidates and the note "System One: project not
    listed", and no new row (REQ-008).
  - Both uses `off`: `tools` lists 24, neither find tool among them, and `find "submit"` is
    refused: "browser_find is off: turn System One on and set marley.system_one.uses.browser_find
    to shadow, suggest or act" (REQ-007).
- **The shots** (run 2, read at full size and cropped):
  - `567-01-clicked`: the Browser tab's page reads "Sign in Signed in.", and its Agent chip
    "clicked button “Sign in”"; the rail lists the project's terminal row and the tab's row
    (REQ-004).
  - `567-02-off-not-listed`: the terminal ends at `line 300: ok`, and the page is as it was, with
    no Agent chip: the refused call did nothing (REQ-007).
  - `567-03-settings`: the Settings window's System One section: Check `Act`, Stop Kind `Off`,
    then Browser Find and Terminal Find, both `Off` after the scenario's last change, with their
    descriptions, then Decisions.
  - The shots hold the stand-in's terminal and the page only, and stay in the scratchpad.
- **Again.** `just regress` on 491, 492, 524, 565, 566 and 584: all 6 passed, their tool counts
  unchanged with the uses off (492's no-script list, 524's and 584's clients' 10 and 18).
- **The gate.** `script/gates.sh --diff`: 16 passed, 0 failed, `GATE GREEN [diff]`, on the first
  run; the full log is the scratchpad's `567/gate1.log`.
- **The golden set.** `just regress`: all 40 passed, `567-find-tools` in 49 s (REQ-010).
- **The focus report.** Every run was in a headless sway of its own; nothing reached the user's
  session, no `claude` ran, and the keyring was not touched.
- **Out of reach:** a real model's ranking (the replay rows stand in), and #565's gate refusing a
  repeated find, which the replay provider skips.
- **Pre-existing, not in scope:** none.
- **Verdict:** PASS.


## Phase 4 — Complete
- **Documented.** `CHANGELOG.md` Added (the find tools, and Decisions' "1 call"). The architecture
  record: `marley_mcp.md` (the served families, the conditional tools and `find`);
  `marley_system_one.md` (the find sets, the consumers, the scenario); `marley_workbench.md`
  (`terminal_find` and the tools turned on under the MCP server, the adapter's `UseSpec` by value
  and `Asked.answers`, a section for `find.rs`, and `browser_find` among the browser's tools);
  `docs/marley/guide.md` (the two tools' rows, the note on their listing and reconnecting, a
  section "The find tools" under System One, the uses line); `three-prong-plan.md`'s S1 row.
  `zed-touchpoints.md` rows 55, 58 and 61 checked against what shipped.
- **Knowledge appended.** L-claude-567-a-choice-over-run-time-items-is-a-static-table-by-count-001,
  L-claude-567-a-tool-a-setting-lists-must-reach-the-servers-data-001;
  AD-claude-567-the-find-tools-answer-by-words-first-and-are-listed-only-while-on-001. No `F-`
  block: Code and Test found nothing past the compiler, and every check passed at its first run.
- **The brain.** Consultation c25b649370f145dab192aa6c1d7aef14 closed with
  `decisions/marleys-find-tools-words-first-the-model-for-the-rest-listed-only-while-on-marley-567`,
  follow-up by 2026-10-27; the duplicate consultation of the same minute closed with no decision.
- **Closed and archived.** TICKET-567 to `tickets/closed/`, the pair to `completed/`; no queued
  spec named `queued/567`.
- **Commit.** The receipt of Test's green still matches the tree: nothing under `crates/`,
  `script/e2e/`, `script/gates.sh` or the manifests changed after it.
