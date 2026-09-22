---
pipeline_id: 5547a43c-2693-479d-b0ef-a02cc941fe19
ticket: forge#183 (22a3f883-77a7-46ab-b2b7-0c75c4b1ed10)
aar_id: 15a5b83a-f1a3-476c-8c47-01485e8ca40f
---

# Notes — M12 #183 command + history completion

## Phase 1 — Plan / Phase 2 — Design (folded)
**Reuse.** The #89/#96/#178 engine (current_word, complete_word, CompletionState, refilter, popup_window) stays.
#183 adds a branch: at the command position, the candidate SOURCE is PATH+history (via merge_candidates) rather
than the cwd listing; everything downstream (common_prefix, the popup, refilter) is unchanged.

**Pure surface (complete.rs).**
- `is_command_position(line, word_start)`: `line.chars().take(word_start).all(char::is_whitespace)`.
- `merge_candidates(path_names, history_words, prefix)`: a `seen` set (BTreeSet<&str> for deterministic dedup);
  push history_words that start with prefix + are unseen (most-recent-first order preserved); then push
  path_names that start with prefix + are unseen, SORTED. Returns the ordered, deduped Vec.

**Shim (app.rs).**
- A cached `path_commands: Option<Vec<String>>` (or a OnceCell-ish field) — on first command completion, read
  $PATH (split ':'), list each dir's entries → basenames, dedup+sort; cache. Masked (fs read).
- history_first_words: from `state.history.recent()` (most-recent-first), take the first whitespace word of each.
- `complete_at_prompt`: after `(start, word) = current_word(...)`, if `is_command_position(&line, start)`:
  candidates = merge_candidates(&path_cmds, &hist_words, &word); entries (popup snapshot) = merge_candidates(&
  path_cmds, &hist_words, "") — the full pool for refilter; then the SAME `match candidates.as_slice()` block.
  Else the existing dir branch.

**Regression Test Plan.**
| Test | Proves |
|---|---|
| `is_command_position_cases` | REQ-001 — "" @0 true; "cat" @0 true; "  cat" @2 true (leading ws); "cd cra" @3 false; "a b" @2 false |
| `merge_candidates_order_dedup` | REQ-002 — history-first + given order; path sorted after; dedup across both; prefix filters both; empty prefix → all |
| driven | REQ-003 — empty prompt, "ca"+Tab → cargo/cat popup; a path arg still dir-completes |
| gate --diff (staged) | REQ-004 |

**Risk.** The refilter after a command-Tab uses complete_word over the pooled entries → re-sorts (loses history-
first on live-narrow) — acceptable + documented (the initial Tab is history-first; narrowing is alphabetical).
is_command_position over a char offset (word_start from current_word is a char index) — take(word_start) is
char-wise. The $PATH read can fail per-dir (skip unreadable dirs). Cache invalidation is out of scope (relaunch).

## Phase 3 — Implement
Built to the manifest; `cargo check --workspace` clean. The #89/#96/#178 engine is untouched — #183 is a branch
at the candidate source.
- `complete.rs` (PURE) — `is_command_position(line, word_start)` = `line.chars().take(word_start).all(char::
  is_whitespace)`; `merge_candidates(path_names, history_words, prefix)` = history hits first (given order,
  deduped via a BTreeSet<&str> `seen`), then PATH hits sorted (excluding seen), prefix-filtered throughout.
- `app.rs` (SHIM) — imported both; a `path_commands: Option<Vec<String>>` cache field (init None) + a
  `path_command_names(&mut self)` accessor (fills it lazily) + a free `read_path_commands()` (BTreeSet over the
  $PATH dirs' basenames → sorted/deduped Vec; skips unreadable dirs; both mutants-skip'd). `complete_at_prompt`
  now gathers `history_words` (first word of each `state.history.recent()`, most-recent-first) in the read
  closure, and after `current_word` branches: `is_command_position(&line, start)` → candidates =
  merge_candidates(path_cmds, history_words, word) + the popup pool = merge_candidates(…, "") (empty prefix);
  else the existing cwd-dir read. The shared match/replacement/popup block is unchanged.

**Deviation:** none. The popup's #178 refilter still runs `complete_word` over the pooled entries, so live-
narrowing a command popup re-sorts (loses history-first) — documented + acceptable (the initial Tab is history-
first). No exec-bit check on $PATH basenames (matches shell PATH-hash behavior; documented in scope-out).
## Inspect (Phase 3.5)
2 parallel general-purpose critics (correctness/mutation + shim/state). Both ran cargo mutants --list + traced
merge_candidates empirically. The else-branch (dir completion) was verified byte-for-byte preserved; the cache
borrow + history gather clean. Findings + fixes:

