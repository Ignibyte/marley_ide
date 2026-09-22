# line-number gutter — Notes

- **Forge ticket:** #100 `7af97ea6-a745-4f4c-9246-07c316beaf85` · **AAR:** `b95bfee9-1fd6-49ad-931a-7590205ce5bd`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-100-gutter.md

## Phase 1 — Plan
- **Request:** forge #100 (M4 4/10) — the viewer's line-number gutter.
- **Pre-flight:** the #97 viewer already renders `line.number.to_string()` muted; formalize via gutter_width
  + gutter_label. The current-line highlight couples to #101's cursor (deferred).
- **Decisions:** D1 gutter_width=to_string().len().max(2), gutter_label=right-align; D2 current-line = #101.
- **AAR id:** `b95bfee9-1fd6-49ad-931a-7590205ce5bd`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **code_view.rs (PURE):** `pub fn gutter_width(line_count: usize) -> usize { line_count.to_string().len().max(2) }`; `pub fn gutter_label(n: usize, width: usize) -> String { format!("{n:>width$}") }`.
- **app.rs SHIM:** in the #97 viewer render, `let gw = gutter_width(cv.lines.len());` before the line loop; replace `line.number.to_string()` with `gutter_label(line.number, gw)`.
- **Mutation targets:** gutter_width to_string().len() + .max(2); gutter_label right-align width.
- **Test plan:** gutter_width_by_count (1→2,9→2,10→2,99→2,100→3,1000→4) + gutter_label_right_aligns ((5,3)→"  5",(42,3)→" 42",(100,3)→"100"). cov/MSI 100. The render masked.
- **Risks:** the current-line highlight lands in #101 (the movable cursor); gutter_width min-2 keeps single-digit files aligned.

## Phase 3 — Implement
- **Built:** gutter_width(line_count) + gutter_label(n, width) (code_view.rs); the #97 viewer computes `gw = gutter_width(cv.lines.len())` once + renders `gutter_label(line.number, gw)` (muted).
- **Verification:** fmt; check 0 err; clippy OK; both gutter tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review (two trivial pure formatters + a masked render swap).
- **Lenses — no findings:** gutter_width = to_string().len().max(2) (digit count + min-2, tested 1/9/10/99/100/1000); gutter_label = format!("{n:>width$}") right-align (tested 5/42/100 in width 3); the render computes gw once (not per line) + swaps the raw number for the aligned label. No panics. No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** gutter_width_by_count + gutter_label_right_aligns. `cargo nextest` → pass.
- **Self-test:** the gutter render needs a viewer open (synthetic, ENV-BLOCKED); engine-tested cov/MSI 100.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100.

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #100 → done. **M4 4/10.** gutter_width + gutter_label (cov/MSI 100) + the muted right-aligned gutter render.
