# TICKET-347 — Regex REPLACE: capture groups ($1/${name}/$$) + two silent-corruption fixes

- **Forge ticket:** #347 `1df08cb6-9ac1-4cd1-ad0c-820386f268a2` (feature, M22/editor/find, #339 follow-up, regex)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `3b62706e-970a-47e3-82d4-e12f82d7ea2c`
- **Pipeline doc:** ../../pipeline/completed/347-regex-capture-replace.spec.md
- **Source ticket:** the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334` (the #339 split-off)
- **Status:** closed — shipped LOCAL: capture-group regex Replace + the F2/F3 fixes. GATE GREEN [diff] 15/15.

## Summary
The second half of M22 #339, which shipped regex FIND + the `.*`/case chips. Today the find bar's Replace is
literal-only (it works literally and does not claim capture support). #347 adds capture-group REPLACE — `$1`,
`${name}`, `$$` — and fixes two PRE-EXISTING silent-corruption bugs that only become reachable once captures land:
**F2**, Replace-One computes its post-replace caret AND its `efind_resume` cursor from the *template* length rather
than the *inserted (expanded)* text's length (a `${2}_${1}` replacement silently skips following matches); and
**F3**, an empty regex match (`a*`, `^`, `\b`) resurrects the infinite loop the resume mechanism was built to
prevent (the resume cursor never advances past a zero-width match). Both live in a `#[cfg_attr(test,
mutants::skip)]` shim inside the coverage-excluded `app.rs` — the exact blind spot #336's `chars().count()` bug hid
in — so the tests must be negative-smoked headless drives, not literal-replacement fixtures.

## Acceptance
Expand `$1`/`${name}`/`$$` per match (via the `regex` crate's own `Captures::expand`); Replace-All applies every
expansion as ONE undo unit, back-to-front, carrying `restore` (cursor-anchored:false); Replace-One's caret AND
`efind_resume` come from the EXPANDED text's length (a `${2}_${1}` case proves it, a literal case cannot); Replace-One
on a zero-width match advances (no loop). Full EARS criteria live in the pipeline spec.
