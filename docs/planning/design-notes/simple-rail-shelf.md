# The simple-rail shelf — 5 pre-authored specs (2026-08-12)

Chad's call, verbatim intent: the current left pane was his own idea and he doesn't like it —
move to a **simplistic, ChatGPT-style rail**. Projects on the left, addable. Panes stay
visible, but clicking one must NOT light its ancestors/relatives — **only the selected item
is highlighted**; everything else that is "open somewhere" gets a small **dot indicator**
instead. Visual reference he likes: <https://www.beautifului.dev/> (Sidebar Nav + Task Rows).
Batch = **M31 "the simple rail"** (418–421) + one adjacent bug (#422, M20). Pivoted to the
top of the BACKLOG queue ahead of TICKET-415 by his explicit "fixing these first".

Authored on `main` @ `a23dd0d` from two same-day Explore sweeps (Rust rail + React POC) —
every cited fact carries a file:line from that read. **Promotion still re-verifies — the
confident sentences are still the dangerous ones.**

## The shelf

| Spec | Ticket | One line |
|---|---|---|
| `418-simple-rail-react-design` | #418 | ✅ SHIPPED 2026-08-12 — the design ticket: new rail in marley-web + MARLEY-PARITY re-baseline (gates 419–421); captures 34–38 + geometry sheet are the port contract |
| `419-rail-single-selection-dots` | #419 | ✅ SHIPPED 2026-08-12 — RailSelection coordinate + RailDot; ancestor lighting dead in the pixels; driven live + parity pair |
| `420-editor-files-rail-rows` | #420 | ✅ SHIPPED 2026-08-12 — RailLevel::File per-view rows; the frozen "Editor" row retired; close persists; driven live |
| `421-rail-add-project` | #421 | ✅ SHIPPED 2026-08-12 — the header ＋ → the AddProject menu → the two shipped verbs; driven live end-to-end |
| `422-pane-divider-drag-routing` | #422 | ✅ SHIPPED 2026-08-12 — divider_rects descriptors + resize_at(path); band+seam pixel-verified; nested dividers live |

## Recommended order

418 → 419 → 420 → 421 → 422, confirmed by Chad 2026-08-12 ("as listed"). 422 is
independent — reorder freely if a quick win is wanted between ports.

## Load-bearing findings (from the 2026-08-12 sweeps)

- **The multi-highlight is structural, not a bug:** `rail_rows` (tabs.rs:1032-1256) computes
  `active` per level independently — Project `:1047`, Section `:1103/:1111`, Arrangement
  `:1146`, Pane `:1164`, Tab `:1180/:1213`, CrossRef `:1241` — and one click makes up to SIX
  rows true at once. All six render arms share one fill: `rail_highlight` (app.rs:1421-1423),
  accent @ 0.22 active / 0.10 hover. There is no "selected vs ancestor" distinction anywhere.
- **The React POC encodes the same ancestry model on purpose** (LeftRail.tsx:166-169
  `projectActive = true`) — the redesign replaces both, so **MARLEY-PARITY.md Zone A must be
  re-baselined** (rail section): the POC becomes the design source for the new rail, new
  reference captures replace shots 02/27/32/35 for the rail rows.
- **"Editor items stopped showing under the project" is not a regression** — it was #237
  (`abbdd3e`, one surface, one rail row) + #240 (`b748fa8`, label frozen to "Editor"), both
  2026-07-10, with pinned tests at tabs.rs:1464-1505. #420 reverses the rail *projection*
  only; the one-editor-surface model stays.
- **There is no rail row DnD at all** (deferred v2, recorded in
  `pipeline/completed/398-add-to-pane-cross-link.spec.md:76-78,138-141`). Chad's "drag and
  drop works only on the first item" maps to the **pane-divider drag**: handles are painted
  per flattened leaf boundary (app.rs:20628-20654) but `resize_boundary` (layout.rs:198-206)
  only mutates a top-level 2-child Split — `resize_split` no-ops for boundary ≥ 1. Plus: the
  strip is drawn at `top(0)` over the title bar (band starts at TOP_BAR_H=30), always
  vertical regardless of split axis, no pointer capture. That's #422.
- **beautifului.dev reference (captured to `docs/warp_architecture/observed/` at spec time):**
  Sidebar Nav = workspace header, quick-search with `/` keycap, small-caps section headers,
  flat icon+label rows, ONE selected row with a subtle rounded fill + count badge; Task Rows
  = left-edge status circles. That's the target grammar for the new rail.

## Recall pins (knowledge ledger)

- **F-#236** — `collapsed_projects` index-aliasing on project close; anything touching
  project add/close must remap index-keyed view state (#421 especially).
- **F-#386** — widening `rail_rows`' signature breaks ~14 `#[cfg(test)]` call sites that
  plain `cargo check` never compiles; verify with `cargo check --tests` (#419/#420).
- **L TICKET-020** — layout.rs boundary math: `then_some` evaluates eagerly; use
  `checked_sub` for index-1 patterns (#422 lives in exactly that file).
