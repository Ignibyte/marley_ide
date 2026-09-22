# TICKET-395 — The empty workspace: dissolve the never-empties guards + the empty-center UI (the #392 capstone)

- **Forge:** #395 `51abebc2-7d0e-43b9-8161-272fdbce5401` (sprint #38 `fc972431`)
- **Type:** feature
- **Milestone:** M27
- **Status:** closed (done 2026-07-23)
- **Depends:** #392 (the no-terminal + zero-tab totality) — hard; #392 makes the flip crash-safe
- **Pipeline:** active — docs/planning/pipeline/active/395-empty-workspace.spec.md (drafted at promotion, Opus 2026-07-22)

## Summary
The small, VISIBLE capstone of "terminate the last terminal" (the D4-split of #392; chad chose "split it"
2026-07-23). Once #392 makes the app total over the empty states (guards on), this flips them:
- dissolve BOTH guards — `close_tab_refusal` (tabs.rs:115) → IndexOutOfRange-only; remove the
  `TabError::LastTab`/`LastTerminal` variants + the one production consumer (app.rs:7322's
  "can't close the last terminal" status flash; `Err(_) => {}` catches the rest);
- REWRITE the #387-pinned guard tests (`close_tab_refusal_truth_table` R4/R6, `close_tab_section_vocab`,
  `close_last_terminal_guard`) to the new table (in-range close always succeeds; zero-terminal + zero-tab
  reachable) — a deliberate, changelog'd spec change;
- the empty Terminal section header (#385 REQ-004 parity);
- the empty-workspace CENTER placeholder — the render center's `tab_count()==0` branch (#392 renders it
  blank; #395 fills it with muted hints: new terminal ⌘T · open file ⌘P · the section ＋s). NOT the
  launcher (#247 = zero-WORKSPACE; an open-but-empty workspace stays open);
- persistence: an empty project round-trips (on #391's codec); the NEW-project default-seed unchanged;
  the PTY reaper on close unchanged.

## Headline acceptance
Closing the last terminal (and the last tab) succeeds; the workspace stays open showing the empty-center
hints + the empty section headers; an empty project persists + restores empty; the two guard variants no
longer exist. Driven capture of the empty state (the #390 pre-seed technique). chad eyeballs the
empty-workspace UX here.
