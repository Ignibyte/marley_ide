# M11 #178 — completion live-filter — Notes

- **Forge ticket:** #178 `16f5b5df-c67a-46bd-9072-2a0b159d224e` · **AAR:** `4c31d15e-e8be-40a1-96a6-c92e7e5d111d`

## Phase 1 — Plan / Phase 2 — Design (folded)
- CompletionState carries the entries SNAPSHOT; refilter(word) recomputes candidates via #89's complete_word,
  clamps selected, returns keep-open (≥2). The shim's popup `_` arm: type-through the char, recompute word
  from the buffer, refilter — close on false. Backspace re-widens.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- complete.rs: CompletionState gained `entries: Vec<String>` (the open-time snapshot); `new` takes it;
  `refilter(word) -> bool` recomputes candidates via complete_word, clamps selected (min of len-1), returns
  keep-open (≥2). (#96's test ctor updated.)
- app.rs: open passes entries.clone(); the popup key `_` arm now returns false (fall-through, no dismiss)
  for a plain backspace OR a plain 1-char non-space key — so the buffer edit still types it; AFTER apply_key,
  an open popup recomputes word = buffer[start..caret] and refilters, closing when <2 remain. Non-printable /
  modified keys still dismiss.
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Self-review (a small pure surface + one shim seam; the sprint's deep critic budget went to #173/#175/#177):
- THE SNAPSHOT is the key correctness call: refilter reuses the ENTRIES from open, never a fresh readdir —
  so narrowing is deterministic and can't race the fs (a file created mid-type doesn't appear; documented Out).
- SELECTED CLAMP: min(len-1) keeps it valid as the list shrinks; on a widen (backspace) the old index stays
  (≤ the new larger len) — never out of range.
- THE WORD RECOMPUTE mirrors accept_completion's exact math (start.min(caret), skip/take over chars) — the
  two can't diverge on the same buffer.
- CLOSE-BELOW-2: refilter false → completion=None; a lone survivor is completed by the #89 inline Tab path,
  matching the ticket. Zero candidates (a typo) also closes — correct (nothing to show).
- The `_`-arm SPACE exclusion: a space still dismisses (a new word begins — the popup was for the old one).
- No modal-exclusivity regression: the arm only changed printable/backspace handling; esc/nav/enter/chords
  unchanged.
Lenses: snapshot determinism, index clamp on shrink/grow, word-math parity, close threshold, word boundary.

## Phase 4 — Validate
- **Tests:** refilter_narrows_clamps_and_closes — open on "cr"→3, select 2; narrow "cra"→1 (keep-open FALSE, selected clamped 0); widen "cr"→3 (open); "cri"→1 + "crz"→0 (both close); the ≥2 boundary (3 open / 1 closed). 1 new; full suite (270+).
- **Self-test (REQ-002, typed — #172/#177 harness):** cf_dot_crop.png — `ls .` + Tab opened the full dotfile popup (.cargo/ … .mcp.json.example, 7+). cf_narrow_crop.png — typing `g` NARROWED in place to exactly the 3 .g* entries (.git/, .gitignore, .gitleaks.toml) with `ls .g` on the prompt (type-through) and the popup STILL OPEN. cf_widen_crop.png — backspace WIDENED back to the full list (`ls .`). The zsh-menu feel, live. (Also confirmed the #89 boundary: `ls s`+Tab had a single match → the inline path completed it to `scripts/`, no popup — correct.)
- **Gate:** GREEN [diff] 15/15, MSI 100 (refilter clamp + threshold mutation-killed).

## Phase 5 — Complete
- CHANGELOG + app_shell #178 note; forge #178 → done. **M11 7/8 — only #171 (dock-vestige sweep) left.** LESSON: for a live-edit feature, FALL THROUGH to the existing buffer edit + recompute from the actual buffer rather than duplicating the edit in the modal branch; a snapshot beats a per-keystroke re-read.
