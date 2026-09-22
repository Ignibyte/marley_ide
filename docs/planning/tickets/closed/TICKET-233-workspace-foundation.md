# TICKET-233 — Workspace-model foundation (formalize focus + switch algebra + pin terminology)

- **Forge ticket:** #233 (e29e08e4-e554-474b-93a9-891128b4a370) (feature, M13 sprint #26)
- **Owner:** session c104f25c-5375-4d73-9dc5-5fcf30470018
- **AAR:** 159c2334-5c9e-41bf-8e46-6287ff285e2e
- **Pipeline doc:** ../../pipeline/active/workspace-foundation.spec.md
- **Source ticket:** M13 "The Workspace Cockpit" sprint #26 (the workspace-centric re-arch spine)
- **Status:** closed

## Summary
The foundation of the workspace-centric re-arch (#234-237 depend on it). Discovery found the
multi-workspace container ALREADY exists (#156 multi-project = `Vec<Project>` + an `active` cursor), so
#233 is contained: pin the workspace terminology (Session/Workspace/Tab — VETOABLE, the code rename
DEFERRED because `tabs::Project→Workspace` collides with the existing `marley_project::Project`),
formalize the pure focused-workspace + switch algebra and add **workspace cycling** (a keybinding that
doesn't exist today), and a minimal focus-aware shim. The invasive pieces — the type rename, the
"no workspace focused" empty state, relaxing the never-empties guards — are deferred to the tickets that
need them (a rename pass; #234 launcher).

## Acceptance
A pure workspace next/prev cycle (cov/MSI 100, reusing `next_index`/`prev_index`); a
`next-workspace`/`prev-workspace` keybinding that switches the focused workspace + re-syncs its
Files/⌘P/git/titlebar; the terminology documented as pinned + the rename recorded as deferred; and the
never-empties + always-focused invariants UNCHANGED. Full EARS in the pipeline spec. Terminology is
vetoable at chad's review (nothing renamed → cheap to reshape).
