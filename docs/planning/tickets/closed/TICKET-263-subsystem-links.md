# TICKET-263 — Fix broken Subsystem cross-links in the Warp reference docs

- **Forge ticket:** #263 4aca09ed-0ce3-4435-aa90-a7b2ba7cf271 (chore, M16)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** 60384728-fc0c-4dee-82c3-639549a14091
- **Pipeline doc:** ../../pipeline/active/263-subsystem-links.spec.md
- **Source ticket:** sprint #29 (forge)
- **Status:** closed

## Summary
The 78 `docs/warp_architecture/crates/*.md` Subsystem fields link a
nonexistent `../architecture/` directory; the real home is `../subsystems/`.
Scoped mechanical replace + a link-existence check.

## Acceptance
Zero `../architecture/` refs remain; every relative link in crates/*.md
resolves; diff touches only the path prefix. EARS in the spec.
