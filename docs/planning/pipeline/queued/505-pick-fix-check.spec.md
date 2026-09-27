---
pipeline_id: 6b14f3cd-3063-4a40-afe1-808950e78dae
ticket: docs/planning/tickets/open/TICKET-505-pick-fix-check.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Pick, fix, check: a pick re-found after the page changed, before and after side by side, and a tool"
type: feature
slice: prong 3 (after wave 2), item 2 of the list after the browser waves; after #518
references: [docs/orca_architecture/README.md, docs/orca_architecture/03-browser-and-design-mode.md, docs/planning/pipeline/completed/496-element-picker.spec.md, docs/planning/tickets/closed/TICKET-518-fuller-pick-bundle.md]
---

## Title
After the agent's fix, a pick's Check finds the same element again, crops it as the pick did,
and shows the two crops side by side with what changed written out, so the user and the agent
both see whether the fix took.

## Scope
### In
- **Locators without generated names (`pick.rs`, `DESCRIBE`).** At the pick, the id locator and
  the CSS path's id anchor leave out ids that look generated: holding `:` or `«` (React 18's
  `useId` gives `:r1:`, and a later release `«r1»`), React 19.2's `_r_…_` and `_R_…_`, starting
  `radix-`, `headlessui-`, `react-aria`, or `mui-` or `ember` followed by digits, or 12 or more
  characters of `[A-Za-z0-9_-]` holding a digit and an uppercase letter (Orca's `looksHashy`).
  Classes stay out of the path, as today.
- **The re-find (`pick.rs`, `Page::refind`).** In the page's main document, in order: the test
  id, the id, the role and name from the pick's accessibility node (`Accessibility.queryAXTree`),
  the text (elements whose trimmed text equals the pick's, walked up to their interactive
  ancestor as the pick was), the CSS path. The first kind that matches decides; among several
  matches, the one whose box center lies nearest the old box's; none gives "not found".
- **The check.** Pick mode off; the found element scrolled into view when it lies outside the
  viewport (`DOM.scrollIntoViewIfNeeded`, then a frame); the element read again (#518's
  `DESCRIBE`) and cropped with the pick's `crop`; `pick::changes(before, after)` lists what
  changed. The latest check is kept on the pick (`Pick.check`).
- **In the tab.** Each tray row gets a Check button. The user's Check opens a comparison card
  over the page, at its top right: the pick's number, summary and what found it, the crop at the
  pick and the crop now side by side, the changes one per line, a close button. After a check,
  the row shows its verdict ("2 changes", "no change", "not found"), which opens the card again.
- **For agents.** `browser_check_pick {id}`: a write tool (it may scroll the page the user
  watches), with the Agent chip and the tab brought forward as the other write tools do; the
  answer gives `found`, `found_by`, `changes`, the new bundle (through #516's `Redactor`, as
  `browser_pick`'s is after #518) and the new crop as its image.
- **The fixture.** `mcp_agent check-pick <id> [<image file>]` in `script/e2e/browser-fixture.sh`.

### Out (explicitly deferred)
- A check started by the page's hot reload (Vite's `[vite] hot updated` console line, which the
  console ring already keeps): once Check has proven itself by hand.
- Re-finding inside cross-site iframes and shadow roots: a pick made there checks as not found in
  the page's main document, and says so.
- Comparing the crops' pixels: what changed comes from the DOM, and the crops are for the eye.
- A history of checks per pick: the latest only.
- Checking annotations the same way.

## Reference (§20)
Orca has nothing here (report 03 §2.7): its fix recipe (`docs/site/content/docs/recipes/
design-mode-fix.mdx`) ends at "Click the element again to verify". The survey's item 2 gives the
rules Marley follows: record at the pick what the check compares, re-find by the most durable
locator first with hashed CSS-in-JS classes kept out, crop both times under the same conditions,
report what changed as text. The locator order follows Playwright's advice that test ids and
user-facing attributes outlast CSS structure (playwright.dev/docs/locators). Upstream Zed: N/A,
no browser. Warp: N/A; the once-over (`docs/planning/design-notes/warp-once-over-2026-09-25.md`)
rules its browser features out as covered by Marley's own, #488 to #499.

### Prior art
- **Behavior maps and reports.** `docs/orca_architecture/README.md` (the #505 row: it waits for
  item 4's fuller pick, #518); report 03 §2.4 (Orca's selector: ids, up to two stable classes,
  `:nth-of-type`), §2.7 and item 2; Orca files read at `1c2cf120e3`:
  `src/main/browser/grab-guest-element-context-script.ts` (`looksHashy`, `getStableClasses`
  skipping `css-…` and hashy classes, `buildSelector`), `src/main/browser/
  browser-grab-screenshot.ts` (a crop of the viewport only, which fails for an element off
  screen: not copied), `docs/site/content/docs/recipes/design-mode-fix.mdx`.
- **Published material.** CDP's `Accessibility.queryAXTree` (a subtree searched by accessible name
  and role, answering nodes with their `backendDOMNodeId`), `DOM.scrollIntoViewIfNeeded`,
  `Page.captureScreenshot`'s `clip`; React's `useId` formats, read in the dev builds on the box
  (19.2.8's `react-dom-client.development.js` makes `_r_<id>_` and `_R_<id>_`); Playwright's
  locator guidance.
- **Code we already ship.** `pick.rs`: `DESCRIBE` (its locators and `unique` checks),
  `INTERACTIVE_ANCESTOR`, `capture_pick`, `crop` (16 px margin, scale 1, JPEG at 80, the clip in
  page coordinates, L-claude-496), `call_on`; `browser.rs`: the tray (`render_tray`,
  `render_pick_row`, `TrayRow`), `Pick`, `pick_requested` (inspect off before the read and the
  crop), the dialog card and the select menu as overlays on the page area, `frame::decode` (a
  base64 JPEG into a `RenderImage`, for the crops), `acting` and `show_for_agent` in
  `browser_tools.rs` for a write tool; `registry.rs`'s `browser_write`. #518's `DESCRIBE`
  additions give the styles, text and HTML the comparison needs.

## UI proof
UI-AFFECTING (a Check button, a verdict and a comparison card in the Browser tab).
`script/e2e/505-pick-fix-check.sh` (`compositor sway`, offline Chromium): a page with a Save
button (a test id, `padding: 8px` from a stylesheet) and a Delete button (a generated id and a
hashed class, no test id). The user picks both; the scenario then plays the agent's fix by
rewriting the page's files and reloading the tab. Shots: `505-01-check` (Save found by its test
id: the two crops, `padding: 8px → 16px`, the text change, the resize), `505-02-generated-names`
(Delete found by role and name after its id and class changed), `505-03-nearest` (a second Delete
far from the first: the one near the old box taken), `505-04-not-found`, `505-05-scrolled` (Save
pushed below the fold: scrolled into view and cropped), `505-06-verdict` (the tray rows' verdicts,
a click reopening a card). The run log holds `mcp_agent pick` (no generated id among the
locators) and `mcp_agent check-pick` (the answer and the saved crop).

