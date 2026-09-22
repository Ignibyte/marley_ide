# M11 #172 — the typing spike — Notes

- **Forge ticket:** #172 `67ddc852-e74a-4e7a-ba71-154c4235d54a` · **AAR:** `58a6f6ef-7a03-4ace-80db-311eb19dcd8c`

## Phase 1 — Plan / Phase 2 — Design (folded)
- The delta hunt: `key()`/`chord()` land (proven again today: tab/down/enter drove #96), `typeText` doesn't.
  Ladder: E1 unicode-attach → E2 real event source → E3 post-to-pid → E4 delays. Success = a typed prompt
  line pixel-proven; fail = the README blocker note.
- **AAR id:** `58a6f6ef-7a03-4ace-80db-311eb19dcd8c`.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- **E0 (baseline retest) SUCCEEDED — zero code change.** `focus "type:echo spike-e0"` rendered the text on
  the prompt (ts_e0_strip.png); `enter` + a second `type:pwd enter` executed BOTH as ✓ blocks with correct
  output (ts_exec_strip.png). The ladder (E1-E4) never ran; typeText ships unmodified.
- Deliverable = documentation: README Gotchas gains the retest note + the bare-verb-names trap; the AX
  memory + forge got the durable rule (PR: retest env boundaries before scoping validation down).

## Inspect (Phase 3.5)
Self-review (a docs-only spike):
- ROOT-CAUSE HONESTY: the M9 failure was never re-reproduced, so the cause is INFERRED (frontmost loss
  between calls / overlay focus — both match the AX memory's documented gotchas), stated as such everywhere.
- REGRESSION RISK: zero (no script/app code touched); the WORKING verbs re-proven in the same captures.
- SIDE FINDING recorded (not scope-crept): blocks already render ↻ run / ⧉ cmd / ⧉ out header affordances —
  #175 must inventory their handlers before adding the right-click menu.
Lenses: honesty-of-inference, regression surface, scope discipline.

## Phase 4 — Validate
- REQ-001: ts_e0_strip.png (the typed prompt line) + ts_exec_strip.png (✓ echo spike-e0 → spike-e0; ✓ pwd → the repo path) — typed, executed, block-rendered. REQ-002: the README recipe/retest note. No .rs in the diff → the static gate set.
- **Gate:** GREEN [diff] 15/15.

## Phase 5 — Complete
- CHANGELOG entry; forge #172 → done; PR-claude-retest-env-boundaries-...-001 recorded. **M11 1/8.** LESSON: E0 = reproduce-the-blocker, always.