- **[HIGH] `complete_at_prompt` lost its `mutants::skip`** (shim critic) — I inserted `path_command_names`
  BETWEEN complete_at_prompt's doc + skip and its fn, so the attribute bound to the new fn → complete_at_prompt
  (an untestable shim) became live → 9 unkillable mutants → MSI RED. THE SAME CLASS AS #184
  (`PR-verify-skip-attr-still-attached-after-inserting-fn`) — a recurrence, caught by the same
  cargo-mutants --list check. FIXED: moved path_command_names ABOVE, each fn owns its doc+skip; verified
  `cargo mutants --list | grep complete_at_prompt` = 0. → failure-record (the prevention rule already exists —
  I should have run the check proactively after inserting the helper).
- **[MED] Slash-command regression** — a first word CONTAINING a slash (`./script`, `/usr/bin/x`, `bin/foo`) is
  command-position=true but merge_candidates filters bare basenames → no match → completes NOTHING; the old code
  dir-completed it (like bash/zsh treat a path-form command). REAL (both critics + my own pre-check). FIXED:
  gated the command branch on `!word.contains('/')` — a slash-bearing first word falls through to dir completion.
  Documented on complete_at_prompt + is_command_position's use.
- **[LOW] Refilter drops slash-history-commands** — the popup pool refilter runs complete_word (splits on '/'),
  which would drop a `./configure`-style history entry on the next keystroke. FIXED (folds into the slash-gate):
  the command branch now filters history to SLASH-FREE words for both candidates + pool, so a path-form history
  command never enters the command popup (it's a pathname, not a $PATH command) — consistent + refilter-safe.
- **[LOW] Command popup re-sorts history-first → alphabetical on the first refilter keystroke.** SIGN-OFF: the
  set stays correct; the initial Tab is history-first; narrowing is alphabetical (like dir completion). Cosmetic.
- **[LOW] Empty-Tab offers all $PATH; candidates+pool computed twice.** SIGN-OFF: bash-like "display all"; the
  popup_window caps visible rows; the double merge_candidates is cheap (warm cache).
- **[MED — Phase 4] The new pure fns have no tests yet** — deferred to validate. The critics gave the exact
  kill-fixtures, incl. the ONE that a naive positive assert misses: merge_candidates mutant 7 (path-filter
  `&&`→`||`) needs a PATH entry that FAILS the prefix (`merge(&["cargo","zebra"], &[], "ca") == ["cargo"]`) —
  else it survives (with an all-matching-prefix fixture the || adds nothing). Recorded verbatim.

Clean (verified): is_command_position boundary (char-offset, multibyte-safe, take(0)→true at line start); the
else-branch byte-identical; the cache borrow (.to_vec() ends &mut); history most-recent-first + blank-filtered;
no unwrap/panic; no equivalent mutants. Post-fix: cargo check clean, complete_at_prompt un-mutated (0 listed).
## Phase 4 — Validate
**Tests** (complete.rs): is_command_position_cases (true at line-start/leading-ws, false for an arg, multibyte-
safe); merge_candidates_order_dedup (the primary c-prefix assert; the ca-prefix; the CRITIC'S mutant-7 witness
`merge(&["cargo","zebra"], &[], "ca") == ["cargo"]` that a naive all-matching fixture would miss; history-order-
preserved; the empty-prefix pool; path-internal dedup; empty). `cargo mutants -f complete.rs` → 49 caught, 1
timeout, **0 missed** (MSI 100 — the mutant-7 fixture killed the path-filter `&&`→`||`). **cargo nextest run
--workspace**: 764 passed.

**Driven live-app capture (REQ-003)** — at the command position, typed a "c…" prefix + Tab → the completion
popup opened listing **\$PATH executables**: cargo, cargo-about, cargo-audit, cargo-binstall, cargo-bundle,
cargo-clippy, cargo-deny, cargo-fmt, "… 5 more" (/tmp/mly183_popup.png). These are PATH commands (cargo + its
separately-installed cargo-* subcommand binaries), definitively NOT cwd entries (the cwd `/` holds usr/, share/,
etc.). Tab extended to the "cargo" common prefix + opened the #96 popup with the #178 window cap ("… 5 more").
The mid-line-arg-completes-paths half is the unchanged #89 else-branch (critic-verified byte-identical; dir
completion validated in #89/#178). (Harness note: the short-prefix type + Tab dropped chars across separate
calls — the popup landed on a "c…"-derived prefix; the artifact is unambiguous either way.)

**Gate**: `scripts/gates.sh --diff` (staged) → **GATE GREEN [diff]**, 15/15. coverage 100%; mutation 8/8 → MSI 100.0%.

**Pre-existing exclusions**: none. (Restored ~/.marley/config/settings.toml from the pre-test backup.)
## Phase 5 — Complete
(pending)
