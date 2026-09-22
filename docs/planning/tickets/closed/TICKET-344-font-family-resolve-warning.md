# TICKET-344 — appearance.font_family: tell the user when it doesn't resolve

- **Forge ticket:** #344 `0c023fd1-3df7-4a80-a19a-0383421e3899` (feature, M22/editor/settings, #337 follow-up)
- **Owner:** session `99b5bc91-ccc7-4f4d-b5d3-8384f289ba9c`
- **AAR:** `ca25106b-de10-4a34-a5ba-e0a8b2d1b621`
- **Pipeline doc:** ../../pipeline/active/344-font-family-resolve-warning.spec.md
- **Source:** the polish/debt goal `/work 341,342,343,344,346,347,349,358,359,334` (a #337 follow-up)
- **Status:** closed

## Summary

An unresolvable `appearance.font_family` degrades to gpui's SILENT system-font substitute — which may be
proportional, breaking the terminal grid — with nothing to explain why. #344 probes the family at boot; on a
non-resolve it falls back to the built-in mono `TERMINAL_FONT` ("Menlo") EXPLICITLY and sets a `status_flash`
naming the family + the fallback. A pure `resolve_font_family` policy (cov/MSI 100) decides; a `new_in` shim
does the gpui probe.

★ Recon correction: the ticket claimed gpui exposes `TextSystem::font_id(&Font) -> Result` as the probe — it is
actually PRIVATE (text_system.rs:109). The public probe is `all_font_names()` membership OR a
`resolve_font`→`get_font_for_id` round-trip (design picks). Handle: `cx.text_system()` in the boot ctor.

## Acceptance

Booting with a garbage `appearance.font_family` applies the built-in mono (`mono_family() == "Menlo"`, a
non-default observable) and sets a `status_flash` naming the family + fallback; a resolvable family applies
silently; empty → built-in, no flash; the pure policy is total. Full EARS in the pipeline spec. Verified
headlessly (boot-with-garbage-settings, the #337 TempDir idiom) — no live drive (a boot font-flash isn't
eyeball-worthy).
