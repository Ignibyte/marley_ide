# M10 — a code tab per file — Notes

- **Forge ticket:** #164 `72e7d16f-9565-415f-87b0-a4b419dcc2d4` · **AAR:** `320ef2f6-95d2-4d51-abcd-b3937f6204ae`

## Phase 1 — Plan
- One-predicate pure change (path-keyed open_or_switch_code) + tests; no shim. Deps #154 (the fn), #161 (cleanup).
- **AAR id:** `320ef2f6-95d2-4d51-abcd-b3937f6204ae`.

## Phase 2 — Design
- Predicate: `position(|t| t.code_view().is_some_and(|cv| cv.path == state.path))`. Matched → `tabs[idx] =
  Tab::code(title, state); active = idx` (refresh + switch). Else → push + activate. Title derivation unchanged.
- Tests: a.rs → b.rs → 2 tabs (b active, a untouched); re-open a.rs (new content) → count still 2, a's tab
  active + refreshed — kills `path==` → `is_some()` and the replace-vs-append swap. Fallback-title case stays.
- Risk: none new — same-shape fn, the doc comment updates to the per-file semantic.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- The predicate → is_some_and(|cv| cv.path == state.path); doc updated to the per-file semantic. fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: adversarial SELF-REVIEW (a 1-line pure predicate change; the mutation gate is the mechanical adversary
— a spawned critic adds nothing beyond gate:5 here, per the spec's phase plan).

- **Path-equality semantics:** PathBuf eq is exact/component-wise. All three callers route through
  open_file_in_viewer with paths from the SAME canonical sources (the file-tree walk, the ⌘P list,
  project_root.join(...)), so a given file always arrives spelled identically → dedup holds. A symlink/../
  alias would dodge dedup and open a second tab — cosmetic (both render the same file), accepted + noted.
- **Same NAME, different dirs** (a/mod.rs vs b/mod.rs): distinct paths → distinct tabs, both titled "mod.rs" —
  correct (they are different files); the ambiguous rail label is out of scope.
- **Refresh semantics:** the matched arm replaces the state (fresh content, scroll reset) and keeps the tab's
  position — #154's reuse semantic, now per-file. Intended.
- **MUTATION TRAP found in MY OWN test plan:** mutating the inner comparison to `true` (any code tab matches)
  would SURVIVE a count-only assertion after open(a),open(b) — the replace keeps the count at 2. The validate
  tests therefore assert count==3 (terminal + a + b) after the two opens AND both paths coexist; the re-open
  case asserts count stays 3 (kills `==`→`!=`, which would append a dup). Recorded so the tests are written
  mutation-first.
- Totality unchanged (no unwrap/index on the new path).

Lenses: eq semantics, caller canonicalization, refresh behavior, mutation-survivability of the planned tests.

## Phase 4 — Validate
- **Tests:** open_or_switch_code_cases extended mutation-first (per the inspect trap): open a.rs → count 2; open b.rs → count 3 with BOTH paths coexisting (kills the any-code-tab/inner-true mutant, which holds the count at 2); re-open a.rs → count stays 3, a.rs tab active + refreshed (kills ==→!=); the "/"-fallback title case retained. 1/1 passes.
- **Self-test:** cpf_reopen.png — TWO code tabs in the rail (audit.toml + complete.md), the re-opened audit.toml ACTIVE (switched to its existing tab, no third) with its content center — REQ-003 pixel-proven.
- **Gate:** GREEN [diff] 15/15 (the path predicate mutation-killed by the mutation-first tests).

## Phase 5 — Complete
- CHANGELOG + app_shell #164 note; forge #164 → done. **M10 3/10.** LESSONS: trace mutants against PLANNED asserts before writing them; the transcript-writer wedge → carry the true phase history on the status line.
