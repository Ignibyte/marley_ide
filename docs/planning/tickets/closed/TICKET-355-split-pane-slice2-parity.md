# TICKET-355 — Verify + guard editor-feature parity in a focused editable split pane (slice 2)

- **Forge ticket:** #355 (9ef38bb2-3b70-4c56-b268-1e3288e74a2a) (feature, M15 → ships M22)
- **Owner:** 466e35ad-09f6-4b81-89e7-b7fd16c1e45d
- **AAR:** 09df44b8-981a-4eab-ad46-38df45ef0f4c
- **Pipeline doc:** ../../pipeline/active/355-split-pane-slice2-parity.spec.md
- **Source ticket:** M22 IDE wrap-up train (#306, #295, #356, #357, #355)
- **Status:** closed

## Summary
"Slice 2 — editor-feature parity in a focused editable split pane." Investigation found parity is ALREADY
delivered by #259 (focus-aware `active_editor()`/`active_editor_mut()` + `editor_geom`) + the top-level
focus-aware overlays (find/completion/hover/signature/rename/def/refs/code-action/pickers, none tab-gated) +
#357 (the last tab-gated overlay, the banner) + #356 (unfocused read-only sync). So #355 is a VERIFICATION
ticket: headless drives proving representative slice-2 features (⌘⇧O, ⌘F, fold) operate on the FOCUSED pane;
any gap fixed; parity documented. Full EARS in the pipeline spec.

## Acceptance
Representative slice-2 features (go-to-symbol, find, fold) are proven by headless drive to operate on a focused
editable split pane; parity is documented.
