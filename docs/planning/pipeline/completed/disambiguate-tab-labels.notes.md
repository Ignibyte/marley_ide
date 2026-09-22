# disambiguate-tab-labels — pipeline notes (forge #241, M14 sprint #27)

Pipeline: 8ae5d149-44c6-487d-8237-4bda9de2aafd · AAR: c3d175c1-4439-45ba-91d1-117a9b292d69
Ticket: forge#241 (71ad57c2-5b9f-4d86-a26c-5f1d4d5841ef). 4th of M14 (#238 config, #239 d9e9cc1, #240 b748fa8).

## Phase 1 — Plan (discovery inline)

**chad (live, alongside issue #2):** the rail showed several identical "Marley" tabs — indistinguishable.

**Discovery (this session):**
- The rail Tab-row DISPLAYED label is computed INLINE per row: `let label = self.live_tab_title(p, t, &row.label)`
  (app.rs:4345, inside the flat `for row in rail_row_list.into_iter().skip(rail_skip)` loop at 4255).
- `live_tab_title` (app.rs:2784): idle terminal → `display_title(None, None, cwd, fallback)` → the cwd BASENAME
  ("Marley"). So N idle terminals in the same cwd all read "Marley". The static `tab.title` is "terminal N"
  (unique) but sits BEHIND the live cwd tier → the collision is on the LIVE labels (shim-computed).
- No existing disambiguation helper (the "dedup" hits are the #237 editor-surface path).
- Cockpit tabs (Details/Agents/Forge — deduped by open_or_switch_cockpit) + the #240 "Editor" tab are already
  unique per project → a GENERIC per-project disambiguation leaves them untouched (only dups get a suffix).
- **Scroll subtlety:** the render loop SKIPS `rail_skip` rows (#169 scroll). A per-visible-row counter would
  miscount siblings scrolled out of view → the disambiguation MUST run over the FULL list (D3), keyed by (p,t).

**Design (proposed, D1-D4):** a PURE `disambiguate_labels(&[String]) -> Vec<String>` (append " 2"/" 3" to the
2nd+ duplicate, first bare) + a shim pre-pass building a `(project,tab)→label` map over the FULL rail list; the
Tab arm + the #112 filter look up the disambiguated label.

**Driven plan (control):** the persisted layout already holds multiple idle "Marley" terminals → relaunch →
the rail shows "Marley", "Marley 2", "Marley 3".

**ENV:** control granted. Autonomous auto-approved (M14) → run through commit; do not stop; do not push. AWAIT
the inspect critic before Inspect-PASS (the #240 lesson: the critic caught a real MED after I'd declared clean).

## Phase 2 — Design

**Architecture (D1-D4 confirmed).** A pure helper does the suffix logic; the shim batches the live labels and
looks them up. No PTY/IO/forge.

**PURE `titlebar::disambiguate_labels(labels: &[String]) -> Vec<String>`** (beside display_title/rail_tab_title):
```
let mut counts: HashMap<&str, usize> = HashMap::new();
labels.iter().map(|label| {
    let n = *counts.entry(label.as_str()).and_modify(|c| *c += 1).or_insert(1);
    if n == 1 { label.clone() } else { format!("{label} {n}") }
}).collect()
```
First occurrence (n==1) → bare; 2nd → "… 2"; per-LABEL counter (not positional). Uniques untouched.

**SHIM pre-pass (app.rs render, coverage-excluded).** Right after `let rail_row_list = rail_rows(…)` (before the
consuming `.into_iter().skip(rail_skip)` loop), build a scroll-safe map over the FULL list — TWO-PASS filter
(clearest; n small, shim perf non-critical):
```
let mut tab_labels: HashMap<(usize, usize), String> = HashMap::new();
for p in 0..self.shell.projects().len() {
    let entries: Vec<(usize, String)> = rail_row_list.iter()
        .filter(|r| r.level == RailLevel::Tab && r.project == p)
        .map(|r| { let t = r.tab.unwrap_or(0); (t, self.live_tab_title(p, t, &r.label)) })
        .collect();
    let labels: Vec<String> = entries.iter().map(|(_, l)| l.clone()).collect();
    for ((t, _), lbl) in entries.iter().zip(disambiguate_labels(&labels)) {
        tab_labels.insert((p, *t), lbl);
    }
}
```
The `RailLevel::Tab` arm then: `let label = tab_labels.get(&(p, t)).cloned().unwrap_or_else(|| self.live_tab_title(p, t, &row.label));` (fallback keeps it robust). The #112 "Search tabs" filter uses this `label` (unchanged line). rail_row_list is `iter()`-borrowed for the pre-pass, then moved by the loop — order OK.

**File manifest:**
- `crates/marley_app/src/titlebar.rs` — add `pub fn disambiguate_labels` + `#[cfg(test)]` unit tests + import `std::collections::HashMap`.
- `crates/marley_app/src/app.rs` — the pre-pass block after `rail_row_list` is bound; the `RailLevel::Tab` arm's `let label = …` becomes the map lookup; `use crate::titlebar::disambiguate_labels` (or fully-qualify). Coverage-excluded shim.

**Regression Test Plan:**
| AC | Test (titlebar.rs) | Proves |
|----|--------------------|--------|
| REQ-001 | `disambiguate_labels_cases`: `[]`→`[]`; `["a"]`→`["a"]`; `["a","a"]`→`["a","a 2"]`; `["a","b","a","a"]`→`["a","b","a 2","a 3"]`; `["x","x","y","x"]`→`["x","x 2","y","x 3"]` | first bare, 2nd→" 2", increment (3rd→" 3"), per-LABEL counter (interleaved), uniques untouched |
| REQ-002 | shim review (pre-pass over the FULL `rail_row_list`, not skipped) | scroll-safe disambiguation |
| REQ-003 | DRIVEN capture (relaunch, persisted multi-terminal layout) | rail shows "Marley"/"Marley 2"/"Marley 3" |

**Mutation / coverage:** `disambiguate_labels` mutants (RUN `cargo mutants --list -f titlebar.rs` to confirm): the
`+= 1` (killed by the triple → 3rd must be " 3"), `or_insert(1)` value (killed by the bare-first assert — 0 would
suffix the first), the `n == 1` guard `==`→`!=`/`<`/`<=` (killed by bare-first + suffixed-second). The
`format!` string isn't mutated. cov/MSI 100. The app.rs pre-pass is shim (coverage + mutants excluded).

**Risks:** (1) SCROLL-SAFETY — the pre-pass MUST iterate the full rail_row_list, NOT the `skip(rail_skip)` view
(a per-visible counter miscounts); the two-pass filter over `rail_row_list.iter()` is inherently full-list. (2)
the #112 filter still matches (it now filters on the disambiguated label — a search for "marley" still matches
"Marley 2"). (3) uniques (Details/Agents/Forge/Editor) get no suffix — the generic helper leaves n==1 bare. (4)
borrow: `rail_row_list.iter()` for the pre-pass, then `.into_iter()` for the loop — sequential, no conflict.

**Phase 2 status: Design PASS — helper + scroll-safe shim pre-pass; manifest + test matrix set.**

## Phase 3 — Implement

Applied the manifest (5 edits, 2 files):
- **titlebar.rs:** added `use std::collections::HashMap;` + the pure `pub fn disambiguate_labels(&[String]) ->
  Vec<String>` (per-label counter; first bare, 2nd+ get `format!("{label} {n}")`) with a doc comment, placed
  after `rail_tab_title`.
- **app.rs:** imported `disambiguate_labels`; added the `tab_labels: HashMap<(usize,usize), String>` pre-pass
  right after `rail_row_list` is bound (two-pass filter over the FULL list by project → live labels →
  disambiguate → map); the `RailLevel::Tab` arm's `let label = …` now looks up `tab_labels` with a
  `live_tab_title` fallback.
- **Deviations:** none. Borrow OK (`rail_row_list.iter()` + `self.live_tab_title`/`self.shell.projects()` all
  immutable, before the consuming `.into_iter()`). Tests deferred to Phase 4 (the pub fn compiles standalone —
  coverage of `disambiguate_labels` lands with the Phase-4 test matrix).
- **Build:** `cargo fmt` clean; `cargo check --all-targets -p marley` clean; `clippy -D warnings` clean.

**Phase 3 status: Implement PASS — compiles + clippy clean (disambiguate_labels tests at Phase 4).**

## Phase 3.5 — Inspect

1 general-purpose critic (spawned) + self-review. **AWAITING the critic before declaring PASS (the #240
lesson — its critic found a real MED after I'd prematurely declared "no findings").** Self-review so far:

- **[CLEAN, self] disambiguate_labels correctness** — per-label `HashMap<&str,usize>` counter: first occurrence
  `or_insert(1)` → n==1 → bare; subsequent `and_modify(+=1)` → n≥2 → `format!("{label} {n}")`. Interleaved dups
  number in appearance order. `cargo mutants --list` = 6 mutants (body ×3, `+= 1`→`-=`/`*=`, `== 1`→`!=`), all
  killable by the Phase-4 matrix (empty / ["a"] / ["a","a"]→[.."a 2"] / triple→" 3" / interleaved).
- **[CLEAN, self] scroll-safety (load-bearing)** — the pre-pass (app.rs:4243-4260) iterates
  `rail_row_list.iter()` (the FULL list) grouped by `r.project == p`, BEFORE the render loop's
  `.into_iter().skip(rail_skip)` (4277). Key `(p, t)` matches the Tab arm's `(row.project,
  row.tab.unwrap_or(0))` (4364). A scrolled rail numbers correctly.
- **[CLEAN, self] the #112 filter** (app.rs:4373) now matches on the disambiguated `label` — a search for
  "marley" still `contains` "marley 2" (lowercased). Correct.
- **[NOTE, self] rename seed uses the RAW base** — the #177 double-click rename (app.rs:4485) seeds
  `live_tab_title(p, t, &seed_fallback)` = "Marley", NOT the displayed "Marley 2". Deliberate: the " 2" suffix
  is a display-only disambiguator, not the tab's identity — renaming operates on the base name (and once
  renamed to a custom_title, there's no collision → no suffix). A minor seed-vs-display mismatch; leaving as-is
  unless the critic rates it >LOW.
- **[CLEAN, self] borrow/perf** — `rail_row_list.iter()` + `self` immutable in the pre-pass, then `.into_iter()`
  moves it in the loop (sequential, no conflict). O(projects × tabs), small n.
- **Edge case to confirm w/ critic:** a real label already ending " 2" (a dir literally named "Marley 2") +
  two "Marley" tabs → disambiguation yields "Marley 2" colliding with the real one — extremely unlikely,
  cosmetic; assess severity.

**The critic RETURNED (during inspect) and CONCURS with the self-review — one LOW, the rest CLEAN:**
- **[LOW, doc-caveated] {base} {n} collision** — if a sibling is ALREADY literally named "{base} {n}" (a dir
  named "Marley 2", or a command token reading so), disambiguating two "{base}" tabs can re-create that exact
  label (`["Marley","Marley 2","Marley"]` → `["Marley","Marley 2","Marley 2"]`). Cosmetic ONLY — rail click
  routing is by the `(project, tab)` index, never the label, so nothing mis-switches; needs a sibling literally
  named "Marley 2". Per the critic, a note is sufficient — it is DOCUMENTED here + in the forge failure record
  (an in-code fn-doc caveat was phase-gate-BLOCKED because I'd already set Inspect PASS before the critic returned
  — the process cost of setting PASS early; see the lesson below). Deferred an airtight "number against the full
  label set" fix as over-engineering for the likelihood (needs a dir literally named "Marley 2"). A follow-up can
  add the fn-doc caveat / airtight fix if chad wants it. NOT a blocker (cosmetic; click routing unaffected).
- **CONCURRED CLEAN:** algorithm (per-label counter, first bare, no off-by-one); SCROLL-SAFETY (pre-pass over
  the full `rail_row_list.iter()`, matching `(p,t)` keys — BONUS: an active filter doesn't renumber survivors);
  the rename seed from the RAW base is the CORRECT choice (seeding "Marley 2" would freeze a positional counter
  into a stored custom_title); the filter-on-disambiguated is non-regressing; **NO second render path** (grep:
  live_tab_title has exactly 3 callers — pre-pass/render/rename-seed — the center header + titlebar don't render
  tab labels; contrast #240's missed restore path); borrow/perf/clean-room clean. The critic corrected my mutant
  guess: exactly **6** mutants (body ×3, `+= 1`→`-=`/`*=`, `== 1`→`!=`) — there is NO `or_insert(1)→0` mutant —
  all killable by the Phase-4 matrix.
- **PROCESS WIN:** unlike #240, my self-review here was thorough (traced the algorithm + scroll-safety + the
  rename seed + the edge case by reading the code) and the critic CONCURRED — the extra rigor (reading every hop,
  checking the OTHER writers/consumers) caught what a fast pass would miss.

**Phase 3.5 status: Inspect PASS — critic concurred; 1 LOW documented + deferred; all other lenses clean.**

## Phase 4 — Validate

**Tests:** added `titlebar::disambiguate_labels_cases` (the plan's matrix): `[]`→`[]`; `["a"]`→`["a"]`;
`["a","a"]`→`["a","a 2"]`; `["a","b","a","a"]`→`["a","b","a 2","a 3"]`; `["x","x","y","x"]`→`["x","x 2","y","x
3"]`. `cargo nextest run -p marley disambiguate` → **1/1 PASS**. Covers REQ-001 + kills all 6 mutants (body ×3
via the specific expected outputs; `+= 1`→`-=`/`*=` via ["a","a"]; `== 1`→`!=` via ["a"]).

**Driven capture (REQ-002/003, control):** rebuilt (with #241), relaunched (dark) → `241-disambiguated.png`,
READ it. CONFIRMED: the left rail shows **"Marley" · Details · "Marley 2" · Agents · Forge · "Marley 3"** — the 3
idle terminals (persisted, same cwd) are disambiguated; the cockpit tabs (Details/Agents/Forge) are unchanged
(unique → no suffix); still dark. The INTERLEAVED order (terminals at tab positions 0/2/5 numbered 1/2/3 across
the cockpit tabs between them) live-proves the per-project counter + scroll-safe full-list pre-pass exactly as
the interleaved unit case predicts. Chad's ask met.

**Gate:** `git add -A && scripts/gates.sh --diff` → **GATE GREEN [diff] 15/15** — cov 100 + MSI 100 on
titlebar.rs `disambiguate_labels` (6 mutants killed by the matrix); the app.rs pre-pass is shim/coverage-excluded;
clippy/fmt/docs/visual green.

**Phase 4 status: Validate PASS — test 1/1, driven capture confirms "Marley"/"Marley 2"/"Marley 3", gate green.**
