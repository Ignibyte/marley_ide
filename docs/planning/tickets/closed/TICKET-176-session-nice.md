# TICKET-176 — M11: persist Files-open + window geometry

- **Forge ticket:** #176 `bfb56704-ae2c-4e93-9271-af8989b4e795` (feature; sprint #22)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `0a8d5340-4085-4382-96d8-08c5fad44dfe`
- **Pipeline doc:** ../../pipeline/active/session-nice.spec.md
- **Status:** closed

## Summary
files.open + window.{x,y,w,h} settings; a pure display-aware bounds sanitizer; the render settle-persist;
run() opens at the saved, sanitized geometry. Deps #163, #168.
