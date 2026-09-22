# TICKET-089 — tab completion at the prompt (Marley-local engine)

- **Forge ticket:** #89 `c684deaf-5779-4e6c-a53a-023e49778ed3` (feature, Terminal Polish; BACKLOG)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7` · **AAR:** `1a8bb125-6624-430b-96eb-7df90d237992`
- **Pipeline doc:** ../../pipeline/active/tab-complete.spec.md
- **Status:** closed

## Summary
Tab at the prompt completes a path against the cwd (Marley-local, fork Option A) instead of a literal tab.
#89 = the pure engine (`complete.rs`: current_word / complete_word / common_prefix, cov/MSI 100) + inline
completion (single→replace, many→extend to the common prefix); the multi-candidate POPUP is a follow-up.
Deps #28 (buffer) + #40 (is_command_running) + #55 (marley_project) + #37 (cwd).

## Acceptance
complete.rs at cov/MSI 100; Tab completes a single match (`cd cra`+Tab → `crates/`, self-test/engine);
FULL gate GREEN. Full EARS in the spec.
