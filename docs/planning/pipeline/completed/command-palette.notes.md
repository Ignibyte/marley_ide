---
pipeline_id: f142d14f-1d43-4409-9965-541d5f02896b
aar_id: 05ba6945-9bcc-41a7-bcbf-b7c49674f49a
---

# command palette — pipeline notes

## Phase 1 — Plan (2026-07-01)

**Intent:** the SPEC-app-shell command palette (R14-R18) in marley_app — a pure `filter_commands` (nucleo
fuzzy ranking) + the centered overlay. M1.B Cockpit seq 3/5.

## Carry to Design (Phase 2)

### palette.rs (PURE — cov 100/MSI 100)
- `#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub struct CommandId(pub u32);`
- `pub struct Command { pub id: CommandId, pub title: String, pub keywords: Vec<String>, pub binding:
  Option<KeyBinding> }` (KeyBinding from #18's keymap.rs — `use crate::keymap::KeyBinding`).
- `pub struct ScoredCommand<'a> { pub command: &'a Command, pub score: u32 }` (derive Debug; NO Default →
  the whole-fn Default mutant on filter_commands is unviable).
- `pub fn filter_commands<'a>(commands: &'a [Command], query: &str) -> Vec<ScoredCommand<'a>>`:
  ```
  if query.is_empty() {                                   // R15 short-circuit
      return commands.iter().map(|c| ScoredCommand { command: c, score: 0 }).collect();
  }
  let mut matcher = nucleo::Matcher::new(nucleo::Config::DEFAULT);
  let mut scored: Vec<(usize, ScoredCommand)> = commands.iter().enumerate()
      .filter_map(|(idx, cmd)| {
          let best = std::iter::once(&cmd.title).chain(cmd.keywords.iter())
              .filter_map(|field| field_score(&mut matcher, field, query))
              .max();                                       // MAX field score (R16); None → excluded
          best.map(|score| (idx, ScoredCommand { command: cmd, score }))
      }).collect();
  scored.sort_by(|a, b| b.1.score.cmp(&a.1.score).then(a.0.cmp(&b.0)));  // desc score, ties by asc reg idx
  scored.into_iter().map(|(_, sc)| sc).collect()
  ```
- `fn field_score(matcher: &mut nucleo::Matcher, field: &str, query: &str) -> Option<u32>`:
  fresh `Vec` scratch buffers per call; `matcher.fuzzy_match(Utf32Str::new(field, &mut hb),
  Utf32Str::new(query, &mut nb))`. **WRITE-THEN-CHECK the exact nucleo 0.5.0 API at implement** (Matcher/
  Config/Utf32Str::new signatures) — do NOT pre-research. Case-insensitivity: verify nucleo's Config
  (DEFAULT is smart-case; if a mixed-case query must still match, lowercase both OR use the ignore-case
  Config so R16's "case-insensitive" is literal).

### app.rs (SHIM — already mutants::skip + cov-excluded)
- A palette state on RootView: e.g. `palette_open: bool` + a query buffer (reuse the marley_editor input
  or a String) + the current `Vec<ScoredCommand>` / selection. The command list is a static
  `Vec<Command>` (a `commands()` builder — Toggle Theme / Split Pane / Close Pane etc., each with its
  keymap binding).
- on the keymap `open-command-palette` action (the #18 interception point) → open the overlay (R14).
- filter-as-you-type → `filter_commands(&self.commands, &self.query)`; render the centered overlay
  (raw gpui or the #17 dialog scrim) listing the results.
- activate (Enter on a selection) → dispatch the command's bound action + close (R17); Escape → close, no
  dispatch (R18). These dispatch/close transitions are shim.

### Mutation map (filter_commands — the spec's 5 targets)
- **empty-query short-circuit dropped** → an empty query would hit nucleo (reorders / different) — killed
  by R15 asserting all-in-REG-order (a nucleo pass would not preserve reg order across differing scores).
- **match predicate inverted** (include a non-match) → killed by R16 asserting a non-matching command is
  EXCLUDED (e.g. query "zzz" against a command with no z-subsequence → absent).
- **MIN instead of MAX field score** → killed by the EMPIRICAL fixture (a command C1 with 2 matching
  fields [scores s_hi > s_lo] + a command C2 with one field [s_lo < s_C2 < s_hi]; with MAX, C1 ranks
  above C2; with MIN, below) — build at validate from nucleo's actual scores.
- **sort order flipped** (asc not desc) → killed by R16 asserting the stronger match ranks FIRST.
- **reg-order tie-break dropped** → killed by 2 commands with the SAME score (e.g. identical titles, or a
  query equal to both) asserting the earlier-registered one comes first.
- FIXTURES: a `commands()` builder (Toggle Theme / Open Settings / Split Pane …) with distinct titles +
  keywords; the tie fixture (2 commands, same-scoring field); the min/max fixture (built empirically).

### Tests (validate)
- `filter_empty_query_returns_all_in_reg_order` (R15).
- `filter_ranks_nucleo_matches_and_breaks_ties_by_reg_order` (R16 — membership + ordering + the tie +
  the min/max ordering).
- The `#[ignore]` headed palette test (open the overlay via the gallery/app + assert non-blank) — OR ride
  the existing headed_shell (the overlay is shim; a minimal headed assertion). Decide at validate.

**RISK:** nucleo's API shape (Matcher/Utf32Str) — write-then-check (the #16 lesson). The min/max fixture
is empirical (nucleo's internal scores). Case-insensitivity must be literal (R16).

**Phase 1 status:** PASS (autonomous). → Phase 2 Design.

## Phase 2 — Design (2026-07-01)

**Confirmed the Carry-to-Design against SPEC-app-shell's Public surface — MATCHES.** `Command { id:
CommandId, title: String, keywords: Vec<String>, binding: Option<KeyBinding> }`, `CommandId(u32)`
(→ `CommandId(pub u32)`), `ScoredCommand<'a> { command: &'a Command, score: u32 }`, `filter_commands<'a>(
&'a [Command], &str) -> Vec<ScoredCommand<'a>>` — all realised exactly; the model reuses #18's
`crate::keymap::KeyBinding` for `Command.binding` (single-owner, seam-clean).

- **Seam CONFIRMED:** palette.rs (the model + `filter_commands` + the private `field_score`) is PURE (a
  deterministic nucleo dep, no gpui/IO) → cov 100 / MSI 100. The overlay STATE + Render + activate/Escape
  dispatch are SHIM in app.rs (already `mutants::skip` + rust_cov-excluded from #16 — no gates.sh change).
- **Mutation map CONFIRMED — covers the spec's 5 targets:** empty-short-circuit (R15 reg-order),
  invert-match-predicate (R16 non-match excluded), MIN-not-MAX (the empirical 2-field fixture),
  sort-flip (R16 stronger-first), tie-break-drop (2 same-score commands → earlier-reg first). The
  whole-fn `Default::default()` mutant on `filter_commands` is UNVIABLE (Vec/ScoredCommand path, no
  Default return that type-checks trivially — confirm at inspect via `cargo mutants --list`).
- **Test plan CONFIRMED:** `filter_empty_query_returns_all_in_reg_order` (R15) +
  `filter_ranks_nucleo_matches_and_breaks_ties_by_reg_order` (R16). R16 asserts MEMBERSHIP + ORDERING +
  the tie + the min/max ordering — NEVER a literal nucleo score (internal/fragile). The min/max fixture
  is built EMPIRICALLY at validate (run filter_commands, read the actual scores, pick fields s.t. C1.max
  > C2 > C1.min). Uncoverable: the gpui overlay render (shim, headed-only).
- **nucleo API (write-then-check at implement — the #16 lesson):** `Matcher::new(Config::DEFAULT)` +
  `fuzzy_match(Utf32Str, Utf32Str) -> Option<u32>` + `Utf32Str::new(s, &mut Vec)` — the exact 0.5.0
  signatures are verified by the compiler, not pre-researched. **Case-insensitivity (R16 "case-insensitive"
  is LITERAL):** verify nucleo's Config; if DEFAULT is smart-case only, lowercase both field+query (or use
  the ignore-case Config) so a mixed-case query still matches.

**Phase 2 status:** PASS. → Phase 3 Implement.

## Phase 3 — Implement (2026-07-01)

Built (in-context; nucleo API settled by write-then-cargo-check — the #16 discipline, 1 iteration):
- **crates/marley_app/src/palette.rs (PURE, nucleo-only, NO gpui):** `CommandId(pub u32)` + `Command {
  id, title, keywords, binding: Option<KeyBinding> }` (+ PartialEq/Eq) + `ScoredCommand<'a> { command, score:
  u32 }` + `filter_commands` (empty→all reg-order score 0; else nucleo max-field-score, sort desc + reg-tie)
  + `field_score` (private).
- **crates/marley_app/src/lib.rs:** `mod palette;` + `pub use palette::{filter_commands, Command,
  CommandId, ScoredCommand};`.
- **crates/marley_app/src/app.rs (SHIM — every new fn `mutants::skip`):** RootView gains `palette_open:
  bool` + `palette_query: String` + `commands: Vec<Command>` (init in `new`); `cockpit_commands()` (the 4
  static commands: Split/Close Pane + Toggle Theme + Open Settings); `handle_palette_key` (Escape/Enter→
  close [R18/R17], backspace/type→query); on_key_down routes to the palette when open + opens it on the
  keymap `open-command-palette` action (R14); the render overlays a centered result list when open.
- **Cargo.toml:** `nucleo = "0.5.0"` (deny licenses OK — no allow-list change needed).

**nucleo 0.5.0 API (the settled shape):** `nucleo::Matcher::new(nucleo::Config::DEFAULT)` +
`matcher.fuzzy_match(nucleo::Utf32Str::new(field, &mut Vec::new()), Utf32Str::new(query, &mut Vec::new()))
-> Option<u16>`. DEVIATION: nucleo returns `u16`, the spec's `ScoredCommand.score` is `u32` → `field_score`
maps `.map(u32::from)`. Case-insensitivity (R16 literal): `field.to_lowercase()` + `query.to_lowercase()`
before the `Utf32Str` (Config::DEFAULT is smart-case; explicit lowercasing makes it unconditional).

**Verify:** cargo check + clippy -D + fmt + rustdoc -D + no-`unsafe` (palette.rs) + `cargo deny check
licenses` = OK. palette.rs has 0 gpui refs (PURE).

**Carry to Validate:** the 2 tests (R15 empty→reg-order; R16 the empirical fixture). **INSPECT must run
`cargo mutants --list -p marley` to see which of the 5 targets are VIABLE** — esp. whether the MAX (the
`current.max(score)` reduce) generates a min-mutant. Build the min/max fixture empirically only if viable.

**Phase 3 status:** PASS. → Phase 3.5 Inspect.

## Inspect (Phase 3.5) — 2026-07-01

Inspected in-context (`cargo mutants --list -p marley` is the oracle). **No findings.** Lenses: mutant
surface, seam, clean-room, §14.

- **Seam CONFIRMED:** `cargo mutants --list` shows **0 app.rs mutants** (handle_palette_key/cockpit_commands
  /render/on_key_down all covered by app.rs's `mutants::skip` + cov-exclude). The only listed mutants are
  in palette.rs.
- **Clean-room:** 0 `warp` refs in palette.rs; `Command`/`ScoredCommand`/`filter_commands` Marley-original.
  **§14:** no unwrap/expect/panic (to_lowercase/fuzzy_match/sort/map_or are total).

**KEY FINDING (simplifies validate) — the min/max/sort/tie targets are NOT viable mutants.** cargo-mutants
generates only WHOLE-FN replacements for this code shape; it does NOT mutate `current.max(score)`,
`b.1.score.cmp(&a.1.score)`, or `.then(a.0.cmp(&b.0))`. So the spec's "min-not-max / sort-flip /
tie-break-drop" targets have **no viable mutant** → coverage-only (no empirical min/max fixture needed).

**CARRY TO VALIDATE — the mutation kill map (5 listed, 4 viable):**
- `field_score -> None` (L42) — killed by **R16**: a MATCHING command must be PRESENT (with None,
  `filter_commands` returns empty for a non-empty query → the match absent).
- `field_score -> Some(0)` (L42) + `field_score -> Some(1)` (L42) — killed by **R16**: a NON-matching
  command must be ABSENT (with `Some(_)`, every field "matches" → a non-match is wrongly included).
- `filter_commands -> vec![]` (L61) — killed by **R15** (empty query → ALL, not empty) + R16 (a match →
  non-empty).
- `filter_commands -> vec![Default::default()]` (L61) — **UNVIABLE** (`ScoredCommand` has a `&'a Command`
  field → no `Default` → won't compile).

**COVERAGE requirements for the R16 test (100% lines):** the fixture must include (a) ≥2 MATCHING commands
(exercise `sort_by`), (b) a command with **2 matching fields** — title AND a keyword both match the query
— to run the `best.map_or(score, |current| current.max(score))` reduce closure, (c) a NON-matching command
(the `filter_map` None → exclusion path). The `.then` tie-break arg is eagerly evaluated on any ≥2-result
sort (covered). Build the fixture EMPIRICALLY (run it, confirm the ordering + the 2-field command).

**Tests for validate:** `filter_empty_query_returns_all_in_reg_order` (R15) +
`filter_ranks_nucleo_matches_and_breaks_ties_by_reg_order` (R16 — present + absent + ordering + a
2-matching-field command for coverage). NO exact nucleo-score asserts.

**Phase 3.5 status:** PASS. → Phase 4 Validate.

## Phase 4 — Validate (2026-07-01)

Wrote the 2 tests in-context; the R16 fixture is empirically verified (nucleo behaves as designed).
`palette.rs` gained `#[cfg(test)] mod tests`: `filter_empty_query_returns_all_in_reg_order` (R15 →
[0,1,2]) + `filter_ranks_nucleo_matches_and_breaks_ties_by_reg_order` (R16 → query "pane" gives [0,1];
Toggle Theme excluded; Split Pane before Close Pane on the equal keyword score — the two-matching-field
commands exercise the `.max` reduce; the non-match exercises the exclusion path).

`cargo nextest run -p marley palette` = **2 passed** (the fixture verified: nucleo excludes the non-match
+ orders as expected — no fixture adjustment needed).

**FULL gate (scripts/gates.sh --diff): `GATE GREEN [diff]` — 15/15 on the FIRST run**: cov 100% (palette.rs
— the reduce/sort/filter_map both-branches all covered by the fixture; app.rs shim excluded), mutation
MSI 100% (the 4 viable mutants killed — field_score→None/Some(0)/Some(1) + filter_commands→vec![]; the
vec![Default] mutant unviable), gate:8 deny GREEN (nucleo license OK — no allow-list change), gate:9
machete GREEN (nucleo used by palette.rs), gate:15 PASS (the shell_dark headed lane). Receipt written.

**Phase 4 status:** PASS. → Phase 5 Complete.
