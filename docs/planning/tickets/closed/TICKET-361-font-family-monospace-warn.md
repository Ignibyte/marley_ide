# TICKET-361 — appearance.font_family: warn when the font resolves but is NOT monospace

- **Forge ticket:** #361 `df3f5b6d-ada1-4c8f-ba04-4e9f66953370` (feature, editor/settings/337-followup/344-followup/font)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `8feaaf3b-a4dd-4053-87bc-65cc0f5c40a7`
- **Pipeline doc:** ../../pipeline/active/361-font-family-monospace-warn.spec.md
- **Source ticket:** the follow-up goal `/work 360,361,362,363,364` (a #344 scope-boundary follow-up)
- **Status:** closed

## Summary
#344 warns when `appearance.font_family` does not RESOLVE (gpui silently substitutes a system font). But a font
that DOES resolve yet is PROPORTIONAL (e.g. `"Helvetica"`/`"Arial"`) passes #344's `font_resolves` probe and
applies silently — breaking the terminal grid's column alignment with no warning. This is #344's correct scope
boundary (a *resolution* warning, not a *monospace* check). #361 closes the gap: after the resolve probe confirms
the family exists, an app-side `font_is_monospace` probe compares gpui's per-character advance of a narrow glyph
(`i`) to a wide one (`m`) via `TextSystem::advance`; if they differ beyond a tolerant epsilon the family is
treated like an unresolvable one (apply the built-in mono `TERMINAL_FONT` + a `status_flash` naming the family
+ "not monospace"). The epsilon decision + the extended policy are pure (cov/MSI 100); the metric read is
app-side (like `font_resolves`).

## Acceptance
Booting with a resolvable PROPORTIONAL family applies the built-in mono + flashes "<family> … not monospace"; a
resolvable MONOSPACE family applies silently; the #344 doesn't-resolve path is unchanged.
