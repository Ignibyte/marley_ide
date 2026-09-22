# Brand-scrub catalog — UI/style cluster

Scope: `crates/marley_app/src/typography.rs`, `crates/marley_app/src/color.rs`,
`crates/ui_components/src/lib.rs`, `crates/ui_components/src/render/keyboard_shortcut.rs`.

Method: `grep -niwE 'warp|zed'` (whole-word, case-insensitive) each file.

Result: **12 mentions, all "Warp", zero "Zed".** All are in comments / doc-comments —
**no theme-names, no user-facing strings, no identifiers, no test-data values.**
One load-bearing provenance disclaimer flagged **KEEP?** (color.rs:9).

Reword rule applied: describe the behavior/value in Marley's own terms, keep the
technical meaning + ticket numbers, drop the brand.

Catalog columns: `file:line | category | CURRENT | PROPOSED reword`

## `crates/marley_app/src/typography.rs` (7)

| file:line | category | CURRENT | PROPOSED reword |
|---|---|---|---|
| typography.rs:17 | doc-comment | `bar, the top-bar search (#230, Warp-calibrated ~12).` | `bar, the top-bar search (#230, calibrated to ~12 for the nav band).` |
| typography.rs:43 | comment | `// M12.2 #195 (Warp visual parity): calibrated to Warp's density (measured ~13pt terminal / ~11pt` | `// M12.2 #195 (dense-density parity): calibrated to a tight type scale (measured ~13pt terminal / ~11pt` |
| typography.rs:87 | comment (in test) | `size: 13.0, // #195 Warp parity` | `size: 13.0, // #195 dense type scale` |
| typography.rs:94 | comment (in test) | `size: 13.0, // #195 Warp parity` | `size: 13.0, // #195 dense type scale` |
| typography.rs:101 | comment (in test) | `size: 11.0, // #195 Warp parity` | `size: 11.0, // #195 dense type scale` |
| typography.rs:108 | comment (in test) | `size: 12.0, // #230 Warp parity — the sidebar/files/nav band` | `size: 12.0, // #230 — the sidebar/files/nav band` |
| typography.rs:123 | comment (in test) | `// Warp-density tune can't silently shrink it into unreadability (REQ-003). ──` | `// a future density tune can't silently shrink it into unreadability (REQ-003). ──` |

## `crates/marley_app/src/color.rs` (1)

| file:line | category | CURRENT | PROPOSED reword |
|---|---|---|---|
| color.rs:9 | doc-comment — **KEEP?** | `/// Marley's ORIGINAL 16-color ANSI palette (clean-room — not lifted from Warp or any AGPL source):` | `/// Marley's ORIGINAL 16-color ANSI palette (clean-room — independently authored, not lifted from any AGPL-licensed source):` |

> **KEEP? — load-bearing.** This is a deliberate **clean-room provenance / legal disclaimer**
> (echoes MEMORY's "clean-room via manual review" + Gate-16 provenance stance). The named brand
> is what makes the disclaimer specific: it asserts the palette was *not* copied from Warp (AGPL).
> Two options for the owner to choose:
> - **KEEP as-is** — the brand here is a disclaimer *against* the brand, not an homage; scrubbing it
>   arguably weakens the provenance record.
> - **Scrub with the proposed reword** — drops the brand but preserves the AGPL clean-room assertion.

## `crates/ui_components/src/lib.rs` (3)

| file:line | category | CURRENT | PROPOSED reword |
|---|---|---|---|
| lib.rs:99 | comment | `// footer / prompt strip) → a darker Warp-like gray L=0.11 (M13 #231, was 0.155) — still a` | `// footer / prompt strip) → a darker panel gray L=0.11 (M13 #231, was 0.155) — still a` |
| lib.rs:158 | comment (in test) | `// The Warp-matched palette (M1.E #35): the dark bg is a TINTED near-black (sat > 0), not` | `// The tuned dark palette (M1.E #35): the dark bg is a TINTED near-black (sat > 0), not` |
| lib.rs:282 | comment (in test) | `assert_eq!(d.surface, gpui::hsla(0.62, 0.09, 0.11, 1.)); // #231: darker Warp-like panel gray` | `assert_eq!(d.surface, gpui::hsla(0.62, 0.09, 0.11, 1.)); // #231: darker panel gray` |

## `crates/ui_components/src/render/keyboard_shortcut.rs` (1)

| file:line | category | CURRENT | PROPOSED reword |
|---|---|---|---|
| keyboard_shortcut.rs:10 | doc-comment | `/// #222 (Warp parity): each key is a real keycap — a `surface` box with a 1px `border`, rounded` | `/// #222: each key is a real keycap — a `surface` box with a 1px `border`, rounded` |

## Summary

- **12 whole-word mentions total** — all "Warp", **no "Zed"**.
- **Category breakdown:** 3 doc-comment, 9 comment (6 of them inside `#[cfg(test)]` blocks). **0 theme-name, 0 string, 0 identifier, 0 test-data value.**
- **User-facing risk: none** — every mention is source-comment only; no theme name (e.g. no "Warp Dark") and no runtime string carries the brand.
- **KEEP? flags: 1** — `color.rs:9`, the clean-room / AGPL provenance disclaimer (reword offered, owner decision).
- All rewords preserve ticket numbers (#195, #230, #231, #222, M1.E #35, M12.2, M13) and the technical meaning (px sizes, L values, REQ-003).
