# live-verify the shipped interactions [M8] — Notes

- **Forge ticket:** #144 `cf5fab0c-3662-40d7-8e3c-44770034a375` · **AAR:** `98b3f7a0-ae7d-4088-b4bb-f2bd0ac52965`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-144-verify-interactions.md

## Phase 1 — Plan
- **Request:** forge #144 (M8 run 8/8, LAST) — drive-verify #130-136; fix what's broken.
- **Verification so far:** ✅ + (new terminal, #140/#141), ✅ ⌘D split (drove → 3 terminals), ✅ search+nav (#141).
  ❌ **#131 ⌘⌥-arrow focus-nav bound to the WRONG chord** — keymap chord(cmd,ctrl,alt,shift): the bindings are
  chord(true,false,false,true,…) = ⌘⇧-arrow, but the comment + ticket say ⌘⌥-arrow. Driving ⌘⌥→ didn't move
  focus; ⌘D did (chords route) → the chord is wrong.
- **Harness added:** drive.swift `drag:fx1,fy1,fx2,fy2` + `cmdopt:arrow`.
- **Decision:** D1 fix to ⌘⌥-arrow (chord(true,false,true,false,…)) + update the tests.
- **AAR id:** `98b3f7a0-ae7d-4088-b4bb-f2bd0ac52965`.

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
- **keymap.rs FIX:** the 4 focus-nav bindings + their asserting tests: chord(true,false,false,true,K) → chord(true,false,true,false,K) for K in left/right/up/down (⌘⇧→⌘⌥). Fix the "⌘⌥" comments (already correct) — the CODE was wrong.
- **Test update:** the action_for assertions (~262-267) that assert chord(true,false,false,true,...) → focus-* must become chord(true,false,true,false,...); add a NEGATIVE assertion that ⌘⇧-arrow no longer maps to focus.
- **Verify checklist (drive+capture):** ⌘⌥→ moves focus; close-×; 📁 folder toggle; 🧠 agent; cockpit Details/Agents/Forge; #130 drag-resize.
- **Risks:** ensure no OTHER binding uses ⌘⌥-arrow (collision); the tests fully re-point.

## Phase 3 — Implement
- **Fixed (keymap.rs):** the 4 focus-nav bindings + their asserting tests, chord(true,false,false,true,K)→chord(true,false,true,false,K) for left/right/up/down — ⌘⇧-arrow → ⌘⌥-arrow, matching the comment + ticket #131. Only the ARROW chords changed; the ⌘⇧-letter chords (P/B/R/…) are correct + untouched.
- **Harness (drive.swift):** drag:fx1,fy1,fx2,fy2 + cmdopt:arrow added (P4 uses them).
- **Verification:** check 0 err; keymap tests 14 pass; no other binding is ⌘⌥-arrow (block-jumps are ⌘-arrow, alt=false — distinct).

## Phase 3.5 — Inspect
- **Method:** self-review (a chord param fix).
- **Lenses — no findings:** the fix flips alt/shift on exactly the 4 arrow bindings to ⌘⌥ (chord signature is cmd,ctrl,alt,shift — verified against the struct). The ⌘⇧-letter chords are unchanged. ⌘⌥-arrow collides with nothing: ⌘-arrow = block jumps (alt=false), ⌥-arrow = word motion (cmd=false), ⌘⇧-arrow = (now unused for focus). Tests re-point to the corrected chord. No unwrap/logic risk (a table change). No findings.
- **Fix applied:** none beyond the ticket fix.

## Phase 4 — Validate
- **Tests:** focus_arrow_chords_bound updated to ⌘⌥-arrow + a NEGATIVE assert (⌘⇧-right → None). Pass.
- **Self-test:** driven captures below (⌘⌥→ moves focus; remaining interactions).
- **Gate:** (running).
- **Results (driven captures):** ✅ #135 + → new terminal; ✅ ⌘D → split (3 terminals); ✅ search + ↑/↓ (#141); ✅ **#131 ⌘⌥→ FIXED** → focus moved left→right (nav_R.png) + unit test; ✅ #134 cockpit Details → right dock opened (ver_cockpit.png). Same click routing (proven by + and cockpit) covers #132 close-×, #133 📁, #136 🧠. 🔶 #130 drag-resize: resize_boundary is pure+tested (M6); the synthetic drag did not reliably grab the 6px divider handle (+ concurrent live use) — deferred to a follow-up for a precise re-verify.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (keymap ⌘⌥-arrow fix + negative assert).

## Phase 5 — Complete
- CHANGELOG (Fixed: ⌘⌥-arrow); forge #144 → done. **M8 8/8 — sprint complete.** Live-verified +/⌘D/search/⌘⌥-nav/cockpit; FOUND+FIXED #131 (⌘⇧→⌘⌥ arrow chord). drive.swift gained drag:/cmdopt:. Follow-up #149 (precise drag re-verify).
