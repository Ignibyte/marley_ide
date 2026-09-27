# A drag in a Browser tab keeps its selection — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-580-drag-selection-in-a-browser-tab.md
- **Pipeline spec:** 580-drag-selection-in-a-browser-tab.spec.md

## Phase 1 — Plan
- **Request:** TICKET-580, filed in #518's Test: a drag in a Browser tab seemed to leave no
  selection once the button was up, so #518's scenario selected with a double click.
- **Classification / tier:** bug, prong 3, small. The finding turns it into a correction with no
  code.
- **Pre-flight:** clean. cargo 1.98.1; the gate and the e2e runner present; the hooks wired; no
  active pipeline; the README marker present; cargo idle.
- **Recall (§18.3):**
  - F-claude-518-a-drag-in-a-browser-tab-leaves-no-selection-001 and
    L-claude-518-select-with-a-double-click-in-a-scenario-001: #518's reading, which the probes
    below overturn.
  - AD-claude-487: the headless sway's virtual pointer goes through the platform layer, so what
    the page logs is what a mouse would give it.
  - L-claude-474: a check must first show that it can see what it rules out. #518's page log was
    read through `browser_snapshot`, whose cut ("…") hid the moment the selection collapsed.
  - Brain consultation 997c07bdc6ea4a8fae70a89707b470e0: nothing on this seam.
- **Discovery** (probes in the session's scratchpad, run in headless sway against the debug
  build at `f79510b63f`):
  - `580-probe.sh`: #518's `select.html` with a script that logs each mouse, pointer, focus and
    selection event to the console, with the selection at that moment, read with `mcp_agent
    console`. A slow drag from x 42 to 420 on the sentence's row gives:
    - `pointerdown` and `mousedown` (detail 1, buttons 1);
    - the selection growing on the moves, up to "Select this sentence for the agent.";
    - `selectionchange` to "" on the move past the text's end (x 402), and a move to 420 with
      the selection empty;
    - then `pointerup`, `mouseup` and `click` with it still empty.
    `browser_look` read "" before the release as well as after. The double click shows
    `mousedown` with detail 2 and "sentence" selected, and the selection stays through `mouseup`,
    `click` and `dblclick`.
  - `580-probe-b.sh`, three slow drags on the same row:
    - on `select.html`, from 42 to 300, ending inside the text: "Select this sentence for the a"
      after the release;
    - on `select.html`, from 42 to 420: "";
    - on `flow.html`, the same sentence in normal flow (the body's margin 40 px, the paragraph a
      block), from 42 to 420: "Select this sentence for the agent.", through `mouseup`.
  - So the release never lost anything. #518's paragraph is `position: absolute`, and so is the
    list, the body's only other child, so the body has no height. A point right of the text on
    that row lies over no text box, and Chromium's hit test there gives a position that collapses
    the selection. The fast drag of #518's first runs also ended past the text.
- **Decisions:** D1 to D3 in the spec.

### Design
- **Approach:** no Marley change.
  - `script/e2e/518-fuller-pick-bundle.sh` gets back a slow drag helper (steps of 60 px with a
    pause after each, as a hand moves) and drags from x 42 to 300 on row 52. The check reads a
    `selected_text` that begins "Select this sentence".
  - The corrections:
    - remove the known limit from `docs/marley_architecture/marley_workbench.md`;
    - replace F-claude-518-a-drag-in-a-browser-tab-leaves-no-selection-001 and
      L-claude-518-select-with-a-double-click-in-a-scenario-001 with one lesson: the finding and
      how it was missed;
    - update REQ-008's verification in #518's archived spec, and add a correction entry to its
      archived notes;
    - close TICKET-580 as not a Marley bug.
  - The ticket's BACKLOG row went at promotion.
- **File manifest:** `script/e2e/518-fuller-pick-bundle.sh` (a scenario). The rest is docs. No
  Marley crate, no Zed crate, no ledger row.

### E2E plan

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-001 | `518-fuller-pick-bundle.sh`, `select.html`: a slow drag from x 42 to 300 on the sentence's row, then the pick of Second item | the log: `selected_text` begins "Select this sentence"; `518-04-selection-pick` shows the text still selected |
| REQ-002 | a grep of `docs/` for "leaves no selection" and "does not outlast the release" | only this pipeline's correction entries |

### Risks
- The drag's end is set in page pixels. x 300 lands inside "agent" with the fonts on the box, and
  the check needs only the selection's start, so any font that takes the drag past "sentence"
  still passes.
- **Phase 1 checklist** (no task tool in this session): pre-flight; recall; the two probes; the
  pair minted; the BACKLOG row removed; the ticket in progress; the prior art; the spec; the
  design. Status: Plan PASS.

## Phase 2 — Code
- **Nothing built.** The diagnosis found no Marley defect (D1), so no application code changes.
  The only gated file the ticket touches is #518's scenario, which is the Test phase's.

## Phase 3 — Test
- **The scenario:** `script/e2e/518-fuller-pick-bundle.sh` got back a slow drag helper
  (`drag_across`: steps of 60 px with a pause after each) and drags from x 42 to 300 on the
  sentence's row. The check reads a `selected_text` that begins "Select this sentence". Shellcheck
  is clean.
- **The run** (`518-run3.log`): 16 checks pass, none fail. The list item's pick after the drag
  carries `selected_text` 'Select this sentence for the a' (REQ-001). Every other check reads as
  in #518's run.
- **The shot** `518-04-selection-pick`: "Select this sentence for the a" still selected after
  Pick 7 of Second item, the tray's row under the toolbar (REQ-001).
- **Focus:** "hyprland: 0 Marley windows before the run, 0 after; the run added no rule and did
  not reload it".
- **REQ-002:** a grep of `docs/` and `script/e2e/` for "leaves no selection", "does not outlast",
  "outlast the release" and the two replaced codes finds only this pipeline's own pair, which
  names what it replaced.
- **The gate:** `script/gates.sh --diff` (`gate-580.log` in the session's scratchpad): 16
  passed, 0 failed, `GATE GREEN [diff]`, with the receipt.
- **Verdict:** Phase 3 PASS.

## Phase 4 — Complete
- **Documented:** the known limit withdrawn from `docs/marley_architecture/marley_workbench.md`;
  #518's archived spec (REQ-008's verification) and notes (the plan's row, the selection bullet,
  the Complete entry, and a correction section). No `CHANGELOG.md` entry, since nothing a user
  sees changed and the changelog never carried the claim. No plan row: no slice ships. No Zed
  path was touched.
- **Knowledge:** F-claude-518-a-drag-in-a-browser-tab-leaves-no-selection-001 removed (no such
  bug); L-claude-518-select-with-a-double-click-in-a-scenario-001 replaced by
  L-claude-580-a-drag-past-an-absolutely-placed-lines-end-collapses-the-selection-001.
- **Brain:** consultation 997c07bdc6ea4a8fae70a89707b470e0 closed with
  `decisions/a-browser-tab-keeps-a-drags-selection-580-needs-no-fix-580`.
- **Closed and archived:** the ticket, as no Marley bug, in `docs/planning/tickets/closed/`; this
  pair in `docs/planning/pipeline/completed/`.