## Locked-In Decisions
- D1 — The re-find order is the survey's: test id, id, role and name, text, CSS path. The first
  kind that matches decides; several matches go to the one nearest the old box's center; no match
  is "not found", with no crop of anything else.
- D2 — Generated names never become locators: at the pick, ids that look generated are left out of
  the id locator and of the CSS path's anchor, and classes stay out of every locator. The rule
  lives in one place in `pick.rs`'s page functions, where #506's recorder takes it too.
- D3 — Both crops are made the same way: pick mode off, the element in view (the pick's was, since
  it was clicked; the check scrolls it in when it is not), the same `crop` with its margin, scale
  and quality. Marley's own overlays are drawn by gpui over the frame, never in the page, so
  neither crop holds them.
- D4 — What changed is read from the DOM, not the pixels: the box (moved, resized, by at least one
  CSS pixel), each of #518's sixteen styles that differs, the text, the role and name, and "its
  HTML changed" when only the HTML did; "nothing Marley compares has changed" otherwise.
- D5 — One check per pick, the latest, kept with the pick in the hub for the session. The tray row
  shows its verdict; the user's own Check opens the card at once, an agent's does not.
- D6 — `browser_check_pick` is a write tool, since it may scroll the page the user watches: the tab
  comes forward where that leaves the focus alone, and the Agent chip says "checked pick N"
  (AD-claude-492).

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the user clicks a pick's Check after the page changed, Marley shall find the element by the first of its locators that matches and show the crop at the pick beside the crop now, with what changed as text. | Shot `505-01-check` |
| REQ-002 | WHEN the user picks an element, its locators shall hold no generated id and no class. | The run log's `mcp_agent pick` |
| REQ-003 | WHEN several elements match the locator that decides, the check shall take the one whose box lies nearest the pick's. | Shot `505-03-nearest`; the run log's found box |
| REQ-004 | WHEN no locator matches, the check shall say the element was not found and show no new crop. | Shot `505-04-not-found` |
| REQ-005 | WHEN the found element lies outside the viewport, Marley shall scroll it into view and crop it with the pick's margin and scale. | Shot `505-05-scrolled`; the run log: the crop's size is the box plus the margins |
| REQ-006 | WHEN an agent calls `browser_check_pick` with a pick's id, the answer shall say whether and how the element was found and what changed, with the new bundle and the new crop as its image. | The run log's `mcp_agent check-pick` and the saved crop |
| REQ-007 | WHEN a pick has been checked, its tray row shall show the verdict, and a click on the verdict shall open the comparison. | Shot `505-06-verdict` |
| REQ-008 | WHEN only an element's generated id and hashed class changed since the pick, the check shall still find it. | Shot `505-02-generated-names` |

## Phase Plan
- **P1 Plan** — promote this pair once #518 has shipped; re-verify the seams (the tray, `crop`,
  #518's bundle) against HEAD; confirm `Accessibility.queryAXTree` answers in Marley's Chromium.
- **P2 Code** — the generated-name rule, `refind`, the check, `changes`, the tray's button and
  verdict, the card, the tool and its registry row, the fixture's command; fmt and clippy clean; a
  review of the diff (nothing cropped when nothing is found, gpui re-entrancy between the tab and
  the hub, the write tool's chip).
- **P3 Test** — write and run the scenario and read every shot; rerun #496's, #497's and #518's
  scenarios, whose tray the Check button widens (L-claude-498); `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, the prong's slice status and `docs/marley_architecture/
  marley_browser.md` and `marley_workbench.md` (§21), ledger capture (§19), close, archive,
  commit.
