# Pick, fix, check: the same element after the agent's fix — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-505-pick-fix-check.md
- **Pipeline spec:** 505-pick-fix-check.spec.md

## Phase 1 — Plan (drafted overnight, 2026-09-25)
- **Request:** Chad, 2026-09-25: "lets do 1 through 8", item 2 of the list after the browser
  waves: after the agent's fix, re-find the picked element, crop it again, show before and after,
  and give agents the same through a tool. The Orca survey of the same day (report 03 item 2, the
  README's #505 row) supplies the rules: what to record at pick time (the box, the style subset,
  the text, the HTML, which #518 adds), the re-find order, hashed classes out of locators, crops
  under the same conditions, "what changed" as text. It also says #505 waits for the fuller pick,
  so this lands after #518.
- **Classification / tier:** feature, prong 3, the pick's second half. Marley crates only
  (`marley_browser`, `marley_workbench`, `marley_mcp`).
- **Recall (§18.3):**
  - AD-claude-496-picks-are-staged-and-sent-to-the-last-terminal-001: the bundle is read at the
    pick and kept for the session; the check keeps its result beside it, so a pick outlives a
    reload and can be checked after one.
  - L-claude-496-chromium-picks-an-element-over-cdp-001: inspect mode off before a crop, the clip
    in page coordinates, `Debugger.enable` replaying scripts.
  - L-claude-488-an-overlay-from-any-session-shows-in-every-screencast-001: another CDP session's
    highlight is drawn into the page and so into a crop; D3 cannot rule it out (Risks).
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: the Check button widens
    the tray row, so the pick scenarios rerun and their logs are read.
  - L-claude-498-draw-over-a-streamed-page-from-the-frame-drawn-001: a child over the page that
    takes clicks `occlude()`s; the card does.
  - AD-claude-492: write tools bring the tab forward without the focus and say what they did in
    the Agent chip; D6.
  - Brain: not consulted in this drafting run (read-only brief); promotion runs `brain_ask`.
- **Discovery:**
  - `crates/marley_browser/src/pick.rs`: `DESCRIBE` (43 to 100): the id locator (59) takes any id,
    and the CSS path (65 to 72) stops at the first ancestor with an id (67), so a generated id
    today anchors both; classes appear only in `name()`, the blockers' description (47).
    `INTERACTIVE_ANCESTOR` (25); `capture_pick` (285); `crop` (339: 16 px margin, JPEG at 80,
    scale 1); `call_on` (364).
  - `crates/marley_workbench/src/browser.rs`: `Pick` (204, `crop` kept out of serde);
    `pick_requested` (1172: inspect off, capture, crop, stage); `pick_sent` (1339) and the hub's
    `discard_pick` (1349); `TrayRow` (3062); the tab's `discard_pick` (3746); `render_dialog`
    (4358, the card pattern over the page area); `render_tray` (4438, rows of one height, the tray
    at most 9 rem and scrolling); `render_pick_row` (4511: the number, summary, listener place,
    caption and Send, or what was sent, then discard); the page area's overlays in `render`
    (4679 on: annotations, the message, the dialog, and the select menu deferred and anchored);
    `show_for_agent` (5681).
  - `crates/marley_workbench/src/browser_tools.rs`: `WRITES` (42), `done` (394), `acting` (409).
    `crates/marley_mcp/src/registry.rs`: `browser_read` (221) and `browser_write` (233, the
    `browser.write` grant).
  - `crates/marley_browser/src/frame.rs` `decode`: a base64 JPEG into a `RenderImage`, which the
    card uses for both crops.
  - React 19.2.8's `useId` in `/srv/stacks/scorchkit_home/node_modules/react-dom/cjs/
    react-dom-client.development.js` (lines 9055 to 9062: `_R_…_` and `_r_…_`); React 18 gives
    `:r1:`-style ids.
- **Decisions:** D1 to D6 in the spec.

### Design
- **Approach.**
  1. *The generated-name rule.* A page-side helper, `generatedName(value)`, joins `DESCRIBE`'s
     function text: true for a value holding `:` or `«`, matching `^_.*[rR]_[0-9A-Za-z]+_$`
     (React 19.2 writes `_R_…_` for an id made while hydrating and `_r_…_` for one made on the
     client, after any `identifierPrefix`), starting `radix-`, `headlessui-`, `react-aria`,
     `mui-<digits>` or `ember<digits>`, or Orca's hashy shape (`^[A-Za-z0-9_-]{12,}$` with a digit
     and an uppercase letter). The id locator skips such an id; the path walks past it to the
     next ancestor. The helper is a Rust `const` of JavaScript that `DESCRIBE` and #506's recorder
     both include.
  2. *`Page::refind(&self, bundle: &PickBundle) -> Result<Refound, CdpError>`.* One page function
     in the main world takes the pick's test-id, id and CSS-path locators and its text and old
     box, and returns, for the first kind that matches, the matches as an array (object ids by
     `returnByValue: false`) with their boxes. Role and name go through CDP between the id and the
     text: `Accessibility.queryAXTree` from the document's object with `accessibleName` and `role`,
     then `DOM.resolveNode` per `backendDOMNodeId` and a box read. The nearest center to the old
     box wins. `Refound { found_by: Option<&'static str>, element: Option<String> }`.
  3. *`Page::check(bundle)` (pick.rs).* `refind`; when found, `DOM.scrollIntoViewIfNeeded` on it
     if its box lies outside the viewport, a short wait for a frame, `DESCRIBE` on it (#518's
     version), then `crop(new_box)`. `pick::changes(before, after) -> Vec<String>`: the box's
     move (`box: moved 0, 24 px`) and size (`box: 200 × 44 → 232 × 44`) when at least 1 CSS px,
     each differing style (`padding: 8px → 16px`), `text: “Save” → “Save changes”`, the role or
     name, then `its HTML changed` only when nothing above changed but the HTML did.
  4. *The hub.* `Pick` gains `check: Option<PickCheck { found_by: Option<String>, changes:
     Vec<String>, bundle: Option<PickBundle>, crop: Option<String> (serde skip), at: u64 }>`.
     `BrowserHub::check_pick(id, cx) -> Task<Result<PickCheck, String>>`: the pick's tab must
     still have its page (else "pick N's tab is closed"); pick mode goes off in that page first;
     the result is stored and `PageStatusChanged` emitted (#504's event, or `cx.notify` if #504
     has not landed).
  5. *The tray and the card.* `render_pick_row` gains a `Button` "Check" before the discard
     (tooltip "Find this element again and compare it with the pick"), and after a check a small
     verdict button (`2 changes`, `no change`, `not found`) that opens the card. The card is
     `BrowserView` state (`comparison: Option<usize>`), drawn as an absolute child of the page
     area at its top right like the dialog card, occluding the page under it: a header with
     `Pick N`, the summary, `found by <kind>` and a close `IconButton`; a row of two columns ("At
     the pick", "Now"), each `img` of the decoded crop scaled to fit (at most 16 rem tall, half the
     card wide); then the changes as small labels. Not found: the header, the sentence, and the
     pick's crop alone.
  6. *The tool.* `browser_check_pick` in `WRITES`, run through `acting` ("checking pick N",
     "checked pick N"); the answer: `id`, `tab`, `found`, `found_by`, `changes`, `bundle` (through
     the `Redactor` as #518 does) and the crop as the image. The registry gains its row with
     `browser_write` and its schemas.
  7. *The fixture.* `mcp_agent check-pick <id> [<image file>]` prints `found_by`, each change and
     the new box, and saves the crop.
- **File manifest.** Marley crates only: `crates/marley_browser/src/pick.rs` (the rule, `refind`,
  `check`, `changes`); `crates/marley_workbench/src/browser.rs` (the hub's `check_pick`, the tray
  row, the card), `crates/marley_workbench/src/browser_tools.rs` (the tool);
  `crates/marley_mcp/src/registry.rs` (its row and schemas). `script/e2e/browser-fixture.sh`
  (`check-pick`), `script/e2e/505-pick-fix-check.sh` (Test). Docs at Complete: `CHANGELOG.md`,
  `docs/marley/three-prong-plan.md`, `docs/marley_architecture/marley_browser.md`,
  `marley_workbench.md`.
- **Ledger rows.** None in `docs/marley/zed-touchpoints.md`: no Zed path. Knowledge at Complete
  (expected): an AD for the check (the order, the same-conditions crop, DOM-based changes) and
  lessons from the run (`queryAXTree` in headless Chromium, what a generated id looks like in the
  frameworks met).

### E2E plan
Setup: `offline_chromium`; a site with `app.css` (`#save { padding: 8px }`) and `index.html`: a
Save button (`data-testid="save-button"`, `id="save"`, text "Save") and a Delete button
(`id=":r5:"`, `class="css-1q2w3e"`, text "Delete", no test id); the scratch repository. The fix is
played by rewriting the files and reloading the tab with Ctrl+R in the page, as an agent's edit and
a hot reload would do.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-002 | pick Save, then Delete (Ctrl+Shift+C and a click each); `mcp_agent pick 2` | the log: Delete's locators hold text and a CSS path, no `#:r5:` and no class |
| REQ-001 | rewrite: `padding: 16px`, Save's text "Save changes"; Ctrl+R; Check on pick 1 | `505-01-check`: found by test id, the two crops, `padding: 8px → 16px`, the text change, the resize |
| REQ-008 | rewrite: Delete's id `«r9»`, class `css-9o8i7u`; Ctrl+R; Check on pick 2 | `505-02-generated-names`: found by role and name |
| REQ-003 | rewrite: a second Delete button 600 px lower; Ctrl+R; Check on pick 2 | `505-03-nearest`: the crop shows the original Delete; the log's box within a few pixels of the pick's |
| REQ-006 | `mcp_agent check-pick 1 <file>` | the log: `found_by` test id, the changes, the new box; the saved crop read at Test |
| REQ-005 | rewrite: a 2,000 px spacer above Save; Ctrl+R (the page opens at its top); Check on pick 1 | `505-05-scrolled`: Save scrolled into view, the new crop as wide as the box plus 32 px (log) |
| REQ-004 | rewrite: both Delete buttons removed; Ctrl+R; Check on pick 2 | `505-04-not-found`: the sentence and the pick's crop alone |
| REQ-007 | close the card; the tray | `505-06-verdict`: pick 1's row reads its changes, pick 2's "not found"; a click on pick 1's verdict opens its card |

Not reachable here: a page the agent really edits with a dev server's hot reload (the rewrite and
reload stand in for it), and picks inside cross-site iframes (Out).

### Risks
- `Accessibility.queryAXTree` is experimental in CDP; if Marley's Chromium answers it badly, the
  role-and-name step reads roles and names from `Accessibility.getFullAXTree` filtered in Rust,
  as `browser_snapshot` does.
- Another CDP client's highlight is drawn into the page and would show in a crop (L-claude-488);
  Marley's own inspect mode is off for both crops, but a stand-in agent's highlight is not
  Marley's to turn off.
- Scrolling the element into view moves the page the user is looking at; the check is a user
  action or an agent's write, and the chip says so.
- A text match walks up to the interactive ancestor, as the pick did; an element whose text
  changed and that has no test id, usable id or role and name falls through to the CSS path,
  which a layout change breaks, and then it is not found. That is the honest answer.
- Comparing computed widths reports every reflow as a change; D4 lists box and styles separately
  so a moved neighbor reads as `box: moved`, not as a style change.

## Promotion (2026-09-26, at `1f33b90233`)
- **Seams re-read** (the draft's line numbers moved with #503, #504, #518 and #580; the shapes
  stand):
  - `pick.rs`: `INTERACTIVE_ANCESTOR` (70); `DESCRIBE` (95), now `function (secretNames)` and
    called through `call_on_with` (746) with `observe::SECRET_NAMES` since #518. The id locator
    (118 to 121) still takes any id, and the CSS path (124 to 131) still stops at the first
    ancestor with an id; classes appear only in `name()` (106), the blockers' description.
    `PickBundle` (356) with #518's `html`, `styles`, `nearby_text`, `selected_text` and
    `component`; `Described` (462); `capture_pick` (637); `crop` (711); `call_on` (736).
  - `browser.rs`: `Pick` (217); `pick_requested` (1317: inspect off, capture, crop, stage, then
    #518's `MapCache` for the listeners and the stack); `pick_captured` (1372), which emits
    `PickStaged` and #504's `PageStatusChanged` through `status_changed` (1543); `pick_sources`
    (1419); `pick_sent` (1511); the hub's `discard_pick` (1521); `TrayRow` (3356); the tab's
    `discard_pick` (4045); `render_dialog` (4657, the card pattern: absolute, `inset_0`,
    `occlude`); `render_tray` (4737); `render_pick_row` (4810: number, summary, listener place,
    then the caption and Send or what was sent, then discard); `render` (4979: the page area a
    `relative` div holding `PageElement`, the annotations, the message and the dialog);
    `show_for_agent` (6073).
  - `browser_tools.rs`: `WRITES` (54); `run` (193), where `browser_picks` and `browser_pick`
    answer before the browser is up; `page_of` (301); `picks` (492) and `pick` (519), both
    through `pick_for_agents` (549, #518); `done` (669); `acting` (684), which answers with
    `done`.
  - `registry.rs`: `browser_read` (238), `browser_write` (250), the pick's description (162),
    `element_context_properties` (692), `pick_schemas` (746), `browser_write_schemas` (842).
  - `frame.rs` `decode` (15).
- **The probe:** `Accessibility.queryAXTree` answers in Chromium 152.0.7977.82, the build
  Marley runs (`/usr/lib/chromium/chromium`). Run standalone and headless, on a temporary profile
  in the session's scratchpad and an offline page, it gave both Delete buttons by role and name
  with their `backendDOMNodeId`s, which `DOM.resolveNode` turned into the right elements and
  boxes. It gave none for a missing name, and `DOM.scrollIntoViewIfNeeded` brought the far one
  into view.
- **Landed since the draft:** #518 (the fuller bundle, `pick_for_agents`, `call_on_with`);
  #504 (`PageStatusChanged`); #574 (a call naming no tab acts in the caller's project); #580 (no
  change to the mouse path).
- **Recall added:**
  - AD-claude-518 and PR-claude-redact-the-whole-text-before-cutting-it-001: `browser_pick`
    serializes the whole `Pick`, so a `check` field kept on it reaches agents too. Its bundle and
    its change lines, which quote the page's text, go through `pick_for_agents` as the pick's
    own fields do.
  - L-claude-518-a-lone-surrogate-fails-the-whole-cdp-message-001: the re-find's page function
    returns texts and locators, so it makes them well formed as `DESCRIBE` does.
  - AD-claude-574: the check acts in the pick's own tab, never the caller's default one.
  - F-claude-574-a-placement-outlived-its-page-001: the check keeps nothing in the hub for a
    page that went; a closed tab answers "pick N's tab is closed".
  - L-claude-498 (both): the card occludes the page under it, and the Check button widens the
    tray row, so #496's, #497's and #518's scenarios rerun.
- **Brain consultation 156f18ae8fab475a873abbf82c924d2b:** nothing on this seam.

### Design changes at promotion
- **Approach 3:** the check reads the found element as the pick did, through `capture_pick` on
  its backend node (`DOM.describeNode` gives one for an object id). It leaves the listeners'
  source maps unread, then `crop`s the new box.
- **Approach 4:** a check changes nothing the rail shows, so it emits no `PageStatusChanged`;
  the hub's `cx.notify()` redraws the tray.
- **Approach 6:** `browser_check_pick` takes its tab from the pick. In `run` it answers after
  `showing`, through `page_of` with the pick's tab, `show_for_agent`, and the chip's
  `agent_started` and `agent_ended` ("checking pick N", "checked pick N"). `acting` answers with
  `done`, so the check calls those two itself. `pick_for_agents` gains the check: its bundle
  handled as the pick's, and each change line redacted.
- **Approach 1:** the generated-name rule is one piece of JavaScript that `DESCRIBE` and #506's
  recorder both take: a `macro_rules!` that expands to the string literal, spliced into each with
  `concat!`, since `concat!` takes literals and macros but not a `const`.
- **E2E plan, a new row (REQ-009):** rewrite Save's label to hold a fake token built in the
  setup from pieces (L-claude-516-fake-secrets-are-put-together-at-run-time-001); Ctrl+R;
  `mcp_agent check-pick 1` and `pick-json 1`. The saved answers show `[redacted` and no
  12-character piece of the token (#518's `no_secrets_in`).
- **Phase 1 checklist** (no task tool in this session): pre-flight (cargo busy with the install,
  which is expected); the pair promoted; the BACKLOG row removed; the ticket in progress; the seams
  re-read; the probe; recall; the brain asked; the design changes written. Status: Plan PASS.

## Phase 2 — Code
- **Built** (Marley crates only):
  - `marley_browser/src/pick.rs`:
    - Two `macro_rules!` of JavaScript spliced in with `concat!`: `interactive!` (the test of an
      element a user acts on, and the step up through shadow roots), now shared by
      `INTERACTIVE_ANCESTOR` and the new `REFIND`; and `generated_name!`, which `DESCRIBE` takes.
      The id locator skips a generated id, and the CSS path walks past one.
    - `REFIND`, run on the page's document: a selector's matches, or for a text each element whose
      text reads it, walked up to its interactive ancestor and the innermost kept. It answers the
      match nearest the old box's center. `PAGE_BOX` gives a candidate's box.
    - `Page::refind(bundle) -> Option<Refound { found_by, backend_node_id }>` in `REFIND_ORDER`:
      test id, id, role and name through `Accessibility.queryAXTree` on the document's object, text,
      CSS path. It enables the DOM domain first.
    - `changes(before, after)`: box moved or resized by at least 1 CSS px, each style that
      differs, text, role, name, else "its HTML changed".
    - `CHANGE_BUDGET` (420) for the agent's change lines, and `PageBox::distance_to`.
  - `marley_workbench/src/browser.rs`:
    - `Pick.check: Option<PickCheck { found_by, changes, bundle, crop (serde skip), checked_at
      (ms) }>`, and `PickCheck::verdict`.
    - `SCROLL_SETTLE`, moved here from `browser_tools.rs` so the check and the tools share it.
    - `BrowserHub::check_pick(id)`: the pick's own tab's session, with pick mode off first. It
      runs `check_element` (refind, `scroll_into_view` on the page's session, `SCROLL_SETTLE` on
      gpui's timer, `capture_pick` on the found node, `crop`, `changes`), then stores the check and
      notifies. No `PageStatusChanged`.
    - The tab: `checking` and `comparison: Option<Comparison { id, checked_at, before, after }>`;
      `check_pick` (the user's, which opens the card when it answers, or puts the error in the
      tray's error row); `open_comparison`, which decodes both crops once; `close_comparison`,
      which drops them from the window's atlas; and `refresh_comparison` at the top of `render`,
      which follows an agent's newer check or closes on a discarded pick.
    - `render_check` in the tray row (the verdict button and Check, "Checking…" while it runs),
      and `render_comparison` with `render_crop` (absolute at the page area's top right,
      occluding, 36 rem wide, each crop at most 16 rem tall, each line cut to 300 characters for
      the card only).
  - `marley_workbench/src/browser_tools.rs`:
    - `browser_check_pick` after `showing`: the pick's tab through `page_of`, `show_for_agent`,
      the chip's "checking pick N" and "checked pick N" (or "failed: …"), and the answer from
      `pick_for_agents`.
    - `pick_for_agents` is split into `redacted` (each page text whole) and `within_budgets`
      (#518's cuts). A check's change lines are made again from the two redacted bundles, then
      cut to `CHANGE_BUDGET`, so a secret in a text change is redacted before any cut.
    - `pick_id` is shared by both pick tools.
  - `marley_mcp/src/registry.rs`: the `check_pick` write row; `bundle_schema`, `check_schema`,
    `pick_id_schema`, `nullable` and `check_pick_schemas`; `browser_pick`'s output gains `check`
    and its description names it.
  - `script/e2e/browser-fixture.sh`: `check-pick <id> [<image file> [<json file>]]`, and `pick`
    prints the latest check.
- **Deviations from the design, and why:**
  - `page.rs` already had `scroll_into_view(session, backend_node_id)` (#498), so the check calls
    it with the page's own session rather than adding a second.
  - The change lines an agent gets are rebuilt from the redacted bundles, not redacted line by
    line. The rebuilt lines quote only redacted text, and they say "changed" only where the
    redacted texts differ.
  - `checked_at` is in milliseconds, not seconds. The card compares it to tell a newer check
    from the one it shows, and two checks can run in one second.
- **Review** (against each REQ and the security line):
  - Nothing is cropped when nothing is found (REQ-004). The not-found card shows the pick's crop
    alone, with the sentence.
  - Every page text in a check reaches agents through `redacted` before `within`, through
    `browser_check_pick` and through `browser_pick`'s `check` alike (REQ-009,
    PR-claude-redact-the-whole-text-before-cutting-it-001).
  - No entity is updated while it is updated: the tab calls the hub's `check_pick` from its own
    listener, and the hub's task updates the hub alone. The tab learns of the result through its
    own task.
  - A discarded pick closes its card at the next render, and the decoded crops are dropped from
    the atlas.
- **Checks:** `cargo check`, `cargo fmt`, and `cargo clippy -p marley_browser -p marley_mcp -p
  marley_workbench --all-targets -- -D warnings`, clean after four fixes: a first doc paragraph too
  long; `render_comparison` and `render_pick_row` over 100 lines, split into `render_crop` and
  `render_check`; an `Option` match written as `map_or_else`; and a `&mut Window` only read.
  `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps` on the three, clean once `refind`'s doc
  stopped linking the private `REFIND_ORDER` (L-claude-504). `cargo dylint` exits 0 with no hit
  in a Marley crate. `node --check` parses the four page functions as `concat!` assembles them,
  and the generated-id test sorts eighteen sample ids as designed.

## Phase 3 — Test
- **The scenario**, `script/e2e/505-pick-fix-check.sh` (`compositor sway`, offline Chromium).
  The page is Save (test id `save-button`, id `save`, 8 px of padding from `app.css`) and
  Delete (id `:r5:`, class `css-1q2w3e`, no test id). The user picks both through the toolbar's
  pick button. Each fix rewrites `index.html` and `app.css` (the stylesheet's URL changes each
  time) and reloads the tab from the toolbar. The user's checks are clicks on the tray's Check;
  the agent's go through `mcp_agent check-pick`. The fake token is made in the setup from
  pieces. The estimated coordinates held on the first run: Check at x 1297 and the verdict's
  right end at x 1255, with tray rows 36 px tall from y 109.
- **Four runs.**
  - Run 1 passed all 12 checks, but in its shots the Check button's tooltip, left up by the
    pointer, covered the card's "found by" label. `check_row` and the verdict click now move the
    pointer to the page's bottom afterwards.
  - Run 2 passed all 12 checks again, but its `505-03-nearest` showed a real bug, which no check
    had caught. The page area was dark except for a 152 × 72 patch at its top left: Delete and its
    margin, exactly the check's crop. `Page::crop` took `Page.captureScreenshot` with a `clip`, and
    that capture reached the page's running screencast as a frame of the clip alone. The tab drew
    it in place of the page, and since a check changes nothing on the page, no newer frame
    replaced it. A standalone probe (Chromium 152, a still page, a screencast running) got full
    frames from both a clipped and a full capture, so it is a race with a frame being captured,
    not every time. A pick hides it, since its new tray row resizes the viewport and a fresh frame
    follows.
  - The fix (Code, in this phase): `crop` captures the whole viewport as a PNG, with
    `Page.getLayoutMetrics` giving the visual viewport's place and size, and cuts the box and its
    margins out in `pick::cut` on `smol::unblock`'s pool. It scales to one pixel per CSS pixel and
    encodes a JPEG at 80, as before. A frame the race hands the screencast is then a whole
    viewport. The pick's crop takes the same path, so both crops are still made the same way
    (D3). Clippy, rustdoc and the build are clean again.
  - The scenario gained `page_drawn`: after each check that crops, the pixel at page (300, 400)
    must have the page's light background, not the tab's dark one. As a negative smoke, it reads
    0.17 on run 2's `505-03-nearest` (fail) and 0.95 on the fixed runs (pass).
  - Runs 3 and 4 each passed all 16 checks, the page drawn whole after every check. Run 4
    (`505-run4.log`) is the one below.
  - REQ-002: Delete's locators are `text: Delete` and `css: #panel > button:nth-of-type(2)`, with
    no id locator, no `:r5:` and no class.
  - REQ-001: pick 1 found by `test id`; its changes hold `padding: 8px → 16px`, the text line and
    the box line.
  - REQ-008: pick 2 found by `role and name` after its id became `«r9»` and its class
    `css-9o8i7u`.
  - REQ-003: with a second Delete 600 px below, the one found lies within 5 px of the pick's box.
  - REQ-006: the agent's check of Save: found by test id, and the changes read
    `box: 61 × 41 → 150 × 57`, `height: 41px → 57px`, `padding: 8px → 16px`,
    `width: 61.0312px → 150.094px`, `text: “Save” → “Save changes”`, `name: “Save” → “Save
    changes”`, with the box now `40,40 150x57` and the crop saved.
  - REQ-005: with 2,000 px above Save, the agent's check scrolled the page to `scroll_y` 2000 and
    cropped a 182 × 89 JPEG, the box's 150.1 × 57.0 plus 16 px on each side.
  - REQ-004: with both Deletes gone, pick 2's check has no `found_by`, no bundle and no changes.
  - REQ-009: with Save's label holding the token, the agent's lines read `text: “Save” → “Save
    [redacted: github token]”` and the same for the name. No 12-character piece of the token is
    in the check's answer or in `browser_pick`'s (whose `check` is that check).
  - Nothing reached the system browser: the leak log is empty.
- **The shots**, read (run 4):
  - `505-00-picks`: two tray rows, pick 2 (Delete) above pick 1 (Save), each ending in Send,
    Check and the discard button.
  - `505-01-check` (REQ-001): pick 1's card at the page area's top right. "Pick 1 button “Save”",
    "found by test id" and the close button; "At the pick" with Save beside "Now" with Save
    changes; then the lines from `box: 61 × 41 → 150 × 57` down. Pick 1's row reads "6 changes".
  - `505-02-generated-names` (REQ-008): pick 2's card, "found by role and name", Delete in both
    crops, "its HTML changed"; pick 2's row reads "1 change".
  - `505-03-nearest` (REQ-003): the page, drawn whole, holds two Deletes, the second far below.
    The card's "Now" crop is the upper one, "found by role and name".
  - `505-05-scrolled` (REQ-005): the page scrolled to Save, the scroll bar's thumb at its foot.
    The card, found by test id, starts with `box: moved 0, 2000 px`; pick 1's row reads "7
    changes".
  - `505-04-not-found` (REQ-004): pick 2's card says "not found", shows the pick's crop alone, and
    reads "Not on the page now: none of the pick's locators finds it." Pick 2's row reads "not
    found", and the page holds Save alone.
  - `505-06-verdict` (REQ-007): the tray reads "not found" on pick 2 and "7 changes" on pick 1,
    and a click on pick 1's verdict opened its card again: found by test id, both crops, the
    seven lines.
- **Focus:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule and did
  not reload it" on each run.
- **The pick scenarios, rerun** for the tray row that Check widens (L-claude-498), and again
  after the crop's fix, since picks crop the same way:
  - #518: 16 checks pass each time. Its crops come from the new path at the boxes' sizes plus
    32 px (Save 18's: 232 × 76).
  - #496: the staged row now ends in Send, Check and the discard button, and both crops are
    232 × 76, the 200 × 44 buttons plus their margins. The line typed into the
    terminal is as before, and after Escape the click on Save reaches the page ("saved").
    `browser_picks` shows pick 1 sent with its caption and pick 2 not sent.
  - #497: the row's `src/app.ts:2` link still opens the file at line 2, and `plain.js:2` still
    opens nothing.
- **Not reached here:** a real dev server's hot reload, which the rewrite and the reload stand
  in for; picks inside cross-site iframes, which are out of scope.
- **The golden set**, with `505-pick-fix-check.sh` added: `just regress` ran all 27 against the
  debug build with the crop's fix, and all 27 passed (#505's in 88 s). An earlier run, started
  before the fix, was stopped once the fix superseded it. Its scenario's EXIT trap ran, and the
  headless sway it left was stopped by hand.
- **The gate:** `script/gates.sh --diff` (`gate-505.log` in the session's scratchpad): 16
  passed, 0 failed, `GATE GREEN [diff]`, with the receipt.
- **Verdict:** Phase 3 PASS. Nothing pre-existing was excluded.

## Phase 4 — Complete
- **Documented:** `CHANGELOG.md` (Added: "Check a pick after a fix"); the plan's row B3d in
  `docs/marley/three-prong-plan.md`; `docs/marley_architecture/marley_browser.md` (the crop's
  whole-viewport capture, and a section, "Checking a pick");
  `docs/marley_architecture/marley_workbench.md` (the Checks bullet under the Browser tab,
  `browser_check_pick` under the browser tools, and a known limit: a check looks in the main
  document only, and its crop shows visible text unredacted). No Zed path was touched, so no
  ledger row.
- **Knowledge appended:** F-claude-505-a-clipped-capture-left-the-tab-showing-only-the-crop-001,
  PR-claude-capture-a-screencast-page-whole-and-cut-it-locally-001,
  L-claude-505-a-shot-that-proves-something-gets-a-pixel-check-001,
  L-claude-505-queryaxtree-finds-by-role-and-name-in-headless-chromium-001,
  AD-claude-505-a-check-finds-a-pick-again-and-compares-it-from-the-dom-001.
- **Brain:** consultation 156f18ae8fab475a873abbf82c924d2b closed with
  `decisions/a-check-finds-a-pick-again-and-compares-it-from-the-dom-505`, follow-up by
  2026-10-26.
- **Closed and archived:** the ticket in `docs/planning/tickets/closed/`, this pair in
  `docs/planning/pipeline/completed/`.
