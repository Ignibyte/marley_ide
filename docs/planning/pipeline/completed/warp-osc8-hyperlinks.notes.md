# Honor OSC 8 explicit hyperlink escapes in terminal output — Notes

- **Forge ticket:** #214 (c9d31e78-86c0-4eda-9692-75876d29b5b5)
- **AAR:** 91bfb0b8-a74d-45a1-af6a-08544f740b7c
- **Local ticket doc:** docs/planning/tickets/open/TICKET-214-warp-osc8-hyperlinks.md
- **Pipeline spec:** warp-osc8-hyperlinks.spec.md

## Phase 1 — Plan
- **Request:** honor OSC 8 explicit hyperlink escapes in terminal output — the deferred half of #196
  (which shipped text-scan links but punted OSC 8 to a separate `terminal_blocks` change).
- **Classification / tier:** work pipeline · feature · M12.2 · ONE shippable slice (cross-crate:
  `terminal_blocks` capture-model + `marley_app` render, but one coherent delta on the #196 foundation).
  The ticket flags a possible capture-vs-render split; assessed as one slice (like #205) — Design reconfirms.
- **Forge recall (§18.3):** `knowledge-search` (OSC8/hyperlink/coalesce/render) surfaced generic
  prevention-rule/AD nodes; `docs-search` hit another project's corpus (oathstar-studio) — not relevant.
  Directly-applicable rules recalled from memory:
  - `PR-derived-value-change-must-sweep-all-consumers` (M1.E) — a new `StyledRun` field ripples to EVERY
    constructor/consumer: `plain_lines` (styled.rs:82), the coalesce test literals, and the render. `cargo
    check --all-targets` is the compiler-guided sweep.
  - `PR-claude-cargo-mutants-guard-mutants-depend-on-syntactic-form` — the new break condition rides the
    EXISTING `match runs.last_mut() { Some(run) if GUARD => … }` arm (styled.rs:37), a match-guard → yields
    the FULL true/false/delete-! mutant set (unlike an `if let`/`if !`). RUN `--list` on the real code.
  - `PR-assert-the-raw-representation-not-a-normalizing-projection` (#50) — the `output_text`-unchanged
    guard MUST assert the RUN STRUCTURE (2 runs) AND the flattened text (unchanged), not just the flattened
    text (a normalizing projection that would pass vacuously).
  - `PR-claude-shim-needs-both-cov-exclude-and-mutants-skip` — the session.rs grid-read extraction is the
    masked shim; confirm it carries BOTH the cov-exclude regex AND `mutants::skip` if it's not R26-covered.
  - `no-default-struct-return-needs-full-value-assert` — `StyledRun` has no `Default`; assert full values.
- **Discovery (the precise edit surface for Design):**
  - **`terminal_blocks/src/styled.rs`** — `StyledRun { text, fg, bg, flags }` (:15, derives `Clone,PartialEq,Eq`);
    `coalesce_row(cells: impl IntoIterator<Item = (char, Color, Color, Flags)>) -> StyledLine` (:33); the
    coalesce guard is the `Some(run) if run.fg==fg && run.bg==bg && run.flags==flags` arm (:37); the trailing-
    trim (:48-58) operates on run text only; `plain_lines` test helper (:82) builds one run per non-empty line.
    Exported at lib.rs:50 (`coalesce_row, StyledLine, StyledRun`).
  - **`terminal_blocks/src/session.rs`** — two cell-tuple build sites feed `coalesce_row`: ~:428 (block-output
    capture path) and ~:450 (`grid_styled_rows`, :190). Each maps a grid cell to `(cell.c, cell.fg, cell.bg,
    cell.flags)`. `grid_styled_rows` IS unit-covered (R26 test `grid_styled_rows_reflects_grid` :1091) — so an
    integration test feeding real OSC 8 bytes is the natural REQ-004 proof. Design confirms mutants::skip vs
    covered per fn.
  - **alacritty_terminal 0.26** — `Cell::hyperlink() -> Option<Hyperlink>`; `Hyperlink::uri() -> &str` (vendored
    at `~/.cargo/registry/.../alacritty_terminal-0.26.0/src/term/cell.rs`). The URI is already parsed by alacritty.
  - **`marley_app/src/links.rs`** — `LinkTarget` enum (:15), `Link`, `scan_links(line: &str) -> Vec<Link>` (:51);
    the #196 linkify trim rules. Reuse `Link`/`LinkTarget` for the explicit composer.
  - **`marley_app/src/app.rs`** — the output render loops `state.session.grid_styled_rows()` (:4665) and scans
    links (~:4888); `marley_command::open_url(url)` at :2131. The render is the shim that will read `run.hyperlink`.
- **Decisions:** D1 inline `Option<String>` URI (not an intern map); D2 `as_deref()` break compare (no per-cell
  clone); D3 explicit hyperlink authoritative over the heuristic; D4 `output_text` byte-identical (hard guard);
  D5 URI verbatim from alacritty's parse (no re-parse; clean-room §20 — OSC 8 is a public standard).
- **Ripple surface (the `derived-value-change-must-sweep-all-consumers` sweep, workspace-grepped):**
  - `coalesce_row` callers = ONLY session.rs:430 + :452 (the 2 shim sites); `lib.rs:50` re-exports it; no
    external-crate caller → the signature change is contained.
  - `StyledRun { … }` literals needing the new field: styled.rs:38 (the coalesce push — sets the value),
    styled.rs:89 (`plain_lines` → `hyperlink: None`), the styled.rs `#[cfg(test)]` expected-value literals
    (~:100+), and **`marley_app/src/color.rs:323`** (a cross-crate test-helper closure → `hyperlink: None`).
  - **Crate naming:** dir `crates/terminal_blocks`, but its lib is named `marley_terminal` (consumers import
    `marley_terminal::StyledRun`, e.g. color.rs:6) — mirrors the marley_app dir/package/lib split.
  - `marley_app` only READS run fields (`color.rs::run_paint` :126, reads `run.fg`/`run.flags`) — never
    constructs `StyledRun` outside that one test helper → the render adaptation is read-only (add a
    `run.hyperlink` read, no construction).
- **Env note:** machine was LOCKED at #205 validate (env-blocked driven capture). REQ-006 driven capture is
  best-effort; the fallback is the REQ-001..005 units + the REQ-004 integration test + the shipped #196
  mechanism. Batch: 5 unpushed (#200/#201/#203/#204/#205) — offer the push when chad's back.

## Phase 2 — Design

### Architecture / approach
One coherent slice (the capture + render are tightly coupled — the render needs the captured URI —
and each piece is small; NO capture-vs-render split despite the ticket's flag). Data flow: an alacritty
grid cell's `hyperlink()` URI → carried through `StyledRun.hyperlink` by `coalesce_row` (a run breaks on
a URI change) → the render composes a line's links preferring the explicit URI over the `scan_links`
heuristic, then reuses the UNCHANGED #196 `split_run_by_links` + `open_link_target`.

**PURE #1 — `terminal_blocks/src/styled.rs`:**
- `StyledRun` gains `pub hyperlink: Option<String>` (D1 inline URI). Derives are fine (Option<String>: Debug/Clone/PartialEq/Eq).
- `coalesce_row`'s item becomes a 5-tuple `(char, Color, Color, Flags, Option<String>)` (minimal — extends the
  existing 4-tuple idiom; a struct was considered but the 5-tuple keeps the diff small). The guard (:37) gains
  a 4th term: `run.fg == fg && run.bg == bg && run.flags == flags && run.hyperlink == hyperlink`.
  **D2 REVISED:** compare `run.hyperlink == hyperlink` DIRECTLY (an `Option<String> == Option<String>` borrows
  both operands — NO per-cell clone), matching the existing `run.fg == fg` idiom; `as_deref()` adds noise for no
  benefit (the borrow already avoids the clone; the owned `String` moves into the run only at a run-break push).
- The push (:38-43) sets `hyperlink` (moved). The trailing-trim (:48-58) is UNTOUCHED (mutates `text` only;
  a trailing hyperlinked-blank run drops like any blank).
- **Mutant delta (confirmed via `cargo mutants --list` on the current file):** the baseline guard = `true`/`false`
  + 3×`==`→`!=` (fg/bg/flags) + 2×`&&`→`||`, all killed by the existing :151-186 tests. Adding the 4th term adds
  EXACTLY 2 new viable mutants: a 4th `==`→`!=` (hyperlink) + a 3rd `&&`→`||` (flags/hyperlink join). Kills: the
  new `==`→`!=` dies to a same-URI-merges OR different-URI-splits test; the new `&&`→`||` dies to the EXISTING
  flags-split test (both runs `None`, so `hyperlink==hyperlink` is `None==None`=true → `(fg&&bg&&flags)||true`
  merges a flags-differing pair → the test's 2-run assert kills it). MSI stays 100.

**SHIM #1 — `terminal_blocks/src/session.rs` (masked, but COVERED not skipped):** the two cell closures
(`term_to_styled_rows` :426-428, `full_term_to_styled_rows` :448-450) extend the tuple with
`cell.hyperlink().map(|h| h.uri().to_string())` (alacritty `Cell::hyperlink(&self) -> Option<Hyperlink>` :219;
`Hyperlink::uri() -> &str`; the `Option<Hyperlink>` is by-value so `.map` borrows fine). These fns are NOT
`mutants::skip` — they're covered (R26 `grid_styled_rows_reflects_grid` :1091; full's range mutants killed at
:740). So the extraction adds a covered+mutated line → **needs a killing assert (D7 → the REQ-004 integration
test)**; it can NOT be `mutants::skip`'d without lowering the floor (§0).

**PURE #2 — `marley_app/src/links.rs`:** a new
`pub fn line_links(runs: &[(&str, Option<&str>)]) -> Vec<Link>` (D6): walk `runs` tracking a byte cursor;
a run with a non-empty `Some(uri)` → push `Link { range: byte..end, target: LinkTarget::Url(uri.into()) }` AND
record `byte..end` as an explicit range; then `scan_links(<line rebuilt from runs>)` adds each heuristic Link
EXCEPT any whose range overlaps an explicit range (a small `fn ranges_overlap(a,b) = a.start < b.end &&
b.start < a.end`); finally `sort_by_key(range.start)`. Builds the line internally from `runs` (single source of
truth — no desync vs a passed line_text). Reuses the existing `Link`/`LinkTarget`; downstream
`split_run_by_links` is UNCHANGED. **Back-compat:** with all-`None` runs, `explicit` is empty → returns
`scan_links(line)` (ordered) exactly as #196. **Authoritative (D3):** an explicit run whose text ALSO scans as a
URL → the scanned one overlaps → dropped → only the explicit URI wins.

**SHIM #2 — `marley_app/src/app.rs` (masked render):** ONE change at :4897 — replace
`let links = scan_links(&line_text);` with
`let runs: Vec<(&str, Option<&str>)> = line.iter().map(|r| (r.text.as_str(), r.hyperlink.as_deref())).collect();
let links = line_links(&runs);`. `line_text` is still built (:4895) for `line_text[seg.range]` slicing; the per-run
`split_run_by_links` loop (:4900-4942) + `open_link_target` are UNCHANGED. Scope: the block-output render (the
#196 target = scrollback); the alt-screen grid render is out (full-screen programs own their screen — same as #196).

### File manifest
| File | Kind | Change |
|---|---|---|
| `crates/terminal_blocks/src/styled.rs` | PURE | `StyledRun.hyperlink: Option<String>` + doc; `coalesce_row` 5-tuple + guard-D `run.hyperlink == hyperlink` + push sets it; `plain_lines`→`hyperlink: None`; compile-fix the existing coalesce test tuples (`, None`); P4 adds `coalesce_row_splits_on_hyperlink_change` |
| `crates/terminal_blocks/src/session.rs` | SHIM+test | the 2 cell closures (:428,:450) add `cell.hyperlink().map(\|h\| h.uri().to_string())`; P4 adds an OSC-8 integration test (feed the escape via the scripted PtyChannel, pump, assert `grid_styled_rows` carries `Some(uri)`) |
| `crates/marley_app/src/links.rs` | PURE | new `line_links(runs) -> Vec<Link>` + `ranges_overlap`; P4 adds the L1-L5 exact-value tests |
| `crates/marley_app/src/app.rs` | SHIM | :4897 build `runs` from `line` + call `line_links` (replaces `scan_links(&line_text)`) |
| `crates/marley_app/src/color.rs` | compile-fix | the :323 `StyledRun` test literal → `hyperlink: None` |

No `lib.rs` changes (`StyledRun`/`coalesce_row` already `pub` re-exported at terminal_blocks lib.rs:50; `line_links`
is a new `pub fn` in the app's own `links` module — add it to app.rs's `use crate::links::{…}`).

### Regression Test Plan (≥1 per REQ)
| REQ | Test | Asserts / kills |
|---|---|---|
| REQ-001 | styled `coalesce_row_splits_on_hyperlink_change` — same-URI case `[('a',fg,bg,∅,Some("u")),('b',…,Some("u"))]` | 1 run, `text=="ab"`, `hyperlink==Some("u")` (carry) |
| REQ-002 | same test — different-URI `Some("u")`/`Some("v")` + Some→None cases | 2 runs each; kills the new `==`→`!=` (hyperlink) |
| REQ-003 | same test — assert the JOINED text == `"ab"` on the 2-run cases (BOTH structure AND flatten, #50); + all-`None` case coalesces to 1 run; the existing coalesce/trim tests stay green | `output_text` byte-identical + back-compat |
| REQ-004 | session.rs OSC-8 integration test: feed `\x1b]8;;https://example.com\x1b\\click\x1b]8;;\x1b\\` → pump → `grid_styled_rows()` has a run `text=="click"`, `hyperlink==Some("https://example.com")` | the shim extraction (covers + kills the covered-path mutant) |
| REQ-005 | links.rs `line_links` L1-L5b: (L1) one explicit run `"click"`→uri, no scan → `[Url(0..5)]`; (L2) all-`None` line w/ a url → `==scan_links` (back-compat); (L3) explicit + a later `None`-url run → both, ordered; (L4) explicit range OVERLAPPING a scanned url → only the explicit URI (D3); **(L5a)** scanned STARTS at explicit END, touches-not-overlaps → BOTH kept — `runs=[("click",Some("https://real.com")),("http://x.io",None)]` → `[0..5 real.com, 5..16 x.io]` (scanned `5..16`; `a.start(5)==b.end(5)`) — kills `a.start < b.end`→`<=`; **(L5b)** scanned ENDS at explicit START → BOTH kept — `runs=[("http://x.io",None),(",",Some("u")),(" more",None)]` → scanned `0..11` (comma-trimmed) + explicit `11..12` (`b.start(11)==a.end(11)`) — kills `b.start < a.end`→`<=` | the composer's real set (offset `+`, `!uri.is_empty()`, the `!any` overlap filter, `ranges_overlap` BOTH `<` + `&&`) — inspect F1: the predicate has 2 distinct `<`→`<=` mutants, one per touching orientation |
| REQ-006 | driven capture if unlocked: `printf '\e]8;;https://example.com\e\\click\e]8;;\e\\\n'` → "click" is a link to example.com; ELSE env-blocked → REQ-001..005 units + the REQ-004 integration + the shipped #196 `Link`→`open_link_target` mechanism | end-to-end (best-effort) |

Uncoverable-path note: REQ-006's live GUI capture is env-blocked while the machine is locked (per
`PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`) — the REQ-004 integration test +
the pure units + the unchanged #196 click path carry it; gate-15 is headless so the gate stays green.

### Risks / decisions
- **R1 (load-bearing):** the alacritty `Term` (Config::default()) must populate `cell.hyperlink()` from OSC 8
  bytes for REQ-004. CONFIRM at implement — the integration test IS the proof; if a config flag gates hyperlink
  parsing, set it (alacritty parses OSC 8 in its vte processor; expected on by default). Feed bytes via the same
  scripted-PtyChannel path the existing session tests use.
- **R2:** the `coalesce_row` 5-tuple ripples to the existing styled.rs test tuples (`, None`) + `plain_lines` +
  `color.rs:323` — `cargo check --all-targets` is the compiler-guided sweep (`derived-value-change-must-sweep-all-consumers`).
- **R3:** a scanned link spanning a non-hyperlinked run INTO a hyperlinked run → `ranges_overlap` drops the scanned
  (the explicit covers its part) — conservative, acceptable v1.
- **D-revised:** D2 → direct `==` compare (no `as_deref`). D6 → `line_links(runs)` builds line+explicit internally
  (single source of truth) + reuses `split_run_by_links`. D7 → REQ-004 is a real integration test (the extraction
  is covered+mutated, not skip). D1/D3/D4/D5 unchanged (spec).

## Phase 3 — Implement
**Built (manifest as designed):**
- `terminal_blocks/src/styled.rs` (PURE) — `StyledRun` gained `pub hyperlink: Option<String>` (+ doc);
  `coalesce_row`'s item is now the 5-tuple `(char, Color, Color, Flags, Option<String>)`, the guard gained
  `&& run.hyperlink == hyperlink` (direct `Option<String>` compare, D2), the push sets `hyperlink`; the
  trailing-trim is untouched; `plain_lines` → `hyperlink: None`. Compile-fixed **9** existing coalesce test
  tuples (the 7 the design enumerated + the `blank` case + the `coalesced_runs_flatten_to_trim_end` `.map`
  closure — both part of "the existing coalesce tests", just not individually listed) — assertions intact.
- `terminal_blocks/src/session.rs` (SHIM) — both grid-read closures (`term_to_styled_rows`,
  `full_term_to_styled_rows`) extend the cell tuple with `cell.hyperlink().map(|h| h.uri().to_string())`
  (one `replace_all` — the two sites were byte-identical). No new `use` needed (`.uri()` is a method).
- `marley_app/src/links.rs` (PURE) — added `pub fn line_links(runs: &[(&str, Option<&str>)]) -> Vec<Link>`
  (explicit-preferred composer: authoritative Url per hyperlinked run + `scan_links` for the rest, dropping
  a heuristic link that overlaps an explicit range) + private `fn ranges_overlap`. Downstream
  `split_run_by_links` unchanged.
- `marley_app/src/app.rs` (SHIM) — the #196 render site (:4897) now builds
  `runs: Vec<(&str, Option<&str>)>` from the line's `StyledRun`s and calls `line_links(&runs)`; `line_text`
  kept for `line_text[seg.range]` slicing; the per-run `split_run_by_links` loop + `open_link_target` untouched.
- `marley_app/src/color.rs` — the `run_paint` test-helper `StyledRun` literal (:323) → `hyperlink: None`.

**Deviations from design (with reason):**
- **2 extra styled.rs test-tuple sites** beyond the ~7 enumerated (the `blank` two-space case + the
  `coalesced_runs_flatten_to_trim_end` `.chars().map(|c| (c, …))` builder) — both are part of "the existing
  coalesce tests"; compile-fixed with `None`. No behavior change.
- **`scan_links` REMOVED from the app.rs `use`** (not just `line_links` added): the render was its ONLY
  app.rs caller, so leaving the import would be an unused-import error. Import is now
  `use crate::links::{line_links, split_run_by_links, LinkTarget};`. Expected consequence of the swap.

**Checks:** `cargo fmt` clean; `cargo check -p marley_terminal -p marley --all-targets` ✓ (the `block v0.1.6`
future-incompat note is a pre-existing gpui-transitive dep, not this change); `cargo clippy -p marley_terminal
-p marley --all-targets -- -D warnings` exit 0. No tests expanded (Phase 4 writes the new hyperlink tests +
the OSC-8 integration test + the `line_links` L1-L5 set).

## Phase 3.5 — Inspect
2 general-purpose critics (codec/mutation/back-compat + composer/render/shim), read-only, each ran
`cargo mutants --list`. Both confirmed the CODE is CORRECT — no byte-slice panic, no back-compat regression,
`output_text` byte-identity holds, clean-room clean, the codec adds EXACTLY +2 viable mutants (a 4th `==`→`!=`
+ a 3rd `&&`→`||`) both already killed by the EXISTING all-`None` tests. Findings:

- **F1 [MED — REAL, test-plan gap] `ranges_overlap` has TWO `<`→`<=` mutants, the design's L5 covered only
  one** (Critic 2, verified). `cargo mutants --list -f links.rs` yields both `a.start < b.end`→`<=` (106:13)
  AND `b.start < a.end`→`<=` (106:32) as distinct viable mutants; each dies only at a DIFFERENT touching
  orientation (scanned STARTS-at-explicit-END vs ENDS-at-explicit-START). The design's single "touching" L5
  kills only one → the other survives → MSI < 100 at validate. Verified both orientations + the concrete inputs
  by hand. **Fix:** amended REQ-005 — L5 → L5a + L5b (above), each with concrete run inputs. The CODE
  (`a.start < b.end && b.start < a.end`) is correct + minimal → NO code change. Class rule recorded.
- **F2 [LOW — REAL, test-quality] the hyperlink field-CARRY is unpinned** (Critic 1, verified). cargo-mutants
  has no operator that rewrites the `hyperlink,` field-init to `None`, and the +2 guard mutants die to the
  existing all-`None` tests → MSI=100 does NOT require any `Some("u")` to flow through `coalesce_row`. **Fix:**
  REQ-001/002 already assert `hyperlink == Some("u")` VALUES — reaffirmed the coalesce test asserts the
  hyperlink value on every case (not just `len`), so the feature is exercised + a carry regression is caught.
- **F3 [LOW — premise correction] REQ-004 is NOT gate-forced, but KEEP it** (Critic 2). The
  `.map(|h| h.uri().to_string())` closure yields NO mutant (a method-call chain is unmutated) and the extraction
  line stays covered (same physical line as `.map`, run per cell) → neither cov nor MSI forces the OSC-8
  integration test. KEEP REQ-004 as the ONLY end-to-end proof (OSC 8 → `cell.hyperlink()` → `StyledRun` →
  `line_links`). **R1 RESOLVED:** Critic 2 traced alacritty `term/mod.rs:984` + vte `ansi.rs:1418` — OSC 8 is
  NOT config-gated, so `Term::new(Config::default())` fed the escape populates `cell.hyperlink()` → REQ-004 viable.
- **F4 [LOW — ACCEPTED, no change] transient per-cell URI `String` alloc** (Critic 1). `cell.hyperlink().map(…
  to_string())` allocates a `String` per hyperlinked cell (N alloc, N−1 dropped after the `==`); bounded by
  on-screen hyperlinked cells, the common non-hyperlinked path allocates nothing. ACCEPTED for the decoupled-
  `String` codec design; a micro-opt is a follow-up only if OSC-8-heavy output profiles hot.

No CODE defects — the diff is correct as implemented; the two test-plan amendments (F1 mandatory for MSI 100,
F2 for feature exercise) are folded into the Phase-4 plan above. Lenses covered: coalesce correctness,
byte-identity/R20b, mutation (both files), back-compat ripple sweep, UTF-8 byte-safety, line/line_text desync,
empty-URI, render preservation, extraction shim + config-gating (R1), clean-room §20.

## Phase 4 — Validate
**Tests added (4):**
- `styled.rs::coalesce_row_splits_on_hyperlink_change` (REQ-001/002/003) — same-URI merges (1 run, carrying
  `Some("u")`), different-URI splits (2 runs, joined text `"ab"`), Some→None boundary splits; asserts the
  hyperlink VALUE on every case (inspect F2 — the field-carry has no mutant, so pinned by value).
- `session.rs::grid_styled_rows_carries_osc8_hyperlink` (REQ-004) — feeds the OSC 8 escape
  `\x1b]8;;https://example.com\x1b\\click\x1b]8;;\x1b\\` through a real `Term`, asserts the live-grid run
  carries `Some("https://example.com")`. **Empirically confirms R1** (alacritty `Config::default()` populates
  `cell.hyperlink()` — NOT config-gated).
- `session.rs::finished_block_output_carries_osc8_hyperlink` (added at validate) — the command-finish path
  (`full_term_to_styled_rows`, #52 full-history capture) carries the hyperlink into the finished block's
  `output_styled`. Covers the `full_term_to_styled_rows` closure region (the coverage gap below) AND proves the
  feature through the BLOCK path, not just the live grid.
- `links.rs::line_links_prefers_explicit_over_scan` (REQ-005) — L1 (display text ≠ url → explicit wins),
  L2 (all-`None` == `scan_links`, back-compat), L3 (explicit + scanned, ordered), L4 (overlap → explicit wins),
  L5a + L5b (BOTH `ranges_overlap` touching orientations — inspect F1), + the empty-URI skip.

**RUN:** `cargo nextest run -p marley_terminal -p marley` → **424 passed, 2 skipped** (the 4 new + all existing).

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** — cov ≥100% lines + MSI ≥100% (styled.rs's +2
coalesce mutants + links.rs's line_links/`ranges_overlap` set — incl BOTH `<`→`<=` mutants killed by L5a/L5b —
all caught). Receipt written for `/commit`.

**Two source-fixes to reach green (§0 — no suppressions):**
1. **gate:1 rustfmt** — my multi-line test asserts needed reflow → `cargo fmt --all`.
2. **gate:4 coverage RED first pass** — the `full_term_to_styled_rows` `|h| h.uri().to_string()` closure region
   was uncovered: the `grid_styled_rows` OSC-8 test only traverses `term_to_styled_rows` (the live snapshot),
   while the command-finish path uses `full_term_to_styled_rows` with (until now) no hyperlinked cell. Fixed by
   adding `finished_block_output_carries_osc8_hyperlink` (feeds a hyperlink through init→preexec→output→precmd).

**REQ-006 driven capture — ENV-BLOCKED (documented):** the machine is LOCKED (the lock screen "Fri Jul 10 · 4:09
· Chad Peppers · Enter Password" — `caffeinate -u` woke the display to the lock screen, NOT a drivable desktop;
NO password attempted, per the binding constraint + `PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`).
Verified instead via: the REQ-001..005 units + the TWO REQ-004 integration tests (the end-to-end extraction proof
through BOTH the live-grid AND the command-finish paths) + the mechanism (the render reuses the shipped,
live-proven #196 `Link`→`open_link_target` path UNCHANGED — only the link SOURCE changed). gate-15 is headless →
green regardless.

**Pre-existing failures:** none.

## Phase 5 — Complete
**Docs (§21):** CHANGELOG.md — #214 under `[Unreleased] ### Added` (below #205). app_shell.md — a #214 note
appended to the #196 clickable-links section (the `line_links` explicit-preferred composer + the `StyledRun`
carry + the two grid-read extractions). terminal_blocks.md — a #214 note on the `styled.rs` bullet
(`StyledRun.hyperlink` + the coalesce break + both `term_to_styled_rows`/`full_term_to_styled_rows` extract).

**Forge capture (§19):**
- `aar-submit` 91bfb0b8 — completed, effectiveness 5. Lessons: (a) a clean bounded slice on the #196 link
  foundation — #196's `split_run_by_links` already does per-run segmentation, so #214 only produced the merged
  line `Vec<Link>` (explicit-preferred) and reused the whole downstream render+click path unchanged; (b)
  extending a coalesced style model with a run-identity field = add it to the coalesce break guard (a
  match-guard → full mutant set), assert the field VALUE not just run count (no mutant pins a field-carry — F2),
  and guard `output_text` byte-identity (assert BOTH the split count AND the unchanged flatten, #50); (c) the
  coverage near-miss below; (d) alacritty parses OSC 8 into `cell.hyperlink()` with `Config::default()` — NOT
  config-gated (the passing integration test + the critic's trace of alacritty `term/mod.rs` + vte `ansi.rs`).
- NEW `prevention-rule` **PR-claude-parallel-extraction-paths-each-need-region-test-001** (02d50858) — a shim
  extraction added to N parallel code paths needs a test per path; identical code on two paths is two distinct
  coverage regions. (The `cell.hyperlink()` extraction went into both `term_to_styled_rows` and
  `full_term_to_styled_rows`; the grid-path OSC-8 test left the command-finish closure region uncovered → cov
  gate RED → added `finished_block_output_carries_osc8_hyperlink`.)
- Recorded at INSPECT (referenced, not duplicated): `failure-record`
  **BF-claude-osc8-test-plan-under-specified-ranges-overlap-boundary-001** (dc8b43c6) + `prevention-rule`
  **PR-claude-two-comparison-overlap-needs-boundary-per-side-001** (1808f074, the F1 two-orientation MSI gap).

**Env-blocked note:** the live OSC-8 render capture was env-blocked (machine LOCKED). An optional re-verify once
unlocked: `printf '\e]8;;https://example.com\e\\click\e]8;;\e\\\n'` in a terminal → "click" renders as a link —
no separate ticket (the 2 integration tests + the unchanged #196 `Link`→`open_link_target` mechanism carry it).

**Close + archive:** forge ticket #214 → done. Local TICKET-214 → closed/. Pipeline doc pair → completed/.
Spec status → Phase 5 — Complete PASS.
