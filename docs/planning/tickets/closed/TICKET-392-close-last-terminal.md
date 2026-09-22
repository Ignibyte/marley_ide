# TICKET-392 — Close the last terminal: guards dissolve + the empty-workspace state

- **Forge:** #392 `2157791d-4d6b-4207-9322-0c73378c3e4f` (sprint #38 `fc972431`)
- **Type:** feature
- **Milestone:** M27
- **Status:** closed (done 2026-07-22)
- **Depends:** #391 (no-terminal totality) — hard, shipped `d8acc8b`
- **Pipeline:** docs/planning/pipeline/completed/392-close-last-terminal.spec.md

> **RE-SCOPED (chad "split it", 2026-07-23):** the design's D4 sizing call found this is 2-3 slices.
> **#392 shipped as the behaviour-neutral TOTALITY SUBSTRATE** (guards ON; `try_active_tab`/`try_workspace`
> twins make the app total over a zero-tab AND a no-terminal project — test-only reachable). **The
> guard-dissolve + the empty-workspace UI (the visible half of the title below) moved to #395**
> (`51abebc2`), which flips both guards + fills the empty-center hints on top of this substrate. See the
> completed spec/notes for the shipped scope.

## Summary
Dissolve BOTH never-empties guards (chad-locked): `close_tab_refusal` shrinks to IndexOutOfRange-only,
the LastTab/LastTerminal variants are removed, the #387-pinned tests are deliberately rewritten. The
Terminal section may empty (muted header, #385 parity); an all-empty workspace renders an
empty-center hint (NOT the launcher — that stays zero-workspace); empty projects round-trip;
default-seed unchanged; PTY reaper unchanged.

## Headline acceptance
Closing the sole terminal succeeds (empty Terminal header); closing the very last tab succeeds
(empty-center placeholder, no panic); an empty project persists + restores empty; the two guard
variants no longer exist anywhere.
