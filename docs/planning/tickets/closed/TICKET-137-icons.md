# TICKET-137 — real icons (replace emoji with a permissive SVG set) [M8 seq-1]

- **Forge ticket:** #137 `7726f8db-4a2b-439d-9b8b-d61b73416d4a` (feature, M8; sprint #19)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `9881ef69-2ebb-4707-9387-587cbc86ecc2`
- **Pipeline doc:** ../../pipeline/active/icons.spec.md
- **Status:** closed

## Summary
Replace the top-bar + cockpit emoji with real themeable monochrome SVG icons: vendored clean-room SVGs + a
gpui `Assets` AssetSource (`with_assets`); pure `Icon` + `icon_path` (cov/MSI 100); `svg()` tinted by theme
color; drop the #134 active bg-pill (svg respects color → active = accent tint). Deps M7.

## Acceptance
icon_path + section_icon at cov/MSI 100; themed SVG icons render (live capture, no emoji); FULL gate GREEN.
Full EARS in the spec.
