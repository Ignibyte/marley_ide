# TICKET-160 — M9: spawn in the ACTIVE project's root

- **Forge ticket:** #160 `0f7a5507-c378-4b11-a675-e5c68aea8c25` (chore; sprint #21)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `801acd3b-6101-4751-8d99-327348f4b7d6`
- **Pipeline doc:** ../../pipeline/active/project-cwd.spec.md
- **Status:** closed

## Summary
The 3 user-facing spawn sites (⌘T tab, ⌘D/menu split, new-agent split) pass the active project's root to
spawn_session_in — closing the #156 gap. Driven: a 2-project shell, ⌘T in project B, lsof proves B's root.
