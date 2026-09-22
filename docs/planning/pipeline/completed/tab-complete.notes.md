# tab completion at the prompt (Marley-local engine) — Notes

- **Forge ticket:** #89 `c684deaf-5779-4e6c-a53a-023e49778ed3` · **AAR:** `1a8bb125-6624-430b-96eb-7df90d237992`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-089-tab-complete.md

## Phase 1 — Plan
- **Request:** forge #89 (Terminal Polish 2/15) — chad live: "tab completion still doesn't quite work".
- **Classification:** work pipeline, `feature`, a NEW pure module `complete.rs` + an app.rs SHIM.
- **Pre-flight facts:** Tab → `key_input_from_keystroke` "tab"→KeyCode::Tab (raw); the cooked-key dispatch
  is `match keystroke.key.as_str()` at app.rs:448; a literal \t is inserted at the prompt today.
  marley_project::list_files_in is whole-tree (not the immediate dir I need). current_prompt (#37) gives
  the cwd.
- **Decisions:** D1 Marley-local (fork Option A); D2 SPLIT — engine + inline here, popup a follow-up; D3
  Tab intercepted only when !is_command_running; D4 complete.rs cov/MSI 100, shim masked.
- **AAR id:** `1a8bb125-6624-430b-96eb-7df90d237992`.

## Phase 2 — Design

### PURE — `crates/marley_app/src/complete.rs` (char-based, no byte-boundary panics)
```rust
//! PURE — the Marley-local tab-completion engine (#89): the word under the caret, path candidates against
//! a directory listing, and the longest common prefix. The Tab handler (app.rs shim) reads the cwd dir.

/// The whitespace-delimited word ENDING at `caret` (a CHAR offset into `line`): the start char index +
/// the word. Empty (start == caret) when the char before the caret is whitespace or `caret == 0`.
pub fn current_word(line: &str, caret: usize) -> (usize, String) {
    let chars: Vec<char> = line.chars().collect();
    let caret = caret.min(chars.len());
    let mut start = caret;
    while start > 0 && !chars[start - 1].is_whitespace() {
        start -= 1;
    }
    (start, chars[start..caret].iter().collect())
}

/// Candidate completions of `word` against directory `entries` (each a name; the shim pre-suffixes a dir
/// with '/'). Splits `word` on its LAST '/' into (dir_prefix, seg); returns the entries whose name starts
/// with `seg`, re-prefixed with `dir_prefix`, sorted.
pub fn complete_word(word: &str, entries: &[String]) -> Vec<String> {
    let (dir_prefix, seg) = match word.rfind('/') {
        Some(i) => (&word[..=i], &word[i + 1..]),
        None => ("", word),
    };
    let mut out: Vec<String> = entries
        .iter()
        .filter(|e| e.starts_with(seg))
        .map(|e| format!("{dir_prefix}{e}"))
        .collect();
    out.sort();
    out
}

/// The longest shared prefix of `cands` ("" for empty / no shared; the full string for one).
pub fn common_prefix(cands: &[String]) -> String {
    let mut iter = cands.iter();
    let Some(first) = iter.next() else {
        return String::new();
    };
    let mut prefix = first.clone();
    for cand in iter {
        while !cand.starts_with(&prefix) {
            prefix.pop();
            if prefix.is_empty() {
                return String::new();
            }
        }
    }
    prefix
}
```
- `lib.rs`: add `mod complete;`.
- NOTE `rfind('/')` + `word[..=i]`/`word[i+1..]` are BYTE indices on `word` — safe because `/` is ASCII
  (a byte boundary); the SEG match + dir_prefix are byte-safe. `current_word`/`common_prefix` are char-safe.

### SHIM — `app.rs` (mutants::skip): the Tab intercept
- After the raw-route block (~1345 — a running command already streamed Tab to the program, so past here is
  COOKED), before the scroll/history checks:
  ```rust
  // Tab at the COOKED prompt completes a path (#89) — never a literal tab.
  if event.keystroke.key == "tab" && !event.keystroke.modifiers.shift
      && !event.keystroke.modifiers.platform && !event.keystroke.modifiers.control {
      view.complete_at_prompt(cx);
      return;
  }
  ```
- `complete_at_prompt(&mut self, cx)` (mutants::skip): read `(line, caret, cwd)` from the focused state in a
  scoped borrow (cwd = the session's prompt pwd if available, else `std::env::current_dir()`); `current_word`
  → resolve the word's dirname against cwd → `std::fs::read_dir` → names (a dir suffixed '/') →
  `complete_word(&word, &entries)` → decide: `[]`→return; `[one]`→replace the `start..caret` span with it
  (+ a trailing space if it's not a dir); `many`→`common_prefix`, and IF its char-len > the word's char-len,
  replace the span with it (extend), else return (the popup follow-up will list them). Re-borrow the focused
  state for the `buffer.edit` + caret advance; `cx.notify()`.

### File manifest
- ADD `crates/marley_app/src/complete.rs` — the engine + tests.
- MODIFY `crates/marley_app/src/lib.rs` — `mod complete;`.
- MODIFY `crates/marley_app/src/app.rs` — the Tab intercept + `complete_at_prompt` (masked).

### Mutation Targets (pure)
- `current_word`: the back-scan `while start>0 && !whitespace`, the `chars[start..caret]` slice.
- `complete_word`: the `rfind('/')` split, the `starts_with(seg)` filter, the `{dir_prefix}{e}` remap, the sort.
- `common_prefix`: the `starts_with(&prefix)` loop, the `pop()`, the empty/one/none returns.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `current_word_cases` — "cd cra"@6→(3,"cra"); "cd "@3→(3,""); ""@0→(0,""); "a"@1→(0,"a"); mid-word caret; caret>len clamps | unit |
| REQ-002 | `complete_word_cases` — no-dir ("cra",[Cargo,crates/,README]→[crates/]); with-dir ("src/ma",[main.rs,mammoth/,z]→[src/main.rs,src/mammoth/]); none; all; sorted | unit |
| REQ-003 | `common_prefix_cases` — one→full; many-shared→prefix; none-shared→""; empty→"" | unit |
| REQ-004 | `cd cra`+Tab → `cd crates/` | self-test (may be env-blocked) |
| REQ-005 | gate GREEN, cov/MSI 100 complete.rs; shim masked | gate |

Uncoverable: `complete_at_prompt` + the Tab intercept + the dir read — masked (gpui/IO), REQ-004.

### Risks / decisions
- D-2.1 the Tab intercept sits AFTER the raw-route so it's cooked-only (a running command's Tab already
  streamed) — no raw-mode Tab break. D-2.2 the char-based current_word/common_prefix avoid byte-boundary
  panics on non-ASCII; the `/`-split is ASCII-byte-safe. D-2.3 cwd from the session prompt else env cwd
  (a `cd`'d shell's cwd needs the prompt pwd — confirm current_prompt exposes it at implement; fall back to
  env). D-2.4 the popup for the ambiguous many-case is SPLIT to a follow-up (created at P5).

## Phase 3 — Implement
- **Built (complete.rs, NEW):** `current_word` (char-based back-scan) + `complete_word` (last-'/' split,
  starts_with filter, dir-prefix remap, sort) + `common_prefix` (pop-until-shared). `mod complete;` added.
- **Built (app.rs):** import the 3 fns; the Tab intercept after the raw-route (cooked-only, plain Tab);
  `complete_at_prompt` (mutants::skip) — reads (line, caret, cwd[=session prompt.pwd else env cwd]),
  current_word, resolves the word's dir, `std::fs::read_dir` (dirs suffixed '/'), complete_word, then
  single→replace(+space for a file) / many→extend-to-common-prefix / none→no-op; re-borrows the focused
  state for buffer.edit + caret.
- **Deviations:** none. cwd from `current_prompt().and_then(|p| p.pwd)` (PromptInfo.pwd) else env cwd.
- **Verification:** `cargo fmt`; `cargo check -p marley` 0 err; clippy `-D warnings` OK. complete.rs is the
  pure surface (tests P4); the Tab handler + dir read are masked.

## Phase 3.5 — Inspect
- **Method:** 1 adversarial critic spawned (general-purpose, running cargo-mutants on complete.rs) + a
  parallel self-review. The critic ran long (re-reading workspace.rs for the borrow check); its MSI
  computation duplicates the Phase-4 gate's own cargo-mutants step (the authoritative MSI-100 check), so
  the pipeline proceeds — any real critic finding (arriving async) is folded in before commit.
- **Self-review (5 lenses) — no correctness findings:** (a) **char-safety** — current_word is char-indexed
  (`chars: Vec<char>`, `caret.min(len)` clamp, `start` a char index) → no byte panic on multibyte; the
  shim's `CharOffset::from(start)` is char-consistent with `caret.as_usize()`. (b) **byte-split safety** —
  `word.rfind('/')` + `word[..=i]`/`word[i+1..]` are byte ops but `/` is ASCII (a char boundary) → safe.
  (c) **dir-prefix ↔ shim agree** — the shim reads `cwd.join(word[..last_slash])` (the dirname, no slash) →
  entry names → complete_word re-prefixes with `dir_prefix` → the replacement is the whole word-with-
  completed-segment, replacing the `[start..caret]` span. (d) **Tab intercept cooked-only** — placed AFTER
  the `input_route==Raw` return (~1345), so a running command's Tab already streamed; plain-Tab-guarded
  (`!shift/!platform/!control`); `return`s before the `apply_key` literal-tab path; no unwrap (read_dir
  Err→return, current_dir unwrap_or, file_type unwrap_or(false)). (e) **borrow/edit** — scoped read of
  (line,caret,cwd) then re-borrow for the edit; `start ≤ caret` (from current_word); no stale index.
- **Critic verdict (arrived async): PASS.** complete.rs **MSI 100** (21 mutants: 19 caught + 2
  timeout-caught [one a genuine infinite-loop mutant, one a build-lock artifact], 0 MISSED). All 5
  CONFIRMs held (char-safety, dir-prefix↔shim agreement, Tab cooked-only + no-literal-tab + no-panic,
  borrow/edit). 4 LOW findings:
  - LOW-1: `complete_word`'s `.filter`/`.sort`/`format!` have NO mutant (cargo-mutants 27.1.0) → MSI 100
    doesn't force sorting/filtering/prefixing → **the tests MUST assert them explicitly** → DONE (the
    complete_word_cases has unsorted→sorted, none-excluded, with-'/'-prepends). Same for current_word's
    `caret.min` clamp (guarded by the `("cd cra",99)` case).
  - **LOW-2 (FIXED):** `complete_at_prompt`'s `end = CharOffset::from(caret)` used the UNCLAMPED caret →
    a latent edit-panic if the caret>len invariant were ever violated (current_word clamps its copy).
    → clamped `caret = caret.min(line.chars().count())` before current_word + the edit.
  - LOW-3 (accepted): a bare leading-slash word `/ma` lists cwd but renders root-anchored — unusual input
    (absolute paths WITH a dir work); documented, not fixed.
  - LOW-4 (no action): common_prefix's `if is_empty()` is a conceptual equivalent mutant (not generated).
- **Fix applied:** LOW-2 (the caret clamp in the shim).

## Phase 4 — Validate
- **Tests added (complete.rs):** `current_word_cases` (REQ-001), `complete_word_cases` (REQ-002 — incl. the
  MSI-invisible sort/filter/prefix guards), `common_prefix_cases` (REQ-003). All 3 pass.
- **Runs (actual):** `cargo nextest -p marley -E '…'` → 3 passed. (Critic's authoritative complete.rs
  cargo-mutants: MSI 100, 0 missed.)
- **SELF-TEST (UI — REQ-004) — ENV-BLOCKED:** `cd cra`+Tab → `crates/` needs synthetic typing, which is
  degraded this session (per #86/#87). REQ-004's mechanism is verified by the engine tests (cov/MSI 100) +
  the critic's Tab-intercept trace (cooked-only, no literal tab, correct replacement) — the only un-driven
  step is the live keypress.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100% on complete.rs. The Tab handler masked.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(5); forge #89 → done; **popup follow-up = #96** (multi-match overlay). **Terminal Polish 2/4.** Marley-local completion engine (complete.rs cov/MSI 100 — critic PASS, MSI 100, 1 LOW caret-clamp fixed). Self-test typing env-blocked; engine-tested + critic-traced.
