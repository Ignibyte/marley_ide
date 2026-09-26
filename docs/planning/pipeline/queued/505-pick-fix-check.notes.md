# Pick, fix, check: the same element after the agent's fix — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-505-pick-fix-check.md
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
