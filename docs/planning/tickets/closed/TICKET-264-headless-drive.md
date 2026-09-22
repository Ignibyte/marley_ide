# TICKET-264 — Headless driven testing (gpui VisualTestContext)

- **Forge ticket:** #264 bdae95c9-2b24-448d-a42d-8296fd230c4d (chore, M16)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** 7c42b598-3032-442a-bb76-48ec050727ba
- **Pipeline doc:** ../../pipeline/active/264-headless-drive.spec.md
- **Source ticket:** sprint #29 (forge)
- **Status:** closed

## Summary
Adopt gpui's `test-support` harness (`VisualTestContext` — verified present
in the 0.2.2 dep) for a headless driven-test lane: boot the root view in a
test window, inject real keystrokes, assert state — no unlocked foreground
GUI, no Accessibility permission, no screencapture. Retires the worst
capture hazards for behavioral checks. The PIXEL half (render_to_image) does
NOT exist in gpui 0.2.2 (newer-Zed-tree API) — deferred to a gpui-upgrade
follow-up ticket filed at completion.

## Acceptance
test-support dev-dep only; headless tests run under plain nextest (boot
sanity, ⌘T tab+1, the #265 ⌘D split); config isolation; docs updated; the
pixel follow-up filed. Full EARS in the spec.
