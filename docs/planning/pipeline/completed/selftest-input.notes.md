# live synthetic-input self-test (M8) — Notes

- **Forge ticket:** #140 `bccf4e3a-626f-47ee-898b-6905a7c58abb` · **AAR:** `ebc5bdc0-ff91-440a-9fcc-7d32102eaac4`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-140-selftest-input.md

## Phase 1 — Plan
- **Request:** forge #140 (M8 run 4/8) — synthetic input already works (proven); add an AX preflight + docs.
- **KEY FINDING:** AXIsProcessTrusted()==true; drive.swift already posts clickat:/cmd:/type: via CGEvent;
  a driven clickat:0.127,0.018 on the "+" spawned terminal 2 (capture). The "env-blocked" caveat was wrong.
- **Gap:** drive.swift silently no-ops if AX ever missing → false-negative captures. Fix: preflight fail-loud.
- **Decisions:** D1 preflight for a fresh machine; D2 gate-is-test (--fast, no .rs).
- **AAR id:** `ebc5bdc0-ff91-440a-9fcc-7d32102eaac4`.

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
- **drive.swift:** import ApplicationServices (AXIsProcessTrusted); a `check` action → prints AX_TRUSTED/AX_NOT_TRUSTED; a PREFLIGHT before the action loop — if any arg posts events (not find/check/wait:) and !AXIsProcessTrusted() → loud stderr + exit(3).
- **README.md:** add a "Synthetic input works (Accessibility)" section — the recipes (find/focus/clickat:fx,fy/cmd:/cmdshift:/type:/keys), the 1024×768pt window ("+"≈fx0.127,fy0.018), the one-time grant, `check` preflight.
- **Test plan:** gate-is-test (§7) — no .rs; `drive.swift check`→AX_TRUSTED (driven) + a driven "+" click→terminal 2 (capture); the !trusted branch code-reviewed.
- **Risks:** the posting-action set (exclude find/check/wait:); ApplicationServices import.

## Phase 3 — Implement
- **Built (drive.swift):** import ApplicationServices; a `check` action → AX_TRUSTED/AX_NOT_TRUSTED; a PREFLIGHT before the loop — any event-posting arg (not find/check/wait:) with !AXIsProcessTrusted() → loud stderr + exit(3); updated the usage comment (check/clickat/cmd/cmdshift). **(README.md):** the check preflight in Usage + the full recipe list + a "Driving CLICKS works" section (the top-bar icon fractions, the "+"→terminal 2 proof, fail-loud).
- **NOTE:** no .rs — scripts + docs only → --fast gate (gate-is-test §7).
- **Verification:** `drive.swift check`→AX_TRUSTED; `find`→NONE (no window). Compiles.

## Phase 3.5 — Inspect
- **Method:** self-review (a swift preflight + docs).
- **Lenses — no findings:** the preflight computes postsEvents = any arg not in {find,check} and not wait:* → the read-only actions (find/check/wait) never trip it; AXIsProcessTrusted() gate → exit(3) with a clear grant message. `check` prints the trust plainly. No secrets; no .rs; the recipes match the code (clickat/cmd/cmdshift/type). The !trusted branch is code-reviewed (AX cannot be revoked in-session; but check confirms AX_TRUSTED, and the branch is a simple stderr+exit). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Type:** gate-is-test (§7) — no .rs; --fast gate + a driven smoke.
- **Driven smoke (below):** drive.swift check→AX_TRUSTED + a driven "+" click → terminal 2.
- **Result:** check→AX_TRUSTED; a driven clickat:0.127,0.018 on the "+" added another terminal (sidebar terminal 1/2/3 — capture drive140.png). REQ-001/003 PASS; REQ-002 (fail-loud) code-reviewed. 228 tests pass (no regression). **--fast GATE GREEN.**
