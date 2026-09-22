# TICKET-060 — command history fuzzy search (cmd-R)

- **Forge ticket:** #60 `9620ea3a-0ade-44c7-acb0-8945b8f64adb` (feature, M2.B seq-2)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `a697bf07-4d48-4ba3-b114-34ed89c9844c`
- **Pipeline doc:** ../../pipeline/active/history-search.spec.md
- **Source ticket:** forge sprint #10 `4c988c99-56be-4434-8017-6909db864935` (M2.B — The Agent Cockpit)
- **Status:** closed

## Summary
cmd-R fuzzy history search. PURE: `CommandHistory::recent()` (most-recent-first, deduped) + a keymap
swap (cmd-r → history, rerun-last → cmd-shift-r) + REUSE `FinderState`. SHIM: a cmd-R overlay ranking
the history via `fuzzy_rank`; Enter inserts the chosen command into the cooked prompt buffer (#59).
cov/MSI 100 on recent()+keymap; the overlay is masked + self-test-verified. Deps #54 + #57 + #59.

## Acceptance
recent() + keymap at cov/MSI 100 (order/dedup; cmd-r=history & cmd-shift-r=rerun-last); the cmd-R overlay
lists ranked history + Enter inserts (self-test capture); FULL gate GREEN. Full EARS in the pipeline spec.
